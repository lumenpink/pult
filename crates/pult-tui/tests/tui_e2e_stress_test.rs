//! Testes de estresse e desempenho da interface de terminal (TUI).
//!
//! Valida a integridade do viewport desacoplado em cargas pesadas (5.000 linhas)
//! garantindo taxas de renderização inferiores a 50ms e imunidade a mutações no editor.

use pult_tui::app::App;
use pult_tui::ui::render;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::time::Instant;

#[test]
fn test_e2e_viewport_5000_lines_scroll_stress() {
    let mut app = App::new();

    // Insere 5.000 linhas no viewport
    for i in 0..5000 {
        app.append_viewport_line(&format!(
            "<!-- TURN: {} --> Registro de evento longo com código executável e análises detalhadas",
            i
        ));
    }

    assert_eq!(app.viewport_lines.len(), 5000);

    // Textarea do editor deve permanecer vazio
    assert!(app.textarea.lines()[0].is_empty());

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("terminal");

    let start = Instant::now();

    // Renderiza no topo
    terminal.draw(|f| render(&app, f)).expect("render topo");

    // Rola para o fundo (5000 linhas)
    app.scroll_to_bottom(35);
    assert_eq!(app.scroll_offset, 5000 - 35);
    terminal.draw(|f| render(&app, f)).expect("render fundo");

    // Rola de volta para o meio e para o topo
    app.scroll_up(2500);
    assert_eq!(app.scroll_offset, 5000 - 35 - 2500);
    terminal.draw(|f| render(&app, f)).expect("render meio");

    app.scroll_to_top();
    assert_eq!(app.scroll_offset, 0);
    terminal
        .draw(|f| render(&app, f))
        .expect("render retorno topo");

    let duration = start.elapsed();
    // 4 renderizações completas com 5000 linhas devem levar < 50ms
    assert!(
        duration.as_millis() < 50,
        "Tempo de renderização: {:?}",
        duration
    );

    // Editor não deve sofrer qualquer mutação acidental
    assert!(app.textarea.lines()[0].is_empty());
}

#[test]
fn test_mutation_viewport_extreme_scroll_bounds() {
    let mut app = App::new();
    for i in 0..100 {
        app.append_viewport_line(&format!("Linha {}", i));
    }

    // Rolagem para baixo além do limite (com 20 linhas visíveis, máx é 80)
    app.scroll_down(500, 20);
    assert_eq!(app.scroll_offset, 80);

    // Rolagem para cima além do limite
    app.scroll_up(500);
    assert_eq!(app.scroll_offset, 0);

    // Viewport vazio não deve entrar em pânico
    let mut empty_app = App::new();
    empty_app.scroll_down(10, 20);
    assert_eq!(empty_app.scroll_offset, 0);
    empty_app.scroll_up(10);
    assert_eq!(empty_app.scroll_offset, 0);
    empty_app.scroll_to_bottom(30);
    assert_eq!(empty_app.scroll_offset, 0);
    empty_app.scroll_to_top();
    assert_eq!(empty_app.scroll_offset, 0);
}
