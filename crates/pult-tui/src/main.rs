//! PULT (Пульт) — Estação de trabalho TUI autônoma para terminal.

use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pult_core::{AstExtractor, RepoDriver, SessionStorage};
use pult_tui::app::App;
use pult_tui::cli::Cli;
use pult_tui::event::{handle_key_event, TuiAction};
use pult_tui::ui::render;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum PipelineEvent {
    Log(String),
    Chunk(String),
    StepBusy(String),
    Completed(i32),
    Failed(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let target = cli.target_repo();
    let repo_dir = match pult_core::repo::resolve_repo_path(target, None) {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Erro ao resolver repositório '{}': {}", target, err);
            std::process::exit(1);
        }
    };

    if let Err(err) = std::env::set_current_dir(&repo_dir) {
        eprintln!(
            "Erro ao definir diretório de trabalho '{}': {}",
            repo_dir.display(),
            err
        );
        std::process::exit(1);
    }

    run_tui(cli, repo_dir).await?;
    Ok(())
}

async fn run_tui(cli: Cli, repo_dir: PathBuf) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::with_repo(repo_dir.clone());

    // 1. Detecção dinâmica de branch e inferência de ticket via Git
    let git_driver = pult_core::CccpGitDriver::new(&repo_dir);
    if let Ok(branch) = git_driver.current_branch() {
        let inferred_ticket = pult_tui::app::infer_ticket_from_branch(&branch);
        app.branch_name = branch.clone();
        app.ticket_id = inferred_ticket;
    }

    // 2. Precedência de flags explícitas da CLI
    if let Some(id) = cli.ticket {
        app.ticket_id = Some(id);
    } else if let Some(ref desc) = cli.draft {
        app.append_viewport_line(&format!("--- Rascunho: {} ---", desc));
    }

    // 3. Suporte a restauração de sessão prévia (--resume / -s)
    if let Some(ref resume_str) = cli.resume {
        let resume_path = std::path::Path::new(resume_str);
        let candidate = if resume_path.is_file() {
            resume_path.to_path_buf()
        } else {
            repo_dir.join(".antigravity/pult/sessions").join(resume_str)
        };
        if candidate.is_file() {
            if let Err(e) = app.load_session_file(&candidate) {
                app.append_viewport_line(&format!(
                    "❌ Falha ao carregar sessão '{}': {}",
                    candidate.display(),
                    e
                ));
            } else {
                app.append_viewport_line(&format!(
                    "✓ Sessão restaurada a partir de '{}'.",
                    candidate.display()
                ));
            }
        } else {
            app.append_viewport_line(&format!(
                "⚠ Arquivo de sessão '{}' não encontrado.",
                resume_str
            ));
        }
    }

    let res = run_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Erro na execução da TUI: {:?}", err);
    }

    Ok(())
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let (pipeline_tx, mut pipeline_rx) = mpsc::channel::<PipelineEvent>(200);

    while !app.should_quit {
        terminal.draw(|f| render(app, f))?;

        if app.is_busy {
            app.tick_spinner();
        }

        // Drena eventos do pipeline assíncrono para o viewport
        let mut received_any = false;
        while let Ok(event) = pipeline_rx.try_recv() {
            received_any = true;
            match event {
                PipelineEvent::Log(msg) => {
                    app.append_viewport_line(&msg);
                }
                PipelineEvent::Chunk(chunk) => {
                    app.append_stream_chunk(&chunk);
                }
                PipelineEvent::StepBusy(label) => {
                    app.busy_label = label;
                }
                PipelineEvent::Completed(code) => {
                    app.append_viewport_line(&format!(
                        "  ✓ [4/4] Concluído com código de saída {}.",
                        code
                    ));
                    app.set_busy(false, "");
                }
                PipelineEvent::Failed(err) => {
                    app.append_viewport_line(&format!("  ❌ [4/4] Erro na execução: {}", err));
                    app.set_busy(false, "");
                }
            }
        }

        if received_any {
            let max_visible = 20;
            app.scroll_to_bottom(max_visible);
        }

        if event::poll(Duration::from_millis(30))? {
            if let Event::Key(key) = event::read()? {
                if let Some(action) = handle_key_event(app, key) {
                    let git_driver = pult_core::CccpGitDriver::new(&app.repo_path);
                    match action {
                        TuiAction::Submit(text) => {
                            if !app.is_busy {
                                app.append_viewport_line(&format!("> {}", text));
                                app.set_busy(true, "Iniciando...");

                                let tx = pipeline_tx.clone();
                                let repo = app.repo_path.clone();
                                let tid = app.ticket_id;
                                let branch = app.branch_name.clone();
                                let session_arc = app.session.clone();

                                tokio::spawn(async move {
                                    run_orchestration_pipeline(
                                        text,
                                        repo,
                                        tid,
                                        branch,
                                        session_arc,
                                        tx,
                                    )
                                    .await;
                                });
                            }
                        }
                        TuiAction::SpecApproved
                        | TuiAction::PhaseAdvanced(_)
                        | TuiAction::FinishTriggered
                        | TuiAction::MergeTriggered => {
                            let _ = app.execute_action(action, &git_driver);
                        }
                        TuiAction::Quit => {
                            app.quit();
                        }
                        TuiAction::FocusToggled | TuiAction::Scrolled => {}
                    }
                }
            }
        }
    }
    Ok(())
}

