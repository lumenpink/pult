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
