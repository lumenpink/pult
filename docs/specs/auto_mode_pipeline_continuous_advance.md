# Especificação Técnica: Orquestração Contínua de Esteira no Modo Auto

## Contexto e Motivação
No modo `[AUTO]`, a intenção é que após a usuária e o subagente alinharem o escopo e a arquitetura na Fase 1, e a usuária aprovar a proposta de projeto (Parada 1), o ciclo de vida do desenvolvimento seja executado de ponta a ponta sem exigir que a usuária aprove manualmente cada etapa intermediária, até que o ciclo atinja a Fase 9 (Parada 2) para homologação final.

## Requisitos Funcionais

### RF-01: Critério de Auto-Avanço Contínuo (`should_auto_advance_on_completed`)
- No `App`, implementar o método `pub fn should_auto_advance_on_completed(&self) -> bool`:
  - Retorna `true` se e somente se:
    1. `self.is_auto_mode == true`
    2. `self.gost_phase >= GostPhase::SpecContract` (Fase 3 ou superior)
    3. `self.gost_phase < GostPhase::CloseTicket` (antes da Fase 9)
  - Retorna `false` na Fase 1 (Ticket) para garantir a Parada 1 (aprovação de projeto mandatória pela usuária).
  - Retorna `false` na Fase 9 (CloseTicket) para garantir a Parada 2 (homologação e fechamento mandatório pela usuária).

### RF-02: Execução e Encadeamento Automático em `main.rs`
- Ao receber o evento `PipelineEvent::Completed(code)`:
  - Se `code == 0` e `app.should_auto_advance_on_completed()` for `true`:
    - Disparar `app.execute_action(TuiAction::SpecApproved, &git_driver)`.
    - Se a ação obtiver sucesso e gerar `app.take_pending_dispatch()` com o próximo prompt de fase, acionar automaticamente o background pipeline da fase seguinte (`run_orchestration_pipeline`).
  - Se `code != 0` ou se `execute_action` retornar um erro (ex.: veto ou bloqueio de quality gate), registrar o erro no viewport, manter `app.set_busy(false, "")` e interromper o avanço autônomo, aguardando intervenção manual.

## Critérios de Aceite
1. Teste unitário e de integração demonstrando que `should_auto_advance_on_completed` retorna `false` na Fase 1 e na Fase 9, e `true` nas Fases 3, 4 e 5 quando `is_auto_mode` é verdadeiro.
2. Teste simulando o encadeamento contínuo onde a conclusão da Fase 3 avança para a Fase 4 e gera o prompt automático da Fase 4, a conclusão da Fase 4 avança para a Fase 5 e gera o prompt da Fase 5, e a conclusão da Fase 5 avança através das Fases 6, 7 e 8 até parar compulsoriamente na Fase 9.
3. Teste garantindo que em caso de erro de qualidade ou veto, o avanço autônomo é pausado.
4. Conformidade estrita Gosplan (`cccp check --strict`) com 100% de conformidade.
