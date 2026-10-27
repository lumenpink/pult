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
    assert!(
        stat.contains("file2.txt"),
        "diff_stat deve listar file2.txt: {}",
        stat
    );

    let full = driver.diff_full("main").expect("diff_full deve suceder");
    assert!(
        full.contains("+nova linha adicionada"),
        "diff_full deve conter o diff: {}",
        full
    );
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
    struct MockPhase9Driver {
        branch: String,
    }

    impl RepoDriver for MockPhase9Driver {
        fn current_branch(&self) -> Result<String, pult_core::driver::DriverError> {
            Ok(self.branch.clone())
        }
        fn checkout_branch(
            &self,
            _branch: &str,
            _create: bool,
        ) -> Result<(), pult_core::driver::DriverError> {
            Ok(())
        }
        fn merge_branch(&self, _branch: &str) -> Result<(), pult_core::driver::DriverError> {
            Ok(())
        }
        fn status(&self) -> Result<pult_core::driver::GitStatus, pult_core::driver::DriverError> {
            Ok(pult_core::driver::GitStatus {
                branch: self.branch.clone(),
                is_clean: true,
                modified_files: Vec::new(),
                untracked_files: Vec::new(),
            })
        }
        fn check_strict(
            &self,
        ) -> Result<pult_core::driver::QualityAudit, pult_core::driver::DriverError> {
            Ok(pult_core::driver::QualityAudit {
                passed: true,
                total_checks: 9,
                conforming: 9,
                warnings: 0,
                failures: 0,
                compliance_index: "100%".into(),
            })
        }
        fn format_commit_message(
            &self,
            _phase: GostPhase,
            ticket_id: u32,
            _scope: Option<&str>,
            subject: &str,
            _body: Option<&str>,
        ) -> Result<String, pult_core::driver::DriverError> {
            Ok(format!("feat(core): {} (ref #{})", subject, ticket_id))
        }
        fn validate_phase_diff(
            &self,
            _phase: GostPhase,
        ) -> Result<(), pult_core::driver::DriverError> {
            Ok(())
        }
        fn commit_phase(
            &self,
            _phase: GostPhase,
            _ticket_id: u32,
            _scope: Option<&str>,
            _subject: &str,
            _body: Option<&str>,
        ) -> Result<String, pult_core::driver::DriverError> {
            Ok("commit-sha".into())
        }
        fn close_ticket(&self, _ticket_id: u32) -> Result<(), pult_core::driver::DriverError> {
            Ok(())
        }
        fn create_ticket(
            &self,
            _title: &str,
            _body: &str,
        ) -> Result<(u32, String), pult_core::driver::DriverError> {
            Ok((32, "feat/32-demo".into()))
        }
        fn detect_branch_phase(
            &self,
            _branch: &str,
            _ticket_id: Option<u32>,
        ) -> Result<GostPhase, pult_core::driver::DriverError> {
            Ok(GostPhase::GreenCode)
        }
        fn diff_stat(&self, _base_branch: &str) -> Result<String, pult_core::driver::DriverError> {
            Ok(" src/lib.rs | 2 +-\n 1 file changed, 1 insertion(+), 1 deletion(-)".into())
        }
        fn diff_full(&self, _base_branch: &str) -> Result<String, pult_core::driver::DriverError> {
            Ok("+pub fn hello() {}".into())
        }
    }

    let driver = MockPhase9Driver {
        branch: "feat/32-demo".into(),
    };
    let mut app = App::new();
    app.set_ticket_info(Some(32), "feat/32-demo", 5);

    // Ao aprovar Fase 5, avança por 6, 7 e 8 até Fase 9
    let res = app.execute_action(TuiAction::SpecApproved, &driver);
    assert!(res.is_ok(), "Avanço para Fase 9 falhou: {:?}", res);
    assert_eq!(app.gost_phase, GostPhase::CloseTicket);

    // O viewport deve conter a apresentação da Fase 9 com arquivos alterados e atalho de diff
    let has_fase9_header = app.viewport_lines.iter().any(|l| l.contains("Fase 9"));
    let has_diff_hint = app
        .viewport_lines
        .iter()
        .any(|l| l.contains("Ctrl+D") || l.contains("/diff"));
    assert!(has_fase9_header, "Viewport deve conter cabeçalho da Fase 9");
    assert!(
        has_diff_hint,
        "Viewport deve instruir usuária sobre /diff ou Ctrl+D"
    );
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
