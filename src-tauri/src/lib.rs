// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod prompts;
mod menu;
mod config;
mod llm_client;
mod hotkey;

use prompts::Prompt;
use config::AppConfig;
use llm_client::LLMClient;
use tauri::{AppHandle, Manager};

#[tauri::command]
fn get_prompts() -> Vec<Prompt> {
    Prompt::get_all_prompts()
}

#[tauri::command]
async fn apply_prompt(prompt_id: String, text: String, app: AppHandle) -> Result<String, String> {
    println!("=== Apply Prompt Debug ===");
    println!("Prompt ID: {}", prompt_id);
    println!("Text length: {} chars", text.len());
    println!("Text preview: {}", &text.chars().take(100).collect::<String>());

    // Find the prompt
    let prompts = Prompt::get_all_prompts();
    let prompt = prompts
        .iter()
        .find(|p| p.id == prompt_id)
        .ok_or_else(|| "Prompt not found".to_string())?;

    println!("Found prompt: {}", prompt.name);

    // Load configuration
    let config = AppConfig::load(&app);
    println!("Selected model: {}", config.selected_model);
    println!("Using Claude CLI: {}", config.llm.use_claude_cli);

    // Create LLM client
    let client = LLMClient::new(config.clone());

    // Process the text with the selected model asynchronously
    let prompt_text = prompt.system_prompt.clone();
    let text_clone = text.clone();
    let model_id = config.selected_model.clone();

    let result = tokio::task::spawn_blocking(move || {
        client.process_text(&prompt_text, &text_clone, &model_id)
    }).await.map_err(|e| format!("Task join error: {}", e))?;

    // Process the result
    match result {
        Ok(result) => {
            println!("✓ Success! Result length: {} chars", result.len());
            Ok(result)
        },
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            // On error, return helpful message
            Ok(format!(
                "[Error: {}]\n\n\
                Applied: {}\n\n\
                Original text:\n{}\n\n\
                Prompt:\n{}\n\n\
                ℹ️ To fix this:\n\
                - If using Claude: Make sure Claude CLI is installed (brew install claude)\n\
                - If using OpenAI: Add your API key in Settings (File → Settings)\n\
                - Check Settings to configure LLM authentication",
                e,
                prompt.name,
                text,
                prompt.system_prompt
            ))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Initialize user config directory and copy default prompts if needed
            match Prompt::ensure_user_config() {
                Ok(path) => println!("✓ User prompts config: {:?}", path),
                Err(e) => eprintln!("⚠ Could not ensure user config: {}", e),
            }

            // Create and set up the menu
            menu::create_menu(app)?;

            // Load config to get hotkey preference
            let config = AppConfig::load(&app.handle());

            // Set up global shortcut with configured hotkey
            let app_handle = app.handle().clone();
            if let Err(e) = hotkey::setup_global_shortcut(&app_handle, &config.global_hotkey) {
                eprintln!("Failed to register global shortcut: {}", e);
            }

            // Set up system tray using StatusNotifier (for Linux compatibility with Polybar, i3bar, etc.)
            #[cfg(target_os = "linux")]
            {
                println!("Setting up system tray...");
                println!("Desktop session: {:?}", std::env::var("XDG_CURRENT_DESKTOP"));
                println!("Session type: {:?}", std::env::var("XDG_SESSION_TYPE"));

                // Create tray service
                use ksni::TrayService;

                struct SamwiseTray {
                    app_handle: AppHandle,
                }

                impl ksni::Tray for SamwiseTray {
                    fn icon_name(&self) -> String {
                        // Use a standard icon that exists on most Linux systems
                        "accessories-text-editor".to_string()
                    }

                    fn title(&self) -> String {
                        "Samwise".to_string()
                    }

                    fn tool_tip(&self) -> ksni::ToolTip {
                        ksni::ToolTip {
                            icon_name: "accessories-text-editor".to_string(),
                            title: "Samwise - Press Ctrl+Shift+Space".to_string(),
                            description: "Click to open or use Ctrl+Shift+Space".to_string(),
                            ..Default::default()
                        }
                    }

                    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
                        use ksni::menu::*;
                        vec![
                            StandardItem {
                                label: "Show Samwise".to_string(),
                                activate: Box::new(|this: &mut Self| {
                                    println!("Tray menu: Show clicked");
                                    if let Some(window) = this.app_handle.get_webview_window("main") {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }),
                                ..Default::default()
                            }.into(),
                            MenuItem::Separator,
                            StandardItem {
                                label: "Quit".to_string(),
                                activate: Box::new(|_this: &mut Self| {
                                    println!("Tray menu: Quit clicked");
                                    std::process::exit(0);
                                }),
                                ..Default::default()
                            }.into(),
                        ]
                    }

                    fn activate(&mut self, _x: i32, _y: i32) {
                        println!("Tray icon: Activated (clicked)");
                        if let Some(window) = self.app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }

                let tray_service = TrayService::new(SamwiseTray {
                    app_handle: app.handle().clone(),
                });

                // Spawn tray service in background thread
                std::thread::spawn(move || {
                    println!("✓ System tray icon created successfully (StatusNotifier protocol)");
                    if let Err(e) = tray_service.run() {
                        eprintln!("⚠ System tray service error: {}", e);
                    }
                });
            }

            #[cfg(not(target_os = "linux"))]
            {
                println!("✓ System tray not needed on this platform (using native tray support)");
            }

            // Keep app running in background when window is closed
            let window = app.get_webview_window("main").unwrap();
            let app_handle_close = app.handle().clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent the window from closing, just hide it
                    if let Some(window) = app_handle_close.get_webview_window("main") {
                        window.hide().unwrap();
                    }
                    api.prevent_close();
                }
            });

            // Hide window on startup (runs in background with tray icon)
            window.hide()?;

            Ok(())
        })
        .on_menu_event(|app, event| {
            menu::handle_menu_event(app, event);
        })
        .invoke_handler(tauri::generate_handler![
            get_prompts,
            apply_prompt,
            config::get_config,
            config::save_config,
            config::check_claude_cli,
            hotkey::update_global_shortcut
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
