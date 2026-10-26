use crate::domain::error::SessionError;
use std::path::PathBuf;
use thiserror::Error;

/// Erros na execução e orquestração do runner Antigravity.
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Erro de E/S ao executar processo: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Falha no processo do runner: {0}")]
    ProcessFailed(String),

    #[error("Erro de persistência de sessão: {0}")]
    Session(#[from] SessionError),

    #[error("Arquivo de sessão não encontrado: {0}")]
    SessionNotFound(PathBuf),
}
