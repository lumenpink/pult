use pult_core::repo::{resolve_repo_path, RepoError};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_resolve_current_directory_dot_and_empty() {
    let temp = tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();

    let resolved_dot = resolve_repo_path(".", Some(&base)).expect("failed to resolve '.'");
    assert_eq!(resolved_dot, base);

    let resolved_empty = resolve_repo_path("", Some(&base)).expect("failed to resolve empty string");
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

    let resolved = resolve_repo_path("nested/project", Some(&base))
        .expect("should resolve relative path");
    assert_eq!(resolved, repo_dir.canonicalize().unwrap());
}

#[test]
fn test_resolve_home_tilde_path() {
    let temp = tempdir().unwrap();
    let home_dir = temp.path().join("fake_home");
    let proj_dir = home_dir.join("src").join("awesome-app");
    fs::create_dir_all(&proj_dir).unwrap();
    let canonical_proj = proj_dir.canonicalize().unwrap();

    std::env::set_var("HOME", home_dir.to_str().unwrap());

    let resolved = resolve_repo_path("~/src/awesome-app", None)
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

    std::env::set_var("HOME", home_dir.to_str().unwrap());

    let resolved = resolve_repo_path("lumenpink/my-cool-repo", None)
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

    std::env::set_var("HOME", home_dir.to_str().unwrap());

    let resolved = resolve_repo_path("acme/widget", None)
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

    let resolved = resolve_repo_path("joaju", Some(&base))
        .expect("should find joaju under base_dir");
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
