use pult_core::domain::GostPhase;
use pult_tui::app::App;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_app_busy_timer_and_formatted_elapsed() {
    let mut app = App::new();
    assert!(app.busy_start_time.is_none());
    assert_eq!(app.busy_elapsed_formatted(), "");

    app.set_busy(true, "Processando...");
    assert!(app.busy_start_time.is_some());
    let elapsed = app.busy_elapsed_formatted();
    assert!(elapsed.starts_with('(') && elapsed.ends_with(')'));
    assert!(elapsed.contains('s'));

    app.set_busy(false, "");
    assert!(app.busy_start_time.is_none());
    assert_eq!(app.busy_elapsed_formatted(), "");
}

#[test]
fn test_header_always_shows_gost_phase_without_spinner() {
    let mut app = App::new();
    app.set_gost_phase(GostPhase::SpecContract);
    app.set_busy(true, "Operando agente...");

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            pult_tui::ui::render(&app, f);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|c| c.symbol()).collect();

    // O cabeçalho deve conter a fase fixa
    assert!(
        content.contains("FASE 3: Doc / Spec Contract"),
        "Cabeçalho deve conter a fase sem ser sobreposto pelo spinner: {}",
        content
    );

    // O título do editor inferior deve conter o spinner e o contador
    assert!(
        content.contains("[Editor ("),
        "Editor inferior deve conter o spinner e contador: {}",
        content
    );
}
