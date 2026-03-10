use anyhow::Context;
use directories::ProjectDirs;
use std::io::ErrorKind;
use std::path::{PathBuf};
use tempfile::NamedTempFile;

use crate::domain::Config;
use crate::ports::ConfigStore;

pub struct FsConfigStore {
    path: PathBuf,
}

impl FsConfigStore {
    pub fn new_default() -> anyhow::Result<Self> {
        Ok(Self { path: default_config_path()? })
    }
}

impl ConfigStore for FsConfigStore {
    fn load(&self) -> anyhow::Result<Config> {
        let contents = match std::fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Config::empty()),
            Err(e) => {
                return Err(e).with_context(|| format!("Failed to read config file at {}", self.path.display()))
            }
        };

        if contents.trim().is_empty() {
            return Ok(Config::empty());
        }

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse TOML in {}", self.path.display()))
    }

    fn save(&self, config: &Config) -> anyhow::Result<()> {
        let dir = self.path
            .parent()
            .context("Could not resolve config directory")?;

        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create config directory {}", dir.display()))?;

        let mut tmp = NamedTempFile::new_in(dir)
            .with_context(|| format!("Failed to create temporary file in {}", dir.display()))?;

        let config_str = toml::to_string_pretty(config).context("Failed to serialize config")?;

        std::io::Write::write_all(&mut tmp, config_str.as_bytes())
            .with_context(|| format!("Failed to write config temp file in {}", dir.display()))?;

        tmp.as_file().sync_all().context("Failed to sync config file content")?;

        tmp.persist(&self.path)
            .map_err(|e| e.error)
            .with_context(|| format!("Failed to persist config file to {}", self.path.display()))?;

        Ok(())
    }
}

fn default_config_path() -> anyhow::Result<PathBuf> {
    let proj = ProjectDirs::from("com", "sqlz", "sqlz")
        .context("Could not determine config directory")?;
    Ok(proj.config_dir().join("config.toml"))
}
