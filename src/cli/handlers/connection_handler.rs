use anyhow::Context;
use crate::app::{AddConnectionRequest, ConnectionService};
use crate::cli::output::OutputFormat;
use crate::cli::connection::{AddConnectionArgs, DriverSubcommand, AuthMethod, AuthSubcommand, ListArgs, NetworkConnectionArgs, DatabaseUrl};
use crate::cli::console::{Console};
use crate::domain::{AuthMode, CellValue, ConnectionKind, ConnectionName, DriverType};
use crate::cli::output::OutputWriter;

pub async fn handle_add(console: &Console, app: &ConnectionService, args: AddConnectionArgs) -> anyhow::Result<()> {
    let (resolved_connection, options) = match args.driver {
        DriverSubcommand::Postgres(arg) => {
            (process_network_driver(DriverType::Postgres, &arg.network, &arg.auth, &args.name, console)?, arg.options)
        }
        DriverSubcommand::Mysql(arg) => {
            (process_network_driver(DriverType::Mysql, &arg.network, &arg.auth, &args.name, console)?, arg.options)
        }
        DriverSubcommand::Sqlite(arg) => {
            (ResolvedConnection {
                driver: DriverType::Sqlite,
                kind: ConnectionKind::Sqlite { path: arg.path },
                auth_mode: AuthMode::None, password: None
            }, arg.options)
        }
    };
    let req = AddConnectionRequest::new(
        args.name.clone(),
        resolved_connection.driver,
        resolved_connection.kind,
        resolved_connection.auth_mode,
        resolved_connection.password,
        options.params.into_iter().collect(),
        options.set_default,
        options.overwrite
    )?;
    let warnings = app.add(req)?;
    for w in &warnings {
        console.warn(&w.to_string())
    }
    if !options.no_test {
        if let Err(e) = handle_test(console, app,
                                   Some(args.name.clone()), options.timeout).await {
            console.error(&format!("Connection test failed: {}", e));
        }
    }
    console.success(&format!("Connection '{}' saved.", &args.name));
    Ok(())
}

fn process_network_args(driver: DriverType, args: &NetworkConnectionArgs) -> anyhow::Result<(ConnectionKind, Option<String>)> {
    let kind = network_args_to_connection_kind(args, driver.default_port().unwrap())?;
    let mut password = None;
    if let Some(url) = &args.url {
        let url_driver = url.driver_type();
        if url_driver != driver {
            anyhow::bail!("Driver '{}' in url doesn't match selected driver '{}'", url_driver, driver);
        }
        password = url.password();
    }
    Ok((kind, password))
}

fn process_auth(
    auth_command: &Option<AuthSubcommand>,
    has_url_password: bool,
    name: &ConnectionName,
    console: &Console,
) -> anyhow::Result<(AuthMode, Option<String>)> {
    let auth_args = match auth_command {
        None => {
            let password = if !has_url_password {
                Some(console.prompt_secret(&format!("Password for '{}'", name))?)
            } else {
                None
            };
            return Ok((AuthMode::Password, password));
        }
        Some(AuthSubcommand::Auth(args)) => args,
    };

    let mode = auth_method_to_auth_mode(&auth_args.method);
    if mode == AuthMode::None {
        if has_url_password {
            anyhow::bail!("password provided in URL conflicts with auth none");
        }
        return Ok((AuthMode::None, None));
    }

    let password = match &auth_args.method {
        AuthMethod::Password(pass) => {
            if has_url_password && (pass.secret.stdin || pass.secret.env.is_some()) {
                anyhow::bail!("password provided in URL conflicts with --stdin/--env");
            }

            if let Some(env_var) = &pass.secret.env {
                Some(read_password_env(env_var)?)
            } else if pass.secret.stdin {
                Some(read_password_stdin()?)
            } else if !has_url_password {
                Some(console.prompt_secret(&format!("Password for '{}'", name))?)
            } else {
                None
            }
        }
        _ => unreachable!(),
    };

    Ok((mode, password))
}
fn process_network_driver(driver: DriverType,
                          args: &NetworkConnectionArgs,
                          auth_command: &Option<AuthSubcommand>, name: &ConnectionName, console: &Console) -> anyhow::Result<ResolvedConnection> {
    let (kind, mut password) = process_network_args(driver, args)?;

    let (auth_mode, resolved_password) = process_auth(auth_command, password.is_some(), name, console)?;
    password = password.or(resolved_password);

    Ok(ResolvedConnection {driver, kind, auth_mode, password})
}

