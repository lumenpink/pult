use pult_core::{Actor, FileSessionStorage, GostPhase, Session, SessionStorage, TurnStatus};
use pult_tui::app::{infer_ticket_from_branch, App};
use tempfile::tempdir;

#[test]
fn test_infer_ticket_from_branch_patterns() {
    assert_eq!(infer_ticket_from_branch("feat/17-git-detection"), Some(17));
    assert_eq!(infer_ticket_from_branch("fix/42-rate-limiting"), Some(42));
    assert_eq!(infer_ticket_from_branch("docs/101-specs-update"), Some(101));
    assert_eq!(infer_ticket_from_branch("refactor/5-clean-code"), Some(5));
    assert_eq!(infer_ticket_from_branch("test/99-edge-cases"), Some(99));
    assert_eq!(infer_ticket_from_branch("17-git-detection"), Some(17));

    // Branches sem identificador numérico
    assert_eq!(infer_ticket_from_branch("main"), None);
    assert_eq!(infer_ticket_from_branch("master"), None);
    assert_eq!(infer_ticket_from_branch("feat/no-number"), None);
    assert_eq!(infer_ticket_from_branch("devel"), None);
    assert_eq!(infer_ticket_from_branch(""), None);
}

#[test]
fn test_resume_session_restores_history_and_gost_state() {
    let tmp = tempdir().expect("tempdir");
    let storage = FileSessionStorage::new(tmp.path());

    // Cria uma sessão formal na Fase 4 (RedTest) com 2 turnos
    let mut session = Session::new_formal(42, "auth-ratelimit", "feat/42-auth-ratelimit");
    session.metadata.gost.phase = 4;

    session.add_turn(
        Actor::User,
        "Implementar limitador de requisições.".to_string(),
        TurnStatus::Completed,
    );
    session.add_turn(
        Actor::Agy,
        "Plano formulado e contrato definido.".to_string(),
        TurnStatus::Completed,
    );

    storage.save(&session).expect("salvar sessão");
    let session_file = tmp.path().join(session.filename());

    // Instancia App e restaura a sessão
    let mut app = App::new();
    let res = app.load_session_file(&session_file);
    assert!(res.is_ok(), "carregamento da sessão deve ter sucesso");

    // Valida sincronização de metadados
    assert_eq!(app.ticket_id, Some(42));
    assert_eq!(app.branch_name, "feat/42-auth-ratelimit");
    assert_eq!(app.gost_phase, GostPhase::RedTest);
    assert_eq!(app.session_id(), Some(session.metadata.session_id.clone()));

    // Valida que o viewport contém o histórico dos turnos anteriores
    let vp_text = app.viewport_lines.join("\n");
    assert!(vp_text.contains("> Implementar limitador de requisições."));
    assert!(vp_text.contains("Plano formulado e contrato definido."));
}

#[test]
fn test_mutation_git_detection_and_resume_edge_cases() {
    let mut app = App::new();

    // Mutante 1: Carregar arquivo inexistente
    let missing_path = std::path::Path::new("/tmp/arquivo_sessao_inexistente_12345.md");
    assert!(app.load_session_file(missing_path).is_err());

    // Mutante 2: Carregar arquivo com sintaxe corrompida
    let tmp = tempdir().expect("tempdir");
    let corrupt_file = tmp.path().join("corrupt.md");
    std::fs::write(&corrupt_file, "conteudo invalido sem frontmatter").expect("write corrupt");
    assert!(app.load_session_file(&corrupt_file).is_err());

    // Mutante 3: Branch com múltiplos níveis e caracteres especiais
    assert_eq!(
        infer_ticket_from_branch("lumen/feat/88-deep-branch-test"),
        Some(88)
    );
    assert_eq!(
        infer_ticket_from_branch("user/fix/7-quick_fix_suite"),
        Some(7)
    );
    assert_eq!(infer_ticket_from_branch("   feat/55-spaces-trimmed   "), Some(55));
    assert_eq!(infer_ticket_from_branch("prefix-without-slash"), None);
}
