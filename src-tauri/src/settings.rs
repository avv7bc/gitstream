use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 10;

fn default_network_timeout_secs() -> u64 {
    DEFAULT_NETWORK_TIMEOUT_SECS
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppSettings {
    #[serde(default = "default_network_timeout_secs")]
    pub network_timeout_secs: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            network_timeout_secs: DEFAULT_NETWORK_TIMEOUT_SECS,
        }
    }
}

/// Чтение настроек из конкретного файла. Отсутствие файла или битый JSON →
/// дефолты + перезапись валидным дефолтом.
fn read_settings_at(path: &PathBuf) -> AppSettings {
    match fs::read_to_string(path) {
        Ok(raw) => match serde_json::from_str::<AppSettings>(&raw) {
            Ok(s) => s,
            Err(_) => {
                let def = AppSettings::default();
                let _ = write_settings_at(path, &def);
                def
            }
        },
        Err(_) => {
            let def = AppSettings::default();
            let _ = write_settings_at(path, &def);
            def
        }
    }
}

/// Запись настроек в конкретный файл (создаёт родительский каталог).
fn write_settings_at(path: &PathBuf, settings: &AppSettings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app)?;
    Ok(read_settings_at(&path))
}

#[tauri::command]
pub fn set_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app)?;
    write_settings_at(&path, &settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "gitstream_settings_test_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn missing_file_returns_defaults_and_creates_file() {
        let p = temp_path();
        assert!(!p.exists());
        let s = read_settings_at(&p);
        assert_eq!(s, AppSettings::default());
        assert_eq!(s.network_timeout_secs, 10);
        assert!(p.exists(), "file should be created with defaults");
        fs::remove_file(&p).ok();
    }

    #[test]
    fn round_trip_write_then_read() {
        let p = temp_path();
        let s = AppSettings { network_timeout_secs: 42 };
        write_settings_at(&p, &s).unwrap();
        let back = read_settings_at(&p);
        assert_eq!(back, s);
        fs::remove_file(&p).ok();
    }

    #[test]
    fn corrupt_json_returns_defaults_and_rewrites() {
        let p = temp_path();
        fs::write(&p, "{ not json").unwrap();
        let s = read_settings_at(&p);
        assert_eq!(s, AppSettings::default());
        let raw = fs::read_to_string(&p).unwrap();
        let parsed: AppSettings = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed, AppSettings::default());
        fs::remove_file(&p).ok();
    }
}
