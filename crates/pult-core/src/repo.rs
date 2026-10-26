//! Resolução flexível e ergonômica de repositório local.
//!
//! Permite localizar repositórios alvo através de:
//! - Caminho corrente (`.` ou vazio)
//! - Caminhos absolutos e relativos (`./path`, `/home/...`)
//! - Expansão de home (`~/...`)
//! - Padrão `owner/repo` (procurado em `~/src/<repo>`, `~/src/<owner>/<repo>`, etc.)
//! - Padrão simples `repo` (procurado em `~/src/<repo>`, diretório base, etc.)

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Erros originados durante a resolução do repositório local.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RepoError {
    #[error("Repositório local não encontrado para '{0}'")]
    NotFound(String),

    #[error("Caminho '{0}' não é um diretório")]
    NotADirectory(PathBuf),

    #[error("Erro de I/O ao resolver repositório: {0}")]
    Io(String),
}

/// Resolve o caminho de um repositório a partir de uma entrada textual.
pub fn resolve_repo_path(input: &str, base_dir: Option<&Path>) -> Result<PathBuf, RepoError> {
    resolve_repo_path_with_env(input, base_dir, None)
}

/// Resolve o caminho de um repositório permitindo sobreposição explícita de `home_dir`.
pub fn resolve_repo_path_with_env(
    input: &str,
    base_dir: Option<&Path>,
    home_dir: Option<&Path>,
) -> Result<PathBuf, RepoError> {
    let clean = input.trim();

    // 1. Diretório corrente quando vazio ou "."
    if clean.is_empty() || clean == "." {
        let dir = match base_dir {
            Some(b) => b.to_path_buf(),
            None => std::env::current_dir().map_err(|e| RepoError::Io(e.to_string()))?,
        };
        return validate_directory(&dir);
    }

    let resolved_home: Option<PathBuf> = home_dir
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from));

    // 2. Expansão de til (~/...)
    let expanded: PathBuf = if clean == "~" {
        resolved_home
            .clone()
            .ok_or_else(|| RepoError::NotFound(clean.to_string()))?
    } else if let Some(stripped) = clean.strip_prefix("~/") {
        resolved_home
            .clone()
            .ok_or_else(|| RepoError::NotFound(clean.to_string()))?
            .join(stripped)
    } else {
        PathBuf::from(clean)
    };

    // 3. Verificação direta de existência no sistema de arquivos
    if expanded.is_absolute() {
        if expanded.exists() {
            return validate_directory(&expanded);
        }
    } else {
        let candidate = match base_dir {
            Some(b) => b.join(&expanded),
            None => expanded.clone(),
        };
        if candidate.exists() {
            return validate_directory(&candidate);
        }
    }

    // 4. Busca por owner/repo ou repo simples
    let (owner, repo_name) = match clean.split_once('/') {
        Some((o, r)) => (Some(o.trim()), r.trim()),
        None => (None, clean),
    };

    // Se for nome simples, verifica se o próprio diretório base já é o repositório
    if owner.is_none() {
        if let Some(b) = base_dir {
            if let Some(folder_name) = b.file_name().and_then(|n| n.to_str()) {
                if folder_name.eq_ignore_ascii_case(repo_name) && b.is_dir() {
                    return validate_directory(b);
                }
            }
        } else if let Ok(cwd) = std::env::current_dir() {
            if let Some(folder_name) = cwd.file_name().and_then(|n| n.to_str()) {
                if folder_name.eq_ignore_ascii_case(repo_name) && cwd.is_dir() {
                    return validate_directory(&cwd);
                }
            }
        }
    }

    // Busca em ~/src/...
    if let Some(ref home) = resolved_home {
        let src_root = home.join("src");

        // ~/src/<clean> (ex.: ~/src/acme/widget ou ~/src/pult)
        let c1 = src_root.join(clean);
        if c1.exists() {
            return validate_directory(&c1);
        }

        // ~/src/<repo_name> (ex.: input lumen/lauda -> ~/src/lauda)
        if owner.is_some() {
            let c2 = src_root.join(repo_name);
            if c2.exists() {
                return validate_directory(&c2);
            }
        }
    }

    // Busca no base_dir se fornecido
    if let Some(b) = base_dir {
        let c3 = b.join(clean);
        if c3.exists() {
            return validate_directory(&c3);
        }

        if owner.is_some() {
            let c4 = b.join(repo_name);
            if c4.exists() {
                return validate_directory(&c4);
            }
        }
    }

    // Busca no current_dir se base_dir for None
    if base_dir.is_none() {
        if let Ok(cwd) = std::env::current_dir() {
            let c5 = cwd.join(clean);
            if c5.exists() {
                return validate_directory(&c5);
            }

            if owner.is_some() {
                let c6 = cwd.join(repo_name);
                if c6.exists() {
                    return validate_directory(&c6);
                }
            }
        }
    }

    // 5. Consulta a catálogos locais (se disponíveis)
    if let Some(cat_path) = find_in_catalogs(clean, resolved_home.as_deref()) {
        return validate_directory(&cat_path);
    }

    Err(RepoError::NotFound(clean.to_string()))
}

fn validate_directory(path: &Path) -> Result<PathBuf, RepoError> {
    let canonical = path
        .canonicalize()
        .map_err(|e| RepoError::Io(e.to_string()))?;
    if !canonical.is_dir() {
        return Err(RepoError::NotADirectory(canonical));
    }
    Ok(canonical)
}

fn find_in_catalogs(name_or_slug: &str, home_dir: Option<&Path>) -> Option<PathBuf> {
    if let Some(home) = home_dir {
        let cat_file = home.join(".config/cccp/catalog.json");
        if cat_file.exists() {
            if let Ok(content) = fs::read_to_string(&cat_file) {
                if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                    for entry in entries {
                        let slug = entry.get("slug").and_then(|s| s.as_str()).unwrap_or("");
                        let name = entry.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        if slug.eq_ignore_ascii_case(name_or_slug)
                            || name.eq_ignore_ascii_case(name_or_slug)
                        {
                            if let Some(p) = entry.get("path").and_then(|p| p.as_str()) {
                                let path = PathBuf::from(p);
                                if path.is_dir() {
                                    return Some(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
