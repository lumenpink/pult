use pult_tui::app::App;

#[test]
fn test_append_stream_chunk_accumulates_without_splitting_words() {
    let mut app = App::new();

    // Envia primeiro pedaço sem newline
    app.append_stream_chunk("Olá, ");
    assert_eq!(app.viewport_lines.len(), 1);
    assert_eq!(app.viewport_lines[0], "Olá, ");

    // Envia continuação com newline
    app.append_stream_chunk("mundo!\n");
    assert_eq!(app.viewport_lines.len(), 1);
    assert_eq!(app.viewport_lines[0], "Olá, mundo!");

    // Envia início da segunda linha
    app.append_stream_chunk("Segunda ");
    assert_eq!(app.viewport_lines.len(), 2);
    assert_eq!(app.viewport_lines[1], "Segunda ");

    // Envia término da segunda linha
    app.append_stream_chunk("linha.");
    assert_eq!(app.viewport_lines.len(), 2);
    assert_eq!(app.viewport_lines[1], "Segunda linha.");
}

#[test]
fn test_closing_chronicle_uses_anarchist_vocabulary() {
    let mut app = App::new();
    app.set_ticket_info(Some(15), "fix/15", 9);
    let chronicle = app.compile_closing_chronicle("tui", "buffer de stream");
    assert!(
        !chronicle.to_lowercase().contains("soberan"),
        "Crônica não pode conter termos monárquicos 'soberano', obtido: {}",
        chronicle
    );
    assert!(
        chronicle.contains("Crônica da Entrega (GOST DD-TDD)"),
        "Crônica deve conter título canônico autônomo, obtido: {}",
        chronicle
    );
}

#[test]
fn test_mutation_stream_chunk_empty_and_consecutive_newlines() {
    let mut app = App::new();

    // Chunk vazio não altera nada
    app.append_stream_chunk("");
    assert_eq!(app.viewport_lines.len(), 0);

    // Consecutivos newlines
    app.append_stream_chunk("Linha 1\n\nLinha 3\n");
    assert_eq!(app.viewport_lines.len(), 3);
    assert_eq!(app.viewport_lines[0], "Linha 1");
    assert_eq!(app.viewport_lines[1], "");
    assert_eq!(app.viewport_lines[2], "Linha 3");
    assert!(!app.is_line_open);

    // Acumulação posterior começa em nova linha
    app.append_stream_chunk("Nova linha");
    assert_eq!(app.viewport_lines.len(), 4);
    assert_eq!(app.viewport_lines[3], "Nova linha");
    assert!(app.is_line_open);
}
