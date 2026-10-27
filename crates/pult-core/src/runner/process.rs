use crate::domain::{Actor, Session, SessionStorage, Turn, TurnStatus};
use crate::runner::error::RunnerError;
use crate::runner::events::{AgyStreamEvent, ResumeInspection, RunnerOptions};
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

fn current_iso_time() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = now / 86400;
    let rem = now % 86400;
    let hours = rem / 3600;
    let minutes = (rem % 3600) / 60;
    let seconds = rem % 60;

    // Aproximação gregoriana canônica
    let mut year = 1970;
    let mut d = days;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        year += 1;
    }

    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1;
    for &dim in &days_in_months {
        if d < dim {
            break;
        }
        d -= dim;
        month += 1;
    }
    let day = d + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

pub type RunnerStream = (
    mpsc::Receiver<AgyStreamEvent>,
    JoinHandle<Result<i32, RunnerError>>,
);

/// Orquestrador assíncrono para execução do binário da CLI Antigravity (`agy`).
pub struct AgyRunner {
    options: RunnerOptions,
}

impl Default for AgyRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl AgyRunner {
    pub fn new() -> Self {
        Self::with_options(RunnerOptions::default())
    }

    pub fn with_options(options: RunnerOptions) -> Self {
        Self { options }
    }

    pub fn options(&self) -> &RunnerOptions {
        &self.options
    }

    pub fn spawn(&self) -> Result<RunnerStream, RunnerError> {
        let mut cmd = Command::new(&self.options.command);
        cmd.args(&self.options.args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Some(ref dir) = self.options.working_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn().map_err(|e| RunnerError::Io { source: e })?;

        let mut stdout = child.stdout.take();
        let mut stderr = child.stderr.take();

        let (tx, rx) = mpsc::channel(100);

        let tx_out = tx.clone();
        tokio::spawn(async move {
            if let Some(ref mut out) = stdout {
                let mut buf = [0u8; 1024];
                while let Ok(n) = out.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = tx_out.send(AgyStreamEvent::StdoutChunk(chunk)).await;
                }
            }
        });

        let tx_err = tx.clone();
        tokio::spawn(async move {
            if let Some(ref mut err) = stderr {
                let mut buf = [0u8; 1024];
                while let Ok(n) = err.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = tx_err.send(AgyStreamEvent::StderrChunk(chunk)).await;
                }
            }
        });

        let child_handle = tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .map_err(|e| RunnerError::Io { source: e })?;
            let exit_code = status.code().unwrap_or(-1);
            let _ = tx.send(AgyStreamEvent::Completed { exit_code }).await;
            Ok(exit_code)
        });

        Ok((rx, child_handle))
    }

    pub async fn execute_turn_and_persist(
        &self,
        session: &mut Session,
        storage: &dyn SessionStorage,
    ) -> Result<Session, RunnerError> {
        let turn_number = session.turns.len() + 1;
        let now = current_iso_time();
        let current_turn = Turn {
            turn_number,
            actor: Actor::Agy,
            timestamp: now,
            content: String::new(),
            status: TurnStatus::InProgress,
            tokens: None,
            tokens_out: None,
            elapsed_ms: None,
        };

        session.turns.push(current_turn);
        storage.save(session).map_err(RunnerError::Session)?;

        let (mut rx, child_handle) = self.spawn()?;
        let mut accumulated = String::new();
        let mut is_truncated = false;

        while let Some(event) = rx.recv().await {
            match event {
                AgyStreamEvent::StdoutChunk(chunk) => {
                    accumulated.push_str(&chunk);
                    if accumulated.contains("STATUS: truncated") {
                        is_truncated = true;
                    }
                    if let Some(t) = session.turns.last_mut() {
                        t.content = accumulated.clone();
                    }
                    let _ = storage.save(session);
                }
                AgyStreamEvent::StderrChunk(_) => {}
                AgyStreamEvent::Truncated => {
                    is_truncated = true;
                }
                AgyStreamEvent::Completed { exit_code } => {
                    if exit_code != 0 && !is_truncated {
                        if let Some(t) = session.turns.last_mut() {
                            t.status = TurnStatus::Interrupted;
                        }
                    }
                }
                AgyStreamEvent::Error(err) => {
                    if let Some(t) = session.turns.last_mut() {
                        t.status = TurnStatus::Interrupted;
                    }
                    return Err(RunnerError::ProcessFailed(err));
                }
            }
        }

        let _ = child_handle.await;

        if let Some(t) = session.turns.last_mut() {
            if is_truncated {
                t.status = TurnStatus::Truncated;
            } else if t.status == TurnStatus::InProgress {
                t.status = TurnStatus::Completed;
            }
        }

        storage.save(session).map_err(RunnerError::Session)?;
        Ok(session.clone())
    }

    pub fn inspect_session_for_resume(
        session_path: &Path,
        active_branch: &str,
    ) -> Result<ResumeInspection, RunnerError> {
        let content =
            fs::read_to_string(session_path).map_err(|e| RunnerError::Io { source: e })?;
        let session = Session::from_markdown(&content).map_err(RunnerError::Session)?;

        let last_turn_status = session.turns.last().map(|t| t.status);
        let needs_continuation = matches!(
            last_turn_status,
            Some(TurnStatus::InProgress) | Some(TurnStatus::Truncated)
        );

        let branch_matches = session
            .metadata
            .ticket
            .as_ref()
            .map(|t| t.branch == active_branch)
            .unwrap_or(true);

        Ok(ResumeInspection {
            session_id: session.metadata.session_id,
            needs_continuation,
            branch_matches,
            last_turn_status,
        })
    }
}
