use std::fs;
use std::path::Path;

#[test]
fn test_ci_workflow_conforms_to_forgejo_cluster() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let workflow_path = workspace_root
        .join(".forgejo")
        .join("workflows")
        .join("ci.yaml");

    let content = fs::read_to_string(&workflow_path)
        .expect("arquivo .forgejo/workflows/ci.yaml deve existir");

    // Proíbe etiqueta 'native' que causa falha imediata por ausência de runner
    assert!(
        !content.contains("runs-on: native"),
        "Workflow não pode conter 'runs-on: native' (inexistente no cluster Forgejo)"
    );

    // Exige runner 'ubuntu-latest' e container canônico 'gost-ci-cross'
    assert!(
        content.contains("runs-on: ubuntu-latest"),
        "Workflow deve utilizar 'runs-on: ubuntu-latest'"
    );
    assert!(
        content.contains("image: gambi.2lp.in/lumen/gost-ci-cross:latest"),
        "Workflow deve executar dentro do container soberano 'gost-ci-cross:latest'"
    );

    // Exige validação de qualidade e compilação
    assert!(
        content.contains("cargo check"),
        "Workflow deve executar cargo check"
    );
    assert!(
        content.contains("cargo test"),
        "Workflow deve executar cargo test"
    );
    assert!(
        content.contains("cccp check --strict"),
        "Workflow deve executar cccp check"
    );

    // Exige job de deploy de documentação
    assert!(
        content.contains("deploy-site"),
        "Workflow deve conter job deploy-site"
    );
}

#[test]
fn test_mutation_ci_workflow_branches_and_secrets() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let workflow_path = workspace_root
        .join(".forgejo")
        .join("workflows")
        .join("ci.yaml");

    let content = fs::read_to_string(&workflow_path).unwrap();

    // Valida gatilhos de branches
    assert!(content.contains("\"main\""));
    assert!(content.contains("\"feat/*\""));
    assert!(content.contains("\"fix/*\""));

    // Valida integridade do clone para CCCP
    assert!(content.contains("fetch-depth: 0"));

    // Valida segredos do Cloudflare para publicação
    assert!(content.contains("secrets.CLOUDFLARE_API_TOKEN"));
    assert!(content.contains("secrets.CLOUDFLARE_ACCOUNT_ID"));
    assert!(content.contains("npx wrangler deploy"));
}
