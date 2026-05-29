use crate::config::ForgeConfig;
use crate::terminal::{heading, info, success};
use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::{Client, Response, StatusCode};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub system_prompt: String,
    pub user_prompt: String,
}

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: &'static str,
    pub default_endpoint: &'static str,
    pub api_key_env: Option<&'static str>,
    pub streaming: bool,
}

#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider: &'static str,
    pub configured: bool,
    pub reachable: Option<bool>,
    pub message: String,
}

#[async_trait]
pub trait ProviderClient: Send + Sync {
    fn name(&self) -> &'static str;
    fn info(&self) -> ProviderInfo;

    async fn complete(&self, request: CompletionRequest, config: &ForgeConfig) -> Result<String> {
        let mut sink = |_delta: &str| {};
        self.complete_stream(request, config, &mut sink).await
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String>;

    async fn health(&self, config: &ForgeConfig) -> ProviderHealth;

    async fn test(&self, config: &ForgeConfig) -> Result<Option<String>> {
        if !self.health(config).await.configured {
            return Ok(None);
        }
        let request = CompletionRequest {
            system_prompt: "You are a terse health-check assistant.".to_string(),
            user_prompt: "Reply with exactly: ok".to_string(),
        };
        self.complete(request, config).await.map(Some)
    }
}

pub struct ProviderRegistry {
    clients: BTreeMap<String, Box<dyn ProviderClient>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let client = Client::new();
        let mut clients: BTreeMap<String, Box<dyn ProviderClient>> = BTreeMap::new();
        for provider in [
            ApiProvider::openai(client.clone()),
            ApiProvider::anthropic(client.clone()),
            ApiProvider::gemini(client.clone()),
            ApiProvider::ollama(client.clone()),
            ApiProvider::openrouter(client),
        ] {
            clients.insert(provider.name().to_string(), Box::new(provider));
        }
        Self { clients }
    }

    pub fn ensure_supported(&self, provider: &str) -> Result<()> {
        if self.clients.contains_key(provider) {
            Ok(())
        } else {
            bail!("unsupported provider `{provider}`")
        }
    }

    pub fn print_supported(&self, config: &ForgeConfig) {
        heading("Supported Providers");
        for (name, client) in &self.clients {
            let configured = if self.is_configured(name, config) {
                "configured"
            } else {
                "missing-key"
            };
            let active = if name == &config.provider {
                " active"
            } else {
                ""
            };
            println!(
                "{:<12} {:<12} model={}{}",
                name,
                configured,
                config.model_for(name),
                active
            );
            if client.info().api_key_env.is_none() && name == "ollama" {
                println!(
                    "{:<12} {:<12} host={}",
                    "",
                    "",
                    config.endpoint_for(name, client.info().default_endpoint)
                );
            }
        }
    }

    pub fn print_info(&self, provider: &str, config: &ForgeConfig) -> Result<()> {
        let client = self.client(provider)?;
        let details = client.info();
        heading(&format!("Provider: {}", details.name));
        println!("model       {}", config.model_for(provider));
        println!(
            "endpoint    {}",
            config.endpoint_for(provider, details.default_endpoint)
        );
        println!("streaming   {}", details.streaming);
        println!("temperature {}", config.temperature_for(provider));
        println!("max_tokens  {}", config.max_tokens_for(provider));
        println!("timeout     {}s", config.timeout_secs_for(provider));
        println!("retries     {}", config.retries_for(provider));
        if let Some(env) = details.api_key_env {
            let configured = if config.api_key_for(provider).is_some() {
                "yes"
            } else {
                "no"
            };
            println!("api_key     {configured} ({env})");
        } else {
            println!("api_key     not required");
        }
        Ok(())
    }

    pub async fn print_health(&self, provider: Option<&str>, config: &ForgeConfig) -> Result<()> {
        heading("Provider Health");
        if let Some(provider) = provider {
            let health = self.client(provider)?.health(config).await;
            print_health_line(&health);
            return Ok(());
        }
        for client in self.clients.values() {
            let health = client.health(config).await;
            print_health_line(&health);
        }
        Ok(())
    }

