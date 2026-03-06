use crate::domain::Config;

pub trait ConfigStore: Send + Sync {
    fn load(&self) -> anyhow::Result<Config>;
    fn save(&self, cfg: &Config) -> anyhow::Result<()>;
}
