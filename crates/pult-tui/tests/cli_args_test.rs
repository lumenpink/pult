use clap::Parser;
use pult_tui::cli::Cli;

#[test]
fn test_cli_default_target_is_dot() {
    let cli = Cli::try_parse_from(["pult"]).expect("parse default");
    assert_eq!(cli.repo, None);
    assert_eq!(cli.repo_pos, None);
    assert_eq!(cli.target_repo(), ".");
}

#[test]
fn test_cli_positional_repo() {
    let cli = Cli::try_parse_from(["pult", "joaju"]).expect("parse positional repo");
    assert_eq!(cli.repo, None);
    assert_eq!(cli.repo_pos.as_deref(), Some("joaju"));
    assert_eq!(cli.target_repo(), "joaju");
}

#[test]
fn test_cli_flag_short_r() {
    let cli = Cli::try_parse_from(["pult", "-r", "joaju", "-t", "12"]).expect("parse -r");
    assert_eq!(cli.repo.as_deref(), Some("joaju"));
    assert_eq!(cli.ticket, Some(12));
    assert_eq!(cli.target_repo(), "joaju");
}

#[test]
fn test_cli_flag_long_repo() {
    let cli = Cli::try_parse_from(["pult", "--repo", "lumen/lauda", "--draft", "nova tela"])
        .expect("parse --repo");
    assert_eq!(cli.repo.as_deref(), Some("lumen/lauda"));
    assert_eq!(cli.draft.as_deref(), Some("nova tela"));
    assert_eq!(cli.target_repo(), "lumen/lauda");
}

#[test]
fn test_cli_flag_alias_dir_and_short_c() {
    let cli_c = Cli::try_parse_from(["pult", "-C", "../navalha"]).expect("parse -C");
    assert_eq!(cli_c.repo.as_deref(), Some("../navalha"));
    assert_eq!(cli_c.target_repo(), "../navalha");

    let cli_dir = Cli::try_parse_from(["pult", "--dir", "/var/repo"]).expect("parse --dir");
    assert_eq!(cli_dir.repo.as_deref(), Some("/var/repo"));
    assert_eq!(cli_dir.target_repo(), "/var/repo");
}

#[test]
fn test_cli_precedence_flag_over_positional() {
    let cli =
        Cli::try_parse_from(["pult", "-r", "flag_repo", "pos_repo"]).expect("parse flag and pos");
    assert_eq!(cli.repo.as_deref(), Some("flag_repo"));
    assert_eq!(cli.repo_pos.as_deref(), Some("pos_repo"));
    // Precedência estrita da flag conforme especificação
    assert_eq!(cli.target_repo(), "flag_repo");
}

#[test]
fn test_mutation_cli_combination_with_multiple_flags() {
    let cli = Cli::try_parse_from([
        "pult",
        "-r",
        "my-repo",
        "-t",
        "42",
        "-d",
        "tarefa em andamento",
        "-s",
        "saved_session.md",
    ])
    .expect("parse combined flags");

    assert_eq!(cli.repo.as_deref(), Some("my-repo"));
    assert_eq!(cli.ticket, Some(42));
    assert_eq!(cli.draft.as_deref(), Some("tarefa em andamento"));
    assert_eq!(cli.resume.as_deref(), Some("saved_session.md"));
    assert_eq!(cli.target_repo(), "my-repo");
}

#[test]
fn test_mutation_cli_positional_combined_with_ticket() {
    let cli = Cli::try_parse_from(["pult", "joaju", "--ticket", "99"])
        .expect("parse positional with ticket");

    assert_eq!(cli.repo, None);
    assert_eq!(cli.repo_pos.as_deref(), Some("joaju"));
    assert_eq!(cli.ticket, Some(99));
    assert_eq!(cli.target_repo(), "joaju");
}

#[test]
fn test_mutation_cli_unrecognized_argument_fails() {
    let res = Cli::try_parse_from(["pult", "--invalid-unknown-option"]);
    assert!(res.is_err());
}

#[test]
fn test_cli_log_flag_default_and_custom() {
    let cli_default = Cli::try_parse_from(["pult", "--log"]).expect("parse --log");
    assert_eq!(
        cli_default.log.as_deref(),
        Some(".antigravity/pult/logs/interacoes.log")
    );

    let cli_short = Cli::try_parse_from(["pult", "-l"]).expect("parse -l");
    assert_eq!(
        cli_short.log.as_deref(),
        Some(".antigravity/pult/logs/interacoes.log")
    );

    let cli_custom = Cli::try_parse_from(["pult", "-l", "meu.log"]).expect("parse custom log");
    assert_eq!(cli_custom.log.as_deref(), Some("meu.log"));

    let cli_none = Cli::try_parse_from(["pult"]).expect("parse sem log");
    assert_eq!(cli_none.log, None);
}
