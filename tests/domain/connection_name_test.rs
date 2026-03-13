use sqel::domain::ConnectionName;
use std::str::FromStr;

#[test]
fn valid_alphanumeric_name() {
    let name = ConnectionName::from_str("mydb01").unwrap();
    assert_eq!(name.as_str(), "mydb01");
}

#[test]
fn valid_name_with_hyphens_and_underscores() {
    let name = ConnectionName::from_str("my-db_01").unwrap();
    assert_eq!(name.as_str(), "my-db_01");
}

#[test]
fn valid_single_character() {
    assert!(ConnectionName::from_str("a").is_ok());
}

#[test]
fn valid_max_length_32() {
    let name = "a".repeat(32);
    assert!(ConnectionName::from_str(&name).is_ok());
}

#[test]
fn empty_name_rejected() {
    let err = ConnectionName::from_str("").unwrap_err();
    assert!(err.to_string().contains("empty"));
}

#[test]
fn too_long_name_rejected() {
    let name = "a".repeat(33);
    let err = ConnectionName::from_str(&name).unwrap_err();
    assert!(err.to_string().contains("longer than 32"));
}

#[test]
fn non_ascii_rejected() {
    let err = ConnectionName::from_str("café").unwrap_err();
    assert!(err.to_string().contains("ASCII"));
}

#[test]
fn spaces_rejected() {
    let err = ConnectionName::from_str("my db").unwrap_err();
    assert!(err.to_string().contains("letters, numbers"));
}

#[test]
fn dots_rejected() {
    assert!(ConnectionName::from_str("my.db").is_err());
}

#[test]
fn slashes_rejected() {
    assert!(ConnectionName::from_str("my/db").is_err());
}

#[test]
fn display_matches_value() {
    let name = ConnectionName::from_str("prod-db").unwrap();
    assert_eq!(format!("{name}"), "prod-db");
}
