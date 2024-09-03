# Prover Key Registration
This project provides a command-line tool for registering new keys with the Prover SDK. 
## **To register a new key in Prover, ensure that your signing key is configured as the admin key in Prover.**

## Table of Contents
- [Installation](#installation)
- [Arguments](#arguments)
- [Usage](#usage)



## Installation

```bash
cargo install --git  https://github.com/cartridge-gg/http-prover.git register
```
or alternatively 
```bash
git clone  https://github.com/cartridge-gg/http-prover.git
cargo build
cargo run -p register
```

## Arguments

- `--private-key` (`-p`): The private key used to authenticate with the Prover SDK. This should be provided as a hex string.
- `--added-key` (`-k`): The public key you wish to register, also provided as a hex string.
- `--url` (`-u`): The URL of the Prover SDK server.

## Usage

To run the program, you need to provide three arguments: `--private-key`, `--added-key`, and `--url`. These can be provided either via command-line arguments or environment variables.  

### Command-Line Arguments
```bash
register --private-key <PRIVATE_KEY> --added-key <ADDED_KEY> --url <URL>
```
or 
```bash
cargo run -p register -- --private-key <PRIVATE_KEY> --added-key <ADDED_KEY> --url <URL>
```
### Environment Variables

You can also set the arguments via environment variables:

```bash
export PRIVATE_KEY=<your_private_key>
export ADDED_KEY=<key_to_add>
export URL=<prover_sdk_url>
```
```bash
register
``` 
or 
```bash
cargo run -p register
```
