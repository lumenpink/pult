# Especificação Técnica: Inspeção de Arquivos e Visualizador de Diff na Fase 9

## Contexto e Motivação
Na Fase 9 (CloseTicket), a usuária precisa de total transparência e visibilidade sobre as alterações efetuadas ao longo do ticket antes de emitir a homologação final:
1. Exibir no viewport a relação de arquivos modificados e estatísticas de linhas alteradas (`git diff --stat`).
2. Exibir a minuta da crônica e mensagem canônica de encerramento (`(closes #<id> - ph9)`).
3. Disponibilizar um visualizador de diff colorido integrado em janela modal scrollável na própria TUI, acessível por comando `/diff` ou atalho rápido `Ctrl+D`.

## Requisitos Funcionais

### RF-01: Métodos de Inspeção no Driver de Repositório (`RepoDriver`)
- Adicionar ao trait `RepoDriver`:
  - `fn diff_stat(&self, base_branch: &str) -> Result<String, DriverError>`: Executa `git diff --stat <base_branch>...HEAD` (ou fallback para `status`).
  - `fn diff_full(&self, base_branch: &str) -> Result<String, DriverError>`: Executa `git diff <base_branch>...HEAD`.

### RF-02: Apresentação Estruturada ao Atingir a Fase 9
- Em `App::execute_action` ou na transição para a Fase 9 (CloseTicket):
  - Exibir no viewport:
    - `★ [GOST Fase 9: Homologação Final & Fechamento]`
    - Relação e estatísticas dos arquivos alterados obtidas via `diff_stat`;
    - Minuta da mensagem canônica do commit de fechamento e crônica compilada;
    - Dica de usabilidade: `"ℹ Pressione Ctrl+D ou /diff para revisar o diff completo. Pressione Ctrl+P para finalizar."`

### RF-03: Modal de Diff Integrado (Ctrl+D e /diff)
- No `App`:
  - Campos `show_diff_modal: bool`, `diff_lines: Vec<String>` e `diff_scroll: usize`.
  - Método `toggle_diff_modal(&mut self, driver: &dyn RepoDriver)`.
- No loop de eventos (`event.rs`):
  - Atalho `Ctrl+D` e comando `/diff` alternam a exibição do modal.
  - Com o modal aberto:
    - Tecla `Esc` ou `Ctrl+D` fecha o modal;
    - Teclas de seta (`Up`/`Down`) ou `j`/`k` e `PageUp`/`PageDown` navegam pelo diff.
- Na interface gráfica (`ui.rs`):
  - Função `render_diff_modal`:
    - Renderiza uma janela flutuante centralizada sobre o terminal;
    - Colore linhas que começam com `+` em verde, `-` em vermelho, `@@` em ciano e cabeçalhos em amarelo.

## Critérios de Aceite
1. Teste unitário e de integração demonstrando que `RepoDriver::diff_stat` e `diff_full` executam os comandos git esperados.
2. Teste comprovando que a transição para a Fase 9 registra as estatísticas de arquivos alterados e minuta no viewport.
3. Teste comprovando que `Ctrl+D` e `/diff` abrem o modal de diff e preenchem `diff_lines`.
4. Teste de renderização do modal com TestBackend sem pânico.
5. Conformidade estrita Gosplan (`cargo fmt`, `cargo clippy`, `cccp check --strict`).
