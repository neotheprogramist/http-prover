#!/usr/bin/env bash

source .venv/bin/activate && \
cairo-compile \
    examples/CairoZero/fibonacci.cairo \
  --output resources/fibonacci_compiled.json \
  --proof_mode && \
deactivate