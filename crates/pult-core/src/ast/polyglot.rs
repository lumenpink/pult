use crate::ast::error::AstError;
use crate::ast::extractor::AstExtractor;
use crate::ast::models::ExtractedAstFile;
use crate::ast::rust::RustAstExtractor;
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Node, Parser};
use walkdir::WalkDir;

/// Linguagens suportadas pelo motor poliglota.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLanguage {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Python,
    Php,
    Bash,
}

impl SupportedLanguage {
    /// Detecta a linguagem a partir da extensão do arquivo.
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "py" => Some(Self::Python),
            "php" => Some(Self::Php),
            "sh" | "bash" => Some(Self::Bash),
            _ => None,
        }
    }

    /// Retorna a gramática Tree-Sitter correspondente para linguagens não-Rust.
    pub fn tree_sitter_language(&self) -> Option<Language> {
        match self {
            Self::Rust => None,
            Self::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            Self::Tsx => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
            Self::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            Self::Python => Some(tree_sitter_python::LANGUAGE.into()),
            Self::Php => Some(tree_sitter_php::LANGUAGE_PHP.into()),
            Self::Bash => Some(tree_sitter_bash::LANGUAGE.into()),
        }
    }
}

/// Extrator de assinaturas de AST poliglota com despacho inteligente.
pub struct PolyglotAstExtractor {
    rust_extractor: RustAstExtractor,
}

impl Default for PolyglotAstExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PolyglotAstExtractor {
    pub fn new() -> Self {
        Self {
            rust_extractor: RustAstExtractor::new(),
        }
    }

    fn is_function_or_method(kind: &str) -> bool {
        matches!(
            kind,
            "function_definition"
                | "function_declaration"
                | "method_definition"
                | "method_declaration"
                | "arrow_function"
                | "generator_function_declaration"
        )
    }

    fn find_body_node<'a>(node: &Node<'a>) -> Option<Node<'a>> {
        if let Some(body) = node.child_by_field_name("body") {
            return Some(body);
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let k = child.kind();
                if k == "statement_block" || k == "compound_statement" || k == "block" {
                    return Some(child);
                }
            }
        }
        None
    }

    fn collect_replacements(
        node: Node,
        lang: SupportedLanguage,
        replacements: &mut Vec<(usize, usize, &'static str)>,
    ) {
        if Self::is_function_or_method(node.kind()) {
            if let Some(body) = Self::find_body_node(&node) {
                let replacement = if lang == SupportedLanguage::Python {
                    " pass"
                } else {
                    "{ ... }"
                };
                replacements.push((body.start_byte(), body.end_byte(), replacement));
                // Não desce dentro do corpo substituído
                return;
            }
        }

        // Continua a travessia nos filhos
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                Self::collect_replacements(child, lang, replacements);
            }
        }
    }
}

impl AstExtractor for PolyglotAstExtractor {
    fn extract_from_str(&self, file_path: &Path, content: &str) -> Result<String, AstError> {
        let lang = SupportedLanguage::from_path(file_path).ok_or_else(|| {
            AstError::UnsupportedLanguage(file_path.to_string_lossy().to_string())
        })?;

        if lang == SupportedLanguage::Rust {
            return self.rust_extractor.extract_from_str(file_path, content);
        }

        let ts_lang = lang.tree_sitter_language().ok_or_else(|| {
            AstError::UnsupportedLanguage(file_path.to_string_lossy().to_string())
        })?;

        let mut parser = Parser::new();
        parser
            .set_language(&ts_lang)
            .map_err(|e| AstError::ParseError {
                file: file_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| AstError::ParseError {
                file: file_path.to_string_lossy().to_string(),
                message: "Falha ao gerar árvore sintática".to_string(),
            })?;

        let mut replacements = Vec::new();
        Self::collect_replacements(tree.root_node(), lang, &mut replacements);

        // Ordena em ordem decrescente de início para substituição segura por offset de bytes
        replacements.sort_by_key(|a| std::cmp::Reverse(a.0));

        let mut result = content.to_string();
        for (start, end, rep) in replacements {
            if start <= end && end <= result.len() {
                result.replace_range(start..end, rep);
            }
        }

        Ok(result.trim().to_string())
    }

    fn extract_from_file(&self, file_path: &Path) -> Result<ExtractedAstFile, AstError> {
        let content = fs::read_to_string(file_path).map_err(|e| AstError::Io {
            path: file_path.to_path_buf(),
            source: e,
        })?;

        let raw_byte_count = content.len();
        let extracted_signatures = self.extract_from_str(file_path, &content)?;
        let reduced_byte_count = extracted_signatures.len();

        Ok(ExtractedAstFile {
            file_path: file_path.to_path_buf(),
            relative_path: file_path.to_string_lossy().to_string(),
            extracted_signatures,
            raw_byte_count,
            reduced_byte_count,
        })
    }

    fn extract_from_dir(
        &self,
        root: &Path,
        ignore_dirs: &[&str],
    ) -> Result<Vec<ExtractedAstFile>, AstError> {
        let mut results = Vec::new();

        let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            if e.depth() > 0 && file_name.starts_with('.') {
                return false;
            }
            if e.file_type().is_dir() && ignore_dirs.contains(&file_name.as_ref()) {
                return false;
            }
            true
        });

        for entry in walker {
            let entry = entry.map_err(|e| AstError::Io {
                path: root.to_path_buf(),
                source: std::io::Error::other(e.to_string()),
            })?;

            if entry.file_type().is_file() {
                let path = entry.path();
                if SupportedLanguage::from_path(path).is_some() {
                    let mut extracted = self.extract_from_file(path)?;
                    if let Ok(rel) = path.strip_prefix(root) {
                        extracted.relative_path = rel.to_string_lossy().to_string();
                    }
                    results.push(extracted);
                }
            }
        }

        Ok(results)
    }
}
