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
- `crates/pult-tui`: Interface TUI desacoplada com split viewport superior, editor multilinha inferior com tui-textarea e alternância de foco (#8).
- `crates/pult-tui`: Máquina de estados GOST DD-TDD interativa, atalhos contextuais (Ctrl+P, Ctrl+N, Ctrl+G, Ctrl+M), guarda estrita da Fase 3 e compilação de crônica de fechamento (#9).
- Suíte completa de testes de integração de ponta a ponta (E2E), teste de estresse do viewport com 5.000 linhas, clippy com zero avisos e auditoria estrita de qualidade (#10).
- Portal oficial pult.2lp.in com identidade anti-design/construtivista, instalador curl portátil e esteira multi-arquitetura com publicação na Banca e Cloudflare R2 (#12).
- `crates/pult-core` & `crates/pult-tui`: Resolução flexível e ergonômica de repositório local compatível com o padrão CCCP Divan, aceitando caminhos absolutos, relativos, expansão de home (`~/...`), `owner/repo`, busca em `~/src/` e catálogos, suportando flags `-r/--repo`, aliases `-C/--dir` e argumento posicional único (closes #13).
- `crates/pult-core` & `crates/pult-tui`: Versão canônica curta (cortada em '+'), indicador visual de atividade e spinner animado no cabeçalho/editor, e pipeline assíncrono de quatro etapas (AST poliglota, filtragem Scout Ollama, persistência de sessão e streaming do AgyRunner para o viewport) (closes #14).
