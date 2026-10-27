use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pult_core::domain::gost::GostPhase;
use pult_tui::app::App;
use pult_tui::event::{handle_key_event, TuiAction};

#[test]
fn test_ctrl_p_universal_in_any_phase() {
    let mut app = App::new();

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
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        let action = handle_key_event(&mut app, key);
        assert_eq!(
            action,
            Some(TuiAction::SpecApproved),
            "Ctrl+P deve emitir SpecApproved na fase {:?}",
            phase
        );
    }
}

#[test]
fn test_ctrl_x_and_slash_auto_toggle_auto_mode() {
    let mut app = App::new();
    assert!(!app.is_auto_mode, "Modo inicial deve ser manual (false)");

    // 1. Testa atalho Ctrl+X
    let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
    let action = handle_key_event(&mut app, key);
    assert_eq!(action, Some(TuiAction::AutoModeToggled(true)));
    assert!(app.is_auto_mode, "Ctrl+X deve ativar modo auto");

    // 2. Testa comando /auto
    let action_cmd = app.execute_slash_command("/auto");
    assert_eq!(action_cmd, Some(TuiAction::AutoModeToggled(false)));
    assert!(
        !app.is_auto_mode,
        "/auto deve alternar de volta para manual"
    );
}

#[test]
fn test_enter_direct_dispatch_for_slash_command() {
    let mut app = App::new();
    // Digita "/approve" no editor
    app.textarea.insert_str("/approve");

    // Pressiona Enter diretamente
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = handle_key_event(&mut app, key);

    assert_eq!(
        action,
        Some(TuiAction::SpecApproved),
        "Enter em '/approve' deve despachar ação diretamente"
    );
    assert!(
        app.textarea.lines()[0].is_empty(),
        "Editor deve ser limpo após submissão do comando"
    );
}

#[test]
fn test_enter_on_regular_text_inserts_newline() {
    let mut app = App::new();
    app.textarea.insert_str("implementar módulo de testes");

    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = handle_key_event(&mut app, key);

    assert_eq!(action, None, "Enter em texto comum não deve despachar ação");
    assert_eq!(
        app.textarea.lines().len(),
        2,
        "Enter em texto comum deve quebrar linha"
    );
}

#[test]
fn test_tab_autocomplete_moves_cursor_to_end_and_includes_auto() {
    let mut app = App::new();
    app.textarea.insert_str("/au");

    let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    let action = handle_key_event(&mut app, key);

    assert_eq!(action, None);
    assert_eq!(
        app.textarea.lines()[0],
        "/auto",
        "Tab em '/au' deve autocompletar para '/auto'"
    );

    let cursor = app.textarea.cursor();
    assert_eq!(
        cursor.1, 5,
        "Cursor deve estar no final da palavra '/auto' (coluna 5), mas estava em {}",
        cursor.1
    );
}
