// LLM Client for various providers
use crate::config::AppConfig;
use std::process::Command;
use serde_json::json;

pub struct LLMClient {
    config: AppConfig,
}

impl LLMClient {
    pub fn new(config: AppConfig) -> Self {
        LLMClient { config }
    }

    pub fn process_text(&self, prompt: &str, text: &str, model_id: &str) -> Result<String, String> {
        match model_id {
            // AtlasCloud models MUST be checked FIRST (before plain claude/gpt models)
            // This includes OpenAI, Anthropic Claude, DeepSeek, and Google models via AtlasCloud
            // Models containing "/" are AtlasCloud models (e.g., "openai/gpt-5.1", "anthropic/claude-3-5-sonnet")
            id if id.contains("/") ||
                  id == "openai/gpt-5.1" ||
                  id == "deepseek-ai/deepseek-v3.2-speciale" ||
                  id == "openai/gpt-5-mini-developer" ||
                  id == "google/gemini-2.5-flash" ||
                  id.starts_with("anthropic/claude") => {
                if let Some(api_key) = &self.config.llm.atlascloud_api_key {
                    self.call_atlascloud_api(prompt, text, api_key, id)
                } else {
                    Err("No AtlasCloud API key configured".to_string())
                }
            }
            // Plain Claude models (without "/" prefix) - use CLI if enabled
            id if id.starts_with("claude") => {
                if self.config.llm.use_claude_cli {
                    self.call_claude_cli(prompt, text)
                } else if let Some(api_key) = &self.config.llm.anthropic_api_key {
                    self.call_anthropic_api(prompt, text, api_key, id)
                } else {
                    Err("Claude CLI is disabled and no Anthropic API key configured".to_string())
                }
            }
            // Plain OpenAI models (without "/" prefix, not AtlasCloud)
            id if id.starts_with("gpt") => {
                if let Some(api_key) = &self.config.llm.openai_api_key {
                    self.call_openai_api(prompt, text, api_key, id)
                } else {
                    Err("No OpenAI API key configured".to_string())
                }
            }
            // Other models
            _ => Err(format!("Unsupported model: {}", model_id)),
        }
    }

    fn call_claude_cli(&self, prompt: &str, text: &str) -> Result<String, String> {
        println!("📤 Calling Claude CLI...");

        // Create a clear, structured prompt for Claude
        let full_prompt = format!(
            "{}\n\n\
            TEXT TO PROCESS:\n\
            \"\"\"\n\
            {}\n\
            \"\"\"\n\n\
            Please provide only the processed text without any explanations or meta-commentary.",
            prompt, text
        );

        println!("   Prompt length: {} chars", full_prompt.len());
        println!("   Full prompt:\n{}", full_prompt);
        println!("   ---");

        // Call Claude CLI
        println!("   Executing: claude <prompt>");
        let output = Command::new("claude")
            .arg(&full_prompt)
            .output()
            .map_err(|e| format!("Failed to execute Claude CLI: {}. Make sure Claude CLI is installed (brew install claude)", e))?;

        println!("   Exit status: {}", output.status);
        println!("   Stdout length: {} bytes", output.stdout.len());
        println!("   Stderr length: {} bytes", output.stderr.len());

        if output.status.success() {
            let result = String::from_utf8(output.stdout)
                .map_err(|e| format!("Failed to parse Claude CLI output: {}", e))?;

            println!("   Raw output: {}", result);

            // Clean up the output - remove any markdown code blocks if present
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

    fn call_anthropic_api(&self, prompt: &str, text: &str, _api_key: &str, _model_id: &str) -> Result<String, String> {
        // TODO: Implement Anthropic API call using reqwest
        // For now, return a placeholder
        Ok(format!(
            "[Anthropic API Integration Coming Soon]\n\n\
            Prompt: {}\n\
            Text: {}\n\n\
            To use this now, enable Claude CLI in settings.",
            prompt, text
        ))
    }

    fn call_openai_api(&self, prompt: &str, text: &str, _api_key: &str, _model_id: &str) -> Result<String, String> {
        // TODO: Implement OpenAI API call using reqwest
        // For now, return a placeholder
        Ok(format!(
            "[OpenAI API Integration Coming Soon]\n\n\
            Prompt: {}\n\
            Text: {}\n\n\
            You can add your OpenAI API key in Settings.",
            prompt, text
        ))
    }

    fn call_atlascloud_api(&self, prompt: &str, text: &str, api_key: &str, model_id: &str) -> Result<String, String> {
        println!("📤 Calling AtlasCloud API...");
        println!("   Model: {}", model_id);

        // Create the full prompt
        let full_prompt = format!(
            "{}\n\n\
            TEXT TO PROCESS:\n\
            \"\"\"\n\
            {}\n\
            \"\"\"\n\n\
            Please provide only the processed text without any explanations or meta-commentary.",
            prompt, text
        );

        // Build the request body using chat/completions format
        let request_body = json!({
            "model": model_id,
            "messages": [
                {
                    "role": "user",
                    "content": full_prompt
                }
            ],
            "max_tokens": 64000,
            "temperature": 1,
            "stream": false
        });

        // Create a blocking runtime for the HTTP request
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
                return Err(format!("AtlasCloud API error ({}): {}", status, error_text));
            }

            // Parse response as JSON
            let json_response: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            // Extract text from AtlasCloud response structure
            // Response can be either:
            // 1. Standard chat/completions format: choices[0].message.content
            // 2. AtlasCloud format: output[0].content[0].text
            let result = json_response
                .get("choices")
                .and_then(|choices| choices.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    // Try AtlasCloud-specific format: output[0].content[0].text
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
                        .or_else(|| json_response.get("message"))
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
}

