use log::warn;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use time::Duration;

/// app configuration on disk
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppConfigDisk {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_data_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub job_config_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub job_day_folder_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub work_quota_default: Option<Duration>,
}

/// app configuration used by the app
#[derive(Deserialize, Debug, Clone)]
#[serde(from = "AppConfigDisk")]
pub struct AppConfig {
    pub default_data_path: PathBuf,
    pub job_config_file_name: String,
    pub job_day_folder_format: String,
    pub work_quota_default: Duration,
    /// Output machine-readable JSON instead of human-readable text (set from CLI flag).
    #[serde(skip)]
    pub json: bool,
}

impl From<AppConfigDisk> for AppConfig {
    fn from(disk: AppConfigDisk) -> Self {
        let mut result = AppConfig::default();
        if let Some(v) = disk.default_data_path { result.default_data_path = v; }
        if let Some(v) = disk.job_config_file_name { result.job_config_file_name = v; }
        if let Some(v) = disk.job_day_folder_format { result.job_day_folder_format = v; }
        if let Some(v) = disk.work_quota_default { result.work_quota_default = v; }
        result
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_data_path: env::var("HOME").map_or_else(|e| {
                warn!(
                        "HOME environment variable not set, defaulting to current directory ({e})."
                    );
                PathBuf::from(".timetrax")
            }, |home_env| {
                let mut path = PathBuf::from(home_env);
                path.push(".timetrax");
                path
            }),
            job_config_file_name: "job.json".to_string(),
            job_day_folder_format: "data".to_string(),
            work_quota_default: Duration::hours(8),
            json: false,
        }
    }
}
