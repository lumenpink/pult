use std::path::PathBuf;
use thiserror::Error;

/// Erros decorrentes de parsing e extração de AST estrutural.
#[derive(Debug, Error)]
pub enum AstError {
    #[error("Erro de E/S ao ler arquivo '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Falha ao analisar sintaxe em '{file}': {message}")]
    ParseError { file: String, message: String },

    #[error("Linguagem não suportada para o arquivo '{0}'")]
    UnsupportedLanguage(String),
}
