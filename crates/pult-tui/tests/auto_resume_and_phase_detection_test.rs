use pult_core::domain::GostPhase;
use pult_core::driver::{DriverError, GitStatus, QualityAudit, RepoDriver};
use pult_tui::app::App;
use pult_tui::event::TuiAction;
use std::fs;
use tempfile::tempdir;

struct MockPhaseDetectionDriver {
    pub current_branch_name: String,
    pub detected_phase: GostPhase,
}

impl RepoDriver for MockPhaseDetectionDriver {
    fn current_branch(&self) -> Result<String, DriverError> {
        Ok(self.current_branch_name.clone())
    }

    fn checkout_branch(&self, _branch: &str, _create: bool) -> Result<(), DriverError> {
        Ok(())
    }

    fn merge_branch(&self, _branch: &str) -> Result<(), DriverError> {
        Ok(())
    }

    fn status(&self) -> Result<GitStatus, DriverError> {
        Ok(GitStatus {
            branch: self.current_branch_name.clone(),
            is_clean: true,
            modified_files: Vec::new(),
            untracked_files: Vec::new(),
        })
    }

    fn check_strict(&self) -> Result<QualityAudit, DriverError> {
        Ok(QualityAudit {
            passed: true,
            total_checks: 10,
            conforming: 10,
            warnings: 0,
            failures: 0,
            compliance_index: "100%".into(),
        })
    }

    fn format_commit_message(
        &self,
        phase: GostPhase,
        ticket_id: u32,
        scope: Option<&str>,
        subject: &str,
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        Ok(format!(
            "{}({}): {} (ref #{} - ph{})",
            phase.commit_prefix().unwrap_or("feat"),
            scope.unwrap_or("tui"),
            subject,
            ticket_id,
            phase.number()
        ))
    }

    fn validate_phase_diff(&self, _phase: GostPhase) -> Result<(), DriverError> {
        Ok(())
    }

    fn commit_phase(
        &self,
        phase: GostPhase,
        ticket_id: u32,
        scope: Option<&str>,
        subject: &str,
        _body: Option<&str>,
    ) -> Result<String, DriverError> {
        self.format_commit_message(phase, ticket_id, scope, subject, _body)
    }

    fn create_ticket(&self, _title: &str, _body: &str) -> Result<(u32, String), DriverError> {
        Ok((27, "feat/27-test".to_string()))
    }

    fn close_ticket(&self, _ticket_id: u32) -> Result<(), DriverError> {
        Ok(())
    }

    fn detect_branch_phase(
        &self,
        _branch: &str,
        _ticket_id: Option<u32>,
    ) -> Result<GostPhase, DriverError> {
        Ok(self.detected_phase)
    }
}

#[test]
fn test_find_session_for_ticket_by_prefix_and_frontmatter() {
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().to_path_buf();
    let sessions_dir = repo_dir.join(".antigravity/pult/sessions");
    fs::create_dir_all(&sessions_dir).unwrap();

    // 1. Arquivo canônico formal 000002-script-shell.md
    let formal_file = sessions_dir.join("000002-script-shell.md");
    let sess_2 = pult_core::Session::new_formal(2, "script-shell", "feat/2-script");
    fs::write(&formal_file, sess_2.to_markdown().unwrap()).unwrap();

    let found = App::find_session_for_ticket(&repo_dir, 2, "feat/2-script");
    assert_eq!(found, Some(formal_file));

    // 2. Arquivo numérico curto 42-auth.md
    let short_file = sessions_dir.join("42-auth.md");
    let sess_42 = pult_core::Session::new_formal(42, "auth", "feat/42-auth");
    fs::write(&short_file, sess_42.to_markdown().unwrap()).unwrap();

    let found_42 = App::find_session_for_ticket(&repo_dir, 42, "feat/42-auth");
    assert_eq!(found_42, Some(short_file));

    // 3. Arquivo com slug diferente mas com frontmatter apontando para ticket 99
    let arbitrary_file = sessions_dir.join("sessao-rascunho-antiga.md");
    let sess_99 = pult_core::Session::new_formal(99, "antiga-99", "fix/99-bug");
    fs::write(&arbitrary_file, sess_99.to_markdown().unwrap()).unwrap();

    let found_99 = App::find_session_for_ticket(&repo_dir, 99, "fix/99-bug");
    assert_eq!(found_99, Some(arbitrary_file));

    // 4. Ticket inexistente retorna None
    let not_found = App::find_session_for_ticket(&repo_dir, 999, "feat/999-none");
    assert!(not_found.is_none());
}

