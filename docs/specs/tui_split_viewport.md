# Especificação Técnica: TUI Split Viewport & Multiline TextArea

## 1. Visão Geral e Propósito
A interface gráfica de terminal (TUI) do `pult` substitui o REPL linear tradicional dos agentes de inteligência artificial por uma estação de trabalho em tela dividida baseada em `ratatui`, `crossterm` e `tui-textarea`.

A arquitetura resolve a quebra de ergonomia causada por saídas longas de código, desacoplando completamente a rolagem vertical do histórico de leitura da área de digitação ativa no editor multilinha.

## 2. Geometria de Layout em Tela Dividida
A tela ocupa 100% da viewport do terminal (`EnterAlternateScreen`, `enable_raw_mode`) com a seguinte subdivisão vertical:

1. **Header / Barra de Status Superior (`Constraint::Length(3)`):**
   - Título: `[PULT v0.1.0-dev]`.
   - Indicador normativo da Fase GOST (ex.: `FASE 5: GreenCode`).
   - Identificação do ticket ativo (ex.: `#8`) e branch Git corrente (ex.: `feat/8-split-viewport-shell`).
2. **Viewport de Leitura Superior (`Constraint::Min(5)`):**
   - Exibe o histórico de turnos anteriores e o streaming em tempo real do agente.
   - Rolagem vertical independente através de `scroll_offset`.
   - Destaque visual na borda quando o foco estiver ativo na viewport.
3. **Editor Multilinha Inferior (`Constraint::Length(7)`):**
   - Instância de `tui_textarea::TextArea` oferecendo 5 linhas úteis de edição livre.
   - Suporte a quebras de linha (`Enter`), navegação por palavras e atalhos de edição de texto.
   - Borda visual com cor destacada quando o foco estiver ativo no editor.
4. **Footer / Barra de Ações Rápidas (`Constraint::Length(1)`):**
   - Mapeamento de atalhos operacionais: `[Ctrl+S: Enviar]`, `[Ctrl+T: Alternar Foco]`, `[Ctrl+Q: Sair]`.

## 3. Gestão e Desacoplamento de Foco
O estado de foco é modelado pela enum `Focus`:
- `Focus::Editor`: Toda digitação de caracteres, teclas de direção e edição são capturadas pelo `TextArea`. O cursor de texto fica visível na área de digitação.
- `Focus::Viewport`: O cursor de digitação é ocultado ou fica estático. As teclas de navegação (`j`/`k`, `Up`/`Down`, `PgUp`/`PgDn`, `Home`/`End`) controlam exclusivamente a rolagem do histórico de saída sem afetar o texto do editor.

### 3.1 Mapeamento Universal de Atalhos
- **`Ctrl+T`:** Alterna ciclicamente o foco entre `Focus::Editor` e `Focus::Viewport`.
- **`Ctrl+S`:** Submete o conteúdo do editor, limpa a caixa de texto e emite a ação `Submit(String)`.
- **`Ctrl+Q`:** Sinaliza `should_quit = true`, restaurando o terminal de forma limpa.

## 4. Estado da Aplicação (`App`)
A struct `App` encapsula:
- `focus: Focus`
- `scroll_offset: usize`
- `viewport_lines: Vec<String>`
- `textarea: TextArea<'static>`
- `should_quit: bool`
- `ticket_id: Option<u32>`
- `branch_name: String`
- `gost_phase: u8`
- `submitted_input: Option<String>`

Métodos de conveniência controlam a rolagem (`scroll_up`, `scroll_down`, `scroll_to_top`, `scroll_to_bottom`) e a injeção de novas linhas de saída (`append_viewport_line`, `append_viewport_text`).
