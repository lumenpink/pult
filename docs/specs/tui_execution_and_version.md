# Especificação: Versão Canônica Curta e Pipeline Assíncrono da TUI (Ticket #14)

- **Ticket:** #14
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
Garantir que a estação de trabalho PULT forneça transparência imediata sobre seu estado, exibindo a versão curta canônica no cabeçalho e executando o funil de orquestração de quatro etapas (AST -> Scout -> Persistência de Turno -> Agente Antigravity em Streaming) ao receber instruções na TUI, com telemetria visual em tempo real.

## 2. Exibição da Versão Canônica
- O arquivo `VERSION` raiz do repositório (mantido pelo `cccp`) é a autoridade canônica (ex.: `0.1.0-dev.87+20261026.9b40793`).
- `pult_core::canonical_version()` retorna a versão completa semver.
- `pult_core::short_version()` corta a versão no caractere `+` (ex.: `0.1.0-dev.87`).
- O cabeçalho da TUI renderiza: `☭ PULT v0.1.0-dev.87`.

## 3. Estado de Atividade e Feedback na TUI
- O `App` mantém:
  - `pub is_busy: bool`: indica se há pipeline assíncrono em execução.
  - `pub spinner_idx: usize`: índice do glifo do spinner animado (`⠋`, `⠙`, `⠹`, `⠸`, `⠼`, `⠴`, `⠦`, `⠧`, `⠇`, `⠏`).
  - `pub busy_label: String`: rótulo descritivo da etapa em execução.
- Quando `is_busy` é verdadeiro:
  - O cabeçalho substitui a fase estática ou exibe o spinner animado ao lado do status.
  - A submissão de novas instruções é bloqueada para evitar corridas concorrentes de turnos.

## 4. Pipeline Assíncrono de Orquestração (Funil de Contexto)
Ao acionar o envio de uma instrução:

1. **Submissão & Eco:**
   - Adiciona `> <prompt>` ao viewport.
   - Define `is_busy = true`.
   - Adiciona `☭ [PULT] [1/4] Extraindo assinaturas de AST da base de código...`

2. **Extração de AST (`PolyglotAstExtractor`):**
   - Varre `app.repo_path` ignorando `.git`, `target`, `node_modules`, `dist`.
   - Adiciona `✓ [1/4] N arquivos de AST analisados.`

3. **Filtragem Cirúrgica Scout (`ScoutEngine`):**
   - Adiciona `☭ [PULT] [2/4] Acionando Scout local (Ollama) para seleção cirúrgica de contexto...`
   - Invoca `scout.filter_with_bypass(prompt, &ast_files)`.
   - Em caso de sucesso ou bypass, relata os símbolos injetados ou status no viewport.

4. **Persistência de Sessão (`FileSessionStorage`):**
   - Adiciona `☭ [PULT] [3/4] Inicializando turno e persistência em disco...`
   - Registra o turno da usuária e prepara o turno do agente na sessão ativa.

5. **Disparo do Agente Soberano (`AgyRunner`):**
   - Adiciona `☭ [PULT] [4/4] Executando agente soberano (agy)...`
   - O `AgyRunner` roda assincronamente transmitindo eventos por canal `mpsc::Receiver<AgyStreamEvent>`.
   - O loop principal da TUI recebe os eventos e anexa os blocos de texto ao viewport em tempo real.
   - Ao receber `AgyStreamEvent::Completed`, relata o código de término, atualiza o turno na sessão e redefine `is_busy = false`.
