use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pult_tui::app::{autocomplete_slash_command, App};
use pult_tui::event::{handle_key_event, TuiAction};
use pult_tui::ui::render;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_autocomplete_slash_command() {
    // Autocompletes quando o primeiro caractere é '/'
    assert_eq!(autocomplete_slash_command("/n"), Some("/new"));
    assert_eq!(autocomplete_slash_command("/ap"), Some("/approve"));
    assert_eq!(autocomplete_slash_command("/nex"), Some("/next"));
    assert_eq!(autocomplete_slash_command("/f"), Some("/finish"));
    assert_eq!(autocomplete_slash_command("/m"), Some("/merge"));
    assert_eq!(autocomplete_slash_command("/q"), Some("/quit"));
    assert_eq!(autocomplete_slash_command("/h"), Some("/help"));

    // Não deve autocompletar se não iniciar com barra
    assert_eq!(autocomplete_slash_command("new"), None);
    assert_eq!(autocomplete_slash_command("texto comum"), None);
    assert_eq!(autocomplete_slash_command("/desconhecido"), None);
}

#[test]
fn test_help_modal_toggle_with_ctrl_h_and_esc() {
    let mut app = App::new();
    assert!(!app.show_help);

    // Ctrl+H abre o modal
    let key_ctrl_h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL);
    let action1 = handle_key_event(&mut app, key_ctrl_h);
    assert_eq!(action1, Some(TuiAction::HelpToggled));
    assert!(app.show_help);

    // Esc fecha o modal de ajuda
    let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let action2 = handle_key_event(&mut app, key_esc);
    assert_eq!(action2, Some(TuiAction::HelpToggled));
    assert!(!app.show_help);
}

#[test]
fn test_slash_command_execution() {
    let mut app = App::new();

    // /help abre ajuda
    let act_help = app.execute_slash_command("/help");
    assert_eq!(act_help, Some(TuiAction::HelpToggled));
    assert!(app.show_help);

    // /new reseta a sessão
    let act_new = app.execute_slash_command("/new");
    assert_eq!(act_new, Some(TuiAction::NewSession));

    // /approve
    let act_approve = app.execute_slash_command("/approve");
    assert_eq!(act_approve, Some(TuiAction::SpecApproved));

    // /finish
    let act_finish = app.execute_slash_command("/finish");
    assert_eq!(act_finish, Some(TuiAction::FinishTriggered));

    // /merge
    let act_merge = app.execute_slash_command("/merge");
    assert_eq!(act_merge, Some(TuiAction::MergeTriggered));

    // /quit
    let act_quit = app.execute_slash_command("/quit");
    assert_eq!(act_quit, Some(TuiAction::Quit));
}

#[test]
fn test_minimalist_footer_only_shows_ctrl_h_help() {
    let app = App::new();
    let backend = TestBackend::new(100, 25);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| render(&app, f)).unwrap();
    let buffer = terminal.backend().buffer();

    let rendered_text: String = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .concat();

    // Rodapé deve exibir [Ctrl+H: Ajuda]
    assert!(rendered_text.contains("Ctrl+H: Ajuda"));

    // Não deve conter a barra antiga poluída com atalhos fixos
    assert!(!rendered_text.contains("Ctrl+P: Aprovar Spec"));
    assert!(!rendered_text.contains("Ctrl+G: Finalizar"));
    assert!(!rendered_text.contains("Ctrl+M: Mesclar"));
}
