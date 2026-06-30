// Configuration management for Samwise
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// The text backend to use. Both are local command-line tools.
fn default_backend() -> String {
    "claude".to_string()
}

// Model passed to the Claude CLI's --model flag. "haiku" is the fast option,
// which is plenty for these short text edits. Use "sonnet" or "opus" for more
// quality, or a full model id.
fn default_claude_model() -> String {
    "haiku".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Which CLI tool runs the text: "claude" or "codex".
    #[serde(default = "default_backend")]
    pub backend: String,
    // Model for the Claude CLI (e.g. "haiku", "sonnet", "opus").
    #[serde(default = "default_claude_model")]
    pub claude_model: String,
    // Model for the Codex CLI. Empty means "use Codex's own default".
    #[serde(default)]
    pub codex_model: String,
    pub global_hotkey: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            backend: default_backend(),
            claude_model: default_claude_model(),
            codex_model: String::new(),
            // Use Super+Alt+S as default - reliable and usually free on most systems
            global_hotkey: "Super+Alt+S".to_string(),
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
                    match serde_json::from_str::<AppConfig>(&contents) {
                        Ok(mut config) => {
                            // Migrate old default hotkeys to new default
                            let old_defaults = vec![
                                "CmdOrCtrl+Shift+Space",
                                "Ctrl+Shift+Space",
                                "Super+Space",
                                "Ctrl+Alt+S",
                            ];
                            if old_defaults.contains(&config.global_hotkey.as_str()) {
                                println!("Migrating hotkey from '{}' to 'Super+Alt+S'", config.global_hotkey);
                                config.global_hotkey = "Super+Alt+S".to_string();
                                // Save the migrated config
                                if let Err(e) = config.save(app) {
                                    eprintln!("Failed to save migrated config: {}", e);
                                }
                            }
                            // Old configs may have a model name here. Map anything
                            // that isn't "codex" back to the "claude" backend.
                            if config.backend != "claude" && config.backend != "codex" {
                                config.backend = default_backend();
                            }
                            config
                        }
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
    cli_available("claude")
}

#[tauri::command]
pub fn check_codex_cli() -> bool {
    cli_available("codex")
}

// True if the given command exists and runs `--version` without error.
fn cli_available(command: &str) -> bool {
    std::process::Command::new(command)
        .arg("--version")
        .output()
        .is_ok()
}
