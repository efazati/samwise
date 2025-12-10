// Global hotkey management for Samwise
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_clipboard_manager::ClipboardExt;
use crate::config::AppConfig;

pub fn setup_global_shortcut(app: &AppHandle, hotkey: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Registering global shortcut: {}", hotkey);

    // First, unregister all existing shortcuts to avoid conflicts
    let _ = app.global_shortcut().unregister_all();

    let app_handle = app.clone();
    let hotkey_str = hotkey.to_string();

    // Set up the callback first
    app.global_shortcut().on_shortcut(hotkey, move |_app, _shortcut, _event| {
        println!("Global shortcut triggered!");

        if let Some(window) = app_handle.get_webview_window("main") {
            // Only show the window (don't toggle - closing window hides it)
            println!("Showing window via hotkey");

            // Get clipboard content
            let clipboard_text = match app_handle.clipboard().read_text() {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("Failed to read clipboard: {}", e);
                    String::new()
                }
            };

            if let Err(e) = window.show() {
                eprintln!("Failed to show window: {}", e);
            }
            if let Err(e) = window.set_focus() {
                eprintln!("Failed to focus window: {}", e);
            }

            // Emit event to frontend with clipboard text
            if let Err(e) = app_handle.emit("hotkey-triggered", clipboard_text) {
                eprintln!("Failed to emit hotkey event: {}", e);
            }
        }
    })?;

    // Register the shortcut
    let shortcut_obj: Shortcut = hotkey_str.parse()
        .map_err(|e| format!("Failed to parse hotkey '{}': {}. Try: Super+Space, Ctrl+Alt+S, or Super+S", hotkey, e))?;

    app.global_shortcut().register(shortcut_obj)
        .map_err(|e| format!("Failed to register hotkey '{}': {}. This hotkey may be in use by another application or your system.", hotkey, e))?;

    println!("✓ Global shortcut registered successfully: {}", hotkey);

    Ok(())
}

#[tauri::command]
pub fn update_global_shortcut(app: AppHandle, new_hotkey: String) -> Result<(), String> {
    // Unregister all existing shortcuts
    app.global_shortcut().unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {}", e))?;

    // Register the new shortcut
    setup_global_shortcut(&app, &new_hotkey)
        .map_err(|e| format!("Failed to register new shortcut: {}", e))?;

    // Update config
    let mut config = AppConfig::load(&app);
    config.global_hotkey = new_hotkey;
    config.save(&app)?;

    Ok(())
}

