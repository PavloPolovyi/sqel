pub trait CredentialProvider: Send + Sync {
    fn get_secret(&self, prompt: &str) -> Result<String, CredentialError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("credential input failed: {0}")]
    IoError(String),
}
