use crate::domain::error::SessionError;
use crate::domain::gost::GostPhase;
use serde::{Deserialize, Serialize};

/// Ator participante de um turno de conversação no PULT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Actor {
    User,
    Scout,
    Agy,
}

impl Actor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Scout => "scout",
            Self::Agy => "agy",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "user" => Some(Self::User),
            "scout" => Some(Self::Scout),
            "agy" => Some(Self::Agy),
            _ => None,
        }
    }
}

/// Estado de conclusão de um turno ou sessão.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TurnStatus {
    #[default]
    Completed,
    InProgress,
    Interrupted,
    Truncated,
}

impl TurnStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::InProgress => "in_progress",
            Self::Interrupted => "interrupted",
            Self::Truncated => "truncated",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "in_progress" => Self::InProgress,
            "interrupted" => Self::Interrupted,
            "truncated" => Self::Truncated,
            _ => Self::Completed,
        }
    }
}

/// Um turno cronológico da sessão de trabalho.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    pub turn_number: usize,
    pub actor: Actor,
    pub timestamp: String,
    pub content: String,
    pub status: TurnStatus,
    pub tokens: Option<usize>,
    pub tokens_out: Option<usize>,
    pub elapsed_ms: Option<u64>,
}

/// Referência a um ticket ativo na forja.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketRef {
    pub id: u32,
    pub state: String,
    pub branch: String,
}

/// Metadados normativos do GOST registrados no frontmatter da sessão.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GostMetadata {
    pub phase: u8,
    pub command: String,
}

/// Metadados do filtro local do Scout (GPU).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextFilter {
    pub scout_model: String,
    pub filtered_symbols: Vec<String>,
}

/// Frontmatter YAML canônico persistido no início do arquivo de sessão (`*.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<TicketRef>,
    pub gost: GostMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_filter: Option<ContextFilter>,
}

/// Representação integral em memória de uma sessão de trabalho do PULT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub metadata: SessionMetadata,
    pub turns: Vec<Turn>,
}

impl Session {
    /// Cria uma nova sessão em modo rascunho (sem ticket vinculado).
    pub fn new_draft(slug: &str) -> Self {
        let now_sec = current_epoch_seconds();
        let (y, m, d) = days_to_ymd(now_sec / 86400);
        let date_str = format!("{:04}{:02}{:02}", y, m, d);
        let rand_tag = (now_sec % 90000 + 10000) as u32;
        let clean_slug = sanitize_slug(slug);
        let session_id = format!("t{:05}-{}-{}", rand_tag, date_str, clean_slug);
        let timestamp = epoch_to_iso(now_sec);

        Self {
            metadata: SessionMetadata {
                session_id,
                created_at: timestamp.clone(),
                updated_at: timestamp,
                ticket: None,
                gost: GostMetadata {
                    phase: 1,
                    command: "TDD".to_string(),
                },
                context_filter: None,
            },
            turns: Vec::new(),
        }
    }

    /// Cria uma nova sessão formal vinculada a um ticket e branch.
    pub fn new_formal(ticket_id: u32, slug: &str, branch: &str) -> Self {
        let now_sec = current_epoch_seconds();
        let (y, m, d) = days_to_ymd(now_sec / 86400);
        let date_str = format!("{:04}{:02}{:02}", y, m, d);
        let clean_slug = sanitize_slug(slug);
        let session_id = format!("{:06}-{}-{}", ticket_id, date_str, clean_slug);
        let timestamp = epoch_to_iso(now_sec);

        Self {
            metadata: SessionMetadata {
                session_id,
                created_at: timestamp.clone(),
                updated_at: timestamp,
                ticket: Some(TicketRef {
                    id: ticket_id,
                    state: "open".to_string(),
                    branch: branch.to_string(),
                }),
                gost: GostMetadata {
                    phase: 3,
                    command: "TDD".to_string(),
                },
                context_filter: None,
            },
            turns: Vec::new(),
        }
    }

    /// Informa se a sessão é provisória/rascunho (sem ticket vinculado).
    pub fn is_draft(&self) -> bool {
        self.metadata.ticket.is_none()
    }

