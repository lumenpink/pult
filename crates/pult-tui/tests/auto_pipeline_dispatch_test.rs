use pult_core::domain::GostPhase;
use pult_core::driver::{DriverError, GitStatus, QualityAudit, RepoDriver};
use pult_tui::app::App;
use pult_tui::event::TuiAction;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct MockDispatchDriver {
    pub current_branch_name: String,
    pub audit_failures: usize,
    pub audit_passed: bool,
    pub committed_phases: Arc<std::sync::Mutex<Vec<(GostPhase, String)>>>,
    pub ticket_closed: Arc<AtomicBool>,
}

impl RepoDriver for MockDispatchDriver {
    fn current_branch(&self) -> Result<String, DriverError> {
        Ok(self.current_branch_name.clone())
    }

    fn checkout_branch(&self, _branch: &str, _create: bool) -> Result<(), DriverError> {
        Ok(())
    }

    fn merge_branch(&self, _branch: &str) -> Result<(), DriverError> {
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
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        let msg = self.format_commit_message(phase, ticket_id, scope, subject, _body)?;
        self.committed_phases.lock().unwrap().push((phase, msg.clone()));
        Ok(msg)
    }

    fn create_ticket(&self, _title: &str, _body: &str) -> Result<(u32, String), DriverError> {
        Ok((26, "feat/26-auto-dispatch".to_string()))
    }

    fn close_ticket(&self, _ticket_id: u32) -> Result<(), DriverError> {
        self.ticket_closed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn test_phase_prompt_synthesis() {
    let prompt_p3 = App::phase_prompt_for(GostPhase::SpecContract, Some(26), "feat/26-auto-dispatch");
    assert!(prompt_p3.contains("[DIRETIVA DE OPERAÇÃO - SUBAGENTE DE EXECUÇÃO PULT]"));
    assert!(prompt_p3.contains("#26"));
    assert!(prompt_p3.contains("docs/specs/"));

    let prompt_p4 = App::phase_prompt_for(GostPhase::RedTest, Some(26), "feat/26-auto-dispatch");
    assert!(prompt_p4.contains("[DIRETIVA DE OPERAÇÃO - SUBAGENTE DE EXECUÇÃO PULT]"));
    assert!(prompt_p4.contains("#26"));
    assert!(prompt_p4.contains("tests/"));

    let prompt_p5 = App::phase_prompt_for(GostPhase::GreenCode, Some(26), "feat/26-auto-dispatch");
    assert!(prompt_p5.contains("[DIRETIVA DE OPERAÇÃO - SUBAGENTE DE EXECUÇÃO PULT]"));
    assert!(prompt_p5.contains("#26"));
    assert!(prompt_p5.contains("src/"));
}

#[test]
fn test_approve_phase1_dispatches_phase3_prompt() {
    let committed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ticket_closed = Arc::new(AtomicBool::new(false));

    let driver = MockDispatchDriver {
        current_branch_name: "main".to_string(),
        audit_failures: 0,
        audit_passed: true,
        committed_phases: committed.clone(),
        ticket_closed: ticket_closed.clone(),
    };

    let mut app = App::new();
    app.set_gost_phase(GostPhase::Ticket);

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 1 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::SpecContract);

    let pending = app.take_pending_dispatch();
    assert!(pending.is_some(), "Deveria haver prompt gerado para despacho na Fase 3");
    let p = pending.unwrap();
    assert!(p.contains("#26"));
    assert!(p.contains("docs/specs/"));
}

#[test]
fn test_approve_phase3_dispatches_phase4_prompt() {
    let committed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ticket_closed = Arc::new(AtomicBool::new(false));

    let driver = MockDispatchDriver {
        current_branch_name: "feat/26-auto-dispatch".to_string(),
        audit_failures: 0,
        audit_passed: true,
        committed_phases: committed.clone(),
        ticket_closed: ticket_closed.clone(),
    };

    let mut app = App::new();
    app.set_ticket_info(Some(26), "feat/26-auto-dispatch", 3);
    app.set_gost_phase(GostPhase::SpecContract);

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 3 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::RedTest);

    let pending = app.take_pending_dispatch();
    assert!(pending.is_some(), "Deveria haver prompt gerado para despacho na Fase 4");
    let p = pending.unwrap();
    assert!(p.contains("#26"));
    assert!(p.contains("tests/"));

    let commits = committed.lock().unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].0, GostPhase::SpecContract);
}

#[test]
fn test_approve_phase4_dispatches_phase5_prompt() {
    let committed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ticket_closed = Arc::new(AtomicBool::new(false));

    let driver = MockDispatchDriver {
        current_branch_name: "feat/26-auto-dispatch".to_string(),
        audit_failures: 0,
        audit_passed: true,
        committed_phases: committed.clone(),
        ticket_closed: ticket_closed.clone(),
    };

    let mut app = App::new();
    app.set_ticket_info(Some(26), "feat/26-auto-dispatch", 4);
    app.set_gost_phase(GostPhase::RedTest);

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 4 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::GreenCode);

    let pending = app.take_pending_dispatch();
    assert!(pending.is_some(), "Deveria haver prompt gerado para despacho na Fase 5");
    let p = pending.unwrap();
    assert!(p.contains("#26"));
    assert!(p.contains("src/"));

    let commits = committed.lock().unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].0, GostPhase::RedTest);
}

#[test]
fn test_approve_phase5_chains_autonomously_to_phase9() {
    let committed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ticket_closed = Arc::new(AtomicBool::new(false));

    let driver = MockDispatchDriver {
        current_branch_name: "feat/26-auto-dispatch".to_string(),
        audit_failures: 0,
        audit_passed: true,
        committed_phases: committed.clone(),
        ticket_closed: ticket_closed.clone(),
    };

    let mut app = App::new();
    app.set_ticket_info(Some(26), "feat/26-auto-dispatch", 5);
    app.set_gost_phase(GostPhase::GreenCode);

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Phase 5 approve failed: {:?}", res);
    assert_eq!(app.phase(), GostPhase::CloseTicket);

    let pending = app.take_pending_dispatch();
    assert!(pending.is_none(), "Fase 9 não despacha para agy, aguarda usuária");

    let commits = committed.lock().unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].0, GostPhase::GreenCode);
}

#[test]
fn test_approve_phase5_stops_at_phase8_if_quality_gate_fails() {
    let committed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let ticket_closed = Arc::new(AtomicBool::new(false));

    let driver = MockDispatchDriver {
        current_branch_name: "feat/26-auto-dispatch".to_string(),
        audit_failures: 2,
        audit_passed: false,
        committed_phases: committed.clone(),
        ticket_closed: ticket_closed.clone(),
    };

    let mut app = App::new();
    app.set_ticket_info(Some(26), "feat/26-auto-dispatch", 5);
    app.set_gost_phase(GostPhase::GreenCode);

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_err(), "Deveria falhar devido ao bloqueio de qualidade na Fase 8");
    assert_eq!(app.phase(), GostPhase::QualityGate);

    let pending = app.take_pending_dispatch();
    assert!(pending.is_none());
}
