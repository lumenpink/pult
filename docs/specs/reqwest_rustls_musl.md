# Especificação Técnica: Migração do Reqwest para Rustls e Habilitação de Compilação Estática MUSL (Ticket #34)

- **Ticket:** #34
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Concluído

## 1. Escopo e Motivação
A esteira de publicação do PULT no Forgejo Actions executa a compilação multiarquitetura estática (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` e `x86_64-pc-windows-gnu`) no job `Build & Publish Pult Multiarch to Banca`.
Ao compilar para o alvo MUSL, o crate `openssl-sys` (dependência transitiva de `reqwest -> native-tls`) falha por não encontrar cabeçalhos ou bibliotecas do OpenSSL compiladas para MUSL no sysroot.

Conforme o padrão do ecossistema Baquara (ex.: Ticket #50 do `joaju`), este problema é eliminado substituindo o backend TLS nativo em C pelo backend em Rust puro (`rustls-tls`), sem `default-features`.

## 2. Requisitos de Implementação
1. Em `crates/pult-core/Cargo.toml`:
   - Configurar `reqwest`:
     ```toml
     reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }
     ```
2. Invariante arquitetural:
   - A árvore de dependências (`cargo tree -i openssl-sys` e `cargo tree -i native-tls`) deve ser rigorosamente vazia.
3. Compatibilidade:
   - O motor `ScoutEngine` e o cliente `OllamaClient` continuam operando sem alterações na API ou no comportamento assíncrono.
4. Validação:
   - Teste automatizado `test_no_openssl_sys_dependency_in_pult_core` incluído em `ci_workflow_test.rs`.
   - Compilação cruzada validada no container canônico `gambi.2lp.in/lumen/gost-ci-cross:latest` tanto para `x86_64-unknown-linux-musl` quanto para `aarch64-unknown-linux-musl`.
   - Quality gates e todos os testes de unidade/integração aprovados.
