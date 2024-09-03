# Cairo Prove 
## Overview

This application is a command-line tool designed to run the SDK for http-prover. The tool processes input files, validates the data, and generates the corresponding proofs. 

## Table of Contents

- [Installation](#installation)
- [Cli Arguments](#command-line-arguments)
  - [Input for cairo](#input-for-cairo)
  - [Input for cairo0](#input-for-cairo0)
  - [Output](#output)
  - [Parameters](#parameters)
- [Examples](#examples)

## Installation

To install the application, clone this repository and build the binary:

```bash
cargo install --git https://github.com/cartridge-gg/http-prover.git cairo-prove
```
## Command-Line Arguments
### The following command-line arguments and environment variables are available to configure the behavior of the application:

`--prover-url` (PROVER_URL): Specifies the URL of the prover service. This is a required argument that the application uses to connect to the prover service.

`--cairo-version` (CAIRO_VERSION, `default: v1`): Defines the version of Cairo to be used. This can be set to v1 or v0 for Cairo0.

`--layout` (LAYOUT): Specifies the layout used for the execution of the Cairo program. This argument determines the memory layout and other execution parameters. For example: `recursive`

`--program-path `(PROGRAM_PATH): Indicates the path to the Cairo program that will run and executed. This argument is mandatory and must point to the valid, compiled to `.sierra.json` program file.

`--program-input-path` (PROGRAM_INPUT_PATH): Provides the path to the input file for the program, which is required if the cairo_version is set to v0. *This flag cannot be used together with the `program_input` flag.*

`--program-input` (PROGRAM_INPUT): Specifies the input data for the program as a comma-separated list of strings. This input is used directly by the program. This flag is not compatible with program_input_path and is *used only when cairo_version is v1*.

`--program-output`(PROGRAM_OUTPUT): Specifies the path where the program's output will be saved. The output of the execution will be written to this file.

`--prover-access-key` (PROVER_ACCESS_KEY): Provides the access key required to authenticate with the prover service. This argument must be private key in hex format

`--wait` (WAIT, default: false): A flag that determines whether the application should wait for the prover's response synchronously. If set to true, the application will block until the prover completes its task, if set to false it ends and returns job, which can be retrived with `get-job` endpoint

Each of these arguments can be set via command-line flags or environment variables, allowing for flexible configuration depending on your deployment environment and needs.

## Input for Cairo

The application expects input in the form of a file that contains data formatted as an array of strings or integers. The format should adhere to the following guidelines:

- The input file should contain a single array of values
- Values should be separated by commas `,` without spaces.
- Each value should be either a hex or a numeric type that can be parsed into a `Felt`.

**Example Input:**

```plaintext
1,2,3,4,5
```
## Input for Cairo0
The application expects input in the form of a json file that contains valid json object.

**Example Input:**
```
{
    "fibonacci_claim_index": 10
}
```
### Output

The output of the application depends on --wait flag,it can be job id or the generated cryptographic proof, which will be saved to a specified output file


### Examples

**Basic Example:**

To generate a proof from an input file and print the result to the console:

```bash
cairo-prove --prover-url http://localhost:3000 --layout recursive --program-path examples/cairo/fibonacci_compiled.json --program-input-path examples/cairo/input.json --wait --program-output proof.json --prover-access-key 0xf5061793648ab019cc27d6c9a2bd8a2b651f9224ae9ae2c0990fd32ed2172f48
```
