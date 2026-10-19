use active_win_pos_rs::get_active_window;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveApp {
    pub name: String,
}

pub fn get_active_app() -> Option<ActiveApp> {
    let window = get_active_window().ok()?;
    
    let name = window.app_name.trim();

    if name.is_empty() {
        return None;
    }

    Some(ActiveApp { name: name.to_string() })
}