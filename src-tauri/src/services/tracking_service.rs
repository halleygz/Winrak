pub use crate::db::Error;
use crate::{
    db::repository::{AppTotal, DailyAppTotal, Repository, Session},
    platform::get_active_app,
    tracker::{CompletedSession, Tracker},
};
use std::path::Path;

/// Owned by the shell behind a mutex; the shell schedules one poll per second.
pub struct TrackingService {
    tracker: Tracker,
    repository: Repository,
}

impl TrackingService {
    pub fn open(db_path: &Path) -> Result<Self, Error> {
        Ok(Self {
            tracker: Tracker::new(),
            repository: Repository::open(db_path)?,
        })
    }

    pub fn poll(&mut self) -> Result<Option<CompletedSession>, Error> {
        let completed = self.tracker.update(get_active_app());
        if let Some(session) = &completed {
            self.repository.save_session(session)?;
        }
        Ok(completed)
    }

    pub fn current_app(&self) -> Option<String> {
        self.tracker.current_app()
    }

    pub fn sessions(&self, from: Option<&str>, to: Option<&str>) -> Result<Vec<Session>, Error> {
        self.repository.sessions(from, to)
    }

    pub fn daily_totals(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<DailyAppTotal>, Error> {
        self.repository.daily_totals(from, to)
    }

    pub fn app_totals(&self, from: Option<&str>, to: Option<&str>) -> Result<Vec<AppTotal>, Error> {
        self.repository.app_totals(from, to)
    }

    pub fn import_txt_log(&mut self, path: &Path) -> Result<usize, Error> {
        self.repository.import_txt_log(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn service_opens_in_supplied_directory() {
        let dir = tempfile::tempdir().unwrap();
        let service = TrackingService::open(&dir.path().join("nested/sessions.sqlite")).unwrap();
        assert_eq!(service.current_app(), None);
        assert!(service.sessions(None, None).unwrap().is_empty());
        assert!(dir.path().join("nested/sessions.sqlite").exists());
    }

    #[cfg(not(feature = "native-active-window"))]
    #[test]
    fn headless_poll_has_no_session() {
        let dir = tempfile::tempdir().unwrap();
        let mut service = TrackingService::open(&dir.path().join("sessions.sqlite")).unwrap();
        assert!(service.poll().unwrap().is_none());
        assert_eq!(service.current_app(), None);
        assert!(service.sessions(None, None).unwrap().is_empty());
    }
}