    /// Retorna o nome de arquivo padronizado `.md` para persistência.
    pub fn filename(&self) -> String {
        format!("{}.md", self.metadata.session_id)
    }

    /// Retorna a fase GOST ativa registrada nos metadados.
    pub fn gost_phase(&self) -> Result<GostPhase, SessionError> {
        GostPhase::from_u8(self.metadata.gost.phase)
    }

    /// Serializa a sessão completa para o formato canônico em Markdown.
    pub fn to_markdown(&self) -> Result<String, SessionError> {
        let yaml_frontmatter = serde_yaml::to_string(&self.metadata)?;
        let mut out = String::new();
        out.push_str("---\n");
        out.push_str(&yaml_frontmatter);
        out.push_str("---\n\n");

        for turn in &self.turns {
            out.push_str("<!-- TURN: ");
            out.push_str(&turn.turn_number.to_string());
            out.push_str(" | ACTOR: ");
            out.push_str(turn.actor.as_str());
            out.push_str(" | TIME: ");
            out.push_str(&turn.timestamp);

            if let Some(t) = turn.tokens {
                out.push_str(" | TOKENS: ");
                out.push_str(&t.to_string());
            }
            if let Some(to) = turn.tokens_out {
                out.push_str(" | TOKENS_OUT: ");
                out.push_str(&to.to_string());
            }
            if let Some(ms) = turn.elapsed_ms {
                let secs = ms as f64 / 1000.0;
                out.push_str(&format!(" | ELAPSED: {:.2}s", secs));
            }
            if turn.status != TurnStatus::Completed {
                out.push_str(" | STATUS: ");
                out.push_str(turn.status.as_str());
            }

            out.push_str(" -->\n");
            out.push_str(&turn.content);
            out.push_str("\n\n");
        }

        Ok(out)
    }

    /// Desserializa um conteúdo Markdown para uma entidade de sessão.
    pub fn from_markdown(content: &str) -> Result<Self, SessionError> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return Err(SessionError::Parse(
                "Frontmatter YAML ausente (arquivo não inicia com '---')".to_string(),
            ));
        }

        let after_start = &trimmed[3..];
        let end_idx = after_start.find("\n---").ok_or_else(|| {
            SessionError::Parse(
                "Delimitador de fechamento '---\' do frontmatter não encontrado".to_string(),
            )
        })?;

        let frontmatter_str = &after_start[..end_idx].trim();
        let metadata: SessionMetadata = serde_yaml::from_str(frontmatter_str)?;

        let body = &after_start[end_idx + 4..];
        let turns = parse_markdown_turns(body);

        Ok(Self { metadata, turns })
    }

    /// Adiciona um novo turno sequencial à sessão, calculando automaticamente seu número.
    pub fn add_turn(&mut self, actor: Actor, content: String, status: TurnStatus) -> &mut Turn {
        let turn_number = self.turns.len() + 1;
        let now_sec = current_epoch_seconds();
        let timestamp = epoch_to_iso(now_sec);
        self.metadata.updated_at = timestamp.clone();

        self.turns.push(Turn {
            turn_number,
            actor,
            timestamp,
            content,
            status,
            tokens: None,
            tokens_out: None,
            elapsed_ms: None,
        });

        self.turns.last_mut().expect("turno recém adicionado")
    }

    /// Define o tempo de resposta em milissegundos no último turno registrado.
    pub fn set_last_turn_elapsed(&mut self, ms: u64) {
        if let Some(last_turn) = self.turns.last_mut() {
            last_turn.elapsed_ms = Some(ms);
            let now_sec = current_epoch_seconds();
            self.metadata.updated_at = epoch_to_iso(now_sec);
        }
    }

    /// Acrescenta dados incrementais ao conteúdo do último turno (útil para streaming).
    pub fn append_to_last_turn(&mut self, chunk: &str) {
        if let Some(last_turn) = self.turns.last_mut() {
            last_turn.content.push_str(chunk);
            let now_sec = current_epoch_seconds();
            self.metadata.updated_at = epoch_to_iso(now_sec);
        }
    }

    /// Atualiza o status do último turno e o timestamp de atualização da sessão.
    pub fn update_last_turn_status(&mut self, status: TurnStatus) {
        if let Some(last_turn) = self.turns.last_mut() {
            last_turn.status = status;
            let now_sec = current_epoch_seconds();
            self.metadata.updated_at = epoch_to_iso(now_sec);
        }
    }
}

