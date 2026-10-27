# Especificação Técnica: Disparo Automático de Pipeline no /approve e Encadeamento Contínuo das Fases

## 1. Contexto & Motivação
No fluxo GOST DD-TDD de 9 fases, o comando `/approve` avançava o estado interno e emitia commits normativos, porém deixava a interface TUI em espera passiva. Isso forçava a usuária a digitar manualmente novos prompts a cada fase (ex: após a criação automática da branch na Fase 2, era necessário solicitar manualmente a elaboração da especificação da Fase 3).

Ademais, as Fases 6 (Refactor), 7 (Mutation Test) e 8 (Quality Gate) compõem uma esteira técnica de consolidação da qualidade e conformidade do código que pode ser executada de forma autônoma após a implementação verde ser aprovada na Fase 5.

Portanto, o ciclo ideal de cooperação e autonomia reserva a intervenção humana estritamente para os 5 pontos de decisão estratégicos:
- **Fase 1 (Ticket):** Discussão e aprovação da funcionalidade.
- **Fase 3 (Spec Contract):** Revisão e aprovação do contrato de especificação.
- **Fase 4 (Red Test):** Revisão e aprovação dos testes falhos de contrato.
- **Fase 5 (Green Implement):** Revisão e aprovação da implementação funcional mínima.
- **Fase 9 (Finish & Close):** Inspeção final, homologação e encerramento do ticket.

## 2. Requisitos Técnicos

### 2.1 Checkpoints Estratégicos e Fases Autônomas
1. **Fase 2 (New Branch):** 100% autônoma. Criada a branch na forja via scout, o sistema transiciona de imediato para a Fase 3 e dispara o pipeline de redação de especificação.
2. **Disparo Reativo nas Fases 3, 4 e 5:**
   - Ao aprovar a Fase 1: o PULT executa a Fase 2 e automaticamente despacha a solicitação de contrato da Fase 3 para o agente/scout.
   - Ao aprovar a Fase 3: o PULT emite o commit normativo `ph3`, avança para a Fase 4 e automaticamente despacha a solicitação de implementação dos testes vermelhos da Fase 4.
   - Ao aprovar a Fase 4: o PULT emite o commit normativo `ph4`, avança para a Fase 5 e automaticamente despacha a solicitação de implementação verde funcional da Fase 5.
3. **Encadeamento Contínuo Pós-Fase 5 (Fases 6, 7 e 8):**
   - Ao aprovar a Fase 5: o PULT emite o commit normativo `ph5`, transiciona sucessivamente por Refatoração (Fase 6), Mutação/Robustez (Fase 7) e Quality Gate (Fase 8).
   - Na Fase 8, avalia a conformidade rigorosa via `driver.check_strict()`.
   - Havendo falhas (`failures > 0`), a esteira estaciona na Fase 8, vetando o avanço e alertando a usuária com o relatório de conformidade.
   - Sem falhas, a esteira atinge a Fase 9 (Close Ticket), pronta para a homologação final da usuária.

### 2.2 Síntese Automática de Prompts de Fase
- O PULT deve fornecer a função geradora de prompts canônicos de fase:
  ```rust
  pub fn phase_prompt_for(phase: GostPhase, ticket_id: Option<u32>, branch: &str) -> String;
  ```
- O prompt sintetizado deve contextualizar o agente com o ID do ticket, nome da branch e objetivo conciso da fase, sempre prefixado com a diretiva de isolamento de escopo.

### 2.3 Resposta Reativa no Event Loop da TUI
- Ao processar `TuiAction::SpecApproved`, o sistema deve:
  1. Executar as ações normativas da fase corrente (commits canônicos, criação de branch, validação de conformidade).
  2. Determinar se há prompt a despachar para a próxima fase.
  3. No caso de Fases 1 -> 3, 3 -> 4, e 4 -> 5, disparar `run_orchestration_pipeline` com o prompt gerado em segundo plano, transmitindo a saída em tempo real para o viewport.
  4. No caso de Fase 5 -> 9, executar a sequência autônoma e posicionar a aplicação na Fase 9 com relatório descritivo.

### 2.4 Invariantes
- Cabeçalhos de commits normativos GOST rigorosamente limitados a $\le 72$ caracteres com sufixo `ph<N>`.
- Vocabulário horizontal e cooperativo, sem terminologias monárquicas.
