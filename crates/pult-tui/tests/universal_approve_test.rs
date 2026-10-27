use pult_core::domain::GostPhase;
use pult_core::driver::{DriverError, GitStatus, QualityAudit, RepoDriver};
use pult_tui::app::App;
use pult_tui::event::TuiAction;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct MockUniversalDriver {
    pub current_branch_name: String,
    pub audit_failures: usize,
    pub audit_passed: bool,
    pub committed_phases: Arc<std::sync::Mutex<Vec<(GostPhase, String)>>>,
    pub ticket_closed: Arc<AtomicBool>,
}

impl RepoDriver for MockUniversalDriver {
    fn current_branch(&self) -> Result<String, DriverError> {
        Ok(self.current_branch_name.clone())
    }

    fn checkout_branch(&self, _branch: &str, _create: bool) -> Result<(), DriverError> {
        Ok(())
    }

    fn status(&self) -> Result<GitStatus, DriverError> {
        Ok(GitStatus {
            branch: self.current_branch_name.clone(),
            is_clean: true,
            modified_files: Vec::new(),
            untracked_files: Vec::new(),
        })
    }

    fn check_strict(&self) -> Result<QualityAudit, DriverError> {
        Ok(QualityAudit {
            passed: self.audit_passed,
            total_checks: 10,
            conforming: 10usize.saturating_sub(self.audit_failures),
            warnings: 0,
            failures: self.audit_failures,
            compliance_index: if self.audit_failures == 0 {
                "100%".into()
            } else {
                "70%".into()
            },
        })
    }

    fn format_commit_message(
        &self,
        phase: GostPhase,
        ticket_id: u32,
        scope: Option<&str>,
        subject: &str,
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        let tag = if phase == GostPhase::CloseTicket {
            format!("(closes #{} - ph9)", ticket_id)
        } else {
            format!("(ref #{} - ph{})", ticket_id, phase.number())
        };
        Ok(format!(
            "{}({}): {} {}",
            phase.commit_prefix().unwrap_or("feat"),
            scope.unwrap_or("tui"),
            subject,
            tag
        ))
    }

    fn validate_phase_diff(&self, _phase: GostPhase) -> Result<(), DriverError> {
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
        let msg = self.format_commit_message(phase, ticket_id, scope, subject, body)?;
        self.committed_phases
            .lock()
            .unwrap()
            .push((phase, msg.clone()));
        Ok(msg)
    }

    fn merge_branch(&self, _branch: &str) -> Result<(), DriverError> {
        Ok(())
    }

    fn close_ticket(&self, _ticket_id: u32) -> Result<(), DriverError> {
        self.ticket_closed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn test_universal_approve_through_all_phases() {
    let mut app = App::new();
    app.set_ticket_info(Some(24), "feat/24-universal-approve", 1);

    let committed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ticket_closed = Arc::new(AtomicBool::new(false));

    let driver = MockUniversalDriver {
        current_branch_name: "feat/24-universal-approve".into(),
        audit_failures: 0,
        audit_passed: true,
        committed_phases: Arc::clone(&committed),
        ticket_closed: Arc::clone(&ticket_closed),
    };

    // Phase 1 -> Phase 2
    app.set_gost_phase(GostPhase::Ticket);
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 1 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::NewBranch);

    // Phase 2 -> Phase 3
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 2 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::SpecContract);

    // Phase 3 -> Phase 4 (deve commitar ph3)
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 3 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::RedTest);

    // Phase 4 -> Phase 5 (deve commitar ph4)
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 4 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::GreenCode);

    // Phase 5 -> Phase 6 (deve commitar ph5)
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 5 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::Refactor);

    // Phase 6 -> Phase 7
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 6 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::MutationTest);

    // Phase 7 -> Phase 8
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 7 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::QualityGate);

    // Phase 8 -> Phase 9 (com 0 falhas, deve avançar)
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 8 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::CloseTicket);

    // Phase 9 -> Close (deve commitar ph9 e fechar ticket)
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 9 approve failed: {:?}", res);
    assert!(ticket_closed.load(Ordering::SeqCst));

    let commits = committed.lock().unwrap();
    assert_eq!(
        commits.len(),
        4,
        "Devem existir commits para as Fases 3, 4, 5 e 9"
    );
    assert!(commits[0].1.contains("(ref #24 - ph3)"));
    assert!(commits[1].1.contains("(ref #24 - ph4)"));
    assert!(commits[2].1.contains("(ref #24 - ph5)"));
    assert!(commits[3].1.contains("(closes #24 - ph9)"));
}

#[test]
fn test_quality_gate_lock_blocks_advancement_on_check_failures() {
    let mut app = App::new();
    app.set_ticket_info(Some(24), "feat/24-universal-approve", 8);
    app.set_gost_phase(GostPhase::QualityGate);

    let committed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ticket_closed = Arc::new(AtomicBool::new(false));

    let locked_driver = MockUniversalDriver {
        current_branch_name: "feat/24-universal-approve".into(),
        audit_failures: 2, // 2 falhas apontadas pelo cccp check
        audit_passed: false,
        committed_phases: Arc::clone(&committed),
        ticket_closed: Arc::clone(&ticket_closed),
    };

    let res = app.execute_action(TuiAction::SpecApproved, &locked_driver);
    assert!(
        res.is_err(),
        "Avanço deveria ser bloqueado com failures > 0"
    );
    let err_msg = res.unwrap_err();
    assert!(err_msg.contains("GOSPLAN QUALITY CONTROL") || err_msg.contains("Falhas"));
    assert_eq!(
        app.phase(),
        GostPhase::QualityGate,
        "Fase deve permanecer em QualityGate"
    );
}
