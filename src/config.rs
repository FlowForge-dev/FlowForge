use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeConfig {
    pub provider: String,
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_retries")]
    pub retries: u32,
    pub memory_enabled: bool,
    pub plugins_enabled: bool,
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    #[serde(alias = "host")]
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub timeout_secs: Option<u64>,
    pub retries: Option<u32>,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        let mut providers = BTreeMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: None,
                api_key_env: Some("OPENAI_API_KEY".to_string()),
                endpoint: None,
                model: None,
                temperature: None,
                max_tokens: None,
                timeout_secs: None,
                retries: None,
            },
        );
        providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key: None,
                api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
                endpoint: None,
                model: None,
                temperature: None,
                max_tokens: None,
                timeout_secs: None,
                retries: None,
            },
        );
        providers.insert(
            "gemini".to_string(),
            ProviderConfig {
                api_key: None,
                api_key_env: Some("GEMINI_API_KEY".to_string()),
                endpoint: None,
                model: None,
                temperature: None,
                max_tokens: None,
                timeout_secs: None,
                retries: None,
            },
        );
        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                api_key: None,
                api_key_env: None,
                endpoint: Some("http://localhost:11434".to_string()),
                model: None,
                temperature: None,
                max_tokens: None,
                timeout_secs: None,
                retries: None,
            },
        );
        providers.insert(
            "openrouter".to_string(),
            ProviderConfig {
                api_key: None,
                api_key_env: Some("OPENROUTER_API_KEY".to_string()),
                endpoint: Some("https://openrouter.ai/api/v1".to_string()),
                model: None,
                temperature: None,
                max_tokens: None,
                timeout_secs: None,
                retries: None,
            },
        );

        Self {
            provider: "openai".to_string(),
            model: "gpt-5.5".to_string(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            timeout_secs: default_timeout_secs(),
            retries: default_retries(),
            memory_enabled: true,
            plugins_enabled: true,
            providers,
        }
    }
}

impl ForgeConfig {
    pub fn provider_config(&self, provider: &str) -> ProviderConfig {
        self.providers.get(provider).cloned().unwrap_or_default()
    }

    pub fn model_for(&self, provider: &str) -> String {
        self.provider_config(provider)
            .model
            .unwrap_or_else(|| self.model.clone())
    }

    pub fn temperature_for(&self, provider: &str) -> f32 {
        self.provider_config(provider)
            .temperature
            .unwrap_or(self.temperature)
    }

    pub fn max_tokens_for(&self, provider: &str) -> u32 {
        self.provider_config(provider)
            .max_tokens
            .unwrap_or(self.max_tokens)
    }

    pub fn timeout_secs_for(&self, provider: &str) -> u64 {
        self.provider_config(provider)
            .timeout_secs
            .unwrap_or(self.timeout_secs)
    }

    pub fn retries_for(&self, provider: &str) -> u32 {
        self.provider_config(provider)
            .retries
            .unwrap_or(self.retries)
    }

    pub fn endpoint_for(&self, provider: &str, default: &str) -> String {
        self.provider_config(provider)
            .endpoint
            .unwrap_or_else(|| default.to_string())
            .trim_end_matches('/')
            .to_string()
    }

    pub fn api_key_for(&self, provider: &str) -> Option<String> {
        let provider_config = self.provider_config(provider);
        if let Some(api_key) = provider_config.api_key {
            return resolve_secret(&api_key);
        }
        provider_config
            .api_key_env
            .and_then(|name| std::env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
    }
}

impl ForgeConfig {
    pub fn load_or_default() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("failed to parse config {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = toml::to_string_pretty(self)?;
        fs::write(&path, raw).with_context(|| format!("failed to write config {}", path.display()))
    }
}

pub fn forge_home() -> Result<PathBuf> {
    if let Ok(home) = std::env::var("FORGEFLOW_HOME") {
        return Ok(PathBuf::from(home));
    }
    let home = dirs::home_dir().context("could not resolve home directory")?;
    Ok(home.join(".forgeflow"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(forge_home()?.join("config.toml"))
}

pub fn data_dir() -> Result<PathBuf> {
    forge_home()
}

pub fn patterns_dir() -> Result<PathBuf> {
    Ok(forge_home()?.join("patterns"))
}

pub fn workflows_dir() -> Result<PathBuf> {
    Ok(forge_home()?.join("workflows"))
}

pub fn plugins_dir() -> Result<PathBuf> {
    Ok(forge_home()?.join("plugins"))
}

fn resolve_secret(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.starts_with("${") && trimmed.ends_with('}') {
        let name = &trimmed[2..trimmed.len() - 1];
        return std::env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty());
    }
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    2048
}

fn default_timeout_secs() -> u64 {
    120
}

fn default_retries() -> u32 {
    2
}
