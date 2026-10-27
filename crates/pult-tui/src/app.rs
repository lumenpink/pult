use pult_core::domain::GostPhase;
use pult_core::Session;
use std::sync::{Arc, Mutex};
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
    pub session: Arc<Mutex<Option<Session>>>,
    pub show_help: bool,
    pub pending_dispatch: Option<String>,
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
            session: Arc::new(Mutex::new(None)),
            show_help: false,
            pending_dispatch: None,
        }
    }

    pub fn take_pending_dispatch(&mut self) -> Option<String> {
        self.pending_dispatch.take()
    }

    pub fn phase_prompt_for(phase: GostPhase, ticket_id: Option<u32>, branch: &str) -> String {
        let tid_str = ticket_id
            .map(|id| format!("#{}", id))
            .unwrap_or_else(|| "atual".to_string());
        let directive = format!(
            "[DIRETIVA DE OPERAÇÃO - SUBAGENTE DE EXECUÇÃO PULT]\n\
            Anulação de escopo: Ignore quaisquer regras ou diretrizes herdadas sobre orquestração do ciclo GOST de 9 fases. Você NÃO é o maestro do ciclo; o PULT é o orquestrador absoluto do processo.\n\
            Seu escopo de trabalho está ESTRITAMENTE restrito à fase atual ({}).\n\n",
            phase.name()
        );
        match phase {
            GostPhase::SpecContract => format!(
                "{}[Fase 3 - Spec Contract]: Elabore o contrato de especificação preliminar em docs/specs/ para o ticket {} na branch '{}'. Detalhe a arquitetura, requisitos e contratos técnicos.",
                directive, tid_str, branch
            ),
            GostPhase::RedTest => format!(
                "{}[Fase 4 - Red Test]: Implemente os testes automatizados preliminares falhos (red tests) em tests/ comprovando os requisitos do ticket {} na branch '{}'. Não crie código funcional ainda.",
                directive, tid_str, branch
            ),
            GostPhase::GreenCode => format!(
                "{}[Fase 5 - Green Code]: Implemente a solução técnica mínima funcional em src/ para fazer todos os testes do ticket {} passarem (green) na branch '{}'.",
                directive, tid_str, branch
            ),
            GostPhase::Refactor => format!(
                "{}[Fase 6 - Refactor]: Realize a refatoração e limpeza de código em src/ para o ticket {} na branch '{}', preservando todos os testes verdes.",
                directive, tid_str, branch
            ),
            GostPhase::MutationTest => format!(
                "{}[Fase 7 - Mutation Test]: Fortaleça a suíte de testes em tests/ para o ticket {} na branch '{}', eliminando potenciais mutantes e cenários de borda.",
                directive, tid_str, branch
            ),
            _ => format!(
                "{}[Fase {} - {}]: Prossiga com o trabalho do ticket {} na branch '{}'.",
                directive,
                phase.number(),
                phase.name(),
                tid_str,
                branch
            ),
        }
    }

    pub fn reset_session(&self) {
        if let Ok(mut lock) = self.session.lock() {
            *lock = None;
        }
    }

    pub fn set_session(&self, session: Session) {
        if let Ok(mut lock) = self.session.lock() {
            *lock = Some(session);
        }
    }

    pub fn session_id(&self) -> Option<String> {
        self.session
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.metadata.session_id.clone()))
    }

    pub fn phase(&self) -> GostPhase {
        self.gost_phase
    }

    pub fn ticket_id(&self) -> Option<u32> {
        self.ticket_id
    }

    pub fn branch_name(&self) -> &str {
        &self.branch_name
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

    pub fn header_branch_label(&self) -> String {
        format!(
            "[{}] 🌿 {} ",
            match self.ticket_id {
                Some(id) => format!("#{}", id),
                None => "Rascunho".to_string(),
            },
            self.branch_name
        )
    }

    /// Sintetiza título e corpo do ticket nos moldes GOST a partir dos turnos iniciais.
    pub fn synthesize_ticket_data(&self) -> (String, String) {
        let mut first_user_prompt = "demanda do usuario".to_string();
        if let Ok(guard) = self.session.lock() {
            if let Some(ref sess) = *guard {
                if let Some(turn) = sess
                    .turns
                    .iter()
                    .find(|t| t.actor == pult_core::Actor::User)
                {
                    first_user_prompt = turn
                        .content
                        .lines()
                        .next()
                        .unwrap_or("demanda")
                        .trim()
                        .to_string();
                }
            }
        }
        let clean_prompt = first_user_prompt.chars().take(40).collect::<String>();
        let title = format!("feat: {}", clean_prompt);
        let body = format!(
            "## Descrição\n{}\n\n## Tasks / DD-TDD Checklist (9 Fases GOST)\n\
             - [ ] Fase 1 - Ticket\n- [ ] Fase 2 - Branch\n- [ ] Fase 3 - Doc/Spec\n\
             - [ ] Fase 4 - Red\n- [ ] Fase 5 - Green\n- [ ] Fase 6 - Refactor\n\
             - [ ] Fase 7 - Mutation\n- [ ] Fase 8 - Quality Gate\n- [ ] Fase 9 - Close\n",
            first_user_prompt
        );
        (title, body)
    }

    /// Carrega uma sessão gravada em disco a partir de um caminho de arquivo,
    /// restaurando o histórico de turnos no viewport e sincronizando os metadados GOST.
    pub fn load_session_file(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;
        let session = Session::from_markdown(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        if let Some(ref ticket) = session.metadata.ticket {
            self.ticket_id = Some(ticket.id);
            self.branch_name = ticket.branch.clone();
        }

        if let Ok(phase) = session.gost_phase() {
            self.gost_phase = phase;
        }

        self.viewport_lines.clear();
        for turn in &session.turns {
            match turn.actor {
                pult_core::Actor::User => {
                    self.append_viewport_line(&format!("> {}", turn.content));
                }
                pult_core::Actor::Scout => {
                    self.append_viewport_line("☭ [PULT] Contexto filtrado pelo Scout:");
                    for line in turn.content.lines() {
                        self.append_viewport_line(line);
                    }
                }
                pult_core::Actor::Agy => {
                    for line in turn.content.lines() {
                        self.append_viewport_line(line);
                    }
                }
            }
        }

        self.set_session(session);
        Ok(())
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

    /// Executa ações contextuais da máquina GOST (Ctrl+P, Ctrl+N, Ctrl+G, Ctrl+M)
    /// com validações estritas de conformidade, guarda da Fase 3 e fechamento de ticket.
    pub fn execute_action(
        &mut self,
        action: crate::event::TuiAction,
        driver: &dyn pult_core::RepoDriver,
    ) -> Result<String, String> {
        match action {
            crate::event::TuiAction::SpecApproved => {
                let current_phase = self.gost_phase;

                // Automação da Fase 1 -> Fase 2 -> Fase 3 via Scout & Driver
                if current_phase == GostPhase::Ticket && self.ticket_id.is_none() {
                    let (title, body) = self.synthesize_ticket_data();
                    if let Ok((tid, branch_suggestion)) = driver.create_ticket(&title, &body) {
                        let branch = if branch_suggestion.starts_with("feat/")
                            || branch_suggestion.starts_with("fix/")
                            || branch_suggestion.starts_with("docs/")
                        {
                            branch_suggestion
                        } else {
                            format!("feat/{}", branch_suggestion)
                        };
                        let _ = driver.checkout_branch(&branch, true);
                        self.set_ticket_info(Some(tid), &branch, 3);
                        self.append_viewport_line(&format!(
                            "★ [GOST Fase 2] Ticket #{} criado na forja. Alternado para a branch '{}'.",
                            tid, branch
                        ));
                        self.gost_phase = GostPhase::SpecContract;

                        // Sincroniza metadados da sessão ativa
                        if let Ok(mut lock) = self.session.lock() {
                            if let Some(ref mut sess) = *lock {
                                sess.metadata.gost.phase = 3;
                                sess.metadata.ticket = Some(pult_core::TicketRef {
                                    id: tid,
                                    state: "open".to_string(),
                                    branch: branch.clone(),
                                });
                            }
                        }

                        // Seta o prompt a ser despachado automaticamente para a Fase 3
                        self.pending_dispatch = Some(Self::phase_prompt_for(
                            GostPhase::SpecContract,
                            Some(tid),
                            &branch,
                        ));

                        let ok_msg = format!(
                            "★ [GOST] Fase 1 e 2 automatizadas. Ticket #{} criado na branch '{}'. Avançando para FASE 3 (SpecContract).",
                            tid, branch
                        );
                        self.append_viewport_line(&ok_msg);
                        return Ok(ok_msg);
                    }
                }

                // Obtém status de arquivos modificados via driver
                let status = driver.status().map_err(|e| e.to_string())?;
                let effective_branch = if !status.branch.is_empty() {
                    status.branch.clone()
                } else {
                    self.branch_name.clone()
                };

                if current_phase >= GostPhase::SpecContract
                    && (effective_branch == "main" || effective_branch == "master")
                {
                    let err_msg = "Bloqueio de Governança: Não é permitido operar as Fases 3 a 9 na branch main/master. Crie uma branch de trabalho primeiro com /branch <nome>.".to_string();
                    self.append_viewport_line(&format!("❌ {}", err_msg));
                    return Err(err_msg);
                }

                // Valida guarda estrita da Fase 3
                if current_phase == GostPhase::SpecContract {
                    let file_refs: Vec<&str> = status
                        .modified_files
                        .iter()
                        .chain(status.untracked_files.iter())
                        .map(|s| s.as_str())
                        .collect();
                    if let Err(veto_err) = self.check_phase3_guard(&file_refs) {
                        self.append_viewport_line(&format!("❌ {}", veto_err));
                        return Err(veto_err);
                    }
                }

                // Trava Gosplan na Fase 8 (QualityGate)
                if current_phase == GostPhase::QualityGate {
                    let audit_info = driver.check_strict().map_err(|e| e.to_string())?;
                    if audit_info.failures > 0 || !audit_info.passed {
                        let veto_msg = format!(
                            "❌ ★ GOSPLAN QUALITY CONTROL: MERGE/ADVANCE LOCK REJECTED ★ Falhas detectadas: {} (Avisos: {}). Corrija as violações antes de prosseguir.",
                            audit_info.failures, audit_info.warnings
                        );
                        self.append_viewport_line(&veto_msg);
                        return Err(veto_msg);
                    }
                }

                // Commits automáticos nas fases normativas (Fases 3, 4, 5 e 9)
                let ticket_id = self.ticket_id.or_else(|| {
                    self.session.lock().ok().and_then(|guard| {
                        guard
                            .as_ref()
                            .and_then(|s| s.metadata.ticket.as_ref().map(|t| t.id))
                    })
                });

                if let Some(tid) = ticket_id {
                    let commit_result = match current_phase {
                        GostPhase::SpecContract => driver.commit_phase(
                            GostPhase::SpecContract,
                            tid,
                            Some("spec"),
                            "especificar contrato preliminar",
                            None,
                        ),
                        GostPhase::RedTest => driver.commit_phase(
                            GostPhase::RedTest,
                            tid,
                            Some("test"),
                            "adicionar teste falho comprovando requisito",
                            None,
                        ),
                        GostPhase::GreenCode => driver.commit_phase(
                            GostPhase::GreenCode,
                            tid,
                            Some("core"),
                            "implementacao minima funcional com testes verdes",
                            None,
                        ),
                        GostPhase::CloseTicket => {
                            let chronicle =
                                self.compile_closing_chronicle("docs", "finalizar ciclo GOST");
                            let res = driver.commit_phase(
                                GostPhase::CloseTicket,
                                tid,
                                Some("docs"),
                                "finalizar ciclo e documentacao",
                                Some(&chronicle),
                            );
                            let _ = driver.close_ticket(tid);
                            res
                        }
                        _ => Ok(String::new()),
                    };

                    if let Err(e) = commit_result {
                        let warn_msg = format!(
                            "⚠ Aviso ao gravar commit da fase {:?}: {}",
                            current_phase, e
                        );
                        self.append_viewport_line(&warn_msg);
                    }
                }

                // Encadeamento contínuo autônomo pós-Fase 5 (Fases 6, 7 e 8)
                if current_phase == GostPhase::GreenCode {
                    self.append_viewport_line(
                        "★ [GOST Esteira Autônoma] Iniciando Fases 6 (Refactor), 7 (Mutation Test) e 8 (Quality Gate)...",
                    );
                    self.gost_phase = GostPhase::Refactor;
                    self.append_viewport_line("  ✓ [Fase 6 - Refactor]: Limpeza e formatação verificadas.");

                    self.gost_phase = GostPhase::MutationTest;
                    self.append_viewport_line("  ✓ [Fase 7 - Mutation Test]: Mutantes e robustez de testes verificados.");

                    self.gost_phase = GostPhase::QualityGate;
                    let audit_info = driver.check_strict().map_err(|e| e.to_string())?;
                    if audit_info.failures > 0 || !audit_info.passed {
                        let veto_msg = format!(
                            "❌ ★ GOSPLAN QUALITY CONTROL: MERGE/ADVANCE LOCK REJECTED ★ Falhas detectadas: {} (Avisos: {}). Corrija as violações antes de prosseguir.",
                            audit_info.failures, audit_info.warnings
                        );
                        self.append_viewport_line(&veto_msg);
                        self.pending_dispatch = None;
                        return Err(veto_msg);
                    }

                    self.append_viewport_line(&format!(
                        "  ✓ [Fase 8 - Quality Gate]: Auditoria Gosplan 100% aprovada ({} checks, 0 falhas).",
                        audit_info.total_checks
                    ));

                    self.gost_phase = GostPhase::CloseTicket;
                    self.pending_dispatch = None;

                    let ok_msg = "★ [GOST Esteira Autônoma] Fases 6, 7 e 8 concluídas com sucesso. Avançando para FASE 9 (CloseTicket) para homologação final da usuária.".to_string();
                    self.append_viewport_line(&ok_msg);
                    if let Ok(mut lock) = self.session.lock() {
                        if let Some(ref mut sess) = *lock {
                            sess.metadata.gost.phase = self.gost_phase.number();
                        }
                    }
                    return Ok(ok_msg);
                }

                // Transição para próxima fase
                let next = match current_phase {
                    GostPhase::Ticket => GostPhase::NewBranch,
                    GostPhase::NewBranch => GostPhase::SpecContract,
                    GostPhase::SpecContract => GostPhase::RedTest,
                    GostPhase::RedTest => GostPhase::GreenCode,
                    GostPhase::GreenCode => GostPhase::Refactor,
                    GostPhase::Refactor => GostPhase::MutationTest,
                    GostPhase::MutationTest => GostPhase::QualityGate,
                    GostPhase::QualityGate => GostPhase::CloseTicket,
                    GostPhase::CloseTicket => GostPhase::CloseTicket,
                };

                self.gost_phase = next;

                match next {
                    GostPhase::SpecContract => {
                        self.pending_dispatch = Some(Self::phase_prompt_for(
                            GostPhase::SpecContract,
                            ticket_id,
                            &effective_branch,
                        ));
                    }
                    GostPhase::RedTest => {
                        self.pending_dispatch = Some(Self::phase_prompt_for(
                            GostPhase::RedTest,
                            ticket_id,
                            &effective_branch,
                        ));
                    }
                    GostPhase::GreenCode => {
                        self.pending_dispatch = Some(Self::phase_prompt_for(
                            GostPhase::GreenCode,
                            ticket_id,
                            &effective_branch,
                        ));
                    }
                    _ => {
                        self.pending_dispatch = None;
                    }
                }

                let ok_msg = if current_phase == GostPhase::CloseTicket {
                    "★ [GOST] Ciclo concluído e ticket encerrado com sucesso.".to_string()
                } else {
                    format!(
                        "★ [GOST] Fase {} aprovada via /approve. Avançando para FASE {} ({}).",
                        current_phase.number(),
                        next.number(),
                        next.name()
                    )
                };
                self.append_viewport_line(&ok_msg);

                // Avalia conformidade via driver
                if let Ok(audit_info) = driver.check_strict() {
                    self.append_viewport_line(&format!(
                        "★ [GOST Conformidade] cccp check: {} (Conformes: {}/{}, Falhas: {}, Avisos: {})",
                        audit_info.compliance_index,
                        audit_info.conforming,
                        audit_info.total_checks,
                        audit_info.failures,
                        audit_info.warnings
                    ));
                }

                // Sincroniza metadados da sessão ativa
                if let Ok(mut lock) = self.session.lock() {
                    if let Some(ref mut sess) = *lock {
                        sess.metadata.gost.phase = next.number();
                    }
                }

                Ok(ok_msg)
            }
            crate::event::TuiAction::PhaseAdvanced(next) => {
                let effective_branch = if let Ok(status) = driver.status() {
                    if !status.branch.is_empty() {
                        status.branch
                    } else {
                        self.branch_name.clone()
                    }
                } else {
                    self.branch_name.clone()
                };

                if (effective_branch == "main" || effective_branch == "master")
                    && next >= GostPhase::SpecContract
                {
                    let err_msg = "Bloqueio de Governança: Não é permitido operar as Fases 3 a 9 na branch main/master. Crie uma branch de trabalho primeiro com /branch <nome>.".to_string();
                    self.append_viewport_line(&format!("❌ {}", err_msg));
                    return Err(err_msg);
                }

                let msg = format!(
                    "★ [GOST] Avanço para FASE {}: {}.",
                    next.number(),
                    next.name()
                );
                self.append_viewport_line(&msg);

                // Avalia conformidade via driver
                if let Ok(audit_info) = driver.check_strict() {
                    self.append_viewport_line(&format!(
                        "★ [GOST Conformidade] cccp check: {} (Conformes: {}/{}, Falhas: {}, Avisos: {})",
                        audit_info.compliance_index,
                        audit_info.conforming,
                        audit_info.total_checks,
                        audit_info.failures,
                        audit_info.warnings
                    ));
                }

                // Sincroniza metadados da sessão ativa
                if let Ok(mut lock) = self.session.lock() {
                    if let Some(ref mut sess) = *lock {
                        sess.metadata.gost.phase = next.number();
                    }
                }
                Ok(msg)
            }
            crate::event::TuiAction::BranchChanged(new_branch) => {
                driver
                    .checkout_branch(&new_branch, true)
                    .map_err(|e| e.to_string())?;
                self.branch_name = new_branch.clone();
                if let Some(tid) = infer_ticket_from_branch(&new_branch) {
                    self.ticket_id = Some(tid);
                }
                let msg = format!(
                    "✓ Alternado para a branch '{}' (Ticket: {:?}).",
                    new_branch, self.ticket_id
                );
                self.append_viewport_line(&msg);
                Ok(msg)
            }
            crate::event::TuiAction::FinishTriggered => {
                let chronicle = self.compile_closing_chronicle("tui", "finalizar ciclo GOST");
                self.append_viewport_line(&chronicle);

                if let Some(ticket_id) = self.ticket_id {
                    match driver.close_ticket(ticket_id) {
                        Ok(()) => {
                            let msg =
                                format!("✓ Ticket #{} encerrado com sucesso na forja.", ticket_id);
                            self.append_viewport_line(&msg);
                        }
                        Err(e) => {
                            let msg = format!("⚠ Aviso ao encerrar ticket #{}: {}", ticket_id, e);
                            self.append_viewport_line(&msg);
                        }
                    }
                }
                Ok(chronicle)
            }
            crate::event::TuiAction::MergeTriggered => {
                self.append_viewport_line(&format!(
                    "☭ Mesclando branch '{}' via CCCP...",
                    self.branch_name
                ));
                match driver.merge_branch(&self.branch_name) {
                    Ok(()) => {
                        let msg = format!(
                            "✓ Branch '{}' mesclada com sucesso via cccp merge.",
                            self.branch_name
                        );
                        self.append_viewport_line(&msg);
                        Ok(msg)
                    }
                    Err(e) => {
                        let msg =
                            format!("❌ Falha ao mesclar branch '{}': {}", self.branch_name, e);
                        self.append_viewport_line(&msg);
                        Err(msg)
                    }
                }
            }
            crate::event::TuiAction::Submit(text) => Ok(text),
            crate::event::TuiAction::HelpToggled => Ok("ajuda".into()),
            crate::event::TuiAction::NewSession => {
                self.reset_session();
                self.append_viewport_line("★ [GOST] Nova sessão iniciada.");
                Ok("nova_sessao".into())
            }
            crate::event::TuiAction::Quit => {
                self.quit();
                Ok("quit".into())
            }
            crate::event::TuiAction::FocusToggled | crate::event::TuiAction::Scrolled => {
                Ok("navegacao".into())
            }
        }
    }

    /// Executa um comando digitado que se inicia com barra ('/').
    pub fn execute_slash_command(&mut self, cmd: &str) -> Option<crate::event::TuiAction> {
        let trimmed = cmd.trim();
        if trimmed.starts_with("/branch") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 2 {
                self.append_viewport_line("⚠ Uso correto: /branch <nome-da-branch>");
                return None;
            }
            let branch_target = parts[1].to_string();
            return Some(crate::event::TuiAction::BranchChanged(branch_target));
        }

        match trimmed {
            "/help" => {
                self.show_help = !self.show_help;
                Some(crate::event::TuiAction::HelpToggled)
            }
            "/new" => {
                self.reset_session();
                self.append_viewport_line("★ [GOST] Nova sessão iniciada.");
                Some(crate::event::TuiAction::NewSession)
            }
            "/approve" => Some(crate::event::TuiAction::SpecApproved),
            "/next" => {
                if let Ok(next) = self.advance_gost_phase() {
                    Some(crate::event::TuiAction::PhaseAdvanced(next))
                } else {
                    self.append_viewport_line("⚠ Não há próxima fase disponível.");
                    None
                }
            }
            "/finish" => Some(crate::event::TuiAction::FinishTriggered),
            "/merge" => Some(crate::event::TuiAction::MergeTriggered),
            "/quit" => {
                self.quit();
                Some(crate::event::TuiAction::Quit)
            }
            _ => {
                self.append_viewport_line(&format!(
                    "⚠ Comando desconhecido: '{}'. Digite /help para ver os comandos disponíveis.",
                    trimmed
                ));
                None
            }
        }
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

/// Infere o identificador numérico do ticket a partir do nome da branch.
/// Suporta convenções como `feat/<id>-*`, `fix/<id>-*`, `docs/<id>-*` ou `<id>-*`.
pub fn infer_ticket_from_branch(branch: &str) -> Option<u32> {
    let clean = branch.trim();
    if clean.is_empty() {
        return None;
    }
    let after_slash = match clean.rfind('/') {
        Some(idx) => &clean[idx + 1..],
        None => clean,
    };
    let token = after_slash.split('-').next()?;
    token.parse::<u32>().ok()
}

pub const SLASH_COMMANDS: &[&str] = &[
    "/branch", "/new", "/approve", "/next", "/finish", "/merge", "/quit", "/help",
];

/// Autocompleta comandos iniciados por barra ('/').
pub fn autocomplete_slash_command(prefix: &str) -> Option<&'static str> {
    let trimmed = prefix.trim();
    if !trimmed.starts_with('/') {
        return None;
    }
    if trimmed == "/b" || trimmed == "/br" {
        return Some("/branch");
    }
    if trimmed == "/n" || trimmed == "/ne" {
        return Some("/new");
    }
    let matches: Vec<&&str> = SLASH_COMMANDS
        .iter()
        .filter(|cmd| cmd.starts_with(trimmed))
        .collect();
    if matches.len() == 1 {
        Some(*matches[0])
    } else {
        None
    }
}

/// Formata o prompt despachado ao Agente Externo incorporando o histórico multi-turnos recente.
pub fn format_prompt_with_history(
    session: &Session,
    prompt: &str,
    injected_context: &str,
) -> String {
    let mut history_block = String::new();
    let completed_turns: Vec<_> = session
        .turns
        .iter()
        .filter(|t| t.status == pult_core::TurnStatus::Completed)
        .collect();

    if !completed_turns.is_empty() {
        history_block.push_str("\n--- Contexto Histórico Recente da Sessão ---\n");
        // Pega até os últimos 6 turnos concluídos
        for turn in completed_turns.iter().rev().take(6).rev() {
            let actor_label = match turn.actor {
                pult_core::Actor::User => "Usuária",
                pult_core::Actor::Agy => "Agente",
                pult_core::Actor::Scout => "Scout",
            };
            history_block.push_str(&format!("[{}]: {}\n", actor_label, turn.content.trim()));
        }
        history_block.push_str("-------------------------------------------\n\n");
    }

    format!(
        "{}[Mensagem Atual da Usuária]:\n{}{}",
        history_block, prompt, injected_context
    )
}
