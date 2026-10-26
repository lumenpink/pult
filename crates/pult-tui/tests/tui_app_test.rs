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

#[test]
fn test_mutation_empty_submission_returns_none() {
    let mut app = App::new();
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let action = handle_key_event(&mut app, key);
    assert_eq!(action, None);
    assert!(app.submit_input().is_none());
}

#[test]
fn test_mutation_scrolling_boundary_limits() {
    let mut app = App::new();
    app.append_viewport_text("L1\nL2\nL3\nL4\nL5");
    assert_eq!(app.viewport_lines.len(), 5);

    // Subir além do topo não dá underflow
    app.scroll_up(100);
    assert_eq!(app.scroll_offset, 0);

    // Rolar para baixo com altura visível 2 -> max_scroll = 5 - 2 = 3
    app.scroll_down(100, 2);
    assert_eq!(app.scroll_offset, 3);

    app.scroll_to_bottom(2);
    assert_eq!(app.scroll_offset, 3);

    app.scroll_to_top();
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_mutation_viewport_navigation_keys() {
    let mut app = App::new();
    app.toggle_focus(); // Ativa Focus::Viewport
    app.append_viewport_text("Linha 1\nLinha 2\nLinha 3\nLinha 4\nLinha 5\nLinha 6");

    let down_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(
        handle_key_event(&mut app, down_key),
        Some(TuiAction::Scrolled)
    );

    let pgdn_key = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
    assert_eq!(
        handle_key_event(&mut app, pgdn_key),
        Some(TuiAction::Scrolled)
    );

    let up_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    assert_eq!(
        handle_key_event(&mut app, up_key),
        Some(TuiAction::Scrolled)
    );

    let end_key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
    assert_eq!(
        handle_key_event(&mut app, end_key),
        Some(TuiAction::Scrolled)
    );

    let home_key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
    assert_eq!(
        handle_key_event(&mut app, home_key),
        Some(TuiAction::Scrolled)
    );
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_mutation_render_small_screen_geometry() {
    let backend = TestBackend::new(25, 10);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let app = App::new();

    terminal
        .draw(|f| {
            render(&app, f);
        })
        .expect("deve renderizar em terminal pequeno sem panic");
}
