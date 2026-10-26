# Especificação Técnica: Antigravity CLI Runner & Concurrent Streaming Persistence

## 1. Visão Geral e Propósito
O `AgyRunner` é o orquestrador assíncrono responsável por invocar o processo da CLI do agente Antigravity (`agy`), capturar seus fluxos de saída (`stdout` e `stderr`) em tempo real sem travar a interface gráfica do terminal (TUI), transmitir os eventos por canais concorrentes e persistir incrementalmente cada turno no arquivo de sessão em disco.

## 2. Invocação de Subprocesso e Streaming Concorrente
- **Binário e Configuração:** O runner permite configurar o executável (`agy` por padrão, ou qualquer binário/script customizado via `RunnerOptions`).
- **Pipes Assíncronos:** Utiliza `tokio::process::Command` com `Stdio::piped()` para `stdout` e `stderr`.
- **Canais Concorrentes:** As tarefas assíncronas leem as saídas em blocos e enviam eventos através de um `tokio::sync::mpsc::Sender<AgyStreamEvent>`:
  - `StdoutChunk(String)`: Trecho incremental de texto gerado pelo agente.
  - `StderrChunk(String)`: Mensagens de diagnóstico, avisos ou logs de ferramentas.
  - `Truncated`: Indicador emitido quando há detecção de truncamento por estouro de cota de tokens.
  - `Completed { exit_code: i32 }`: Conclusão do processo filho.
  - `Error(String)`: Falha catastrófica de execução ou erro de I/O.

## 3. Persistência Incremental em Tempo Real
Durante a execução de um turno do agente (`Actor::Agy`):
1. **Início do Turno:** Um novo turno é inserido na `Session` com `TurnStatus::InProgress`. O arquivo `.md` é persistido em disco via `FileSessionStorage`.
2. **Streaming em Tempo Real:** Conforme chunks chegam pelo canal MPSC, o conteúdo do turno é acumulado no buffer da sessão em memória e gravado periodicamente em disco.
3. **Conclusão:**
   - Se o processo sair com sucesso e sem corte: `TurnStatus::Completed`.
   - Se a saída contiver o padrão de truncamento `STATUS: truncated` ou término abrupto: `TurnStatus::Truncated`.
   - Se o processo sair com erro: `TurnStatus::Failed`.
4. A persistência em disco assegura que se o terminal for fechado acidentalmente ou sofrer pane, nenhum byte gerado pela IA seja perdido.

## 4. Retomada de Sessão e Reconciliação com o Git (`resume`)
- O runner fornece o método `resume_session(path: &Path, driver: &dyn RepoDriver)` para inspecionar sessões salvas em `.antigravity/pult/`.
- Localiza o último turno registrado:
  - Se estiver com status `TurnStatus::InProgress` ou `TurnStatus::Truncated`, permite disparar `/continue` retomando a geração exatamente a partir do último estado íntegro.
- Cruza a branch gravada no frontmatter da sessão com a branch ativa do repositório Git, alertando a usuária sobre divergências.