    pub async fn test(&self, provider: Option<&str>, config: &ForgeConfig) -> Result<()> {
        heading("Provider Test");
        if provider.is_none() {
            info(
                "Configured remote providers will receive a tiny live test prompt and may use paid credits.",
            );
        } else if provider != Some("ollama") {
            info("This sends a tiny live test prompt and may use paid provider credits.");
        }
        if let Some(provider) = provider {
            self.test_one(provider, config).await?;
            return Ok(());
        }
        for name in self.clients.keys() {
            self.test_one(name, config).await?;
        }
        Ok(())
    }

    pub async fn complete(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
    ) -> Result<String> {
        self.client(&config.provider)?
            .complete(request, config)
            .await
    }

    pub async fn complete_stream(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        self.client(&config.provider)?
            .complete_stream(request, config, on_delta)
            .await
    }

    fn client(&self, provider: &str) -> Result<&dyn ProviderClient> {
        self.clients
            .get(provider)
            .map(|client| client.as_ref())
            .ok_or_else(|| anyhow!("provider `{provider}` is not registered"))
    }

    fn is_configured(&self, provider: &str, config: &ForgeConfig) -> bool {
        if provider == "ollama" {
            true
        } else {
            config.api_key_for(provider).is_some()
        }
    }

    async fn test_one(&self, provider: &str, config: &ForgeConfig) -> Result<()> {
        let client = self.client(provider)?;
        match client.test(config).await {
            Ok(Some(output)) => success(&format!("{provider}: {}", output.trim())),
            Ok(None) => info(&format!("{provider}: skipped, not configured")),
            Err(err) => println!("{provider}: failed: {err:#}"),
        }
        Ok(())
    }
}

fn print_health_line(health: &ProviderHealth) {
    let configured = if health.configured {
        "configured"
    } else {
        "missing-key"
    };
    let reachable = match health.reachable {
        Some(true) => "reachable",
        Some(false) => "unreachable",
        None => "not-checked",
    };
    println!(
        "{:<12} {:<12} {:<12} {}",
        health.provider, configured, reachable, health.message
    );
}

#[derive(Debug, Clone, Copy)]
enum ProviderKind {
    OpenAi,
    Anthropic,
    Gemini,
    Ollama,
    OpenRouter,
}

#[derive(Clone)]
struct ApiProvider {
    kind: ProviderKind,
    name: &'static str,
    default_endpoint: &'static str,
    api_key_env: Option<&'static str>,
    client: Client,
}

impl ApiProvider {
    fn openai(client: Client) -> Self {
        Self {
            kind: ProviderKind::OpenAi,
            name: "openai",
            default_endpoint: "https://api.openai.com/v1",
            api_key_env: Some("OPENAI_API_KEY"),
            client,
        }
    }

    fn anthropic(client: Client) -> Self {
        Self {
            kind: ProviderKind::Anthropic,
            name: "anthropic",
            default_endpoint: "https://api.anthropic.com/v1",
            api_key_env: Some("ANTHROPIC_API_KEY"),
            client,
        }
    }

    fn gemini(client: Client) -> Self {
        Self {
            kind: ProviderKind::Gemini,
            name: "gemini",
            default_endpoint: "https://generativelanguage.googleapis.com/v1beta",
            api_key_env: Some("GEMINI_API_KEY"),
            client,
        }
    }

    fn ollama(client: Client) -> Self {
        Self {
            kind: ProviderKind::Ollama,
            name: "ollama",
            default_endpoint: "http://localhost:11434",
            api_key_env: None,
            client,
        }
    }

    fn openrouter(client: Client) -> Self {
        Self {
            kind: ProviderKind::OpenRouter,
            name: "openrouter",
            default_endpoint: "https://openrouter.ai/api/v1",
            api_key_env: Some("OPENROUTER_API_KEY"),
            client,
        }
    }

    fn endpoint(&self, config: &ForgeConfig) -> String {
        config.endpoint_for(self.name, self.default_endpoint)
    }

