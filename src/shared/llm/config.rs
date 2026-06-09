use super::client_anthropic::AnthropicClient;
use super::client_google::GoogleClient;
use super::client_ollama::OllamaClient;
use super::client_openai::OpenAiClient;
use super::service::{LlmService, Registry, TieredRegistry};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Provider {
    OpenAi,
    Anthropic,
    Ollama,
    Google,
}

impl Provider {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(Self::OpenAi),
            "anthropic" => Some(Self::Anthropic),
            "ollama" => Some(Self::Ollama),
            "google" => Some(Self::Google),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider: Provider,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone)]
struct TierSpec {
    provider: Provider,
    model: String,
}

/// Load LLM configuration from environment variables and build a Registry.
///
/// Provider credentials:
///   OPENAI_API_KEY, OPENAI_BASE_URL
///   ANTHROPIC_API_KEY, ANTHROPIC_BASE_URL
///   OLLAMA_BASE_URL (no key required)
///   GOOGLE_API_KEY, GOOGLE_BASE_URL
///
/// Tier assignments:
///   LLM_TIER_HIGH_PROVIDER, LLM_TIER_HIGH_MODEL
///   LLM_TIER_MEDIUM_PROVIDER, LLM_TIER_MEDIUM_MODEL
///   LLM_TIER_LOW_PROVIDER, LLM_TIER_LOW_MODEL
pub fn registry_from_env() -> anyhow::Result<Arc<dyn Registry>> {
    let mut providers: HashMap<Provider, ProviderConfig> = HashMap::new();

    let load = |p: Provider, prefix: &str| -> Option<ProviderConfig> {
        let api_key = std::env::var(format!("{prefix}API_KEY")).ok();
        let base_url = std::env::var(format!("{prefix}BASE_URL")).ok();
        if api_key.is_some() || base_url.is_some() || matches!(p, Provider::Ollama) {
            Some(ProviderConfig { provider: p, base_url, api_key })
        } else {
            None
        }
    };

    if let Some(c) = load(Provider::OpenAi, "OPENAI_") {
        providers.insert(Provider::OpenAi, c);
    }
    if let Some(c) = load(Provider::Anthropic, "ANTHROPIC_") {
        providers.insert(Provider::Anthropic, c);
    }
    if let Some(c) = load(Provider::Ollama, "OLLAMA_") {
        providers.insert(Provider::Ollama, c);
    }
    if let Some(c) = load(Provider::Google, "GOOGLE_") {
        providers.insert(Provider::Google, c);
    }

    let parse_tier = |provider_env: &str, model_env: &str| -> Option<TierSpec> {
        let provider_str = std::env::var(provider_env).ok()?;
        let model = std::env::var(model_env).ok()?;
        if provider_str.is_empty() || model.is_empty() {
            return None;
        }
        Provider::from_str(&provider_str).map(|p| TierSpec { provider: p, model })
    };

    let build = |tier_name: &str, spec: Option<TierSpec>| -> anyhow::Result<Option<Arc<dyn LlmService>>> {
        let Some(spec) = spec else { return Ok(None) };

        let pcfg = match providers.get(&spec.provider) {
            Some(c) => c,
            None => {
                tracing::warn!(tier = tier_name, "credentials not found for provider, skipping tier");
                return Ok(None);
            }
        };

        let service: Arc<dyn LlmService> = match spec.provider {
            Provider::OpenAi => Arc::new(OpenAiClient::new(
                pcfg.base_url.clone(),
                pcfg.api_key.clone().unwrap_or_default(),
                spec.model,
            )),
            Provider::Anthropic => Arc::new(AnthropicClient::new(
                pcfg.base_url.clone(),
                pcfg.api_key.clone().unwrap_or_default(),
                spec.model,
            )),
            Provider::Ollama => Arc::new(OllamaClient::new(
                pcfg.base_url.clone(),
                spec.model,
            )),
            Provider::Google => Arc::new(GoogleClient::new(
                pcfg.base_url.clone(),
                pcfg.api_key.clone().unwrap_or_default(),
                spec.model,
            )),
        };

        Ok(Some(service))
    };

    let high_spec = parse_tier("LLM_TIER_HIGH_PROVIDER", "LLM_TIER_HIGH_MODEL");
    let medium_spec = parse_tier("LLM_TIER_MEDIUM_PROVIDER", "LLM_TIER_MEDIUM_MODEL");
    let low_spec = parse_tier("LLM_TIER_LOW_PROVIDER", "LLM_TIER_LOW_MODEL");

    let high = build("high", high_spec)?;
    let medium = build("medium", medium_spec)?;
    let low = build("low", low_spec)?;

    if high.is_none() && medium.is_none() && low.is_none() {
        tracing::warn!("no LLM tiers configured; AI features will be unavailable");
    }

    Ok(Arc::new(TieredRegistry { high, medium, low }))
}
