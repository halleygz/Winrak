use super::Error;
use rusqlite::Connection;
use std::{path::Path, time::Duration};

/// The shell supplies a file in Tauri's app-data directory; no cwd fallback.
pub fn open(path: &Path) -> Result<Connection, Error> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut connection = Connection::open(path)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    migrate(&mut connection)?;
    Ok(connection)
}

pub fn migrate(connection: &mut Connection) -> Result<(), Error> {
    let version: i64 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version == 0 {
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("migrations/001_create_sessions.sql"))?;
        transaction.commit()?;
    } else if version != 1 {
        return Err(Error::Time(format!(
            "Unsupported database schema version {version}"
        )));
    }
    Ok(())
}
