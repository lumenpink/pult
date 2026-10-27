use pult_tui::app::App;
use pult_tui::ui::render;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_app_busy_state_and_spinner() {
    let mut app = App::new();
    assert!(!app.is_busy);
    assert_eq!(app.spinner_glyph(), "⠋");

    app.is_busy = true;
    app.tick_spinner();
    assert_eq!(app.spinner_glyph(), "⠙");
    app.tick_spinner();
    assert_eq!(app.spinner_glyph(), "⠹");

    // Bloqueia input quando ocupado
    app.textarea.insert_str("Nova instrução");
    assert_eq!(app.submit_input(), None);

    app.is_busy = false;
    assert_eq!(app.submit_input(), Some("Nova instrução".to_string()));
}

#[test]
fn test_app_spinner_wraps_around() {
    let mut app = App::new();
    for _ in 0..10 {
        app.tick_spinner();
    }
    assert_eq!(app.spinner_glyph(), "⠋");
}

#[test]
fn test_render_with_busy_state_does_not_panic() {
    let mut app = App::new();
    app.set_busy(true, "Scout...");
    app.append_viewport_line("☭ [PULT] Processando...");

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(&app, f)).unwrap();
}
