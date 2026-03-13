use sqel::domain::DriverType;

#[test]
fn from_scheme_postgres() {
    assert_eq!(DriverType::from_scheme("postgres"), Some(DriverType::Postgres));
}

#[test]
fn from_scheme_postgresql() {
    assert_eq!(DriverType::from_scheme("postgresql"), Some(DriverType::Postgres));
}

#[test]
fn from_scheme_mysql() {
    assert_eq!(DriverType::from_scheme("mysql"), Some(DriverType::Mysql));
}

#[test]
fn from_scheme_sqlite() {
    assert_eq!(DriverType::from_scheme("sqlite"), Some(DriverType::Sqlite));
}

#[test]
fn from_scheme_unknown_returns_none() {
    assert_eq!(DriverType::from_scheme("oracle"), None);
}

#[test]
fn from_scheme_empty_returns_none() {
    assert_eq!(DriverType::from_scheme(""), None);
}

#[test]
fn default_port_postgres() {
    assert_eq!(DriverType::Postgres.default_port(), Some(5432));
}

#[test]
fn default_port_mysql() {
    assert_eq!(DriverType::Mysql.default_port(), Some(3306));
}

#[test]
fn default_port_sqlite_is_none() {
    assert_eq!(DriverType::Sqlite.default_port(), None);
}

#[test]
fn display_postgres() {
    assert_eq!(format!("{}", DriverType::Postgres), "postgres");
}

#[test]
fn display_mysql() {
    assert_eq!(format!("{}", DriverType::Mysql), "mysql");
}

#[test]
fn display_sqlite() {
    assert_eq!(format!("{}", DriverType::Sqlite), "sqlite");
}

#[test]
fn supported_schemes_contains_all() {
    let schemes: Vec<&str> = DriverType::supported_schemes_iter().collect();
    assert!(schemes.contains(&"postgres"));
    assert!(schemes.contains(&"postgresql"));
    assert!(schemes.contains(&"mysql"));
    assert!(schemes.contains(&"sqlite"));
}
