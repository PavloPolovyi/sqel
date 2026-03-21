use crate::app::ConnectionService;
use crate::cli::Console;
use crate::cli::output::{OutputFormat, OutputWriter};
use crate::cli::query::QueryCommand;

pub async fn handle_query(console: &Console, app: &ConnectionService, args: &QueryCommand) -> anyhow::Result<()> {
    let mut driver = app.connect(args.conn.clone(), console).await?;
    if let Some(_) = &args.file {
        todo!()
    } else if let Some(q) = &args.query {
        if is_read_query(q) {
            let result = driver.query(q).await?;
            let writer = OutputWriter::new(
                args.output.output.unwrap_or(OutputFormat::Table),
                args.output.out.clone(),
                args.output.no_headers,
                args.output.no_pager
            );
            writer.write(result).await?;
        } else {
            let affected_rows = driver.execute(q).await?;
            console.success(&format!("{} rows affected", affected_rows));
        }
    }
    Ok(())
}

fn is_read_query(sql: &str) -> bool {
    let normalized = strip_leading_comments(sql);
    let upper = normalized.trim_start().to_uppercase();

    upper.starts_with("SELECT")
        || (upper.starts_with("WITH") && cte_is_read_only(&normalized))
        || upper.starts_with("EXPLAIN")
        || upper.starts_with("SHOW")
        || upper.starts_with("DESCRIBE")
        || upper.starts_with("DESC ")
        || upper.starts_with("PRAGMA")
        || upper.starts_with("TABLE")
        || upper.starts_with("VALUES")
}

fn strip_leading_comments(sql: &str) -> &str {
    let mut s = sql.trim_start();
    loop {
        if s.starts_with("--") {
            s = s.find('\n').map_or("", |i| &s[i + 1..]).trim_start();
        } else if s.starts_with("/*") {
            s = s.find("*/").map_or("", |i| &s[i + 2..]).trim_start();
        } else {
            break;
        }
    }
    s
}

fn cte_is_read_only(sql: &str) -> bool {
    let upper = sql.to_uppercase();
    let mut depth = 0i32;
    let mut last_keyword_is_select = true;
    for word in upper.split_whitespace() {
        match word {
            w if w.contains('(') => depth += w.matches('(').count() as i32 - w.matches(')').count() as i32,
            w if w.contains(')') => depth += w.matches('(').count() as i32 - w.matches(')').count() as i32,
            "SELECT" if depth == 0 => last_keyword_is_select = true,
            "INSERT" | "UPDATE" | "DELETE" | "MERGE" if depth == 0 => last_keyword_is_select = false,
            _ => {}
        }
    }
    last_keyword_is_select
}
