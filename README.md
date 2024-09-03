# Cairo Proving System

This repository contains a comprehensive toolset for proving computations using the Cairo language. The repository includes a server, an SDK for interacting with the server, a binary `cairo-prove` for executing proofs, and helper binaries such as `keygen` and `register`.

## Table of Contents

- [Overview](#overview)
- [Components](#components)
  - [Server](#server)
  - [SDK](#sdk)
  - [Cairo-Prove Binary](#cairo-prove-binary)
  - [Keygen Binary](#keygen-binary)
  - [Register Binary](#register-binary)
  - [Common Library](#common-library)
- [Examples](#examples)
- [Scripts](#scripts)

## Overview

The Cairo Proving System provides tools to prove and verify computations written in the Cairo language. This repository includes:

1. A **server** that manages and verifies proofs.
2. An **SDK** to interact with the server programmatically.
3. A **cairo-prove binary** that implements the SDK and allows users to perform proofs from the command line.
4. Helper binaries like **keygen** and **register** to manage keys and registration.

## Components

### Server

The server is the core of the proving system. It handles proof requests, manages authorization, and verifies proofs.

- **Directory:** `prover`
- **Description:** The server is built using a modular design, with components handling authentication, proof generation, verification, and a thread pool to manage concurrent tasks.
- **Inner README:** [Server README](prover/README.md)

### SDK

The SDK provides a Rust-based interface for interacting with the server. It abstracts the underlying API calls and simplifies the development of client applications.

- **Directory:** `prover-sdk`
- **Description:** The SDK includes modules for handling access keys, managing errors, and building client requests.
- **Inner README:** [SDK README](prover-sdk/README.md)

### Cairo-Prove Binary

The `cairo-prove` binary is a command-line tool that leverages the SDK to perform proofs. It’s intended for users who want to interact with the proving system without writing custom code.

- **Inner README:** [Cairo-Prove README](bin/cairo-prove/README.md)

### Keygen Binary

The `keygen` binary is a helper tool for generating cryptographic keys required by the server and SDK.

- **Inner README:** [Keygen README](bin/keygen/README.md)

### Register Binary

The `register` binary is used to register new keys with the server. 
- **Inner README:** [Register README](bin/register/README.md)

### Common Library

The common library provides shared utilities and data structures used across the various components.

- **Directory:** `common`
- **Description:** Includes modules for handling prover inputs, requests, and shared models.

## Examples

Examples demonstrating how to use the tools are provided in the `examples` directory. These include:

- **Cairo Examples:** `examples/cairo`
- **Cairo 0 Examples:** `examples/cairo0`

Each example contains necessary inputs and compiled programs to run with the `cairo-prove` binary.

## Scripts

Helper scripts are included in the `scripts` directory for tasks like running end-to-end tests.

- **End-to-End Testing Script:** `scripts/e2e_test.sh`

## Getting Started

To get started with the Cairo Proving System, please refer to the individual READMEs linked above for detailed instructions on building, configuring, and running each component.

