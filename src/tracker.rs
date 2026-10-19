use crate::platform::ActiveApp;
use chrono::{DateTime, Local};
use std::time::{Duration, Instant};

struct ActiveSession{
    app: ActiveApp,
    started_at: DateTime<Local>,
    started_instant: Instant,
}

pub struct CompletedSession {
    pub app_name: String,
    pub started_at: DateTime<Local>,
    pub ended_at: DateTime<Local>,
    pub duration: Duration,
}

pub struct Tracker {
    current: Option<ActiveSession>
}

impl Tracker {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn update(
        &mut self,
        detected_app: Option<ActiveApp>,
    ) -> Option<CompletedSession>{
        let app = detected_app?;

        if let Some(current) = &self.current {
            if current.app == app {
                return None;
            }
        }

        let now = Local::now();
        let mut completed = None;

        if let Some(previous) = self.current.take() {
            completed = Some(CompletedSession {
                app_name: previous.app.name,
                started_at: previous.started_at,
                ended_at: now,
                duration: previous.started_instant.elapsed(),
            });
        }

        self.current = Some(ActiveSession { 
            app, 
            started_at: now, 
            started_instant: Instant::now(),
        });

        completed
    }
}