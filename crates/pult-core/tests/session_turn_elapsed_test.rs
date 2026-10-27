use pult_core::domain::{Actor, Session, TurnStatus};

#[test]
fn test_turn_elapsed_serialization_and_deserialization() {
    let mut session = Session::new_formal(28, "elapsed-test", "feat/28-test");
    session.add_turn(
        Actor::Agy,
        "Resposta do agente externo".to_string(),
        TurnStatus::Completed,
    );
    session.set_last_turn_elapsed(4250); // 4.25 segundos

    let md = session.to_markdown().unwrap();
    assert!(
        md.contains("ELAPSED: 4.25s") || md.contains("ELAPSED: 4250ms"),
        "Markdown deve conter tag ELAPSED: {}",
        md
    );

    let parsed = Session::from_markdown(&md).unwrap();
    assert_eq!(parsed.turns.len(), 1);
    assert_eq!(parsed.turns[0].elapsed_ms, Some(4250));
}

#[test]
fn test_mutation_parse_elapsed_milliseconds_and_seconds_variants() {
    let md = "---\nsession_id: 000028-variants\nticket:\n  id: 28\n  state: open\n  branch: feat/28-test\ngost:\n  phase: 3\n  command: test\ncreated_at: 2026-10-27T10:00:00Z\nupdated_at: 2026-10-27T10:00:00Z\n---\n\n<!-- TURN: 1 | ACTOR: user | TIME: 2026-10-27T10:00:00Z | ELAPSED: 500ms -->\nPergunta\n\n<!-- TURN: 2 | ACTOR: agy | TIME: 2026-10-27T10:00:01Z | ELAPSED: 2.50s -->\nResposta\n\n<!-- TURN: 3 | ACTOR: scout | TIME: 2026-10-27T10:00:02Z -->\nSem elapsed\n\n";

    let session = Session::from_markdown(md).unwrap();
    assert_eq!(session.turns.len(), 3);
    assert_eq!(session.turns[0].elapsed_ms, Some(500));
    assert_eq!(session.turns[1].elapsed_ms, Some(2500));
    assert_eq!(session.turns[2].elapsed_ms, None);
}
