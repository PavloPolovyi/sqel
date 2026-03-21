use std::path::PathBuf;
use clap::{Args, ValueHint, ArgGroup};
use crate::cli::output::OutputArgs;
use crate::domain::ConnectionName;

#[derive(Args, Debug, Clone)]
#[command(group = ArgGroup::new("input").required(true))]
pub struct QueryCommand {
    /// Connection name (uses default if omitted)
    #[arg(short, long)]
    pub conn: Option<ConnectionName>,

    /// Inline SQL query
    #[arg(short = 'q', long, group = "input")]
    pub query: Option<String>,

    /// Path to .sql file
    #[arg(short = 'f', long, group = "input", value_hint = ValueHint::FilePath)]
    pub file: Option<PathBuf>,

    #[command(flatten)]
    pub output: OutputArgs,
}
