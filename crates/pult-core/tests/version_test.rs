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


#[test]
fn test_mutation_version_string_trimming_and_format() {
    let canonical = canonical_version();
    assert_eq!(canonical, canonical.trim(), "canonical_version must be trimmed");
    assert!(!canonical.contains('\n'), "canonical_version must not contain newlines");
    assert!(!canonical.contains('\r'), "canonical_version must not contain carriage returns");

    let short = short_version();
    assert!(!short.is_empty(), "short_version must not be empty");
    assert_eq!(short, short.trim(), "short_version must be trimmed");
}
