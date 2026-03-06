use std::io::Write;
use std::path::PathBuf;
use comfy_table::Table;
use crate::domain::CellValue;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use clap::{Args, ValueHint, ValueEnum};

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
}

pub trait CellDisplay {
    fn to_string_value(&self) -> String;
    fn to_json_value(&self) -> serde_json::Value;
}

impl CellDisplay for CellValue<'_> {
    fn to_string_value(&self) -> String {
        match &self {
            CellValue::Null => "NULL".to_string(),
            CellValue::Bool(b) => b.to_string(),
            CellValue::Int(i) => i.to_string(),
            CellValue::Float(f) => f.to_string(),
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
            CellValue::Text(s) => serde_json::Value::String(s.to_string()),
            CellValue::Bytes(b) => serde_json::Value::String(BASE64.encode(b)),
            CellValue::Json(map) => map.clone()
        }
    }
}

pub struct OutputWriter {
    pub output: OutputFormat,
    pub out: Option<PathBuf>,
    pub no_headers: bool
}

impl OutputWriter {
    pub fn new(output: OutputFormat, out: Option<PathBuf>, no_headers: bool) -> Self {
        OutputWriter { output, out, no_headers }
    }

    pub fn write_table<C: CellDisplay>(&self, headers: &[&str], rows: &[Vec<C>]) -> anyhow::Result<()> {
        match &self.output {
            OutputFormat::Table => self.render_table(headers, rows),
            OutputFormat::Csv => self.render_delimited(headers, rows, b','),
            OutputFormat::Tsv => self.render_delimited(headers, rows, b'\t'),
            OutputFormat::Jsonl => self.render_jsonl(headers, rows)
        }
    }

    fn render_table<C: CellDisplay>(&self, headers: &[&str], rows: &[Vec<C>]) -> anyhow::Result<()> {
        let mut table = Table::new();
        table.set_header(headers);
        for row in rows {
            table.add_row(row.iter().map(|c| c.to_string_value()));
        }
        writeln!(self.writer()?, "{table}")?;
        Ok(())
    }

    fn render_delimited<C: CellDisplay>(
        &self,
        headers: &[&str],
        rows: &[Vec<C>],
        delimiter: u8,
    ) -> anyhow::Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(self.writer()?);
        if !self.no_headers {
            wtr.write_record(headers)?;
        }
        for row in rows {
            wtr.write_record(row.iter().map(|c| c.to_string_value()))?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn render_jsonl<C: CellDisplay>(&self, headers: &[&str], rows: &[Vec<C>]) -> anyhow::Result<()> {
        let mut out = self.writer()?;
        for row in rows {
            let obj: serde_json::Map<String, serde_json::Value> = headers.iter()
                .zip(row.iter())
                .map(|(h, v)| (h.to_string(), v.to_json_value()))
                .collect();
            serde_json::to_writer(&mut out, &obj)?;
            out.write(b"\n")?;
        }
        out.flush()?;
        Ok(())
    }

    fn writer(&self) -> anyhow::Result<Box<dyn Write>> {
        match &self.out {
            Some(path) => Ok(Box::new(std::fs::File::create(&path)?)),
            None => Ok(Box::new(std::io::stdout()))
        }
    }
}
