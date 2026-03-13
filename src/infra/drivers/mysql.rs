use sqlx::{MySqlConnection};
use sqlx::mysql::{MySqlConnectOptions, MySqlRow};
use sqlx::{Column, Connection, Row, TypeInfo, ValueRef};
use crate::domain::{Connection as Conn, DatabaseError};

pub struct MySqlDriver {
    connection: MySqlConnection
}

impl MySqlDriver {
    pub async fn connect(connection: &Conn, password: &str) -> Result<Self, DatabaseError> {
        let (host, port, db, user) = connection
            .kind().as_network().ok_or("expected network connection")?;

        let options = MySqlConnectOptions::new()
            .host(host)
            .port(port)
            .username(user)
            .database(db)
            .password(password)
            .options(connection.params());
        Ok(Self { connection: MySqlConnection::connect_with(&options).await? })
    }
}