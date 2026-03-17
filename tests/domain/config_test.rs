use std::collections::BTreeMap;
use std::str::FromStr;
use sqel::domain::*;

fn conn_name(name: &str) -> ConnectionName {
    ConnectionName::from_str(name).unwrap()
}

fn make_connection(name: &str) -> Connection {
    Connection::new(
        conn_name(name),
        DriverType::Postgres,
        ConnectionKind::Network {
            host: "localhost".into(),
            port: 5432,
            db: "testdb".into(),
            user: "admin".into(),
        },
        AuthMode::None,
        CredentialStorage::None,
        BTreeMap::new(),
    )
}

#[test]
fn add_to_empty_config() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), false, false).unwrap();
    assert_eq!(config.list().count(), 1);
}

#[test]
fn add_duplicate_without_overwrite_fails() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), false, false).unwrap();
    let err = config.add(make_connection("dev"), false, false).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn add_duplicate_with_overwrite_succeeds() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), false, false).unwrap();
    config.add(make_connection("dev"), false, true).unwrap();
    assert_eq!(config.list().count(), 1);
}

#[test]
fn add_with_set_default() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), true, false).unwrap();
    assert_eq!(config.get_default(), Some(&conn_name("dev")));
}

#[test]
fn add_without_set_default() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), false, false).unwrap();
    assert_eq!(config.get_default(), None);
}

#[test]
fn remove_existing_connection() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), false, false).unwrap();
    let removed = config.remove(&conn_name("dev")).unwrap();
    assert_eq!(removed.name().as_str(), "dev");
    assert_eq!(config.list().count(), 0);
}

#[test]
fn remove_nonexistent_connection_fails() {
    let mut config = Config::empty();
    let err = config.remove(&conn_name("nope")).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn remove_default_clears_default() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), true, false).unwrap();
    config.remove(&conn_name("dev")).unwrap();
    assert_eq!(config.get_default(), None);
}

#[test]
fn remove_non_default_preserves_default() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), true, false).unwrap();
    config.add(make_connection("staging"), false, false).unwrap();
    config.remove(&conn_name("staging")).unwrap();
    assert_eq!(config.get_default(), Some(&conn_name("dev")));
}

#[test]
fn get_existing_connection() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), false, false).unwrap();
    let name = conn_name("dev");
    assert!(config.get(&name).is_some());
}

#[test]
fn get_nonexistent_returns_none() {
    let config = Config::empty();
    let name = conn_name("nope");
    assert!(config.get(&name).is_none());
}

#[test]
fn set_default_valid_connection() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), false, false).unwrap();
    config.set_default(&conn_name("dev")).unwrap();
    assert_eq!(config.get_default(), Some(&conn_name("dev")));
}

#[test]
fn set_default_nonexistent_fails() {
    let mut config = Config::empty();
    let err = config.set_default(&conn_name("nope")).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn set_default_overrides_previous() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), true, false).unwrap();
    config.add(make_connection("prod"), false, false).unwrap();
    config.set_default(&conn_name("prod")).unwrap();
    assert_eq!(config.get_default(), Some(&conn_name("prod")));
}

#[test]
fn unset_default_returns_previous() {
    let mut config = Config::empty();
    config.add(make_connection("dev"), true, false).unwrap();
    let prev = config.unset_default();
    assert_eq!(prev, Some(conn_name("dev")));
    assert_eq!(config.get_default(), None);
}

#[test]
fn unset_default_when_none_returns_none() {
    let mut config = Config::empty();
    assert_eq!(config.unset_default(), None);
}

#[test]
fn list_returns_all_connections() {
    let mut config = Config::empty();
    config.add(make_connection("alpha"), false, false).unwrap();
    config.add(make_connection("beta"), false, false).unwrap();
    config.add(make_connection("gamma"), false, false).unwrap();
    assert_eq!(config.list().count(), 3);
}
