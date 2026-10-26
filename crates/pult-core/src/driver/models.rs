use serde::{Deserialize, Serialize};

/// Resultado estruturado de uma auditoria de conformidade `cccp check --strict`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityAudit {
    pub passed: bool,
    pub total_checks: usize,
    pub conforming: usize,
    pub warnings: usize,
    pub failures: usize,
    pub compliance_index: String,
}

/// Estado da árvore de trabalho e do repositório Git.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub is_clean: bool,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
}

/// Detalhes de um ticket inspecionado ou gerenciado via `cccp ticket`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketDetails {
    pub id: u32,
    pub title: String,
    pub state: String,
    pub body: String,
}
