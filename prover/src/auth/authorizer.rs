use std::{collections::HashSet, path::PathBuf};

use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Error)]
pub enum AuthorizerError {
    #[error("file read error")]
    FileReadError(#[from] std::io::Error),
    #[error("invalid file format")]
    FormatError(#[from] serde_json::Error),
}

pub(crate) trait AuthorizationProvider {
    async fn is_authorized(&self, public_key: &str) -> Result<bool, AuthorizerError>;
}

#[derive(Debug, Clone)]
pub enum Authorizer {
    Open,
    File(FileAuthorizer),
    Memory(MemoryAuthorizer),
}

impl AuthorizationProvider for Authorizer {
    async fn is_authorized(&self, public_key: &str) -> Result<bool, AuthorizerError> {
        Ok(match self {
            Authorizer::Open => true,
            Authorizer::File(authorizer) => authorizer.is_authorized(public_key).await?,
            Authorizer::Memory(authorizer) => authorizer.is_authorized(public_key).await?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileAuthorizer(PathBuf);

impl FileAuthorizer {
    pub async fn new(path: PathBuf) -> Result<Self, AuthorizerError> {
        File::open(&path)
            .await
            .map_err(AuthorizerError::FileReadError)?;

        Ok(Self(path))
    }
}

impl AuthorizationProvider for FileAuthorizer {
    async fn is_authorized(&self, public_key: &str) -> Result<bool, AuthorizerError> {
        let mut file = File::open(&self.0)
            .await
            .map_err(AuthorizerError::FileReadError)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(AuthorizerError::FileReadError)?;

        let authorized_keys: HashSet<String> = serde_json::from_str::<Vec<String>>(&contents)
            .map_err(AuthorizerError::FormatError)?
            .into_iter()
            .collect();

        Ok(authorized_keys.contains(public_key))
    }
}

#[derive(Debug, Clone)]
pub struct MemoryAuthorizer(HashSet<String>);

impl AuthorizationProvider for MemoryAuthorizer {
    async fn is_authorized(&self, public_key: &str) -> Result<bool, AuthorizerError> {
        Ok(self.0.contains(public_key))
    }
}

impl From<Vec<String>> for MemoryAuthorizer {
    fn from(keys: Vec<String>) -> Self {
        Self(keys.into_iter().collect())
    }
}
