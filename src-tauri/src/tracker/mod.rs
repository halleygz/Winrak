mod session;
use crate::platform::ActiveApp;
use chrono::Local;
use session::ActiveSession;
pub use session::CompletedSession;
use std::time::Instant;

pub struct Tracker {
    current: Option<ActiveSession>,
}

impl Tracker {
    pub fn current_app(&self) -> Option<String> {
        self.current
            .as_ref()
            .map(|session| session.app.name.clone())
    }

    pub fn new() -> Self {
        Self { current: None }
    }

    // Keep the original transition body verbatim.
    #[allow(clippy::collapsible_if)]
    pub fn update(&mut self, detected_app: Option<ActiveApp>) -> Option<CompletedSession> {
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
impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    fn app(name: &str) -> Option<ActiveApp> {
        Some(ActiveApp { name: name.into() })
    }

    #[test]
    fn transitions_and_missing_detection_preserve_session() {
        let mut tracker = Tracker::new();
        assert!(tracker.update(None).is_none());
        assert!(tracker.update(app("A")).is_none());
        let started = tracker.current.as_ref().unwrap().started_instant;
        assert!(tracker.update(app("A")).is_none());
        assert!(tracker.update(None).is_none());
        assert_eq!(tracker.current.as_ref().unwrap().started_instant, started);
        assert_eq!(tracker.current_app().as_deref(), Some("A"));
        let completed = tracker.update(app("B")).unwrap();
        assert_eq!(completed.app_name, "A");
        assert_eq!(tracker.current_app().as_deref(), Some("B"));
        assert_eq!(
            completed.ended_at,
            tracker.current.as_ref().unwrap().started_at
        );
    }

    #[test]
    fn duration_uses_instant_not_wall_clock() {
        let mut tracker = Tracker::new();
        tracker.update(app("A"));
        let current = tracker.current.as_mut().unwrap();
        current.started_at = Local::now() - chrono::Duration::days(1);
        current.started_instant = Instant::now() - Duration::from_secs(3);
        let completed = tracker.update(app("B")).unwrap();
        assert!(completed.duration >= Duration::from_secs(3));
        assert!(completed.duration < Duration::from_secs(10));
    }
}
