use crate::ast::ExtractedAstFile;
use serde::{Deserialize, Serialize};

/// Informações de telemetria de um modelo rodando na GPU (retornado por /api/ps).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuModelInfo {
    pub name: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub size_vram: u64,
    #[serde(default)]
    pub expires_at: String,
}

impl GpuModelInfo {
    pub fn is_in_vram(&self) -> bool {
        self.size_vram > 0
    }
}

/// Contexto de entrada para a inferência de filtragem do Scout.
#[derive(Debug, Clone)]
pub struct ScoutPromptContext<'a> {
    pub task_intent: &'a str,
    pub ast_files: &'a [ExtractedAstFile],
}

impl<'a> ScoutPromptContext<'a> {
    pub fn new(task_intent: &'a str, ast_files: &'a [ExtractedAstFile]) -> Self {
        Self {
            task_intent,
            ast_files,
        }
    }

    pub fn format_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str("Você é o Scout, assistente local de filtragem cirúrgica de código.\n");
        prompt.push_str("Analise a tarefa e o sumário de assinaturas da base de código abaixo.\n");
        prompt.push_str("Identifique EXCLUSIVAMENTE os 2 a 4 símbolos, funções, structs ou traits diretamente pertinentes para implementar a tarefa.\n");
        prompt.push_str("Responda APENAS com um bloco de código yaml no seguinte formato exato:\n\n");
        prompt.push_str("```yaml\n");
        prompt.push_str("injected_ast:\n");
        prompt.push_str("  - \"caminho/arquivo: assinatura do simbolo\"\n");
        prompt.push_str("```\n\n");
        prompt.push_str(&format!("## Tarefa Pretendida:\n{}\n\n", self.task_intent));
        prompt.push_str("## Sumário AST da Base de Código:\n");

        for file in self.ast_files {
            if !file.extracted_signatures.is_empty() {
                prompt.push_str(&format!(
                    "### {}\n```\n{}\n```\n\n",
                    file.relative_path, file.extracted_signatures
                ));
            }
        }

        prompt
    }
}

/// Resultado estruturado da inferência do Scout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoutResult {
    pub symbols: Vec<String>,
    pub yaml_block: String,
    pub model_used: String,
    pub vram_allocated_bytes: Option<u64>,
    pub bypassed: bool,
}

impl ScoutResult {
    pub fn bypassed() -> Self {
        Self {
            symbols: Vec::new(),
            yaml_block: String::new(),
            model_used: "none".to_string(),
            vram_allocated_bytes: None,
            bypassed: true,
        }
    }
}
