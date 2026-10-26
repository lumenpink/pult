use pult_core::ast::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const SAMPLE_RUST_SOURCE: &str = r#"
use std::collections::HashMap;

/// Configuração pública do limitador de requisições.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimiter {
    pub max_tokens: usize,
    pub refill_rate: u64,
    private_bucket: HashMap<String, usize>,
}

/// Modos de operação do limitador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimiterMode {
    Strict,
    Permissive,
}

pub type TokenCount = usize;

pub const DEFAULT_MAX_TOKENS: usize = 1000;

/// Contrato público do limitador.
pub trait Limiter {
    fn check_quota(&self, client_id: &str) -> bool;
    fn consume(&mut self, client_id: &str, amount: usize) -> Result<(), String>;
}

impl RateLimiter {
    pub fn new(max_tokens: usize, refill_rate: u64) -> Self {
        // Corpo funcional longo que deve ser totalmente omitido na extração de AST
        let mut map = HashMap::new();
        for i in 0..100 {
            map.insert(format!("client_{}", i), max_tokens);
        }
        Self {
            max_tokens,
            refill_rate,
            private_bucket: map,
        }
    }

    fn internal_helper(&self) -> bool {
        // Função privada que deve ser completamente descartada
        true
    }
}

impl Limiter for RateLimiter {
    fn check_quota(&self, client_id: &str) -> bool {
        // Implementação funcional com lógica executável
        if let Some(tokens) = self.private_bucket.get(client_id) {
            *tokens > 0
        } else {
            false
        }
    }

    fn consume(&mut self, client_id: &str, amount: usize) -> Result<(), String> {
        let entry = self.private_bucket.entry(client_id.to_string()).or_insert(self.max_tokens);
        if *entry >= amount {
            *entry -= amount;
            Ok(())
        } else {
            Err("Quota excedida".to_string())
        }
    }
}

pub fn public_utility(val: usize) -> usize {
    // Utilitário público cujo corpo deve ser omitido
    val * 42
}

fn private_utility() {
    // Utilitário privado ignorado
}
"#;

#[test]
fn test_rust_ast_extractor_omits_bodies_and_private_items() {
    let extractor = RustAstExtractor::new();
    let extracted = extractor
        .extract_from_str(Path::new("src/rate_limiter.rs"), SAMPLE_RUST_SOURCE)
        .expect("extração de AST deve funcionar");

    // Deve conter tipos públicos
    assert!(extracted.contains("pub struct RateLimiter"));
    assert!(extracted.contains("pub enum LimiterMode"));
    assert!(extracted.contains("pub type TokenCount"));
    assert!(extracted.contains("pub const DEFAULT_MAX_TOKENS"));
    assert!(extracted.contains("pub trait Limiter"));

    // Deve conter assinaturas públicas de métodos com corpo omitido ';'
    assert!(extracted.contains("pub fn new"));
    assert!(extracted.contains("fn check_quota"));
    assert!(extracted.contains("fn consume"));
    assert!(extracted.contains("pub fn public_utility"));

    // NÃO deve conter funções e utilitários privados
    assert!(!extracted.contains("internal_helper"));
    assert!(!extracted.contains("private_utility"));

    // NÃO deve conter lógica interna dos corpos funcionais
    assert!(!extracted.contains("map.insert"));
    assert!(!extracted.contains("val * 42"));
    assert!(!extracted.contains("Quota excedida"));

    // Verificação de redução dramática no tamanho (economia de tokens)
    assert!(extracted.len() < SAMPLE_RUST_SOURCE.len() / 2);
}

#[test]
fn test_rust_ast_extractor_directory_scanning() {
    let temp_dir = tempdir().expect("pasta temporária");
    let src_dir = temp_dir.path().join("src");
    let target_dir = temp_dir.path().join("target");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&target_dir).unwrap();

    fs::write(src_dir.join("lib.rs"), SAMPLE_RUST_SOURCE).unwrap();
    fs::write(target_dir.join("generated.rs"), "pub struct IgnoreMe;").unwrap();

    let extractor = RustAstExtractor::new();
    let results = extractor
        .extract_from_dir(temp_dir.path(), &["target", ".git"])
        .expect("varredura deve suceder");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].relative_path, "src/lib.rs");
    assert!(results[0].reduction_ratio() > 40.0);
}

#[test]
fn test_mutation_complex_generics_lifetimes_and_async() {
    let source = r#"
pub struct AsyncWorker<'a, T: Send + Sync + 'static>
where
    T: std::fmt::Debug,
{
    pub name: &'a str,
    pub payload: T,
}

pub trait JobQueue<'a> {
    async fn enqueue(&mut self, item: &'a str) -> Result<u64, String>;
}
"#;

    let extractor = RustAstExtractor::new();
    let extracted = extractor
        .extract_from_str(Path::new("src/worker.rs"), source)
        .expect("deve processar generics e async");

    assert!(extracted.contains("pub struct AsyncWorker"));
    assert!(extracted.contains("where T : std :: fmt :: Debug"));
    assert!(extracted.contains("pub trait JobQueue"));
    assert!(extracted.contains("async fn enqueue"));
}

#[test]
fn test_mutation_parse_error_on_invalid_syntax() {
    let malformed = "pub struct Broken { incomplete";
    let extractor = RustAstExtractor::new();
    let result = extractor.extract_from_str(Path::new("src/broken.rs"), malformed);
    assert!(result.is_err());
    match result {
        Err(AstError::ParseError { file, .. }) => {
            assert_eq!(file, "src/broken.rs");
        }
        other => panic!("esperava ParseError, obteve {:?}", other),
    }
}

#[test]
fn test_mutation_io_error_on_missing_file() {
    let extractor = RustAstExtractor::new();
    let result = extractor.extract_from_file(Path::new("/caminho/completamente/inexistente.rs"));
    assert!(result.is_err());
    match result {
        Err(AstError::Io { .. }) => {}
        other => panic!("esperava AstError::Io, obteve {:?}", other),
    }
}

#[test]
fn test_mutation_reduction_ratio_edge_cases() {
    let file = ExtractedAstFile {
        file_path: std::path::PathBuf::from("dummy.rs"),
        relative_path: "dummy.rs".to_string(),
        extracted_signatures: "".to_string(),
        raw_byte_count: 0,
        reduced_byte_count: 0,
    };
    assert_eq!(file.reduction_ratio(), 0.0);
}
