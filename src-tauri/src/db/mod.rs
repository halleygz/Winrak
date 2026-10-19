pub mod connection;
pub mod repository;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid date {0:?}; expected YYYY-MM-DD")]
    InvalidDate(String),
    #[error("The from date must not be after the to date")]
    InvalidRange,
    #[error("{0}")]
    Time(String),
    #[error("Duration exceeds SQLite's integer range")]
    DurationOverflow,
}
