# `prove`

```sh
cargo run --bin prove -- --prover-key <your secret key for sdk> --cairo-version <1/0> your_input.json
```
## by the default cairo version is 1

input format 
```json
{
    "program":{program},
    "program_input":{program input}
}
```

# output
## scheduler outputs file with proof in result.json file 
```json
{
    proof
}
```
# testing prover with fibonacci 
To test prover we have to have compiled cairo program, and merged with input. 
We can do it for Cairo Zero with: 
```rust
cargo make --makefile /home/mateuszpc/dev/http-prover/Makefile.toml prepareCairoZero
```
For Cairo:
```rust
cargo make --makefile /home/mateuszpc/dev/http-prover/Makefile.toml prepareCairo
```
# to run tests
```
cargo test --package prove --test prove -- test_cairo0_fibonacci
```
```
cargo test --package prove --test prove -- test_cairo1_fibonacci
``