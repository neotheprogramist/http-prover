# `prove`

```sh
cargo run --bin prove -- --key <your secret key for sdk> --cairo_version <1/0> your_input.json
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