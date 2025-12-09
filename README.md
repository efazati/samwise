# Samwise

A lightweight, always-available desktop utility for text transformation with AI. Select text anywhere, press a hotkey, and transform it instantly.

## Quick Start

```bash
# First time setup
make setup

# Run the app
make dev

# Build production binary
make build
```

## Features

- **Global Hotkey**: Press `Cmd/Ctrl + Shift + Space` anywhere to activate
- **Clipboard Integration**: Automatically loads copied text
- **System Tray**: Runs in background, accessible from tray icon
- **6 AI Prompts**: Fix Grammar, Improve Text, Summarize, Expand, Simplify, Make Professional
- **Multiple LLMs**: ChatGPT (OpenAI) and Claude (Anthropic)
- **Claude CLI Support**: Use your Claude.ai subscription (no API key needed)

## How It Works

When you launch Samwise:
- App starts in **background mode** (window hidden, tray icon visible)
- **Global hotkey** is registered and ready
- Always available until you explicitly quit

Workflow:
1. **Select and copy text** anywhere (email, document, webpage, etc.)
2. **Press hotkey**: `Cmd/Ctrl + Shift + Space`
3. **Window appears** with your clipboard text loaded
4. **Choose an action**: Click a prompt (Fix Grammar, Summarize, etc.)
5. **Get instant results**: AI transforms your text
6. **Close window**: Hides to tray, ready for next use

## Configuration

### LLM Setup

**Option 1: Claude CLI (Recommended)**
```bash
brew install claude
# Restart Samwise - it auto-detects and uses CLI
```

**Option 2: API Keys**
1. Open Settings: `Cmd/Ctrl + ,`
2. Add your API keys:
   - OpenAI: https://platform.openai.com/api-keys
   - Anthropic: https://console.anthropic.com/settings/keys

### Hotkey Configuration

1. Open Settings: `Cmd/Ctrl + ,`
2. Change "Global Hotkey" field
3. Press Enter to apply

Examples: `CmdOrCtrl+Shift+Space`, `Alt+Space`, `CmdOrCtrl+K`

## Available Commands

```bash
make dev          # Start development
make build        # Build production binary
make clean        # Clean build artifacts
make setup        # Install dependencies
make help         # Show all commands
```

## Tech Stack

- **Backend**: Rust + Tauri v2
- **Frontend**: React + TypeScript
- **Build**: Vite + Cargo

## Background Mode

Samwise runs in the background automatically:

- **On startup**: Window is hidden, tray icon appears
- **Global hotkey**: Always active (`Cmd/Ctrl + Shift + Space`)
- **Closing window**: Hides to tray (doesn't quit)
- **To quit**: Use `File → Exit` or `Cmd/Ctrl + Q`

### Start on Login (Optional)

**Linux**: Add to startup applications
```bash
# In your desktop environment settings, add:
# Command: /path/to/samwise
```

**macOS**: Add to Login Items
```
System Preferences → Users & Groups → Login Items → Add Samwise.app
```

**Windows**: Add shortcut to Startup folder
```
Win + R → shell:startup → Add Samwise shortcut
```

## Troubleshooting

**Hotkey not working?**
- Make sure Samwise is running (check system tray)
- Check if another app is using the same hotkey
- Try a different hotkey in Settings

**Tray icon not showing? (Linux)**

System tray support varies by desktop environment:

- **GNOME**: Install extension `sudo apt-get install gnome-shell-extension-appindicator`
- **Other DEs**: Install `sudo apt-get install libappindicator3-1 gir1.2-appindicator3-0.1`
- **If tray still doesn't work**: The app works perfectly with just the hotkey!

Note: The app is fully functional even without a visible tray icon. Use `Ctrl+Shift+Space` to show it and `Ctrl+Q` to quit.

**Actions not working?**
- Configure LLM in Settings (Claude CLI or API keys)
- Check terminal output for errors
- Verify Claude CLI: `claude --version`

**App won't start?**
- Install system dependencies: `sudo apt-get install libxdo-dev libappindicator3-dev`
- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install dependencies: `make setup`

**Stop all processes:**
```bash
make kill
```

## License

MIT
