// Prompt definitions for Samwise
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
struct PromptsFile {
    prompts: Vec<Prompt>,
}

impl Prompt {
    /// Get the path to the prompts.yaml file
    /// Priority: 1) User config dir, 2) Project root (fallback)
    fn get_prompts_file_path() -> PathBuf {
        // Try user config directory first
        if let Some(config_dir) = dirs::config_dir() {
            let user_prompts = config_dir.join("samwise").join("prompts.yaml");
            if user_prompts.exists() {
                return user_prompts;
            }
        }

        // Fallback to project root (for development or bundled default)
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        project_root.join("prompts.yaml")
    }

    /// Load prompts from YAML file
    pub fn get_all_prompts() -> Vec<Prompt> {
        let prompts_path = Self::get_prompts_file_path();

        match fs::read_to_string(&prompts_path) {
            Ok(content) => {
                match serde_yaml::from_str::<PromptsFile>(&content) {
                    Ok(prompts_file) => {
                        println!("✓ Loaded {} prompts from: {:?}", prompts_file.prompts.len(), prompts_path);
                        prompts_file.prompts
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to parse prompts.yaml: {}", e);
                        Self::get_default_prompts()
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to read prompts.yaml from {:?}: {}", prompts_path, e);
                Self::get_default_prompts()
            }
        }
    }

    /// Copy default prompts.yaml to user config directory
    pub fn ensure_user_config() -> Result<PathBuf, std::io::Error> {
        if let Some(config_dir) = dirs::config_dir() {
            let samwise_config = config_dir.join("samwise");
            let user_prompts = samwise_config.join("prompts.yaml");

            // Create directory if it doesn't exist
            fs::create_dir_all(&samwise_config)?;

            // Copy default prompts if user version doesn't exist
            if !user_prompts.exists() {
                let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .to_path_buf();
                let default_prompts = project_root.join("prompts.yaml");

                if default_prompts.exists() {
                    fs::copy(&default_prompts, &user_prompts)?;
                    println!("✓ Created user prompts config at: {:?}", user_prompts);
                }
            }

            Ok(user_prompts)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find config directory",
            ))
        }
    }

    /// Fallback default prompts (in case YAML file is not found)
    fn get_default_prompts() -> Vec<Prompt> {
        eprintln!("⚠ Using hardcoded default prompts as fallback");
        vec![
            Prompt {
                id: "fix_grammar".to_string(),
                name: "Fix Grammar".to_string(),
                description: "Perfect grammar, spelling, and punctuation".to_string(),
                system_prompt: "You are an expert editor. Correct all grammar, spelling, punctuation, and typographical errors in the text below. Fix errors only - do not change word choice, style, or tone. Preserve the original voice and intent completely. Return ONLY the corrected text with no explanations.".to_string(),
                icon: "✓".to_string(),
            },
            Prompt {
                id: "improve_text".to_string(),
                name: "Improve Writing".to_string(),
                description: "Enhance clarity and readability".to_string(),
                system_prompt: "You are a writing coach. Improve the text below to make it more clear, engaging, and easier to read. Enhance clarity, improve flow, use strong verbs, remove redundancy. Maintain the original tone and voice. Return ONLY the improved text.".to_string(),
                icon: "✨".to_string(),
            },
            Prompt {
                id: "summarize".to_string(),
                name: "Summarize".to_string(),
                description: "Extract key points concisely".to_string(),
                system_prompt: "You are an expert at distilling information. Create a concise summary capturing all main ideas and critical details. Use approximately 25-35% of the original length. Be factually accurate with no additions. Return ONLY the summary.".to_string(),
                icon: "📝".to_string(),
            },
            Prompt {
                id: "expand".to_string(),
                name: "Expand".to_string(),
                description: "Add depth and context".to_string(),
                system_prompt: "You are a content developer. Expand the text with relevant details, examples, and context. Add supporting information that enhances understanding. Maintain the original tone. Aim for 150-200% of original length. Return ONLY the expanded text.".to_string(),
                icon: "📖".to_string(),
            },
            Prompt {
                id: "simplify".to_string(),
                name: "Simplify".to_string(),
                description: "Make easily understandable".to_string(),
                system_prompt: "You are an expert at clear communication. Simplify the text for maximum accessibility. Use simple words (8th-grade level), short sentences, plain language instead of jargon. Maintain all key information. Return ONLY the simplified text.".to_string(),
                icon: "💡".to_string(),
            },
            Prompt {
                id: "professional".to_string(),
                name: "Make Professional".to_string(),
                description: "Business-ready formal tone".to_string(),
                system_prompt: "You are a business communications expert. Rewrite in a professional, polished tone suitable for business contexts. Use formal but clear language, be diplomatic, remove casual phrases. Return ONLY the professional version.".to_string(),
                icon: "💼".to_string(),
            },
            Prompt {
                id: "fact_check".to_string(),
                name: "Fact Check".to_string(),
                description: "Verify claims and identify issues".to_string(),
                system_prompt: "You are a fact-checker and critical analyst. Analyze the text for accuracy, logic, and issues. List factual claims, assess accuracy, identify logical issues, note missing context, flag unsupported assertions, detect bias. Format clearly with sections.".to_string(),
                icon: "🔍".to_string(),
            },
            Prompt {
                id: "make_concise".to_string(),
                name: "Make Concise".to_string(),
                description: "Remove fluff, keep essence".to_string(),
                system_prompt: "You are an expert editor specializing in concise writing. Make the text as concise as possible while preserving all essential information. Remove redundancy, eliminate filler, use direct language. Aim for 40-60% of original length. Return ONLY the concise version.".to_string(),
                icon: "⚡".to_string(),
            },
        ]
    }
}

