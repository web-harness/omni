use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderId {
    Anthropic,
    OpenAI,
    Google,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: ProviderId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: ProviderId,
    pub name: String,
    pub prefix: String,
}

pub fn list_models() -> Vec<ModelConfig> {
    vec![
        ModelConfig {
            id: "claude-3-7-sonnet".into(),
            name: "Claude 3.7 Sonnet".into(),
            provider: ProviderId::Anthropic,
        },
        ModelConfig {
            id: "claude-3-5-haiku".into(),
            name: "Claude 3.5 Haiku".into(),
            provider: ProviderId::Anthropic,
        },
        ModelConfig {
            id: "gpt-5".into(),
            name: "GPT-5".into(),
            provider: ProviderId::OpenAI,
        },
        ModelConfig {
            id: "gpt-4o".into(),
            name: "GPT-4o".into(),
            provider: ProviderId::OpenAI,
        },
        ModelConfig {
            id: "gemini-2.5-pro".into(),
            name: "Gemini 2.5 Pro".into(),
            provider: ProviderId::Google,
        },
        ModelConfig {
            id: "gemini-2.0-flash".into(),
            name: "Gemini 2.0 Flash".into(),
            provider: ProviderId::Google,
        },
        ModelConfig {
            id: "llama-3.3-70b".into(),
            name: "Llama 3.3 70B".into(),
            provider: ProviderId::Ollama,
        },
        ModelConfig {
            id: "deepseek-r1".into(),
            name: "DeepSeek R1".into(),
            provider: ProviderId::Ollama,
        },
    ]
}

pub fn list_providers() -> Vec<Provider> {
    vec![
        Provider {
            id: ProviderId::Anthropic,
            name: "Anthropic".into(),
            prefix: "anthropic".into(),
        },
        Provider {
            id: ProviderId::OpenAI,
            name: "OpenAI".into(),
            prefix: "openai".into(),
        },
        Provider {
            id: ProviderId::Google,
            name: "Google".into(),
            prefix: "google".into(),
        },
        Provider {
            id: ProviderId::Ollama,
            name: "Ollama".into(),
            prefix: "ollama".into(),
        },
    ]
}

pub async fn list_providers_with_keys() -> Result<Vec<(Provider, bool)>, std::io::Error> {
    let providers = list_providers();
    let mut result = Vec::new();
    for p in providers {
        let has_key = crate::config_store::has_api_key(&p.prefix).await?;
        result.push((p, has_key));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{list_models, list_providers, ProviderId};

    #[test]
    fn list_models_returns_expected_catalog() {
        let models = list_models();
        assert!(models.len() >= 8);
        assert!(models.iter().any(|m| m.id == "claude-3-7-sonnet"));
        assert!(models.iter().any(|m| m.id == "gpt-5"));
        assert!(models.iter().any(|m| m.id == "gemini-2.5-pro"));
    }

    #[test]
    fn list_providers_returns_core_providers() {
        let providers = list_providers();
        assert_eq!(providers.len(), 4);
        assert!(providers.iter().any(|p| p.id == ProviderId::Anthropic));
        assert!(providers.iter().any(|p| p.id == ProviderId::OpenAI));
        assert!(providers.iter().any(|p| p.id == ProviderId::Google));
        assert!(providers.iter().any(|p| p.id == ProviderId::Ollama));
    }
}
