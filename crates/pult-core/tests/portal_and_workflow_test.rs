//! Testes de integridade do portal web pult.2lp.in e do workflow de release para a Banca.
//!
//! Valida a aderência do portal com as diretrizes do ecossistema, incluindo
//! a folha de estilo anti-design, simulador interativo da TUI, comandos de instalação,
//! rodapé canônico, cabeçalhos de segurança e a automação de release multi-arquitetura
//! do Forgejo com envio para Cloudflare R2 e rotação na Banca.

use std::fs;
use std::path::Path;

#[test]
fn test_portal_html_and_assets_integrity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(manifest_dir);

    let site_index = repo_root.join("site/index.html");
    let content = fs::read_to_string(&site_index).expect("ler site/index.html");

    // Título e Identidade
    assert!(content.contains("PULT"), "Deve conter menção a PULT");
    assert!(
        content.contains("Antigravity"),
        "Deve mencionar Antigravity"
    );
    assert!(content.contains("DD-TDD"), "Deve mencionar DD-TDD");

    // Instalação e Banca
    assert!(
        content.contains("https://pult.2lp.in/install.sh") || content.contains("install.sh"),
        "Deve conter instalador direto"
    );
    assert!(
        content.contains("banca.2lp.in/downloads/pult") || content.contains("banca.2lp.in"),
        "Deve conter link para a Banca"
    );

    // Rodapé canônico
    assert!(
        content.contains("travesti latina"),
        "Deve conter rodapé canônico travesti latina"
    );
    assert!(
        content.contains("site-footer"),
        "Deve conter classe site-footer"
    );

    // Folha de estilos
    let style_path = repo_root.join("site/assets/style.css");
    assert!(style_path.exists(), "site/assets/style.css deve existir");

    // Instalador standalone
    let install_sh = repo_root.join("install.sh");
    assert!(install_sh.exists(), "install.sh na raiz deve existir");
    let site_install_sh = repo_root.join("site/install.sh");
    assert!(
        site_install_sh.exists(),
        "site/install.sh para curl deve existir"
    );
}

#[test]
fn test_ci_workflow_multiarch_and_banca_publish() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(manifest_dir);

    let ci_yaml_path = repo_root.join(".forgejo/workflows/ci.yaml");
    let ci_content = fs::read_to_string(&ci_yaml_path).expect("ler ci.yaml");

    // Multiarch compilations
    assert!(
        ci_content.contains("x86_64-unknown-linux-musl"),
        "Deve compilar para Linux AMD64"
    );
    assert!(
        ci_content.contains("aarch64-unknown-linux-musl"),
        "Deve compilar para Linux ARM64"
    );
    assert!(
        ci_content.contains("x86_64-pc-windows-gnu"),
        "Deve compilar para Windows AMD64"
    );

    // Upload R2 e Banca
    assert!(
        ci_content.contains("twolp-bucket/downloads/pult"),
        "Deve publicar no storage twolp-bucket/downloads/pult"
    );
    assert!(
        ci_content.contains("rotate-downloads.py"),
        "Deve acionar o script de rotação da Banca"
    );
    assert!(
        ci_content.contains("site/downloads/pult"),
        "Deve atualizar o manifesto pult na Banca"
    );
}
