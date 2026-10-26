use pult_core::domain::{
    Actor, FileSessionStorage, GostPhase, Session, SessionStorage, Turn, TurnStatus,
};
use pult_core::runner::{AgyRunner, AgyStreamEvent, RunnerError, RunnerOptions};
use tempfile::tempdir;

#[tokio::test]
async fn test_runner_streaming_chunks() {
    let runner = AgyRunner::with_options(RunnerOptions {
        command: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            "echo 'primeiro chunk'; echo 'segundo chunk'".to_string(),
        ],
        working_dir: None,
    });

    let (mut rx, child_handle) = runner.spawn().expect("deve disparar processo");

    let mut stdout_acc = String::new();
    let mut completed_code = None;

    while let Some(event) = rx.recv().await {
        match event {
            AgyStreamEvent::StdoutChunk(c) => stdout_acc.push_str(&c),
            AgyStreamEvent::Completed { exit_code } => completed_code = Some(exit_code),
            _ => {}
        }
    }

    let status = child_handle.await.expect("join handle").expect("exit code");
    assert_eq!(status, 0);
    assert_eq!(completed_code, Some(0));
    assert!(stdout_acc.contains("primeiro chunk"));
    assert!(stdout_acc.contains("segundo chunk"));
}

#[tokio::test]
async fn test_runner_streaming_session_persistence() {
    let temp_dir = tempdir().expect("tempdir");
    let storage = FileSessionStorage::new(temp_dir.path());

    let mut session = Session::new_draft("Teste de streaming");
    session.metadata.gost.phase = GostPhase::GreenCode.number();
    storage.save(&session).expect("salvar sessao inicial");

    let runner = AgyRunner::with_options(RunnerOptions {
        command: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            "echo 'resposta da IA em streaming'; echo 'mais um detalhe técnico'".to_string(),
        ],
        working_dir: None,
    });

    let final_session = runner
        .execute_turn_and_persist(&mut session, &storage)
        .await
        .expect("deve executar e persistir turno");

    assert_eq!(final_session.turns.len(), 1);
    let turn = &final_session.turns[0];
    assert_eq!(turn.actor, Actor::Agy);
    assert_eq!(turn.status, TurnStatus::Completed);
    assert!(turn.content.contains("resposta da IA em streaming"));
    assert!(turn.content.contains("mais um detalhe técnico"));

    // Confirma que o arquivo em disco foi atualizado
    let loaded = storage
        .load(&session.metadata.session_id)
        .expect("carregar do disco");
    assert_eq!(loaded.turns.len(), 1);
    assert_eq!(loaded.turns[0].status, TurnStatus::Completed);
}

#[tokio::test]
async fn test_runner_detect_truncation() {
    let temp_dir = tempdir().expect("tempdir");
    let storage = FileSessionStorage::new(temp_dir.path());

    let mut session = Session::new_draft("Teste de truncamento");
    storage.save(&session).expect("salvar sessao");

    let runner = AgyRunner::with_options(RunnerOptions {
        command: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            "echo 'Texto antes do corte...'; echo 'STATUS: truncated'".to_string(),
        ],
        working_dir: None,
    });

    let final_session = runner
        .execute_turn_and_persist(&mut session, &storage)
        .await
        .expect("deve processar saída truncada");

    assert_eq!(final_session.turns.len(), 1);
    let turn = &final_session.turns[0];
    assert_eq!(turn.status, TurnStatus::Truncated);
    assert!(turn.content.contains("Texto antes do corte"));
}

#[tokio::test]
async fn test_runner_nonexistent_command_error() {
    let runner = AgyRunner::with_options(RunnerOptions {
        command: "/bin/nonexistent_command_xyz12345".to_string(),
        args: vec![],
        working_dir: None,
    });

    let result = runner.spawn();
    assert!(result.is_err());
    match result {
        Err(RunnerError::Io { .. }) => {}
        other => panic!("esperava RunnerError::Io, obteve {:?}", other),
    }
}

#[tokio::test]
async fn test_runner_resume_session() {
    let temp_dir = tempdir().expect("tempdir");
    let storage = FileSessionStorage::new(temp_dir.path());

    let mut session = Session::new_formal(42, "auth-ratelimit", "feat/42-auth-ratelimit");
    session.turns.push(Turn {
        turn_number: 1,
        actor: Actor::User,
        timestamp: "2026-10-26T10:00:00Z".to_string(),
        content: "Precisamos do contrato".to_string(),
        status: TurnStatus::Completed,
        tokens: Some(48),
        tokens_out: None,
    });
    session.turns.push(Turn {
        turn_number: 2,
        actor: Actor::Agy,
        timestamp: "2026-10-26T10:00:10Z".to_string(),
        content: "Proposta de código parcial...".to_string(),
        status: TurnStatus::Truncated,
        tokens: None,
        tokens_out: Some(150),
    });
    storage.save(&session).expect("salvar sessao");

    let session_path = temp_dir
        .path()
        .join(format!("{}.md", session.metadata.session_id));
    let inspection = AgyRunner::inspect_session_for_resume(&session_path, "feat/42-auth-ratelimit")
        .expect("inspecionar sessao para resume");

    assert_eq!(inspection.session_id, session.metadata.session_id);
    assert!(inspection.needs_continuation);
    assert!(inspection.branch_matches);
    assert_eq!(inspection.last_turn_status, Some(TurnStatus::Truncated));
}
