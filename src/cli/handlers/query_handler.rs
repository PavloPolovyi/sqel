use anyhow::Context;
use crate::app::ConnectionService;
use crate::cli::Console;
use crate::cli::output::{OutputFormat, OutputWriter};
use crate::cli::query::QueryCommand;

pub async fn handle_query(console: &Console, app: &ConnectionService, args: &QueryCommand) -> anyhow::Result<()> {
    let mut driver = app.connect(args.conn.clone(), console).await?;
    if let Some(path) = &args.file {
        let content = std::fs::read_to_string(path)?;
        let queries = split_statements(&content);
        for query in queries.iter().filter(|q| !strip_leading_comments(q).is_empty()) {
            let affected_rows = driver.execute(query).await
                .with_context(|| format!("Failed to execute query: {}", query))?;
            if affected_rows == 0 {
                console.success(&format!("{} ... OK", truncate_query(query)));
            } else {
                console.success(&format!("{} ... OK ({} rows affected)", truncate_query(query), affected_rows));
            }
        }
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

fn split_statements(sql: &str) -> Vec<&str> {
    let mut stmts = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    let bytes = sql.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'$' => {
                let tag_start = i;
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'$' {
                    let tag = &sql[tag_start..=i];
                    i += 1;
                    while i + tag.len() <= bytes.len() {
                        if sql[i..].starts_with(tag) { i += tag.len() - 1; break; }
                        i += 1;
                    }
                } else {
                    continue;
                }
            }
            b'\'' | b'"' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote { i += 1; }
            }
            b'-' if bytes.get(i + 1) == Some(&b'-') => {
                while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') { i += 1; }
                i += 1;
            }
            b';' if depth <= 0 => {
                let stmt = sql[start..i].trim();
                if !stmt.is_empty() { stmts.push(stmt); }
                start = i + 1;
            }
            _ if keyword_at(sql, i, "BEGIN") => { depth += 1; i += 4; }
            _ if keyword_at(sql, i, "END")   => { depth -= 1; i += 2; }
            _ => {}
        }
        i += 1;
    }

    let tail = sql[start..].trim();
    if !tail.is_empty() { stmts.push(tail); }
    stmts
}

fn keyword_at(sql: &str, pos: usize, kw: &str) -> bool {
    let bytes = sql.as_bytes();
    let end = pos + kw.len();
    if end > bytes.len() || !sql[pos..end].eq_ignore_ascii_case(kw) { return false; }
    let before = if pos > 0 { bytes[pos - 1] } else { b' ' };
    let after = if end < bytes.len() { bytes[end] } else { b' ' };
    !before.is_ascii_alphanumeric() && before != b'_'
        && !after.is_ascii_alphanumeric() && after != b'_'
}

fn truncate_query(sql: &str) -> String {
    strip_leading_comments(sql).split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
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
