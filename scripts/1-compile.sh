#!/usr/bin/env bash

set -eux

. .venv/bin/activate

mkdir -p resources

cairo-compile \
    examples/CairoZero/fibonacci.cairo \
  --output resources/fibonacci_compiled.json \
  --proof_mode

deactivate
