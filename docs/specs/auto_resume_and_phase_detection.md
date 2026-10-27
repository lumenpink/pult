# Especificação Técnica: Localização Automática de Sessão e Inferência de Fase GOST por Commits

## 1. Contexto & Motivação
Ao iniciar o `pult` fora da branch `main` ou ao alternar entre branches de trabalho com `/branch <nome>`, a TUI necessitava de flags explícitas (ex.: `--resume <arquivo>`) para restaurar o histórico da conversa e frequentemente reiniciava a máquina de estados GOST na Fase 1 (`Ticket`).

Para oferecer ergonomia contínua e eliminar atritos manuais, o PULT deve inspecionar deterministicamente o ambiente Git no momento da inicialização ou alternância de branch:
1. Extrair o identificador numérico do ticket a partir do padrão de nomenclatura da branch.
2. Localizar e carregar automaticamente a sessão `.md` correspondente no diretório `.antigravity/pult/sessions/`.
3. Inspecionar o histórico de commits do branch para identificar em qual fase GOST o trabalho se encontra, atualizando a máquina de estados para a fase correta.

## 2. Requisitos Técnicos

### 2.1 Mapeamento e Carga de Sessão (`find_session_for_ticket`)
- Dado o diretório do repositório, o ID do ticket (`u32`) e o nome da branch (`&str`):
  1. Varre arquivos `.md` presentes em `.antigravity/pult/sessions/`.
  2. Verifica por correspondência no nome do arquivo:
     - Formato canônico formal: `0000<id>-*.md` (ex.: `000002-script.md`).
     - Formato curto ou numérico: `<id>-*.md` (ex.: `2-script.md`).
  3. Caso nenhum arquivo corresponda ao nome, realiza varredura no Frontmatter YAML dos arquivos `.md` existentes:
     - Se `metadata.ticket.id == Some(ticket_id)` ou `metadata.ticket.branch == branch`.
  4. Se encontrado, invoca `app.load_session_file(&path)` restaurando os turnos anteriores para a memória e o viewport.

### 2.2 Inferência de Fase GOST via Histórico de Commits (`detect_branch_phase`)
- Adição do método no trait `RepoDriver` e implementação em `CccpGitDriver`:
  ```rust
  fn detect_branch_phase(&self, branch: &str, ticket_id: Option<u32>) -> Result<GostPhase, DriverError>;
  ```
- Executa comando git para inspecionar os commits presentes em `branch` (prioritariamente `main..<branch>`, com fallback para commits locais caso `main` não seja ancestral direto):
  - Se contiver `(closes #<id> - ph9)` ou tag de fechamento de ticket: **Fase 9 (`CloseTicket`)**.
  - Se contiver commit de implementação verde `(ref #<id> - ph5)`:
    - Se a auditoria Gosplan (`check_strict`) passar com 0 falhas: **Fase 9 (`CloseTicket`)** (ou **Fase 6 (`Refactor`)** caso requeira revisão).
  - Se contiver commit de teste vermelho `(ref #<id> - ph4)`: **Fase 5 (`GreenCode`)**.
  - Se contiver commit de especificação preliminar `(ref #<id> - ph3)`: **Fase 4 (`RedTest`)**.
  - Se a branch for de trabalho (`feat/`, `fix/`, etc.) e não contiver nenhum commit de fase: **Fase 3 (`SpecContract`)** (branch já criada, pronta para contrato).
  - Se estiver na branch `main`/`master` ou sem identificador de ticket: **Fase 1 (`Ticket`)**.

### 2.3 Integração no Ciclo de Vida da TUI
- **Inicialização (`crates/pult-tui/src/main.rs`):**
  - Se nenhuma flag `--resume` for fornecida explicitamente:
    - Detecta branch ativa.
    - Se `branch != "main" && branch != "master"`:
      - Extrai `ticket_id`.
      - Procura e restaura a sessão `.md`.
      - Detecta a fase GOST via commits e atualiza `app.gost_phase`.
      - Registra mensagens de status no viewport da TUI.
- **Alternância de Branch (`TuiAction::BranchChanged` em `app.rs`):**
  - Ao executar `/branch <nome>`, realiza checkout, extrai `ticket_id`, restaura a sessão associada e infere a fase correta a partir dos commits.

### 2.4 Invariantes
- Cabeçalhos de commit $\le 72$ caracteres com sufixo canônico de fase `ph<N>`.
- Vocabulário autônomo e horizontal.
