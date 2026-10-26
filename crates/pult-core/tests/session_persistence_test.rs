use pult_core::domain::*;
use tempfile::tempdir;

#[test]
fn test_gost_phase_lifecycle() {
    let mut current = GostPhase::Ticket;
    let expected_order = [
        GostPhase::Ticket,
        GostPhase::NewBranch,
        GostPhase::SpecContract,
        GostPhase::RedTest,
        GostPhase::GreenCode,
        GostPhase::Refactor,
        GostPhase::MutationTest,
        GostPhase::QualityGate,
        GostPhase::CloseTicket,
    ];

    for (idx, expected) in expected_order.iter().enumerate() {
        assert_eq!(current, *expected);
        assert_eq!(current.number(), (idx + 1) as u8);

        if idx + 1 < expected_order.len() {
            current = current.next().expect("deve ter próxima fase");
        } else {
            assert_eq!(current.next(), None);
        }
    }
}

#[test]
fn test_gost_phase_production_code_guard() {
    assert!(!GostPhase::Ticket.allows_production_code());
    assert!(!GostPhase::NewBranch.allows_production_code());
    assert!(!GostPhase::SpecContract.allows_production_code());
    assert!(!GostPhase::RedTest.allows_production_code());

    assert!(GostPhase::GreenCode.allows_production_code());
    assert!(GostPhase::Refactor.allows_production_code());
    assert!(GostPhase::MutationTest.allows_production_code());
    assert!(GostPhase::QualityGate.allows_production_code());
    assert!(GostPhase::CloseTicket.allows_production_code());
}

#[test]
fn test_session_markdown_serialization_and_deserialization() {
    let mut session = Session::new_formal(42, "auth-rate-limiting", "feat/42-rate-limiting");

    session.turns.push(Turn {
        turn_number: 1,
        actor: Actor::User,
        timestamp: "2026-10-26T10:00:00Z".to_string(),
        content: "Precisamos definir o contrato do limitador de requisições.".to_string(),
        status: TurnStatus::Completed,
        tokens: Some(48),
        tokens_out: None,
    });

    session.turns.push(Turn {
        turn_number: 1,
        actor: Actor::Agy,
        timestamp: "2026-10-26T10:00:12Z".to_string(),
        content: "Proposta de contrato em `src/domain/ports.rs`:\n```rust\npub trait RateLimiter {}\n```".to_string(),
        status: TurnStatus::Completed,
        tokens: None,
        tokens_out: Some(310),
    });

    let markdown = session.to_markdown().expect("deve serializar para markdown");
    assert!(markdown.starts_with("---\n"));
    assert!(markdown.contains("session_id:"));
    assert!(markdown.contains("id: 42"));
    assert!(markdown.contains("<!-- TURN: 1 | ACTOR: user"));
    assert!(markdown.contains("<!-- TURN: 1 | ACTOR: agy"));

    let parsed = Session::from_markdown(&markdown).expect("deve desserializar markdown");
    assert_eq!(parsed.metadata.session_id, session.metadata.session_id);
    assert_eq!(parsed.metadata.ticket.as_ref().unwrap().id, 42);
    assert_eq!(parsed.turns.len(), 2);
    assert_eq!(parsed.turns[0].actor, Actor::User);
    assert_eq!(parsed.turns[0].content, "Precisamos definir o contrato do limitador de requisições.");
    assert_eq!(parsed.turns[1].actor, Actor::Agy);
}

#[test]
fn test_draft_session_naming_convention() {
    let draft = Session::new_draft("investigacao-parser");
    assert!(draft.is_draft());
    let filename = draft.filename();
    assert!(filename.starts_with('t'));
    assert!(filename.ends_with("-investigacao-parser.md"));
}

#[test]
fn test_formal_session_naming_convention() {
    let formal = Session::new_formal(42, "auth-rate-limiting", "feat/42-rate-limiting");
    assert!(!formal.is_draft());
    let filename = formal.filename();
    assert!(filename.starts_with("000042-"));
    assert!(filename.ends_with("-auth-rate-limiting.md"));
}

#[test]
fn test_file_session_storage_roundtrip() {
    let temp_dir = tempdir().expect("deve criar pasta temporária");
    let storage = FileSessionStorage::new(temp_dir.path());

    let session = Session::new_formal(101, "cli-driver", "feat/101-cli-driver");
    let path = storage.save(&session).expect("deve salvar arquivo");
    assert!(path.exists());

    let loaded = storage.load(&session.metadata.session_id).expect("deve carregar sessão");
    assert_eq!(loaded.metadata.session_id, session.metadata.session_id);

    let found = storage.find_by_ticket(101).expect("busca por ticket deve funcionar");
    assert!(found.is_some());
    assert_eq!(found.unwrap().metadata.session_id, session.metadata.session_id);

    let all = storage.list().expect("deve listar sessões");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].session_id, session.metadata.session_id);
}
