use std::path::PathBuf;

use bootloader_cairo_vm::air_private_input::AirPrivateInput;
use bootloader_cairo_vm::hint_processor::builtin_hint_processor::bootloader::types::{
    Task, TaskSpec,
};
use bootloader_cairo_vm::types::program::Program;
use bootloader_cairo_vm::vm::runners::cairo_pie::CairoPie;
use madara_prover_rpc_server::services::starknet_prover::run_bootloader_in_proof_mode;
use stone_prover_sdk::fri::generate_prover_parameters;
use stone_prover_sdk::json::write_json_to_file;
use stone_prover_sdk::models::{ProverConfig, ProverParameters, PublicInput};

const BOOTLOADER_PROGRAM: &[u8] = include_bytes!("../../bootloader/madara-bootloader.json");
const RUN_PROGRAM: &[u8] = include_bytes!("../../bootloader/cairo_pie.zip");

#[test]
fn test_bootloader_run() {
    bootloader_run();
}

pub fn bootloader_run() {
    // Prepare program pie
    let pies = vec![RUN_PROGRAM.to_vec()];
    let programs = vec![];

    // Run pie in the bootloader
    let bootloader_program = Program::from_bytes(BOOTLOADER_PROGRAM, Some("main")).unwrap();
    let prover_config = ProverConfig::default();
    let bootloader_tasks = make_bootloader_tasks(&programs, &pies);

    let execution_artifacts =
        run_bootloader_in_proof_mode(&bootloader_program, bootloader_tasks).unwrap();

    let prover_parameters = get_prover_parameters(None, execution_artifacts.public_input.n_steps);

    prepare_prover_files(
        &execution_artifacts.public_input,
        &execution_artifacts.private_input,
        &execution_artifacts.memory,
        &execution_artifacts.trace,
        &prover_config,
        &prover_parameters,
    );
}

fn make_bootloader_tasks(programs: &[Vec<u8>], pies: &[Vec<u8>]) -> Vec<TaskSpec> {
    let program_tasks = programs.iter().map(|program_bytes| {
        let program = Program::from_bytes(program_bytes, Some("main"));
        program
            .map(|program| TaskSpec {
                task: Task::Program(program),
            })
            .unwrap()
    });

    let cairo_pie_tasks = pies.iter().map(|pie_bytes| {
        let pie = CairoPie::from_bytes(pie_bytes);
        pie.map(|pie| TaskSpec {
            task: Task::Pie(pie),
        })
        .unwrap()
    });

    program_tasks.chain(cairo_pie_tasks).collect()
}

fn prepare_prover_files(
    public_input: &PublicInput,
    private_input: &AirPrivateInput,
    memory: &Vec<u8>,
    trace: &Vec<u8>,
    prover_config: &ProverConfig,
    parameters: &ProverParameters,
)
//  -> Result<ProverWorkingDirectory, std::io::Error>
{
    let tmp_dir_path = PathBuf::from(".");

    let public_input_file = tmp_dir_path.join("public_input.json");
    let private_input_file = tmp_dir_path.join("private_input.json");
    let memory_file = tmp_dir_path.join("memory.bin");
    let prover_config_file = tmp_dir_path.join("prover_config_file.json");
    let prover_parameter_file = tmp_dir_path.join("parameters.json");
    let trace_file = tmp_dir_path.join("trace.bin");
    // let proof_file = tmp_dir_path.join("proof.json");

    // Write public input and config/parameters files
    write_json_to_file(public_input, &public_input_file).unwrap();
    write_json_to_file(prover_config, &prover_config_file).unwrap();
    write_json_to_file(parameters, &prover_parameter_file).unwrap();

    // Write private input file
    let private_input_serializable = private_input.to_serializable(
        trace_file.to_string_lossy().to_string(),
        memory_file.to_string_lossy().to_string(),
    );
    write_json_to_file(private_input_serializable, &private_input_file).unwrap();

    // Write memory and trace files
    std::fs::write(&memory_file, memory).unwrap();
    std::fs::write(&trace_file, trace).unwrap();

    // Ok(ProverWorkingDirectory {
    //     dir: tmp_dir,
    //     public_input_file,
    //     private_input_file,
    //     _memory_file: memory_file,
    //     _trace_file: trace_file,
    //     prover_config_file,
    //     prover_parameter_file,
    //     proof_file,
    //     annotations_file: None,
    //     extra_annotations_file: None,
    // })
}

pub fn get_prover_parameters(
    user_provided_parameters: Option<String>,
    nb_steps: u32,
) -> ProverParameters {
    if let Some(params_str) = user_provided_parameters {
        return serde_json::from_str(&params_str).unwrap();
    }

    let last_layer_degree_bound = 64;
    generate_prover_parameters(nb_steps, last_layer_degree_bound)
}
