use pult_core::{Actor, FileSessionStorage, Session, SessionStorage, TurnStatus};
use tempfile::tempdir;

#[test]
fn test_multiturn_continuous_persistence_and_streaming() {
    let tmp = tempdir().expect("tempdir");
    let storage = FileSessionStorage::new(tmp.path());

    // 1. Inicia sessão rascunho
    let mut session = Session::new_draft("conversa-multiturno");
    assert_eq!(session.turns.len(), 0);

    // 2. Primeiro turno: Usuária envia comando
    let turn1 = session.add_turn(
        Actor::User,
        "Olá, vamos criar a especificação técnica.".to_string(),
        TurnStatus::Completed,
    );
    assert_eq!(turn1.turn_number, 1);
    assert_eq!(turn1.actor, Actor::User);
    storage.save(&session).expect("salvar turno 1");

    // 3. Segundo turno: Agente Externo inicia streaming com InProgress
    let turn2 = session.add_turn(
        Actor::Agy,
        String::new(),
        TurnStatus::InProgress,
    );
    assert_eq!(turn2.turn_number, 2);
    assert_eq!(turn2.status, TurnStatus::InProgress);
    storage.save(&session).expect("salvar turno 2 inicial");

    // 4. Chegam chunks parciais via stream
    session.append_to_last_turn("Aqui está ");
    storage.save(&session).expect("salvar chunk 1");

    session.append_to_last_turn("o rascunho da especificação.\n");
    storage.save(&session).expect("salvar chunk 2");

    // Valida que o arquivo em disco já contém o streaming parcial
    let loaded_partial = storage.load(&session.metadata.session_id).expect("load partial");
    assert_eq!(loaded_partial.turns.len(), 2);
    assert_eq!(
        loaded_partial.turns[1].content,
        "Aqui está o rascunho da especificação."
    );
    assert_eq!(loaded_partial.turns[1].status, TurnStatus::InProgress);

    // 5. Agente conclui com sucesso
    session.update_last_turn_status(TurnStatus::Completed);
    storage.save(&session).expect("salvar turno 2 final");

    // 6. Terceiro turno: Usuária faz pergunta de acompanhamento na MESMA sessão
    let turn3 = session.add_turn(
        Actor::User,
        "Perfeito! Agora defina o teste Red.".to_string(),
        TurnStatus::Completed,
    );
    assert_eq!(turn3.turn_number, 3);
    storage.save(&session).expect("salvar turno 3");

    // 7. Quarto turno: Agente responde
    let turn4 = session.add_turn(
        Actor::Agy,
        "Testes criados no módulo tests/red_test.rs.".to_string(),
        TurnStatus::Completed,
    );
    assert_eq!(turn4.turn_number, 4);
    storage.save(&session).expect("salvar turno 4");

    // 8. Recarrega sessão final do disco e valida integridade dos 4 turnos
    let final_session = storage.load(&session.metadata.session_id).expect("load final");
    assert_eq!(final_session.turns.len(), 4);
    assert_eq!(final_session.turns[0].turn_number, 1);
    assert_eq!(final_session.turns[0].actor, Actor::User);
    assert_eq!(final_session.turns[1].turn_number, 2);
    assert_eq!(final_session.turns[1].actor, Actor::Agy);
    assert_eq!(final_session.turns[1].status, TurnStatus::Completed);
    assert_eq!(final_session.turns[2].turn_number, 3);
    assert_eq!(final_session.turns[2].actor, Actor::User);
    assert_eq!(final_session.turns[3].turn_number, 4);
    assert_eq!(final_session.turns[3].actor, Actor::Agy);
}
