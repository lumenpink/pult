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
    let cli = Cli::try_parse_from(["pult", "-r", "flag_repo", "pos_repo"])
        .expect("parse flag and pos");
    assert_eq!(cli.repo.as_deref(), Some("flag_repo"));
    assert_eq!(cli.repo_pos.as_deref(), Some("pos_repo"));
    // Precedência estrita da flag conforme especificação
    assert_eq!(cli.target_repo(), "flag_repo");
}
