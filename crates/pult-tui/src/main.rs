//! PULT (Пульт) — Estação de trabalho TUI autônoma para terminal.

use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pult_core::{AstExtractor, SessionStorage};
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

    let mut app = App::with_repo(repo_dir);
    if let Some(id) = cli.ticket {
        app.set_ticket_info(Some(id), "feat/active", 1);
    } else if let Some(ref desc) = cli.draft {
        app.append_viewport_line(&format!("--- Rascunho: {} ---", desc));
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
                if let Some(TuiAction::Submit(text)) = handle_key_event(app, key) {
                    if !app.is_busy {
                        app.append_viewport_line(&format!("> {}", text));
                        app.set_busy(true, "Iniciando...");

                        let tx = pipeline_tx.clone();
                        let repo = app.repo_path.clone();
                        let tid = app.ticket_id;
                        let branch = app.branch_name.clone();

                        tokio::spawn(async move {
                            run_orchestration_pipeline(text, repo, tid, branch, tx).await;
                        });
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
    let mut session = match ticket_id {
        Some(id) => pult_core::Session::new_formal(id, &branch_name, &branch_name),
        None => pult_core::Session::new_draft(&prompt),
    };

    let user_turn = pult_core::Turn {
        turn_number: 1,
        actor: pult_core::Actor::User,
        timestamp: "agora".to_string(),
        content: prompt.clone(),
        status: pult_core::TurnStatus::Completed,
        tokens: None,
        tokens_out: None,
    };
    session.turns.push(user_turn);
    if let Err(e) = storage.save(&session) {
        let _ = tx
            .send(PipelineEvent::Log(format!(
                "  ⚠ Aviso ao salvar sessão: {}",
                e
            )))
            .await;
    } else {
        let _ = tx
            .send(PipelineEvent::Log(format!(
                "  ✓ [3/4] Sessão salva em {}",
                session.metadata.session_id
            )))
            .await;
    }

    // 4. Execução do Agente Externo (agy)
    let _ = tx
        .send(PipelineEvent::StepBusy("Agente (agy)...".into()))
        .await;
    let _ = tx
        .send(PipelineEvent::Log(
            "☭ [PULT] [4/4] Disparando Agente Externo (agy)...".into(),
        ))
        .await;

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
                        let _ = tx.send(PipelineEvent::Chunk(chunk)).await;
                    }
                    pult_core::AgyStreamEvent::StderrChunk(chunk) => {
                        let _ = tx.send(PipelineEvent::Chunk(chunk)).await;
                    }
                    pult_core::AgyStreamEvent::Truncated => {
                        let _ = tx
                            .send(PipelineEvent::Log(
                                "  ⚠ Aviso: Saída truncada pelo agente.".into(),
                            ))
                            .await;
                    }
                    pult_core::AgyStreamEvent::Completed { exit_code } => {
                        let _ = tx.send(PipelineEvent::Completed(exit_code)).await;
                        break;
                    }
                    pult_core::AgyStreamEvent::Error(err) => {
                        let _ = tx.send(PipelineEvent::Failed(err)).await;
                        break;
                    }
                }
            }
            let _ = child_handle.await;
        }
        Err(e) => {
            let _ = tx
                .send(PipelineEvent::Failed(format!(
                    "Falha ao iniciar processo agy: {}",
                    e
                )))
                .await;
        }
    }
}
