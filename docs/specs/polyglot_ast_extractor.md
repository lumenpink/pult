# Especificação Técnica: Polyglot AST Extractor Engine

## 1. Visão Geral e Propósito
O `PolyglotAstExtractor` estende a capacidade de extração de assinaturas estruturais do `pult-core` para repositórios poliglotas, implementando a trait `AstExtractor`. O extrator atua como primeira linha de defesa contra o estouro de contexto de IA, reduzindo o código-fonte em 70% a 90% ao preservar apenas declarações de tipos, contratos, classes e assinaturas de funções/métodos, substituindo o interior de corpos executáveis por blocos elididos (`{ ... }`, `pass` ou `:`).

## 2. Roteamento por Extensão e Gramáticas
O despachante seleciona a estratégia de parsing com base na extensão do arquivo de destino:

1. **Rust (`.rs`):**
   - Despacha diretamente para o `RustAstExtractor` nativo (`syn` + `quote`).
2. **TypeScript (`.ts`, `.tsx`):**
   - Gramática `tree-sitter-typescript`.
   - Extrai: `interface_declaration`, `type_alias_declaration`, `enum_declaration`, `class_declaration` (com assinaturas de métodos e propriedades públicas/protegidas), `function_declaration`, `export_statement`.
   - Omite: blocos `{ ... }` de corpos de funções/métodos executáveis.
3. **JavaScript (`.js`, `.mjs`, `.cjs`):**
   - Gramática `tree-sitter-javascript`.
   - Extrai: `class_declaration`, `function_declaration`, `export_statement`, métodos de protótipo.
   - Omite: blocos `{ ... }` de corpos de funções/métodos.
4. **Python (`.py`):**
   - Gramática `tree-sitter-python`.
   - Extrai: `class_definition` (com métodos e anotações de tipo de classe), `function_definition` (assinatura completa com decorators e type hints).
   - Omite: corpo interno da função/método, substituindo por `...` ou `pass`.
5. **PHP (`.php`):**
   - Gramática `tree-sitter-php`.
   - Extrai: `interface_declaration`, `class_declaration`, `trait_declaration`, `enum_declaration`, assinaturas de métodos públicos e propriedades tipadas.
   - Omite: blocos `{ ... }` internos de métodos.
6. **Shell / Bash (`.sh`, `.bash`):**
   - Gramática `tree-sitter-bash`.
   - Extrai: `function_definition` (nomes de funções declaradas no script) e variáveis exportadas.
   - Omite: corpos `{ ... }` de comandos internos de funções.

## 3. Resiliência e Tolerância a Sintaxe Incompleta
Ambientes de edição interativa produzem código com erros sintáticos parciais enquanto o usuário digita. O motor baseado em `tree-sitter` utiliza análise GLR/LR tolerante a falhas:
- Se houver nós sintáticos `ERROR` na árvore sintática, a extração dos nós irmãos válidos continua sem interrupções.
- Arquivos que não conseguem ser interpretados por nenhuma gramática conhecida retornam erro tipado `AstError::UnsupportedLanguage` ou um resumo textual seguro, impedindo panics do processo.

## 4. Varredura de Diretório e Filtros
Ao executar `extract_from_dir`:
- Varredura recursiva de arquivos via `WalkDir`.
- Filtros automáticos: exclusão de diretórios ocultos (`.*`), `target`, `node_modules`, `vendor`, `.venv`, `dist`, `build`.
- Somente arquivos com extensões suportadas (`.rs`, `.py`, `.ts`, `.tsx`, `.js`, `.mjs`, `.cjs`, `.php`, `.sh`, `.bash`) são processados.
