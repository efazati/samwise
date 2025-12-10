// Prompt definitions for Samwise
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub icon: String,
}

impl Prompt {
    pub fn get_all_prompts() -> Vec<Prompt> {
        vec![
            Prompt {
                id: "fix_grammar".to_string(),
                name: "Fix Grammar".to_string(),
                description: "Correct grammar, spelling, and punctuation".to_string(),
                system_prompt: "Please correct the grammar, spelling, and punctuation in the text below. Keep the original meaning, tone, and intent exactly the same. Do not add new information or remove anything. Return only the corrected version.".to_string(),
                icon: "✓".to_string(),
            },
            Prompt {
                id: "improve_text".to_string(),
                name: "Improve Text".to_string(),
                description: "Make text clearer and smoother".to_string(),
                system_prompt: "Please rewrite the text below to make it clearer and smoother, but keep the same meaning. Use simple, everyday words (no fancy or technical vocabulary). Don't make it longer than necessary but you can make up to 50 percent longer, and keep the style sounding like the original. Return only the improved version.".to_string(),
                icon: "✨".to_string(),
            },
            Prompt {
                id: "summarize".to_string(),
                name: "Summarize".to_string(),
                description: "Create a concise summary".to_string(),
                system_prompt: "Please summarize the text below in a clear, concise way while keeping the main ideas and key details. Don't add new information or opinions. Keep the tone neutral and accurate.".to_string(),
                icon: "📝".to_string(),
            },
            Prompt {
                id: "expand".to_string(),
                name: "Expand".to_string(),
                description: "Add more detail and context".to_string(),
                system_prompt: "Please expand on the text below by adding relevant details. Keep the original meaning and tone without using complex words, but make it more comprehensive and informative. Return only the expanded version.".to_string(),
                icon: "📖".to_string(),
            },
            Prompt {
                id: "simplify".to_string(),
                name: "Simplify".to_string(),
                description: "Make text easier to understand".to_string(),
                system_prompt: "Please rewrite the text below using simpler language that anyone can understand. Keep the same meaning but use shorter sentences and common words. Make it clear and straightforward.".to_string(),
                icon: "💡".to_string(),
            },
            Prompt {
                id: "professional".to_string(),
                name: "Make Professional".to_string(),
                description: "Convert to formal business tone".to_string(),
                system_prompt: "Please rewrite the text below in a professional, business-appropriate tone. Use formal language while keeping it clear and concise. Maintain the original meaning and key points.".to_string(),
                icon: "💼".to_string(),
            },
        ]
    }
}

