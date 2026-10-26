use tui_textarea::TextArea;

/// Painel que detém o foco ativo do teclado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Editor,
    Viewport,
}

/// Estado global da interface TUI do PULT.
pub struct App {
    pub focus: Focus,
    pub scroll_offset: usize,
    pub viewport_lines: Vec<String>,
    pub textarea: TextArea<'static>,
    pub should_quit: bool,
    pub ticket_id: Option<u32>,
    pub branch_name: String,
    pub gost_phase: u8,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(ratatui::style::Style::default());
        Self {
            focus: Focus::Editor,
            scroll_offset: 0,
            viewport_lines: Vec::new(),
            textarea,
            should_quit: false,
            ticket_id: None,
            branch_name: "main".to_string(),
            gost_phase: 1,
        }
    }

    pub fn set_ticket_info(&mut self, ticket_id: Option<u32>, branch_name: &str, gost_phase: u8) {
        self.ticket_id = ticket_id;
        self.branch_name = branch_name.to_string();
        self.gost_phase = gost_phase;
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Editor => Focus::Viewport,
            Focus::Viewport => Focus::Editor,
        };
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: usize, max_visible: usize) {
        let max_scroll = self.viewport_lines.len().saturating_sub(max_visible);
        self.scroll_offset = (self.scroll_offset + n).min(max_scroll);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self, max_visible: usize) {
        self.scroll_offset = self.viewport_lines.len().saturating_sub(max_visible);
    }

    pub fn append_viewport_line(&mut self, line: &str) {
        self.viewport_lines.push(line.to_string());
    }

    pub fn append_viewport_text(&mut self, text: &str) {
        for line in text.lines() {
            self.append_viewport_line(line);
        }
    }

    pub fn submit_input(&mut self) -> Option<String> {
        let lines = self.textarea.lines();
        let content = lines.join("\n").trim().to_string();
        if content.is_empty() {
            return None;
        }
        self.textarea = TextArea::default();
        Some(content)
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
