use pult_core::ast::{AstExtractor, PolyglotAstExtractor};
use pult_core::domain::{
    Actor, FileSessionStorage, GostPhase, Session, SessionStorage, Turn, TurnStatus,
};
use pult_core::runner::{AgyRunner, RunnerOptions};
use pult_core::scout::{OllamaClient, ScoutEngine};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_e2e_full_workflow_orchestration() {
    let temp_dir = tempdir().expect("tempdir");
    let workspace = temp_dir.path();

    // 1. Cria base de código poliglota
    let src_dir = workspace.join("src");
    fs::create_dir_all(&src_dir).expect("mkdir src");
    fs::write(
        src_dir.join("lib.rs"),
        "pub struct RateLimiter { pub max_tokens: usize }\nimpl RateLimiter { pub fn check(&self) -> bool { true } }",
    )
    .expect("write rs");
    fs::write(
        src_dir.join("service.py"),
        "class AuthService:\n    def authenticate(self, user: str) -> bool:\n        return True\n",
    )
    .expect("write py");
    fs::write(
        src_dir.join("api.ts"),
        "export interface UserApi { getUser(id: string): Promise<string>; }\n",
    )
    .expect("write ts");

    // 2. Extração Estrutural de AST Poliglota
    let extractor = PolyglotAstExtractor::new();
    let ast_files = extractor
        .extract_from_dir(workspace, &[".git", "target"])
        .expect("extrair ast da base");

    assert_eq!(ast_files.len(), 3);
    for file in &ast_files {
        assert!(file.reduction_ratio() >= 0.0);
    }

    // 3. Filtragem Cirúrgica Scout
    let scout = ScoutEngine::with_client(OllamaClient::with_base_url("http://127.0.0.1:59999"));
    let scout_result = scout
        .filter_with_bypass("Implementar controle de acesso", &ast_files)
        .await
        .expect("scout filter");
    assert!(scout_result.bypassed || !scout_result.symbols.is_empty());

    // 4. Sessão e Persistência Concorrente com Runner
    let storage_dir = workspace.join(".antigravity/pult/sessions");
    let storage = FileSessionStorage::new(&storage_dir);

    let mut session = Session::new_formal(10, "e2e-suite", "test/10-e2e-quality-gate");
    session.metadata.gost.phase = GostPhase::GreenCode.number();
    session.turns.push(Turn {
        turn_number: 1,
        actor: Actor::User,
        timestamp: "2026-10-26T16:00:00Z".to_string(),
        content: "Validar suíte E2E".to_string(),
        status: TurnStatus::Completed,
        tokens: Some(15),
        tokens_out: None,
    });
    storage.save(&session).expect("salvar sessao inicial");

    let runner = AgyRunner::with_options(RunnerOptions {
        command: "sh".to_string(),
        args: vec![
            "-c".to_string(),
            "echo 'Iniciando testes de integração...'; echo 'Todos os módulos conformes.'".to_string(),
        ],
        working_dir: Some(workspace.to_path_buf()),
    });

    let final_session = runner
        .execute_turn_and_persist(&mut session, &storage)
        .await
        .expect("executar runner e persistir");

    assert_eq!(final_session.turns.len(), 2);
    assert_eq!(final_session.turns[1].actor, Actor::Agy);
    assert_eq!(final_session.turns[1].status, TurnStatus::Completed);
    assert!(final_session.turns[1].content.contains("Todos os módulos conformes"));

    // 5. Validação de Carregamento e Reconciliação
    let reloaded = storage
        .load(&session.metadata.session_id)
        .expect("recarregar sessao do disco");
    assert_eq!(reloaded.turns.len(), 2);
    assert_eq!(reloaded.metadata.ticket.as_ref().unwrap().id, 10);
}

#[test]
fn test_e2e_deploy_assets_and_security_headers() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(manifest_dir);

    let headers_path = repo_root.join("site/_headers");
    let headers_content = fs::read_to_string(&headers_path).expect("ler site/_headers");
    assert!(headers_content.contains("X-Content-Type-Options: nosniff"));
    assert!(headers_content.contains("X-Frame-Options: DENY"));
    assert!(headers_content.contains("Referrer-Policy: strict-origin-when-cross-origin"));

    let wrangler_path = repo_root.join("wrangler.jsonc");
    let wrangler_content = fs::read_to_string(&wrangler_path).expect("ler wrangler.jsonc");
    assert!(wrangler_content.contains("pult.2lp.in"));
    assert!(wrangler_content.contains("./site"));
}
