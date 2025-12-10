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

    // Create menu items for AtlasCloud models
    let atlas_gpt51_item = MenuItemBuilder::with_id("llm_atlas_gpt51", "GPT-5.1 (AtlasCloud)")
        .build(app)?;

    let atlas_deepseek_item = MenuItemBuilder::with_id("llm_atlas_deepseek", "DeepSeek V3.2 (AtlasCloud)")
        .build(app)?;

    let atlas_gpt5mini_item = MenuItemBuilder::with_id("llm_atlas_gpt5mini", "GPT-5 Mini Developer (AtlasCloud)")
        .build(app)?;

    let atlas_gemini_item = MenuItemBuilder::with_id("llm_atlas_gemini", "Gemini 2.5 Flash (AtlasCloud)")
        .build(app)?;

    // Create menu items for AtlasCloud Claude models (regular Claude, not Claude Code)
    let atlas_claude_sonnet_item = MenuItemBuilder::with_id("llm_atlas_claude_sonnet", "Claude 3.5 Sonnet (AtlasCloud)")
        .build(app)?;

    let atlas_claude_opus_item = MenuItemBuilder::with_id("llm_atlas_claude_opus", "Claude 3 Opus (AtlasCloud)")
        .build(app)?;

    let atlas_claude_haiku_item = MenuItemBuilder::with_id("llm_atlas_claude_haiku", "Claude 3 Haiku (AtlasCloud)")
        .build(app)?;

    // Create LLM Models submenu
    // Section titles are made more prominent with separators and clear labeling
    let llm_menu = SubmenuBuilder::new(app, "LLM Models")
        .separator()
        .text("llm_section_openai", "━━━ ChatGPT (OpenAI) ━━━")
        .item(&gpt4_item)
        .item(&gpt35_item)
        .separator()
        .text("llm_section_anthropic", "━━━ Claude (Anthropic / CLI) ━━━")
        .item(&claude_sonnet_item)
        .item(&claude_opus_item)
        .item(&claude_haiku_item)
        .separator()
        .text("llm_section_atlascloud", "━━━ AtlasCloud ━━━")
        .item(&atlas_gpt51_item)
        .item(&atlas_deepseek_item)
        .item(&atlas_gpt5mini_item)
        .item(&atlas_gemini_item)
        .separator()
        .text("llm_section_atlascloud_claude", "━━━ Claude (AtlasCloud) ━━━")
        .item(&atlas_claude_sonnet_item)
        .item(&atlas_claude_opus_item)
        .item(&atlas_claude_haiku_item)
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
        "llm_atlas_gpt51" => {
            println!("Selected: GPT-5.1 (AtlasCloud)");
            app.emit("llm-selected", "openai/gpt-5.1").unwrap();
        }
        "llm_atlas_deepseek" => {
            println!("Selected: DeepSeek V3.2 (AtlasCloud)");
            app.emit("llm-selected", "deepseek-ai/deepseek-v3.2-speciale").unwrap();
        }
        "llm_atlas_gpt5mini" => {
            println!("Selected: GPT-5 Mini Developer (AtlasCloud)");
            app.emit("llm-selected", "openai/gpt-5-mini-developer").unwrap();
        }
        "llm_atlas_gemini" => {
            println!("Selected: Gemini 2.5 Flash (AtlasCloud)");
            app.emit("llm-selected", "google/gemini-2.5-flash").unwrap();
        }
        "llm_atlas_claude_sonnet" => {
            println!("Selected: Claude 3.5 Sonnet (AtlasCloud)");
            app.emit("llm-selected", "anthropic/claude-3-5-sonnet").unwrap();
        }
        "llm_atlas_claude_opus" => {
            println!("Selected: Claude 3 Opus (AtlasCloud)");
            app.emit("llm-selected", "anthropic/claude-3-opus").unwrap();
        }
        "llm_atlas_claude_haiku" => {
            println!("Selected: Claude 3 Haiku (AtlasCloud)");
            app.emit("llm-selected", "anthropic/claude-3-haiku").unwrap();
        }
        _ => {}
    }
}

