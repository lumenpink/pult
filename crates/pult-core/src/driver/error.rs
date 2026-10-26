use thiserror::Error;

/// Erros decorrentes de operações com CCCP e Git.
#[derive(Debug, Error)]
pub enum DriverError {
    #[error("Erro de E/S ao executar comando: {0}")]
    Io(#[from] std::io::Error),

    #[error("Comando '{command}' falhou com código {exit_code:?}: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Cabeçalho do commit excede limite de {max} caracteres (tamanho atual: {length})")]
    HeaderTooLong { length: usize, max: usize },

    #[error("A Fase {0} do GOST não autoriza commits normativos")]
    PhaseDoesNotAllowCommit(u8),

    #[error(
        "Veto da Fase 3/4 da GOST: arquivo de lógica '{file}' não pode ser alterado ({reason})"
    )]
    Phase3Veto { file: String, reason: String },

    #[error("Nome de branch inválido: {0}")]
    InvalidBranchName(String),
}
