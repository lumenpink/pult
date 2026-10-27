use crate::domain::gost::GostPhase;
use crate::driver::error::DriverError;
use crate::driver::models::{GitStatus, QualityAudit};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Driver para execução de operações no repositório (Git / CCCP).
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

/// Implementação padrão que delega ao binário `cccp` e comandos Git nativos.
pub struct CccpGitDriver {
    pub workspace_root: PathBuf,
}

impl CccpGitDriver {
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
        }
    }

    fn run_git(&self, args: &[&str]) -> Result<String, DriverError> {
        let output = Command::new("git")
            .current_dir(&self.workspace_root)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(DriverError::CommandFailed {
                command: format!("git {}", args.join(" ")),
                exit_code: output.status.code(),
                stderr,
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn run_cccp(&self, args: &[&str]) -> Result<String, DriverError> {
        let output = Command::new("cccp")
            .current_dir(&self.workspace_root)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(DriverError::CommandFailed {
                command: format!("cccp {}", args.join(" ")),
                exit_code: output.status.code(),
                stderr,
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl RepoDriver for CccpGitDriver {
    fn current_branch(&self) -> Result<String, DriverError> {
        self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    fn checkout_branch(&self, branch: &str, create: bool) -> Result<(), DriverError> {
        if branch.trim().is_empty() {
            return Err(DriverError::InvalidBranchName(
                "Nome de branch não pode ser vazio".to_string(),
            ));
        }

        if create {
            self.run_git(&["checkout", "-b", branch])?;
        } else {
            self.run_git(&["checkout", branch])?;
        }

        Ok(())
    }

    fn status(&self) -> Result<GitStatus, DriverError> {
        let branch = self.current_branch()?;
        let porcelain = self.run_git(&["status", "--porcelain"])?;

        let mut modified_files = Vec::new();
        let mut untracked_files = Vec::new();

        for line in porcelain.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(stripped) = trimmed.strip_prefix("??") {
                untracked_files.push(stripped.trim().to_string());
            } else if trimmed.len() >= 3 {
                modified_files.push(trimmed[3..].trim().to_string());
            }
        }

        let is_clean = modified_files.is_empty() && untracked_files.is_empty();

        Ok(GitStatus {
            branch,
            is_clean,
            modified_files,
            untracked_files,
        })
    }

    fn check_strict(&self) -> Result<QualityAudit, DriverError> {
        let output = Command::new("cccp")
            .current_dir(&self.workspace_root)
            .args(["check", "--strict"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut total_checks = 0;
        let mut conforming = 0;
        let mut warnings = 0;
        let mut failures = 0;
        let mut compliance_index = "0%".to_string();

        for line in stdout.lines() {
            if line.contains("Total de verificações:") {
                for part in line.split('|') {
                    let part = part.trim();
                    if let Some(val) = part.strip_prefix("Total de verificações:") {
                        total_checks = val.trim().parse().unwrap_or(0);
                    } else if let Some(val) = part.strip_prefix("Conformes:") {
                        conforming = val.trim().parse().unwrap_or(0);
                    } else if let Some(val) = part.strip_prefix("Avisos:") {
                        warnings = val.trim().parse().unwrap_or(0);
                    } else if let Some(val) = part.strip_prefix("Falhas:") {
                        failures = val.trim().parse().unwrap_or(0);
                    }
                }
            } else if let Some(val) = line.strip_prefix("Índice de Conformidade Gosplan:") {
                compliance_index = val.trim().to_string();
            }
        }

        let passed = output.status.success() && failures == 0;

        Ok(QualityAudit {
            passed,
            total_checks,
            conforming,
            warnings,
            failures,
            compliance_index,
        })
    }

    fn format_commit_message(
        &self,
        phase: GostPhase,
        ticket_id: u32,
        scope: Option<&str>,
        subject: &str,
        body: Option<&str>,
    ) -> Result<String, DriverError> {
        let prefix = phase
            .commit_prefix()
            .ok_or_else(|| DriverError::PhaseDoesNotAllowCommit(phase.number()))?;

        let tag = if phase == GostPhase::CloseTicket {
            format!("(closes #{})", ticket_id)
        } else {
            format!("(ref #{})", ticket_id)
        };

        let header = match scope {
            Some(s) if !s.trim().is_empty() => {
                format!("{}({}): {} {}", prefix, s.trim(), subject.trim(), tag)
            }
            _ => format!("{}: {} {}", prefix, subject.trim(), tag),
        };

        if header.len() > 72 {
            return Err(DriverError::HeaderTooLong {
                length: header.len(),
                max: 72,
            });
        }

        let mut out = header;
        out.push_str("\n\n");
        out.push_str(&format!(
            "GOST: Fase {} ({})\n",
            phase.number(),
            phase.name()
        ));

        if let Some(b) = body {
            let b_trimmed = b.trim();
            if !b_trimmed.is_empty() {
                out.push('\n');
                out.push_str(b_trimmed);
                out.push('\n');
            }
        }

        Ok(out)
    }

    fn validate_phase_diff(&self, phase: GostPhase) -> Result<(), DriverError> {
        if phase.allows_production_code() {
            return Ok(());
        }

        // Fases 1 a 4 vetam código de produção executável em src/
        let status = self.status()?;
        let all_changed: Vec<&String> = status
            .modified_files
            .iter()
            .chain(status.untracked_files.iter())
            .collect();

        for file in all_changed {
            let is_in_src = file.contains("/src/") || file.starts_with("src/");
            let is_test = file.contains("/tests/") || file.starts_with("tests/");
            let is_doc = file.ends_with(".md") || file.contains("/docs/");

            if is_in_src && !is_test && !is_doc {
                // Permitir arquivos de contratos ou specs
                let is_spec = file.contains("contract")
                    || file.contains("spec")
                    || file.contains("domain/gost")
                    || file.contains("domain/error");
                if phase == GostPhase::SpecContract && !is_spec {
                    return Err(DriverError::Phase3Veto {
                        file: file.clone(),
                        reason: "Fase 3 veda alterações em lógica de produção fora de contratos"
                            .to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn commit_phase(
        &self,
        phase: GostPhase,
        ticket_id: u32,
        scope: Option<&str>,
        subject: &str,
        body: Option<&str>,
    ) -> Result<String, DriverError> {
        self.validate_phase_diff(phase)?;
        let msg = self.format_commit_message(phase, ticket_id, scope, subject, body)?;
        self.run_git(&["commit", "-m", &msg])?;
        Ok(msg)
    }

    fn merge_branch(&self, branch: &str) -> Result<(), DriverError> {
        self.run_cccp(&["merge", branch])?;
        Ok(())
    }
}
