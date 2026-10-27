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
- `crates/pult-core` & `crates/pult-tui`: Acumulação de stream de saída por chunks preservando quebras de linha fluídas sem fatiar palavras e expurgo integral de terminologia soberana em favor de vocabulário autônomo, horizontal e cooperativo (closes #15).
- `crates/pult-core` & `crates/pult-tui`: Persistência contínua multi-turnos ao longo da conversa com streaming incremental em disco e resiliência a interrupções (closes #16).
- `crates/pult-tui`: Detecção dinâmica de branch via Git, inferência determinística de ticket_id e suporte à retomada de sessões via flag `--resume`/`-s` (closes #17).
- `crates/pult-tui` & `crates/pult-core`: Conexão dos atalhos contextuais GOST (Ctrl+P, Ctrl+N, Ctrl+G, Ctrl+M) no loop de eventos, validação de veto da Fase 3 e fechamento automático de tickets (closes #18).
- `crates/pult-tui`: Rodapé minimalista com atalho único Ctrl+H, modal flutuante de ajuda central de comandos, interpretador de comandos com barra (/new, /approve, /next, etc.) e autocomplete inteligente (closes #19).
- `crates/pult-core` & `crates/pult-tui`: Flag `-l/--log` e subsistema thread-safe de log de auditoria com gravação de payloads de AST, Scout, Agente Externo e saídas (closes #20).
- `crates/pult-core` & `crates/pult-tui`: Síntese semântica de slug de sessão pelo ScoutEngine com limite estrito de 60 caracteres em kebab-case e fallback determinístico (closes #21).
- `crates/pult-core` & `crates/pult-tui`: Orquestração do ciclo DD-TDD pela PULT, proteção estrita contra operações na branch main/master, injeção de diretivas especializadas de fase com autorização de arquivos e proibição expressa de comandos git/cccp para o Agente Externo, auditoria contínua via cccp check e comando interativo /branch (closes #22).
- `crates/pult-core` & `crates/pult-tui`: Sincronização 1:1 de sessão e log com retargeting atômico na formalização de tickets e gravação integral de payloads de Scout e agy (closes #23 - ph9).
- `crates/pult-core` & `crates/pult-tui`: Comando /approve universal operando em todas as fases da TUI com avanço automático, automação de commits normativos GOST 2.0 com marcadores phX e trava rígida Gosplan em auditorias (closes #24 - ph9).
