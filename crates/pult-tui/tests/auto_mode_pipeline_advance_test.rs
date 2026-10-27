use pult_core::domain::gost::GostPhase;
use pult_core::driver::{DriverError, GitStatus, QualityAudit, RepoDriver};
use pult_tui::app::App;
use pult_tui::event::TuiAction;

struct MockAutoDriver {
    current_branch_name: String,
}

impl RepoDriver for MockAutoDriver {
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
        self.format_commit_message(phase, ticket_id, scope, subject, body)
    }
    fn close_ticket(&self, _ticket_id: u32) -> Result<(), DriverError> {
        Ok(())
    }
    fn create_ticket(&self, _title: &str, _body: &str) -> Result<(u32, String), DriverError> {
        Ok((31, "feat/31-auto-pipeline".into()))
    }
    fn detect_branch_phase(
        &self,
        _branch: &str,
        _ticket_id: Option<u32>,
    ) -> Result<GostPhase, DriverError> {
        Ok(GostPhase::Ticket)
    }
}

#[test]
fn test_should_auto_advance_on_completed_rules() {
    let mut app = App::new();

    // 1. Quando is_auto_mode é false, nunca auto-avança em nenhuma fase
    app.is_auto_mode = false;
    for phase in [
        GostPhase::Ticket,
        GostPhase::NewBranch,
        GostPhase::SpecContract,
        GostPhase::RedTest,
        GostPhase::GreenCode,
        GostPhase::Refactor,
        GostPhase::MutationTest,
        GostPhase::QualityGate,
        GostPhase::CloseTicket,
    ] {
        app.gost_phase = phase;
        assert!(
            !app.should_auto_advance_on_completed(),
            "Modo manual nunca deve auto-avançar na fase {:?}",
            phase
        );
    }

    // 2. Quando is_auto_mode é true:
    app.is_auto_mode = true;

    // Fase 1 (Ticket): Parada 1 obrigatória para aprovação da proposta
    app.gost_phase = GostPhase::Ticket;
    assert!(
        !app.should_auto_advance_on_completed(),
        "Fase 1 (Ticket) deve pausar e aguardar aprovação da usuária"
    );

    // Fases 3, 4 e 5: auto-avanço habilitado
    for active_phase in [
        GostPhase::SpecContract,
        GostPhase::RedTest,
        GostPhase::GreenCode,
    ] {
        app.gost_phase = active_phase;
        assert!(
            app.should_auto_advance_on_completed(),
            "Fase {:?} deve auto-avançar no modo auto",
            active_phase
        );
    }

    // Fase 9 (CloseTicket): Parada 2 obrigatória para homologação final
    app.gost_phase = GostPhase::CloseTicket;
    assert!(
        !app.should_auto_advance_on_completed(),
        "Fase 9 (CloseTicket) deve pausar e aguardar homologação da usuária"
    );
}

#[test]
fn test_auto_pipeline_chain_from_phase3_to_phase9() {
    let mut app = App::new();
    app.is_auto_mode = true;
    app.set_ticket_info(Some(31), "feat/31-auto-pipeline", 3);

    let driver = MockAutoDriver {
        current_branch_name: "feat/31-auto-pipeline".into(),
    };

    // --- Passo 1: Conclusão da Fase 3 (SpecContract) ---
    assert_eq!(app.gost_phase, GostPhase::SpecContract);
    assert!(app.should_auto_advance_on_completed());

    // Dispara aprovação automática
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok());
    assert_eq!(app.gost_phase, GostPhase::RedTest);
    let prompt_ph4 = app.take_pending_dispatch();
    assert!(
        prompt_ph4.is_some(),
        "Deve gerar prompt automático para Fase 4"
    );

    // --- Passo 2: Conclusão da Fase 4 (RedTest) ---
    assert_eq!(app.gost_phase, GostPhase::RedTest);
    assert!(app.should_auto_advance_on_completed());

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok());
    assert_eq!(app.gost_phase, GostPhase::GreenCode);
    let prompt_ph5 = app.take_pending_dispatch();
    assert!(
        prompt_ph5.is_some(),
        "Deve gerar prompt automático para Fase 5"
    );

    // --- Passo 3: Conclusão da Fase 5 (GreenCode) ---
    assert_eq!(app.gost_phase, GostPhase::GreenCode);
    assert!(app.should_auto_advance_on_completed());

    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok());

    // Deve ter encadeado automaticamente através de 6, 7 e 8 até a Fase 9 (CloseTicket)
    assert_eq!(app.gost_phase, GostPhase::CloseTicket);
    assert!(
        app.take_pending_dispatch().is_none(),
        "Fase 9 não possui auto-prompt"
    );

    // --- Parada 2: Fase 9 (CloseTicket) PAUSA obrigatoriamente ---
    assert!(
        !app.should_auto_advance_on_completed(),
        "Na Fase 9, should_auto_advance_on_completed deve ser falso para aguardar usuária"
    );
}
