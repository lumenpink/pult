# Especificação Técnica: Acumulação de Stream por Chunks, Quebra Fluida e Vocabulário Autônomo (Ticket #15)

- **Ticket:** #15
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
Eliminar a fragmentação artificial de linhas no viewport da TUI gerada por buffers parciais de streaming, garantir que o texto aproveite 100% da largura útil da tela com quebra orgânica por palavras no Ratatui, e expurgar todo o vocabulário monárquico ("soberano/soberana") em favor de princípios anarquistas, horizontais e autônomos.

## 2. Acumulação e Bufferização de Chunks (`append_stream_chunk`)
- O método `append_stream_chunk(&mut self, chunk: &str)` no `App`:
  - Recebe pedaços arbitrários emitidos pelos canais de stdout/stderr do Agente Externo.
  - Divide o chunk pelo caractere `\n`.
  - O primeiro fragmento do chunk é concatenado diretamente (`push_str`) à última linha existente em `viewport_lines` (se não vazia e a linha anterior ainda estiver aberta).
  - Cada fragmento subsequente após um `\n` inicia uma nova linha em `viewport_lines`.
  - Um indicador ou estado rastreia se a última linha foi terminada com `\n`.

## 3. Renderização Fluida no Viewport
- No `render_viewport` de `ui.rs`:
  - O widget `Paragraph` aplica `Wrap { trim: false }`, permitindo que frases longas quebrem nas fronteiras de palavras em toda a largura do bloco retangular.
  - A rolagem vertical considera as linhas visíveis sem distorcer o fluxo contínuo.

## 4. Expurgo Terminológico ("Soberano" -> Autônomo / Agente Externo)
- Todas as ocorrências de "soberano", "soberana" e "soberania" são removidas:
  - `"Crônica da Entrega Soberana"` $\to$ `"Crônica da Entrega (GOST DD-TDD)"`.
  - `"Disparando agente soberano"` $\to$ `"Disparando Agente Externo (agy)"`.
  - `"Estação TUI soberana"` $\to$ `"Estação TUI autônoma para terminal"`.
  - `"Contrato soberano"` $\to$ `"Driver de repositório"`.
  - `"Mesclagem Soberana"` $\to$ `"Mesclagem Cooperativa"`.
