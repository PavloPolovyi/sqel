use sqlx::postgres::{PgConnectOptions, PgRow};
use sqlx::postgres::types::PgInterval;
use sqlx::{Column, Connection, PgConnection, Row, TypeInfo, ValueRef};
use crate::domain::{CellValue, Connection as Conn};

pub struct PostgresDriver {
    connection: PgConnection,
}

impl PostgresDriver {
    pub async fn connect(connection: &Conn, password: Option<&str>) -> Result<Self, anyhow::Error> {
        let (host, port, db, user) = connection
            .kind().as_network().ok_or(anyhow::anyhow!("expected network connection"))?;

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
                "NUMERIC"    => CellValue::Decimal(row.try_get::<rust_decimal::Decimal, _>(i)?),
                "BYTEA"      => CellValue::Bytes(row.try_get::<Vec<u8>, _>(i)?),
                "UUID"       => CellValue::Text(row.try_get::<uuid::Uuid, _>(i)?.to_string()),
                "TIMESTAMPTZ"=> CellValue::Text(row.try_get::<chrono::DateTime<chrono::Utc>, _>(i)?.to_string()),
                "TIMESTAMP"  => CellValue::Text(row.try_get::<chrono::NaiveDateTime, _>(i)?.to_string()),
                "DATE"       => CellValue::Text(row.try_get::<chrono::NaiveDate, _>(i)?.to_string()),
                "TIME"       => CellValue::Text(row.try_get::<chrono::NaiveTime, _>(i)?.to_string()),
                "INTERVAL"   => CellValue::Text(format_pg_interval(&row.try_get::<PgInterval, _>(i)?)),
                "INET" | "CIDR" => CellValue::Text(row.try_get::<ipnetwork::IpNetwork, _>(i)?.to_string()),
                "MACADDR"    => CellValue::Text(row.try_get::<mac_address::MacAddress, _>(i)?.to_string()),
                _ => match row.try_get::<String, _>(i) {
                    Ok(s) => CellValue::Text(s),
                    Err(_) => match raw.as_str() {
                        Ok(s) => CellValue::Text(s.to_owned()),
                        Err(_) => CellValue::Bytes(raw.as_bytes()
                            .map_err(|e| sqlx::Error::ColumnDecode {
                                index: col.name().to_string(),
                                source: e,
                            })?.to_vec()),
                    },
                },
            }
        };

        out.push(value);
    }

    Ok(out)
}

fn format_pg_interval(interval: &PgInterval) -> String {
    let mut parts = Vec::new();
    let years = interval.months / 12;
    let months = interval.months % 12;
    if years != 0 { parts.push(format!("{years} yr")); }
    if months != 0 { parts.push(format!("{months} mon")); }
    if interval.days != 0 { parts.push(format!("{} day", interval.days)); }

    let total_us = interval.microseconds;
    let hours = total_us / 3_600_000_000;
    let mins = (total_us % 3_600_000_000) / 60_000_000;
    let secs = (total_us % 60_000_000) / 1_000_000;
    let us = total_us % 1_000_000;

    if hours != 0 || mins != 0 || secs != 0 || us != 0 {
        if us != 0 {
            parts.push(format!("{hours:02}:{mins:02}:{secs:02}.{us:06}"));
        } else {
            parts.push(format!("{hours:02}:{mins:02}:{secs:02}"));
        }
    }

    if parts.is_empty() { "00:00:00".to_string() } else { parts.join(" ") }
}

impl_sqlx_driver!(PostgresDriver, decode_pg_row);
