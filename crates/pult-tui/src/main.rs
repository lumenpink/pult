//! PULT (Пульт) — Estação de trabalho TUI soberana para terminal.

use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pult_tui::app::App;
use pult_tui::event::{handle_key_event, TuiAction};
use pult_tui::ui::render;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "pult",
    version = pult_core::VERSION,
    about = "Estação TUI soberana para terminal"
)]
struct Cli {
    /// ID do ticket formal a ser aberto
    #[arg(short, long)]
    ticket: Option<u32>,

    /// Abrir sessão rascunho com descrição
    #[arg(short, long)]
    draft: Option<String>,

    /// Retomar sessão existente a partir de arquivo markdown
    #[arg(short, long)]
    resume: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    run_tui(cli)?;
    Ok(())
}

fn run_tui(cli: Cli) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
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
