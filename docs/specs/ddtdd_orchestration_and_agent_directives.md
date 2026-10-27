# Especificação: Orquestração DD-TDD e Diretivas de Fase na PULT

- **Ticket Relacionado**: #22
- **Fase DD-TDD**: Fase 3 (Spec / Contrato)
- **Status**: Aprovado para Implementação

## 1. Visão Geral e Motivação

O ciclo GOST DD-TDD prescreve uma sequência rigorosa de 9 fases:
1. Formulação do Plano / Escopo (Ticket)
2. Criação da Branch de Trabalho Dedicada
3. Especificação e Contrato (Doc / Spec)
4. Teste Quebrando (Red Test)
5. Implementação Mínima Funcional (Green Code)
6. Refatoração e Polimento (Refactor)
7. Testes de Mutação e Casos de Borda (Mutation Test)
8. Validação de Conformidade e Quality Gate
9. Fechamento e Crônica (Finish & Close)

A responsabilidade por governar o avanço desse ciclo pertence à PULT, e não ao Agente Externo (`agy`). Para garantir disciplina horizontal, estabilidade e conformidade:
1. **Proteção estrita da branch principal**: NUNCA permitir alterações de arquivos ou avanço para fases de modificação de código (Fases 3 a 9) diretamente na branch `main` ou `master`. A usuária e o agente devem criar uma branch de trabalho antes de avançar para a Fase 3.
2. **Diretivas especializadas por fase**: A cada turno disparado para o Agente Externo, a PULT injeta uma diretiva explícita delimitando as permissões da fase ativa. O Agente tem autorização expressa para modificar arquivos pertinentes à fase na branch de trabalho, mas tem **proibição expressa de executar comandos `git` ou `cccp`**.
3. **Auditoria contínua de conformidade (`cccp check`)**: A cada transição de fase e após a conclusão de cada ciclo de interação, a PULT executa `cccp check --strict`, exibindo o resumo de conformidade no viewport e registrando no log de auditoria.
4. **Comando `/branch <nome>` na TUI**: Permite que a usuária crie ou alterne branches de trabalho interativamente.

---

## 2. Contratos de Tipos e Métodos

### 2.1 Diretivas de Fase (`GostPhase::agent_directive`)

Em `crates/pult-core/src/domain/gost.rs`:

```rust
impl GostPhase {
    /// Gera a diretiva normativa a ser injetada no prompt do Agente Externo para a fase ativa.
    pub fn agent_directive(&self, branch: &str) -> String;
}
```

Requisitos da diretiva:
- Mencionar claramente a fase ativa e seu propósito normativo.
- Para fases >= 3, conceder autorização para editar arquivos na branch informada.
- Conter a cláusula mandatória:
  `PROIBIÇÃO EXPRESSA: Não execute comandos git ou cccp (commits, tags, merge, branches e fechamento de tickets são de responsabilidade exclusiva da PULT).`

### 2.2 Guarda de Proteção da Branch Principal (`main` / `master`)

- Se a branch ativa for `main` ou `master`:
  - Fases 1 (Ticket) e 2 (NewBranch): Permitidas para discussão e planejamento.
  - Fases 3 a 9: Vetadas para modificação de arquivos ou avanço de fase. Emite erro no viewport e interrompe a execução antes de chamar o Agente Externo.

### 2.3 Auditoria Contínua de Conformidade (`cccp check --strict`)

- Disparada em:
  - Transições de fase (`TuiAction::SpecApproved`, `TuiAction::PhaseAdvanced`, etc.).
  - Término da execução do runner no pipeline assíncrono.
- Registrada no `AuditLogger` sob o evento `QUALITY_AUDIT`.

---

## 3. Critérios de Aceite

1. O método `GostPhase::agent_directive` gera o texto correto para todas as 9 fases, contendo a proibição de `git`/`cccp`.
2. A tentativa de despachar ações de fase >= 3 na branch `main` ou `master` é bloqueada preventivamente pela PULT.
3. Transições e conclusões de turnos acionam a verificação de conformidade e registram o resultado no viewport e na auditoria.
4. O comando `/branch <nome>` é reconhecido pela TUI para criação/checkout de branches de trabalho.
