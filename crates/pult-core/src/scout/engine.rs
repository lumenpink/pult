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
            full_prompt: String::new(),
            raw_response: raw_response.to_string(),
        })
    }

    pub fn parse_scout_response_with_payloads(
        raw_response: &str,
        model_used: &str,
        vram_allocated_bytes: Option<u64>,
        full_prompt: &str,
        raw_resp: &str,
    ) -> Result<ScoutResult, ScoutError> {
        let mut res = Self::parse_scout_response(raw_response, model_used, vram_allocated_bytes)?;
        res.full_prompt = full_prompt.to_string();
        res.raw_response = raw_resp.to_string();
        Ok(res)
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

        Self::parse_scout_response_with_payloads(
            &response,
            &self.model_name,
            vram,
            &prompt,
            &response,
        )
    }

    /// Detecta se a entrada do usuário é uma resposta curta afirmativa ou de confirmação
    pub fn is_short_confirmation(text: &str) -> bool {
        let cleaned = text
            .trim()
            .trim_matches(|c: char| c.is_ascii_punctuation())
            .to_lowercase();
        matches!(
            cleaned.as_str(),
            "y" | "yes" | "sim" | "s" | "ok" | "concordo" | "aprovado" | "aprovo"
        )
    }

    pub async fn filter_with_bypass(
        &self,
        task_intent: &str,
        ast_files: &[ExtractedAstFile],
    ) -> Result<ScoutResult, ScoutError> {
        if ast_files.is_empty() || Self::is_short_confirmation(task_intent) {
            let ctx = ScoutPromptContext::new(task_intent, ast_files);
            let mut bypassed = ScoutResult::bypassed();
            bypassed.full_prompt = ctx.format_prompt();
            return Ok(bypassed);
        }

        match self.filter_symbols(task_intent, ast_files).await {
            Ok(res) => Ok(res),
            Err(ScoutError::OllamaUnreachable { .. }) => {
                let ctx = ScoutPromptContext::new(task_intent, ast_files);
                let mut bypassed = ScoutResult::bypassed();
                bypassed.full_prompt = ctx.format_prompt();
                Ok(bypassed)
            }
            Err(e) => Err(e),
        }
    }

    /// Gera um slug semântico em formato kebab-case com no máximo 60 caracteres.
    /// Se o Ollama estiver acessível, solicita ao modelo local uma síntese concisa.
    /// Caso contrário, executa fallback determinístico algorítmico.
    pub async fn generate_session_slug(&self, input_text: &str) -> String {
        let trimmed = input_text.trim();
        if !trimmed.chars().any(|c| c.is_alphanumeric()) {
            return "sessao-trabalho".to_string();
        }

        let prompt = format!(
            "Resuma o seguinte objetivo de software em exatamente uma frase curta no formato kebab-case (apenas letras minúsculas a-z, números 0-9 e hífens), sem pontuação, sem acentos, com no máximo 50 caracteres:\n{}\nResponda APENAS com o slug em kebab-case, nada mais.",
            trimmed
        );

        if let Ok(resp) = self.client.generate(&self.model_name, &prompt).await {
            let clean = sanitize_kebab_slug(&resp);
            if !clean.is_empty() && clean != "sessao-trabalho" {
                return clean;
            }
        }

        sanitize_kebab_slug(trimmed)
    }
}

pub fn sanitize_kebab_slug(input: &str) -> String {
    let mut mapped = String::with_capacity(input.len());
    for ch in input.chars() {
        let ch_lower = ch.to_ascii_lowercase();
        match ch {
            'á' | 'à' | 'ã' | 'â' | 'ä' | 'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => mapped.push('a'),
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => mapped.push('e'),
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => mapped.push('i'),
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' | 'Ó' | 'Ò' | 'Õ' | 'Ô' | 'Ö' => mapped.push('o'),
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => mapped.push('u'),
            'ç' | 'Ç' => mapped.push('c'),
            'ñ' | 'Ñ' => mapped.push('n'),
            _ => {
                if ch_lower.is_ascii_alphanumeric() {
                    mapped.push(ch_lower);
                } else {
                    mapped.push('-');
                }
            }
        }
    }

    // Comprime hífens consecutivos
    let mut clean = String::with_capacity(mapped.len());
    let mut last_was_dash = false;
    for ch in mapped.chars() {
        if ch == '-' {
            if !last_was_dash {
                clean.push('-');
                last_was_dash = true;
            }
        } else {
            clean.push(ch);
            last_was_dash = false;
        }
    }

    let trimmed = clean.trim_matches('-');
    if trimmed.is_empty() {
        return "sessao-trabalho".to_string();
    }

    if trimmed.len() <= 60 {
        return trimmed.to_string();
    }

    let slice = &trimmed[..60];
    let candidate = match slice.rfind('-') {
        Some(idx) if idx >= 20 => &slice[..idx],
        _ => slice,
    };

    let result = candidate.trim_matches('-');
    if result.is_empty() {
        "sessao-trabalho".to_string()
    } else {
        result.to_string()
    }
}
