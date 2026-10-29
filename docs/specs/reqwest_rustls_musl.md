# Especificação Técnica: Migração do Reqwest para Rustls e Habilitação de Compilação Estática MUSL (Ticket #34)

- **Ticket:** #34
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

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
   - A árvore de dependências (`cargo tree -i openssl-sys`) deve ser rigorosamente vazia.
3. Compatibilidade:
   - O motor `ScoutEngine` e o cliente `OllamaClient` devem continuar operando sem alterações na API ou no comportamento assíncrono.
4. Validação:
   - A suíte completa de testes de unidade e integração deve compilar e passar.
   - O comando de compilação para o alvo `x86_64-unknown-linux-musl` deve ter êxito sem erros de vinculação ao OpenSSL.
