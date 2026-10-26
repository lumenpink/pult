//! PULT (Пульт) — Estação de trabalho TUI soberana para terminal.

use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pult_tui::app::App;
use pult_tui::cli::Cli;
use pult_tui::event::{handle_key_event, TuiAction};
use pult_tui::ui::render;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    run_tui(cli, repo_dir)?;
    Ok(())
}

fn run_tui(cli: Cli, repo_dir: PathBuf) -> io::Result<()> {
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

    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Erro na execução da TUI: {:?}", err);
    }

    Ok(())
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| render(app, f))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let Some(TuiAction::Submit(text)) = handle_key_event(app, key) {
                    app.append_viewport_line(&format!("> {}", text));
                }
            }
        }
    }
    Ok(())
}
