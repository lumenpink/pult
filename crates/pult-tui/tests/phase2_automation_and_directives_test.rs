use pult_core::domain::{Actor, GostPhase, Session, TurnStatus};
use pult_tui::app::App;

#[test]
fn test_agent_directive_overrides_global_gemini_rules_and_has_no_gost_preaching() {
    for n in 1..=9 {
        let phase = GostPhase::from_u8(n).expect("valid phase");
        let directive = phase.agent_directive("feat/25-automation");

        // 1. Deve conter o cabeçalho imperativo de anulação de escopo
        assert!(
            directive.contains("[DIRETIVA DE OPERAÇÃO - SUBAGENTE DE EXECUÇÃO PULT]"),
            "Fase {} deve conter o cabeçalho de isolamento",
            n
        );
        assert!(
            directive.contains("Anulação de escopo") && directive.contains("Você NÃO é o maestro do ciclo"),
            "Fase {} deve cancelar explicitamente regras globais de orquestração",
            n
        );

        // 2. Não deve conter pregações ou roadmaps sobre fases futuras
        assert!(
            !directive.contains("próximas fases ciclo gost"),
            "Fase {} não deve conter pregação sobre fases futuras",
            n
        );
    }
}

#[test]
fn test_history_context_formatting_prepends_previous_turns() {
    let mut session = Session::new_draft("test-slug");
    session.add_turn(
        Actor::User,
        "Podemos implementar rate limiting?".to_string(),
        TurnStatus::Completed,
    );
    session.add_turn(
        Actor::Agy,
        "Sim, sugiro algoritmo de token bucket.".to_string(),
        TurnStatus::Completed,
    );

    let formatted = pult_tui::app::format_prompt_with_history(&session, "y", "/* AST */");

    assert!(formatted.contains("--- Contexto Histórico Recente da Sessão ---"));
    assert!(formatted.contains("[Usuária]: Podemos implementar rate limiting?"));
    assert!(formatted.contains("[Agente]: Sim, sugiro algoritmo de token bucket."));
    assert!(formatted.contains("[Mensagem Atual da Usuária]:\ny"));
    assert!(formatted.contains("/* AST */"));
}

#[test]
fn test_ui_header_displays_branch_leaf_icon() {
    let mut app = App::new();
    app.set_ticket_info(Some(25), "feat/25-automation", 1);
    let span_text = app.header_branch_label();
    assert!(span_text.contains("🌿 feat/25-automation"));
}
