use super::{Error, connection};
use crate::tracker::CompletedSession;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, TimeZone};
use rusqlite::{Connection, Row, Transaction, params};
use serde::Serialize;
use std::{path::Path, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub app_name: String,
    /// RFC3339 for native sessions; local ISO datetime (no invented offset) for legacy rows.
    pub started_at: String,
    pub ended_at: String,
    pub duration_seconds: u64,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyAppTotal {
    pub date: String,
    pub app_name: String,
    pub total_seconds: u64,
    /// Number of persisted calendar-day segments, as in the original log.
    pub session_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTotal {
    pub app_name: String,
    pub total_seconds: u64,
    pub session_count: u64,
}

pub struct Repository {
    connection: Connection,
}

impl Repository {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            connection: connection::open(path)?,
        })
    }

    /// Persist all segments atomically, retaining the logger's exact allocation algorithm.
    pub fn save_session(&mut self, session: &CompletedSession) -> Result<usize, Error> {
        let segments = split_session(session)?;
        let transaction = self.connection.transaction()?;
        for segment in &segments {
            insert(&transaction, segment)?;
        }
        transaction.commit()?;
        Ok(segments.len())
    }

    /// Inclusive local calendar-day bounds; None means unbounded.
    pub fn sessions(&self, from: Option<&str>, to: Option<&str>) -> Result<Vec<Session>, Error> {
        validate_range(from, to)?;
        let mut statement = self.connection.prepare(
            "SELECT id, app_name, started_at, ended_at, duration_seconds, date FROM sessions
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR date <= ?2)
             ORDER BY date, started_at, id",
        )?;
        Ok(statement
            .query_map(params![from, to], read_session)?
            .collect::<Result<_, _>>()?)
    }

    pub fn daily_totals(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<DailyAppTotal>, Error> {
        validate_range(from, to)?;
        let mut statement = self.connection.prepare(
            "SELECT date, app_name, SUM(duration_seconds), COUNT(*) FROM sessions
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR date <= ?2)
             GROUP BY date, app_name ORDER BY date, app_name COLLATE BINARY",
        )?;
        Ok(statement
            .query_map(params![from, to], |r| {
                Ok(DailyAppTotal {
                    date: r.get(0)?,
                    app_name: r.get(1)?,
                    total_seconds: r.get(2)?,
                    session_count: r.get(3)?,
                })
            })?
            .collect::<Result<_, _>>()?)
    }

    pub fn app_totals(&self, from: Option<&str>, to: Option<&str>) -> Result<Vec<AppTotal>, Error> {
        validate_range(from, to)?;
        let mut statement = self.connection.prepare(
            "SELECT app_name, SUM(duration_seconds), COUNT(*) FROM sessions
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR date <= ?2)
             GROUP BY app_name ORDER BY SUM(duration_seconds) DESC, app_name COLLATE BINARY",
        )?;
        Ok(statement
            .query_map(params![from, to], |r| {
                Ok(AppTotal {
                    app_name: r.get(0)?,
                    total_seconds: r.get(1)?,
                    session_count: r.get(2)?,
                })
            })?
            .collect::<Result<_, _>>()?)
    }

    /// Import valid lines in one transaction. Missing files are a successful no-op.
    /// Canonical path + physical line number + original bytes identify source rows,
    /// so appending and retrying does not duplicate or conflate identical sessions.
    pub fn import_txt_log(&mut self, path: &Path) -> Result<usize, Error> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e.into()),
        };
        let source = std::fs::canonicalize(path)?.to_string_lossy().into_owned();
        let transaction = self.connection.transaction()?;
        let mut date = None;
        let mut imported = 0;
        for (index, bytes) in bytes.split(|b| *b == b'\n').enumerate() {
            let Ok(raw) = std::str::from_utf8(bytes) else {
                continue;
            };
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            if !line.contains(" | ") {
                // An invalid heading must not leak the preceding day's context.
                if line.contains(',') {
                    date = NaiveDate::parse_from_str(line, "%A, %B, %d, %Y").ok();
                }
                continue;
            }
            let Some(date) = date else { continue };
            let Some(session) = parse_legacy_line(date, line) else {
                continue;
            };
            let line_number = i64::try_from(index + 1).map_err(|_| Error::DurationOverflow)?;
            let exists: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM legacy_import_lines
                 WHERE source_path = ?1 AND line_number = ?2 AND raw_line = ?3)",
                params![source, line_number, raw],
                |r| r.get(0),
            )?;
            if exists {
                continue;
            }
            let id = insert(&transaction, &session)?;
            transaction.execute(
                "INSERT INTO legacy_import_lines(source_path, line_number, raw_line, session_id)
                 VALUES (?1, ?2, ?3, ?4)",
                params![source, line_number, raw, id],
            )?;
            imported += 1;
        }
        transaction.commit()?;
        Ok(imported)
    }
}