fn read_password_stdin() -> anyhow::Result<String> {
    let mut password = String::new();
    std::io::stdin().read_line(&mut password)?;
    Ok(password.trim_end().to_string())
}

fn read_password_env(var: &str) -> anyhow::Result<String> {
    std::env::var(var)
        .with_context(|| format!("Failed to read password from env variable '{}'", var))
}

pub fn handle_remove(console: &Console, app: &ConnectionService, name: ConnectionName) -> anyhow::Result<()> {
    let res = app.remove(&name)?;
    for warn in res {
        console.warn(&warn.to_string())
    }
    console.success(&format!("Connection '{}' removed.", name));
    Ok(())
}

pub fn handle_set_default(console: &Console, app: &ConnectionService, name: ConnectionName) -> anyhow::Result<()> {
    app.set_default(&name)?;
    console.success(&format!("Connection '{}' set as default.", name));
    Ok(())
}

pub fn handle_unset_default(console: &Console, app: &ConnectionService) -> anyhow::Result<()> {
    match app.unset_default()? {
        Some(name) => console.success(&format!("Default connection '{}' unset.", name)),
        None => console.info("No default connection was set."),
    }
    Ok(())
}

pub fn handle_list(app: &ConnectionService, args: &ListArgs) -> anyhow::Result<()> {
    let res = app.list()?;

    let headers = ["", "NAME", "DRIVER", "LOCATION", "USER", "AUTH", "PARAMS"];
    let default_name = res.default_connection;

    let rows: Vec<Vec<CellValue>> = res
        .connections
        .iter()
        .map(|c| {
            let is_default = default_name.as_ref() == Some(c.name());
            let marker = if is_default { "*" } else { "" };

            let user: String = match c.kind() {
                ConnectionKind::Network { user, .. } => user.as_str().into(),
                ConnectionKind::Sqlite { .. } => "-".into(),
            };

            let auth: String = match c.auth() {
                AuthMode::None => "none".into(),
                AuthMode::Password => format!("password ({})", c.credential_storage()),
            };

            let mut params: String = c.params().iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<String>>()
                .join(",");
            if params.is_empty() {
                params = "-".to_string();
            }

            vec![
                CellValue::Text(marker.into()),
                CellValue::Text(c.name().as_str().into()),
                CellValue::Text(c.driver().to_string().into()),
                CellValue::Text(c.location().into()),
                CellValue::Text(user),
                CellValue::Text(auth),
                CellValue::Text(params.into())
            ]
        })
        .collect();

    let writer = OutputWriter::new(
        args.output.output.unwrap_or(OutputFormat::Table),
        args.output.out.clone(),
        args.output.no_headers,
    );

    writer.write_table(&headers, &rows)?;
    Ok(())
}

pub async fn handle_test(console: &Console, app: &ConnectionService, 
                   name: Option<ConnectionName>, timeout: u64) -> anyhow::Result<()> {
    app.test(name, timeout, console).await?;
    console.success("Connected successfully.");
    Ok(())
}

struct ResolvedConnection {
    driver: DriverType,
    kind: ConnectionKind,
    auth_mode: AuthMode,
    password: Option<String>
}

fn network_args_to_connection_kind(args: &NetworkConnectionArgs, default_port: u16) -> anyhow::Result<ConnectionKind> {
    match &args.url {
        None => {
            Ok(ConnectionKind::Network {
                host: args.host.clone().unwrap(),
                port: args.port.unwrap_or(default_port),
                db: args.db.clone().unwrap(),
                user: args.user.clone().unwrap(),
            })
        }
        Some(url) => db_url_to_connection_kind(url, default_port)
    }
}
fn db_url_to_connection_kind(url: &DatabaseUrl, default_port: u16) -> anyhow::Result<ConnectionKind> {
    let url = url.url();
    if url.username().is_empty() {
        return Err(anyhow::anyhow!("missing username in url"));
    }
    Ok(
        ConnectionKind::Network {
            host: url.host_str()
                .ok_or_else(|| anyhow::anyhow!("missing host in url"))?
                .to_string(),
            port: url.port().unwrap_or(default_port),
            db: url.path().trim_start_matches('/').to_string(),
            user: url.username().to_string()
        }
    )
}
fn auth_method_to_auth_mode(method: &AuthMethod) -> AuthMode {
    match method {
        AuthMethod::Password(_) => AuthMode::Password,
        AuthMethod::None => AuthMode::None
    }
}
