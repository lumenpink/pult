use pult_tui::app::App;

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