fn parse_markdown_turns(body: &str) -> Vec<Turn> {
    let mut turns = Vec::new();
    let marker = "<!-- TURN:";
    let mut cursor = 0;

    while let Some(start_rel) = body[cursor..].find(marker) {
        let comment_start = cursor + start_rel;
        let tag_close_rel = match body[comment_start..].find("-->") {
            Some(idx) => idx,
            None => break,
        };
        let comment_end = comment_start + tag_close_rel + 3;
        let header = &body[comment_start + marker.len()..comment_start + tag_close_rel].trim();

        // Determina onde o conteúdo deste turno termina (próximo <!-- TURN: ou fim do body)
        let next_turn_rel = body[comment_end..].find(marker);
        let content_end = match next_turn_rel {
            Some(idx) => comment_end + idx,
            None => body.len(),
        };

        let raw_content = body[comment_end..content_end].trim().to_string();
        let (turn_number, actor, timestamp, tokens, tokens_out, status, elapsed_ms) =
            parse_turn_header(header);

        turns.push(Turn {
            turn_number,
            actor,
            timestamp,
            content: raw_content,
            status,
            tokens,
            tokens_out,
            elapsed_ms,
        });

        cursor = content_end;
    }

    turns
}

fn parse_turn_header(
    header: &str,
) -> (
    usize,
    Actor,
    String,
    Option<usize>,
    Option<usize>,
    TurnStatus,
    Option<u64>,
) {
    let mut turn_number = 1;
    let mut actor = Actor::User;
    let mut timestamp = String::new();
    let mut tokens = None;
    let mut tokens_out = None;
    let mut status = TurnStatus::Completed;
    let mut elapsed_ms = None;

    for part in header.split('|') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("ACTOR:") {
            if let Some(a) = Actor::parse(val) {
                actor = a;
            }
        } else if let Some(val) = part.strip_prefix("TIME:") {
            timestamp = val.trim().to_string();
        } else if let Some(val) = part.strip_prefix("TOKENS:") {
            tokens = val.trim().parse().ok();
        } else if let Some(val) = part.strip_prefix("TOKENS_OUT:") {
            tokens_out = val.trim().parse().ok();
        } else if let Some(val) = part.strip_prefix("STATUS:") {
            status = TurnStatus::parse(val);
        } else if let Some(val) = part.strip_prefix("ELAPSED:") {
            let val = val.trim();
            if let Some(s_val) = val.strip_suffix('s') {
                if let Ok(secs) = s_val.parse::<f64>() {
                    elapsed_ms = Some((secs * 1000.0).round() as u64);
                }
            } else if let Some(ms_val) = val.strip_suffix("ms") {
                if let Ok(ms) = ms_val.parse::<u64>() {
                    elapsed_ms = Some(ms);
                }
            }
        } else if let Ok(n) = part.trim().parse::<usize>() {
            turn_number = n;
        }
    }

    (
        turn_number,
        actor,
        timestamp,
        tokens,
        tokens_out,
        status,
        elapsed_ms,
    )
}

fn sanitize_slug(slug: &str) -> String {
    let s: String = slug
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    // Colapsa múltiplos traços consecutivos e apara extremidades
    let parts: Vec<&str> = s.split('-').filter(|p| !p.is_empty()).collect();
    let collapsed = parts.join("-");

    if collapsed.len() > 60 {
        collapsed[..60].trim_matches('-').to_string()
    } else if collapsed.is_empty() {
        "sessao".to_string()
    } else {
        collapsed
    }
}

pub(crate) fn current_epoch_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m as u32, d as u32)
}

pub(crate) fn epoch_to_iso(epoch_secs: u64) -> String {
    let days = epoch_secs / 86400;
    let rem = epoch_secs % 86400;
    let hours = rem / 3600;
    let minutes = (rem % 3600) / 60;
    let seconds = rem % 60;
    let (y, m, d) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}
