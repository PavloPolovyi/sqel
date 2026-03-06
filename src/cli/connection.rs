use std::path::PathBuf;
use std::str::FromStr;
use clap::{Args, ArgGroup, Subcommand, ValueHint};
use url::Url;
use crate::cli::output::{OutputArgs};
use crate::domain::{ConnectionName, DriverType};

#[derive(Args, Debug, Clone)]
pub struct ConnectionRefArgs {
    /// Name of connection (uses default if omitted)
    pub name: Option<ConnectionName>,
}

#[derive(Args, Debug)]
#[command(about = "Create, list, update, and remove named database connections.", arg_required_else_help = true)]
pub struct ConnectionCommand {
    #[command(subcommand)]
    pub command: ConnectionSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ConnectionSubcommand {
    /// Create and save a new database connection
    Add(AddConnectionArgs),
    /// List all saved database connections
    #[command(alias = "ls")]
    List(ListArgs),
    /// Remove database connection
    #[command(alias = "rm")]
    Remove {
        /// Name of connection to remove
        name: ConnectionName,
    },
    /// Test database connection
    Test {
        #[command(flatten)]
        conn: ConnectionRefArgs,
        /// Connection test timeout in seconds
        #[arg(short = 't', long, default_value_t = 10)]
        timeout: u64,
    },
    /// Set connection as default
    SetDefault {
        /// Name of connection to set as default
        name: ConnectionName,
    },
    /// Unset default connection
    UnsetDefault,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[command(flatten)]
    pub output: OutputArgs,
}

#[derive(Args, Debug)]
#[command(
    about = "Create a new named database connection.",
    long_about = "Creates and stores a reusable database connection configuration.\n\n\
You must provide a driver subcommand:\n\
  - url <URL>       -- use a full connection URL\n\
  - postgres|mysql  -- specify individual connection parameters\n\
  - sqlite          -- use a local file path\n\n\
You can overwrite existing connection by passing --overwrite flag.\n\
Passwords are stored using OS keychain."
)]
pub struct AddConnectionArgs {
    /// Unique connection name used to reference this connection in other commands
    pub name: ConnectionName,

    #[command(subcommand)]
    pub driver: DriverSubcommand,
}

#[derive(Args, Debug)]
pub struct AddConnectionOptions {
    /// Do not test the connection after saving
    #[arg(long)]
    pub no_test: bool,

    /// Overwrite an existing connection with the same name
    #[arg(long)]
    pub overwrite: bool,

    /// Connection test timeout in seconds (only used when testing is enabled)
    #[arg(short = 't', long, default_value_t = 10, conflicts_with = "no_test")]
    pub timeout: u64,

    /// Set connection as default
    #[arg(long)]
    pub set_default: bool,

    /// Extra connection options (repeatable), e.g. --param ssl=true --param connect_timeout=10
    #[arg(long = "param", value_name = "KEY=VALUE", value_parser = parse_key_val)]
    pub params: Vec<(String, String)>,
}

#[derive(Subcommand, Debug)]
pub enum DriverSubcommand {
    /// Connect to PostgreSQL
    Postgres(PostgresArgs),
    /// Connect to MySQL
    Mysql(MysqlArgs),
    /// Connect to SQLite
    Sqlite(SqliteArgs),
}

#[derive(Args, Debug)]
pub struct PostgresArgs {
    #[command(flatten)]
    pub network: NetworkConnectionArgs,

    #[command(flatten)]
    pub options: AddConnectionOptions,

    #[command(subcommand)]
    pub auth: Option<AuthSubcommand>,
}

#[derive(Args, Debug)]
pub struct MysqlArgs {
    #[command(flatten)]
    pub network: NetworkConnectionArgs,

    #[command(flatten)]
    pub options: AddConnectionOptions,

    #[command(subcommand)]
    pub auth: Option<AuthSubcommand>,
}

#[derive(Args, Debug)]
pub struct SqliteArgs {
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub path: PathBuf,
    #[command(flatten)]
    pub options: AddConnectionOptions,
}

/// Connection parameters for network-based databases (e.g. PostgreSQL, MySQL).
///
/// Supports two mutually exclusive modes:
/// - A full connection URL via `--url`
/// - Individual parameters: `--host`, `--port`, `--db`, `--user`
#[derive(Args, Debug)]
pub struct NetworkConnectionArgs {
    /// Full database connection URL, (mutually exclusive with --host, --db, --user, --port)
    #[arg(long, conflicts_with_all = ["host", "db", "user", "port"])]
    pub url: Option<DatabaseUrl>,

    /// Database server hostname or IP address
    #[arg(short = 'H', long, required_unless_present = "url")]
    pub host: Option<String>,

    /// Server port (defaults to the standard port for the selected database)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Database name
    #[arg(short = 'd', long, required_unless_present = "url")]
    pub db: Option<String>,

    /// Username for authentication
    #[arg(short, long, required_unless_present = "url")]
    pub user: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum AuthSubcommand {
    /// Auth configuration (optional). If omitted, defaults depend on driver.
    Auth(AuthArgs),
}

#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub method: AuthMethod,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum AuthMethod {
    /// Username/password authentication (password stored in keychain)
    Password(PasswordAuthArgs),
    /// No authentication
    None,
}

#[derive(Args, Debug, PartialEq)]
pub struct PasswordAuthArgs {
    #[command(flatten)]
    pub secret: SecretSourceArgs
}

#[derive(Args, Debug, Clone, PartialEq)]
#[command(group(
    ArgGroup::new("secret_source")
        .args(["stdin", "env"])
        .multiple(false)
))]
pub struct SecretSourceArgs {
    /// Read secret securely from standard input
    #[arg(long)]
    pub stdin: bool,

    /// Read secret from an environment variable
    #[arg(long, value_name = "ENV_VAR")]
    pub env: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DatabaseUrl(Url);

impl DatabaseUrl {
    pub fn url(&self) -> &Url {
        &self.0
    }

    pub fn password(&self) -> Option<String> {
        self.0.password().map(|p| p.to_string())
    }

    pub fn driver_type(&self) -> DriverType {
        DriverType::from_scheme(&self.0.scheme()).unwrap()
    }
}

impl FromStr for DatabaseUrl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s).map_err(|e| e.to_string())?;
        let scheme = url.scheme();
        DriverType::from_scheme(scheme).ok_or_else(|| {
            format!(
                "'{}' is not supported. Supported drivers: {}.",
                scheme,
                DriverType::supported_schemes_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        Ok(DatabaseUrl(url))
    }
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| "expected KEY=VALUE string".to_string())?;
    let k = k.trim();
    if k.is_empty() {
        return Err("Key cannot be empty".to_string());
    }
    Ok((k.to_string(), v.to_string()))
}
