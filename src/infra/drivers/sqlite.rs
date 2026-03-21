use std::time::Duration;
use sqlx::sqlite::{SqliteConnectOptions, SqliteRow};
use sqlx::{Column, Connection, Row, SqliteConnection, TypeInfo, ValueRef};
use crate::domain::{CellValue, Connection as Conn};

pub struct SqliteDriver {
    connection: SqliteConnection
}

impl SqliteDriver {
    pub async fn connect(connection: &Conn) -> Result<Self, anyhow::Error> {
        let path = connection.kind().as_sqlite()
            .ok_or(anyhow::anyhow!("expected sqlite connection"))?;

        let mut options = SqliteConnectOptions::new()
            .filename(path);

        for (key, value) in connection.params() {
            options = match key.as_str() {
                "journal_mode" => options.journal_mode(value.parse()?),
                "busy_timeout" => options.busy_timeout(Duration::from_secs(value.parse()?)),
                "read_only" => options.read_only(value.parse()?),
                "create_if_missing" => options.create_if_missing(value.parse()?),
                "foreign_keys" => options.foreign_keys(value.parse()?),
                k if k.starts_with("pragma.") => options.pragma(k["pragma.".len()..].to_owned(), value.to_owned()),
                _ => return Err(anyhow::anyhow!("unsupported sqlite parameter: {}", key))
            }
        }

        Ok(Self { connection: SqliteConnection::connect_with(&options).await? })
    }
}

fn decode_sqlite_row(row: &SqliteRow) -> Result<Vec<CellValue>, sqlx::Error> {
    let mut out = Vec::with_capacity(row.len());
    for (i, col) in row.columns().iter().enumerate() {
        let value_ref = row.try_get_raw(i)?;
        let value = if value_ref.is_null() {
            CellValue::Null
        } else {
            match col.type_info().name() {
                "BOOLEAN"  => CellValue::Bool(row.try_get::<bool, _>(i)?),
                "INTEGER"  => CellValue::Int(row.try_get::<i64, _>(i)?),
                "REAL"     => CellValue::Float(row.try_get::<f64, _>(i)?),
                "BLOB"     => CellValue::Bytes(row.try_get::<Vec<u8>, _>(i)?),
                "DATETIME" => CellValue::Text(row.try_get::<chrono::NaiveDateTime, _>(i)?.to_string()),
                "DATE"     => CellValue::Text(row.try_get::<chrono::NaiveDate, _>(i)?.to_string()),
                "TIME"     => CellValue::Text(row.try_get::<chrono::NaiveTime, _>(i)?.to_string()),
                _          => CellValue::Text(row.try_get::<String, _>(i)?),
            }
        };
        out.push(value);
    }
    Ok(out)
}

impl_sqlx_driver!(SqliteDriver, decode_sqlite_row);
