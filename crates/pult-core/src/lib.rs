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
pub const ROOT_VERSION: &str = include_str!("../../../VERSION");

pub static SHORT_VERSION: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let trimmed = ROOT_VERSION.trim();
    trimmed.split('+').next().unwrap_or(trimmed).to_string()
});

/// Retorna a versão canônica completa SemVer gerenciada pelo repositório.
pub fn canonical_version() -> &'static str {
    ROOT_VERSION.trim()
}

/// Retorna a versão curta cortada no caractere '+' (ex.: '0.1.0-dev.87').
pub fn short_version() -> &'static str {
    &SHORT_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }
}
