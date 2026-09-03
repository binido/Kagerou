use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("migration step {step} failed: {source}")]
    Migration {
        step: usize,
        source: rusqlite::Error,
    },

    #[error(
        "database schema version {found} is not supported (this build supports up to {supported})"
    )]
    UnsupportedSchemaVersion { found: i64, supported: i64 },

    #[error("not found")]
    NotFound,

    #[error("invalid input: {0}")]
    InvalidInput(String),
}
