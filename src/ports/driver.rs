use crate::domain::{DatabaseError, QueryResult};

pub trait Driver {
    async fn query<'a>(&'a mut self, sql: &'a str) -> Result<QueryResult<'a>, DatabaseError>;
}