use crate::domain::gost::GostPhase;
use crate::driver::error::DriverError;
use crate::driver::models::{GitStatus, QualityAudit};
use std::path::{Path, PathBuf};

/// Contrato soberano para execução de operações no repositório (Git / CCCP).
pub trait RepoDriver: Send + Sync {
    /// Obtém o nome da branch Git atualmente ativa.
    fn current_branch(&self) -> Result<String, DriverError>;

    /// Faz checkout de uma branch existente ou cria uma nova com base no branch atual.
    fn checkout_branch(&self, branch: &str, create: bool) -> Result<(), DriverError>;

    /// Audita o status da árvore de trabalho (arquivos modificados, untracked, clean).
    fn status(&self) -> Result<GitStatus, DriverError>;

    /// Executa a auditoria de conformidade estrita do Gosplan (`cccp check --strict`).
    fn check_strict(&self) -> Result<QualityAudit, DriverError>;

    /// Formata uma mensagem canônica de commit GOST para a fase informada.
    fn format_commit_message(
        &self,
        phase: GostPhase,
        ticket_id: u32,
        scope: Option<&str>,
        subject: &str,
        body: Option<&str>,
    ) -> Result<String, DriverError>;

    /// Valida o diff atual contra as restrições da fase GOST ativa.
    fn validate_phase_diff(&self, phase: GostPhase) -> Result<(), DriverError>;

    /// Executa o commit canônico associado à fase corrente do GOST.
    fn commit_phase(
        &self,
        phase: GostPhase,
        ticket_id: u32,
        scope: Option<&str>,
        subject: &str,
        body: Option<&str>,
    ) -> Result<String, DriverError>;

    /// Executa o merge com validação de tag de fechamento (`cccp merge <branch>`).
    fn merge_branch(&self, branch: &str) -> Result<(), DriverError>;
}

/// Implementação padrão que delega ao binário soberano `cccp` e comandos Git nativos.
pub struct CccpGitDriver {
    pub workspace_root: PathBuf,
}

impl CccpGitDriver {
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
        }
    }
}

impl RepoDriver for CccpGitDriver {
    fn current_branch(&self) -> Result<String, DriverError> {
        todo!("Fase 5: Implementação de current_branch")
    }

    fn checkout_branch(&self, _branch: &str, _create: bool) -> Result<(), DriverError> {
        todo!("Fase 5: Implementação de checkout_branch")
    }

    fn status(&self) -> Result<GitStatus, DriverError> {
        todo!("Fase 5: Implementação de status")
    }

    fn check_strict(&self) -> Result<QualityAudit, DriverError> {
        todo!("Fase 5: Implementação de check_strict")
    }

    fn format_commit_message(
        &self,
        _phase: GostPhase,
        _ticket_id: u32,
        _scope: Option<&str>,
        _subject: &str,
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        todo!("Fase 5: Implementação de format_commit_message")
    }

    fn validate_phase_diff(&self, _phase: GostPhase) -> Result<(), DriverError> {
        todo!("Fase 5: Implementação de validate_phase_diff")
    }

    fn commit_phase(
        &self,
        _phase: GostPhase,
        _ticket_id: u32,
        _scope: Option<&str>,
        _subject: &str,
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        todo!("Fase 5: Implementação de commit_phase")
    }

    fn merge_branch(&self, _branch: &str) -> Result<(), DriverError> {
        todo!("Fase 5: Implementação de merge_branch")
    }
}
