use async_trait::async_trait;
use crate::domain::{DatabaseError, QueryResult};

#[async_trait]
pub trait Driver: Send {
    async fn query<'a>(&'a mut self, sql: &'a str) -> Result<QueryResult<'a>, DatabaseError>;

    async fn execute(&mut self, sql: &str) -> Result<u64, DatabaseError>;

    async fn test(&mut self) -> Result<(), DatabaseError>;
}