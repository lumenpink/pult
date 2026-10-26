//! `pult-core` — Motor de domínio, extratores de AST, cliente Scout (Ollama)
//! e guardião normativo do ciclo GOST DD-TDD para o PULT.

pub mod ast;
pub mod domain;
pub mod driver;
pub mod repo;
pub mod runner;
pub mod scout;

pub use ast::{
    AstError, AstExtractor, ExtractedAstFile, PolyglotAstExtractor, RustAstExtractor,
    SupportedLanguage,
};
pub use domain::{
    Actor, ContextFilter, FileSessionStorage, GostMetadata, GostPhase, Session, SessionError,
    SessionMetadata, SessionStorage, TicketRef, Turn, TurnStatus,
};
pub use driver::{CccpGitDriver, DriverError, GitStatus, QualityAudit, RepoDriver, TicketDetails};
pub use repo::{resolve_repo_path, resolve_repo_path_with_env, RepoError};
pub use runner::{
    AgyRunner, AgyStreamEvent, ResumeInspection, RunnerError, RunnerOptions, RunnerStream,
};
pub use scout::{
    GpuModelInfo, OllamaClient, ScoutEngine, ScoutError, ScoutPromptContext, ScoutResult,
    DEFAULT_OLLAMA_MODEL, DEFAULT_OLLAMA_URL,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }
}
