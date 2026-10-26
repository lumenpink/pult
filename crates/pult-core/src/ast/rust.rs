use crate::ast::error::AstError;
use crate::ast::extractor::AstExtractor;
use crate::ast::models::ExtractedAstFile;
use std::path::Path;

/// Extrator estrutural de AST de alta performance para a linguagem Rust via `syn` e `quote`.
pub struct RustAstExtractor;

impl Default for RustAstExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAstExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl AstExtractor for RustAstExtractor {
    fn extract_from_str(&self, _file_path: &Path, _content: &str) -> Result<String, AstError> {
        todo!("Fase 5: Implementação de extract_from_str")
    }

    fn extract_from_file(&self, _file_path: &Path) -> Result<ExtractedAstFile, AstError> {
        todo!("Fase 5: Implementação de extract_from_file")
    }

    fn extract_from_dir(
        &self,
        _root: &Path,
        _ignore_dirs: &[&str],
    ) -> Result<Vec<ExtractedAstFile>, AstError> {
        todo!("Fase 5: Implementação de extract_from_dir")
    }
}
