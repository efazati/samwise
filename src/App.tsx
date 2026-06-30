import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface Prompt {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  icon: string;
}

interface AppConfig {
  backend: string; // "claude" or "codex"
  claude_model: string; // "haiku" | "sonnet" | "opus" | full id
  codex_model: string; // empty = Codex default
  global_hotkey: string;
}

function App() {
  const [prompts, setPrompts] = useState<Prompt[]>([]);
  const [inputText, setInputText] = useState("");
  const [outputText, setOutputText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [selectedPrompt, setSelectedPrompt] = useState<string | null>(null);
  const [selectedBackend, setSelectedBackend] = useState<string>("claude");
  const [showSettings, setShowSettings] = useState(false);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [claudeCliAvailable, setClaudeCliAvailable] = useState(false);
  const [codexCliAvailable, setCodexCliAvailable] = useState(false);
  const [isCancelled, setIsCancelled] = useState(false);
  const [showInputStroke, setShowInputStroke] = useState(false);

  useEffect(() => {
    loadPrompts();
    loadConfig();
    checkClaudeCli();
    checkCodexCli();
    setupMenuListeners();
  }, []);

  async function setupMenuListeners() {
    // Listen for menu events from Rust
    await listen("menu-settings", () => {
      setShowSettings(true);
    });

    await listen<string>("backend-selected", async (event) => {
      const newBackend = event.payload;
      setSelectedBackend(newBackend);
      console.log("Backend selected:", newBackend);

      // Save to config - load config first if not already loaded
      let currentConfig = config;
      if (!currentConfig) {
        try {
          currentConfig = await invoke<AppConfig>("get_config");
          setConfig(currentConfig);
        } catch (error) {
          console.error("Failed to load config for saving:", error);
        }
      }

      if (currentConfig) {
        const newConfig = { ...currentConfig, backend: newBackend };
        await saveConfig(newConfig);
      }
    });

    // Listen for global hotkey trigger
    await listen<string>("hotkey-triggered", (event) => {
      console.log("Hotkey triggered! Clipboard text:", event.payload);

      // Auto-fill input with clipboard text
      if (event.payload && event.payload.trim()) {
        setInputText(event.payload.trim());
        setOutputText(""); // Clear previous output
        setSelectedPrompt(null);
      }
    });
  }

  async function loadPrompts() {
    try {
      const loadedPrompts = await invoke<Prompt[]>("get_prompts");
      setPrompts(loadedPrompts);
    } catch (error) {
      console.error("Failed to load prompts:", error);
    }
  }

  async function loadConfig() {
    try {
      const loadedConfig = await invoke<AppConfig>("get_config");
      setConfig(loadedConfig);
      setSelectedBackend(loadedConfig.backend);
    } catch (error) {
      console.error("Failed to load config:", error);
    }
  }

  async function saveConfig(newConfig: AppConfig) {
    try {
      await invoke("save_config", { config: newConfig });
      setConfig(newConfig);
    } catch (error) {
      console.error("Failed to save config:", error);
      alert("Failed to save configuration");
    }
  }

  async function checkClaudeCli() {
    try {
      const available = await invoke<boolean>("check_claude_cli");
      setClaudeCliAvailable(available);
    } catch (error) {
      console.error("Failed to check Claude CLI:", error);
    }
  }

  async function checkCodexCli() {
    try {
      const available = await invoke<boolean>("check_codex_cli");
      setCodexCliAvailable(available);
    } catch (error) {
      console.error("Failed to check Codex CLI:", error);
    }
  }

  async function updateHotkey(newHotkey: string) {
    if (!config) return;

    if (!newHotkey.trim()) {
      alert("Please enter a valid hotkey combination");
      return;
    }

    try {
      await invoke("update_global_shortcut", { newHotkey });
      const newConfig = { ...config, global_hotkey: newHotkey };
      setConfig(newConfig);
      alert(`Hotkey updated successfully!\n\nTry pressing: ${newHotkey}`);
    } catch (error) {
      console.error("Failed to update hotkey:", error);
      const errorMsg = String(error);
      alert(
        `Failed to register hotkey: ${errorMsg}\n\n` +
        `This hotkey may be in use by another app or your system.\n\n` +
        `Try these reliable alternatives:\n` +
        `• Super+Space\n` +
        `• Ctrl+Alt+S\n` +
        `• Super+S`
      );
    }
  }

  function applyPrompt(promptId: string) {
    if (!inputText.trim()) {
      // Trigger stroke animation
      setShowInputStroke(true);
      setTimeout(() => {
        setShowInputStroke(false);
      }, 3000); // Show for 3 seconds, then fade out
      return;
    }

    // Set loading state first
    setIsLoading(true);
    setSelectedPrompt(promptId);
    setOutputText("");
    setIsCancelled(false);

    // Use queueMicrotask to ensure React renders the loading UI before invoking backend
    queueMicrotask(() => {
      invoke<string>("apply_prompt", {
        promptId,
        text: inputText,
      })
        .then((result) => {
          // Only update output if not cancelled
          if (!isCancelled) {
            setOutputText(result);
          }
        })
        .catch((error) => {
          console.error("Error applying prompt:", error);
          // Only show error if not cancelled
          if (!isCancelled) {
            setOutputText(`Error: ${error}`);
          }
        })
        .finally(() => {
          setIsLoading(false);
          setIsCancelled(false);
        });
    });
  }

  function cancelOperation() {
    setIsCancelled(true);
    setIsLoading(false);
    setOutputText("Operation cancelled by user.");
  }

  function clearAll() {
    setInputText("");
    setOutputText("");
    setSelectedPrompt(null);
  }

  async function copyToClipboard() {
    if (!outputText) return;

    try {
      await navigator.clipboard.writeText(outputText);
      // You could add a toast notification here
      const btn = document.querySelector('.copy-btn');
      if (btn) {
        btn.textContent = '✓ Copied!';
        setTimeout(() => {
          btn.textContent = '📋 Copy';
        }, 2000);
      }
    } catch (error) {
      console.error('Failed to copy:', error);
      alert('Failed to copy to clipboard');
    }
  }

  function getBackendDisplayName(backend: string): string {
    const names: Record<string, string> = {
      claude: "Claude CLI",
      codex: "Codex",
    };
    return names[backend] || backend;
  }

  // The model in use for the active backend, capitalized for display.
  function getActiveModelLabel(): string {
    if (!config) return "";
    const model = selectedBackend === "codex" ? config.codex_model : config.claude_model;
    if (!model) return "default";
    return model.charAt(0).toUpperCase() + model.slice(1);
  }

  return (
    <>
      {/* Loading overlay at top level to ensure it's always visible */}
      {isLoading && (
        <div className="loading-overlay">
          <div className="loading-content">
            <div className="spinner"></div>
            <p className="loading-title">
              {selectedPrompt
                ? `Processing with ${prompts.find(p => p.id === selectedPrompt)?.name || 'AI'}...`
                : 'Processing...'}
            </p>
            <p className="loading-subtitle">Please wait, this may take a few seconds</p>
            <button className="cancel-btn" onClick={cancelOperation}>
              Cancel
            </button>
          </div>
        </div>
      )}

      <main className="container">
      <div className="content">
        <div className="prompts-section">
          <div className="sidebar-header">
            <h1>✨ Samwise</h1>
            <div className="model-indicator" title="Backend and model in use">
              <span className="model-name">
                {getBackendDisplayName(selectedBackend)} · {getActiveModelLabel()}
              </span>
            </div>
          </div>

          <h2>Choose an Action</h2>
          <div className="prompts-grid">
            {prompts.map((prompt) => (
              <button
                key={prompt.id}
                className={`prompt-card ${
                  selectedPrompt === prompt.id ? "selected" : ""
                }`}
                onClick={() => applyPrompt(prompt.id)}
                disabled={isLoading}
                title={prompt.description}
              >
                <span className="prompt-icon">{prompt.icon}</span>
                <span className="prompt-name">{prompt.name}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="text-section">
          <div className="input-area">
            <label htmlFor="input-text">
              <strong>Input Text</strong>
            </label>
            <textarea
              id="input-text"
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              placeholder="Paste or type your text here..."
              disabled={isLoading}
              className={showInputStroke ? "input-stroke-animation" : ""}
            />
          </div>

          {outputText && (
            <div className="output-area">
              <div className="output-header">
                <label>
                  <strong>Result</strong>
                </label>
                <div className="output-actions">
                  <button
                    className="copy-btn"
                    onClick={copyToClipboard}
                    disabled={isLoading}
                    title="Copy to clipboard"
                  >
                    📋 Copy
                  </button>
                  <button
                    className="clear-btn"
                    onClick={clearAll}
                    disabled={isLoading}
                  >
                    🗑️ Clear
                  </button>
                </div>
              </div>
              <textarea
                value={outputText}
                readOnly
                placeholder="Transformed text will appear here..."
              />
            </div>
          )}
        </div>
      </div>

      {showSettings && config && (
        <div className="modal-overlay" onClick={() => setShowSettings(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Settings</h2>
              <button
                className="close-btn"
                onClick={() => setShowSettings(false)}
              >
                ×
              </button>
            </div>
            <div className="modal-body">
              <div className="setting-group">
                <label>
                  <strong>Global Hotkey</strong>
                </label>
                <input
                  type="text"
                  className="hotkey-input"
                  placeholder="CmdOrCtrl+Shift+Space"
                  value={config.global_hotkey}
                  onChange={(e) => {
                    const newConfig = { ...config, global_hotkey: e.target.value };
                    setConfig(newConfig);
                  }}
                  onBlur={() => {
                    if (config.global_hotkey) {
                      updateHotkey(config.global_hotkey);
                    }
                  }}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      updateHotkey(config.global_hotkey);
                    }
                  }}
                />
                <p className="setting-hint">
                  Press this anywhere to show Samwise with clipboard text.
                  <br />
                  <strong>Recommended (most reliable):</strong> Super+Alt+S, Super+Space, Ctrl+Alt+S
                  <br />
                  <strong>Other options:</strong> CmdOrCtrl+Shift+Space, Alt+Space, CmdOrCtrl+K
                </p>
              </div>

              <div className="setting-group">
                <label>
                  <strong>Backend</strong>
                </label>
                <div className="backend-choice">
                  <label>
                    <input
                      type="radio"
                      name="backend"
                      value="claude"
                      checked={selectedBackend === "claude"}
                      onChange={() => {
                        setSelectedBackend("claude");
                        saveConfig({ ...config, backend: "claude" });
                      }}
                    />
                    {" "}Claude CLI
                  </label>
                  <label>
                    <input
                      type="radio"
                      name="backend"
                      value="codex"
                      checked={selectedBackend === "codex"}
                      onChange={() => {
                        setSelectedBackend("codex");
                        saveConfig({ ...config, backend: "codex" });
                      }}
                    />
                    {" "}Codex
                  </label>
                </div>
                <p className="setting-hint">
                  You can also switch from the "Backend" menu.
                </p>
              </div>

              {selectedBackend === "claude" && (
                <div className="setting-group">
                  <label>
                    <strong>Claude Model</strong>
                  </label>
                  <select
                    className="model-select"
                    value={config.claude_model}
                    onChange={(e) => {
                      saveConfig({ ...config, claude_model: e.target.value });
                    }}
                  >
                    <option value="haiku">Haiku (fastest)</option>
                    <option value="sonnet">Sonnet (balanced)</option>
                    <option value="opus">Opus (best)</option>
                  </select>
                  <p className="setting-hint">
                    Haiku is fast and works well for these short edits. Pick Sonnet or Opus for harder text.
                  </p>
                </div>
              )}

              {selectedBackend === "codex" && (
                <div className="setting-group">
                  <label>
                    <strong>Codex Model</strong>
                  </label>
                  <input
                    type="text"
                    className="api-key-input"
                    placeholder="Leave empty for Codex default"
                    value={config.codex_model}
                    onChange={(e) => {
                      setConfig({ ...config, codex_model: e.target.value });
                    }}
                    onBlur={() => saveConfig(config)}
                  />
                  <p className="setting-hint">
                    Optional. Leave empty to use whatever model Codex is set up with.
                  </p>
                </div>
              )}

              <div className="setting-group">
                <label>
                  <strong>Backend Status</strong>
                </label>
                <ul className="auth-status">
                  <li>
                    Claude CLI: {claudeCliAvailable ?
                      <span className="status-ok">✓ Installed</span> :
                      <span className="status-warn">⚠ Not found</span>
                    }
                  </li>
                  <li>
                    Codex CLI: {codexCliAvailable ?
                      <span className="status-ok">✓ Installed</span> :
                      <span className="status-warn">⚠ Not found</span>
                    }
                  </li>
                </ul>
                <p className="setting-hint">
                  Install Claude with <code>brew install claude</code>, Codex with <code>npm install -g @openai/codex</code>.
                </p>
              </div>
            </div>
            <div className="modal-footer">
              <button
                className="btn-primary"
                onClick={() => setShowSettings(false)}
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
      </main>
    </>
  );
}

export default App;
