//! Parser de argumentos de linha de comando para o PULT.

use clap::Parser;

/// Interface de linha de comando autônoma do PULT.
#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(
    name = "pult",
    version = pult_core::VERSION,
    about = "Estação TUI autônoma para terminal"
)]
pub struct Cli {
    /// Repositório alvo (caminho absoluto, relativo, owner/repo ou repo)
    #[arg(short = 'r', long = "repo", alias = "dir", short_alias = 'C')]
    pub repo: Option<String>,

    /// Repositório alvo posicional (quando invocado sem flag explícita)
    pub repo_pos: Option<String>,

    /// ID do ticket formal a ser aberto
    #[arg(short, long)]
    pub ticket: Option<u32>,

    /// Abrir sessão rascunho com descrição
    #[arg(short, long)]
    pub draft: Option<String>,

    /// Retomar sessão existente a partir de arquivo markdown
    #[arg(short = 's', long = "resume")]
    pub resume: Option<String>,

    /// Gravar trilha completa de auditoria de interações em arquivo de log
    #[arg(
        short = 'l',
        long = "log",
        num_args = 0..=1,
        default_missing_value = ".antigravity/pult/logs/interacoes.log"
    )]
    pub log: Option<String>,
}

impl Cli {
    /// Determina o identificador textual do repositório alvo com base na precedência:
    /// 1. Flag explícita `-r/--repo` (ou aliases `-C/--dir`)
    /// 2. Argumento posicional único `repo_pos`
    /// 3. Diretório corrente (".")
    pub fn target_repo(&self) -> &str {
        if let Some(ref r) = self.repo {
            r.as_str()
        } else if let Some(ref pos) = self.repo_pos {
            pos.as_str()
        } else {
            "."
        }
    }
}
