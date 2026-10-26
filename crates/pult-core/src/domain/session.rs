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
    /// Contrato: cria uma nova sessão em modo rascunho (sem ticket vinculado).
    pub fn new_draft(_slug: &str) -> Self {
        todo!("Fase 5: Implementação de new_draft")
    }

    /// Contrato: cria uma nova sessão formal vinculada a um ticket e branch.
    pub fn new_formal(_ticket_id: u32, _slug: &str, _branch: &str) -> Self {
        todo!("Fase 5: Implementação de new_formal")
    }

    /// Contrato: informa se a sessão é provisória/rascunho (sem ticket vinculado).
    pub fn is_draft(&self) -> bool {
        self.metadata.ticket.is_none()
    }

    /// Contrato: retorna o nome de arquivo padronizado `.md` para persistência.
    pub fn filename(&self) -> String {
        todo!("Fase 5: Implementação de filename")
    }

    /// Contrato: serializa a sessão completa para o formato canônico em Markdown.
    pub fn to_markdown(&self) -> Result<String, SessionError> {
        todo!("Fase 5: Implementação de to_markdown")
    }

    /// Contrato: desserializa um conteúdo Markdown para uma entidade de sessão.
    pub fn from_markdown(_content: &str) -> Result<Self, SessionError> {
        todo!("Fase 5: Implementação de from_markdown")
    }

    /// Retorna a fase GOST ativa registrada nos metadados.
    pub fn gost_phase(&self) -> Result<GostPhase, SessionError> {
        GostPhase::from_u8(self.metadata.gost.phase)
    }
}
