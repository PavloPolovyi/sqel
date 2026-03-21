use std::pin::Pin;
use futures::Stream;

pub struct QueryResult<'a> {
    pub headers: Vec<String>,
    pub stream: Pin<Box<dyn Stream<Item = Result<Vec<CellValue>, anyhow::Error>> + Send + 'a>>,
}

impl QueryResult<'static> {
    pub fn from_rows(headers: Vec<String>, rows: Vec<Vec<CellValue>>) -> Self {
        let stream = futures::stream::iter(rows.into_iter().map(Ok));
        QueryResult {
            headers,
            stream: Box::pin(stream),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Decimal(rust_decimal::Decimal),
    Text(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
}