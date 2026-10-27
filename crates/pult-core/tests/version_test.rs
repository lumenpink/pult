use pult_core::{canonical_version, short_version};

#[test]
fn test_canonical_version_is_valid_semver() {
    let ver = canonical_version();
    assert!(!ver.is_empty(), "canonical_version cannot be empty");
    assert!(
        ver.starts_with("0.1.0"),
        "expected version to start with 0.1.0, got: {}",
        ver
    );
}

#[test]
fn test_short_version_cuts_at_plus_character() {
    let short = short_version();
    assert!(
        !short.contains('+'),
        "short_version must not contain '+', got: {}",
        short
    );
    let canonical = canonical_version();
    let expected = canonical.split('+').next().unwrap();
    assert_eq!(short, expected);
}
