use crate::domain::{AuthMode, Connection, DriverType};
use crate::infra::drivers::mysql::MySqlDriver;
use crate::infra::drivers::postgres::PostgresDriver;
use crate::infra::drivers::sqlite::SqliteDriver;
use crate::ports::Driver;

#[macro_use]
mod macros;
mod postgres;
mod mysql;
mod sqlite;

pub async fn connect(connection: &Connection, password: Option<String>) -> Result<Box<dyn Driver>, anyhow::Error> {
    match (connection.auth(), &password) {
        (AuthMode::Password, None) => return Err(anyhow::anyhow!("password required")),
        (AuthMode::None, Some(_)) => return Err(anyhow::anyhow!("password provided but auth mode is none")),
        _ => {}
    }

    let password = password.as_deref();

    match connection.driver() {
        DriverType::Postgres => Ok(Box::new(PostgresDriver::connect(connection, password).await?)),
        DriverType::Mysql => Ok(Box::new(MySqlDriver::connect(connection, password).await?)),
        DriverType::Sqlite => Ok(Box::new(SqliteDriver::connect(connection).await?)),
    }
}
