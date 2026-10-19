use crate::tracker::CompletedSession;
use chrono::{Datelike, Local, NaiveDate, TimeZone};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::time::Duration;

pub struct Logger {
    file: File,
    last_date: Option<NaiveDate>,
}

impl Logger {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            file,
            last_date: None,
        })
    }

    pub fn log_session(
        &mut self,
        session: &CompletedSession,
    ) -> io::Result<()> {
        let mut start = session.started_at.clone();
        let end = session.ended_at.clone();
        let mut allocated = Duration::ZERO;

        while start.date_naive() < end.date_naive() {
            let date = start.date_naive();

            let next_date = date.succ_opt().ok_or_else(|| {
                io::Error::other("Date out of range")
            })?;

            let midnight = Local
                .with_ymd_and_hms(
                    next_date.year(), 
                    next_date.month(), 
                    next_date.day(), 
                0, 0, 0,
                )
                .single()
                .ok_or_else(|| {
                    io::Error::other("Could not determine next midnight")
                })?;
            
            let segment_seconds = midnight
                .signed_duration_since(start)
                .num_seconds()
                .max(0) as u64;

            let segment_duration = Duration::from_secs(segment_seconds);
            allocated += segment_duration;
            
            self.write_entry(
                date,
                &session.app_name,
                start.clone(),
                midnight.clone(),
                segment_duration,
            )?;

            start = midnight;
        }

        let final_duration = if start == session.started_at {
            session.duration
        } else {
            session.duration.saturating_sub(allocated)
        };

        self.write_entry(
            end.date_naive(),
            &session.app_name,
            start,
            end,
            final_duration,
        )
    }

    fn write_entry(
        &mut self,
        date: NaiveDate,
        app_name: &str,
        start: chrono::DateTime<Local>,
        end: chrono::DateTime<Local>,
        duration: Duration,
    ) -> io::Result<()> {
        if self.last_date != Some(date) {
            writeln!(
                self.file,
                "\n{}",
                date.format("%A, %B, %d, %Y")
            )?;

            self.last_date = Some(date);
        }

        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        writeln!(
            self.file,
            "{} - {} | {} | {}h {}m {}s",
            start.format("%H:%M:%S"),
            end.format("%H:%M:%S"),
            app_name,
            hours,
            minutes,
            seconds,
        )?;

        self.file.flush()
    }
}