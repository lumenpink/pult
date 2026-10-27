# Especificação Técnica: Staging Automático em commit_phase e Bypass Ágil do Scout

## Contexto e Motivação
Na orquestração das fases do GOST pelo PULT, foram identificados dois gargalos fundamentais de execução no núcleo `pult-core`:
1. `RepoDriver::commit_phase` executava diretamente `git commit -m <msg>` sem antes fazer o staging (`git add -A`). Quando o subagente (`agy`) criava novos arquivos normativos de especificação (`docs/spec.md`), testes (`tests/`) ou código funcional (`src/`), estes permaneciam como arquivos não rastreados (*untracked*), fazendo com que o Git abortasse o commit com erro de árvore limpa e deixando alterações órfãs.
2. `ScoutEngine::filter_with_bypass` acionava a LLM local (Ollama) mesmo quando a base de código não continha nenhum arquivo AST indexável (`ast_files.is_empty()`) ou quando a mensagem da usuária era uma simples confirmação conversacional (`y`, `yes`, `sim`, `ok`). Isso adicionava 10 segundos de latência desnecessária e produzia respostas conversacionais alucinadas no log.

## Requisitos Funcionais

### RF-01: Staging Automático de Arquivos Normativos
- Em `RepoDriver::commit_phase`, após a validação de guarda estrita (`validate_phase_diff`), o driver deve executar `git add -A` antes de invocar `git commit -m <msg>`.
- Todos os arquivos criados ou modificados na fase ativa devem ser capturados no commit correspondente.

### RF-02: Bypass Imediato no Scout Engine
- Em `ScoutEngine`, criar o método utilitário `is_short_confirmation(text: &str) -> bool` que identifica respostas curtas de concordância (ex.: `y`, `yes`, `sim`, `s`, `ok`, `concordo`, `aprovado`, e variações com pontuação ou espaços).
- Em `ScoutEngine::filter_with_bypass`:
  - Se `ast_files.is_empty()` OU `is_short_confirmation(task_intent)` for verdadeiro, fazer o bypass imediato retornando `ScoutResult::bypassed()`, sem efetuar chamadas HTTP para o Ollama.
  - O resultado bypassed deve conter `filtered_symbols: vec![]` e preservar os metadados de execução rápida.

## Critérios de Aceite
1. Teste unitário e de integração demonstrando que `RepoDriver::commit_phase` efetua o `git add -A` e inclui arquivos anteriormente *untracked* no commit do Git.
2. Teste unitário comprovando que `ScoutEngine::filter_with_bypass` retorna instantaneamente `bypassed` quando `ast_files` está vazio ou quando a entrada é `"y"`, `"sim"`, etc., sem acionar `client.generate`.
3. Validação estrita Gosplan (`cccp check --strict`) com 100% de conformidade.
