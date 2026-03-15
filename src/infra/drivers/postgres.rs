use sqlx::postgres::{PgConnectOptions, PgRow};
use sqlx::{Column, Connection, PgConnection, Row, TypeInfo, ValueRef};
use crate::domain::{CellValue, Connection as Conn, DatabaseError};

pub struct PostgresDriver {
    connection: PgConnection,
}

impl PostgresDriver {
    pub async fn connect(connection: &Conn, password: Option<&str>) -> Result<Self, DatabaseError> {
        let (host, port, db, user) = connection
            .kind().as_network().ok_or("expected network connection")?;

        let mut options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(user)
            .database(db)
            .options(connection.params());

        if let Some(password) = password {
            options = options.password(password);
        }

        Ok(Self { connection: PgConnection::connect_with(&options).await? })
    }
}

fn decode_pg_row(row: &PgRow) -> Result<Vec<CellValue>, sqlx::Error> {
    let mut out = Vec::with_capacity(row.len());

    for (i, col) in row.columns().iter().enumerate() {
        let raw = row.try_get_raw(i)?;

        let value = if raw.is_null() {
            CellValue::Null
        } else {
            match col.type_info().name() {
                "BOOL"       => CellValue::Bool(row.try_get::<bool, _>(i)?),
                "INT2"       => CellValue::Int(row.try_get::<i16, _>(i)? as i64),
                "INT4"       => CellValue::Int(row.try_get::<i32, _>(i)? as i64),
                "INT8"       => CellValue::Int(row.try_get::<i64, _>(i)?),
                "FLOAT4"     => CellValue::Float(row.try_get::<f32, _>(i)? as f64),
                "FLOAT8"     => CellValue::Float(row.try_get::<f64, _>(i)?),
                "BYTEA"      => CellValue::Bytes(row.try_get::<Vec<u8>, _>(i)?),
                "UUID"       => CellValue::Text(row.try_get::<uuid::Uuid, _>(i)?.to_string()),
                "TIMESTAMPTZ"=> CellValue::Text(row.try_get::<chrono::DateTime<chrono::Utc>, _>(i)?.to_string()),
                "TIMESTAMP"  => CellValue::Text(row.try_get::<chrono::NaiveDateTime, _>(i)?.to_string()),
                "DATE"       => CellValue::Text(row.try_get::<chrono::NaiveDate, _>(i)?.to_string()),
                "TIME"       => CellValue::Text(row.try_get::<chrono::NaiveTime, _>(i)?.to_string()),
                _            => CellValue::Text(row.try_get::<String, _>(i)?),
            }
        };

        out.push(value);
    }

    Ok(out)
}

impl_sqlx_driver!(PostgresDriver, decode_pg_row);
