//! `pult-core` — Motor de domínio, extratores de AST, cliente Scout (Ollama)
//! e guardião normativo do ciclo GOST DD-TDD para o PULT.

pub mod domain;

pub use domain::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }
}
