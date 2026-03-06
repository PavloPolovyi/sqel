mod cli;
mod domain;
mod infra;
mod ports;
mod app;

use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = cli::Cli::parse();
    if let Err(e) = cli::run(args).await {
        let console = cli::Console::new();
        console.error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
