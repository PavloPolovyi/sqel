use clap::{Parser, Subcommand};
use crate::cli::connection::ConnectionCommand;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "sqel is a cross-engine database command-line tool.\n\
It allows you to create named connections, run queries, export data, and \
perform cross-database diff and copy operations across Postgres, MySQL, \
SQLite, Snowflake, and more."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage saved database connections
    Conn(ConnectionCommand),
}
