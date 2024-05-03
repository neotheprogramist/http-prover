use tokio::process::Command;

use crate::process::{self, ProcessError};

pub trait Runner {
    fn run(
        &self,
        input: &str,
    ) -> impl std::future::Future<Output = Result<String, ProcessError>> + Send;
}

pub struct PodmanRunner(String);

impl PodmanRunner {
    pub fn new(image_name: &str) -> Self {
        PodmanRunner(image_name.to_string())
    }
}

impl Runner for PodmanRunner {
    async fn run(&self, input: &str) -> Result<String, ProcessError> {
        let mut command = Command::new("podman");
        command.arg("run").arg("-i").arg("--rm").arg(&self.0);
        let result = process::run(command, Some(input.to_string())).await?;
        Ok(result)
    }
}
