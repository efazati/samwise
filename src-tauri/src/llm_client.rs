// LLM Client for various providers
use crate::config::AppConfig;
use std::process::Command;

pub struct LLMClient {
    config: AppConfig,
}

impl LLMClient {
    pub fn new(config: AppConfig) -> Self {
        LLMClient { config }
    }

    pub fn process_text(&self, prompt: &str, text: &str, model_id: &str) -> Result<String, String> {
        match model_id {
            // Claude models - use CLI if enabled
            id if id.starts_with("claude") => {
                if self.config.llm.use_claude_cli {
                    self.call_claude_cli(prompt, text)
                } else if let Some(api_key) = &self.config.llm.anthropic_api_key {
                    self.call_anthropic_api(prompt, text, api_key, id)
                } else {
                    Err("Claude CLI is disabled and no Anthropic API key configured".to_string())
                }
            }
            // OpenAI models
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
}

