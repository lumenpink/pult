# Especificação Técnica: Spinner Exclusivo no Editor com Cronômetro e Telemetria de Duração nos Turnos e Logs

## 1. Contexto & Motivação
Na interface TUI, o indicador animado de progresso (spinner) e o rótulo de processamento ocupavam tanto o cabeçalho superior quanto a borda do editor multilinha. Isso sobrepunha a visualização da fase GOST ativa (`[FASE N: Nome]`), dificultando a percepção do estado da governança enquanto uma ação assíncrona era executada. Além disso, a ausência de um retorno temporal impedia que a usuária soubesse há quantos segundos o modelo ou subagente estava processando.

Por fim, no núcleo de dados (`Session` e `AuditLogger`), o registro dos turnos não mensurava a latência real de processamento de cada ação (tempo decorrido estritamente entre o envio da solicitação e o recebimento da resposta completa), gerando lacunas na telemetria de desempenho do Scout e da CLI do agente externo (`agy`).

## 2. Requisitos Técnicos

### 2.1 Interface: Desacoplamento do Cabeçalho e Spinner no Editor
- **Cabeçalho Superior (`crates/pult-tui/src/ui.rs`):**
  - Manter sempre o indicador da fase GOST corrente (`[FASE N: Nome]`) visível e desobstruído.
  - Eliminar qualquer sobreposição do spinner ou busy label no cabeçalho.
- **Borda do Editor:**
  - O spinner animado (`⠋`, `⠙`, `⠹`, ...) deve aparecer exclusivamente no título do bloco do editor multilinha quando `app.is_busy` for verdadeiro.
  - Incorporar contador dinâmico de tempo decorrido no formato `(Xs)` para durações inferiores a um minuto e `(Xm Ys)` para durações superiores:
    - Exemplo: `[Editor (⠹ Agente (agy)... (14s))]`
  - Em repouso (`!app.is_busy`), exibe o título canônico `[Editor Multilinha]`.

### 2.2 Controle de Temporização na TUI (`App`)
- Adicionar ao struct `App`:
  - `pub busy_start_time: Option<std::time::Instant>`
- Ao invocar `set_busy(true, label)`:
  - Inicializa `busy_start_time = Some(Instant::now())`.
- Ao invocar `set_busy(false, "")`:
  - Limpa `busy_start_time = None`.
- Fornecer método auxiliar `busy_elapsed_formatted(&self) -> String`.

### 2.3 Núcleo: Telemetria de Duração em Turnos (`Turn.elapsed_ms`)
- **Entidade `Turn` (`crates/pult-core/src/domain/session.rs`):**
  - Adição do campo opcional:
    ```rust
    pub elapsed_ms: Option<u64>,
    ```
- **Serialização em Comentário Markdown:**
  - Formatar a tag `<!-- TURN: ... -->` incluindo o campo `ELAPSED`:
    - `<!-- TURN: 2 | ACTOR: agy | TIME: ... | ELAPSED: 4.23s -->` (quando `elapsed_ms` estiver presente).
- **Desserialização (`parse_markdown_turns`):**
  - Extrair o valor de `ELAPSED` (em segundos ou milissegundos) e restaurar para `elapsed_ms`.
- **Métodos Auxiliares em `Session`:**
  - `pub fn set_last_turn_elapsed(&mut self, ms: u64)`: define o tempo de resposta no último turno registrado.

### 2.4 Trilha de Auditoria (`AuditLogger`)
- No arquivo de log `.antigravity/pult/logs/<session>.log`:
  - Registrar explicitamente a duração de cada ação ao finalizar:
    - `[TIME] [SCOUT_COMPLETED] (0.84s) ...`
    - `[TIME] [AGY_COMPLETED] (8.45s) Concluído com código de saída 0`
    - `[TIME] [PIPELINE_TOTAL] (9.32s) Ação finalizada`

### 2.5 Invariantes
- Cabeçalhos de commits $\le 72$ caracteres com sufixo canônico `ph<N>`.
- Vocabulário autônomo, horizontal e cooperativo.
