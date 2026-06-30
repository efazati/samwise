// Menu system for Samwise
use tauri::{App, AppHandle, Emitter, Manager, Wry};
use tauri::menu::{
    CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder,
};

use crate::config::AppConfig;

// Handles to the backend check items, kept in app state so we can move the
// checkmark when the backend changes.
struct BackendMenuItems {
    claude: CheckMenuItem<Wry>,
    codex: CheckMenuItem<Wry>,
}

pub fn create_menu(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items for File menu
    let settings_item = MenuItemBuilder::with_id("settings", "Settings")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;

    let exit_item = MenuItemBuilder::with_id("exit", "Exit")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    // Create File submenu
    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&settings_item)
        .separator()
        .item(&exit_item)
        .build()?;

    // Backend menu: pick which CLI tool runs the text. A checkmark shows which
    // one is active, so the dropdown makes the current backend obvious.
    let backend = AppConfig::load(app.handle()).backend;

    let claude_item = CheckMenuItemBuilder::new("Claude CLI")
        .id("backend_claude")
        .checked(backend == "claude")
        .build(app)?;

    let codex_item = CheckMenuItemBuilder::new("Codex")
        .id("backend_codex")
        .checked(backend == "codex")
        .build(app)?;

    let backend_menu = SubmenuBuilder::new(app, "Backend")
        .item(&claude_item)
        .item(&codex_item)
        .build()?;

    // Keep the check items so we can update them later.
    app.manage(BackendMenuItems {
        claude: claude_item,
        codex: codex_item,
    });

    // Create the main menu
    let menu = MenuBuilder::new(app)
        .item(&file_menu)
        .item(&backend_menu)
        .build()?;

    // Set the menu for the app
    app.set_menu(menu)?;

    Ok(())
}

pub fn handle_menu_event(app: &AppHandle<Wry>, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        // File menu items
        "settings" => {
            println!("Settings clicked");
            // Emit event to frontend to open settings
            app.emit("menu-settings", ()).unwrap();
        }
        "exit" => {
            println!("Exit clicked");
            std::process::exit(0);
        }

        // Backend menu items
        "backend_claude" => {
            println!("Selected backend: Claude CLI");
            set_active_backend(app, "claude");
            app.emit("backend-selected", "claude").unwrap();
        }
        "backend_codex" => {
            println!("Selected backend: Codex");
            set_active_backend(app, "codex");
            app.emit("backend-selected", "codex").unwrap();
        }
        _ => {}
    }
}

// Move the checkmark to the active backend so the dropdown always shows it.
fn set_active_backend(app: &AppHandle<Wry>, backend: &str) {
    if let Some(items) = app.try_state::<BackendMenuItems>() {
        let _ = items.claude.set_checked(backend == "claude");
        let _ = items.codex.set_checked(backend == "codex");
    }
}
