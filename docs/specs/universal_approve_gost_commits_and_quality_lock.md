# Especificação Técnica: Comando Approve Universal, Commits GOST 2.0 e Trava Estrita Gosplan

## 1. Contexto & Motivação
O ciclo de desenvolvimento GOST 2.0 DD-TDD exige rigor e rastreabilidade estrita entre as 9 fases (Ticket -> Branch -> Spec -> Red Test -> Green Implement -> Refactor -> Mutation Test -> Quality Gate -> Finish & Close).
Anteriormente, o comando `/approve` encontrava-se rigidamente acoplado à Fase 3, exigindo atalhos manuais (`Ctrl+N` ou `/next`) nas demais fases. Além disso, os commits nas 4 fases normativas (3, 4, 5 e 9) não continham o sufixo canônico de fase `ph<N>` e não havia uma trava dura que impedisse a progressão de fase caso o `cccp check --strict` reportasse falhas.

## 2. Requisitos Técnicos

### 2.1 Comando `/approve` Universal
- O comando `/approve` (assim como seu atalho associado) deve ser aceito e processado em todas as 9 fases do ciclo DD-TDD.
- Em cada fase, `/approve`:
  1. Valida se as pré-condições da fase corrente estão satisfeitas.
  2. Em caso de fase com commit obrigatório (Fases 3, 4, 5 e 9), formata e dispara o commit canônico GOST 2.0.
  3. Coleta incrementalmente a crônica de decisões da fase.
  4. Executa a transição atômica para a fase subsequente (ex: Fase 1 -> Fase 2, Fase 3 -> Fase 4, ..., Fase 8 -> Fase 9).
  5. Não exige atalhos manuais adicionais para transicionar.

### 2.2 Gestão de Commits Normativos GOST 2.0
Nas 4 fases com commit normatizado:
- **Fase 3 (Spec Contract):**
  - Assunto: `<tipo>(<escopo>): specify <resumo> (ref #<id> - ph3)`
- **Fase 4 (Red Test):**
  - Assunto: `test(<escopo>): add failing test for <resumo> (ref #<id> - ph4)`
- **Fase 5 (Green Implement):**
  - Assunto: `<tipo>(<escopo>): implement <resumo> (ref #<id> - ph5)`
- **Fase 9 (Finish & Close):**
  - Assunto: `<tipo>(<escopo>): finalize <resumo> (closes #<id> - ph9)`
  - Corpo obrigatório ($\ge 80$ caracteres) contendo os cabeçalhos:
    - `[Resumo]:`
    - `[Solução]:`
    - `[Validação]:`
- **Invariante de Formatação:** O cabeçalho (assunto) do commit deve possuir rigorosamente $\le 72$ caracteres.

### 2.3 Trava Estrita Gosplan (Quality Gate Lock)
- Na avaliação da Fase 8 (Quality Gate):
  - Se a auditoria (`AuditInfo` / `cccp check --strict`) registrar `failures > 0` ou `!passed`:
    - A transição para a Fase 9 é sumariamente vetada.
    - O comando `/approve` retorna erro explicativo na viewport da TUI (`★ GOSPLAN QUALITY LOCK: Avanço vetado com falhas no check`).
    - Nenhuma transição de fase ocorre enquanto os apontamentos não forem sanados.

### 2.4 Coleta Incremental de Crônica via Scout
- A cada transição aprovada pelo usuário via `/approve`, os fatos, decisões arquiteturais e hipóteses descartadas da fase são acumulados no histórico estruturado da sessão.
- Na Fase 9, esse acervo subsidia a redação automática do Laudo de Resolução no corpo do commit de encerramento.
