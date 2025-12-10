# Samwise - Agent/Developer Guide

## Project Overview

Samwise is a cross-platform desktop app (Linux, macOS, Windows) built with Tauri v2 + React + TypeScript. It provides always-available text transformation using AI models via a global hotkey.

## Architecture

```
Frontend (React)          Backend (Rust)
├── src/App.tsx          ├── src-tauri/src/lib.rs      (main logic)
├── src/App.css          ├── src-tauri/src/prompts.rs  (prompt definitions)
└── src/main.tsx         ├── src-tauri/src/menu.rs     (menu system)
                         ├── src-tauri/src/config.rs   (configuration)
                         ├── src-tauri/src/llm_client.rs (LLM integration)
                         └── src-tauri/src/hotkey.rs   (global hotkey)
```

## Key Components

### 1. Prompts System (`prompts.rs`)
Defines 6 AI prompts: Fix Grammar, Improve Text, Summarize, Expand, Simplify, Make Professional.

**Adding a new prompt:**
```rust
Prompt {
    id: "translate".to_string(),
    name: "Translate".to_string(),
    description: "Translate to another language".to_string(),
    system_prompt: "Translate the following text...".to_string(),
    icon: "🌐".to_string(),
}
```

### 2. LLM Client (`llm_client.rs`)
Handles communication with AI models:
- **Claude**: Via CLI (preferred) or Anthropic API
- **ChatGPT**: Via OpenAI API

**Authentication hierarchy:**
- Claude models: Try CLI first → Fall back to API
- OpenAI models: Use API key

### 3. Configuration (`config.rs`)
Stores settings in `~/.config/samwise/config.json` (Linux) or equivalent:
```json
{
  "llm": {
    "openai_api_key": "sk-...",
    "anthropic_api_key": null,
    "use_claude_cli": true,
    "claude_cli_model": "claude-3-5-sonnet-20241022"
  },
  "selected_model": "claude-3-5-sonnet",
  "global_hotkey": "Super+Alt+S"
}
```

### 4. Global Hotkey (`hotkey.rs`)
Registers system-wide keyboard shortcut:
- Default: `Cmd/Ctrl + Shift + Space`
- Captures clipboard on trigger
- Shows window with text pre-loaded

### 5. Window Management & System Tray
Smart window behavior:
- **Closing window** (X button) → Hides window (doesn't quit app)
- **Quit app** → Use File menu → Quit, or tray icon → Quit
- **System tray icon** → Shows in taskbar with tooltip showing hotkey
  - Left-click: Toggle window visibility
  - Right-click: Menu (Show/Hide/Quit)
- **Global hotkey** (default: Super+Alt+S) → Toggles window visibility
  - When hidden → Shows window with clipboard text
  - When visible → Hides window
- For i3 users: Scratchpad also works great

## Development Workflow

### Running the App
```bash
make dev          # Development with hot reload
make build        # Production build
```

### Adding Features

**New Tauri Command:**
1. Add to `lib.rs`:
```rust
#[tauri::command]
fn my_command(param: String) -> Result<String, String> {
    Ok(format!("Result: {}", param))
}
```

2. Register in `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    get_prompts,
    apply_prompt,
    my_command  // Add here
])
```

3. Call from React:
```typescript
const result = await invoke<string>("my_command", { param: "test" });
```

**New Menu Item:**
Edit `menu.rs` → Add to `create_menu()` and `handle_menu_event()`

**New Prompt:**
Edit `prompts.rs` → Add to `get_all_prompts()` vector

## Common Tasks

### Change Window Behavior
Edit `tauri.conf.json`:
```json
"windows": [{
  "width": 1000,
  "height": 700,
  "alwaysOnTop": true,
  "decorations": false
}]
```

### Change Default Hotkey
Edit `config.rs` → Change `Default::default()` → `global_hotkey` field

### Add New LLM Provider
1. Add to `llm_client.rs` → `process_text()` match statement
2. Add to `menu.rs` → New menu items
3. Update `config.rs` → Add API key field

## Building for Production

```bash
# Build for your platform
make build

# Output locations:
# Linux:   src-tauri/target/release/bundle/appimage/
# macOS:   src-tauri/target/release/bundle/dmg/
# Windows: src-tauri/target/release/bundle/msi/
```

## Debugging

### View Rust Logs
```bash
# Terminal shows Rust println! output when running make dev
```

### View Frontend Logs
Right-click window → Inspect → Console

### Check Config File
```bash
# Linux
cat ~/.config/samwise/config.json

# macOS
cat ~/Library/Application\ Support/com.samwise.app/config.json
```

## Known Issues & Solutions

**"Hotkey already registered"**
- Another instance is running
- Solution: `pkill -f samwise` then restart

**"Failed to execute Claude CLI"**
- Claude CLI not installed
- Solution: `brew install claude`

**Actions not working**
- No LLM configured
- Solution: Configure Claude CLI or add API keys in Settings

**Empty page/blank window on Linux**
- Fixed in latest version with proper CSP configuration
- If still occurring, ensure you're running the latest build

**Using Samwise with i3 Window Manager**
- System tray works! Uses Tauri's native tray (no snixembed needed)
- Tray tooltip shows the hotkey: "Samwise - Press Super+Alt+S to toggle"
- Optional: Use i3 scratchpad for additional quick access:
  ```bash
  # Add to ~/.config/i3/config
  for_window [class="samwise"] move scratchpad
  bindsym $mod+grave [class="samwise"] scratchpad show
  ```
- Both methods work great together!

## Dependencies

### Rust Crates
- `tauri` - Desktop app framework
- `tauri-plugin-global-shortcut` - System-wide hotkeys
- `tauri-plugin-clipboard-manager` - Clipboard access
- `tray-icon` - System tray integration
- `serde` / `serde_json` - Serialization

### npm Packages
- `react` / `react-dom` - UI framework
- `@tauri-apps/api` - Tauri frontend API
- `vite` - Build tool
- `typescript` - Type safety

## Project Structure

```
samwise/
├── src/                    # React frontend
│   ├── App.tsx            # Main UI component
│   ├── App.css            # Styling
│   └── main.tsx           # Entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Main app logic
│   │   ├── main.rs        # Entry point
│   │   ├── prompts.rs     # Prompt definitions
│   │   ├── menu.rs        # Menu system
│   │   ├── config.rs      # Configuration
│   │   ├── llm_client.rs  # LLM integration
│   │   └── hotkey.rs      # Global hotkey
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri config
├── Makefile               # Build automation
├── README.md              # This file
└── AGENT.md               # Developer guide
```

## Future Enhancements

- [ ] Streaming LLM responses
- [ ] Custom prompt management
- [ ] Prompt history
- [ ] Direct text replacement (paste back to source app)
- [ ] Multiple language support
- [ ] Prompt templates with variables
- [ ] Keyboard shortcuts for prompts
- [ ] Window positioning near cursor

## Resources

- Tauri Docs: https://tauri.app/
- Tauri Plugins: https://tauri.app/plugin/
- React Docs: https://react.dev/

