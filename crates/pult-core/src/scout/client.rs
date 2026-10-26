use crate::scout::error::ScoutError;
use crate::scout::models::GpuModelInfo;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

pub const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5-coder:3b";
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
pub const DOCKER_GATEWAY_OLLAMA_URL: &str = "http://172.17.0.1:11434";

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Debug, Deserialize)]
struct TagModel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct PsResponse {
    #[serde(default)]
    models: Vec<GpuModelInfo>,
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OllamaClient {
    pub fn new() -> Self {
        let base_url = env::var("OLLAMA_HOST").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
        Self::with_base_url(&base_url)
    }

    pub fn with_base_url(url: &str) -> Self {
        let clean_url = url.trim_end_matches('/').to_string();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            base_url: clean_url,
            client,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn check_health(&self) -> Result<(), ScoutError> {
        let url = format!("{}/api/tags", self.base_url);
        let res = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map_err(|e| ScoutError::OllamaUnreachable {
                endpoint: self.base_url.clone(),
                message: e.to_string(),
            })?;

        if !res.status().is_success() {
            return Err(ScoutError::OllamaUnreachable {
                endpoint: self.base_url.clone(),
                message: format!("Status HTTP {}", res.status()),
            });
        }
        Ok(())
    }

    pub async fn list_tags(&self) -> Result<Vec<String>, ScoutError> {
        let url = format!("{}/api/tags", self.base_url);
        let res =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| ScoutError::OllamaUnreachable {
                    endpoint: self.base_url.clone(),
                    message: e.to_string(),
                })?;

        let tags: TagsResponse = res
            .json()
            .await
            .map_err(|e| ScoutError::ParseError(e.to_string()))?;
        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    pub async fn list_running_models(&self) -> Result<Vec<GpuModelInfo>, ScoutError> {
        let url = format!("{}/api/ps", self.base_url);
        let res =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| ScoutError::OllamaUnreachable {
                    endpoint: self.base_url.clone(),
                    message: e.to_string(),
                })?;

        let text = res
            .text()
            .await
            .map_err(|e| ScoutError::ParseError(e.to_string()))?;
        Self::parse_running_models(&text)
    }

    pub fn parse_running_models(json_str: &str) -> Result<Vec<GpuModelInfo>, ScoutError> {
        let ps: PsResponse = serde_json::from_str(json_str)
            .map_err(|e| ScoutError::ParseError(format!("JSON inválido para /api/ps: {}", e)))?;
        Ok(ps.models)
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String, ScoutError> {
        let url = format!("{}/api/generate", self.base_url);
        let req = GenerateRequest {
            model,
            prompt,
            stream: false,
        };

        let res = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| ScoutError::OllamaUnreachable {
                endpoint: self.base_url.clone(),
                message: e.to_string(),
            })?;

        if !res.status().is_success() {
            return Err(ScoutError::OllamaUnreachable {
                endpoint: self.base_url.clone(),
                message: format!("Erro ao gerar: status HTTP {}", res.status()),
            });
        }

        let gen_res: GenerateResponse = res
            .json()
            .await
            .map_err(|e| ScoutError::ParseError(format!("Resposta de geração inválida: {}", e)))?;

        Ok(gen_res.response)
    }
}
