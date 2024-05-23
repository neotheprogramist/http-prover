#!/bin/bash

# This script assumes Python is installed and accessible from the command line as 'python'

# Variables for JSON files and output
FILE1="resources/fibonacci_compiled.json"
FILE2="examples/CairoZero/input.json"
OUTPUT="examples/CairoZero/prover_input.json"

# Call the Python script to combine the JSON files
python scripts/combine_json.py "$FILE1" "$FILE2" "$OUTPUT"
