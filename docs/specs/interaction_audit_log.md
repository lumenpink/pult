# Especificação Técnica: Trilha de Auditoria e Log de Interações (Ticket #20)

- **Ticket:** #20
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
Para permitir rastreabilidade total, depuração e auditoria de cada ciclo de interação entre o PULT, o Scout local e o Agente Externo (agy), é introduzida a opção de linha de comando `-l, --log [ARQUIVO]` acompanhada por um subsistema de telemetria e auditoria contínua em disco.

## 2. Interface de Linha de Comando (`Cli`)
- Nova opção:
  ```rust
  /// Gravar trilha completa de auditoria de interações em arquivo de log
  #[arg(short = 'l', long = "log", num_args = 0..=1, default_missing_value = ".antigravity/pult/logs/interacoes.log")]
  pub log: Option<String>,
  ```
- Se passada como `-l` ou `--log` sem valor, assume o caminho relativo `.antigravity/pult/logs/interacoes.log` dentro do repositório alvo.
- Se omitida, a gravação de auditoria em arquivo permanece inativa (`None`).

## 3. Gestor de Auditoria (`AuditLogger`)
- Módulo `pult_core::audit` (ou driver de log):
  - Estrutura `AuditLogger`:
    - Inicializada com o caminho de destino (`PathBuf`).
    - Garante criação dos diretórios pais (`std::fs::create_dir_all`).
    - Escreve eventos formatados em modo append com flush imediato (`OpenOptions::new().create(true).append(true)`).
  - Categorias de Eventos Registrados:
    - `[USER_INPUT]` : Prompt inicial e contexto da sessão.
    - `[AST_EXTRACTION]` : Quantidade de arquivos analisados e resumo de assinaturas.
    - `[SCOUT_REQUEST]` : Prompt e contexto estrutural submetido ao modelo local Ollama.
    - `[SCOUT_RESPONSE]` : Resposta bruta do Scout, modelo utilizado e símbolos selecionados.
    - `[AGY_DISPATCH]` : Comando disparado, argumentos e prompt final expandido.
    - `[AGY_STREAM]` : Chunks parciais emitidos pelo processo externo.
    - `[AGY_RESULT]` : Código de saída, truncamento ou erro do agente.
    - `[QUALITY_GATE]` : Resultado e índice de conformidade do `cccp check --strict`.

## 4. Integração no Pipeline
- O orquestrador assíncrono em `main.rs` recebe opcionalmente uma instância de `Arc<AuditLogger>`.
- A cada estágio do pipeline, o evento correspondente é registrado de forma não-bloqueante no log de auditoria.
