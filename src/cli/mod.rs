mod args;
mod connection;
mod handlers;
mod console;
mod output;

pub use args::Cli;
pub use console::Console;
use crate::app::ConnectionService;
use crate::cli::connection::ConnectionSubcommand;
use crate::infra::{FsConfigStore, KeychainSecretStore};

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let console = Console::new();
    let secret_store = KeychainSecretStore::new("sqlz".to_string());
    let config_store = FsConfigStore::new_default()?;
    let conn_app = ConnectionService::new(Box::new(secret_store), Box::new(config_store));

    match cli.command {
        args::Command::Conn(conn_sub) => match conn_sub.command {
            ConnectionSubcommand::Add(args) => handlers::handle_add(&console, &conn_app, args),
            ConnectionSubcommand::List(args) => handlers::handle_list(&conn_app, &args),
            ConnectionSubcommand::Remove { name } => handlers::handle_remove(&console, &conn_app, name),
            ConnectionSubcommand::Test { conn, timeout } => handlers::handle_test(&console, &conn_app, conn.name, timeout),
            ConnectionSubcommand::SetDefault { name } => handlers::handle_set_default(&console, &conn_app, name),
            ConnectionSubcommand::UnsetDefault => handlers::handle_unset_default(&console, &conn_app),
        },
    }
}