    fn model(&self, config: &ForgeConfig) -> String {
        config.model_for(self.name)
    }

    fn timeout(&self, config: &ForgeConfig) -> Duration {
        Duration::from_secs(config.timeout_secs_for(self.name))
    }

    fn api_key(&self, config: &ForgeConfig) -> Result<String> {
        config.api_key_for(self.name).ok_or_else(|| {
            anyhow!(
                "provider `{}` is missing an API key; configure [providers.{}].api_key or {}",
                self.name,
                self.name,
                self.api_key_env.unwrap_or("a provider-specific key")
            )
        })
    }

    async fn send_streaming_request(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        let retries = config.retries_for(self.name);
        let mut last_error = None;

        for attempt in 0..=retries {
            match self
                .try_stream_once(request.clone(), config, on_delta)
                .await
            {
                Ok(output) => return Ok(output),
                Err(err) => {
                    let retryable = is_retryable_error(&err);
                    last_error = Some(err);
                    if attempt == retries || !retryable {
                        break;
                    }
                    sleep(Duration::from_millis(300 * 2_u64.pow(attempt))).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("provider request failed")))
    }

    async fn try_stream_once(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        match self.kind {
            ProviderKind::OpenAi => self.openai_stream(request, config, on_delta).await,
            ProviderKind::Anthropic => self.anthropic_stream(request, config, on_delta).await,
            ProviderKind::Gemini => self.gemini_stream(request, config, on_delta).await,
            ProviderKind::Ollama => self.ollama_stream(request, config, on_delta).await,
            ProviderKind::OpenRouter => self.openrouter_stream(request, config, on_delta).await,
        }
    }

    async fn openai_stream(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        let url = format!("{}/responses", self.endpoint(config));
        let body = json!({
            "model": self.model(config),
            "input": [
                {"role": "system", "content": request.system_prompt},
                {"role": "user", "content": request.user_prompt}
            ],
            "temperature": config.temperature_for(self.name),
            "max_output_tokens": config.max_tokens_for(self.name),
            "stream": true
        });
        let response = self
            .client
            .post(url)
            .bearer_auth(self.api_key(config)?)
            .json(&body)
            .timeout(self.timeout(config))
            .send()
            .await?;
        ensure_success(response, |response| async move {
            process_sse(response, on_delta, parse_openai_delta).await
        })
        .await
    }

    async fn anthropic_stream(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        let url = format!("{}/messages", self.endpoint(config));
        let body = json!({
            "model": self.model(config),
            "system": request.system_prompt,
            "messages": [{"role": "user", "content": request.user_prompt}],
            "temperature": config.temperature_for(self.name),
            "max_tokens": config.max_tokens_for(self.name),
            "stream": true
        });
        let response = self
            .client
            .post(url)
            .header("x-api-key", self.api_key(config)?)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .timeout(self.timeout(config))
            .send()
            .await?;
        ensure_success(response, |response| async move {
            process_sse(response, on_delta, parse_anthropic_delta).await
        })
        .await
    }

    async fn gemini_stream(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        let url = format!(
            "{}/models/{}:streamGenerateContent?alt=sse",
            self.endpoint(config),
            self.model(config)
        );
        let body = json!({
            "systemInstruction": {"parts": [{"text": request.system_prompt}]},
            "contents": [{"role": "user", "parts": [{"text": request.user_prompt}]}],
            "generationConfig": {
                "temperature": config.temperature_for(self.name),
                "maxOutputTokens": config.max_tokens_for(self.name)
            }
        });
        let response = self
            .client
            .post(url)
            .header("x-goog-api-key", self.api_key(config)?)
            .json(&body)
            .timeout(self.timeout(config))
            .send()
            .await?;
        ensure_success(response, |response| async move {
            process_sse(response, on_delta, parse_gemini_delta).await
        })
        .await
    }

    async fn ollama_stream(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        let url = format!("{}/api/chat", self.endpoint(config));
        let body = json!({
            "model": self.model(config),
            "messages": [
                {"role": "system", "content": request.system_prompt},
                {"role": "user", "content": request.user_prompt}
            ],
            "stream": true,
            "options": {
                "temperature": config.temperature_for(self.name),
                "num_predict": config.max_tokens_for(self.name)
            }
        });
        let response = self
            .client
            .post(url)
            .json(&body)
            .timeout(self.timeout(config))
            .send()
            .await?;
        ensure_success(response, |response| async move {
            process_json_lines(response, on_delta, parse_ollama_delta).await
        })
        .await
    }

    async fn openrouter_stream(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.endpoint(config));
        let model = self.model(config);
        let body = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": request.system_prompt},
                {"role": "user", "content": request.user_prompt}
            ],
            "temperature": config.temperature_for(self.name),
            "max_tokens": config.max_tokens_for(self.name),
            "stream": true
        });
        let response = self
            .client
            .post(url)
            .bearer_auth(self.api_key(config)?)
            .header("HTTP-Referer", "https://github.com/forgeflow/forgeflow")
            .header("X-Title", "ForgeFlow CLI")
            .json(&body)
            .timeout(self.timeout(config))
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if status == StatusCode::NOT_FOUND || body.contains("Model not found") {
                bail!(
                    "OpenRouter model `{model}` was not found by the selected upstream provider. Try: forge provider configure openrouter --model openrouter/free"
                )
            }
            if retryable_status(status) {
                bail!("retryable provider error {status}: {body}")
            }
            bail!("provider error {status}: {body}")
        }
        ensure_success(response, |response| async move {
            process_sse(response, on_delta, parse_openrouter_delta).await
        })
        .await
    }
}

