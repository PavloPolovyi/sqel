use async_trait::async_trait;
use crate::domain::{QueryResult};

#[async_trait]
pub trait Driver: Send {
    async fn query<'a>(&'a mut self, sql: &'a str) -> Result<QueryResult<'a>, anyhow::Error>;

    async fn execute(&mut self, sql: &str) -> Result<u64, anyhow::Error>;
}
