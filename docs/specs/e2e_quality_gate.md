# Especificação Técnica: End-to-End Integration Suite & Quality Gate

## 1. Visão Geral e Propósito
A suíte de testes de integração ponta a ponta (`E2E`) valida a interoperabilidade completa entre todos os subsistemas do **PULT**:
1. Extração estrutural de AST poliglota (`PolyglotAstExtractor`).
2. Filtragem de contexto pelo motor Scout local (`ScoutEngine`).
3. Orquestração e persistência concorrente em streaming do runner Antigravity (`AgyRunner` + `FileSessionStorage`).
4. Governança da máquina de estados do ciclo DD-TDD em 9 fases e guarda estrita da Fase 3 (`GostPhase`).
5. Estabilidade da interface TUI em condições extremas de densidade de histórico (5.000 linhas de scroll).
6. Conformidade dos cabeçalhos HTTP e configuração de deploy estático em `site/` e `wrangler.jsonc`.

## 2. Roteiro do Teste de Integração E2E
O pipeline orquestrado em teste executa o ciclo completo de um ticket:

1. **Rascunho Inicial:** Uma sessão preliminar `tXXXXX-...` é aberta para formulação de intenção.
2. **Promocão para Ticket Formal:** A sessão rascunho é convertida em sessão formal vinculada ao ticket `#10` e branch `test/10-e2e-quality-gate`.
3. **Extração de AST da Base de Código:** O extrator poliglota varre diretórios temporários contendo arquivos `.rs`, `.py` e `.ts`, gerando sumários compactados com omissão de corpos funcionais.
4. **Filtragem Cirúrgica Scout:** O motor Scout processa a intenção e os arquivos de AST, emitindo o bloco estruturado `injected_ast`.
5. **Execução do Agente e Streaming:** O runner assíncrono executa o turno do agente, enviando chunks por canais MPSC e persistindo o turno em tempo real no arquivo de sessão em disco (`status: in_progress` $\to$ `status: completed`).
6. **Auditoria de Guardas GOST:** Verifica que qualquer tentativa de alterar arquivos executáveis em `src/` durante a Fase 3 é vetada com erro dedicado.
7. **Fechamento e Crônica da Fase 9:** Ao atingir a Fase 9, a crônica densa de encerramento é compilada, consolidando o histórico de turnos.

## 3. Teste de Estresse de Rolagem de Alta Densidade
- Injeta 5.000 linhas de texto na viewport de leitura da TUI.
- Executa operações sequenciais de rolagem rápida (`scroll_down`, `scroll_up`, `scroll_to_bottom`, `scroll_to_top`).
- Comprova que a rolagem ocorre sem panics, sem estouro de pilha e sem corromper o cursor ou o buffer do `tui-textarea` inferior.

## 4. Auditoria de Segurança e Deploy Estático
- Validação dos cabeçalhos em `site/_headers`: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy`.
- Validação do arquivo de configuração `wrangler.jsonc` com a rota customizada `pult.2lp.in` e diretório de assets `./site`.
