use pult_core::domain::{Actor, GostPhase, Session, TurnStatus};
use pult_tui::app::App;

#[test]
fn test_agent_directive_overrides_global_gemini_rules_and_has_no_gost_preaching() {
    for n in 1..=9 {
        let phase = GostPhase::from_u8(n).expect("valid phase");
        let directive = phase.agent_directive("feat/25-automation");

        // 1. Deve conter o cabeçalho imperativo de anulação de escopo
        assert!(
            directive.contains("[DIRETIVA DE OPERAÇÃO - SUBAGENTE DE EXECUÇÃO PULT]"),
            "Fase {} deve conter o cabeçalho de isolamento",
            n
        );
        assert!(
            directive.contains("Anulação de escopo")
                && directive.contains("Você NÃO é o maestro do ciclo"),
            "Fase {} deve cancelar explicitamente regras globais de orquestração",
            n
        );

        // 2. Não deve conter pregações ou roadmaps sobre fases futuras
        assert!(
            !directive.contains("próximas fases ciclo gost"),
            "Fase {} não deve conter pregação sobre fases futuras",
            n
        );
    }
}

#[test]
fn test_history_context_formatting_prepends_previous_turns() {
    let mut session = Session::new_draft("test-slug");
    session.add_turn(
        Actor::User,
        "Podemos implementar rate limiting?".to_string(),
        TurnStatus::Completed,
    );
    session.add_turn(
        Actor::Agy,
        "Sim, sugiro algoritmo de token bucket.".to_string(),
        TurnStatus::Completed,
    );

    let formatted = pult_tui::app::format_prompt_with_history(&session, "y", "/* AST */");

    assert!(formatted.contains("--- Contexto Histórico Recente da Sessão ---"));
    assert!(formatted.contains("[Usuária]: Podemos implementar rate limiting?"));
    assert!(formatted.contains("[Agente]: Sim, sugiro algoritmo de token bucket."));
    assert!(formatted.contains("[Mensagem Atual da Usuária]:\ny"));
    assert!(formatted.contains("/* AST */"));
}

#[test]
fn test_ui_header_displays_branch_leaf_icon() {
    let mut app = App::new();
    app.set_ticket_info(Some(25), "feat/25-automation", 1);
    let span_text = app.header_branch_label();
    assert!(span_text.contains("🌿 feat/25-automation"));
}

struct MockAutomatedPhase2Driver {
    pub current_branch_name: String,
    pub checkout_called: std::sync::Arc<std::sync::Mutex<Vec<(String, bool)>>>,
    pub ticket_created: std::sync::Arc<std::sync::Mutex<Option<(String, String)>>>,
}

impl pult_core::RepoDriver for MockAutomatedPhase2Driver {
    fn current_branch(&self) -> Result<String, pult_core::DriverError> {
        Ok(self.current_branch_name.clone())
    }
    fn checkout_branch(&self, branch: &str, create: bool) -> Result<(), pult_core::DriverError> {
        self.checkout_called
            .lock()
            .unwrap()
            .push((branch.to_string(), create));
        Ok(())
    }
    fn status(&self) -> Result<pult_core::GitStatus, pult_core::DriverError> {
        Ok(pult_core::GitStatus {
            branch: self.current_branch_name.clone(),
            is_clean: true,
            modified_files: Vec::new(),
            untracked_files: Vec::new(),
        })
    }
    fn check_strict(&self) -> Result<pult_core::QualityAudit, pult_core::DriverError> {
        Ok(pult_core::QualityAudit {
            passed: true,
            total_checks: 10,
            conforming: 10,
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
    ) -> Result<String, pult_core::DriverError> {
        Ok("commit".into())
    }
    fn validate_phase_diff(&self, _phase: GostPhase) -> Result<(), pult_core::DriverError> {
        Ok(())
    }
    fn commit_phase(
        &self,
        _phase: GostPhase,
        _ticket_id: u32,
        _scope: Option<&str>,
        _subject: &str,
        _body: Option<&str>,
    ) -> Result<String, pult_core::DriverError> {
        Ok("hash".into())
    }
    fn merge_branch(&self, _branch: &str) -> Result<(), pult_core::DriverError> {
        Ok(())
    }
    fn close_ticket(&self, _ticket_id: u32) -> Result<(), pult_core::DriverError> {
        Ok(())
    }
    fn create_ticket(
        &self,
        title: &str,
        body: &str,
    ) -> Result<(u32, String), pult_core::DriverError> {
        *self.ticket_created.lock().unwrap() = Some((title.to_string(), body.to_string()));
        Ok((42, "feat/42-rate-limiting".to_string()))
    }
}

#[test]
fn test_phase1_approve_automates_phase2_transition_to_phase3() {
    let mut app = App::new();
    app.set_gost_phase(GostPhase::Ticket);

    let mut session = Session::new_draft("rate-limiting");
    session.add_turn(
        Actor::User,
        "Implementar rate limiting via token bucket".to_string(),
        TurnStatus::Completed,
    );
    app.set_session(session);

    let checkout_called = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let ticket_created = std::sync::Arc::new(std::sync::Mutex::new(None));

    let driver = MockAutomatedPhase2Driver {
        current_branch_name: "main".to_string(),
        checkout_called: std::sync::Arc::clone(&checkout_called),
        ticket_created: std::sync::Arc::clone(&ticket_created),
    };

    let res = app.execute_action(pult_tui::event::TuiAction::SpecApproved, &driver);
    assert!(
        res.is_ok(),
        "Aprovação da Fase 1 deve automatizar criação e avançar: {:?}",
        res
    );

    // Valida que o ticket foi sintetizado e criado
    let created = ticket_created.lock().unwrap();
    assert!(created.is_some());
    let (title, body) = created.as_ref().unwrap();
    assert!(title.contains("rate limiting"));
    assert!(body.contains("DD-TDD Checklist"));

    // Valida que a branch foi criada e checkout realizado
    let checkouts = checkout_called.lock().unwrap();
    assert_eq!(checkouts.len(), 1);
    assert_eq!(checkouts[0].0, "feat/42-rate-limiting");
    assert!(checkouts[0].1); // create = true

    // Valida que a fase pulou automaticamente para a Fase 3
    assert_eq!(app.phase(), GostPhase::SpecContract);
    assert_eq!(app.ticket_id(), Some(42));
    assert_eq!(app.branch_name(), "feat/42-rate-limiting");
}
