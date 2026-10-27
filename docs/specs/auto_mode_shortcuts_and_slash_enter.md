# Especificação Técnica: Atalhos de Modo Auto, Execução com Enter para Slash Commands e Autocomplete

## Contexto e Motivação
A experiência interativa da TUI do PULT necessita de fluidez e ergonomia ágil no ciclo de desenvolvimento:
1. Atualmente, o atalho de aprovação `Ctrl+P` está restrito à Fase 3 em `crates/pult-tui/src/event.rs`, impedindo o avanço natural com uma tecla única ao longo das 9 fases.
2. Comandos que começam com barra (`/...`) exigem a combinação `Ctrl+S` para envio, quando um simples toque em `Enter` deveria submeter o comando diretamente.
3. O autocompletar via tecla `Tab` recriava o `TextArea` na posição de cursor (0, 0), fazendo com que a continuação da digitação ocorresse no início do comando.
4. É necessário permitir alternar rapidamente entre o modo de esteira automática `[AUTO]` e o modo `[MANUAL]`, via tecla de atalho não conflitante com multiplexadores (`Ctrl+X`) e comando `/auto`.

## Requisitos Funcionais

### RF-01: Universalização do Atalho Ctrl+P
- Em `crates/pult-tui/src/event.rs`, ao receber `Ctrl+P`, emitir imediatamente `TuiAction::SpecApproved` em qualquer fase do GOST, permitindo a aprovação contínua equivalente a `/approve`.

### RF-02: Alternância de Modo Auto (Ctrl+X e /auto)
- Adicionar ao `App` o campo `is_auto_mode: bool` (padrão `false`) e o método `toggle_auto_mode() -> bool`.
- Ao receber `Ctrl+X` ou o comando `/auto`, alternar o estado do modo auto e registrar a mudança no viewport.
- Atualizar a barra inferior (`render_footer` em `ui.rs`) para exibir de forma concisa `[AUTO]` em destaque verde/ciano quando ativo ou `[MANUAL]` em cinza quando inativo.

### RF-03: Submissão Direta com Enter para Slash Commands
- Quando o foco estiver no editor (`Focus::Editor`) e a tecla `Enter` for pressionada:
  - Se o conteúdo do editor for de uma linha única iniciando com `/` e corresponder a um comando slash reconhecido, submeter e executar o comando imediatamente (`execute_slash_command`), retornando a respectiva `TuiAction`.
  - Caso contrário (texto comum ou multilinha), manter a inserção normal de nova linha no editor.

### RF-04: Autocomplete com Tab Corrigido
- Ao teclar `Tab` em linha iniciando com `/`:
  - Completar o comando slash utilizando `autocomplete_slash_command` (incluindo `/auto` na lista de comandos conhecidos).
  - Posicionar o cursor obrigatoriamente no final da linha (`CursorMove::End`) para possibilitar envio imediato ou complementação de argumentos (como `/branch <nome>`).

## Critérios de Aceite
1. Teste comprovando que `Ctrl+P` emite `TuiAction::SpecApproved` em qualquer fase (Fases 1, 3, 4, 5, 9).
2. Teste comprovando que `Ctrl+X` e `/auto` alternam `is_auto_mode` e emitem `TuiAction::AutoModeToggled`.
3. Teste comprovando que pressionar `Enter` com `/approve`, `/auto`, `/help` executa o comando diretamente sem `Ctrl+S`.
4. Teste comprovando que `Tab` completa `/au` para `/auto` e deixa o cursor no final da linha.
5. Conformidade total de qualidade Gosplan (`cargo fmt`, `cargo clippy`, `cccp check --strict`).
