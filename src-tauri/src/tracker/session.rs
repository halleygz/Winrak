use crate::platform::ActiveApp;
use chrono::{DateTime, Local};
use std::time::{Duration, Instant};

pub(super) struct ActiveSession {
    pub app: ActiveApp,
    pub started_at: DateTime<Local>,
    pub started_instant: Instant,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedSession {
    pub app_name: String,
    pub started_at: DateTime<Local>,
    pub ended_at: DateTime<Local>,
    pub duration: Duration,
}
