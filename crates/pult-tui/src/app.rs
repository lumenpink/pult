use pult_core::domain::GostPhase;
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
    pub gost_phase: GostPhase,
    pub repo_path: std::path::PathBuf,
    pub repo_name: String,
    pub is_busy: bool,
    pub spinner_idx: usize,
    pub busy_label: String,
    pub is_line_open: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        Self::with_repo(cwd)
    }

    pub fn with_repo(repo_path: std::path::PathBuf) -> Self {
        let repo_name = repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("pult")
            .to_string();
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
            gost_phase: GostPhase::Ticket,
            repo_path,
            repo_name,
            is_busy: false,
            spinner_idx: 0,
            busy_label: String::new(),
            is_line_open: false,
        }
    }

    pub fn phase(&self) -> GostPhase {
        self.gost_phase
    }

    pub fn set_gost_phase(&mut self, phase: GostPhase) {
        self.gost_phase = phase;
    }

    pub fn set_ticket_info(
        &mut self,
        ticket_id: Option<u32>,
        branch_name: &str,
        gost_phase_num: u8,
    ) {
        self.ticket_id = ticket_id;
        self.branch_name = branch_name.to_string();
        if let Ok(p) = GostPhase::from_u8(gost_phase_num) {
            self.gost_phase = p;
        }
    }

    pub fn advance_gost_phase(&mut self) -> Result<GostPhase, &'static str> {
        let current_num = self.gost_phase.number();
        if current_num >= 9 {
            return Err("Já se encontra na última fase do GOST (Fase 9: Close Ticket)");
        }
        let next = GostPhase::from_u8(current_num + 1).map_err(|_| "Fase GOST inválida")?;
        self.gost_phase = next;
        Ok(next)
    }

    pub fn approve_spec_contract(&mut self) -> Result<GostPhase, &'static str> {
        if self.gost_phase != GostPhase::SpecContract {
            return Err("Apenas a Fase 3 (Doc / Spec Contract) pode ser aprovada para Red Test");
        }
        self.gost_phase = GostPhase::RedTest;
        Ok(GostPhase::RedTest)
    }

    pub fn check_phase3_guard(&self, modified_files: &[&str]) -> Result<(), String> {
        if self.gost_phase != GostPhase::SpecContract {
            return Ok(());
        }

        for &file in modified_files {
            let is_doc =
                file.starts_with("docs/") || file.ends_with(".md") || file.starts_with(".gost/");
            if !is_doc && (file.starts_with("src/") || file.contains("/src/")) {
                return Err(format!(
                    "☭ Veto Fase 3: Modificação em código de produção proibida na fase de contratos! Arquivo violador: {}",
                    file
                ));
            }
        }
        Ok(())
    }

    pub fn compile_closing_chronicle(&self, scope: &str, summary: &str) -> String {
        let ticket_str = match self.ticket_id {
            Some(id) => format!(" (closes #{})", id),
            None => String::new(),
        };

        let header = format!("docs({}): {}{}", scope, summary, ticket_str);

        let mut body = String::new();
        body.push_str("\n### Crônica da Entrega (GOST DD-TDD)\n");
        if let Some(id) = self.ticket_id {
            body.push_str(&format!("- Ticket: #{}\n", id));
        }
        body.push_str(&format!("- Branch: {}\n", self.branch_name));
        body.push_str(&format!(
            "- Turnos Processados: {}\n",
            self.viewport_lines.len()
        ));
        body.push_str("- Fases Concluídas: 1 a 9\n");

        format!("{}\n{}", header, body)
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
        self.is_line_open = false;
    }

    pub fn append_stream_chunk(&mut self, chunk: &str) {
        if chunk.is_empty() {
            return;
        }

        let ends_with_newline = chunk.ends_with('\n');
        let parts: Vec<&str> = chunk.split('\n').collect();

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 && ends_with_newline && part.is_empty() {
                continue;
            }

            if i == 0 && self.is_line_open {
                if let Some(last) = self.viewport_lines.last_mut() {
                    last.push_str(part);
                } else {
                    self.viewport_lines.push(part.to_string());
                }
            } else {
                self.viewport_lines.push(part.to_string());
            }
        }

        self.is_line_open = !ends_with_newline;
    }

    pub fn append_viewport_text(&mut self, text: &str) {
        for line in text.lines() {
            self.append_viewport_line(line);
        }
    }

    pub fn spinner_glyph(&self) -> &'static str {
        const GLYPHS: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        GLYPHS[self.spinner_idx % GLYPHS.len()]
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_idx = (self.spinner_idx + 1) % 10;
    }

    pub fn set_busy(&mut self, busy: bool, label: &str) {
        self.is_busy = busy;
        self.busy_label = label.to_string();
    }

    pub fn submit_input(&mut self) -> Option<String> {
        if self.is_busy {
            return None;
        }
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
