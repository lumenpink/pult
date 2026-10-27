# Especificação Técnica: Persistência Contínua de Sessão Multi-Turnos (Ticket #16)

- **Ticket:** #16
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
Na versão atual do PULT, cada envio de prompt recriava a instância de `Session` e sobrescrevia o arquivo em disco ou gerava novos rascunhos dispersos. Além disso, as respostas transmitidas via streaming pelo Agente Externo não eram persistidas incrementalmente no disco em tempo real, correndo risco de perda caso o processo fosse interrompido. Esta especificação define a persistência contínua multi-turnos de uma sessão única ao longo de todo o ciclo de vida da conversa até que a usuária decida explicitamente criar uma nova sessão.

## 2. Ciclo de Vida da Sessão Ativa
- O `App` ou controlador TUI mantém uma instância ativa `Option<Session>` associada a um `FileSessionStorage`.
- Ao inicializar o PULT:
  - Se `--resume <ARQUIVO>` for fornecido, a sessão é carregada do disco.
  - Caso contrário, uma sessão é criada lazily no primeiro envio ou na inicialização.
- Submissões subsequentes da usuária **não recriam** a sessão:
  - Adicionam um novo turno `Actor::User` com status `Completed` e o timestamp atual.
  - Salvam a sessão imediatamente em disco.
- Uma nova sessão só é criada quando explicitamente solicitada (ex.: comando `/new` ou `Ctrl+N`).

## 3. Gravação Incremental do Stream do Agente Externo
- Imediatamente antes de iniciar a execução do processo do Agente Externo (`AgyRunner`):
  - Um turno `Actor::Agy` é criado na sessão ativa com status `TurnStatus::InProgress` e conteúdo vazio.
  - A sessão é persistida em disco pelo `FileSessionStorage`.
- Conforme chunks/linhas são recebidos pelo canal assíncrono MPSC:
  - O conteúdo acumulado do turno `Actor::Agy` ativo é atualizado.
  - O arquivo de sessão em `.antigravity/sessions/` é sincronizado atomicamente no disco.
- Ao término do streaming:
  - `AgyRunnerEvent::Completed` $\to$ define status como `TurnStatus::Completed`.
  - `AgyRunnerEvent::Truncated` $\to$ define status como `TurnStatus::Truncated`.
  - `AgyRunnerEvent::Error` $\to$ define status como `TurnStatus::Error`.
  - A sessão finalizada é salva uma última vez com o status definitivo.

## 4. Garantia de Resiliência
- Se o terminal ou o processo for encerrado abruptamente durante o streaming do Agente Externo, a inspeção posterior do arquivo `.md` da sessão conterá todos os turnos anteriores intactos e o turno em andamento preservado com os dados recebidos até o momento da interrupção.
