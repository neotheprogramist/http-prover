use std::{collections::HashSet, path::PathBuf, sync::Arc};

use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt, sync::Mutex};

#[derive(Debug, Error)]
pub enum AuthorizerError {
    #[error("file read error")]
    FileAccessError(#[from] std::io::Error),
    #[error("invalid file format")]
    FormatError(#[from] serde_json::Error),
}

pub(crate) trait AuthorizationProvider {
    async fn is_authorized(&self, public_key: &str) -> Result<bool, AuthorizerError>;
    async fn authorize(&self, public_key: &str) -> Result<(), AuthorizerError>;
}

#[derive(Debug, Clone)]
pub enum Authorizer {
    Open,
    Persistent(FileAuthorizer),
    ReadOnlyFile(FileAuthorizer, MemoryAuthorizer),
    Memory(MemoryAuthorizer),
}

impl AuthorizationProvider for Authorizer {
    async fn is_authorized(&self, public_key: &str) -> Result<bool, AuthorizerError> {
        Ok(match self {
            Authorizer::Open => true,
            Authorizer::Persistent(authorizer) => authorizer.is_authorized(public_key).await?,
            Authorizer::Memory(authorizer) => authorizer.is_authorized(public_key).await?,
            Authorizer::ReadOnlyFile(file_authorizer, memory_authorizer) => {
                memory_authorizer.is_authorized(public_key).await?
                    || file_authorizer.is_authorized(public_key).await?
            }
        })
    }

    async fn authorize(&self, public_key: &str) -> Result<(), AuthorizerError> {
        match self {
            Authorizer::Open => Ok(()),
            Authorizer::Persistent(authorizer) => authorizer.authorize(public_key).await,
            Authorizer::Memory(authorizer) => authorizer.authorize(public_key).await,
            Authorizer::ReadOnlyFile(_, memory_authorizer) => {
                memory_authorizer.authorize(public_key).await
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileAuthorizer(PathBuf);

impl FileAuthorizer {
    pub async fn new(path: PathBuf) -> Result<Self, AuthorizerError> {
        File::open(&path)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        Ok(Self(path))
    }
}

impl AuthorizationProvider for FileAuthorizer {
    async fn is_authorized(&self, public_key: &str) -> Result<bool, AuthorizerError> {
        let mut file = File::open(&self.0)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let authorized_keys: HashSet<String> = serde_json::from_str::<Vec<String>>(&contents)
            .map_err(AuthorizerError::FormatError)?
            .into_iter()
            .collect();

        Ok(authorized_keys.contains(public_key))
    }

    async fn authorize(&self, public_key: &str) -> Result<(), AuthorizerError> {
        let mut file = File::open(&self.0)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        let mut authorized_keys: HashSet<String> = serde_json::from_str::<Vec<String>>(&contents)
            .map_err(AuthorizerError::FormatError)?
            .into_iter()
            .collect();

        authorized_keys.insert(public_key.into());

        let new_contents =
            serde_json::to_string(&authorized_keys).map_err(AuthorizerError::FormatError)?;

        tokio::fs::write(&self.0, new_contents)
            .await
            .map_err(AuthorizerError::FileAccessError)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MemoryAuthorizer(Arc<Mutex<HashSet<String>>>);

impl AuthorizationProvider for MemoryAuthorizer {
    async fn is_authorized(&self, public_key: &str) -> Result<bool, AuthorizerError> {
        Ok(self.0.lock().await.contains(public_key))
    }

    async fn authorize(&self, public_key: &str) -> Result<(), AuthorizerError> {
        self.0.lock().await.insert(public_key.into());
        Ok(())
    }
}

impl From<Vec<String>> for MemoryAuthorizer {
    fn from(keys: Vec<String>) -> Self {
        Self(Arc::new(Mutex::new(keys.into_iter().collect())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_authorizer() {
        let authorizer = MemoryAuthorizer::from(vec!["test".into()]);

        assert!(authorizer.is_authorized("test").await.unwrap());
        assert!(!authorizer.is_authorized("test2").await.unwrap());
        authorizer.authorize("test2").await.unwrap();
        assert!(authorizer.is_authorized("test2").await.unwrap());
    }
}
