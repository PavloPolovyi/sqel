use std::io::{BufWriter, ErrorKind, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::domain::{CellValue, QueryResult};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use clap::{Args, ValueHint, ValueEnum};
use futures::StreamExt;
use tablestream::{Column, Stream};

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Csv,
    Tsv,
    Jsonl
}

#[derive(Args, Debug, Clone)]
pub struct OutputArgs {
    /// Output format
    #[arg(long, value_enum)]
    pub output: Option<OutputFormat>,

    /// Write output to a file instead of stdout
    #[arg(long, value_name = "PATH", value_hint=ValueHint::FilePath)]
    pub out: Option<PathBuf>,

    /// Omit header row (CSV/TSV)
    #[arg(long)]
    pub no_headers: bool,

    /// Disable pagination of result (Table)
    #[arg(long)]
    pub no_pager: bool,
}

pub trait CellDisplay {
    fn to_string_value(&self) -> String;
    fn to_json_value(&self) -> serde_json::Value;
}

impl CellDisplay for CellValue {
    fn to_string_value(&self) -> String {
        match &self {
            CellValue::Null => "NULL".to_string(),
            CellValue::Bool(b) => if *b { "☑" } else { "☐" }.to_string(),
            CellValue::Int(i) => i.to_string(),
            CellValue::Float(f) => f.to_string(),
            CellValue::Decimal(d) => d.to_string(),
            CellValue::Text(s) => s.to_string(),
            CellValue::Bytes(b) => format!("<{} bytes>", b.len()),
            CellValue::Json(j) => j.to_string()
        }
    }

    fn to_json_value(&self) ->  serde_json::Value {
        match &self {
            CellValue::Null => serde_json::Value::Null,
            CellValue::Bool(b) => serde_json::Value::Bool(*b),
            CellValue::Int(i) => serde_json::json!(i),
            CellValue::Float(f) => serde_json::json!(f),
            CellValue::Decimal(d) => serde_json::json!(d),
            CellValue::Text(s) => serde_json::Value::String(s.to_string()),
            CellValue::Bytes(b) => serde_json::Value::String(BASE64.encode(b)),
            CellValue::Json(map) => map.clone()
        }
    }
}

pub struct OutputWriter {
    pub output: OutputFormat,
    pub out: Option<PathBuf>,
    pub no_headers: bool,
    pub no_pager: bool
}

impl OutputWriter {
    pub fn new(output: OutputFormat, out: Option<PathBuf>, no_headers: bool, no_pager: bool) -> Self {
        OutputWriter { output, out, no_headers, no_pager }
    }

    pub async fn write(&self, result: QueryResult<'_>) -> anyhow::Result<()> {
        match &self.out {
            Some(path) => {
                let file = std::fs::File::create(path)?;
                let mut writer = BufWriter::with_capacity(64 * 1024, file);
                self.write_to(&mut writer, result).await
            }
            None => {
                let stdout = std::io::stdout();
                let mut writer = BufWriter::with_capacity(64 * 1024, stdout.lock());
                self.write_to(&mut writer, result).await
            }
        }
    }

    async fn write_to<W: Write>(&self, writer: &mut W, result: QueryResult<'_>) -> anyhow::Result<()> {
        match &self.output {
            OutputFormat::Table => self.render_table_stream(writer, result).await,
            OutputFormat::Csv   => self.render_delimited_stream(writer, result, b',').await,
            OutputFormat::Tsv   => self.render_delimited_stream(writer, result, b'\t').await,
            OutputFormat::Jsonl => self.render_jsonl_stream(writer, result).await,
        }
    }

    async fn render_table_stream<W: Write>(&self, writer: &mut W, result: QueryResult<'_>) -> anyhow::Result<()> {
        let pager = if self.out.is_none() && std::io::stdout().is_terminal() && !self.no_pager {
            self.spawn_pager()
        } else {
            None
        };

        if let Some(mut pager) = pager {
            let stdin = pager.stdin.as_mut()
                .ok_or_else(|| anyhow::anyhow!("Error accessing pager process"))?;
            self.stream_to_tablestream(stdin, result, true).await?;
            pager.wait()?;
        } else {
            let mut writer = BufWriter::with_capacity(64 * 1024, writer);
            self.stream_to_tablestream(&mut writer, result, self.out.is_some()).await?;
        }

        Ok(())
    }

    fn spawn_pager(&self) -> Option<std::process::Child> {
        let mut less = Command::new("less");
        less.args(["-S", "-F", "-R"]);
        less.stdin(Stdio::piped()).spawn().ok()
            .or_else(|| Command::new("more").stdin(Stdio::piped()).spawn().ok())
    }

    async fn stream_to_tablestream<W: Write>(&self, writer: &mut W, mut result: QueryResult<'_>, paged: bool) -> anyhow::Result<()> {
        let columns: Vec<Column<Vec<CellValue>>> = result.headers.iter()
            .enumerate()
            .map(|(i, name)| {
                Column::new(move |f, row: &Vec<CellValue>| write!(f, "{}", row[i].to_string_value()))
                    .header(name.as_str())
            })
            .collect();
        let mut stream = Stream::new(writer, columns);
        if paged {
            stream = stream.max_width(10_000).grow(false);
        }
        while let Some(row) = result.stream.next().await {
            match stream.row(row?) {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::BrokenPipe => break,
                Err(e) => return Err(e.into()),
            }
        }
        match stream.finish() {
            Err(e) if e.kind() == ErrorKind::BrokenPipe => {}
            other => other?,
        }
        Ok(())
    }

    async fn render_delimited_stream<W: Write>(&self, writer: &mut W, mut result: QueryResult<'_>, delimiter: u8) -> anyhow::Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(writer);
        if !self.no_headers {
            wtr.write_record(result.headers)?;
        }

        while let Some(row) = result.stream.next().await {
            wtr.write_record(row?.iter().map(|c| c.to_string_value()))?;
        }

        wtr.flush()?;
        Ok(())
    }

    async fn render_jsonl_stream<W: Write>(&self, writer: &mut W, mut result: QueryResult<'_>) -> anyhow::Result<()> {
        while let Some(row) = result.stream.next().await {
            let obj: serde_json::Map<String, serde_json::Value> = result.headers.iter()
                .zip(row?.iter())
                .map(|(h, v)| (h.to_string(), v.to_json_value()))
                .collect();
            serde_json::to_writer(&mut *writer, &obj)?;
            writer.write_all(b"\n")?;
        }

        writer.flush()?;
        Ok(())
    }
}
