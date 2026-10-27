use pult_core::domain::{GostPhase, Session};
use pult_core::driver::{DriverError, GitStatus, QualityAudit, RepoDriver};
use pult_tui::app::App;
use pult_tui::event::TuiAction;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct MockRepoDriver {
    pub current_branch_name: String,
    pub modified_files: Vec<String>,
    pub ticket_closed: Arc<AtomicBool>,
    pub branch_merged: Arc<AtomicBool>,
}

impl RepoDriver for MockRepoDriver {
    fn current_branch(&self) -> Result<String, DriverError> {
        Ok(self.current_branch_name.clone())
    }

    fn checkout_branch(&self, _branch: &str, _create: bool) -> Result<(), DriverError> {
        Ok(())
    }

    fn status(&self) -> Result<GitStatus, DriverError> {
        Ok(GitStatus {
            branch: self.current_branch_name.clone(),
            is_clean: self.modified_files.is_empty(),
            modified_files: self.modified_files.clone(),
            untracked_files: Vec::new(),
        })
    }

    fn check_strict(&self) -> Result<QualityAudit, DriverError> {
        Ok(QualityAudit {
            passed: true,
            total_checks: 9,
            conforming: 9,
            warnings: 0,
            failures: 0,
            compliance_index: "100%".into(),
        })
    }

    fn format_commit_message(
        &self,
        _phase: GostPhase,
        _ticket_id: u32,
        _scope: Option<&str>,
        _subject: &str,
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        Ok("commit message".into())
    }

    fn validate_phase_diff(&self, phase: GostPhase) -> Result<(), DriverError> {
        if phase == GostPhase::SpecContract {
            for f in &self.modified_files {
                if f.starts_with("src/") {
                    return Err(DriverError::Phase3Veto {
                        file: f.clone(),
                        reason: "Modificação em src/ proibida na Fase 3".into(),
                    });
                }
            }
        }
        Ok(())
    }

    fn commit_phase(
        &self,
        _phase: GostPhase,
        _ticket_id: u32,
        _scope: Option<&str>,
        _subject: &str,
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        Ok("commit".into())
    }

    fn merge_branch(&self, _branch: &str) -> Result<(), DriverError> {
        self.branch_merged.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn close_ticket(&self, _ticket_id: u32) -> Result<(), DriverError> {
        self.ticket_closed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn test_execute_action_spec_approved_veto_on_src_modification() {
    let mut app = App::new();
    app.set_gost_phase(GostPhase::SpecContract);

    let driver = MockRepoDriver {
        current_branch_name: "feat/18-guard".into(),
        modified_files: vec!["src/lib.rs".into()],
        ticket_closed: Arc::new(AtomicBool::new(false)),
        branch_merged: Arc::new(AtomicBool::new(false)),
    };

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_err(), "deve vetar avanço com src/ modificado");
    assert_eq!(app.phase(), GostPhase::SpecContract);

    let vp_text = app.viewport_lines.join("\n");
    assert!(vp_text.contains("Veto"));
}

#[test]
fn test_execute_action_spec_approved_success_advances_phase() {
    let mut app = App::new();
    app.set_gost_phase(GostPhase::SpecContract);

    let mut session = Session::new_formal(18, "guard-test", "feat/18-guard");
    session.metadata.gost.phase = 3;
    app.set_session(session);

    let driver = MockRepoDriver {
        current_branch_name: "feat/18-guard".into(),
        modified_files: vec!["docs/specs/test.md".into()],
        ticket_closed: Arc::new(AtomicBool::new(false)),
        branch_merged: Arc::new(AtomicBool::new(false)),
    };

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "deve aprovar quando src/ estiver limpo");
    assert_eq!(app.phase(), GostPhase::RedTest);

    // Valida que atualizou a sessão
    let lock = app.session.lock().unwrap();
    assert_eq!(lock.as_ref().unwrap().metadata.gost.phase, 4);
}

#[test]
fn test_execute_action_finish_and_merge() {
    let mut app = App::new();
    app.set_ticket_info(Some(18), "feat/18-guard", 9);

    let closed_flag = Arc::new(AtomicBool::new(false));
    let merged_flag = Arc::new(AtomicBool::new(false));

    let driver = MockRepoDriver {
        current_branch_name: "feat/18-guard".into(),
        modified_files: Vec::new(),
        ticket_closed: closed_flag.clone(),
        branch_merged: merged_flag.clone(),
    };

    // Testa Finish (Ctrl+G)
    let res_finish = app.execute_action(TuiAction::FinishTriggered, &driver);
    assert!(res_finish.is_ok());
    assert!(closed_flag.load(Ordering::SeqCst));
    let vp_text = app.viewport_lines.join("\n");
    assert!(vp_text.contains("Crônica da Entrega"));

    // Testa Merge (Ctrl+M)
    let res_merge = app.execute_action(TuiAction::MergeTriggered, &driver);
    assert!(res_merge.is_ok());
    assert!(merged_flag.load(Ordering::SeqCst));
}

#[test]
fn test_mutation_shortcuts_invalid_phases_and_merge_failure() {
    let mut app = App::new();
    app.set_gost_phase(GostPhase::Ticket);

    struct FailingMergeDriver;
    impl RepoDriver for FailingMergeDriver {
        fn current_branch(&self) -> Result<String, DriverError> {
            Ok("main".into())
        }
        fn checkout_branch(&self, _b: &str, _c: bool) -> Result<(), DriverError> {
            Ok(())
        }
        fn status(&self) -> Result<GitStatus, DriverError> {
            Ok(GitStatus {
                branch: "main".into(),
                is_clean: true,
                modified_files: Vec::new(),
                untracked_files: Vec::new(),
            })
        }
        fn check_strict(&self) -> Result<QualityAudit, DriverError> {
            Ok(QualityAudit {
                passed: true,
                total_checks: 9,
                conforming: 9,
                warnings: 0,
                failures: 0,
                compliance_index: "100%".into(),
            })
        }
        fn format_commit_message(
            &self,
            _p: GostPhase,
            _t: u32,
            _s: Option<&str>,
            _sub: &str,
            _b: Option<&str>,
        ) -> Result<String, DriverError> {
            Ok("msg".into())
        }
        fn validate_phase_diff(&self, _p: GostPhase) -> Result<(), DriverError> {
            Ok(())
        }
        fn commit_phase(
            &self,
            _p: GostPhase,
            _t: u32,
            _s: Option<&str>,
            _sub: &str,
            _b: Option<&str>,
        ) -> Result<String, DriverError> {
            Ok("msg".into())
        }
        fn merge_branch(&self, branch: &str) -> Result<(), DriverError> {
            Err(DriverError::CommandFailed {
                command: "cccp merge".into(),
                exit_code: Some(1),
                stderr: format!("conflito ao mesclar {}", branch),
            })
        }
        fn close_ticket(&self, _id: u32) -> Result<(), DriverError> {
            Ok(())
        }
    }

    let driver = FailingMergeDriver;

    // Mutante 1: approve_spec_contract direto fora da Fase 3
    let res_spec = app.approve_spec_contract();
    assert!(res_spec.is_err());
    assert_eq!(app.phase(), GostPhase::Ticket);

    // Mutante 2: Falha no merge é tratada e gera erro no retorno e viewport
    let res_merge = app.execute_action(TuiAction::MergeTriggered, &driver);
    assert!(res_merge.is_err());
    let vp_text = app.viewport_lines.join("\n");
    assert!(vp_text.contains("Falha ao mesclar"));
}
