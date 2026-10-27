use pult_core::domain::GostPhase;
use pult_core::driver::models::{GitStatus, QualityAudit};
use pult_core::driver::RepoDriver;
use pult_core::DriverError;
use pult_tui::app::{App, SLASH_COMMANDS};
use pult_tui::event::TuiAction;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct MockDriver {
    pub current_branch_name: Arc<Mutex<String>>,
    pub check_strict_called: Arc<AtomicUsize>,
    pub checkout_called: Arc<Mutex<Vec<(String, bool)>>>,
}

impl RepoDriver for MockDriver {
    fn current_branch(&self) -> Result<String, DriverError> {
        Ok(self.current_branch_name.lock().unwrap().clone())
    }

    fn checkout_branch(&self, branch: &str, create: bool) -> Result<(), DriverError> {
        self.checkout_called
            .lock()
            .unwrap()
            .push((branch.to_string(), create));
        *self.current_branch_name.lock().unwrap() = branch.to_string();
        Ok(())
    }

    fn status(&self) -> Result<GitStatus, DriverError> {
        Ok(GitStatus {
            branch: self.current_branch_name.lock().unwrap().clone(),
            is_clean: true,
            modified_files: vec![],
            untracked_files: vec![],
        })
    }

    fn check_strict(&self) -> Result<QualityAudit, DriverError> {
        self.check_strict_called.fetch_add(1, Ordering::SeqCst);
        Ok(QualityAudit {
            passed: true,
            total_checks: 12,
            conforming: 12,
            warnings: 0,
            failures: 0,
            compliance_index: "100%".to_string(),
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
        Ok("commit msg".to_string())
    }

    fn commit_phase(
        &self,
        _phase: GostPhase,
        _ticket_id: u32,
        _scope: Option<&str>,
        _subject: &str,
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        Ok("hash123".to_string())
    }

    fn validate_phase_diff(&self, _phase: GostPhase) -> Result<(), DriverError> {
        Ok(())
    }

    fn close_ticket(&self, _id: u32) -> Result<(), DriverError> {
        Ok(())
    }

    fn merge_branch(&self, _branch: &str) -> Result<(), DriverError> {
        Ok(())
    }
}

#[test]
fn test_agent_directives_contain_git_cccp_prohibition() {
    for n in 1..=9 {
        let phase = GostPhase::from_u8(n).expect("valid phase");
        let directive = phase.agent_directive("feat/22-orchestration");
        assert!(
            directive.contains("PROIBIÇÃO EXPRESSA"),
            "Fase {} deve conter PROIBIÇÃO EXPRESSA",
            n
        );
        assert!(
            directive.contains("git") && directive.contains("cccp"),
            "Fase {} deve citar proibição de comandos git e cccp",
            n
        );
        if n >= 3 {
            assert!(
                directive.contains("feat/22-orchestration"),
                "Fases >= 3 devem autorizar na branch de trabalho"
            );
        }
    }
}

#[test]
fn test_main_branch_protection_guards_against_code_phases() {
    let mut app = App::new();
    app.branch_name = "main".to_string();
    app.gost_phase = GostPhase::Ticket;

    let driver = MockDriver::default();

    // Na Fase 1 (Ticket), avançar para Fase 2 (NewBranch) na main é permitido
    let res = app.execute_action(TuiAction::PhaseAdvanced(GostPhase::NewBranch), &driver);
    assert!(res.is_ok());

    // Mas tentar avançar para Fase 3 (SpecContract) diretamente na main DEVE ser vetado!
    let res_phase3 = app.execute_action(TuiAction::PhaseAdvanced(GostPhase::SpecContract), &driver);
    assert!(
        res_phase3.is_err(),
        "Avanço para Fase 3 na main deve ser bloqueado"
    );
    assert!(res_phase3.unwrap_err().contains("main"));
}

#[test]
fn test_branch_slash_command_and_checkout() {
    assert!(SLASH_COMMANDS.contains(&"/branch"));

    let mut app = App::new();
    app.branch_name = "main".to_string();

    let driver = MockDriver::default();

    // Executa /branch feat/22-nova-feature
    let action = app.execute_slash_command("/branch feat/22-nova-feature");
    assert!(matches!(action, Some(TuiAction::BranchChanged(_))));

    if let Some(act) = action {
        let res = app.execute_action(act, &driver);
        assert!(res.is_ok());
        assert_eq!(app.branch_name, "feat/22-nova-feature");
        assert_eq!(app.ticket_id, Some(22));
        assert_eq!(driver.checkout_called.lock().unwrap().len(), 1);
        assert_eq!(
            driver.checkout_called.lock().unwrap()[0],
            ("feat/22-nova-feature".to_string(), true)
        );
    }
}

#[test]
fn test_quality_check_triggered_on_phase_transitions() {
    let mut app = App::new();
    app.branch_name = "feat/22-nova-feature".to_string();
    app.set_gost_phase(GostPhase::RedTest);

    let driver = MockDriver::default();

    let res = app.execute_action(TuiAction::PhaseAdvanced(GostPhase::GreenCode), &driver);
    assert!(res.is_ok());

    // Deve ter chamado check_strict e registrado conformidade
    assert_eq!(driver.check_strict_called.load(Ordering::SeqCst), 1);
    let last_logs = app.viewport_lines.join("\n");
    assert!(last_logs.contains("Conformidade"));
}

#[test]
fn test_mutation_master_branch_protection_and_spec_approved() {
    let mut app = App::new();
    app.branch_name = "master".to_string();
    app.set_gost_phase(GostPhase::SpecContract);

    let driver = MockDriver::default();

    // Aprovar spec diretamente na master DEVE ser vetado pelo bloqueio de governança
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_err());
    let err_msg = res.unwrap_err();
    assert!(err_msg.contains("Bloqueio de Governança") && err_msg.contains("master"));
}

#[test]
fn test_mutation_branch_slash_command_edge_cases() {
    let mut app = App::new();

    // Comando /branch sem argumento
    let act_empty = app.execute_slash_command("/branch");
    assert!(act_empty.is_none());
    assert!(app.viewport_lines.last().unwrap().contains("Uso correto"));

    // Comando /branch com apenas espaços
    let act_spaces = app.execute_slash_command("/branch    ");
    assert!(act_spaces.is_none());
    assert!(app.viewport_lines.last().unwrap().contains("Uso correto"));

    // Autocomplete para prefixos de branch
    assert_eq!(
        pult_tui::app::autocomplete_slash_command("/b"),
        Some("/branch")
    );
    assert_eq!(
        pult_tui::app::autocomplete_slash_command("/br"),
        Some("/branch")
    );
    assert_eq!(
        pult_tui::app::autocomplete_slash_command("/branch"),
        Some("/branch")
    );
}

#[test]
fn test_mutation_all_nine_phase_directives_distinct_content() {
    let mut seen_directives = std::collections::HashSet::new();
    for n in 1..=9 {
        let phase = GostPhase::from_u8(n).expect("valid phase");
        let directive = phase.agent_directive("work-branch");
        assert!(!directive.trim().is_empty());
        assert!(seen_directives.insert(directive), "Diretiva da fase {} não deve ser duplicada", n);
    }
}

