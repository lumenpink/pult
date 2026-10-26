# Especificação: Resolução Flexível de Repositório Local (Ticket #13)

- **Ticket:** #13
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
Permitir que a estação de trabalho soberana Pult seja iniciada a partir de qualquer diretório apontando para qualquer projeto local no sistema, combinando flags e argumentos posicionais com inferência ergonômica idêntica ao `cccp ticket divan`.

## 2. Interface da Linha de Comando (CLI)
- **Flag explícita:** `-r, --repo <DIR_OR_REPO>` (aliases: `-C`, `--dir`).
- **Argumento posicional:** `[repo]` (argumento posicional opcional).
- **Precedência:**
  1. Se `-r` / `--repo` for informado, ele tem precedência estrita para definir o repositório-alvo.
  2. Se `-r` for omitido e houver argumento posicional, ele é resolvido como o repositório-alvo.
  3. Se nenhum for informado, assume o diretório corrente (`.`).
- **Combinação com outras flags:**
  - `pult -r joaju -t 12`
  - `pult --repo lumen/lauda --draft "nova tela"`
  - `pult -r ../navalha --resume sessao.md`
  - `pult joaju`

## 3. Algoritmo de Resolução (`pult_core::repo::resolve_repo_path`)
Entrada: `input: &str`, `base_dir: Option<&Path>`.

1. **Vazio ou Ponto:** Se `input` for vazio ou `.`, retorna o diretório atual (`std::env::current_dir()`).
2. **Expansão de Home:** Se iniciar com `~/`, expande a variável de ambiente `$HOME`.
3. **Caminho Existente:** Se o caminho existir no sistema de arquivos (absoluto ou relativo a `base_dir`), canoniciza e valida se é diretório.
4. **Formato `owner/repo`:**
   - Procura em `$HOME/src/<repo>`
   - Procura em `$HOME/src/<owner>/<repo>`
   - Procura em `<base_dir>/<repo>`
   - Procura em `<base_dir>/<owner>/<repo>`
5. **Nome Simples (`repo`):**
   - Procura em `$HOME/src/<repo>`
   - Procura em `<base_dir>/<repo>`
   - Valida se `base_dir` possui o mesmo nome (`base_dir.file_name() == repo`)
   - Inspeciona catálogos locais (`.antigravity/catalog.json` ou `~/.config/cccp/catalog.json`), se disponíveis.
6. **Erro:** Caso nenhuma localização física seja encontrada, retorna `RepoError::NotFound(input)`.

## 4. Integração na TUI (`crates/pult-tui`)
- O repositório resolvido define o diretório de trabalho raiz da aplicação (`std::env::set_current_dir`).
- O `App` registra o caminho canônico do repositório, exibindo o nome do projeto no título da janela e na barra de telemetria superior.
