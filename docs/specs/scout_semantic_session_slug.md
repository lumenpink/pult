# Especificação Técnica: Síntese Semântica de Slug de Sessão pelo Scout (Ticket #21)

- **Ticket:** #21
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
Anteriormente, o nome dos arquivos de sessão do PULT era derivado de um corte cru do primeiro prompt da usuária, resultando em identificadores truncados de forma deselegante e sem coerência semântica. Esta especificação define a geração de um slug semântico em formato `kebab-case` ($\le 60$ caracteres) sintetizado pelo motor local do Scout (`ScoutEngine`), com fallback determinístico caso o Ollama esteja offline.

## 2. Motor de Síntese Semântica (`ScoutEngine::generate_session_slug`)
- Assinatura assíncrona:
  ```rust
  pub async fn generate_session_slug(&self, input_text: &str) -> String;
  ```
- Comportamento:
  1. Se o Ollama estiver acessível e o modelo local responder:
     - Envia prompt cirúrgico solicitando:
       `"Resuma o seguinte objetivo de software em exatamente uma frase curta no formato kebab-case (letras minúsculas e hífens apenas), sem pontuação, com no máximo 60 caracteres:\n{input_text}"`
     - Limpa a resposta, filtra caracteres não-alfanuméricos (mantendo hífens) e garante que o comprimento seja $\le 60$ caracteres.
  2. Fallback determinístico (se Ollama estiver inacessível, der timeout ou retornar vazio):
     - Executa sanitização puramente algorítmica:
       - Converte para minúsculas.
       - Substitui pontuações e espaços por hífens (`-`).
       - Remove hífens duplicados consecutivos e nas pontas.
       - Trunca no máximo em 60 caracteres sem quebrar palavras ao meio quando possível.
       - Caso o resultado fique vazio, utiliza `"sessao-trabalho"`.

## 3. Aplicação na Criação de Sessões
- Na criação de novas sessões no pipeline do PULT (`run_orchestration_pipeline`):
  - Obtém o slug gerado: `let slug = scout.generate_session_slug(&prompt).await;`
  - Instancia `Session::new_formal(ticket_id, &slug, &branch_name)` ou `Session::new_draft(&slug)`.
- O nome de arquivo resultante passa a ser:
  - Formal: `{:06}-{}-{}.md` (ex.: `000021-20261027-adicionar-limitador-de-taxa.md`)
  - Rascunho: `t{:05}-{}-{}.md` (ex.: `t48192-20261027-especificar-contrato-fase-tres.md`)
