use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use sqel::domain::*;

#[test]
fn network_connection_location() {
    let conn = Connection::new(
        ConnectionName::from_str("dev").unwrap(),
        DriverType::Postgres,
        ConnectionKind::Network {
            host: "db.example.com".into(),
            port: 5432,
            db: "mydb".into(),
            user: "admin".into(),
        },
        AuthMode::None,
        CredentialStorage::None,
        BTreeMap::new(),
    );
    assert_eq!(conn.location(), "db.example.com:5432/mydb");
}

#[test]
fn sqlite_connection_location() {
    let conn = Connection::new(
        ConnectionName::from_str("local").unwrap(),
        DriverType::Sqlite,
        ConnectionKind::Sqlite { path: PathBuf::from("/tmp/test.db") },
        AuthMode::None,
        CredentialStorage::None,
        BTreeMap::new(),
    );
    assert_eq!(conn.location(), "/tmp/test.db");
}

#[test]
fn as_network_returns_some_for_network() {
    let kind = ConnectionKind::Network {
        host: "localhost".into(),
        port: 5432,
        db: "testdb".into(),
        user: "admin".into(),
    };
    let (host, port, db, user) = kind.as_network().unwrap();
    assert_eq!(host, "localhost");
    assert_eq!(port, 5432);
    assert_eq!(db, "testdb");
    assert_eq!(user, "admin");
}

#[test]
fn as_network_returns_none_for_sqlite() {
    let kind = ConnectionKind::Sqlite { path: PathBuf::from("/tmp/test.db") };
    assert!(kind.as_network().is_none());
}

#[test]
fn connection_preserves_params() {
    let mut params = BTreeMap::new();
    params.insert("sslmode".into(), "require".into());
    params.insert("timeout".into(), "30".into());

    let conn = Connection::new(
        ConnectionName::from_str("dev").unwrap(),
        DriverType::Postgres,
        ConnectionKind::Network {
            host: "localhost".into(),
            port: 5432,
            db: "testdb".into(),
            user: "admin".into(),
        },
        AuthMode::Password,
        CredentialStorage::KeyStore,
        params,
    );

    assert_eq!(conn.params().len(), 2);
    assert_eq!(conn.params().get("sslmode").unwrap(), "require");
}

#[test]
fn connection_getters() {
    let conn = Connection::new(
        ConnectionName::from_str("prod").unwrap(),
        DriverType::Mysql,
        ConnectionKind::Network {
            host: "db.prod".into(),
            port: 3306,
            db: "app".into(),
            user: "root".into(),
        },
        AuthMode::Password,
        CredentialStorage::Prompt,
        BTreeMap::new(),
    );

    assert_eq!(conn.name().as_str(), "prod");
    assert_eq!(conn.driver(), DriverType::Mysql);
    assert_eq!(conn.auth(), AuthMode::Password);
    assert_eq!(conn.credential_storage(), CredentialStorage::Prompt);
}

#[test]
fn credential_storage_display() {
    assert_eq!(format!("{}", CredentialStorage::None), "none");
    assert_eq!(format!("{}", CredentialStorage::KeyStore), "keystore");
    assert_eq!(format!("{}", CredentialStorage::Prompt), "prompt");
}