#[async_trait]
impl ProviderClient for ApiProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: self.name,
            default_endpoint: self.default_endpoint,
            api_key_env: self.api_key_env,
            streaming: true,
        }
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
        config: &ForgeConfig,
        on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    ) -> Result<String> {
        self.send_streaming_request(request, config, on_delta).await
    }

    async fn health(&self, config: &ForgeConfig) -> ProviderHealth {
        let configured = self.api_key_env.is_none() || config.api_key_for(self.name).is_some();
        if !configured {
            return ProviderHealth {
                provider: self.name,
                configured,
                reachable: None,
                message: format!("set {}", self.api_key_env.unwrap_or("api_key")),
            };
        }

        if matches!(self.kind, ProviderKind::Ollama) {
            let url = format!("{}/api/tags", self.endpoint(config));
            let result = self
                .client
                .get(url)
                .timeout(Duration::from_secs(3))
                .send()
                .await;
            return match result {
                Ok(response) if response.status().is_success() => ProviderHealth {
                    provider: self.name,
                    configured,
                    reachable: Some(true),
                    message: "local server responded".to_string(),
                },
                Ok(response) => ProviderHealth {
                    provider: self.name,
                    configured,
                    reachable: Some(false),
                    message: format!("server returned {}", response.status()),
                },
                Err(err) => ProviderHealth {
                    provider: self.name,
                    configured,
                    reachable: Some(false),
                    message: err.to_string(),
                },
            };
        }

        ProviderHealth {
            provider: self.name,
            configured,
            reachable: None,
            message: "API key is configured; run `forge provider test` for a live call".to_string(),
        }
    }
}

async fn ensure_success<F, Fut>(response: Response, parser: F) -> Result<String>
where
    F: FnOnce(Response) -> Fut,
    Fut: std::future::Future<Output = Result<String>>,
{
    let status = response.status();
    if status.is_success() {
        return parser(response).await;
    }

    let body = response.text().await.unwrap_or_default();
    if retryable_status(status) {
        bail!("retryable provider error {status}: {body}")
    }
    bail!("provider error {status}: {body}")
}

