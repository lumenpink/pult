# Especificação Técnica: GOST State Machine, Action Buttons & Phase 3 Guard

## 1. Visão Geral e Propósito
A integração da máquina de estados do ciclo normativo **GOST DD-TDD** e de guardas atômicas diretamente na interface TUI do `pult` garante que nenhuma etapa seja burlada ou executada fora de conformidade com as 9 fases soberanas.

O operador tem controle tátil através de atalhos operacionais dedicados (`Ctrl+P`, `Ctrl+N`, `Ctrl+G`, `Ctrl+M`), enquanto o núcleo bloqueia ativamente infrações antes que os hooks do `cccp` precisem intervir.

## 2. Máquina de Estados da GOST DD-TDD na TUI
A TUI rastreia a fase ativa através da enum `GostPhase` do `pult-core`:
1. `Ticket` (Fase 1)
2. `NewBranch` (Fase 2)
3. `SpecContract` (Fase 3)
4. `RedTest` (Fase 4)
5. `GreenCode` (Fase 5)
6. `Refactor` (Fase 6)
7. `MutationTest` (Fase 7)
8. `QualityGate` (Fase 8)
9. `CloseTicket` (Fase 9)

### 2.1 Transições e Ações
- **`advance_gost_phase(&mut self) -> Result<GostPhase, &'static str>`:** Promove o estado sequencialmente para a próxima fase. Emite erro ao tentar avançar além da Fase 9.
- **`approve_spec_contract(&mut self) -> Result<GostPhase, &'static str>`:** Valida que a aplicação se encontra na Fase 3 (`SpecContract`) e transiciona atomicamente para a Fase 4 (`RedTest`).
- **`compile_closing_chronicle(&self, scope: &str, summary: &str) -> String`:** Gera a crônica densa de encerramento formatada para o commit canônico da Fase 9:
  ```text
  docs(<scope>): <summary> (closes #<id>)

  ### Crônica da Entrega Soberana (GOST DD-TDD)
  - Ticket: #<id>
  - Branch: <branch>
  - Turnos Processados: <N>
  - Fases Concluídas: 1 a 9
  ```

## 3. Guarda Estrita da Fase 3 (`check_phase3_guard`)
Durante a Fase 3 (`SpecContract`), o agente e a desenvolvedora têm permissão exclusiva para editar documentação e contratos de tipos/interfaces (`docs/specs/` ou arquivos com trait/interface).
- Qualquer detecção de alteração em arquivos executáveis de produção (ex.: corpos de funções em `src/`) aciona o **Veto Absoluto da Fase 3**.
- A TUI exibe alerta em vermelho no viewport de histórico e impede a transição ou persistência ilegal.

## 4. Atalhos Operacionais Contextuais
O rodapé da TUI adapta os botões exibidos conforme a fase ativa:
- **Universal:** `[Ctrl+S: Enviar]`, `[Ctrl+T: Alternar Foco]`, `[Ctrl+Q: Sair]`.
- **Fase 3:** Destaque para `[Ctrl+P: Aprovar Contrato]`.
- **Fases 1 a 8:** `[Ctrl+N: Próx. Fase]`.
- **Fase 9:** Destaque para `[Ctrl+G: Fechar Ticket]`.
- **Pós-Fechamento / Main:** `[Ctrl+M: Merge CCCP]`.