fn read_session(row: &Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        app_name: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        duration_seconds: row.get(4)?,
        date: row.get(5)?,
    })
}

fn insert(transaction: &Transaction<'_>, session: &Session) -> Result<i64, Error> {
    let seconds = i64::try_from(session.duration_seconds).map_err(|_| Error::DurationOverflow)?;
    transaction.execute(
        "INSERT INTO sessions(app_name, started_at, ended_at, duration_seconds, date)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            session.app_name,
            session.started_at,
            session.ended_at,
            seconds,
            session.date
        ],
    )?;
    Ok(transaction.last_insert_rowid())
}

fn validate_range(from: Option<&str>, to: Option<&str>) -> Result<(), Error> {
    for value in [from, to].into_iter().flatten() {
        let parsed = NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| Error::InvalidDate(value.into()))?;
        if parsed.format("%Y-%m-%d").to_string() != value {
            return Err(Error::InvalidDate(value.into()));
        }
    }
    if let (Some(from), Some(to)) = (from, to)
        && from > to
    {
        return Err(Error::InvalidRange);
    }
    Ok(())
}

fn split_session(session: &CompletedSession) -> Result<Vec<Session>, Error> {
    let mut start = session.started_at;
    let end = session.ended_at;
    let mut allocated = Duration::ZERO;
    let mut segments = Vec::new();
    while start.date_naive() < end.date_naive() {
        let date = start.date_naive();
        let next_date = date
            .succ_opt()
            .ok_or_else(|| Error::Time("Date out of range".into()))?;
        let midnight = Local
            .with_ymd_and_hms(
                next_date.year(),
                next_date.month(),
                next_date.day(),
                0,
                0,
                0,
            )
            .single()
            .ok_or_else(|| Error::Time("Could not determine next midnight".into()))?;
        let segment_seconds = midnight.signed_duration_since(start).num_seconds().max(0) as u64;
        let segment_duration = Duration::from_secs(segment_seconds);
        allocated += segment_duration;
        segments.push(Session {
            id: 0,
            app_name: session.app_name.clone(),
            started_at: start.to_rfc3339(),
            ended_at: midnight.to_rfc3339(),
            duration_seconds: segment_duration.as_secs(),
            date: date.to_string(),
        });
        start = midnight;
    }
    let final_duration = if start == session.started_at {
        session.duration
    } else {
        session.duration.saturating_sub(allocated)
    };
    segments.push(Session {
        id: 0,
        app_name: session.app_name.clone(),
        started_at: start.to_rfc3339(),
        ended_at: end.to_rfc3339(),
        duration_seconds: final_duration.as_secs(),
        date: end.date_naive().to_string(),
    });
    Ok(segments)
}

