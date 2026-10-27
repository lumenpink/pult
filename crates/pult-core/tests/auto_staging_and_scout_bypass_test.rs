use pult_core::ast::ExtractedAstFile;
use pult_core::domain::gost::GostPhase;
use pult_core::driver::{CccpGitDriver, RepoDriver};
use pult_core::scout::ScoutEngine;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_repo_driver_commit_phase_auto_stages_untracked_files() {
    let dir = tempdir().expect("tempdir");
    let repo_path = dir.path();

    // Inicializa git repo local
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(repo_path)
        .output()
        .expect("git init");

    std::process::Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(repo_path)
        .output()
        .expect("git config user.name");

    std::process::Command::new("git")
        .args(["config", "user.email", "tester@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("git config user.email");

    // Commit inicial
    fs::write(repo_path.join("README.md"), "# Initial").unwrap();
    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "chore: genesis"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");

    let driver = CccpGitDriver::new(repo_path.to_path_buf());

    // Cria um arquivo untracked
    fs::create_dir_all(repo_path.join("docs")).unwrap();
    fs::write(repo_path.join("docs/spec.md"), "# Spec").unwrap();

    // commit_phase deve fazer o stage automático e comitar sem erro
    let res = driver.commit_phase(
        GostPhase::SpecContract,
        29,
        Some("spec"),
        "especificar contrato",
        None,
    );
    assert!(res.is_ok(), "commit_phase falhou ao comitar arquivo untracked: {:?}", res);

    // Verifica que o arquivo agora está comitado e o working tree está limpo
    let status_output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .expect("git status");
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        status_str.trim().is_empty(),
        "Working tree deveria estar limpo após o commit_phase: {}",
        status_str
    );
}

#[tokio::test]
async fn test_scout_bypass_on_empty_ast_files() {
    let engine = ScoutEngine::default();
    let res = engine
        .filter_with_bypass("implemente uma funcionalidade nova", &[])
        .await;

    assert!(res.is_ok());
    let scout_result = res.unwrap();
    assert!(
        scout_result.symbols.is_empty(),
        "AST vazio deve retornar símbolos vazios via bypass"
    );
    assert!(scout_result.bypassed, "Deve estar marcado como bypassed");
}

#[tokio::test]
async fn test_scout_bypass_on_short_confirmations() {
    let engine = ScoutEngine::default();
    let dummy_ast = vec![ExtractedAstFile {
        file_path: PathBuf::from("src/main.rs"),
        relative_path: "src/main.rs".into(),
        extracted_signatures: "fn main()".into(),
        raw_byte_count: 50,
        reduced_byte_count: 10,
    }];

    for prompt in ["y", "Y", "yes", "sim", "Sim", "s", "ok", "OK.", "concordo", "aprovado"] {
        let res = engine.filter_with_bypass(prompt, &dummy_ast).await;
        assert!(res.is_ok(), "Falha no bypass para prompt '{}': {:?}", prompt, res);
        let scout_result = res.unwrap();
        assert!(
            scout_result.symbols.is_empty(),
            "Confirmação '{}' deve acionar bypass imediato",
            prompt
        );
        assert!(scout_result.bypassed, "Deve estar marcado como bypassed");
    }
}
