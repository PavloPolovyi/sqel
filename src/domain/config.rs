use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::domain::connection::{Connection};
use crate::domain::ConnectionName;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    default_connection: Option<ConnectionName>,
    connections: BTreeMap<ConnectionName, Connection>
}

impl Config {
    pub fn empty() -> Self {
        Config {default_connection: None, connections: BTreeMap::new()}
    }

    pub fn add(&mut self, conn: Connection, set_default: bool, overwrite: bool) -> Result<(), ConfigError> {
        let name = conn.name().clone();

        if !overwrite && self.connections.contains_key(&name) {
            return Err(ConfigError::AlreadyExists);
        }

        if set_default {
            self.default_connection = Some(name.clone());
        }
        self.connections.insert(name, conn);

        Ok(())
    }

    pub fn remove(&mut self, name: &ConnectionName) -> Result<Connection, ConfigError> {
        let conn = self.connections.remove(name).ok_or(ConfigError::NotFound)?;
        if self.default_connection.as_ref() == Some(name) {
            self.default_connection = None;
        }
        Ok(conn)
    }

    pub fn list(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }

    pub fn get(&self, name: &ConnectionName) -> Option<&Connection> {
        self.connections.get(name)
    }

    pub fn set_default(&mut self, name: &ConnectionName) -> Result<(), ConfigError> {
        if self.connections.contains_key(name) {
            self.default_connection = Some(name.clone());
            return Ok(());
        }
        Err(ConfigError::NotFound)
    }

    pub fn unset_default(&mut self) -> Option<ConnectionName> {
        self.default_connection.take()
    }

    pub fn get_default(&self) -> Option<&ConnectionName> {
        self.default_connection.as_ref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("connection with such name already exists")]
    AlreadyExists,
    #[error("connection not found")]
    NotFound,
}
