//! Owner and patient persistence. Get resolves historical rows; lists hide deletions.
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