async fn process_sse(
    response: Response,
    on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    parser: fn(&str) -> Result<Option<String>>,
) -> Result<String> {
    let mut output = String::new();
    let mut buffer = String::new();
    let mut event_data = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        buffer.push_str(&String::from_utf8_lossy(&chunk?));
        while let Some(newline) = buffer.find('\n') {
            let line = buffer[..newline].trim_end_matches('\r').to_string();
            buffer = buffer[newline + 1..].to_string();
            if line.is_empty() {
                flush_sse_event(&event_data, &mut output, on_delta, parser)?;
                event_data.clear();
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                event_data.push_str(data.trim_start());
            }
        }
    }

    if !buffer.trim().is_empty()
        && let Some(data) = buffer.trim_end_matches('\r').strip_prefix("data:")
    {
        event_data.push_str(data.trim_start());
    }
    flush_sse_event(&event_data, &mut output, on_delta, parser)?;
    Ok(output)
}

async fn process_json_lines(
    response: Response,
    on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    parser: fn(&str) -> Result<Option<String>>,
) -> Result<String> {
    let mut output = String::new();
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        buffer.push_str(&String::from_utf8_lossy(&chunk?));
        while let Some(newline) = buffer.find('\n') {
            let line = buffer[..newline].trim().to_string();
            buffer = buffer[newline + 1..].to_string();
            flush_json_line(&line, &mut output, on_delta, parser)?;
        }
    }
    flush_json_line(buffer.trim(), &mut output, on_delta, parser)?;
    Ok(output)
}

fn flush_sse_event(
    data: &str,
    output: &mut String,
    on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    parser: fn(&str) -> Result<Option<String>>,
) -> Result<()> {
    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
        return Ok(());
    }
    if let Some(delta) = parser(data)? {
        on_delta(&delta);
        output.push_str(&delta);
    }
    Ok(())
}

fn flush_json_line(
    line: &str,
    output: &mut String,
    on_delta: &mut (dyn for<'a> FnMut(&'a str) + Send),
    parser: fn(&str) -> Result<Option<String>>,
) -> Result<()> {
    if line.is_empty() {
        return Ok(());
    }
    if let Some(delta) = parser(line)? {
        on_delta(&delta);
        output.push_str(&delta);
    }
    Ok(())
}

fn parse_openai_delta(data: &str) -> Result<Option<String>> {
    let value: Value = serde_json::from_str(data).with_context(|| "invalid OpenAI stream event")?;
    if value["type"] == "response.output_text.delta" {
        return Ok(value["delta"].as_str().map(ToString::to_string));
    }
    if value["type"] == "error" || value["type"] == "response.failed" {
        bail!("OpenAI stream error: {value}");
    }
    Ok(None)
}

fn parse_anthropic_delta(data: &str) -> Result<Option<String>> {
    let value: Value =
        serde_json::from_str(data).with_context(|| "invalid Anthropic stream event")?;
    if value["type"] == "content_block_delta" {
        return Ok(value["delta"]["text"].as_str().map(ToString::to_string));
    }
    if value["type"] == "error" {
        bail!("Anthropic stream error: {value}");
    }
    Ok(None)
}

