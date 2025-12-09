// Configuration management for Samwise
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub use_claude_cli: bool,
    pub claude_cli_model: String,
}

impl Default for LLMConfig {
    fn default() -> Self {
        LLMConfig {
            openai_api_key: None,
            anthropic_api_key: None,
            use_claude_cli: true, // Default to CLI if available
            claude_cli_model: "claude-3-5-sonnet-20241022".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LLMConfig,
    pub selected_model: String,
    pub global_hotkey: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            llm: LLMConfig::default(),
            selected_model: "claude-3-5-sonnet".to_string(),
            global_hotkey: "CmdOrCtrl+Shift+Space".to_string(),
        }
    }
}

impl AppConfig {
    fn config_path(app: &AppHandle) -> PathBuf {
        let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
        fs::create_dir_all(&app_data_dir).expect("Failed to create config directory");
        app_data_dir.join("config.json")
    }

    pub fn load(app: &AppHandle) -> Self {
        let config_path = Self::config_path(app);

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => {
                    match serde_json::from_str(&contents) {
                        Ok(config) => config,
                        Err(e) => {
                            eprintln!("Failed to parse config: {}", e);
                            Self::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read config: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), String> {
        let config_path = Self::config_path(app);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> AppConfig {
    AppConfig::load(&app)
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config.save(&app)?;
    Ok(())
}

#[tauri::command]
pub fn check_claude_cli() -> bool {
    // Check if Claude CLI is available
    std::process::Command::new("claude")
        .arg("--version")
        .output()
        .is_ok()
}

