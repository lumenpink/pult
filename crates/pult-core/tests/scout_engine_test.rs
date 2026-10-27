use pult_core::ast::ExtractedAstFile;
use pult_core::scout::{
    GpuModelInfo, OllamaClient, ScoutEngine, ScoutError, ScoutPromptContext, ScoutResult,
};
use std::path::PathBuf;

#[test]
fn test_ollama_client_endpoint_resolution() {
    let client = OllamaClient::with_base_url("http://custom-host:11434");
    assert_eq!(client.base_url(), "http://custom-host:11434");

    let default_client = OllamaClient::new();
    assert!(default_client.base_url().contains("11434"));
}

#[test]
fn test_parse_ps_vram_telemetry() {
    let ps_json = r#"{
        "models": [
            {
                "name": "qwen2.5-coder:3b",
                "model": "qwen2.5-coder:3b",
                "size": 1929912626,
                "size_vram": 1929912626,
                "expires_at": "2026-10-26T18:00:00Z"
            }
        ]
    }"#;

    let models: Vec<GpuModelInfo> =
        OllamaClient::parse_running_models(ps_json).expect("deve parsear /api/ps");
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "qwen2.5-coder:3b");
    assert_eq!(models[0].size_vram, 1929912626);
    assert!(models[0].is_in_vram());
}

#[test]
fn test_scout_prompt_context_construction() {
    let files = vec![ExtractedAstFile {
        file_path: PathBuf::from("src/domain/ports.rs"),
        relative_path: "src/domain/ports.rs".to_string(),
        extracted_signatures: "pub trait RateLimiter { fn check(&self); }".to_string(),
        raw_byte_count: 500,
        reduced_byte_count: 50,
    }];

    let ctx = ScoutPromptContext::new("Implementar rate limiter", &files);
    let prompt = ctx.format_prompt();

    assert!(prompt.contains("Implementar rate limiter"));
    assert!(prompt.contains("src/domain/ports.rs"));
    assert!(prompt.contains("pub trait RateLimiter"));
    assert!(prompt.contains("injected_ast"));
}

#[test]
fn test_scout_parse_injected_ast_response() {
    let response = r#"
Com base na tarefa informada, aqui estão os símbolos essenciais:

```yaml
injected_ast:
  - "src/domain/ports.rs: pub trait RateLimiter"
  - "src/auth/token.rs: pub struct TokenBucket"
```
"#;

    let result = ScoutEngine::parse_scout_response(response, "qwen2.5-coder:3b", Some(1929912626))
        .expect("deve parsear bloco injected_ast");

    assert_eq!(result.symbols.len(), 2);
    assert!(result.symbols[0].contains("RateLimiter"));
    assert!(result.symbols[1].contains("TokenBucket"));
    assert_eq!(result.model_used, "qwen2.5-coder:3b");
    assert_eq!(result.vram_allocated_bytes, Some(1929912626));
    assert!(result.yaml_block.contains("injected_ast:"));
}

#[tokio::test]
async fn test_scout_unreachable_endpoint_and_bypass() {
    let unreachable_client = OllamaClient::with_base_url("http://127.0.0.1:59998");
    let engine = ScoutEngine::with_client(unreachable_client);

    let check = engine.client().check_health().await;
    assert!(check.is_err());
    match check {
        Err(ScoutError::OllamaUnreachable { .. }) => {}
        other => panic!("esperava OllamaUnreachable, obteve {:?}", other),
    }

    // Bypass gracioso sem erro fatal
    let files = vec![];
    let bypassed: ScoutResult = engine
        .filter_with_bypass("tarefa de teste", &files)
        .await
        .expect("bypass deve retornar Ok com resultado vazio");

    assert!(bypassed.bypassed);
    assert!(bypassed.symbols.is_empty());
}

#[tokio::test]
async fn test_live_ollama_connectivity_if_running() {
    let client = OllamaClient::new();
    if let Ok(tags) = client.list_tags().await {
        println!("Ollama tags disponíveis localmente: {:?}", tags);
        assert!(!tags.is_empty());
    }
}

