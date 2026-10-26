use crate::domain::TurnStatus;
use std::path::PathBuf;

/// Eventos transmitidos em tempo real pelo canal MPSC durante a execução do processo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgyStreamEvent {
    /// Pedaço de texto emitido no canal stdout.
    StdoutChunk(String),
    /// Pedaço de texto emitido no canal stderr.
    StderrChunk(String),
    /// Indicador de truncamento detectado no fluxo de saída.
    Truncated,
    /// Conclusão da execução com código de saída do processo.
    Completed { exit_code: i32 },
    /// Erro de execução ou falha do canal.
    Error(String),
}

/// Opções de configuração para o runner de subprocesso.
#[derive(Debug, Clone)]
pub struct RunnerOptions {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}

impl Default for RunnerOptions {
    fn default() -> Self {
        Self {
            command: "agy".to_string(),
            args: vec!["run".to_string()],
            working_dir: None,
        }
    }
}

/// Diagnóstico de inspeção para retomada de sessão (`--resume`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeInspection {
    pub session_id: String,
    pub needs_continuation: bool,
    pub branch_matches: bool,
    pub last_turn_status: Option<TurnStatus>,
}
