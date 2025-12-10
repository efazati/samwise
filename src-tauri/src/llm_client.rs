// LLM Client for various providers
// Unified interface: system_prompt (instructions) + user_content (text to process)
// Each provider adapter handles conversion to its own format

use crate::config::AppConfig;
use std::process::Command;
use serde_json::json;

pub struct LLMClient {
    config: AppConfig,
}

// Unified input structure - all providers receive this
struct LLMRequest {
    system_prompt: String,  // Instructions/role (what the prompt needs)
    user_content: String,   // Text to process
}

impl LLMClient {
    pub fn new(config: AppConfig) -> Self {
        LLMClient { config }
    }

    pub fn process_text(&self, prompt: &str, text: &str, model_id: &str) -> Result<String, String> {
        let request = LLMRequest {
            system_prompt: prompt.to_string(),
            user_content: text.to_string(),
        };

        // Route to appropriate provider based on model_id
        match model_id {
            // AtlasCloud models (contain "/" or specific model names)
            id if id.contains("/") ||
                  id == "openai/gpt-5.1" ||
                  id == "deepseek-ai/deepseek-v3.2-speciale" ||
                  id == "openai/gpt-5-mini-developer" ||
                  id == "google/gemini-2.5-flash" ||
                  id.starts_with("anthropic/claude") => {
                if let Some(api_key) = &self.config.llm.atlascloud_api_key {
                    self.call_atlascloud(&request, api_key, id)
                } else {
                    Err("No AtlasCloud API key configured".to_string())
                }
            }
            // Plain Claude models - use CLI if enabled
            id if id.starts_with("claude") => {
                if self.config.llm.use_claude_cli {
                    self.call_claude_cli(&request)
                } else if let Some(api_key) = &self.config.llm.anthropic_api_key {
                    self.call_anthropic_api(&request, api_key, id)
                } else {
                    Err("Claude CLI is disabled and no Anthropic API key configured".to_string())
                }
            }
            // Plain OpenAI models
            id if id.starts_with("gpt") => {
                if let Some(api_key) = &self.config.llm.openai_api_key {
                    self.call_openai_api(&request, api_key, id)
                } else {
                    Err("No OpenAI API key configured".to_string())
                }
            }
            _ => Err(format!("Unsupported model: {}", model_id)),
        }
    }

    // ============================================================================
    // Provider Implementations
    // Each provider converts LLMRequest (system_prompt + user_content) to its format
    // ============================================================================

    fn call_claude_cli(&self, request: &LLMRequest) -> Result<String, String> {
        println!("📤 Calling Claude CLI...");
        println!("   System prompt: {} chars", request.system_prompt.len());
        println!("   User content: {} chars", request.user_content.len());

        // For raw mode (empty system prompt), send text directly without enhancement
        // For other prompts, strengthen the system prompt to ensure Claude only returns the result
        let output = if request.system_prompt.is_empty() {
            // Raw mode: just send the user content directly
            Command::new("claude")
                .arg("-p")
                .arg(&request.user_content)
                .output()
                .map_err(|e| format!("Failed to execute Claude CLI: {}. Make sure Claude CLI is installed (brew install claude)", e))?
        } else {
            // Enhanced mode: add explicit instruction to avoid conversational responses
            let enhanced_prompt = format!(
                "{}\n\nIMPORTANT: Return ONLY the processed text. Do not include any explanations, meta-commentary, questions, or conversational text. Just return the result directly.",
                request.system_prompt
            );

            Command::new("claude")
                .arg("-p")
                .arg(&request.user_content)
                .arg("--system-prompt")
                .arg(&enhanced_prompt)
                .output()
                .map_err(|e| format!("Failed to execute Claude CLI: {}. Make sure Claude CLI is installed (brew install claude)", e))?
        };

        if output.status.success() {
            let result = String::from_utf8(output.stdout)
                .map_err(|e| format!("Failed to parse Claude CLI output: {}", e))?;

            // Clean up output
            let cleaned = result
                .trim()
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();

            println!("📥 Claude CLI response received ({} chars)", cleaned.len());
            Ok(cleaned.to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("   Stderr: {}", error);
            Err(format!("Claude CLI error: {}\n\nMake sure Claude CLI is installed and authenticated.", error))
        }
    }

    fn call_atlascloud(&self, request: &LLMRequest, api_key: &str, model_id: &str) -> Result<String, String> {
        println!("📤 Calling AtlasCloud API...");
        println!("   Model: {}", model_id);
        println!("   System prompt: {} chars", request.system_prompt.len());
        println!("   User content: {} chars", request.user_content.len());

        // AtlasCloud format: systemPrompt (top-level) + messages array with user role
        let request_body = json!({
            "model": model_id,
            "systemPrompt": request.system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": request.user_content
                }
            ],
            "max_tokens": 4096,
            "temperature": 0.7
        });

