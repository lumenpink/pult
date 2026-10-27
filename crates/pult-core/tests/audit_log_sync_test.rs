use pult_core::audit::AuditLogger;
use pult_core::scout::ScoutEngine;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_audit_logger_retarget_switches_file_and_preserves_logs() {
    let tmp = tempdir().expect("tempdir");
    let log1 = tmp.path().join("t54026-draft.log");
    let log2 = tmp.path().join("000023-formal.log");

    let logger = AuditLogger::new(&log1).expect("create logger");
    logger.log("START", "Sessao de rascunho iniciada");

    assert!(log1.exists());
    let content1 = fs::read_to_string(&log1).expect("read log1");
    assert!(content1.contains("Sessao de rascunho iniciada"));

    // Retarget para formal
    logger.retarget(log2.clone()).expect("retarget log");
    assert_eq!(logger.file_path(), log2);

    logger.log("TICKET_LINK", "Ticket #23 associado com sucesso");

    assert!(log2.exists());
    let content2 = fs::read_to_string(&log2).expect("read log2");
    assert!(content2.contains("Ticket #23 associado com sucesso"));
}

#[test]
fn test_scout_result_carries_full_prompt_and_raw_response() {
    let raw_resp = "```yaml\ninjected_ast:\n  - \"fn process\"\n```";
    let res = ScoutEngine::parse_scout_response_with_payloads(
        raw_resp,
        "qwen2.5-coder:3b",
        Some(1024),
        "FULL_PROMPT_SENT_TO_OLLAMA",
        raw_resp,
    )
    .expect("parsed");

    assert_eq!(res.full_prompt, "FULL_PROMPT_SENT_TO_OLLAMA");
    assert_eq!(res.raw_response, raw_resp);
    assert_eq!(res.symbols, vec!["fn process"]);
}

#[test]
fn test_mutation_audit_logger_retarget_creates_deep_nested_directory() {
    let tmp = tempdir().expect("tempdir");
    let log_initial = tmp.path().join("draft.log");
    let deep_log = tmp.path().join("a/b/c/nested_logs/formal.log");

    let logger = AuditLogger::new(&log_initial).expect("create logger");
    logger.retarget(deep_log.clone()).expect("deep retarget");

    assert_eq!(logger.file_path(), deep_log);
    logger.log("DEEP", "Mensagem gravada em pasta aninhada");

    assert!(deep_log.exists());
    let content = fs::read_to_string(&deep_log).expect("read deep log");
    assert!(content.contains("Mensagem gravada em pasta aninhada"));
}

#[test]
fn test_mutation_audit_logger_retarget_idempotent_same_path() {
    let tmp = tempdir().expect("tempdir");
    let log_path = tmp.path().join("same.log");

    let logger = AuditLogger::new(&log_path).expect("create logger");
    logger.log("STEP1", "Primeiro passo");

    // Retarget para o mesmo caminho
    logger.retarget(log_path.clone()).expect("same path retarget");
    logger.log("STEP2", "Segundo passo");

    let content = fs::read_to_string(&log_path).expect("read log");
    assert!(content.contains("Primeiro passo"));
    assert!(content.contains("Segundo passo"));
}

#[test]
fn test_mutation_scout_result_bypassed_payload_defaults() {
    let bypassed = pult_core::scout::models::ScoutResult::bypassed();
    assert!(bypassed.bypassed);
    assert!(bypassed.full_prompt.is_empty());
    assert!(bypassed.raw_response.is_empty());
}
