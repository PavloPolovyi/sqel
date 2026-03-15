use sqlx::{Column, MySqlConnection, TypeInfo, ValueRef};
use sqlx::mysql::{MySqlConnectOptions, MySqlRow};
use sqlx::{Connection, Row};
use crate::domain::{CellValue, Connection as Conn, DatabaseError};

pub struct MySqlDriver {
    connection: MySqlConnection
}

impl MySqlDriver {
    pub async fn connect(connection: &Conn, password: Option<&str>) -> Result<Self, DatabaseError> {
        let (host, port, db, user) = connection
            .kind().as_network().ok_or("expected network connection")?;

        let mut options = MySqlConnectOptions::new()
            .host(host)
            .port(port)
            .username(user)
            .database(db);

        if let Some(password) = password {
            options = options.password(password);
        }

        for (key, value) in connection.params() {
            options = match key.as_str() {
                "ssl_mode" => options.ssl_mode(value.parse()
                        .map_err(|e| format!("invalid SSL mode: {}", e))?),
                "ssl_ca" => options.ssl_ca(value),
                "ssl_client_cert" => options.ssl_client_cert(value),
                "ssl_client_key" => options.ssl_client_key(value),
                "charset" => options.charset(value),
                "collation" => options.collation(value),
                _ => return Err(DatabaseError::from(format!("unsupported mysql parameter: {}", key)))
            }
        }

        Ok(Self { connection: MySqlConnection::connect_with(&options).await? })
    }
}

fn decode_mysql_row(row: &MySqlRow) -> Result<Vec<CellValue>, sqlx::Error> {
    let mut out = Vec::with_capacity(row.len());
    for (i, col) in row.columns().iter().enumerate() {
        let value_ref = row.try_get_raw(i)?;
        let value = if value_ref.is_null() {
            CellValue::Null
        } else {
            match col.type_info().name() {
                "BOOLEAN"  => CellValue::Bool(row.try_get::<bool, _>(i)?),

                "TINYINT"  => CellValue::Int(row.try_get::<i8,  _>(i)? as i64),
                "SMALLINT" => CellValue::Int(row.try_get::<i16, _>(i)? as i64),
                "INT" | "MEDIUMINT" => CellValue::Int(row.try_get::<i32, _>(i)? as i64),
                "BIGINT"   => CellValue::Int(row.try_get::<i64, _>(i)?),

                "TINYINT UNSIGNED"  => CellValue::Int(row.try_get::<u8,  _>(i)? as i64),
                "SMALLINT UNSIGNED" => CellValue::Int(row.try_get::<u16, _>(i)? as i64),
                "INT UNSIGNED" | "MEDIUMINT UNSIGNED" => CellValue::Int(row.try_get::<u32, _>(i)? as i64),
                "BIGINT UNSIGNED" => CellValue::Int(row.try_get::<u64, _>(i)? as i64),

                "FLOAT"   => CellValue::Float(row.try_get::<f32, _>(i)? as f64),
                "DOUBLE"  => CellValue::Float(row.try_get::<f64, _>(i)?),
                "DECIMAL"  => CellValue::Text(row.try_get::<String, _>(i)?),

                "TIMESTAMP" | "DATETIME" => CellValue::Text(
                    row.try_get::<chrono::NaiveDateTime, _>(i)?.to_string()),
                "DATE" => CellValue::Text(
                    row.try_get::<chrono::NaiveDate, _>(i)?.to_string()),
                "TIME" => CellValue::Text(
                    row.try_get::<chrono::NaiveTime, _>(i)?.to_string()),

                "JSON" => CellValue::Json(row.try_get::<serde_json::Value, _>(i)?),

                "BINARY" | "VARBINARY"
                | "TINYBLOB" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB"
                    => CellValue::Bytes(row.try_get::<Vec<u8>, _>(i)?),

                _ => CellValue::Text(row.try_get::<String, _>(i)?),
            }
        };
        out.push(value);
    }
    Ok(out)
}

impl_sqlx_driver!(MySqlDriver, decode_mysql_row);
