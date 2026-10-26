use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pult_core::domain::GostPhase;
use pult_tui::app::App;
use pult_tui::event::{handle_key_event, TuiAction};
use pult_tui::ui::render;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_gost_phase_advancement_cycle() {
    let mut app = App::new();
    assert_eq!(app.phase(), GostPhase::Ticket);

    let expected_phases = [
        GostPhase::NewBranch,
        GostPhase::SpecContract,
        GostPhase::RedTest,
        GostPhase::GreenCode,
        GostPhase::Refactor,
        GostPhase::MutationTest,
        GostPhase::QualityGate,
        GostPhase::CloseTicket,
    ];

    for expected in expected_phases {
        let new_phase = app.advance_gost_phase().expect("deve avancar de fase");
        assert_eq!(new_phase, expected);
        assert_eq!(app.phase(), expected);
    }

    // Tentar avançar além da Fase 9 deve retornar Err
    let overflow = app.advance_gost_phase();
    assert!(overflow.is_err());
    assert_eq!(app.phase(), GostPhase::CloseTicket);
}

#[test]
fn test_approve_spec_contract_shortcut_ctrl_p() {
    let mut app = App::new();
    app.set_gost_phase(GostPhase::SpecContract);

    let key_ctrl_p = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
    let action = handle_key_event(&mut app, key_ctrl_p);

    assert_eq!(action, Some(TuiAction::SpecApproved));
    assert_eq!(app.phase(), GostPhase::RedTest);

    // Tentar aprovar fora da Fase 3 deve falhar
    let action_invalid = handle_key_event(&mut app, key_ctrl_p);
    assert_ne!(action_invalid, Some(TuiAction::SpecApproved));
}

#[test]
fn test_check_phase3_guard_veto() {
    let mut app = App::new();
    app.set_gost_phase(GostPhase::SpecContract);

    // Modificação em arquivos de especificação é permitida
    let allowed_diff = &["docs/specs/architecture.md", "README.md"];
    assert!(app.check_phase3_guard(allowed_diff).is_ok());

    // Modificação em arquivos de código de produção na Fase 3 deve ser vetada
    let forbidden_diff = &["docs/specs/architecture.md", "src/adapters/database.rs"];
    let veto = app.check_phase3_guard(forbidden_diff);
    assert!(veto.is_err());
    let err_msg = veto.unwrap_err();
    assert!(err_msg.contains("Veto Fase 3"));
    assert!(err_msg.contains("src/adapters/database.rs"));
}

#[test]
fn test_compile_closing_chronicle() {
    let mut app = App::new();
    app.set_ticket_info(Some(9), "feat/9-gost-state-guard", 9);
    app.append_viewport_line("<!-- TURN: 1 | ACTOR: user -->");
    app.append_viewport_line("Definir máquina de estados da TUI");
    app.append_viewport_line("<!-- TURN: 2 | ACTOR: agy -->");
    app.append_viewport_line("Contratos e atalhos implementados com sucesso.");

    let chronicle =
        app.compile_closing_chronicle("tui", "implement gost state guard and shortcuts");

    // Header deve seguir o padrão canônico (<= 72 chars)
    let first_line = chronicle.lines().next().unwrap();
    assert!(first_line.len() <= 72);
    assert!(first_line.contains("docs(tui): implement gost state guard and shortcuts (closes #9)"));

    // Corpo deve conter metadados e contagem de turnos
    assert!(chronicle.contains("Ticket: #9"));
    assert!(chronicle.contains("Branch: feat/9-gost-state-guard"));
    assert!(chronicle.contains("Fases Concluídas: 1 a 9"));
}

#[test]
fn test_render_footer_contextual_action_buttons() {
    let backend = TestBackend::new(120, 20);
    let mut terminal = Terminal::new(backend).expect("terminal");

    let mut app = App::new();

    // Na Fase 3: deve exibir atalho de aprovação de spec
    app.set_gost_phase(GostPhase::SpecContract);
    terminal.draw(|f| render(&app, f)).unwrap();
    let content_p3 = format!("{:?}", terminal.backend().buffer());
    assert!(content_p3.contains("Ctrl+P: Aprovar Spec"));

    // Na Fase 9: deve exibir atalho de fechamento
    app.set_gost_phase(GostPhase::CloseTicket);
    terminal.draw(|f| render(&app, f)).unwrap();
    let content_p9 = format!("{:?}", terminal.backend().buffer());
    assert!(content_p9.contains("Ctrl+G: Fechar"));
    assert!(content_p9.contains("Ctrl+M: Merge CCCP"));
}

#[test]
fn test_mutation_phase3_guard_inactive_in_other_phases() {
    let mut app = App::new();
    app.set_gost_phase(GostPhase::GreenCode);

    // Na Fase 5, alterações em src/ são legítimas e esperadas
    let src_diff = &["src/main.rs", "crates/pult-core/src/lib.rs"];
    assert!(app.check_phase3_guard(src_diff).is_ok());
}

#[test]
fn test_mutation_chronicle_draft_session_without_ticket() {
    let mut app = App::new();
    app.set_ticket_info(None, "draft-branch", 9);
    let chronicle = app.compile_closing_chronicle("core", "draft session finish");

    assert!(chronicle.starts_with("docs(core): draft session finish\n"));
    assert!(!chronicle.contains("closes #"));
}

#[test]
fn test_mutation_ctrl_n_advances_phase_and_logs() {
    let mut app = App::new();
    assert_eq!(app.phase(), GostPhase::Ticket);

    let key_ctrl_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
    let action = handle_key_event(&mut app, key_ctrl_n);

    assert_eq!(action, Some(TuiAction::PhaseAdvanced(GostPhase::NewBranch)));
    assert_eq!(app.phase(), GostPhase::NewBranch);
    assert!(app
        .viewport_lines
        .last()
        .unwrap()
        .contains("Avanço para FASE 2"));
}

#[test]
fn test_mutation_ctrl_g_and_ctrl_m_actions() {
    let mut app = App::new();

    let key_ctrl_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
    assert_eq!(
        handle_key_event(&mut app, key_ctrl_g),
        Some(TuiAction::FinishTriggered)
    );

    let key_ctrl_m = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL);
    assert_eq!(
        handle_key_event(&mut app, key_ctrl_m),
        Some(TuiAction::MergeTriggered)
    );
}
