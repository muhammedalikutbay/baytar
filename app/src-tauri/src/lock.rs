//! Optional app lock. Only an argon2id PHC hash of the PIN is stored (settings key
//! `lock_pin_hash`); the PIN itself is never stored or logged. The lock keys are not part
//! of the generic settings allow-list in `settings.rs`.
use crate::AppDatabase;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;
use tauri::State;

pub const PIN_HASH_KEY: &str = "lock_pin_hash";
pub const IDLE_KEY: &str = "lock_idle_minutes";
const MAX_IDLE_MINUTES: u32 = 240;
const FREE_ATTEMPTS: u32 = 5;
const BASE_WAIT_MS: u64 = 30_000;
const MAX_WAIT_MS: u64 = 300_000;

/// Failed-attempt tracking with an injectable clock (milliseconds on any monotonic scale).
#[derive(Debug, Default, Clone)]
pub struct Throttle {
    failures: u32,
    blocked_until_ms: u64,
}

impl Throttle {
    pub fn remaining_ms(&self, now_ms: u64) -> u64 {
        self.blocked_until_ms.saturating_sub(now_ms)
    }
    pub fn record_failure(&mut self, now_ms: u64) {
        self.failures = self.failures.saturating_add(1);
        if self.failures >= FREE_ATTEMPTS {
            let doublings = (self.failures - FREE_ATTEMPTS).min(10);
            let wait = (BASE_WAIT_MS << doublings).min(MAX_WAIT_MS);
            self.blocked_until_ms = now_ms + wait;
        }
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

fn wait_seconds(ms: u64) -> u64 {
    ms.div_ceil(1000)
}

static THROTTLE: LazyLock<Mutex<Throttle>> = LazyLock::new(Mutex::default);
static START: LazyLock<Instant> = LazyLock::new(Instant::now);
fn clock_ms() -> u64 {
    START.elapsed().as_millis() as u64
}

#[derive(Debug, PartialEq, Eq)]
pub enum LockError {
    InvalidPin,
    WrongPin,
    NotEnabled,
    InvalidIdle,
    Waiting(u64),
    Internal,
}
impl LockError {
    fn code(&self) -> String {
        match self {
            Self::InvalidPin => "invalid_pin".into(),
            Self::WrongPin => "wrong_pin".into(),
            Self::NotEnabled => "not_enabled".into(),
            Self::InvalidIdle => "invalid_idle".into(),
            Self::Waiting(secs) => format!("wait:{secs}"),
            Self::Internal => "internal".into(),
        }
    }
}

pub fn validate_pin(pin: &str) -> Result<(), LockError> {
    if (4..=8).contains(&pin.len()) && pin.bytes().all(|b| b.is_ascii_digit()) {
        Ok(())
    } else {
        Err(LockError::InvalidPin)
    }
}

pub fn hash_pin(pin: &str) -> Result<String, LockError> {
    validate_pin(pin)?;
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| LockError::Internal)
}

/// Constant-time comparison is done by the argon2 verifier.
pub fn verify_pin(pin: &str, phc: &str) -> bool {
    PasswordHash::new(phc)
        .map(|parsed| {
            Argon2::default()
                .verify_password(pin.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

fn read_setting(conn: &Connection, key: &str) -> Result<Option<String>, LockError> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
        row.get(0)
    })
    .optional()
    .map_err(|_| LockError::Internal)
}
fn write_setting(conn: &Connection, key: &str, value: &str) -> Result<(), LockError> {
    conn.execute(
        "INSERT INTO settings(key, value, updated_at) VALUES (?1, ?2, datetime('now')) ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
        [key, value],
    )
    .map(|_| ())
    .map_err(|_| LockError::Internal)
}
fn delete_setting(conn: &Connection, key: &str) -> Result<(), LockError> {
    conn.execute("DELETE FROM settings WHERE key = ?1", [key])
        .map(|_| ())
        .map_err(|_| LockError::Internal)
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LockStatus {
    pub enabled: bool,
    pub idle_minutes: u32,
    pub wait_seconds: u64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResult {
    pub ok: bool,
    pub wait_seconds: u64,
}

pub fn status_with(
    conn: &Connection,
    throttle: &Throttle,
    now_ms: u64,
) -> Result<LockStatus, LockError> {
    let idle_minutes = read_setting(conn, IDLE_KEY)?
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|minutes| *minutes <= MAX_IDLE_MINUTES)
        .unwrap_or(0);
    Ok(LockStatus {
        enabled: read_setting(conn, PIN_HASH_KEY)?.is_some(),
        idle_minutes,
        wait_seconds: wait_seconds(throttle.remaining_ms(now_ms)),
    })
}

/// Checks the PIN against the stored hash, applying the backoff. While waiting, the hash is
/// not even evaluated so a correct PIN cannot bypass the delay.
pub fn verify_with(
    conn: &Connection,
    throttle: &mut Throttle,
    now_ms: u64,
    pin: &str,
) -> Result<VerifyResult, LockError> {
    let Some(hash) = read_setting(conn, PIN_HASH_KEY)? else {
        return Err(LockError::NotEnabled);
    };
    let remaining = throttle.remaining_ms(now_ms);
    if remaining > 0 {
        return Ok(VerifyResult {
            ok: false,
            wait_seconds: wait_seconds(remaining),
        });
    }
    if verify_pin(pin, &hash) {
        throttle.reset();
        return Ok(VerifyResult {
            ok: true,
            wait_seconds: 0,
        });
    }
    throttle.record_failure(now_ms);
    Ok(VerifyResult {
        ok: false,
        wait_seconds: wait_seconds(throttle.remaining_ms(now_ms)),
    })
}

fn require_pin(
    conn: &Connection,
    throttle: &mut Throttle,
    now_ms: u64,
    pin: &str,
) -> Result<(), LockError> {
    let result = verify_with(conn, throttle, now_ms, pin)?;
    if result.ok {
        Ok(())
    } else if result.wait_seconds > 0 {
        Err(LockError::Waiting(result.wait_seconds))
    } else {
        Err(LockError::WrongPin)
    }
}

pub fn set_pin_with(
    conn: &Connection,
    throttle: &mut Throttle,
    now_ms: u64,
    pin: &str,
    current_pin: Option<&str>,
) -> Result<(), LockError> {
    validate_pin(pin)?;
    if read_setting(conn, PIN_HASH_KEY)?.is_some() {
        require_pin(conn, throttle, now_ms, current_pin.unwrap_or(""))?;
    }
    write_setting(conn, PIN_HASH_KEY, &hash_pin(pin)?)
}

pub fn disable_with(
    conn: &Connection,
    throttle: &mut Throttle,
    now_ms: u64,
    pin: &str,
) -> Result<(), LockError> {
    require_pin(conn, throttle, now_ms, pin)?;
    delete_setting(conn, PIN_HASH_KEY)
}

pub fn set_idle_with(conn: &Connection, minutes: u32) -> Result<(), LockError> {
    if minutes > MAX_IDLE_MINUTES {
        return Err(LockError::InvalidIdle);
    }
    write_setting(conn, IDLE_KEY, &minutes.to_string())
}

fn with_conn<T>(
    state: &State<'_, AppDatabase>,
    f: impl FnOnce(&Connection, &mut Throttle, u64) -> Result<T, LockError>,
) -> Result<T, String> {
    let run = || {
        let guard = state.database.lock().map_err(|_| LockError::Internal)?;
        let db = guard.as_ref().ok_or(LockError::Internal)?;
        let mut throttle = THROTTLE.lock().map_err(|_| LockError::Internal)?;
        f(db.conn(), &mut throttle, clock_ms())
    };
    run().map_err(|error| error.code())
}

#[tauri::command]
pub fn lock_status(state: State<'_, AppDatabase>) -> Result<LockStatus, String> {
    with_conn(&state, |conn, throttle, now| {
        status_with(conn, throttle, now)
    })
}

#[tauri::command]
pub fn lock_set_pin(
    state: State<'_, AppDatabase>,
    pin: String,
    current_pin: Option<String>,
) -> Result<(), String> {
    with_conn(&state, |conn, throttle, now| {
        set_pin_with(conn, throttle, now, &pin, current_pin.as_deref())
    })
}

#[tauri::command]
pub fn lock_disable(state: State<'_, AppDatabase>, pin: String) -> Result<(), String> {
    with_conn(&state, |conn, throttle, now| {
        disable_with(conn, throttle, now, &pin)
    })
}

#[tauri::command]
pub fn lock_verify(state: State<'_, AppDatabase>, pin: String) -> Result<VerifyResult, String> {
    with_conn(&state, |conn, throttle, now| {
        verify_with(conn, throttle, now, &pin)
    })
}

#[tauri::command]
pub fn lock_set_idle(state: State<'_, AppDatabase>, minutes: u32) -> Result<(), String> {
    with_conn(&state, |conn, _, _| set_idle_with(conn, minutes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use baytar_core::db::Database;

    fn db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn pin_length_and_charset_are_validated() {
        for bad in ["", "123", "123456789", "12a4", "12 34", "١٢٣٤"] {
            assert_eq!(validate_pin(bad), Err(LockError::InvalidPin), "{bad}");
        }
        for good in ["1234", "12345678", "0000"] {
            assert!(validate_pin(good).is_ok());
        }
    }

    #[test]
    fn hash_is_argon2id_salted_and_verifies() {
        let a = hash_pin("4321").unwrap();
        let b = hash_pin("4321").unwrap();
        assert!(a.starts_with("$argon2id$"));
        assert_ne!(a, b);
        assert!(!a.contains("4321"));
        assert!(verify_pin("4321", &a));
        assert!(!verify_pin("4322", &a));
        assert!(!verify_pin("4321", "not a phc string"));
    }

    #[test]
    fn backoff_starts_after_five_failures_and_doubles_to_cap() {
        let mut t = Throttle::default();
        for _ in 0..4 {
            t.record_failure(0);
            assert_eq!(t.remaining_ms(0), 0);
        }
        t.record_failure(1_000);
        assert_eq!(t.remaining_ms(1_000), 30_000);
        assert_eq!(t.remaining_ms(31_000), 0);
        t.record_failure(31_000);
        assert_eq!(t.remaining_ms(31_000), 60_000);
        t.record_failure(100_000);
        assert_eq!(t.remaining_ms(100_000), 120_000);
        t.record_failure(300_000);
        assert_eq!(t.remaining_ms(300_000), 240_000);
        t.record_failure(600_000);
        assert_eq!(t.remaining_ms(600_000), 300_000);
        t.record_failure(1_000_000);
        assert_eq!(t.remaining_ms(1_000_000), 300_000);
        t.reset();
        assert_eq!(t.remaining_ms(1_000_000), 0);
    }

    #[test]
    fn verify_flow_with_injected_clock() {
        let database = db();
        let conn = database.conn();
        let mut t = Throttle::default();
        assert_eq!(
            verify_with(conn, &mut t, 0, "1234"),
            Err(LockError::NotEnabled)
        );
        set_pin_with(conn, &mut t, 0, "1234", None).unwrap();
        let hash: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'lock_pin_hash'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(hash.starts_with("$argon2id$") && !hash.contains("1234"));
        assert!(status_with(conn, &t, 0).unwrap().enabled);

        for _ in 0..4 {
            assert_eq!(
                verify_with(conn, &mut t, 0, "9999").unwrap(),
                VerifyResult {
                    ok: false,
                    wait_seconds: 0
                }
            );
        }
        assert_eq!(
            verify_with(conn, &mut t, 0, "9999").unwrap(),
            VerifyResult {
                ok: false,
                wait_seconds: 30
            }
        );
        // A correct PIN during the wait is still refused.
        assert_eq!(
            verify_with(conn, &mut t, 10_000, "1234").unwrap(),
            VerifyResult {
                ok: false,
                wait_seconds: 20
            }
        );
        assert_eq!(status_with(conn, &t, 10_000).unwrap().wait_seconds, 20);
        // After the wait the right PIN works and resets the counter.
        assert!(verify_with(conn, &mut t, 30_000, "1234").unwrap().ok);
        assert_eq!(
            verify_with(conn, &mut t, 30_000, "0000")
                .unwrap()
                .wait_seconds,
            0
        );
    }

    #[test]
    fn change_and_disable_require_the_current_pin() {
        let database = db();
        let conn = database.conn();
        let mut t = Throttle::default();
        set_pin_with(conn, &mut t, 0, "1234", None).unwrap();
        assert_eq!(
            set_pin_with(conn, &mut t, 0, "5678", None),
            Err(LockError::WrongPin)
        );
        assert_eq!(
            set_pin_with(conn, &mut t, 0, "56", Some("1234")),
            Err(LockError::InvalidPin)
        );
        set_pin_with(conn, &mut t, 0, "5678", Some("1234")).unwrap();
        assert_eq!(
            disable_with(conn, &mut t, 0, "1234"),
            Err(LockError::WrongPin)
        );
        disable_with(conn, &mut t, 0, "5678").unwrap();
        assert!(!status_with(conn, &t, 0).unwrap().enabled);
    }

    #[test]
    fn idle_minutes_are_bounded_and_status_reads_them() {
        let database = db();
        let conn = database.conn();
        let t = Throttle::default();
        assert_eq!(status_with(conn, &t, 0).unwrap().idle_minutes, 0);
        set_idle_with(conn, 15).unwrap();
        assert_eq!(status_with(conn, &t, 0).unwrap().idle_minutes, 15);
        set_idle_with(conn, 240).unwrap();
        assert_eq!(set_idle_with(conn, 241), Err(LockError::InvalidIdle));
        set_idle_with(conn, 0).unwrap();
        assert_eq!(status_with(conn, &t, 0).unwrap().idle_minutes, 0);
    }

    #[test]
    fn lock_keys_are_not_exposed_by_the_generic_settings_list() {
        let source = include_str!("settings.rs");
        assert!(!source.contains("lock_pin_hash") && !source.contains("lock_idle_minutes"));
    }
}
