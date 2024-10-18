use std::{fs, path::PathBuf};

use cairo_vm::types::layout_name::LayoutName;
use common::prover_input::{Cairo0ProverInput, CairoProverInput};
use starknet_types_core::felt::Felt;
use tokio::process::Command;
use tracing::trace;

use crate::{cairo1_run::run_cairo_program, errors::ProverError};

use super::prove::ProvePaths;
pub enum CairoVersionedInput {
    Cairo(CairoProverInput),
    Cairo0(Cairo0ProverInput),
}

impl CairoVersionedInput {
    pub async fn prepare_and_run(&self, paths: &'_ RunPaths<'_>) -> Result<(), ProverError> {
        self.prepare(paths)?;
        self.run(paths).await
    }
    fn prepare(&self, paths: &RunPaths<'_>) -> Result<(), ProverError> {
        match self {
            CairoVersionedInput::Cairo(input) => {
                let program = serde_json::to_string(&input.program)?;
                let input = prepare_input(&input.program_input);
                fs::write(paths.program, program)?;
                fs::write(paths.program_input_path, input)?;
            }
            CairoVersionedInput::Cairo0(input) => {
                fs::write(
                    paths.program_input_path.clone(),
                    serde_json::to_string(&input.program_input)?,
                )?;
                fs::write(paths.program, serde_json::to_string(&input.program)?)?;
            }
        }
        Ok(())
    }
    async fn run(&self, paths: &RunPaths<'_>) -> Result<(), ProverError> {
        match self {
            CairoVersionedInput::Cairo(input) => {
                trace!("Running cairo1-run");
                for path in &[
                    paths.trace_file,
                    paths.memory_file,
                    paths.public_input_file,
                    paths.private_input_file,
                    paths.program_input_path,
                    paths.program,
                ] {
                    std::fs::File::create(path)?;
                }
                let program_json =
                    serde_json::from_value(serde_json::to_value(input.program.clone())?)?;
                run_cairo_program(
                    program_json,
                    LayoutName::recursive,
                    input.program_input.clone(),
                    paths,
                )
                .unwrap(); //change this line to use layout type instead of string
                println!("Finished running cairo program");
                Ok(())
            }
            CairoVersionedInput::Cairo0(input) => {
                trace!("Running cairo0-run");
                let command = paths.cairo0_run_command(&input.layout);
                command_run(command).await
            }
        }
    }
}

pub struct RunPaths<'a> {
    pub trace_file: &'a PathBuf,
    pub memory_file: &'a PathBuf,
    pub public_input_file: &'a PathBuf,
    pub private_input_file: &'a PathBuf,
    pub program_input_path: &'a PathBuf,
    pub program: &'a PathBuf,
}

impl RunPaths<'_> {
    pub fn cairo0_run_command(&self, layout: &str) -> Command {
        let mut command = Command::new("cairo-run");
        command
            .arg("--trace_file")
            .arg(self.trace_file)
            .arg("--memory_file")
            .arg(self.memory_file)
            .arg("--layout")
            .arg(layout)
            .arg("--proof_mode")
            .arg("--air_public_input")
            .arg(self.public_input_file)
            .arg("--air_private_input")
            .arg(self.private_input_file)
            .arg("--program_input")
            .arg(self.program_input_path)
            .arg("--program")
            .arg(self.program);
        command
    }
}

impl<'a> From<&'a ProvePaths> for RunPaths<'a> {
    fn from(
        ProvePaths {
            trace_file,
            memory_file,
            public_input_file,
            private_input_file,
            program_input: program_input_path,
            program: program_path,
            ..
        }: &'a ProvePaths,
    ) -> Self {
        Self {
            trace_file,
            memory_file,
            public_input_file,
            private_input_file,
            program_input_path,
            program: program_path,
        }
    }
}

pub async fn command_run(mut command: Command) -> Result<(), ProverError> {
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let child = command.spawn()?;
    let output = child.wait_with_output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ProverError::CustomError(stderr.into()));
    }
    Ok(())
}

pub fn prepare_input(felts: &[Felt]) -> String {
    felts
        .iter()
        .fold("[".to_string(), |a, i| a + &i.to_string() + " ")
        .trim_end()
        .to_string()
        + "]"
}

#[test]
fn test_prepare_input() {
    assert_eq!("[]", prepare_input(&[]));
    assert_eq!("[1]", prepare_input(&[1.into()]));
    assert_eq!(
        "[1 2 3 4]",
        prepare_input(&[1.into(), 2.into(), 3.into(), 4.into()])
    );
}
