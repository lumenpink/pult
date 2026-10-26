use pult_core::ast::{AstExtractor, PolyglotAstExtractor};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_polyglot_typescript_extraction() {
    let extractor = PolyglotAstExtractor::new();
    let ts_code = r#"
export interface User {
    id: string;
    email: string;
}

export type UserRole = "admin" | "editor" | "viewer";

export class UserService {
    private secretKey: string;

    constructor(secret: string) {
        this.secretKey = secret;
        console.log("initialized");
    }

    public async findUser(id: string): Promise<User | null> {
        const query = `SELECT * FROM users WHERE id = ${id}`;
        console.log(query);
        return { id, email: "user@example.com" };
    }
}
"#;

    let path = Path::new("src/user_service.ts");
    let result = extractor
        .extract_from_str(path, ts_code)
        .expect("deve extrair typescript com sucesso");

    assert!(result.contains("interface User"));
    assert!(result.contains("type UserRole"));
    assert!(result.contains("class UserService"));
    assert!(result.contains("findUser"));
    // O corpo funcional com sql e console.log deve ser omitido
    assert!(!result.contains("SELECT * FROM users"));
    assert!(!result.contains("console.log"));
    assert!(result.contains("{ ... }"));
}

#[test]
fn test_polyglot_python_extraction() {
    let extractor = PolyglotAstExtractor::new();
    let py_code = r#"
class PaymentProcessor:
    """Processador de pagamentos do sistema."""
    gateway_url: str

    def __init__(self, url: str):
        self.gateway_url = url
        self._cache = {}

    def process_charge(self, amount_cents: int, currency: str = "USD") -> bool:
        payload = {"amount": amount_cents, "curr": currency}
        print("Enviando cobranca para gateway...")
        return True

def standalone_helper(token: str) -> str:
    temp = token.strip()
    return temp.upper()
"#;

    let path = Path::new("services/payment.py");
    let result = extractor
        .extract_from_str(path, py_code)
        .expect("deve extrair python com sucesso");

    assert!(result.contains("class PaymentProcessor"));
    assert!(result.contains("def __init__"));
    assert!(result.contains("def process_charge"));
    assert!(result.contains("def standalone_helper"));
    // Corpos internos devem ser omitidos
    assert!(!result.contains("Enviando cobranca"));
    assert!(!result.contains("payload ="));
    assert!(!result.contains("temp.upper()"));
}

#[test]
fn test_polyglot_php_extraction() {
    let extractor = PolyglotAstExtractor::new();
    let php_code = r#"<?php

namespace App\Services;

interface NotifierInterface {
    public function send(string $recipient, string $message): bool;
}

class EmailNotifier implements NotifierInterface {
    private string $smtpHost;

    public function __construct(string $smtpHost) {
        $this->smtpHost = $smtpHost;
        $this->connect();
    }

    public function send(string $recipient, string $message): bool {
        $data = ["to" => $recipient, "body" => $message];
        mail($recipient, "Notificacao", $message);
        return true;
    }
}
"#;

    let path = Path::new("src/EmailNotifier.php");
    let result = extractor
        .extract_from_str(path, php_code)
        .expect("deve extrair php com sucesso");

    assert!(result.contains("interface NotifierInterface"));
    assert!(result.contains("class EmailNotifier"));
    assert!(result.contains("function send"));
    // O corpo interno de envio de email deve ser omitido
    assert!(!result.contains("mail($recipient"));
    assert!(!result.contains("$data ="));
}

#[test]
fn test_polyglot_bash_extraction() {
    let extractor = PolyglotAstExtractor::new();
    let sh_code = r#"#!/usr/bin/env bash

set -euo pipefail

deploy_service() {
    echo "Iniciando deploy..."
    docker-compose pull
    docker-compose up -d
}

function verify_health() {
    curl -sf http://localhost:8080/health || exit 1
}

deploy_service
"#;

    let path = Path::new("scripts/deploy.sh");
    let result = extractor
        .extract_from_str(path, sh_code)
        .expect("deve extrair bash com sucesso");

    assert!(result.contains("deploy_service()"));
    assert!(result.contains("verify_health"));
    // Comandos internos devem ser omitidos
    assert!(!result.contains("docker-compose pull"));
    assert!(!result.contains("curl -sf http"));
}

#[test]
fn test_polyglot_rust_dispatch() {
    let extractor = PolyglotAstExtractor::new();
    let rs_code = r#"
pub struct EngineConfig {
    pub port: u16,
    secret_token: String,
}

impl EngineConfig {
    pub fn new(port: u16) -> Self {
        let token = format!("tok-{}", port);
        Self { port, secret_token: token }
    }
}
"#;

    let path = Path::new("src/config.rs");
    let result = extractor
        .extract_from_str(path, rs_code)
        .expect("deve despachar para rust syn");

    assert!(result.contains("pub struct EngineConfig"));
    assert!(result.contains("pub port : u16"));
    assert!(!result.contains("secret_token"));
    assert!(result.contains("pub fn new"));
    assert!(!result.contains("format!(\"tok-"));
    assert!(result.contains(";"));
}

#[test]
fn test_polyglot_syntax_error_tolerance() {
    let extractor = PolyglotAstExtractor::new();
    // Código TypeScript com erro de sintaxe no meio
    let incomplete_ts = r#"
export interface ValidInterface {
    name: string;
}

export function brokenFunc(a: number, {
    this is completely broken syntax !!!
}

export function followingValidFunc(): void {
    const x = 10;
}
"#;

    let path = Path::new("src/broken.ts");
    let result = extractor
        .extract_from_str(path, incomplete_ts)
        .expect("deve tolerar sintaxe quebrada sem panic");

    assert!(result.contains("interface ValidInterface"));
    assert!(result.contains("followingValidFunc"));
}

#[test]
fn test_polyglot_directory_scan() {
    let temp_dir = tempdir().expect("tempdir");
    let base = temp_dir.path();

    fs::create_dir_all(base.join("src")).expect("mkdir src");
    fs::create_dir_all(base.join("node_modules/pkg")).expect("mkdir node_modules");
    fs::create_dir_all(base.join(".git/objects")).expect("mkdir .git");

    fs::write(
        base.join("src/lib.rs"),
        "pub fn core_run() { println!(\"run\"); }",
    )
    .expect("write rs");
    fs::write(
        base.join("src/app.py"),
        "def main():\n    print(\"starting app\")\n",
    )
    .expect("write py");
    fs::write(
        base.join("src/types.ts"),
        "export interface Config { port: number; }\n",
    )
    .expect("write ts");
    fs::write(
        base.join("node_modules/pkg/index.js"),
        "function ignored() { return true; }\n",
    )
    .expect("write ignored");

    let extractor = PolyglotAstExtractor::new();
    let extracted = extractor
        .extract_from_dir(base, &["node_modules"])
        .expect("extract dir");

    let relative_paths: Vec<_> = extracted.iter().map(|f| &f.relative_path).collect();
    assert!(relative_paths.iter().any(|p| p.ends_with("lib.rs")));
    assert!(relative_paths.iter().any(|p| p.ends_with("app.py")));
    assert!(relative_paths.iter().any(|p| p.ends_with("types.ts")));
    assert!(!relative_paths.iter().any(|p| p.contains("node_modules")));
}