        // Create blocking runtime for HTTP request
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        let result = rt.block_on(async {
            let client = reqwest::Client::new();
            let response = client
                .post("https://api.atlascloud.ai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| format!("HTTP request failed: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                eprintln!("   Error response: {}", error_text);
                return Err(format!("AtlasCloud API error ({}): {}", status, error_text));
            }

            // Parse response
            let json_response: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            // Extract text from various possible response formats
            let result = json_response
                .get("choices")
                .and_then(|choices| choices.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    // Try AtlasCloud-specific format
                    json_response
                        .get("output")
                        .and_then(|output| output.as_array())
                        .and_then(|arr| arr.get(0))
                        .and_then(|msg| msg.get("content"))
                        .and_then(|content| content.as_array())
                        .and_then(|arr| arr.get(0))
                        .and_then(|item| item.get("text"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .or_else(|| {
                    // Fallback: try direct text fields
                    json_response
                        .get("text")
                        .or_else(|| json_response.get("content"))
                        .and_then(|v| {
                            match v {
                                serde_json::Value::String(s) => Some(s.clone()),
                                serde_json::Value::Object(obj) => {
                                    obj.get("content")
                                        .or_else(|| obj.get("text"))
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                }
                                _ => None,
                            }
                        })
                })
                .ok_or_else(|| {
                    format!(
                        "Unexpected response format. Response: {}",
                        serde_json::to_string_pretty(&json_response).unwrap_or_default()
                    )
                })?;

            Ok(result)
        })?;

        println!("📥 AtlasCloud response received ({} chars)", result.len());
        Ok(result)
    }

    fn call_openai_api(&self, request: &LLMRequest, api_key: &str, model_id: &str) -> Result<String, String> {
        println!("📤 Calling OpenAI API...");
        println!("   Model: {}", model_id);
        println!("   System prompt: {} chars", request.system_prompt.len());
        println!("   User content: {} chars", request.user_content.len());

        // OpenAI format: messages array with system role + user role
        let request_body = json!({
            "model": model_id,
            "messages": [
                {
                    "role": "system",
                    "content": request.system_prompt
                },
                {
                    "role": "user",
                    "content": request.user_content
                }
            ],
            "max_tokens": 4096,
            "temperature": 0.7
        });

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        let result = rt.block_on(async {
            let client = reqwest::Client::new();
            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| format!("HTTP request failed: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                eprintln!("   Error response: {}", error_text);
                return Err(format!("OpenAI API error ({}): {}", status, error_text));
            }

            let json_response: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let result = json_response
                .get("choices")
                .and_then(|choices| choices.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    format!(
                        "Unexpected response format. Response: {}",
                        serde_json::to_string_pretty(&json_response).unwrap_or_default()
                    )
                })?;

            Ok(result)
        })?;

        println!("📥 OpenAI response received ({} chars)", result.len());
        Ok(result)
    }

    fn call_anthropic_api(&self, request: &LLMRequest, api_key: &str, model_id: &str) -> Result<String, String> {
        println!("📤 Calling Anthropic API...");
        println!("   Model: {}", model_id);
        println!("   System prompt: {} chars", request.system_prompt.len());
        println!("   User content: {} chars", request.user_content.len());

        // Anthropic format: system (top-level) + messages array with user role
        let request_body = json!({
            "model": model_id,
            "system": request.system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": request.user_content
                }
            ],
            "max_tokens": 4096
        });

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        let result = rt.block_on(async {
            let client = reqwest::Client::new();
            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| format!("HTTP request failed: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                eprintln!("   Error response: {}", error_text);
                return Err(format!("Anthropic API error ({}): {}", status, error_text));
            }

            let json_response: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            // Anthropic response format: content[0].text
            let result = json_response
                .get("content")
                .and_then(|content| content.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|item| item.get("text"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    format!(
                        "Unexpected response format. Response: {}",
                        serde_json::to_string_pretty(&json_response).unwrap_or_default()
                    )
                })?;

            Ok(result)
        })?;

        println!("📥 Anthropic response received ({} chars)", result.len());
        Ok(result)
    }
}
