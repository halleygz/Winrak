use super::{TrackingState, TrackingStatus};
use crate::db::repository::{AppTotal, DailyAppTotal, Session};
use tauri::State;

#[tauri::command]
pub fn get_sessions(
    state: State<'_, TrackingState>,
    from: Option<String>,
    to: Option<String>,
) -> Result<Vec<Session>, String> {
    let runtime = state.runtime.lock().map_err(|e| e.to_string())?;
    runtime
        .service
        .as_ref()
        .ok_or("Database is unavailable. Retry tracking.")?
        .sessions(from.as_deref(), to.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_daily_totals(
    state: State<'_, TrackingState>,
    from: Option<String>,
    to: Option<String>,
) -> Result<Vec<DailyAppTotal>, String> {
    let runtime = state.runtime.lock().map_err(|e| e.to_string())?;
    runtime
        .service
        .as_ref()
        .ok_or("Database is unavailable. Retry tracking.")?
        .daily_totals(from.as_deref(), to.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_totals(
    state: State<'_, TrackingState>,
    from: Option<String>,
    to: Option<String>,
) -> Result<Vec<AppTotal>, String> {
    let runtime = state.runtime.lock().map_err(|e| e.to_string())?;
    runtime
        .service
        .as_ref()
        .ok_or("Database is unavailable. Retry tracking.")?
        .app_totals(from.as_deref(), to.as_deref())
        .map_err(|e| e.to_string())
}

/// One lock keeps summary, chart and history queries on the same persisted state.
#[derive(serde::Serialize)]
pub struct DashboardData {
    sessions: Vec<Session>,
    daily: Vec<DailyAppTotal>,
    apps: Vec<AppTotal>,
}

#[tauri::command]
pub fn get_dashboard(
    state: State<'_, TrackingState>,
    from: Option<String>,
    to: Option<String>,
) -> Result<DashboardData, String> {
    let runtime = state.runtime.lock().map_err(|e| e.to_string())?;
    let service = runtime
        .service
        .as_ref()
        .ok_or("Database is unavailable. Retry tracking.")?;
    Ok(DashboardData {
        sessions: service
            .sessions(from.as_deref(), to.as_deref())
            .map_err(|e| e.to_string())?,
        daily: service
            .daily_totals(from.as_deref(), to.as_deref())
            .map_err(|e| e.to_string())?,
        apps: service
            .app_totals(from.as_deref(), to.as_deref())
            .map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub fn get_current_app(state: State<'_, TrackingState>) -> Result<Option<String>, String> {
    let runtime = state.runtime.lock().map_err(|e| e.to_string())?;
    Ok(runtime
        .service
        .as_ref()
        .and_then(|service| service.current_app()))
}

#[tauri::command]
pub fn get_tracking_status(state: State<'_, TrackingState>) -> Result<TrackingStatus, String> {
    let runtime = state.runtime.lock().map_err(|e| e.to_string())?;
    Ok(TrackingStatus {
        current_app: runtime.service.as_ref().and_then(|s| s.current_app()),
        tracking_error: runtime.tracking_error.clone(),
        import_error: runtime.import_error.clone(),
        database_path: state.db_path.display().to_string(),
    })
}

#[tauri::command]
pub fn retry_tracking(state: State<'_, TrackingState>) -> Result<(), String> {
    state.initialize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Runtime;
    use std::sync::Mutex;

    #[test]
    fn initialization_import_is_repeatable_and_retains_log() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("log.txt");
        let text = "Monday, October, 19, 2026\n08:00:00 - 08:00:05 | Editor | 0h 0m 5s\n";
        std::fs::write(&log, text).unwrap();
        let state = TrackingState {
            runtime: Mutex::new(Runtime {
                service: None,
                tracking_error: None,
                import_error: None,
            }),
            db_path: dir.path().join("winrak.db"),
            import_paths: vec![log.clone()],
        };
        state.initialize().unwrap();
        state.initialize().unwrap();
        let runtime = state.runtime.lock().unwrap();
        let rows = runtime
            .service
            .as_ref()
            .unwrap()
            .sessions(Some("2026-10-19"), Some("2026-10-19"))
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].duration_seconds, 5);
        assert_eq!(std::fs::read_to_string(log).unwrap(), text);
    }
}
