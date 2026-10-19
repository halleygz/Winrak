use super::ActiveApp;

#[cfg(feature = "native-active-window")]
pub fn get_active_app() -> Option<ActiveApp> {
    let window = active_win_pos_rs::get_active_window().ok()?;
    let name = window.app_name.trim();
    if name.is_empty() {
        return None;
    }
    Some(ActiveApp {
        name: name.to_string(),
    })
}

/// Headless builds cannot detect a desktop window.
#[cfg(not(feature = "native-active-window"))]
pub fn get_active_app() -> Option<ActiveApp> {
    None
}
