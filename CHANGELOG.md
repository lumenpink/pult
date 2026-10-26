# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Inicialização do repositório em conformidade com o padrão GOST (#1).
- `crates/pult-core`: Entidades centrais de domínio (`GostPhase`, `Session`, `Turn`, `Actor`, `TurnStatus`) (#2).
- `crates/pult-core`: Parser e serializador canônico de sessões em Markdown com Frontmatter YAML e turnos delimitados por comentários HTML (#2).
- `crates/pult-core`: Repositório de persistência atômica em disco `FileSessionStorage` com suporte a sessões rascunho (`tXXXXX-...`) e formais (`000042-...`) (#2).
- `crates/pult-core`: Driver soberano CCCP e Git com guardas de fase GOST, formatação canônica de commits (<= 72 chars) e injeção da fase ativa (#3).
- `crates/pult-core`: Extrator estrutural de AST de Rust via `syn` e `quote` omitindo corpos de funções funcionais e filtrando itens privados (#4).
- CI: Configuração canônica do workflow Forgejo com runner `ubuntu-latest` e container soberano `gost-ci-cross:latest` (#11).
- `crates/pult-core`: Extrator de AST poliglota com Tree-Sitter (TypeScript, JavaScript, Python, PHP, Bash) e despacho para Rust (#5).
- `crates/pult-core`: Cliente assíncrono para Ollama local e motor Scout para filtragem cirúrgica de AST com telemetria de VRAM e suporte a bypass (#6).
- `crates/pult-core`: Orquestrador assíncrono para a CLI Antigravity com streaming via canais MPSC, persistência incremental e detecção de truncamento (#7).
