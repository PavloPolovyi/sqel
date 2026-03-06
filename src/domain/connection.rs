use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::collections::BTreeMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::domain::DriverType;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionName(String);

impl ConnectionName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ConnectionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ConnectionName {
    type Err = ConnectionNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ConnectionNameError::Empty);
        }
        if !s.is_ascii() {
            return Err(ConnectionNameError::NonAscii);
        }
        if s.len() > 32 {
            return Err(ConnectionNameError::TooLong);
        }
        if !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
            return Err(ConnectionNameError::InvalidCharacters);
        }

        Ok(ConnectionName(s.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionNameError {
    #[error("cannot be empty")]
    Empty,
    #[error("only ASCII characters are allowed")]
    NonAscii,
    #[error("may only contain letters, numbers, '-' or '_'")]
    InvalidCharacters,
    #[error("cannot be longer than 32 characters")]
    TooLong
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConnectionKind {
    Network {
        host: String,
        port: u16,
        db: String,
        user: String,
    },
    Sqlite {
        path: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    name: ConnectionName,
    driver: DriverType,
    kind: ConnectionKind,
    auth: AuthMode,
    credential_storage: CredentialStorage,
    #[serde(default)]
    params: BTreeMap<String, String>,
}

impl Connection {
    pub fn new(
        name: ConnectionName,
        driver: DriverType,
        kind: ConnectionKind,
        auth: AuthMode,
        credential_storage: CredentialStorage,
        params: BTreeMap<String, String>
    ) -> Self {
        Connection {name, driver, kind, auth, credential_storage, params}
    }

    pub fn name(&self) -> &ConnectionName {
        &self.name
    }

    pub fn driver(&self) -> DriverType {
        self.driver
    }

    pub fn kind(&self) -> &ConnectionKind {
        &self.kind
    }

    pub fn auth(&self) -> AuthMode {
        self.auth
    }

    pub fn credential_storage(&self) -> CredentialStorage {
        self.credential_storage
    }

    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.params
    }

    pub fn location(&self) -> String {
        match self.kind() {
            ConnectionKind::Network { host, port, db, .. } => format!("{host}:{port}/{db}"),
            ConnectionKind::Sqlite { path } => path.display().to_string()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMode {
    None,
    Password,
    // future: Token, Certificate, Kerberos, etc.
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialStorage {
    None,
    KeyStore,
    Prompt,
}

impl Display for CredentialStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialStorage::None => f.write_str("none"),
            CredentialStorage::KeyStore => f.write_str("keystore"),
            CredentialStorage::Prompt => f.write_str("prompt"),
        }
    }
}
