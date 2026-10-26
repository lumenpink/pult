//! `pult-core` — Motor de domínio, extratores de AST, cliente Scout (Ollama)
//! e guardião normativo do ciclo GOST DD-TDD para o PULT.

pub mod domain;
pub mod driver;

pub use domain::{
    Actor, ContextFilter, FileSessionStorage, GostMetadata, GostPhase, Session, SessionError,
    SessionMetadata, SessionStorage, TicketRef, Turn, TurnStatus,
};
pub use driver::{CccpGitDriver, DriverError, GitStatus, QualityAudit, RepoDriver, TicketDetails};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }
}
