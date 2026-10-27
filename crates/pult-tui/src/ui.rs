use crate::app::{App, Focus};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(app: &App, frame: &mut Frame) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Viewport de histórico
            Constraint::Length(7), // Editor multilinha (5 linhas úteis + bordas)
            Constraint::Length(1), // Footer de atalhos
        ])
        .split(size);

    render_header(app, frame, chunks[0]);
    render_viewport(app, frame, chunks[1]);
    render_editor(app, frame, chunks[2]);
    render_footer(app, frame, chunks[3]);

    if app.show_help {
        render_help_modal(app, frame, size);
    }
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let title_span = Span::styled(
        format!(" ☭ PULT v{} ", pult_core::short_version()),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let phase_span = Span::styled(
        format!(
            " FASE {}: {} ",
            app.gost_phase.number(),
            app.gost_phase.name()
        ),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let repo_span = Span::styled(
        format!(" [{}] ", app.repo_name),
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    );
    let branch_span = Span::styled(
        app.header_branch_label(),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let header_line = Line::from(vec![
        title_span,
        Span::raw(" | "),
        phase_span,
        Span::raw(" | "),
        repo_span,
        Span::raw(" | "),
        branch_span,
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(header_line).block(block);
    frame.render_widget(paragraph, area);
}

fn render_viewport(app: &App, frame: &mut Frame, area: Rect) {
    let border_style = if app.focus == Focus::Viewport {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" [Histórico & Saídas] ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_height = area.height.saturating_sub(2) as usize;
    let lines: Vec<Line> = app
        .viewport_lines
        .iter()
        .skip(app.scroll_offset)
        .take(inner_height.max(1))
        .map(|l| Line::from(l.as_str()))
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn render_editor(app: &App, frame: &mut Frame, area: Rect) {
    let border_style = if app.focus == Focus::Editor {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if app.is_busy {
        let elapsed = app.busy_elapsed_formatted();
        let label = if app.busy_label.is_empty() {
            "Processando...".to_string()
        } else {
            app.busy_label.clone()
        };
        if elapsed.is_empty() {
            format!(" [Editor ({} {})] ", app.spinner_glyph(), label)
        } else {
            format!(" [Editor ({} {} {})] ", app.spinner_glyph(), label, elapsed)
        }
    } else {
        " [Editor Multilinha] ".to_string()
    };

    let mut textarea = app.textarea.clone();
    textarea.set_block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(&textarea, area);
}

fn render_footer(_app: &App, frame: &mut Frame, area: Rect) {
    let span = Span::styled(
        " [Ctrl+H: Ajuda] ",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let paragraph = Paragraph::new(Line::from(vec![span]));
    frame.render_widget(paragraph, area);
}

fn render_help_modal(_app: &App, frame: &mut Frame, area: Rect) {
    let modal_width = 72.min(area.width.saturating_sub(4));
    let modal_height = 16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let popup_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(ratatui::widgets::Clear, popup_area);

    let help_lines = vec![
        Line::from(vec![
            Span::styled(
                "Comando   ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Atalho   ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Descrição",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::raw(
            "────────────────────────────────────────────────────────────",
        )]),
        Line::from(vec![
            Span::styled("/branch   ", Style::default().fg(Color::Green)),
            Span::styled("         ", Style::default().fg(Color::Cyan)),
            Span::raw("Criar ou alternar branch (/branch <nome>)"),
        ]),
        Line::from(vec![
            Span::styled("/new      ", Style::default().fg(Color::Green)),
            Span::styled("         ", Style::default().fg(Color::Cyan)),
            Span::raw("Criar / reiniciar sessão de trabalho"),
        ]),
        Line::from(vec![
            Span::styled("/approve  ", Style::default().fg(Color::Green)),
            Span::styled("Ctrl+P   ", Style::default().fg(Color::Cyan)),
            Span::raw("Aprovar contrato/spec (Fase 3 -> Fase 4)"),
        ]),
        Line::from(vec![
            Span::styled("/next     ", Style::default().fg(Color::Green)),
            Span::styled("Ctrl+N   ", Style::default().fg(Color::Cyan)),
            Span::raw("Avançar para a próxima fase GOST"),
        ]),
        Line::from(vec![
            Span::styled("/finish   ", Style::default().fg(Color::Green)),
            Span::styled("Ctrl+G   ", Style::default().fg(Color::Cyan)),
            Span::raw("Compilar crônica e encerrar ticket"),
        ]),
        Line::from(vec![
            Span::styled("/merge    ", Style::default().fg(Color::Green)),
            Span::styled("Ctrl+M   ", Style::default().fg(Color::Cyan)),
            Span::raw("Executar cccp merge da branch de trabalho"),
        ]),
        Line::from(vec![
            Span::styled("/help     ", Style::default().fg(Color::Green)),
            Span::styled("Ctrl+H   ", Style::default().fg(Color::Cyan)),
            Span::raw("Abrir ou fechar esta janela de ajuda"),
        ]),
        Line::from(vec![
            Span::styled("/quit     ", Style::default().fg(Color::Green)),
            Span::styled("Ctrl+Q   ", Style::default().fg(Color::Cyan)),
            Span::raw("Encerrar o PULT"),
        ]),
        Line::from(vec![
            Span::styled("          ", Style::default().fg(Color::Green)),
            Span::styled("Ctrl+T   ", Style::default().fg(Color::Cyan)),
            Span::raw("Alternar foco (Editor <-> Viewport)"),
        ]),
        Line::from(vec![
            Span::styled("          ", Style::default().fg(Color::Green)),
            Span::styled("Ctrl+S   ", Style::default().fg(Color::Cyan)),
            Span::raw("Enviar prompt do editor"),
        ]),
        Line::from(vec![
            Span::styled("          ", Style::default().fg(Color::Green)),
            Span::styled("Tab      ", Style::default().fg(Color::Cyan)),
            Span::raw("Autocompletar comando com barra (/)"),
        ]),
        Line::from(vec![
            Span::styled("          ", Style::default().fg(Color::Green)),
            Span::styled("Esc      ", Style::default().fg(Color::Cyan)),
            Span::raw("Fechar modal de ajuda"),
        ]),
    ];

    let block = Block::default()
        .title(" ☭ Central de Comandos e Atalhos (PULT) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(help_lines).block(block);
    frame.render_widget(paragraph, popup_area);
}
