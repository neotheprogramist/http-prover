pub mod run;
pub mod runner_error;
use std::io::{self, Write};
use std::path::PathBuf;

use bincode::enc::write::Writer;
use cairo_lang_sierra::program::Program;
use cairo_vm::air_public_input::PublicInputError;
pub use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::vm::errors::trace_errors::TraceError;
pub use cairo_vm::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::{runners::cairo_runner::CairoRunner, vm_core::VirtualMachine},
    Felt252,
};
use run::FuncArg;
use run::{cairo_run_program, Cairo1RunConfig};
use runner_error::CairoRunError;
use starknet_types_core::felt::Felt;

use crate::threadpool::run::RunPaths;

pub fn get_cairo_pie(
    program_file: Program,
    output_file: PathBuf,
    layout: LayoutName,
    input: Vec<Felt>,
) -> Result<Option<String>, CairoRunError> {
    let args = FuncArg::Array(input);

    let cairo_run_config = Cairo1RunConfig {
        proof_mode: false,
        serialize_output: true,
        relocate_mem: false,
        layout,
        trace_enabled: false,
        args: &[args],
        finalize_builtins: true,
        append_return_values: false,
    };
    // Try to parse the file as a sierra program
    let (runner, _, serialized_output) = cairo_run_program(&program_file, cairo_run_config)?;
    runner.get_cairo_pie()?.write_zip_file(&output_file)?;

    Ok(serialized_output)
}
pub fn run_cairo_program(
    program_file: Program,
    layout: LayoutName,
    input: Vec<Felt>,
    run_paths: &RunPaths,
) -> Result<(), CairoRunError> {
    let args = FuncArg::Array(input);
    println!("Running program: {}", program_file);
    let cairo_run_config = Cairo1RunConfig {
        proof_mode: true,
        serialize_output: false,
        relocate_mem: true,
        layout,
        trace_enabled: true,
        args: &[args],
        finalize_builtins: true,
        append_return_values: false,
    };
    // Try to parse the file as a sierra program
    println!("Running program: {}", program_file);
    let (runner, _, _serialized_output) = cairo_run_program(&program_file, cairo_run_config)?;
    println!("Program read");
    let json = runner.get_air_public_input()?.serialize_json()?;
    println!("saving public input");
    std::fs::write(run_paths.public_input_file, json.clone())?;
    // Get absolute paths of trace_file & memory_file
    let trace_path = run_paths
        .trace_file
        .as_path()
        .canonicalize()
        .unwrap_or(run_paths.trace_file.clone())
        .to_string_lossy()
        .to_string();
    let memory_path = run_paths
        .memory_file
        .as_path()
        .canonicalize()
        .unwrap_or(run_paths.memory_file.clone())
        .to_string_lossy()
        .to_string();

    let json = runner
        .get_air_private_input()
        .to_serializable(trace_path, memory_path)
        .serialize_json()
        .map_err(PublicInputError::Serde)?;
    std::fs::write(run_paths.private_input_file, json)?;

    let relocated_trace = runner
        .relocated_trace
        .ok_or(CairoRunError::Trace(TraceError::TraceNotRelocated))?;

    let trace_file = std::fs::File::create(run_paths.trace_file)?;
    let mut trace_writer =
        FileWriter::new(io::BufWriter::with_capacity(3 * 1024 * 1024, trace_file));
    cairo_vm::cairo_run::write_encoded_trace(&relocated_trace, &mut trace_writer)?;
    trace_writer.flush()?;

    let memory_file = std::fs::File::create(run_paths.memory_file)?;
    let mut memory_writer =
        FileWriter::new(io::BufWriter::with_capacity(5 * 1024 * 1024, memory_file));

    cairo_vm::cairo_run::write_encoded_memory(&runner.relocated_memory, &mut memory_writer)?;
    memory_writer.flush()?;
    Ok(())
}

pub struct FileWriter {
    buf_writer: io::BufWriter<std::fs::File>,
    bytes_written: usize,
}

impl Writer for FileWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), bincode::error::EncodeError> {
        self.buf_writer
            .write_all(bytes)
            .map_err(|e| bincode::error::EncodeError::Io {
                inner: e,
                index: self.bytes_written,
            })?;

        self.bytes_written += bytes.len();

        Ok(())
    }
}

