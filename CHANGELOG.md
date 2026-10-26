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
