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
    if config.llm.force_atlascloud_for_claude {
        println!("Force AtlasCloud for Claude: true (will use AtlasCloud API even if CLI is available)");
    }

    // Create LLM client
    let client = LLMClient::new(config.clone());

    // Process the text with the selected model asynchronously
    // For "raw" prompt, use empty system prompt to send text directly to LLM
    let prompt_text = if prompt.id == "raw" {
        "".to_string()
    } else {
        prompt.system_prompt.clone()
    };
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
            match hotkey::setup_global_shortcut(&app_handle, &config.global_hotkey) {
                Ok(_) => println!("✓ Global hotkey registered: {}", config.global_hotkey),
                Err(e) => {
                    eprintln!("⚠ Failed to register global shortcut '{}': {}", config.global_hotkey, e);
                    eprintln!("  → Try a different hotkey in Settings (e.g., Super+Space, Ctrl+Alt+S)");
                    eprintln!("  → The app will still work - you can open it from the tray icon or menu");
                }
            }

            // Set up system tray using Tauri's built-in support (works without snixembed!)
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{TrayIconBuilder, MouseButton};

            let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
            let i3_socket = std::env::var("I3SOCK").is_ok();

            if desktop.contains("i3") || i3_socket {
                println!("✓ i3 window manager detected!");
                println!("  → Tray icon will appear (no snixembed needed!)");
                println!("  → TIP: Use i3 scratchpad for quick access:");
                println!("     Add to ~/.config/i3/config:");
                println!("       for_window [class=\"samwise\"] move scratchpad");
                println!("       bindsym $mod+grave [class=\"samwise\"] scratchpad show");
            }

            // Get the hotkey from config for the tooltip
            let hotkey_text = config.global_hotkey.clone();
            let app_handle = app.handle();

            // Create tray menu with hotkey hint only on show
            let show_label = format!("Show Window ({})", hotkey_text);
            let show_item = MenuItem::with_id(app_handle, "show", show_label, true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app_handle, "hide", "Hide Window (Close window)", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app_handle, &[&show_item, &hide_item, &quit_item])?;

            // Build tray icon with tooltip showing the hotkey
            let tooltip_text = format!("Samwise - Press {} to show (Close window to hide)", hotkey_text);

            // Load the samwise icon for tray
            // The icon should be loaded from tauri.conf.json bundle configuration
            // We put 32x32.png first in the icon list since tray icons need PNG format
            let tray_icon = {
                // The default_window_icon() uses the first icon from tauri.conf.json
                // We've configured 32x32.png to be first, which should work for tray icons
                match app.default_window_icon() {
                    Some(icon) => {
                        println!("✓ Using window icon for tray (from tauri.conf.json - should be 32x32.png)");
                        icon.clone()
                    }
                    None => {
                        eprintln!("⚠ No default window icon available");
                        // This shouldn't happen, but provide a fallback
                        panic!("No window icon available - check tauri.conf.json icon configuration");
                    }
                }
            };

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(tray_icon)
                .tooltip(&tooltip_text)
                .on_menu_event(|app_handle, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::TrayIconEvent;
                    if matches!(event, TrayIconEvent::Click { button: MouseButton::Left, .. }) {
                        let app_handle = tray.app_handle();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app_handle)?;

            println!("✓ System tray icon created (using Tauri native - no snixembed needed!)");

            // Prevent window from closing - hide it instead
            let window = app.get_webview_window("main").unwrap();
            let app_handle_for_close = app.handle().clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    println!("Window close requested - hiding instead of closing");
                    api.prevent_close();
                    if let Some(win) = app_handle_for_close.get_webview_window("main") {
                        let _ = win.hide();
                    }
                }
            });

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
