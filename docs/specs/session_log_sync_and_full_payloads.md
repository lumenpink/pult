# Especificação: Sincronização 1:1 Sessão-Log e Payloads Integrais

- **Ticket Relacionado**: #23
- **Fase DD-TDD**: Fase 3 (Spec / Contrato)
- **Status**: Aprovado para Implementação

## 1. Visão Geral e Motivação

A auditoria e rastreabilidade na PULT dependem da correlação estrita entre a crônica de turnos em Markdown (`.antigravity/pult/sessions/<session_id>.md`) e a trilha cronológica detalhada de eventos de telemetria (`.antigravity/pult/logs/<session_id>.log`).

Anteriormente:
1. O arquivo de log usava nomes genéricos ou estáticos, dificultando associar qual log pertencia a qual sessão.
2. Quando uma sessão passava de rascunho (`tXXXXX-...`) para formal (`000042-...`), o log não era renomeado em conjunto.
3. As requisições e respostas do motor Scout e do Agente Externo (`agy`) omitiam os textos completos, registrando apenas o tamanho em bytes ou arrays de símbolos selecionados.

Esta especificação formaliza o pareamento 1:1, a renomeação atômica síncrona e o registro integral e sem cortes de todos os payloads de LLM.

---

## 2. Contratos e Tipos

### 2.1 `AuditLogger::retarget` e Atualização de Arquivo

Em `crates/pult-core/src/audit.rs`:
```rust
impl AuditLogger {
    /// Atualiza o caminho do arquivo de log em tempo de execução e garante a criação do diretório pai.
    pub fn retarget(&self, new_path: PathBuf);

    /// Retorna o caminho atual do arquivo de log ativo.
    pub fn file_path(&self) -> PathBuf;
}
```

### 2.2 Exposição dos Payloads no `ScoutResult`

Em `crates/pult-core/src/scout/models.rs`:
```rust
pub struct ScoutResult {
    pub symbols: Vec<String>,
    pub yaml_block: String,
    pub model_used: String,
    pub vram_allocated_bytes: Option<u64>,
    pub bypassed: bool,
    pub full_prompt: String,
    pub raw_response: String,
}
```

### 2.3 Eventos Canônicos de Auditoria

No log de auditoria `.antigravity/pult/logs/<session_id>.log`, os eventos registram textualmente:
- `SCOUT_FULL_PROMPT`: Todo o texto exato do prompt submetido ao Ollama.
- `SCOUT_RAW_RESPONSE`: A resposta textual bruta devolvida pelo modelo do Scout.
- `AGY_FULL_PROMPT`: O texto completo enviado à CLI do `agy` (diretiva + histórico + contexto + prompt).
- `AGY_FULL_RESPONSE`: O texto consolidado emitido pelo Agente Externo.

---

## 3. Critérios de Aceite

1. O arquivo de log adota por padrão o nome `.antigravity/pult/logs/<session_id>.log`.
2. Ao promover uma sessão rascunho para formal com ticket, o arquivo de log existente é renomeado para corresponder ao novo ID.
3. O `ScoutResult` armazena e expõe `full_prompt` e `raw_response`.
4. Os payloads integrais são gravados nos eventos `SCOUT_FULL_PROMPT`, `SCOUT_RAW_RESPONSE`, `AGY_FULL_PROMPT` e `AGY_FULL_RESPONSE`.
