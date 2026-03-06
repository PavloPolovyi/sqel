use std::fmt::{Display, Formatter};
use clap::ValueEnum;
use serde::{Serialize, Deserialize};

#[derive(ValueEnum, Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum DriverType {
    Postgres,
    Mysql,
    Sqlite
}

impl DriverType {
    const SCHEMES: &'static [(&'static str, DriverType)] = &[
        ("postgresql", DriverType::Postgres),
        ("postgres", DriverType::Postgres),
        ("mysql", DriverType::Mysql),
        ("sqlite", DriverType::Sqlite),
    ];

    pub fn from_scheme(scheme: &str) -> Option<Self> {
        Self::SCHEMES.iter().find_map(|(s, driver)| (scheme == *s).then_some(*driver))
    }

    pub fn supported_schemes_iter() -> impl Iterator<Item = &'static str> {
        Self::SCHEMES.iter().map(|(s, _)| *s)
    }

    pub fn default_port(&self) -> Option<u16> {
        match self {
            DriverType::Postgres => Some(5432),
            DriverType::Mysql => Some(3306),
            DriverType::Sqlite => None,
        }
    }
}

impl Display for DriverType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverType::Postgres => f.write_str("postgres"),
            DriverType::Mysql => f.write_str("mysql"),
            DriverType::Sqlite => f.write_str("sqlite"),
        }
    }
}
