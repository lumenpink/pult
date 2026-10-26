use thiserror::Error;

/// Erros de domínio e persistência do PULT.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Erro de E/S ao persistir sessão: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de serialização/deserialização YAML no frontmatter: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Erro ao interpretar arquivo de sessão Markdown: {0}")]
    Parse(String),

    #[error("Transição inválida de fase GOST: de Fase {from} para Fase {to}")]
    InvalidTransition { from: u8, to: u8 },

    #[error("Violação de guarda normativa da Fase {phase}: {reason}")]
    PhaseViolation { phase: u8, reason: String },

    #[error("Sessão não encontrada: {0}")]
    NotFound(String),
}
