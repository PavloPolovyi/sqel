use std::pin::Pin;
use futures::Stream;

pub type DatabaseError = Box<dyn std::error::Error + Send + Sync>;

pub struct QueryResult<'a> {
    pub headers: Vec<String>,
    pub stream: Pin<Box<dyn Stream<Item = Result<Vec<CellValue>, DatabaseError>> + Send + 'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
}