fn parse_gemini_delta(data: &str) -> Result<Option<String>> {
    let value: Value = serde_json::from_str(data).with_context(|| "invalid Gemini stream event")?;
    let mut text = String::new();
    if let Some(candidates) = value["candidates"].as_array() {
        for candidate in candidates {
            if let Some(parts) = candidate["content"]["parts"].as_array() {
                for part in parts {
                    if let Some(delta) = part["text"].as_str() {
                        text.push_str(delta);
                    }
                }
            }
        }
    }
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

fn parse_ollama_delta(data: &str) -> Result<Option<String>> {
    let value: Value = serde_json::from_str(data).with_context(|| "invalid Ollama stream event")?;
    if let Some(error) = value["error"].as_str() {
        bail!("Ollama stream error: {error}");
    }
    Ok(value["message"]["content"]
        .as_str()
        .map(ToString::to_string))
}

fn parse_openrouter_delta(data: &str) -> Result<Option<String>> {
    let value: Value =
        serde_json::from_str(data).with_context(|| "invalid OpenRouter stream event")?;
    if let Some(error) = value.get("error") {
        bail!("OpenRouter stream error: {error}");
    }
    Ok(value["choices"]
        .as_array()
        .and_then(|choices| choices.first())
        .and_then(|choice| choice["delta"]["content"].as_str())
        .map(ToString::to_string))
}

fn retryable_status(status: StatusCode) -> bool {
    status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

fn is_retryable_error(err: &anyhow::Error) -> bool {
    if err.to_string().contains("retryable provider error") {
        return true;
    }
    err.downcast_ref::<reqwest::Error>()
        .map(|err| err.is_timeout() || err.is_connect())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ForgeConfig, ProviderConfig};
    use httpmock::prelude::*;

    fn test_config(provider: &str, endpoint: String) -> ForgeConfig {
        let mut config = ForgeConfig::default();
        config.provider = provider.to_string();
        config.model = match provider {
            "gemini" => "gemini-2.5-flash".to_string(),
            "anthropic" => "claude-sonnet-4-5".to_string(),
            "ollama" => "llama3.2".to_string(),
            "openrouter" => "openai/gpt-4o-mini".to_string(),
            _ => "gpt-4.1-mini".to_string(),
        };
        config.timeout_secs = 5;
        config.retries = 0;
        config.providers.insert(
            provider.to_string(),
            ProviderConfig {
                api_key: Some("test-key".to_string()),
                api_key_env: None,
                endpoint: Some(endpoint),
                model: None,
                temperature: Some(0.1),
                max_tokens: Some(32),
                timeout_secs: Some(5),
                retries: Some(0),
            },
        );
        config
    }

    async fn run_provider(provider: &str, server: &MockServer) -> String {
        let registry = ProviderRegistry::new();
        let config = test_config(provider, server.url(""));
        registry
            .complete(
                CompletionRequest {
                    system_prompt: "system".to_string(),
                    user_prompt: "user".to_string(),
                },
                &config,
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn openai_streams_response_text() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST).path("/responses");
            then.status(200)
                .header("content-type", "text/event-stream")
                .body(
                    "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hel\"}\n\n\
                 data: {\"type\":\"response.output_text.delta\",\"delta\":\"lo\"}\n\n\
                 data: [DONE]\n\n",
                );
        });
        assert_eq!(run_provider("openai", &server).await, "hello");
    }

    #[tokio::test]
    async fn anthropic_streams_text_delta() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST).path("/messages");
            then.status(200).header("content-type", "text/event-stream").body(
                "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"hi\"}}\n\n\
                 data: {\"type\":\"message_stop\"}\n\n",
            );
        });
        assert_eq!(run_provider("anthropic", &server).await, "hi");
    }

    #[tokio::test]
    async fn gemini_streams_candidate_parts() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST)
                .path("/models/gemini-2.5-flash:streamGenerateContent");
            then.status(200)
                .header("content-type", "text/event-stream")
                .body(
                    "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"gem\"}]}}]}\n\n\
                 data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"ini\"}]}}]}\n\n",
                );
        });
        assert_eq!(run_provider("gemini", &server).await, "gemini");
    }

    #[tokio::test]
    async fn ollama_streams_json_lines() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST).path("/api/chat");
            then.status(200)
                .header("content-type", "application/x-ndjson")
                .body(
                    "{\"message\":{\"content\":\"ol\"},\"done\":false}\n\
                 {\"message\":{\"content\":\"lama\"},\"done\":true}\n",
                );
        });
        assert_eq!(run_provider("ollama", &server).await, "ollama");
    }

    #[tokio::test]
    async fn openrouter_streams_chat_deltas() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST).path("/chat/completions");
            then.status(200)
                .header("content-type", "text/event-stream")
                .body(
                    "data: {\"choices\":[{\"delta\":{\"content\":\"open\"}}]}\n\n\
                 data: {\"choices\":[{\"delta\":{\"content\":\"router\"}}]}\n\n\
                 data: [DONE]\n\n",
                );
        });
        assert_eq!(run_provider("openrouter", &server).await, "openrouter");
    }
}
