use std::collections::BTreeMap;
use std::time::Duration;
use crate::domain::{AuthMode, Connection, ConnectionKind, ConnectionName, CredentialStorage, DriverType};
use crate::ports::{ConfigStore, CredentialProvider, SecretStore};

pub struct ConnectionService {
    secret_store: Box<dyn SecretStore>,
    config_store: Box<dyn ConfigStore>
}

impl ConnectionService {
    pub fn new(secret_store: Box<dyn SecretStore>, config_store: Box<dyn ConfigStore>) -> Self {
        ConnectionService {secret_store, config_store}
    }

    pub fn add(&self, req: AddConnectionRequest) -> anyhow::Result<Vec<ConnectionWarning>> {
        let mut warnings: Vec<ConnectionWarning> = Vec::new();
        let mut config = self.config_store.load()?;

        if req.overwrite {
            if let Some(existing) = config.get(&req.name) {
                if existing.credential_storage() == CredentialStorage::KeyStore {
                    let _ = self.secret_store.delete(req.name.as_str());
                }
            }
        }

        let credential_storage: CredentialStorage = match req.auth_mode {
            AuthMode::Password => {
                let password = req.password.unwrap();
                match self.secret_store.set(req.name.as_str(), &password) {
                    Ok(_) => CredentialStorage::KeyStore,
                    Err(e) => {
                        warnings.push(ConnectionWarning::KeychainFailed(e.to_string()));
                        CredentialStorage::Prompt
                    }
                }
            },
            AuthMode::None => CredentialStorage::None
        };

        let conn = Connection::new(req.name, req.driver, req.kind, req.auth_mode, credential_storage, req.params);
        let name = conn.name().clone();
        let stored_secret = conn.credential_storage() == CredentialStorage::KeyStore;
        let save_result = (|| -> anyhow::Result<()> {
            config.add(conn, req.set_default, req.overwrite)?;
            self.config_store.save(&config)
        })();
        if let Err(e) = save_result {
            if stored_secret {
                let _ = self.secret_store.delete(name.as_str());
            }
            return Err(e)
        }

        Ok(warnings)
    }

    pub fn list(&self) -> anyhow::Result<ListResult> {
        let config = self.config_store.load()?;
        Ok(ListResult::new(config.get_default().cloned(), config.list().cloned().collect()))
    }

    pub fn remove(&self, name: &ConnectionName) -> anyhow::Result<Vec<ConnectionWarning>> {
        let mut config = self.config_store.load()?;
        let connection = config.remove(name)?;
        self.config_store.save(&config)?;
        let mut warnings: Vec<ConnectionWarning> = Vec::new();
        if connection.credential_storage() == CredentialStorage::KeyStore {
            if let Err(err) = self.secret_store.delete(connection.name().as_str()) {
                warnings.push(ConnectionWarning::KeychainFailed(err.to_string()));
            }
        }
        Ok(warnings)
    }

    pub fn set_default(&self, name: &ConnectionName) -> anyhow::Result<()> {
        let mut config = self.config_store.load()?;
        config.set_default(&name)?;
        self.config_store.save(&config)?;
        Ok(())
    }

    pub fn unset_default(&self) -> anyhow::Result<Option<ConnectionName>> {
        let mut config = self.config_store.load()?;
        let option = config.unset_default();
        self.config_store.save(&config)?;
        Ok(option)
    }

    pub async fn test(&self, name: Option<ConnectionName>, timeout: u64,
                credentials_provider: &dyn CredentialProvider) -> anyhow::Result<()> {
        let mut driver = self.connect(name, credentials_provider).await?;
        match tokio::time::timeout(Duration::from_secs(timeout), driver.test()).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(anyhow::anyhow!("test failed: {}", e)),
            Err(_) =>Err(anyhow::anyhow!("timeout testing connection"))
        }
    }

    pub async fn connect(&self, name: Option<ConnectionName>, credentials_provider: &dyn CredentialProvider) -> anyhow::Result<Box<dyn crate::ports::Driver>> {
        let config = self.config_store.load()?;
        let name = name
            .or_else(|| config.get_default().cloned())
            .ok_or_else(|| anyhow::anyhow!("no default connection"))?;
        let connection = config.get(&name).ok_or_else(|| anyhow::anyhow!("connection not found"))?;
        let password = match connection.credential_storage() {
            CredentialStorage::None => None,
            CredentialStorage::KeyStore => {
                self.secret_store.get(connection.name().as_str())?
            }
            CredentialStorage::Prompt => {
                Some(credentials_provider.get_secret(&format!("Password for '{}'", name))?)
            }
        };
        crate::infra::drivers::connect(&connection, password).await
            .map_err(|e| anyhow::anyhow!("failed to connect: {}", e))
    }
}

#[derive(Debug)]
pub struct ListResult {
    pub default_connection: Option<ConnectionName>,
    pub connections: Vec<Connection>
}

impl ListResult {
    pub fn new(default_connection: Option<ConnectionName>, connections: Vec<Connection>) -> Self {
        ListResult {default_connection, connections}
    }
}


#[derive(Debug)]
pub struct AddConnectionRequest {
    name: ConnectionName,
    driver: DriverType,
    kind: ConnectionKind,
    auth_mode: AuthMode,
    password: Option<String>,
    params: BTreeMap<String, String>,
    set_default: bool,
    overwrite: bool
}

impl AddConnectionRequest {
    pub fn new(name: ConnectionName,
               driver: DriverType,
               kind: ConnectionKind,
               auth_mode: AuthMode,
               password: Option<String>,
               params: BTreeMap<String, String>,
               set_default: bool,
               overwrite: bool) -> anyhow::Result<Self> {
        match (&auth_mode, &password) {
            (AuthMode::Password, None) => anyhow::bail!("password is required for password authentication"),
            (AuthMode::None, Some(_)) => anyhow::bail!("password is provided, but auth mode is none"),
            _ => {}
        }

        if matches!(&kind, ConnectionKind::Sqlite {..}) && auth_mode != AuthMode::None {
            anyhow::bail!("SQLite connections doesn't support authentication")
        }

        Ok(AddConnectionRequest {name, driver, kind, auth_mode, password, params, set_default, overwrite})
    }
}

#[derive(Debug)]
pub enum ConnectionWarning {
    KeychainFailed(String),
    ConnectionTestFailed(String),
}

impl std::fmt::Display for ConnectionWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionWarning::KeychainFailed(msg) => write!(f, "Keychain error: {msg}"),
            ConnectionWarning::ConnectionTestFailed(msg) => write!(f, "Connection test failed: {msg}"),
        }
    }
}