impl FileWriter {
    fn new(buf_writer: io::BufWriter<std::fs::File>) -> Self {
        Self {
            buf_writer,
            bytes_written: 0,
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf_writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use crate::threadpool::prove::ProvePaths;

    use super::*;
    use cairo_vm::types::layout_name::LayoutName;
    use itertools::Itertools;
    use starknet_types_core::felt::Felt;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    #[test]
    fn test_get_cairo_pie() -> Result<(), CairoRunError> {
        let input = vec![
            Felt::from(1),
            Felt::from_dec_str(
                "1084568281184221360887085980818130019116060769753707796384172133640093947392",
            )
            .unwrap(),
            Felt::from_dec_str(
                "617075754465154585683856897856256838130216341506379215893724690153393808813",
            )
            .unwrap(),
            Felt::from(4),
            Felt::from(1),
            Felt::from_dec_str(
                "1962399278914746334808042087624794244340195160841430388580769389462301739649",
            )
            .unwrap(),
            Felt::from_dec_str(
                "946638316592298107720053446348402985413061731752482859793467974131030053837",
            )
            .unwrap(),
            Felt::from(0),
            Felt::from(0),
            Felt::from(0),
            Felt::from(193823),
            Felt::from(0),
            Felt::from(0),
        ];
        let filename = PathBuf::from("../examples/batcher.json");
        let binding = TempDir::new()?;
        let cairo_pie_output = binding.path().join("cairo_pie_output.zip");
        let layout = LayoutName::recursive;
        println!("Running program: {}", filename.display());
        let program = fs::read(filename)?;
        println!("Program read");
        let program_json = serde_json::from_slice(&program).unwrap();
        match get_cairo_pie(program_json, cairo_pie_output.to_path_buf(), layout, input) {
            Err(CairoRunError::Cli(err)) => err.exit(),
            Ok(output) => {
                if let Some(output_string) = output {
                    println!("Program Output : {}", output_string);
                }
                Ok(())
            }
            Err(CairoRunError::RunPanic(panic_data)) => {
                if !panic_data.is_empty() {
                    let panic_data_string_list = panic_data
                        .iter()
                        .map(|m| {
                            // Try to parse to utf8 string
                            let msg = String::from_utf8(m.to_bytes_be().to_vec());
                            if let Ok(msg) = msg {
                                format!("{} ('{}')", m, msg)
                            } else {
                                m.to_string()
                            }
                        })
                        .join(", ");
                    println!("Run panicked with: [{}]", panic_data_string_list);
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
    #[test]
    fn test_run_cairo_program() -> Result<(), CairoRunError> {
        let input = vec![
            Felt::from(1),
            Felt::from_dec_str(
                "1084568281184221360887085980818130019116060769753707796384172133640093947392",
            )
            .unwrap(),
            Felt::from_dec_str(
                "617075754465154585683856897856256838130216341506379215893724690153393808813",
            )
            .unwrap(),
            Felt::from(4),
            Felt::from(1),
            Felt::from_dec_str(
                "1962399278914746334808042087624794244340195160841430388580769389462301739649",
            )
            .unwrap(),
            Felt::from_dec_str(
                "946638316592298107720053446348402985413061731752482859793467974131030053837",
            )
            .unwrap(),
            Felt::from(0),
            Felt::from(0),
            Felt::from(0),
            Felt::from(193823),
            Felt::from(0),
            Felt::from(0),
        ];
        let filename = PathBuf::from("../examples/batcher.json");
        let layout = LayoutName::recursive;
        let program = fs::read(filename)?;
        let program_json = serde_json::from_slice(&program).unwrap();
        let dir = TempDir::new()?;
        let prove_path = ProvePaths::new(dir);
        let run_path = RunPaths::from(&prove_path);
        match run_cairo_program(program_json, layout, input, &run_path) {
            Err(CairoRunError::Cli(err)) => err.exit(),
            Ok(_output) => Ok(()),
            Err(CairoRunError::RunPanic(panic_data)) => {
                if !panic_data.is_empty() {
                    let panic_data_string_list = panic_data
                        .iter()
                        .map(|m| {
                            // Try to parse to utf8 string
                            let msg = String::from_utf8(m.to_bytes_be().to_vec());
                            if let Ok(msg) = msg {
                                format!("{} ('{}')", m, msg)
                            } else {
                                m.to_string()
                            }
                        })
                        .join(", ");
                    println!("Run panicked with: [{}]", panic_data_string_list);
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
