use keyring::Entry;
use crate::ports::{SecretStore, SecretStoreError};

pub struct KeychainSecretStore {
    service: String
}

impl KeychainSecretStore {
    pub fn new(service: String) -> Self {
        KeychainSecretStore { service }
    }
}

impl SecretStore for KeychainSecretStore {
    fn get(&self, key: &str) -> Result<Option<String>, SecretStoreError> {
        let entry = Entry::new(&self.service, key).map_err(classify_keyring_error)?;
        match entry.get_password() {
            Ok(s) => Ok(Some(s)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(classify_keyring_error(e)),
        }
    }

    fn set(&self, key: &str, secret: &str) -> Result<(), SecretStoreError> {
        let entry = Entry::new(&self.service, key).map_err(classify_keyring_error)?;
        Ok(entry.set_password(secret).map_err(classify_keyring_error)?)
    }

    fn delete(&self, key: &str) -> Result<(),SecretStoreError> {
        let entry = Entry::new(&self.service, key).map_err(classify_keyring_error)?;
        Ok(entry.delete_credential().map_err(classify_keyring_error)?)
    }
}

fn classify_keyring_error(e: keyring::Error) -> SecretStoreError {
    match e {
        keyring::Error::NoEntry => SecretStoreError::NotFound,
        keyring::Error::NoStorageAccess(inner) => SecretStoreError::Unavailable(inner.to_string()),
        keyring::Error::PlatformFailure(inner) => SecretStoreError::Unavailable(inner.to_string()),
        other => SecretStoreError::Other(other.to_string()),
    }
}
