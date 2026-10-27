# Especificação Técnica: Diretivas Pontuais sem Menção a GOST e Automação da Fase 2 com Scout

## 1. Contexto & Motivação
Na interação com a CLI externa (`agy`), observou-se que o agente herdava as diretrizes globais do usuário (`~/.gemini/GEMINI.md`), presumindo erroneamente ser o orquestrador do ciclo DD-TDD de 9 fases e produzindo respostas prolixas com roadmaps futuros. Ademais, cada turno despachado para a `agy` sofria de amnésia contextual por não receber o histórico das interações recentes da sessão. Por fim, a criação do ticket na forja e checkout da branch na Fase 2 demandavam intervenção manual externa à TUI.

## 2. Requisitos Técnicos

### 2.1 Preâmbulo de Isolamento de Escopo e Cancelamento de Regras Globais
- Todo prompt despachado ao `AgyRunner` deve conter no topo o cabeçalho imperativo:
  ```
  [DIRETIVA DE OPERAÇÃO - SUBAGENTE DE EXECUÇÃO PULT]
  Anulação de escopo: Ignore quaisquer regras ou diretrizes herdadas sobre orquestração do ciclo GOST de 9 fases. Você NÃO é o maestro do ciclo; o PULT é o orquestrador absoluto do processo.
  Seu escopo de trabalho está ESTRITAMENTE restrito à fase atual (<FASE_NOME>). Não teorize, não mencione fases futuras e não execute ações fora do escopo desta fase.
  ```
- O método `agent_directive` em `crates/pult-core/src/domain/gost.rs` deve ser depurado para expurgar referências a fases futuras, mantendo apenas as restrições estritas de arquivos da fase corrente (ex.: Fase 3 só doc/specs; Fase 4 só tests; Fase 5 código + testes).

### 2.2 Resolução da Amnésia Multi-Turnos (Injeção de Histórico)
- Na preparação do prompt em `crates/pult-tui/src/main.rs`:
  - Se a sessão ativa contiver turnos anteriores (`session.turns`), formatar os últimos turnos como bloco de contexto histórico:
    ```
    --- Contexto do Histórico da Sessão ---
    [Usuária]: ...
    [Agente]: ...
    --------------------------------------
    ```
  - Permitir que respostas concisas da usuária (como `y`, `prossiga`, `refine o caso X`) sejam contextualizadas integralmente pelo modelo.

### 2.3 Automação da Fase 2 com Scout e Criação de Branch
- Ao aprovar a Fase 1 (Ticket) via `/approve` ou comando de transição:
  - O PULT aciona o `ScoutEngine` para sintetizar o título do ticket (padrão Conventional Commits, $\le 72$ chars) e o corpo nos moldes da GOST a partir dos turnos discutidos.
  - O PULT executa a criação do ticket na forja via `cccp ticket new`.
  - Com o ID numérico e o slug retornados, o driver cria a branch `feat/<id>-<slug>` e realiza o checkout atômico.
  - A sessão ativa é promovida de rascunho (`tXXXXX-...`) para formal (`0000<id>-<slug>.md`), sincronizando também o arquivo de log.
  - O estado da máquina GOST transiciona automaticamente para a Fase 3 (SpecContract).

### 2.4 Destaque Cromático da Branch Ativa na TUI
- No cabeçalho da interface TUI (`crates/pult-tui/src/ui.rs`), renderizar o indicador `🌿 <branch_name>` em destaque visual contrastante para orientar continuamente a usuária sobre o branch git ativo.
