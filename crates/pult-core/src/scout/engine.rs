use crate::ast::ExtractedAstFile;
use crate::scout::client::{OllamaClient, DEFAULT_OLLAMA_MODEL};
use crate::scout::error::ScoutError;
use crate::scout::models::{ScoutPromptContext, ScoutResult};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct InjectedAstYaml {
    #[serde(default)]
    injected_ast: Vec<String>,
}

pub struct ScoutEngine {
    client: OllamaClient,
    model_name: String,
}

impl Default for ScoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoutEngine {
    pub fn new() -> Self {
        Self::with_client(OllamaClient::new())
    }

    pub fn with_client(client: OllamaClient) -> Self {
        Self {
            client,
            model_name: DEFAULT_OLLAMA_MODEL.to_string(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model_name = model.to_string();
        self
    }

    pub fn client(&self) -> &OllamaClient {
        &self.client
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn parse_scout_response(
        raw_response: &str,
        model_used: &str,
        vram_allocated_bytes: Option<u64>,
    ) -> Result<ScoutResult, ScoutError> {
        let yaml_content = if let Some(start) = raw_response.find("```yaml") {
            let after_start = &raw_response[start + 7..];
            if let Some(end) = after_start.find("```") {
                &after_start[..end]
            } else {
                after_start
            }
        } else if let Some(start) = raw_response.find("```") {
            let after_start = &raw_response[start + 3..];
            if let Some(end) = after_start.find("```") {
                &after_start[..end]
            } else {
                after_start
            }
        } else {
            raw_response
        };

        let parsed: InjectedAstYaml = serde_yaml::from_str(yaml_content).unwrap_or_else(|_| {
            let mut symbols = Vec::new();
            for line in raw_response.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("- ") {
                    let sym = trimmed
                        .trim_start_matches("- ")
                        .trim_matches('"')
                        .trim_matches('\'');
                    symbols.push(sym.to_string());
                }
            }
            InjectedAstYaml {
                injected_ast: symbols,
            }
        });

        let mut yaml_block = String::new();
        yaml_block.push_str("```yaml\ninjected_ast:\n");
        for sym in &parsed.injected_ast {
            yaml_block.push_str(&format!("  - \"{}\"\n", sym));
        }
        yaml_block.push_str("```");

        Ok(ScoutResult {
            symbols: parsed.injected_ast,
            yaml_block,
            model_used: model_used.to_string(),
            vram_allocated_bytes,
            bypassed: false,
        })
    }

    pub async fn filter_symbols(
        &self,
        task_intent: &str,
        ast_files: &[ExtractedAstFile],
    ) -> Result<ScoutResult, ScoutError> {
        let ctx = ScoutPromptContext::new(task_intent, ast_files);
        let prompt = ctx.format_prompt();

        let response = self.client.generate(&self.model_name, &prompt).await?;
        let vram = self
            .client
            .list_running_models()
            .await
            .ok()
            .and_then(|models| {
                models
                    .into_iter()
                    .find(|m| m.name.starts_with(&self.model_name))
                    .map(|m| m.size_vram)
            });

        Self::parse_scout_response(&response, &self.model_name, vram)
    }

    pub async fn filter_with_bypass(
        &self,
        task_intent: &str,
        ast_files: &[ExtractedAstFile],
    ) -> Result<ScoutResult, ScoutError> {
        match self.filter_symbols(task_intent, ast_files).await {
            Ok(res) => Ok(res),
            Err(ScoutError::OllamaUnreachable { .. }) => Ok(ScoutResult::bypassed()),
            Err(e) => Err(e),
        }
    }
}
