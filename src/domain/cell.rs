use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue<'a> {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(Cow<'a, str>),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
}