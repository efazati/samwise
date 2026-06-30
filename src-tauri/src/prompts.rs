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

        let mut prompts = match fs::read_to_string(&prompts_path) {
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
        };

        // Ensure "raw" prompt is always first
        prompts.sort_by(|a, b| {
            match (a.id.as_str(), b.id.as_str()) {
                ("raw", _) => std::cmp::Ordering::Less,
                (_, "raw") => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });

        prompts
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
                id: "raw".to_string(),
                name: "Direct Chat".to_string(),
                description: "Talk directly to the LLM - your text goes straight to the model".to_string(),
                system_prompt: "".to_string(),
                icon: "💬".to_string(),
            },
            Prompt {
                id: "fix_grammar".to_string(),
                name: "Fix Grammar".to_string(),
                description: "Fix only grammar, spelling, and punctuation".to_string(),
                system_prompt: "Fix only the grammar, spelling, and punctuation of the text. Do not reword, rephrase, restructure, or change the style, tone, or vocabulary. Change a word only when it is grammatically wrong. If the text is already correct, return it exactly as it is.".to_string(),
                icon: "✓".to_string(),
            },
            Prompt {
                id: "improve_text".to_string(),
                name: "Improve Writing".to_string(),
                description: "Make it clearer and easier to read".to_string(),
                system_prompt: "Rewrite the text so it reads more clearly and naturally. Fix awkward phrasing, tighten wordy parts, and improve the flow. Keep the original meaning, tone, and language. Do not add new ideas or change the facts.".to_string(),
                icon: "✨".to_string(),
            },
            Prompt {
                id: "summarize".to_string(),
                name: "Summarize".to_string(),
                description: "Shorten to the key points".to_string(),
                system_prompt: "Summarize the text in about one third of its length. Keep the main ideas and the important details. Do not add opinions or new information.".to_string(),
                icon: "📝".to_string(),
            },
            Prompt {
                id: "expand".to_string(),
                name: "Expand".to_string(),
                description: "Add helpful detail and context".to_string(),
                system_prompt: "Expand the text with helpful detail, context, and examples. Keep the same tone and meaning. Do not change the original facts or add unrelated ideas.".to_string(),
                icon: "📖".to_string(),
            },
            Prompt {
                id: "simplify".to_string(),
                name: "Simplify".to_string(),
                description: "Make it easy for anyone to read".to_string(),
                system_prompt: "Rewrite the text so it is easy to read at about an 8th-grade level. Use short sentences and simple, everyday words. Keep all the meaning. Do not leave anything important out.".to_string(),
                icon: "💡".to_string(),
            },
            Prompt {
                id: "professional".to_string(),
                name: "Make Professional".to_string(),
                description: "Polished, business-ready tone".to_string(),
                system_prompt: "Rewrite the text in a polished, professional tone for business use. Keep it clear and direct. Remove slang, casual phrases, and emojis. Keep the original meaning.".to_string(),
                icon: "💼".to_string(),
            },
            Prompt {
                id: "fact_check".to_string(),
                name: "Fact Check".to_string(),
                description: "Check claims and flag problems".to_string(),
                system_prompt: "Check the text for factual and logical problems. List each claim, say whether it looks correct, questionable, or wrong, and note anything missing or unsupported. Be brief and clear.".to_string(),
                icon: "🔍".to_string(),
            },
            Prompt {
                id: "make_concise".to_string(),
                name: "Make Concise".to_string(),
                description: "Cut the fluff, keep the meaning".to_string(),
                system_prompt: "Make the text shorter while keeping every important point. Cut filler, repetition, and weak words. Keep the meaning and tone. Aim for about half the length.".to_string(),
                icon: "⚡".to_string(),
            },
        ]
    }
}

