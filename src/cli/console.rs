use std::io::{IsTerminal};
use crate::ports::{CredentialError, CredentialProvider};

pub struct Console {
    pub stdout_color: bool,
    pub stderr_color: bool,
    pub interactive: bool
}

impl Console {
    const GREEN: &str = "\x1b[92m";
    const BLUE: &str = "\x1b[94m";
    const YELLOW: &str = "\x1b[93m";
    const RED: &str = "\x1b[91m";
    const RESET: &str = "\x1b[0m";

    pub fn new() -> Self {
        let stdout_tty = std::io::stdout().is_terminal();
        let stderr_tty = std::io::stderr().is_terminal();
        let no_color = std::env::var("NO_COLOR").is_ok();
        Console {
            stdout_color: stdout_tty && !no_color,
            stderr_color: stderr_tty && !no_color,
            interactive: std::io::stdin().is_terminal(),
        }
    }

    pub fn success(&self, msg: &str) {
        if self.stdout_color {
            println!("{}✓{} {msg}", Self::GREEN, Self::RESET)
        }
        else {
            println!("OK: {msg}")
        }
    }

    pub fn info(&self, msg: &str) {
        if self.stdout_color {
            println!("{}ℹ{} {msg}", Self::BLUE, Self::RESET);
        } else {
            println!("INFO: {msg}");
        }
    }

    pub fn warn(&self, msg: &str) {
        if self.stderr_color {
            eprintln!("{}⚠{} {msg}", Self::YELLOW, Self::RESET)
        }
        else {
            eprintln!("WARN: {msg}");
        }
    }

    pub fn error(&self, msg: &str) {
        if self.stderr_color {
            eprintln!("{}❌ ERROR{} {msg}", Self::RED, Self::RESET);
        }
        else {
            eprintln!("ERROR: {msg}");
        }
    }

    pub fn prompt_secret(&self, prompt: &str) -> anyhow::Result<String> {
        if !self.interactive {
            anyhow::bail!("secret input required but no interactive terminal available. Use --stdin or --env");
        }
        eprint!("{prompt}: ");
        Ok(rpassword::read_password()?)
    }
}

impl CredentialProvider for Console {
    fn get_secret(&self, prompt: &str) -> Result<String, CredentialError> {
        self.prompt_secret(prompt)
            .map_err(|e| CredentialError::IoError(e.to_string()))
    }
}
