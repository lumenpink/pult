use pult_core::domain::GostPhase;
use pult_core::driver::*;
use std::env;

#[test]
fn test_format_commit_message_by_phase() {
    let driver = CccpGitDriver::new(env::current_dir().unwrap());

    // Fases 1 e 2 não geram commits no Git
    assert!(driver
        .format_commit_message(GostPhase::Ticket, 42, Some("core"), "formula escopo", None)
        .is_err());
    assert!(driver
        .format_commit_message(GostPhase::NewBranch, 42, Some("core"), "cria branch", None)
        .is_err());

    // Fase 3 (SpecContract)
    let msg3 = driver
        .format_commit_message(
            GostPhase::SpecContract,
            42,
            Some("auth"),
            "especifica trait",
            None,
        )
        .expect("fase 3 formata");
    assert!(msg3.starts_with("docs(auth): especifica trait (ref #42)"));

    // Fase 4 (RedTest)
    let msg4 = driver
        .format_commit_message(GostPhase::RedTest, 42, Some("auth"), "teste falho", None)
        .expect("fase 4 formata");
    assert!(msg4.starts_with("test(auth): teste falho (ref #42)"));

    // Fase 5 (GreenCode)
    let msg5 = driver
        .format_commit_message(
            GostPhase::GreenCode,
            42,
            Some("auth"),
            "implementa token",
            None,
        )
        .expect("fase 5 formata");
    assert!(msg5.starts_with("feat(auth): implementa token (ref #42)"));

    // Fase 6 (Refactor)
    let msg6 = driver
        .format_commit_message(
            GostPhase::Refactor,
            42,
            Some("auth"),
            "otimiza bucket",
            None,
        )
        .expect("fase 6 formata");
    assert!(msg6.starts_with("refactor(auth): otimiza bucket (ref #42)"));

    // Fase 7 (MutationTest)
    let msg7 = driver
        .format_commit_message(
            GostPhase::MutationTest,
            42,
            Some("auth"),
            "elimina mutantes",
            None,
        )
        .expect("fase 7 formata");
    assert!(msg7.starts_with("test(auth): elimina mutantes (ref #42)"));

    // Fase 8 (QualityGate)
    let msg8 = driver
        .format_commit_message(
            GostPhase::QualityGate,
            42,
            Some("auth"),
            "formata lint",
            None,
        )
        .expect("fase 8 formata");
    assert!(msg8.starts_with("fix(auth): formata lint (ref #42)"));

    // Fase 9 (CloseTicket)
    let msg9 = driver
        .format_commit_message(
            GostPhase::CloseTicket,
            42,
            Some("auth"),
            "finaliza modulo",
            None,
        )
        .expect("fase 9 formata");
    assert!(msg9.starts_with("docs(auth): finaliza modulo (closes #42)"));
}

#[test]
fn test_format_commit_message_header_limit() {
    let driver = CccpGitDriver::new(env::current_dir().unwrap());
    let long_subject = "esta e uma mensagem de assunto excessivamente longa criada para estourar o limite rigido de 72 caracteres do gosplan";

    let result =
        driver.format_commit_message(GostPhase::GreenCode, 42, Some("auth"), long_subject, None);

    assert!(result.is_err());
    match result {
        Err(DriverError::HeaderTooLong { length, max }) => {
            assert!(length > 72);
            assert_eq!(max, 72);
        }
        other => panic!("esperava HeaderTooLong, obteve {:?}", other),
    }
}

#[test]
fn test_current_branch_and_status() {
    let driver = CccpGitDriver::new(env::current_dir().unwrap());
    let branch = driver.current_branch().expect("deve obter branch atual");
    assert!(!branch.is_empty());

    let status = driver.status().expect("deve obter status git");
    assert_eq!(status.branch, branch);
}