fn parse_legacy_line(date: NaiveDate, line: &str) -> Option<Session> {
    // Split from both edges so an application's name may itself contain " | ".
    let (times, rest) = line.split_once(" | ")?;
    let (app, duration) = rest.rsplit_once(" | ")?;
    if app.is_empty() {
        return None;
    }
    let (start, end) = times.split_once(" - ")?;
    let start = NaiveTime::parse_from_str(start, "%H:%M:%S").ok()?;
    let end = NaiveTime::parse_from_str(end, "%H:%M:%S").ok()?;
    let parts: Vec<_> = duration.split_whitespace().collect();
    if parts.len() != 3 {
        return None;
    }
    let hours = parts[0].strip_suffix('h')?.parse::<u64>().ok()?;
    let minutes = parts[1].strip_suffix('m')?.parse::<u64>().ok()?;
    let seconds = parts[2].strip_suffix('s')?.parse::<u64>().ok()?;
    if minutes >= 60 || seconds >= 60 {
        return None;
    }
    let duration_seconds = hours
        .checked_mul(3600)?
        .checked_add(minutes.checked_mul(60)?)?
        .checked_add(seconds)?;
    if duration_seconds > i64::MAX as u64 {
        return None;
    }
    // The log's heading is the segment's starting date. 23:xx -> 00:00 is
    // next midnight, not an end before start. Equal midnight clocks may be a
    // full calendar day (including a 23-hour DST day). The raw text is retained
    // because historical offsets and some equal-clock cases are unknowable.
    let full_day = start == NaiveTime::MIN && end == start && duration_seconds >= 23 * 3600;
    let end_date = if end < start || full_day {
        date.succ_opt()?
    } else {
        date
    };
    Some(Session {
        id: 0,
        app_name: app.into(),
        started_at: date.and_time(start).to_string().replace(' ', "T"),
        ended_at: end_date.and_time(end).to_string().replace(' ', "T"),
        duration_seconds,
        date: date.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn local(day: u32, hour: u32, minute: u32, second: u32) -> chrono::DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 10, day, hour, minute, second)
            .single()
            .unwrap()
    }
    fn completed(
        app: &str,
        start: chrono::DateTime<Local>,
        end: chrono::DateTime<Local>,
        seconds: u64,
    ) -> CompletedSession {
        CompletedSession {
            app_name: app.into(),
            started_at: start,
            ended_at: end,
            duration: Duration::from_secs(seconds),
        }
    }
    fn repo() -> (tempfile::TempDir, Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repository = Repository::open(&dir.path().join("sessions.sqlite")).unwrap();
        (dir, repository)
    }

    #[test]
    fn schema_persistence_and_reopen() {
        let (dir, mut repository) = repo();
        let version: i64 = repository
            .connection
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
        let indexes: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'sessions_%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(indexes, 2);
        repository
            .save_session(&completed("A", local(19, 1, 0, 0), local(19, 1, 0, 5), 7))
            .unwrap();
        drop(repository);
        let repository = Repository::open(&dir.path().join("sessions.sqlite")).unwrap();
        let rows = repository.sessions(None, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].duration_seconds, 7); // Never recompute elapsed from wall time.
        assert_eq!(rows[0].date, "2026-10-19");
        assert_eq!(rows[0].app_name, "A");
        assert!(rows[0].id > 0);
    }

    #[test]
    fn inclusive_ranges_and_stable_aggregates() {
        let (_dir, mut repository) = repo();
        for (app, day, duration) in [("B", 19, 5), ("A", 19, 5), ("A", 20, 7), ("A", 19, 0)] {
            repository
                .save_session(&completed(
                    app,
                    local(day, 1, 0, 0),
                    local(day, 1, 1, 0),
                    duration,
                ))
                .unwrap();
        }
        assert_eq!(
            repository
                .sessions(Some("2026-10-19"), Some("2026-10-19"))
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            repository.sessions(None, Some("2026-10-18")).unwrap().len(),
            0
        );
        assert_eq!(
            repository.sessions(Some("2026-10-20"), None).unwrap().len(),
            1
        );
        let daily = repository.daily_totals(None, None).unwrap();
        assert_eq!(daily.len(), 3);
        assert_eq!(
            (
                daily[0].date.as_str(),
                daily[0].app_name.as_str(),
                daily[0].total_seconds,
                daily[0].session_count
            ),
            ("2026-10-19", "A", 5, 2)
        );
        let totals = repository.app_totals(None, None).unwrap();
        assert_eq!(
            (
                totals[0].app_name.as_str(),
                totals[0].total_seconds,
                totals[0].session_count
            ),
            ("A", 12, 3)
        );
        let tied = repository.app_totals(None, Some("2026-10-19")).unwrap();
        assert_eq!(
            tied.iter().map(|t| t.app_name.as_str()).collect::<Vec<_>>(),
            ["A", "B"]
        );
    }

    #[test]
    fn rejects_invalid_dates_and_reversed_ranges_everywhere() {
        let (_dir, repository) = repo();
        for bad in [
            "2026-2-01",
            "2026-02-30",
            "garbage",
            "2026-10-19T00:00:00",
            "",
        ] {
            assert!(matches!(
                repository.sessions(Some(bad), None),
                Err(Error::InvalidDate(_))
            ));
            assert!(repository.daily_totals(None, Some(bad)).is_err());
            assert!(repository.app_totals(Some(bad), None).is_err());
        }
        assert!(matches!(
            repository.sessions(Some("2026-10-20"), Some("2026-10-19")),
            Err(Error::InvalidRange)
        ));
        assert!(
            repository
                .daily_totals(Some("2026-10-20"), Some("2026-10-19"))
                .is_err()
        );
        assert!(
            repository
                .app_totals(Some("2026-10-20"), Some("2026-10-19"))
                .is_err()
        );
    }

    #[test]
    fn midnight_split_and_saturating_remainder_match_logger() {
        let (_dir, mut repository) = repo();
        let start = local(19, 23, 59, 50);
        let end = local(20, 0, 0, 5);
        assert_eq!(
            repository
                .save_session(&completed("A", start, end, 17))
                .unwrap(),
            2
        );
        let rows = repository.sessions(None, None).unwrap();
        assert_eq!(
            rows.iter().map(|r| r.duration_seconds).collect::<Vec<_>>(),
            [10, 7]
        );
        assert_eq!(rows[0].date, "2026-10-19");
        assert_eq!(rows[1].date, "2026-10-20");
        assert_eq!(rows[0].ended_at, rows[1].started_at);
        let segments = split_session(&completed("A", start, end, 3)).unwrap();
        assert_eq!(
            segments
                .iter()
                .map(|r| r.duration_seconds)
                .collect::<Vec<_>>(),
            [10, 0]
        );
    }

    #[test]
    fn multi_day_and_exact_midnight_include_zero_final_segment() {
        let start = local(19, 23, 59, 50);
        let end = local(21, 0, 0, 0);
        let seconds = end.signed_duration_since(start).num_seconds() as u64;
        let segments = split_session(&completed("A", start, end, seconds)).unwrap();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].duration_seconds, 10);
        assert_eq!(
            segments[1].duration_seconds,
            local(21, 0, 0, 0)
                .signed_duration_since(local(20, 0, 0, 0))
                .num_seconds() as u64
        );
        assert_eq!(segments[2].duration_seconds, 0);
    }

    #[test]
    fn subsecond_midnight_allocation_floors_like_logger() {
        let start = local(19, 23, 59, 59) + chrono::Duration::milliseconds(700);
        let end = local(20, 0, 0, 1);
        let session = CompletedSession {
            app_name: "A".into(),
            started_at: start,
            ended_at: end,
            duration: Duration::from_millis(1800),
        };
        let segments = split_session(&session).unwrap();
        assert_eq!(
            segments
                .iter()
                .map(|s| s.duration_seconds)
                .collect::<Vec<_>>(),
            [0, 1]
        );
    }

    #[test]
    fn overflowing_duration_rolls_back() {
        let (_dir, mut repository) = repo();
        let session = completed("A", local(19, 23, 59, 50), local(20, 0, 0, 5), u64::MAX);
        assert!(matches!(
            repository.save_session(&session),
            Err(Error::DurationOverflow)
        ));
        assert!(repository.sessions(None, None).unwrap().is_empty());
    }

    #[test]
    fn import_skips_malformed_preserves_duplicates_and_is_idempotent_after_reopen() {
        let (dir, mut repository) = repo();
        let path = dir.path().join("log.txt");
        let valid = "08:24:40 - 08:24:45 | A | 0h 0m 5s\n";
        std::fs::write(&path, format!("orphan\n{valid}\nMonday, October, 19, 2026\n{valid}bad row\n{valid}08:00:00 - 08:00:01 | A | wrong\n08:00:00 - 08:00:01 | A | 0h 60m 0s\n08:00:00 - 08:00:01 | A | 18446744073709551615h 0m 0s\n")).unwrap();
        assert_eq!(repository.import_txt_log(&path).unwrap(), 2);
        let rows = repository.sessions(None, None).unwrap();
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0].id, rows[1].id);
        assert_eq!(repository.import_txt_log(&path).unwrap(), 0);
        drop(repository);
        let mut repository = Repository::open(&dir.path().join("sessions.sqlite")).unwrap();
        assert_eq!(repository.import_txt_log(&path).unwrap(), 0);
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        write!(file, "{valid}").unwrap();
        assert_eq!(repository.import_txt_log(&path).unwrap(), 1);
        assert_eq!(repository.sessions(None, None).unwrap().len(), 3);
        assert_eq!(
            repository
                .import_txt_log(&dir.path().join("absent.txt"))
                .unwrap(),
            0
        );
    }

    #[test]
    fn import_midnight_rows_are_already_split_and_keep_source_text() {
        let (dir, mut repository) = repo();
        let path = dir.path().join("log.txt");
        let source = "Monday, October, 19, 2026\n23:59:50 - 00:00:00 | A | 0h 0m 10s\nTuesday, October, 20, 2026\n00:00:00 - 00:00:00 | A | 24h 0m 0s\nWednesday, October, 21, 2026\n00:00:00 - 00:00:00 | A | 0h 0m 0s\n00:00:00 - 00:00:05 | A | title | 0h 0m 7s\n";
        std::fs::write(&path, source).unwrap();
        assert_eq!(repository.import_txt_log(&path).unwrap(), 4);
        let rows = repository.sessions(None, None).unwrap();
        assert_eq!(rows[0].ended_at, "2026-10-20T00:00:00");
        assert_eq!(rows[1].ended_at, "2026-10-21T00:00:00");
        assert_eq!(rows[2].started_at, rows[2].ended_at);
        assert_eq!(rows[3].app_name, "A | title");
        assert_eq!(rows[3].duration_seconds, 7);
        let raw: String = repository
            .connection
            .query_row(
                "SELECT raw_line FROM legacy_import_lines ORDER BY line_number LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(raw, "23:59:50 - 00:00:00 | A | 0h 0m 10s");
        assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    }

    #[test]
    fn invalid_heading_and_non_utf8_lines_are_best_effort() {
        let (dir, mut repository) = repo();
        let path = dir.path().join("log.txt");
        let mut source = b"Monday, October, 19, 2026\n".to_vec();
        source.extend_from_slice(b"\xff\n08:00:00 - 08:00:01 | A | 0h 0m 1s\nBad, heading\n08:00:00 - 08:00:01 | A | 0h 0m 1s\n");
        std::fs::write(&path, source).unwrap();
        assert_eq!(repository.import_txt_log(&path).unwrap(), 1);
    }

    #[test]
    fn import_transaction_rolls_back_rows_and_markers_together() {
        let (dir, mut repository) = repo();
        let path = dir.path().join("log.txt");
        std::fs::write(&path, "Monday, October, 19, 2026\n08:00:00 - 08:00:01 | A | 0h 0m 1s\n08:00:01 - 08:00:02 | B | 0h 0m 1s\n").unwrap();
        repository.connection.execute_batch("CREATE TRIGGER fail_b BEFORE INSERT ON sessions WHEN NEW.app_name = 'B' BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
        assert!(repository.import_txt_log(&path).is_err());
        assert!(repository.sessions(None, None).unwrap().is_empty());
        let markers: i64 = repository
            .connection
            .query_row("SELECT COUNT(*) FROM legacy_import_lines", [], |r| r.get(0))
            .unwrap();
        assert_eq!(markers, 0);
        repository
            .connection
            .execute_batch("DROP TRIGGER fail_b")
            .unwrap();
        assert_eq!(repository.import_txt_log(&path).unwrap(), 2);
    }

    #[test]
    fn local_day_allocation_uses_timezone_midnights_not_fixed_24_hours() {
        // Run with TZ=America/New_York to exercise the 23-hour DST day.
        let start = Local
            .with_ymd_and_hms(2026, 3, 8, 0, 0, 0)
            .single()
            .unwrap();
        let end = Local
            .with_ymd_and_hms(2026, 3, 9, 0, 0, 0)
            .single()
            .unwrap();
        let elapsed = end.signed_duration_since(start).num_seconds() as u64;
        let segments = split_session(&completed("A", start, end, elapsed + 2)).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].duration_seconds, elapsed);
        assert_eq!(segments[1].duration_seconds, 2);
        assert_eq!(segments[0].ended_at, segments[1].started_at);
    }

    #[test]
    fn models_use_camel_case_json() {
        let value = serde_json::to_value(AppTotal {
            app_name: "A".into(),
            total_seconds: 5,
            session_count: 1,
        })
        .unwrap();
        assert_eq!(value["appName"], "A");
        assert_eq!(value["totalSeconds"], 5);
        assert_eq!(value["sessionCount"], 1);
    }
}
