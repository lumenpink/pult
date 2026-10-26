use crate::ast::error::AstError;
use crate::ast::models::ExtractedAstFile;
use std::path::Path;

/// Contrato extensível para extração de assinaturas e tipos de AST de código-fonte.
pub trait AstExtractor: Send + Sync {
    /// Extrai assinaturas públicas a partir do conteúdo textual de um arquivo em memória.
    fn extract_from_str(&self, file_path: &Path, content: &str) -> Result<String, AstError>;

    /// Lê o arquivo do disco e extrai seu sumário de assinaturas.
    fn extract_from_file(&self, file_path: &Path) -> Result<ExtractedAstFile, AstError>;

    /// Varre recursivamente um diretório aplicando filtros de exclusão e extrai o AST de todos os arquivos suportados.
    fn extract_from_dir(
        &self,
        root: &Path,
        ignore_dirs: &[&str],
    ) -> Result<Vec<ExtractedAstFile>, AstError>;
}
