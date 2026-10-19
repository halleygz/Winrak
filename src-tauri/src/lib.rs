//! Core modules are independently testable without the desktop feature.
pub mod db;
pub mod platform;
pub mod services;
pub mod tracker;

#[cfg(feature = "desktop")]
mod commands;

#[cfg(feature = "desktop")]
pub fn run() {
    use commands::{Runtime, TrackingState, sessions::*};
    use std::{sync::Mutex, thread, time::Duration};
    use tauri::{Emitter, Manager};

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_sessions, get_daily_totals, get_app_totals, get_current_app,
            get_dashboard,
            get_tracking_status, retry_tracking
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let mut import_paths = vec![data_dir.join("log.txt")];
            if let Ok(cwd) = std::env::current_dir() {
                import_paths.push(cwd.join("log.txt"));
                if let Some(parent) = cwd.parent() {
                    import_paths.push(parent.join("log.txt"));
                }
            }
            if let Ok(resources) = app.path().resource_dir() {
                let resource_log = resources.join("log.txt");
                if resource_log.is_file() {
                    import_paths.push(resource_log);
                }
            }
            // Import one physical source only once even when candidate paths alias.
            let mut seen = std::collections::HashSet::new();
            import_paths.retain(|path| seen.insert(std::fs::canonicalize(path).unwrap_or_else(|_| path.clone())));
            let state = TrackingState {
                runtime: Mutex::new(Runtime { service: None, tracking_error: None, import_error: None }),
                db_path: data_dir.join("winrak.db"), import_paths,
            };
            // Keep the UI available when SQLite/import initialization fails.
            let _ = state.initialize();
            app.manage(state);
            let handle = app.handle().clone();
            thread::spawn(move || loop {
                let state = handle.state::<TrackingState>();
                let (completed, focus) = match state.runtime.lock() {
                    Ok(mut runtime) => {
                        let result = runtime.service.as_mut().map(|service| service.poll());
                        let completed = match result {
                            Some(Ok(session)) => session,
                            Some(Err(error)) => {
                                runtime.tracking_error = Some(format!("A completed session could not be saved: {error}. This session may be lost."));
                                None
                            }
                            None => None,
                        };
                        let focus = runtime.service.as_ref().and_then(|s| s.current_app());
                        (completed, focus)
                    }
                    Err(_) => (None, None),
                };
                // Neither event emission nor sleeping holds the service mutex.
                if let Some(session) = completed {
                    let _ = handle.emit("session_completed", session);
                }
                let _ = handle.emit("current_focus", focus);
                thread::sleep(Duration::from_secs(1));
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Winrak desktop shell");
}
