mod active_window;
pub use active_window::get_active_app;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveApp {
    pub name: String,
}
