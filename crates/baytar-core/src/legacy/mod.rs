//! Lossless import of legacy PatientsCare data.

pub mod import;
pub mod inspect;
pub mod normalize;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LegacyError {
    #[error("legacy database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("invalid database path")]
    InvalidPath,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error(
        "the file is not a PatientsCare database: it is not a SQLite file, or it has none of the \
         PatientOwner, Patient and ToDo tables"
    )]
    NotLegacyDatabase,
    #[error("the backup taken before the import failed, nothing was imported: {0}")]
    Backup(#[from] crate::backup::BackupError),
    #[error("integrity check after the import failed, nothing was imported: {0}")]
    Integrity(#[from] crate::db::DbError),
    #[error("could not serialize the import report: {0}")]
    Report(#[from] serde_json::Error),
    #[error("inconsistent import plan: {0}")]
    Inconsistent(String),
}
