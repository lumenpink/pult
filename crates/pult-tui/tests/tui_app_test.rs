use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pult_tui::app::{App, Focus};
use pult_tui::event::{handle_key_event, TuiAction};
use pult_tui::ui::render;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_app_initial_state_and_focus_toggle() {
    let mut app = App::new();
    assert_eq!(app.focus, Focus::Editor);
    assert_eq!(app.scroll_offset, 0);
    assert!(!app.should_quit);

    app.toggle_focus();
    assert_eq!(app.focus, Focus::Viewport);

    app.toggle_focus();
    assert_eq!(app.focus, Focus::Editor);
}

#[test]
fn test_viewport_scroll_decoupled_from_editor() {
    let mut app = App::new();
    for i in 0..50 {
        app.append_viewport_line(&format!("Linha de log número {}", i));
    }

    assert_eq!(app.scroll_offset, 0);

    // Rola para baixo no viewport
    app.scroll_down(5, 10);
    assert_eq!(app.scroll_offset, 5);

    // O editor continua intacto
    assert!(app.textarea.lines()[0].is_empty());

    // Rola de volta para o topo
    app.scroll_to_top();
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_editor_submission_with_ctrl_s() {
    let mut app = App::new();

    // Simula digitação no textarea
    app.textarea.insert_str("Prompt para o agente");
    assert_eq!(app.textarea.lines()[0], "Prompt para o agente");

    // Envia evento Ctrl+S
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let action = handle_key_event(&mut app, key);

    assert_eq!(
        action,
        Some(TuiAction::Submit("Prompt para o agente".to_string()))
    );
    // Textarea deve ser limpo após submissão
    assert!(app.textarea.lines()[0].is_empty());
}

#[test]
fn test_ctrl_q_signals_quit() {
    let mut app = App::new();
    assert!(!app.should_quit);

    let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let action = handle_key_event(&mut app, key);

    assert_eq!(action, Some(TuiAction::Quit));
    assert!(app.should_quit);
}

#[test]
fn test_render_split_viewport_in_test_backend() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("terminal");

    let mut app = App::new();
    app.set_ticket_info(Some(8), "feat/8-split-viewport-shell", 5);
    app.append_viewport_line("<!-- TURN: 1 | ACTOR: user -->");
    app.append_viewport_line("Implementar split viewport no terminal.");

    terminal
        .draw(|f| {
            render(&app, f);
        })
        .expect("renderização no backend de teste deve suceder");

    let buffer = terminal.backend().buffer();
    let content = format!("{:?}", buffer);

    assert!(content.contains("PULT"));
    assert!(content.contains("FASE 5"));
    assert!(content.contains("feat/8-split-viewport-shell"));
}
