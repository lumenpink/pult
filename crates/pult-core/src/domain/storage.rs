use crate::domain::error::SessionError;
use crate::domain::session::{Session, SessionMetadata};
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
}

impl SessionStorage for FileSessionStorage {
    fn save(&self, _session: &Session) -> Result<PathBuf, SessionError> {
        todo!("Fase 5: Implementação de FileSessionStorage::save")
    }

    fn load(&self, _session_id_or_filename: &str) -> Result<Session, SessionError> {
        todo!("Fase 5: Implementação de FileSessionStorage::load")
    }

    fn list(&self) -> Result<Vec<SessionMetadata>, SessionError> {
        todo!("Fase 5: Implementação de FileSessionStorage::list")
    }

    fn find_by_ticket(&self, _ticket_id: u32) -> Result<Option<Session>, SessionError> {
        todo!("Fase 5: Implementação de FileSessionStorage::find_by_ticket")
    }
}
