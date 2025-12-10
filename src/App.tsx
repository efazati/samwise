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

interface LLMConfig {
  openai_api_key: string | null;
  anthropic_api_key: string | null;
  atlascloud_api_key: string | null;
  use_claude_cli: boolean;
  claude_cli_model: string;
}

interface AppConfig {
  llm: LLMConfig;
  selected_model: string;
  global_hotkey: string;
}

function App() {
  const [prompts, setPrompts] = useState<Prompt[]>([]);
  const [inputText, setInputText] = useState("");
  const [outputText, setOutputText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [selectedPrompt, setSelectedPrompt] = useState<string | null>(null);
  const [selectedModel, setSelectedModel] = useState<string>("claude-3-5-sonnet");
  const [showSettings, setShowSettings] = useState(false);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [claudeCliAvailable, setClaudeCliAvailable] = useState(false);
  const [isCancelled, setIsCancelled] = useState(false);
  const [showInputStroke, setShowInputStroke] = useState(false);

  useEffect(() => {
    loadPrompts();
    loadConfig();
    checkClaudeCli();
    setupMenuListeners();
  }, []);

  async function setupMenuListeners() {
    // Listen for menu events from Rust
    await listen("menu-settings", () => {
      setShowSettings(true);
    });

    await listen<string>("llm-selected", async (event) => {
      const newModel = event.payload;
      setSelectedModel(newModel);
      console.log("LLM Model selected:", newModel);

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
        const newConfig = { ...currentConfig, selected_model: newModel };
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
      setSelectedModel(loadedConfig.selected_model);
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

  function getModelDisplayName(modelId: string): string {
    const modelNames: Record<string, string> = {
      "gpt-4": "GPT-4",
      "gpt-3.5-turbo": "GPT-3.5 Turbo",
      "claude-3-5-sonnet": "Claude 3.5 Sonnet",
      "claude-3-opus": "Claude 3 Opus",
      "claude-3-haiku": "Claude 3 Haiku",
      "openai/gpt-5.1": "GPT-5.1 (AtlasCloud)",
      "deepseek-ai/deepseek-v3.2-speciale": "DeepSeek V3.2 (AtlasCloud)",
      "openai/gpt-5-mini-developer": "GPT-5 Mini Developer (AtlasCloud)",
      "google/gemini-2.5-flash": "Gemini 2.5 Flash (AtlasCloud)",
      "anthropic/claude-3-5-sonnet": "Claude 3.5 Sonnet (AtlasCloud)",
      "anthropic/claude-3-opus": "Claude 3 Opus (AtlasCloud)",
      "anthropic/claude-3-haiku": "Claude 3 Haiku (AtlasCloud)",
    };
    return modelNames[modelId] || modelId;
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
            <div className="model-indicator" title="Currently selected AI model">
              <span className="model-name">{getModelDisplayName(selectedModel)}</span>
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
                  <strong>Current Model</strong>
                </label>
                <p className="setting-description">
                  {getModelDisplayName(selectedModel)}
                </p>
                <p className="setting-hint">
                  Use the "LLM Models" menu to change the model.
                </p>
              </div>

              {claudeCliAvailable && (
                <div className="setting-group">
                  <label>
                    <input
                      type="checkbox"
                      checked={config.llm.use_claude_cli}
                      onChange={(e) => {
                        const newConfig = {
                          ...config,
                          llm: { ...config.llm, use_claude_cli: e.target.checked }
                        };
                        saveConfig(newConfig);
                      }}
                    />
                    {" "}
                    <strong>Use Claude CLI</strong>
                  </label>
                  <p className="setting-description">
                    ✅ Claude CLI is installed and available
                  </p>
                  <p className="setting-hint">
                    When enabled, Claude models will use the CLI instead of API
                  </p>
                </div>
              )}

              {!claudeCliAvailable && (
                <div className="setting-group">
                  <label>
                    <strong>Claude CLI</strong>
                  </label>
                  <p className="setting-description" style={{color: "#ef4444"}}>
                    ❌ Claude CLI not detected
                  </p>
                  <p className="setting-hint">
                    Install with: <code>brew install claude</code>
                  </p>
                </div>
              )}

              <div className="setting-group">
                <label>
                  <strong>OpenAI API Key</strong>
                </label>
                <input
                  type="password"
                  className="api-key-input"
                  placeholder="sk-..."
                  value={config.llm.openai_api_key || ""}
                  onChange={(e) => {
                    const newConfig = {
                      ...config,
                      llm: { ...config.llm, openai_api_key: e.target.value || null }
                    };
                    setConfig(newConfig);
                  }}
                  onBlur={() => saveConfig(config)}
                />
                <p className="setting-hint">
                  For GPT-4, GPT-3.5 Turbo. Get yours at <a href="https://platform.openai.com/api-keys" target="_blank">platform.openai.com</a>
                </p>
              </div>

              <div className="setting-group">
                <label>
                  <strong>Anthropic API Key</strong>
                </label>
                <input
                  type="password"
                  className="api-key-input"
                  placeholder="sk-ant-..."
                  value={config.llm.anthropic_api_key || ""}
                  onChange={(e) => {
                    const newConfig = {
                      ...config,
                      llm: { ...config.llm, anthropic_api_key: e.target.value || null }
                    };
                    setConfig(newConfig);
                  }}
                  onBlur={() => saveConfig(config)}
                />
                <p className="setting-hint">
                  For Claude models via API. Get yours at <a href="https://console.anthropic.com/settings/keys" target="_blank">console.anthropic.com</a>
                </p>
              </div>

              <div className="setting-group">
                <label>
                  <strong>AtlasCloud API Key</strong>
                </label>
                <input
                  type="password"
                  className="api-key-input"
                  placeholder="Your AtlasCloud API key"
                  value={config.llm.atlascloud_api_key || ""}
                  onChange={(e) => {
                    const newConfig = {
                      ...config,
                      llm: { ...config.llm, atlascloud_api_key: e.target.value || null }
                    };
                    setConfig(newConfig);
                  }}
                  onBlur={() => saveConfig(config)}
                />
                <p className="setting-hint">
                  For AtlasCloud models (GPT-5.1, DeepSeek V3.2, GPT-5 Mini Developer, Gemini 2.5 Flash). Get yours at <a href="https://atlascloud.ai" target="_blank">atlascloud.ai</a>
                </p>
              </div>

              <div className="setting-group">
                <label>
                  <strong>Authentication Status</strong>
                </label>
                <ul className="auth-status">
                  <li>
                    Claude: {claudeCliAvailable && config.llm.use_claude_cli ?
                      <span className="status-ok">✓ CLI Ready</span> :
                      config.llm.anthropic_api_key ?
                      <span className="status-ok">✓ API Key Set</span> :
                      <span className="status-warn">⚠ Not Configured</span>
                    }
                  </li>
                  <li>
                    OpenAI: {config.llm.openai_api_key ?
                      <span className="status-ok">✓ API Key Set</span> :
                      <span className="status-warn">⚠ Not Configured</span>
                    }
                  </li>
                  <li>
                    AtlasCloud: {config.llm.atlascloud_api_key ?
                      <span className="status-ok">✓ API Key Set</span> :
                      <span className="status-warn">⚠ Not Configured</span>
                    }
                  </li>
                </ul>
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