#[test]
fn test_mutation_malformed_ps_json() {
    let bad_json = "not a json string";
    let res = OllamaClient::parse_running_models(bad_json);
    assert!(res.is_err());
    match res {
        Err(ScoutError::ParseError(_)) => {}
        other => panic!("esperava ParseError, obteve {:?}", other),
    }
}

#[test]
fn test_mutation_response_without_code_fences() {
    let plain_list =
        "- \"src/auth/token.rs: pub struct TokenBucket\"\n- \"src/lib.rs: pub fn run\"";
    let res = ScoutEngine::parse_scout_response(plain_list, "qwen2.5-coder:3b", None)
        .expect("deve parsear lista simples sem fences");
    assert_eq!(res.symbols.len(), 2);
    assert!(res.symbols[0].contains("TokenBucket"));
    assert!(res.symbols[1].contains("run"));
}

#[test]
fn test_mutation_empty_ast_context() {
    let ctx = ScoutPromptContext::new("Tarefa sem arquivos", &[]);
    let prompt = ctx.format_prompt();
    assert!(prompt.contains("Tarefa sem arquivos"));
    assert!(!prompt.contains("###"));
}

#[test]
fn test_mutation_engine_custom_model_and_url_normalization() {
    let client = OllamaClient::with_base_url("http://localhost:11434///");
    assert_eq!(client.base_url(), "http://localhost:11434");

    let engine = ScoutEngine::with_client(client).with_model("llama3.2:1b");
    assert_eq!(engine.model_name(), "llama3.2:1b");
}

#[tokio::test]
async fn test_scout_generate_session_slug_format_and_length_limit() {
    let engine = ScoutEngine::new();

    // Frase longa com caracteres especiais e acentos
    let input = "Implementar o Limitador de Taxas em Memória com Token Bucket & Redis! [Fase 3]";
    let slug = engine.generate_session_slug(input).await;

    // Deve ser <= 60 caracteres
    assert!(
        slug.len() <= 60,
        "slug deve ter no máximo 60 chars, teve {}",
        slug.len()
    );

    // Deve ser não-vazio
    assert!(!slug.is_empty());

    // Deve conter apenas minúsculas, números e hífens
    assert!(
        slug.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
        "slug contém caracteres inválidos: {}",
        slug
    );

    // Não deve começar nem terminar com hífen
    assert!(!slug.starts_with('-'));
    assert!(!slug.ends_with('-'));
    assert!(!slug.contains("--"));

    // Entrada extremamente longa (> 200 chars)
    let long_input = "Um prompt extremamente longo que detalha exaustivamente cada nuance arquitetural e os requisitos não-funcionais que precisam ser cobertos durante a execução da esteira DD-TDD do GOST";
    let long_slug = engine.generate_session_slug(long_input).await;
    assert!(long_slug.len() <= 60);
    assert!(!long_slug.starts_with('-'));
    assert!(!long_slug.ends_with('-'));

    // Entrada vazia ou somente caracteres especiais
    let empty_slug = engine.generate_session_slug("!@#$%^&*()").await;
    assert_eq!(empty_slug, "sessao-trabalho");
}

#[test]
fn test_mutation_sanitize_kebab_slug_direct_edge_cases() {
    use pult_core::scout::engine::sanitize_kebab_slug;

    // Mutante 1: Acentos e cedilha variados
    let accented = "Ação, Éxito, Ícone, Órgão, Úmido, Criança & Pão!";
    let clean = sanitize_kebab_slug(accented);
    assert_eq!(clean, "acao-exito-icone-orgao-umido-crianca-pao");

    // Mutante 2: Truncamento exato em 60 chars sem partir palavra
    let text = "este-e-um-slug-com-palavras-perfeitas-para-testar-o-limite-exato-de-sessenta-chars";
    let truncated = sanitize_kebab_slug(text);
    assert!(truncated.len() <= 60);
    assert!(!truncated.ends_with('-'));

    // Mutante 3: Texto de 59 caracteres não deve ser reduzido
    let exact_59 = "a-b-c-d-e-f-g-h-i-j-k-l-m-n-o-p-q-r-s-t-u-v-w-x-y-z-0-1-2-3-4-5-6-7";
    let res = sanitize_kebab_slug(exact_59);
    assert!(res.len() <= 60);
}
