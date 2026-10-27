# Especificação Técnica: Conexão dos Atalhos Contextuais GOST e Guarda da Fase 3 (Ticket #18)

- **Ticket:** #18
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
A máquina de estados GOST e os atalhos operacionais (`Ctrl+P`, `Ctrl+N`, `Ctrl+G`, `Ctrl+M`) definidos na especificação `gost_state_guard.md` estavam desacoplados no loop principal da aplicação em `crates/pult-tui/src/main.rs`. Apenas o envio de texto (`TuiAction::Submit`) era tratado, ignorando as ações contextuais da metodologia GOST DD-TDD. Além disso, a validação de segurança da Fase 3 (que proíbe edição em `src/` antes dos testes Red) não era verificada antes da aprovação de contratos.

## 2. Ações Contextuais no Loop da TUI
O loop `run_loop` intercepta e executa os seguintes comportamentos para cada variante de `TuiAction`:

1. **`TuiAction::SpecApproved` (`Ctrl+P`)**:
   - Valida o status do repositório Git via `RepoDriver::status()`.
   - Se qualquer arquivo com prefixo `src/` (ou em qualquer crate sob `crates/*/src/`) estiver modificado ou adicionado (staged ou unstaged), o avanço é vetado com alerta explícito no viewport e a transição é bloqueada.
   - Caso `src/` esteja estritamente não modificado, invoca `app.approve_spec_contract()`, avançando para a Fase 4 (`RedTest`), e atualiza a fase na sessão ativa.

2. **`TuiAction::PhaseAdvanced` (`Ctrl+N`)**:
   - Invoca `app.advance_gost_phase()`.
   - Registra o avanço e o nome da nova fase no viewport.
   - Atualiza `metadata.gost.phase` na sessão ativa e persiste o arquivo de sessão.

3. **`TuiAction::FinishTriggered` (`Ctrl+G`)**:
   - Invoca `app.compile_closing_chronicle()` e exibe o texto no viewport.
   - Se a aplicação possuir um `ticket_id`, executa `driver.close_ticket(ticket_id)` e registra a confirmação no viewport.

4. **`TuiAction::MergeTriggered` (`Ctrl+M`)**:
   - Executa a mesclagem cooperativa da branch de trabalho ativa via `driver.merge_branch(&branch)`.
   - Reporta o resultado da mesclagem diretamente no viewport.

## 3. Critérios de Aceite
- Ao pressionar `Ctrl+P` na Fase 3 com arquivos em `src/` alterados, a operação é rejeitada com aviso de veto.
- Ao pressionar `Ctrl+P` sem alterações em `src/`, a aplicação avança para a Fase 4.
- Atalhos `Ctrl+N`, `Ctrl+G` e `Ctrl+M` executam suas operações normativas no loop de eventos.
