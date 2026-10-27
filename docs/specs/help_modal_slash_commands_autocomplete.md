# Especificação Técnica: Modal de Ajuda Ctrl+H, Comandos com Barra e Autocomplete (Ticket #19)

- **Ticket:** #19
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
A barra de rodapé da TUI exibia uma profusão de atalhos fixos que poluíam a tela. Além disso, a ergonomia do PULT deve permitir que todas as operações possam ser disparadas tanto por atalhos de controle (`Ctrl+`) quanto por comandos com barra (`/`), com suporte a autocomplete inteligente e um modal de ajuda flutuante para consulta rápida.

## 2. Rodapé Minimalista
- O rodapé passa a exibir apenas:
  `[Ctrl+H: Ajuda]`
- Nenhum outro texto de atalho é exibido na barra inferior.

## 3. Modal de Ajuda Flutuante (`Ctrl+H` ou `/help`)
- Estado no `App`: `pub show_help: bool`.
- Alternado via `Ctrl+H` ou digitando o comando `/help`.
- Fechado com tecla `Esc` ou novamente `Ctrl+H`.
- Renderiza um bloco flutuante centralizado (`Clear` + `Block` com borda arredondada ou dupla) contendo uma tabela limpa:
  - `/new` | `Ctrl+N` : Criar nova sessão de trabalho
  - `/approve` | `Ctrl+P` : Aprovar especificação e avançar para Red Test (Fase 4)
  - `/next` | `Ctrl+N` : Avançar para a próxima fase GOST
  - `/finish` | `Ctrl+G` : Compilar crônica e encerrar ticket
  - `/merge` | `Ctrl+M` : Executar cccp merge na branch
  - `/help` | `Ctrl+H` : Abrir / fechar esta modal de ajuda
  - `/quit` | `Ctrl+Q` : Encerrar a aplicação PULT
  - N/A | `Ctrl+T` : Alternar foco entre Editor e Viewport
  - N/A | `Ctrl+S` : Enviar instrução do editor

## 4. Interpretador de Comandos com Barra no Editor
- Quando a usuária pressiona Enter ou `Ctrl+S` e a primeira linha inicia com `/`:
  - `/help` $\to$ abre a modal de ajuda.
  - `/new` $\to$ reseta a sessão ativa (`app.reset_session()`) e loga `"★ Nova sessão iniciada."`.
  - `/approve` $\to$ emite `TuiAction::SpecApproved`.
  - `/next` $\to$ emite `TuiAction::PhaseAdvanced(next)`.
  - `/finish` $\to$ emite `TuiAction::FinishTriggered`.
  - `/merge` $\to$ emite `TuiAction::MergeTriggered`.
  - `/quit` $\to$ emite `TuiAction::Quit`.
  - Comando desconhecido com barra $\to$ loga aviso no viewport com sugestão de `/help`.

## 5. Autocomplete de Comandos
- Quando o primeiro caractere da primeira linha no editor for `/` e o usuário pressionar `Tab` (ou conforme digita):
  - Função pura `autocomplete_slash_command(prefix: &str) -> Option<&'static str>`:
    - Comandos candidatos: `/new`, `/approve`, `/next`, `/finish`, `/merge`, `/quit`, `/help`.
    - Retorna o casamento único mais próximo completando o comando no editor.