async fn run_orchestration_pipeline(
    prompt: String,
    repo_dir: PathBuf,
    ticket_id: Option<u32>,
    branch_name: String,
    session_arc: std::sync::Arc<std::sync::Mutex<Option<pult_core::Session>>>,
    tx: mpsc::Sender<PipelineEvent>,
) {
    // 1. Extração Estrutural de AST Poliglota
    let _ = tx
        .send(PipelineEvent::StepBusy("Extraindo AST...".into()))
        .await;
    let _ = tx
        .send(PipelineEvent::Log(
            "☭ [PULT] [1/4] Extraindo assinaturas de AST da base de código...".into(),
        ))
        .await;

    let extractor = pult_core::PolyglotAstExtractor::new();
    let ast_files =
        match extractor.extract_from_dir(&repo_dir, &[".git", "target", "node_modules", "dist"]) {
            Ok(files) => {
                let _ = tx
                    .send(PipelineEvent::Log(format!(
                        "  ✓ [1/4] {} arquivos de AST analisados com sucesso.",
                        files.len()
                    )))
                    .await;
                files
            }
            Err(e) => {
                let _ = tx
                    .send(PipelineEvent::Log(format!(
                        "  ℹ [1/4] Aviso na extração de AST: {} (prosseguindo sem AST prévia)",
                        e
                    )))
                    .await;
                Vec::new()
            }
        };

    // 2. Filtragem Cirúrgica Scout (Ollama Local)
    let _ = tx
        .send(PipelineEvent::StepBusy("Scout (Ollama)...".into()))
        .await;
    let _ = tx
        .send(PipelineEvent::Log(
            "☭ [PULT] [2/4] Acionando Scout local (Ollama) para seleção de símbolos...".into(),
        ))
        .await;

    let scout = pult_core::ScoutEngine::new();
    let scout_result = scout.filter_with_bypass(&prompt, &ast_files).await;

    let mut injected_context = String::new();
    match scout_result {
        Ok(res) => {
            if res.bypassed {
                let _ = tx.send(PipelineEvent::Log(
                    "  ℹ [2/4] Scout em modo bypass (Ollama inacessível ou sem modelo local carregado).".into(),
                )).await;
            } else {
                let count = res.symbols.len();
                let _ = tx
                    .send(PipelineEvent::Log(format!(
                        "  ✓ [2/4] Scout identificou {} símbolos relevantes (modelo: {}).",
                        count, res.model_used
                    )))
                    .await;
                if !res.yaml_block.is_empty() {
                    injected_context = format!(
                        "\nContexto estrutural de AST selecionado pelo Scout:\n{}\n",
                        res.yaml_block
                    );
                }
            }
        }
        Err(e) => {
            let _ = tx
                .send(PipelineEvent::Log(format!(
                    "  ℹ [2/4] Scout bypass devido a: {}",
                    e
                )))
                .await;
        }
    }

    // 3. Persistência de Turno e Sessão
    let _ = tx
        .send(PipelineEvent::StepBusy("Gravando sessão...".into()))
        .await;
    let _ = tx
        .send(PipelineEvent::Log(
            "☭ [PULT] [3/4] Gravando turno na sessão em disco...".into(),
        ))
        .await;

    let storage_dir = repo_dir.join(".antigravity/pult/sessions");
    let storage = pult_core::FileSessionStorage::new(&storage_dir);

    let (turn_number, session_id, save_err) = {
        let mut guard = session_arc.lock().unwrap();
        if guard.is_none() {
            let new_session = match ticket_id {
                Some(id) => pult_core::Session::new_formal(id, &branch_name, &branch_name),
                None => pult_core::Session::new_draft(&prompt),
            };
            *guard = Some(new_session);
        }
        let sess = guard.as_mut().unwrap();
        let user_turn = sess.add_turn(
            pult_core::Actor::User,
            prompt.clone(),
            pult_core::TurnStatus::Completed,
        );
        let turn_num = user_turn.turn_number;
        let sid = sess.metadata.session_id.clone();
        let err = storage.save(sess).err().map(|e| e.to_string());
        (turn_num, sid, err)
    };

    if let Some(e) = save_err {
        let _ = tx
            .send(PipelineEvent::Log(format!(
                "  ⚠ Aviso ao salvar sessão: {}",
                e
            )))
            .await;
    }

    let _ = tx
        .send(PipelineEvent::Log(format!(
            "  ✓ [3/4] Turno #{} salvo na sessão {}",
            turn_number, session_id
        )))
        .await;

    // 4. Execução do Agente Externo (agy)
    let _ = tx
        .send(PipelineEvent::StepBusy("Agente (agy)...".into()))
        .await;
    let _ = tx
        .send(PipelineEvent::Log(
            "☭ [PULT] [4/4] Disparando Agente Externo (agy)...".into(),
        ))
        .await;

    // Inicializa turno do Agente em InProgress e persiste imediatamente
    {
        let mut guard = session_arc.lock().unwrap();
        if let Some(sess) = guard.as_mut() {
            sess.add_turn(
                pult_core::Actor::Agy,
                String::new(),
                pult_core::TurnStatus::InProgress,
            );
            let _ = storage.save(sess);
        }
    }

    let full_prompt = format!("{}{}", prompt, injected_context);

    let runner = pult_core::AgyRunner::with_options(pult_core::RunnerOptions {
        command: "agy".to_string(),
        args: vec![
            "--print".to_string(),
            full_prompt,
            "--dangerously-skip-permissions".to_string(),
        ],
        working_dir: Some(repo_dir),
    });

    match runner.spawn() {
        Ok((mut stream_rx, child_handle)) => {
            while let Some(event) = stream_rx.recv().await {
                match event {
                    pult_core::AgyStreamEvent::StdoutChunk(chunk) => {
                        {
                            let mut guard = session_arc.lock().unwrap();
                            if let Some(sess) = guard.as_mut() {
                                sess.append_to_last_turn(&chunk);
                                let _ = storage.save(sess);
                            }
                        }
                        let _ = tx.send(PipelineEvent::Chunk(chunk)).await;
                    }
                    pult_core::AgyStreamEvent::StderrChunk(chunk) => {
                        {
                            let mut guard = session_arc.lock().unwrap();
                            if let Some(sess) = guard.as_mut() {
                                sess.append_to_last_turn(&chunk);
                                let _ = storage.save(sess);
                            }
                        }
                        let _ = tx.send(PipelineEvent::Chunk(chunk)).await;
                    }
                    pult_core::AgyStreamEvent::Truncated => {
                        {
                            let mut guard = session_arc.lock().unwrap();
                            if let Some(sess) = guard.as_mut() {
                                sess.update_last_turn_status(pult_core::TurnStatus::Truncated);
                                let _ = storage.save(sess);
                            }
                        }
                        let _ = tx
                            .send(PipelineEvent::Log(
                                "  ⚠ Aviso: Saída truncada pelo agente.".into(),
                            ))
                            .await;
                    }
                    pult_core::AgyStreamEvent::Completed { exit_code } => {
                        {
                            let mut guard = session_arc.lock().unwrap();
                            if let Some(sess) = guard.as_mut() {
                                sess.update_last_turn_status(pult_core::TurnStatus::Completed);
                                let _ = storage.save(sess);
                            }
                        }
                        let _ = tx.send(PipelineEvent::Completed(exit_code)).await;
                        break;
                    }
                    pult_core::AgyStreamEvent::Error(err) => {
                        {
                            let mut guard = session_arc.lock().unwrap();
                            if let Some(sess) = guard.as_mut() {
                                sess.update_last_turn_status(pult_core::TurnStatus::Interrupted);
                                let _ = storage.save(sess);
                            }
                        }
                        let _ = tx.send(PipelineEvent::Failed(err)).await;
                        break;
                    }
                }
            }
            let _ = child_handle.await;
        }
        Err(e) => {
            {
                let mut guard = session_arc.lock().unwrap();
                if let Some(sess) = guard.as_mut() {
                    sess.update_last_turn_status(pult_core::TurnStatus::Interrupted);
                    let _ = storage.save(sess);
                }
            }
            let _ = tx
                .send(PipelineEvent::Failed(format!(
                    "Falha ao iniciar processo agy: {}",
                    e
                )))
                .await;
        }
    }
}
