use pult_core::repo::{resolve_repo_path, resolve_repo_path_with_env, RepoError};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_resolve_current_directory_dot_and_empty() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

    let resolved_dot = resolve_repo_path(".", Some(&base)).expect("failed to resolve '.'");
    assert_eq!(resolved_dot, base);

    let resolved_empty =
        resolve_repo_path("", Some(&base)).expect("failed to resolve empty string");
    assert_eq!(resolved_empty, base);
}

#[test]
fn test_resolve_absolute_existing_path() {
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("my-project");
    fs::create_dir_all(&repo_dir).unwrap();
    let canonical_repo = repo_dir.canonicalize().unwrap();

    let resolved = resolve_repo_path(canonical_repo.to_str().unwrap(), None)
        .expect("should resolve absolute path");
    assert_eq!(resolved, canonical_repo);
}

#[test]
fn test_resolve_relative_path() {
    let temp = tempdir().unwrap();
    let repo_dir = temp.path().join("nested").join("project");
    fs::create_dir_all(&repo_dir).unwrap();
    let base = temp.path().canonicalize().unwrap();

    let resolved =
        resolve_repo_path("nested/project", Some(&base)).expect("should resolve relative path");
    assert_eq!(resolved, repo_dir.canonicalize().unwrap());
}

#[test]
fn test_resolve_home_tilde_path() {
    let temp = tempdir().unwrap();
    let home_dir = temp.path().join("fake_home");
    let proj_dir = home_dir.join("src").join("awesome-app");
    fs::create_dir_all(&proj_dir).unwrap();
    let canonical_proj = proj_dir.canonicalize().unwrap();

    let resolved = resolve_repo_path_with_env("~/src/awesome-app", None, Some(&home_dir))
        .expect("should expand ~ and resolve");
    assert_eq!(resolved, canonical_proj);
}

#[test]
fn test_resolve_owner_slash_repo_in_home_src() {
    let temp = tempdir().unwrap();
    let home_dir = temp.path().join("fake_home_owner");
    let proj_dir = home_dir.join("src").join("my-cool-repo");
    fs::create_dir_all(&proj_dir).unwrap();
    let canonical_proj = proj_dir.canonicalize().unwrap();

    let resolved = resolve_repo_path_with_env("lumenpink/my-cool-repo", None, Some(&home_dir))
        .expect("should find my-cool-repo in ~/src");
    assert_eq!(resolved, canonical_proj);
}

#[test]
fn test_resolve_owner_slash_repo_in_nested_home_src() {
    let temp = tempdir().unwrap();
    let home_dir = temp.path().join("fake_home_nested");
    let proj_dir = home_dir.join("src").join("acme").join("widget");
    fs::create_dir_all(&proj_dir).unwrap();
    let canonical_proj = proj_dir.canonicalize().unwrap();

    let resolved = resolve_repo_path_with_env("acme/widget", None, Some(&home_dir))
        .expect("should find acme/widget in ~/src");
    assert_eq!(resolved, canonical_proj);
}

#[test]
fn test_resolve_simple_repo_name_in_base_dir() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();
    let proj_dir = base.join("joaju");
    fs::create_dir_all(&proj_dir).unwrap();
    let canonical_proj = proj_dir.canonicalize().unwrap();

    let resolved =
        resolve_repo_path("joaju", Some(&base)).expect("should find joaju under base_dir");
    assert_eq!(resolved, canonical_proj);
}

#[test]
fn test_resolve_simple_repo_name_matches_base_dir_itself() {
    let temp = tempdir().unwrap();
    let proj_dir = temp.path().join("joaju");
    fs::create_dir_all(&proj_dir).unwrap();
    let canonical_proj = proj_dir.canonicalize().unwrap();

    let resolved = resolve_repo_path("joaju", Some(&canonical_proj))
        .expect("should recognize base dir itself if folder matches repo name");
    assert_eq!(resolved, canonical_proj);
}

#[test]
fn test_resolve_not_found_error() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

    let err = resolve_repo_path("non_existent_project_xyz", Some(&base)).unwrap_err();
    match err {
        RepoError::NotFound(target) => {
            assert_eq!(target, "non_existent_project_xyz");
        }
        other => panic!("expected NotFound, got: {:?}", other),
    }
}

#[test]
fn test_resolve_not_a_directory_error() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();
    let file_path = base.join("regular_file.txt");
    fs::write(&file_path, "hello").unwrap();

    let err = resolve_repo_path("regular_file.txt", Some(&base)).unwrap_err();
    match err {
        RepoError::NotADirectory(path) => {
            assert_eq!(path, file_path.canonicalize().unwrap());
        }
        other => panic!("expected NotADirectory, got: {:?}", other),
    }
}

#[test]
fn test_mutation_whitespace_and_casing_tolerance() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();
    let proj_dir = base.join("MyAwesomeProject");
    fs::create_dir_all(&proj_dir).unwrap();
    let canonical = proj_dir.canonicalize().unwrap();

    // Com espaços nas bordas e caixa diferente
    let resolved = resolve_repo_path("  myawesomeproject  ", Some(&base))
        .expect("should handle whitespace and case-insensitive matching");
    assert_eq!(resolved, canonical);

    // Whitespace puro deve resolver para base_dir
    let resolved_ws = resolve_repo_path("   \t  \n ", Some(&base))
        .expect("whitespace only should resolve to base_dir");
    assert_eq!(resolved_ws, base);
}

#[test]
fn test_mutation_tilde_alone_expands_to_home() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("fake_home_alone");
    fs::create_dir_all(&home).unwrap();
    let canonical_home = home.canonicalize().unwrap();

    let resolved = resolve_repo_path_with_env("~", None, Some(&canonical_home))
        .expect("~ alone should expand to home");
    assert_eq!(resolved, canonical_home);
}

#[test]
fn test_mutation_catalog_resolution() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("fake_home_cat");
    let cccp_dir = home.join(".config").join("cccp");
    fs::create_dir_all(&cccp_dir).unwrap();

    let repo_target = temp.path().join("external_repo");
    fs::create_dir_all(&repo_target).unwrap();
    let canonical_target = repo_target.canonicalize().unwrap();

    let catalog_json = serde_json::json!([
        {
            "name": "superapp",
            "slug": "lumenpink/superapp",
            "path": canonical_target.to_str().unwrap()
        }
    ]);
    fs::write(cccp_dir.join("catalog.json"), catalog_json.to_string()).unwrap();

    // Resolução por slug do catálogo
    let res_slug = resolve_repo_path_with_env("lumenpink/superapp", None, Some(&home))
        .expect("should find via catalog slug");
    assert_eq!(res_slug, canonical_target);

    // Resolução por nome do catálogo
    let res_name = resolve_repo_path_with_env("superapp", None, Some(&home))
        .expect("should find via catalog name");
    assert_eq!(res_name, canonical_target);
}
