use axum::{async_trait, extract::FromRequestParts, http::request::Parts, response::Response};
use tempfile::TempDir;

pub struct TempDirHandle(pub TempDir);

impl Clone for TempDirHandle {
    fn clone(&self) -> Self {
        TempDirHandle(TempDir::new().expect("failed to create temp dir"))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for TempDirHandle
where
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request_parts(_req: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let temp_dir = TempDir::new().map_err(|_| {
            Response::builder()
                .status(500)
                .body("Failed to create temp dir".into())
                .unwrap()
        })?;
        Ok(TempDirHandle(temp_dir))
    }
}
