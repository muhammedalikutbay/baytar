//! Lossless import of legacy PatientsCare data.

pub mod inspect;
pub mod normalize;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LegacyError {
    #[error("legacy database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("invalid database path")]
    InvalidPath,
}
