//! Domain persistence. Get resolves historical rows; lists hide deletions.
pub mod inventory;
pub mod owners;
pub mod patients;
pub mod search;
use rusqlite::{Connection, Transaction, TransactionBehavior};
use serde::{Deserialize, Deserializer, Serialize};
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("invalid {field}: {reason}")]
    Invalid { field: String, reason: String },
    #[error("record not found")]
    NotFound,
    #[error("record conflicts with its current state")]
    Conflict,
    #[error("database operation failed: {0}")]
    Database(rusqlite::Error),
}
pub type Result<T> = std::result::Result<T, RepoError>;
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ListQuery {
    pub include_deleted: bool,
    pub limit: u32,
    pub offset: u32,
}
impl Default for ListQuery {
    fn default() -> Self {
        Self {
            include_deleted: false,
            limit: 100,
            offset: 0,
        }
    }
}
pub(crate) fn invalid(field: &str, reason: &str) -> RepoError {
    RepoError::Invalid {
        field: field.into(),
        reason: reason.into(),
    }
}
pub(crate) fn required(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    Ok(())
}
// Missing fields default to None; explicit null deserializes as Some(None).
pub(crate) fn nullable<'de, D, T>(d: D) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}
pub(crate) fn transaction(conn: &Connection) -> Result<Transaction<'_>> {
    Ok(Transaction::new_unchecked(
        conn,
        TransactionBehavior::Immediate,
    )?)
}
pub(crate) fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}
pub(crate) fn digits(value: Option<&str>) -> Option<String> {
    value
        .map(|s| s.chars().filter(char::is_ascii_digit).collect::<String>())
        .filter(|s| !s.is_empty())
}

impl From<rusqlite::Error> for RepoError {
    fn from(error: rusqlite::Error) -> Self {
        if error.sqlite_error_code() == Some(rusqlite::ErrorCode::ConstraintViolation) {
            Self::Conflict
        } else {
            Self::Database(error)
        }
    }
}

pub mod appointments;
pub mod attachments;
pub mod medications;
pub mod prescriptions;
pub mod reminders;
pub mod vaccinations;
pub mod vaccine_types;
pub mod visit_templates;
pub mod visits;

pub(crate) fn date(field: &str, value: &str) -> Result<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .filter(|d| d.format("%Y-%m-%d").to_string() == value && value.len() == 10)
        .ok_or_else(|| invalid(field, "expected a valid YYYY-MM-DD date"))
}
pub(crate) fn time(field: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        if !chrono::NaiveTime::parse_from_str(value, "%H:%M")
            .is_ok_and(|t| t.format("%H:%M").to_string() == value)
        {
            return Err(invalid(field, "expected a valid HH:MM time"));
        }
    }
    Ok(())
}
pub(crate) fn positive(field: &str, value: Option<f64>) -> Result<()> {
    if value.is_some_and(|v| !v.is_finite() || v <= 0.0) {
        return Err(invalid(field, "must be finite and greater than zero"));
    }
    Ok(())
}
pub(crate) fn positive_days(field: &str, value: Option<i64>) -> Result<()> {
    if value.is_some_and(|v| v <= 0) {
        return Err(invalid(field, "must be greater than zero"));
    }
    Ok(())
}
pub(crate) fn active_patient(conn: &Connection, id: i64) -> Result<crate::model::Patient> {
    let patient = patients::get(conn, id)?;
    if patient.deleted_at.is_some() || owners::get(conn, patient.owner_id)?.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    Ok(patient)
}
pub(crate) fn range(from: &str, to: &str) -> Result<()> {
    date("from", from)?;
    date("to", to)?;
    if from > to {
        return Err(invalid("to", "must be on or after from"));
    }
    Ok(())
}

/// Trims free text; an empty or blank value becomes `None`.
pub(crate) fn clean(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

/// Canonical stored phone: E.164 (`+905551112233`) when the input is a recognizable Turkish
/// number or already international, otherwise digits only (the v1 behaviour). The original
/// text is kept separately in `phone_raw`.
pub(crate) fn phone_value(value: Option<&str>) -> Option<String> {
    use crate::legacy::normalize::{phone, Raw};
    let text = value?.trim();
    if text.is_empty() {
        return None;
    }
    if let Some(e164) = phone(Raw::Text(text)).value {
        return Some(e164);
    }
    let all = digits(Some(text))?;
    if text.starts_with('+') && (8..=15).contains(&all.len()) {
        return Some(format!("+{all}"));
    }
    Some(all)
}

pub mod accounts;
pub mod charges;
pub mod payments;
pub mod services;
