/// Generates a `Driver` impl for any sqlx-based backend.
///
/// Parameters:
///   $driver – the driver struct name (e.g., PostgresDriver)
///   $decode_fn – the row→CellValue function (e.g., decode_pg_row)
macro_rules! impl_sqlx_driver {
    ($driver:ident, $decode_fn:path) => {
        #[async_trait::async_trait]
        impl crate::ports::Driver for $driver {
            //    Streams rows one by one, decoding each with $decode_fn.
            //
            //    Why the "first-row" dance?
            //    We need column metadata (headers) before we can start
            //    yielding rows. sqlx only exposes column info *on* a Row,
            //    so we pull the first row, read headers from it, decode it,
            //    then chain it back onto the rest of the stream.
            async fn query<'a>(
                &'a mut self,
                sql: &'a str,
            ) -> Result<
                crate::domain::QueryResult<'a>,
                crate::domain::DatabaseError,
            > {
                use futures::{stream, StreamExt};
                use sqlx::Column;

                let mut raw_stream = sqlx::query(sql)
                    .fetch(&mut self.connection);

                let first = match raw_stream.next().await {
                    Some(Ok(row)) => row,
                    Some(Err(e)) => return Err(Box::new(e)),
                    None => return Ok(crate::domain::QueryResult {
                        headers: vec![],
                        stream: Box::pin(stream::empty()),
                    }),
                };

                let headers: Vec<String> = first.columns()
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect();

                let first_decoded = $decode_fn(&first)
                    .map_err(|e| Box::new(e) as crate::domain::DatabaseError)?;

                let stream = stream::once(async move {
                    Ok(first_decoded)
                }).chain(raw_stream.map(|r| match r {
                    Ok(row) => $decode_fn(&row).map_err(|e| Box::new(e) as crate::domain::DatabaseError),
                    Err(e) => Err(Box::new(e) as crate::domain::DatabaseError),
                }));

                Ok(crate::domain::QueryResult {
                    headers,
                    stream: Box::pin(stream),
                })
            }

            async fn execute(&mut self, sql: &str, ) -> Result<u64, crate::domain::DatabaseError> {
                let result = sqlx::query(sql)
                    .execute(&mut self.connection)
                    .await?;
                Ok(result.rows_affected())
            }
        }
    };
}
