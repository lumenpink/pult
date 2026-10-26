use crate::domain::error::SessionError;
use crate::domain::session::{Session, SessionMetadata};
use std::fs;
use std::path::{Path, PathBuf};

/// Contrato para persistência e recuperação de sessões em disco sob `.antigravity/pult/sessions/`.
pub trait SessionStorage: Send + Sync {
    /// Salva ou atualiza uma sessão atomicamente em disco.
    fn save(&self, session: &Session) -> Result<PathBuf, SessionError>;

    /// Carrega uma sessão completa a partir de seu identificador ou nome de arquivo.
    fn load(&self, session_id_or_filename: &str) -> Result<Session, SessionError>;

    /// Lista os metadados de todas as sessões registradas no diretório.
    fn list(&self) -> Result<Vec<SessionMetadata>, SessionError>;

    /// Localiza uma sessão formal vinculada ao ID do ticket, se houver.
    fn find_by_ticket(&self, ticket_id: u32) -> Result<Option<Session>, SessionError>;
}

/// Implementação padrão do repositório de persistência baseada no sistema de arquivos.
pub struct FileSessionStorage {
    pub root_dir: PathBuf,
}

impl FileSessionStorage {
    /// Cria uma nova instância apontando para o diretório raiz das sessões.
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    /// Retorna o caminho do diretório padrão em `.antigravity/pult/sessions/`.
    pub fn default_path(workspace_root: impl AsRef<Path>) -> PathBuf {
        workspace_root
            .as_ref()
            .join(".antigravity")
            .join("pult")
            .join("sessions")
    }

    fn resolve_path(&self, id_or_filename: &str) -> PathBuf {
        let name = if id_or_filename.ends_with(".md") {
            id_or_filename.to_string()
        } else {
            format!("{}.md", id_or_filename)
        };
        self.root_dir.join(name)
    }
}

impl SessionStorage for FileSessionStorage {
    fn save(&self, session: &Session) -> Result<PathBuf, SessionError> {
        fs::create_dir_all(&self.root_dir)?;
        let target_path = self.resolve_path(&session.filename());
        let content = session.to_markdown()?;

        // Gravação atômica via arquivo temporário
        let tmp_path = target_path.with_extension("tmp");
        fs::write(&tmp_path, content)?;
        fs::rename(&tmp_path, &target_path)?;

        Ok(target_path)
    }

    fn load(&self, session_id_or_filename: &str) -> Result<Session, SessionError> {
        let path = self.resolve_path(session_id_or_filename);
        if !path.exists() {
            return Err(SessionError::NotFound(session_id_or_filename.to_string()));
        }

        let content = fs::read_to_string(&path)?;
        Session::from_markdown(&content)
    }

    fn list(&self) -> Result<Vec<SessionMetadata>, SessionError> {
        if !self.root_dir.exists() {
            return Ok(Vec::new());
        }

        let mut list = Vec::new();
        for entry in fs::read_dir(&self.root_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = Session::from_markdown(&content) {
                        list.push(session.metadata);
                    }
                }
            }
        }

        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(list)
    }

    fn find_by_ticket(&self, ticket_id: u32) -> Result<Option<Session>, SessionError> {
        if !self.root_dir.exists() {
            return Ok(None);
        }

        for entry in fs::read_dir(&self.root_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = Session::from_markdown(&content) {
                        if let Some(ticket) = &session.metadata.ticket {
                            if ticket.id == ticket_id {
                                return Ok(Some(session));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}
