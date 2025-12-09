// Menu system for Samwise
use tauri::{App, AppHandle, Emitter, Wry};
use tauri::menu::{MenuBuilder, SubmenuBuilder, MenuItemBuilder};

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

    // Create menu items for LLM Models menu
    let gpt4_item = MenuItemBuilder::with_id("llm_gpt4", "GPT-4")
        .build(app)?;

    let gpt35_item = MenuItemBuilder::with_id("llm_gpt35", "GPT-3.5 Turbo")
        .build(app)?;

    let claude_sonnet_item = MenuItemBuilder::with_id("llm_claude_sonnet", "Claude 3.5 Sonnet")
        .build(app)?;

    let claude_opus_item = MenuItemBuilder::with_id("llm_claude_opus", "Claude 3 Opus")
        .build(app)?;

    let claude_haiku_item = MenuItemBuilder::with_id("llm_claude_haiku", "Claude 3 Haiku")
        .build(app)?;

    // Create LLM Models submenu (only ChatGPT and Claude)
    let llm_menu = SubmenuBuilder::new(app, "LLM Models")
        .text("llm_section_openai", "ChatGPT (OpenAI)")
        .item(&gpt4_item)
        .item(&gpt35_item)
        .separator()
        .text("llm_section_anthropic", "Claude (Anthropic)")
        .item(&claude_sonnet_item)
        .item(&claude_opus_item)
        .item(&claude_haiku_item)
        .build()?;

    // Create the main menu
    let menu = MenuBuilder::new(app)
        .item(&file_menu)
        .item(&llm_menu)
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

        // LLM Models menu items
        "llm_gpt4" => {
            println!("Selected: GPT-4");
            app.emit("llm-selected", "gpt-4").unwrap();
        }
        "llm_gpt35" => {
            println!("Selected: GPT-3.5 Turbo");
            app.emit("llm-selected", "gpt-3.5-turbo").unwrap();
        }
        "llm_claude_sonnet" => {
            println!("Selected: Claude 3.5 Sonnet");
            app.emit("llm-selected", "claude-3-5-sonnet").unwrap();
        }
        "llm_claude_opus" => {
            println!("Selected: Claude 3 Opus");
            app.emit("llm-selected", "claude-3-opus").unwrap();
        }
        "llm_claude_haiku" => {
            println!("Selected: Claude 3 Haiku");
            app.emit("llm-selected", "claude-3-haiku").unwrap();
        }
        _ => {}
    }
}

