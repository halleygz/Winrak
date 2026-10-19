pub mod sessions;

use crate::services::tracking_service::TrackingService;
use serde::Serialize;
use std::{path::PathBuf, sync::Mutex};

pub struct Runtime {
    pub service: Option<TrackingService>,
    pub tracking_error: Option<String>,
    pub import_error: Option<String>,
}

pub struct TrackingState {
    pub runtime: Mutex<Runtime>,
    pub db_path: PathBuf,
    pub import_paths: Vec<PathBuf>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingStatus {
    pub current_app: Option<String>,
    pub tracking_error: Option<String>,
    pub import_error: Option<String>,
    pub database_path: String,
}

impl TrackingState {
    pub fn initialize(&self) -> Result<(), String> {
        let mut runtime = self.runtime.lock().map_err(|e| e.to_string())?;
        if runtime.service.is_none() {
            match TrackingService::open(&self.db_path) {
                Ok(service) => runtime.service = Some(service),
                Err(error) => {
                    runtime.tracking_error = Some(error.to_string());
                    return Err(error.to_string());
                }
            }
        }
        runtime.tracking_error = None;
        runtime.import_error = None;
        for path in &self.import_paths {
            if let Err(error) = runtime.service.as_mut().unwrap().import_txt_log(path) {
                let message = format!("{}: {error}", path.display());
                runtime.import_error = Some(message);
                break;
            }
        }
        Ok(())
    }
}
