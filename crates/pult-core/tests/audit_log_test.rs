use pult_core::audit::AuditLogger;
use tempfile::tempdir;

#[test]
fn test_audit_logger_creates_file_and_logs_events() {
    let tmp = tempdir().expect("tempdir");
    let log_path = tmp.path().join("logs/sub/interacoes.log");

    let logger = AuditLogger::new(&log_path).expect("criar logger");

    logger.log("USER_INPUT", "Criar spec da fase 3");
    logger.log("SCOUT_REQUEST", "Prompt com 12 assinaturas AST");
    logger.log(
        "SCOUT_RESPONSE",
        "Modelo deepseek-r1:14b retornou 3 símbolos",
    );
    logger.log("AGY_DISPATCH", "agy --print --dangerously-skip-permissions");
    logger.log("AGY_STREAM", "Chunk inicial de saída");
    logger.log("QUALITY_GATE", "cccp check 100% conforme");

    assert!(log_path.is_file(), "arquivo de log deve ter sido criado");

    let content = std::fs::read_to_string(&log_path).expect("ler log");
    assert!(content.contains("[USER_INPUT] Criar spec da fase 3"));
    assert!(content.contains("[SCOUT_REQUEST] Prompt com 12 assinaturas AST"));
    assert!(content.contains("[SCOUT_RESPONSE] Modelo deepseek-r1:14b retornou 3 símbolos"));
    assert!(content.contains("[AGY_DISPATCH] agy --print"));
    assert!(content.contains("[AGY_STREAM] Chunk inicial de saída"));
    assert!(content.contains("[QUALITY_GATE] cccp check 100% conforme"));
}

#[test]
fn test_audit_logger_appends_across_multiple_instances() {
    let tmp = tempdir().expect("tempdir");
    let log_path = tmp.path().join("audit.log");

    {
        let logger1 = AuditLogger::new(&log_path).expect("logger1");
        logger1.log("SESSION_START", "Sessão 001");
    }

    {
        let logger2 = AuditLogger::new(&log_path).expect("logger2");
        logger2.log("SESSION_START", "Sessão 002");
    }

    let content = std::fs::read_to_string(&log_path).expect("ler log");
    assert!(content.contains("[SESSION_START] Sessão 001"));
    assert!(content.contains("[SESSION_START] Sessão 002"));
}
