use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use sqel::app::AddConnectionRequest;
use sqel::domain::*;

#[test]
fn valid_network_connection_with_password() {
    let result = AddConnectionRequest::new(
        ConnectionName::from_str("dev").unwrap(),
        DriverType::Postgres,
        ConnectionKind::Network {
            host: "localhost".into(),
            port: 5432,
            db: "testdb".into(),
            user: "admin".into(),
        },
        AuthMode::Password,
        Some("secret".into()),
        BTreeMap::new(),
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn valid_network_connection_no_auth() {
    let result = AddConnectionRequest::new(
        ConnectionName::from_str("dev").unwrap(),
        DriverType::Postgres,
        ConnectionKind::Network {
            host: "localhost".into(),
            port: 5432,
            db: "testdb".into(),
            user: "admin".into(),
        },
        AuthMode::None,
        None,
        BTreeMap::new(),
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn valid_sqlite_connection() {
    let result = AddConnectionRequest::new(
        ConnectionName::from_str("local").unwrap(),
        DriverType::Sqlite,
        ConnectionKind::Sqlite { path: PathBuf::from("/tmp/test.db") },
        AuthMode::None,
        None,
        BTreeMap::new(),
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn password_auth_without_password_fails() {
    let result = AddConnectionRequest::new(
        ConnectionName::from_str("dev").unwrap(),
        DriverType::Postgres,
        ConnectionKind::Network {
            host: "localhost".into(),
            port: 5432,
            db: "testdb".into(),
            user: "admin".into(),
        },
        AuthMode::Password,
        None,
        BTreeMap::new(),
        false,
        false,
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("password is required"));
}

#[test]
fn no_auth_with_password_fails() {
    let result = AddConnectionRequest::new(
        ConnectionName::from_str("dev").unwrap(),
        DriverType::Postgres,
        ConnectionKind::Network {
            host: "localhost".into(),
            port: 5432,
            db: "testdb".into(),
            user: "admin".into(),
        },
        AuthMode::None,
        Some("secret".into()),
        BTreeMap::new(),
        false,
        false,
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("auth mode is none"));
}

#[test]
fn sqlite_with_password_auth_fails() {
    let result = AddConnectionRequest::new(
        ConnectionName::from_str("local").unwrap(),
        DriverType::Sqlite,
        ConnectionKind::Sqlite { path: PathBuf::from("/tmp/test.db") },
        AuthMode::Password,
        Some("secret".into()),
        BTreeMap::new(),
        false,
        false,
    );
    let err = result.unwrap_err();
    assert!(err.to_string().contains("SQLite"));
}
