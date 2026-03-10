use futures::{stream, StreamExt};
use sqlx::{TypeInfo};
use sqlx::{Column, Connection, PgConnection, Row};
use sqlx::postgres::{PgConnectOptions, PgRow};
use sqlx::{ValueRef};
use crate::domain::{CellValue, Connection as Conn, DatabaseError, QueryResult};
use crate::ports::Driver;

pub struct PostgresDriver {
    connection: PgConnection
}

impl PostgresDriver {
    pub async fn connect(connection: &Conn, password: &str) -> Result<Self, DatabaseError> {
        let (host, port, db, user) = connection
            .kind().as_network().ok_or("expected network connection")?;

        let options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(user)
            .database(db)
            .password(password)
            .options(connection.params());
        Ok(Self {connection: PgConnection::connect_with(&options).await?})
    }
}


impl Driver for PostgresDriver {
    async fn query<'a>(&'a mut self, sql: &'a str) -> Result<QueryResult<'a>, DatabaseError> {
        let mut raw_stream = sqlx::query(sql)
            .fetch(&mut self.connection);

        let first = match raw_stream.next().await {
            Some(Ok(row)) => row,
            Some(Err(e)) => return Err(Box::new(e)),
            None => return Ok(QueryResult {
                headers: vec![],
                stream: Box::pin(stream::empty()),
            }),
        };

        let headers: Vec<String> = first.columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        let first_decoded = decode_pg_row(&first)
            .map_err(|e| Box::new(e) as DatabaseError)?;

        let stream = stream::once(async move {
            Ok(first_decoded)
        }).chain(raw_stream.map(|r| match r {
                Ok(row) => decode_pg_row(&row).map_err(|e| Box::new(e) as DatabaseError),
                Err(e) => Err(Box::new(e) as DatabaseError),
            }));

        Ok(QueryResult {
            headers,
            stream: Box::pin(stream),
        })
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
