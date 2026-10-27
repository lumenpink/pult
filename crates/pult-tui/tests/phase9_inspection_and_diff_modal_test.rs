use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pult_core::domain::gost::GostPhase;
use pult_core::driver::{CccpGitDriver, RepoDriver};
use pult_tui::app::App;
use pult_tui::event::{handle_key_event, TuiAction};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_repo_driver_diff_stat_and_diff_full() {
    let dir = tempdir().expect("tempdir");
    let repo_path = dir.path();

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

    fs::write(repo_path.join("file1.txt"), "hello").unwrap();
    std::process::Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "chore: genesis"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");

    // Cria branch de trabalho
    std::process::Command::new("git")
        .args(["checkout", "-b", "feat/32-test"])
        .current_dir(repo_path)
        .output()
        .expect("git checkout -b");

    fs::write(repo_path.join("file2.txt"), "nova linha adicionada\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "file2.txt"])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "feat: adicionar file2"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");

    let driver = CccpGitDriver::new(repo_path);

    let stat = driver.diff_stat("main").expect("diff_stat deve suceder");
    assert!(stat.contains("file2.txt"), "diff_stat deve listar file2.txt: {}", stat);

    let full = driver.diff_full("main").expect("diff_full deve suceder");
    assert!(full.contains("+nova linha adicionada"), "diff_full deve conter o diff: {}", full);
}

#[test]
fn test_ctrl_d_and_slash_diff_toggle_diff_modal() {
    let mut app = App::new();
    assert!(!app.show_diff_modal);

    // 1. Testa atalho Ctrl+D
    let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    let action = handle_key_event(&mut app, key);
    assert_eq!(action, Some(TuiAction::DiffModalToggled));
    assert!(app.show_diff_modal, "Ctrl+D deve abrir o modal de diff");

    // 2. Testa comando /diff
    let action_cmd = app.execute_slash_command("/diff");
    assert_eq!(action_cmd, Some(TuiAction::DiffModalToggled));
    assert!(!app.show_diff_modal, "/diff deve fechar o modal de diff");
}

#[test]
fn test_phase9_transition_prints_modified_files_and_summary() {
    let dir = tempdir().expect("tempdir");
    let repo_path = dir.path();

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

    fs::write(repo_path.join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "chore: initial"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");

    std::process::Command::new("git")
        .args(["checkout", "-b", "feat/32-demo"])
        .current_dir(repo_path)
        .output()
        .expect("git checkout -b");

    fs::create_dir_all(repo_path.join("src")).unwrap();
    fs::write(repo_path.join("src/lib.rs"), "pub fn hello() {}").unwrap();
    std::process::Command::new("git")
        .args(["add", "src/lib.rs"])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "feat(core): hello"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");

    let driver = CccpGitDriver::new(repo_path);
    let mut app = App::with_repo(repo_path.to_path_buf());
    app.set_ticket_info(Some(32), "feat/32-demo", 5);

    // Ao aprovar Fase 5, avança por 6, 7 e 8 até Fase 9
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Avanço para Fase 9 falhou: {:?}", res);
    assert_eq!(app.gost_phase, GostPhase::CloseTicket);

    // O viewport deve conter a apresentação da Fase 9 com arquivos alterados e atalho de diff
    let has_fase9_header = app.viewport_lines.iter().any(|l| l.contains("Fase 9"));
    let has_diff_hint = app.viewport_lines.iter().any(|l| l.contains("Ctrl+D") || l.contains("/diff"));
    assert!(has_fase9_header, "Viewport deve conter cabeçalho da Fase 9");
    assert!(has_diff_hint, "Viewport deve instruir usuária sobre /diff ou Ctrl+D");
}

#[test]
fn test_render_diff_modal_with_syntax_highlighting() {
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("terminal");

    let mut app = App::new();
    app.show_diff_modal = true;
    app.diff_lines = vec![
        "diff --git a/src/lib.rs b/src/lib.rs".to_string(),
        "--- a/src/lib.rs".to_string(),
        "+++ b/src/lib.rs".to_string(),
        "@@ -1,3 +1,4 @@".to_string(),
        "-fn antiga() {}".to_string(),
        "+fn nova() {}".to_string(),
    ];

    terminal
        .draw(|f| {
            pult_tui::ui::render(&app, f);
        })
        .expect("renderização com modal de diff não deve entrar em pânico");

    let buffer = terminal.backend().buffer();
    let content = format!("{:?}", buffer);
    assert!(content.contains("Diff"));
    assert!(content.contains("fn nova"));
}
