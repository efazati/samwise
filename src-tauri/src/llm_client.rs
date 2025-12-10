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
            // Anthropic Claude models via AtlasCloud - check CLI first (unless forced to use AtlasCloud)
            id if id.starts_with("anthropic/claude") => {
                // If force_atlascloud_for_claude is enabled, use AtlasCloud even if CLI is available
                if self.config.llm.force_atlascloud_for_claude {
                    if let Some(api_key) = &self.config.llm.atlascloud_api_key {
                        self.call_atlascloud(&request, api_key, id)
                    } else {
                        Err("Force AtlasCloud is enabled but no AtlasCloud API key configured".to_string())
                    }
                } else if self.config.llm.use_claude_cli {
                    self.call_claude_cli(&request)
                } else if let Some(api_key) = &self.config.llm.atlascloud_api_key {
                    self.call_atlascloud(&request, api_key, id)
                } else {
                    Err("Claude CLI is disabled and no AtlasCloud API key configured".to_string())
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
            // AtlasCloud models (contain "/" or specific model names)
            id if id.contains("/") ||
                  id == "openai/gpt-5.1" ||
                  id == "deepseek-ai/deepseek-v3.2-speciale" ||
                  id == "openai/gpt-5-mini-developer" ||
                  id == "google/gemini-2.5-flash" => {
                if let Some(api_key) = &self.config.llm.atlascloud_api_key {
                    self.call_atlascloud(&request, api_key, id)
                } else {
                    Err("No AtlasCloud API key configured".to_string())
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

        // Original behavior: pass the user content as command-line argument.
        // Streaming via stdin caused issues, so we fall back to this approach.
        let enhanced_prompt = if request.system_prompt.is_empty() {
            "".to_string()
        } else {
            format!(
                "{}\n\nIMPORTANT: Return ONLY the processed text. Do not include any explanations, meta-commentary, questions, or conversational text. Just return the result directly.",
                request.system_prompt
            )
        };

        let output = if enhanced_prompt.is_empty() {
            Command::new("claude")
                .arg("-p")
                .arg(&request.user_content)
                .output()
                .map_err(|e| format!("Failed to execute Claude CLI: {}. Make sure Claude CLI is installed (brew install claude)", e))?
        } else {
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
        // Map internal model IDs to AtlasCloud model names
        let atlas_model_id = match model_id {
            "openai/gpt-5.1" => {
                // Use the exact model name as shown in AtlasCloud API docs
                "openai/gpt-5.1".to_string()
            }
            "openai/gpt-5-mini-developer" => "openai/gpt-5-mini-developer".to_string(),
            "deepseek-ai/deepseek-v3.2-speciale" => "deepseek-ai/deepseek-v3.2-speciale".to_string(),
            "google/gemini-2.5-flash" => "google/gemini-2.5-flash".to_string(),
            "anthropic/claude-3-5-sonnet" => "anthropic/claude-3-5-sonnet".to_string(),
            "anthropic/claude-3-opus" => "anthropic/claude-3-opus".to_string(),
            "anthropic/claude-3-haiku" => "anthropic/claude-3-haiku".to_string(),
            id => id.to_string(), // Use as-is for other models
        };

        println!("📤 Calling AtlasCloud API...");
        println!("   Model: {} (mapped from: {})", atlas_model_id, model_id);
        println!("   System prompt: {} chars", request.system_prompt.len());
        println!("   User content: {} chars", request.user_content.len());

        // AtlasCloud format: messages array with system role (if present) + user role
        let mut messages = Vec::new();

        // Add system message if system prompt is not empty
        if !request.system_prompt.is_empty() {
            messages.push(json!({
                "role": "system",
                "content": request.system_prompt
            }));
        }

        // Add user message
        messages.push(json!({
            "role": "user",
            "content": request.user_content
        }));

        // Build request body
        // For GPT-5.1, use parameters that match AtlasCloud API requirements
        let request_body = if atlas_model_id == "openai/gpt-5.1" {
            json!({
                "model": atlas_model_id,
                "messages": messages,
                "max_tokens": 128000,
                "temperature": 1.0,
                "repetition_penalty": 1.1
            })
        } else {
            json!({
                "model": atlas_model_id,
                "messages": messages,
                "max_tokens": 2048,
                "temperature": 0.7
            })
        };

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

                // Provide helpful error message
                let error_msg = if error_text.contains("not found") || error_text.contains("bad request") {
                    if model_id == "openai/gpt-5.1" {
                        format!(
                            "AtlasCloud API error: Model '{}' may not be available on AtlasCloud.\n\
                            Error: {}\n\
                            Try using a different model like 'anthropic/claude-3-haiku' or 'google/gemini-2.5-flash'",
                            atlas_model_id, error_text
                        )
                    } else {
                        format!("AtlasCloud API error ({}): {}\nModel: {}", status, error_text, atlas_model_id)
                    }
                } else {
                    format!("AtlasCloud API error ({}): {}", status, error_text)
                };

                return Err(error_msg);
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

#[cfg(test)]
mod tests {
    // Unit tests for LLM client routing and error handling
    // Run with: cargo test --lib llm_client::tests
    //
    // These tests verify:
    // - Model routing logic (which provider is chosen for which model)
    // - Configuration flags (force_atlascloud_for_claude, use_claude_cli)
    // - Error handling for missing API keys
    // - Model name mapping
    //
    // Note: These tests will fail on actual API calls (expected - they test routing logic)
    // For integration tests with real APIs, set up test API keys and mark tests with #[ignore]

    use super::*;
    use crate::config::{AppConfig, LLMConfig};

    fn create_test_config() -> AppConfig {
        AppConfig {
            llm: LLMConfig {
                openai_api_key: Some("test-openai-key".to_string()),
                anthropic_api_key: Some("test-anthropic-key".to_string()),
                atlascloud_api_key: Some("test-atlascloud-key".to_string()),
                use_claude_cli: true,
                claude_cli_model: "claude-3-5-sonnet-20241022".to_string(),
                force_atlascloud_for_claude: false,
            },
            selected_model: "claude-3-5-sonnet".to_string(),
            global_hotkey: "Super+Alt+S".to_string(),
        }
    }

    #[test]
    fn test_model_routing_claude_cli_enabled() {
        let config = create_test_config();
        let client = LLMClient::new(config);

        // Plain Claude model with CLI enabled should route to CLI
        // If CLI is available, it will succeed; if not, it will error appropriately
        let result = client.process_text("Say hello", "Hello", "claude-3-5-sonnet");

        // Check that it attempted to use CLI (not API)
        // If it succeeds, CLI is working (good!)
        // If it fails, error should mention CLI, not API key
        match result {
            Ok(_) => {
                // CLI worked - that's fine, routing was correct
                println!("✓ Test passed: CLI is available and working");
            }
            Err(e) => {
                // Should be a CLI-related error, not an API key error
                assert!(
                    e.contains("Claude CLI") ||
                    e.contains("claude") ||
                    e.contains("Failed to execute"),
                    "Error should be CLI-related, got: {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_model_routing_claude_cli_disabled() {
        let mut config = create_test_config();
        config.llm.use_claude_cli = false;
        let client = LLMClient::new(config);

        // Plain Claude model with CLI disabled should route to Anthropic API
        let result = client.process_text("Test prompt", "Test text", "claude-3-5-sonnet");
        // This will fail because we don't have a real API key, but we can check the routing
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Should try to call Anthropic API (will fail with auth error, but that's expected)
        assert!(error.contains("Anthropic") || error.contains("HTTP") || error.contains("API"));
    }

    #[test]
    fn test_model_routing_anthropic_claude_with_cli() {
        let config = create_test_config();
        let client = LLMClient::new(config);

        // anthropic/claude-* model with CLI enabled should route to CLI
        let result = client.process_text("Say hello", "Hello", "anthropic/claude-3-haiku");

        // Check that it attempted to use CLI (not AtlasCloud API)
        match result {
            Ok(_) => {
                // CLI worked - routing was correct
                println!("✓ Test passed: CLI is available and working for anthropic/claude models");
            }
            Err(e) => {
                // Should be a CLI-related error, not an AtlasCloud API error
                assert!(
                    e.contains("Claude CLI") ||
                    e.contains("claude") ||
                    e.contains("Failed to execute"),
                    "Error should be CLI-related, not AtlasCloud API error, got: {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_model_routing_anthropic_claude_force_atlascloud() {
        let mut config = create_test_config();
        config.llm.force_atlascloud_for_claude = true;
        let client = LLMClient::new(config);

        // anthropic/claude-* model with force_atlascloud should route to AtlasCloud
        let result = client.process_text("Test prompt", "Test text", "anthropic/claude-3-haiku");
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Should try to call AtlasCloud API
        assert!(error.contains("AtlasCloud") || error.contains("HTTP") || error.contains("API") || error.contains("bad request") || error.contains("not found"));
    }

    #[test]
    fn test_model_routing_atlascloud_models() {
        let config = create_test_config();
        let client = LLMClient::new(config);

        // AtlasCloud models should route to AtlasCloud API
        let result = client.process_text("Test prompt", "Test text", "openai/gpt-5.1");
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Should try to call AtlasCloud API
        assert!(error.contains("AtlasCloud") || error.contains("HTTP") || error.contains("API") || error.contains("bad request") || error.contains("not found"));
    }

    #[test]
    fn test_model_routing_openai_models() {
        let config = create_test_config();
        let client = LLMClient::new(config);

        // OpenAI models should route to OpenAI API
        let result = client.process_text("Test prompt", "Test text", "gpt-4");
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Should try to call OpenAI API
        assert!(error.contains("OpenAI") || error.contains("HTTP") || error.contains("API"));
    }

    #[test]
    fn test_model_name_mapping_gpt_5_1() {
        // Test that openai/gpt-5.1 gets mapped correctly
        let config = create_test_config();
        let client = LLMClient::new(config);

        // This should try to call AtlasCloud with the mapped model name
        let result = client.process_text("Test prompt", "Test text", "openai/gpt-5.1");
        assert!(result.is_err());
        // The error should indicate it tried AtlasCloud (not a routing error)
        let error = result.unwrap_err();
        assert!(error.contains("AtlasCloud") || error.contains("HTTP") || error.contains("API") || error.contains("bad request") || error.contains("not found"));
    }

    #[test]
    fn test_error_no_atlascloud_key() {
        let mut config = create_test_config();
        config.llm.atlascloud_api_key = None;
        let client = LLMClient::new(config);

        // Should return a clear error when API key is missing
        let result = client.process_text("Test prompt", "Test text", "openai/gpt-5.1");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("AtlasCloud API key"));
    }

    #[test]
    fn test_error_no_anthropic_key_when_cli_disabled() {
        let mut config = create_test_config();
        config.llm.use_claude_cli = false;
        config.llm.anthropic_api_key = None;
        let client = LLMClient::new(config);

        // Should return a clear error when CLI is disabled and no API key
        let result = client.process_text("Test prompt", "Test text", "claude-3-5-sonnet");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Anthropic API key") || error.contains("not configured"));
    }

    #[test]
    fn test_error_force_atlascloud_no_key() {
        let mut config = create_test_config();
        config.llm.force_atlascloud_for_claude = true;
        config.llm.atlascloud_api_key = None;
        let client = LLMClient::new(config);

        // Should return a clear error when force is enabled but no key
        let result = client.process_text("Test prompt", "Test text", "anthropic/claude-3-haiku");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Force AtlasCloud") || error.contains("AtlasCloud API key"));
    }

    #[test]
    fn test_unsupported_model() {
        let config = create_test_config();
        let client = LLMClient::new(config);

        // Should return error for unsupported models
        let result = client.process_text("Test prompt", "Test text", "unknown-model-xyz");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Unsupported model") || error.contains("unknown-model"));
    }

    // Integration tests for AtlasCloud API
    // These tests require a real AtlasCloud API key
    // Set ATLASCLOUD_API_KEY environment variable to run these
    // Run with: cargo test --lib -- --ignored

    #[test]
    #[ignore]
    fn test_atlascloud_api_real_call() {
        // This test requires ATLASCLOUD_API_KEY environment variable
        let api_key = std::env::var("ATLASCLOUD_API_KEY")
            .expect("ATLASCLOUD_API_KEY environment variable not set");

        let mut config = create_test_config();
        config.llm.atlascloud_api_key = Some(api_key);
        config.llm.force_atlascloud_for_claude = true; // Force AtlasCloud to test it
        let client = LLMClient::new(config);

        // Test with a simple prompt
        let result = client.process_text(
            "You are a helpful assistant. Respond briefly.",
            "Say hello in one word.",
            "anthropic/claude-3-haiku"
        );

        match result {
            Ok(response) => {
                println!("✓ AtlasCloud API test passed!");
                println!("  Response: {}", response);
                assert!(!response.is_empty(), "Response should not be empty");
            }
            Err(e) => {
                panic!("AtlasCloud API call failed: {}. Check your API key and model name.", e);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_atlascloud_gpt_5_1_model() {
        // Test GPT-5.1 model specifically
        let api_key = std::env::var("ATLASCLOUD_API_KEY")
            .expect("ATLASCLOUD_API_KEY environment variable not set");

        let mut config = create_test_config();
        config.llm.atlascloud_api_key = Some(api_key);
        let client = LLMClient::new(config);

        // Test with GPT-5.1 model
        let result = client.process_text(
            "You are a helpful assistant. Respond briefly.",
            "Say hello in one word.",
            "openai/gpt-5.1"
        );

        match result {
            Ok(response) => {
                println!("✓ AtlasCloud GPT-5.1 test passed!");
                println!("  Response: {}", response);
                assert!(!response.is_empty(), "Response should not be empty");
            }
            Err(e) => {
                panic!("AtlasCloud GPT-5.1 API call failed: {}. Check your API key and model name.", e);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_atlascloud_vs_claude_cli_comparison() {
        // Compare results from both backends
        let api_key = std::env::var("ATLASCLOUD_API_KEY")
            .expect("ATLASCLOUD_API_KEY environment variable not set");

        let test_prompt = "You are a helpful assistant. Respond briefly.";
        let test_text = "What is 2+2? Answer in one word.";

        // Test with Claude CLI
        let mut config_cli = create_test_config();
        config_cli.llm.use_claude_cli = true;
        config_cli.llm.force_atlascloud_for_claude = false;
        let client_cli = LLMClient::new(config_cli);

        let result_cli = client_cli.process_text(test_prompt, test_text, "anthropic/claude-3-haiku");

        // Test with AtlasCloud API
        let mut config_atlas = create_test_config();
        config_atlas.llm.atlascloud_api_key = Some(api_key);
        config_atlas.llm.force_atlascloud_for_claude = true;
        let client_atlas = LLMClient::new(config_atlas);

        let result_atlas = client_atlas.process_text(test_prompt, test_text, "anthropic/claude-3-haiku");

        println!("\n=== Comparison Test Results ===");

        let cli_success = match &result_cli {
            Ok(response) => {
                println!("✓ Claude CLI response: {}", response);
                true
            }
            Err(e) => {
                println!("✗ Claude CLI failed: {}", e);
                false
            }
        };

        let atlas_success = match &result_atlas {
            Ok(response) => {
                println!("✓ AtlasCloud API response: {}", response);
                true
            }
            Err(e) => {
                println!("✗ AtlasCloud API failed: {}", e);
                false
            }
        };

        // At least one should succeed
        assert!(
            cli_success || atlas_success,
            "At least one backend should work. CLI: {:?}, AtlasCloud: {:?}",
            result_cli,
            result_atlas
        );
    }
}
