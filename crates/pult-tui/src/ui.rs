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
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let ticket_str = match app.ticket_id {
        Some(id) => format!("#{}", id),
        None => "Rascunho".to_string(),
    };

    let title_span = Span::styled(
        format!(" ☭ PULT v{} ", pult_core::VERSION),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let phase_span = Span::styled(
        format!(" FASE {} ", app.gost_phase.number()),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let branch_span = Span::styled(
        format!(" [{}] {} ", ticket_str, app.branch_name),
        Style::default().fg(Color::Green),
    );

    let header_line = Line::from(vec![
        title_span,
        Span::raw(" | "),
        phase_span,
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

    let mut textarea = app.textarea.clone();
    textarea.set_block(
        Block::default()
            .title(" [Editor Multilinha] ")
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(&textarea, area);
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let mut spans = vec![
        Span::styled("[Ctrl+S: Enviar]  ", Style::default().fg(Color::Green)),
        Span::styled(
            "[Ctrl+T: Alternar Foco]  ",
            Style::default().fg(Color::Yellow),
        ),
    ];

    match app.gost_phase {
        pult_core::domain::GostPhase::SpecContract => {
            spans.push(Span::styled(
                "[Ctrl+P: Aprovar Spec]  ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
        }
        pult_core::domain::GostPhase::CloseTicket => {
            spans.push(Span::styled(
                "[Ctrl+G: Fechar]  ",
                Style::default()
                    .fg(Color::LightMagenta)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                "[Ctrl+M: Merge CCCP]  ",
                Style::default().fg(Color::LightGreen),
            ));
        }
        _ => {
            spans.push(Span::styled(
                "[Ctrl+N: Próx. Fase]  ",
                Style::default().fg(Color::Cyan),
            ));
        }
    }

    spans.push(Span::styled("[Ctrl+Q: Sair]", Style::default().fg(Color::Red)));

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}