#[test]
fn test_auto_sync_branch_state_restores_session_and_phase() {
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().to_path_buf();
    let sessions_dir = repo_dir.join(".antigravity/pult/sessions");
    fs::create_dir_all(&sessions_dir).unwrap();

    let session_file = sessions_dir.join("000027-auto-resume.md");
    let sess_27 = pult_core::Session::new_formal(27, "auto-resume", "feat/27-auto-resume");
    fs::write(&session_file, sess_27.to_markdown().unwrap()).unwrap();

    let mut app = App::with_repo(repo_dir);
    let driver = MockPhaseDetectionDriver {
        current_branch_name: "feat/27-auto-resume".to_string(),
        detected_phase: GostPhase::RedTest, // Fase 4 inferida pelos commits
    };

    app.auto_sync_branch_state("feat/27-auto-resume", &driver);

    assert_eq!(app.ticket_id(), Some(27));
    assert_eq!(app.branch_name(), "feat/27-auto-resume");
    assert_eq!(app.phase(), GostPhase::RedTest);
    assert!(app.session_id().is_some());
}

#[test]
fn test_branch_changed_action_reconciles_session_and_phase() {
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().to_path_buf();
    let sessions_dir = repo_dir.join(".antigravity/pult/sessions");
    fs::create_dir_all(&sessions_dir).unwrap();

    let session_file = sessions_dir.join("000005-feature-cinco.md");
    fs::write(
        &session_file,
        "---\nsession_id: 000005-feature-cinco\nticket:\n  id: 5\n  branch: feat/5-feature-cinco\n---\n<!-- TURN 1: user -->\nContexto da feature cinco\n",
    )
    .unwrap();

    let mut app = App::with_repo(repo_dir);
    let driver = MockPhaseDetectionDriver {
        current_branch_name: "feat/5-feature-cinco".to_string(),
        detected_phase: GostPhase::GreenCode, // Fase 5
    };

    let res = app.execute_action(
        TuiAction::BranchChanged("feat/5-feature-cinco".into()),
        &driver,
    );
    assert!(res.is_ok());
    assert_eq!(app.ticket_id(), Some(5));
    assert_eq!(app.phase(), GostPhase::GreenCode);
}

#[test]
fn test_mutation_sessions_dir_nonexistent_returns_none() {
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().to_path_buf(); // sem .antigravity/pult/sessions

    let res = App::find_session_for_ticket(&repo_dir, 1, "feat/1-test");
    assert!(res.is_none());
}

#[test]
fn test_mutation_empty_sessions_dir_returns_none() {
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().to_path_buf();
    let sessions_dir = repo_dir.join(".antigravity/pult/sessions");
    fs::create_dir_all(&sessions_dir).unwrap();

    let res = App::find_session_for_ticket(&repo_dir, 1, "feat/1-test");
    assert!(res.is_none());
}

#[test]
fn test_mutation_main_branch_returns_ticket_phase() {
    let tmp = tempdir().unwrap();
    let driver = pult_core::CccpGitDriver::new(tmp.path());

    let phase_main = driver.detect_branch_phase("main", None).unwrap();
    assert_eq!(phase_main, GostPhase::Ticket);

    let phase_master = driver.detect_branch_phase("master", None).unwrap();
    assert_eq!(phase_master, GostPhase::Ticket);
}

#[test]
fn test_mutation_auto_sync_branch_state_idempotent() {
    let tmp = tempdir().unwrap();
    let repo_dir = tmp.path().to_path_buf();
    let mut app = App::with_repo(repo_dir);
    let driver = MockPhaseDetectionDriver {
        current_branch_name: "feat/10-idempotent".to_string(),
        detected_phase: GostPhase::SpecContract,
    };

    app.auto_sync_branch_state("feat/10-idempotent", &driver);
    assert_eq!(app.ticket_id(), Some(10));
    assert_eq!(app.phase(), GostPhase::SpecContract);

    app.auto_sync_branch_state("feat/10-idempotent", &driver);
    assert_eq!(app.ticket_id(), Some(10));
    assert_eq!(app.phase(), GostPhase::SpecContract);
}
