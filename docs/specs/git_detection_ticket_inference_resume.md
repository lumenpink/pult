# Especificação Técnica: Detecção Dinâmica de Git, Inferência de Ticket e Suporte a Resume (Ticket #17)

- **Ticket:** #17
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
A TUI do PULT iniciava com metadados estáticos (`branch_name = "main"`, `ticket_id = None`), ignorando o contexto real do repositório Git subjacente. Além disso, os argumentos `--resume` / `-s` eram aceitos pelo parser CLI, mas descartados sem ação. Esta especificação define a inicialização contextual da TUI inspecionando o repositório Git ativo, a extração determinística do identificador de ticket a partir do nome da branch e o carregamento completo do histórico de uma sessão prévia com sincronização de fase GOST.

## 2. Detecção Dinâmica de Git e Inferência de Ticket
- Na inicialização do PULT (`run_tui`):
  - Consulta o branch ativo via `CccpGitDriver::new(&repo_dir).current_branch()`.
  - Se a chamada for bem-sucedida, atualiza `app.branch_name`.
  - Extrai o identificador numérico de ticket (`ticket_id: Option<u32>`) usando a função pura `infer_ticket_from_branch(branch: &str) -> Option<u32>`:
    - Suporta prefixos convencionais: `feat/<id>-*`, `fix/<id>-*`, `docs/<id>-*`, `refactor/<id>-*`, `test/<id>-*`.
    - Suporta formato direto: `<id>-*`.
    - Ramos sem numeração (ex.: `main`, `master`, `devel`) retornam `None`.
  - Se a flag explícita `--ticket <id>` tiver sido fornecida na CLI, ela tem precedência sobre a inferência do branch.
  - O cabeçalho da TUI reflete o ticket inferido/explícito e o nome real do branch.

## 3. Restauração de Sessão via `--resume` (`-s`)
- Quando `--resume <ARQUIVO_OU_ID>` for especificado:
  - Localiza o arquivo `.md` correspondente no caminho fornecido ou em `.antigravity/pult/sessions/`.
  - Carrega a sessão através de `pult_core::Session::from_markdown` ou `FileSessionStorage`.
  - Popula `app.viewport_lines` com o histórico de todos os turnos:
    - Turno de usuário: prefixado com `> ` seguido pelo conteúdo.
    - Turno do Scout: delimitado por marcadores de contexto estrutural.
    - Turno do Agente: exibido integralmente linha por linha.
  - Sincroniza a fase GOST da aplicação com `session.metadata.gost.phase`.
  - Se a sessão possuir `ticket`, sincroniza `app.ticket_id` e `app.branch_name`.
  - Armazena a sessão carregada em `app.session` para que novos comandos continuem a partir do próximo turno sequencial.
