use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProveInput {
    pub program: CompiledProgram,
    pub program_input: serde_json::Value
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledProgram {
    pub attributes: Vec<String>,
    pub builtins: Vec<String>,
    pub compiler_version: String,
    pub data: Vec<String>,
    pub debug_info: serde_json::Value,
    pub hints:serde_json::Value,
    pub identifiers: serde_json::Value,
    pub main_scope: String,
    pub prime: String,
    pub reference_manager: serde_json::Value,
}
#[cfg(test)]
mod tests{
    use crate::prove::prove_input::CompiledProgram;
    use crate::prove::prove_input::ProveInput;

#[test]
fn test_deserialize_compiled_program() -> serde_json::Result<()> {
    let input = r#"{
        "attributes": [],
        "builtins": [
            "output",
            "pedersen"
        ],
        "compiler_version": "0.13.1",
        "data": [
            "0x40780017fff7fff",
            "0x4",
            "0x800000000000011000000000000000000000000000000000000000000000000",
            "0x1104800180018000",
            "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff9",
            "0x208b7fff7fff7ffe"
        ],
        "debug_info": {
            "file_contents": {
                "<start>": "__start__:\nap += main.Args.SIZE + main.ImplicitArgs.SIZE;\ncall main;\n\n__end__:\njmp rel 0;\n"
            },
            "instruction_locations": {
                "0": {
                    "accessible_scopes": ["__main__"],
                    "flow_tracking_data": {
                        "ap_tracking": {
                            "group": 0,
                            "offset": 0
                        },
                        "reference_ids": {}
                    },
                    "hints": [],
                    "inst": {
                        "end_col": 46,
                        "end_line": 2,
                        "input_file": {
                            "filename": "<start>"
                        },
                        "start_col": 1,
                        "start_line": 2
                    }
                }
            }
        },
        "hints": [],
        "identifiers": {
            "__main__.__end__": {
                "pc": 4,
                "type": "label"
            },
            "__main__.__start__": {
                "pc": 0,
                "type": "label"
            }
        },
        "main_scope": "__main__",
        "prime": "0x800000000000011000000000000000000000000000000000000000000000001",
        "reference_manager": {
            "references": []
        }
    }"#;

    let compiled_program = serde_json::from_str::<CompiledProgram>(input)?;
    let expected = CompiledProgram {
        attributes: vec![],
        builtins: vec!["output".to_string(), "pedersen".to_string()],
        compiler_version: "0.13.1".to_string(),
        data: vec![
            "0x40780017fff7fff".to_string(),
            "0x4".to_string(),
            "0x800000000000011000000000000000000000000000000000000000000000000".to_string(),
            "0x1104800180018000".to_string(),
            "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff9".to_string(),
            "0x208b7fff7fff7ffe".to_string(),
        ],
        debug_info: serde_json::json!({
            "file_contents": {
                "<start>": "__start__:\nap += main.Args.SIZE + main.ImplicitArgs.SIZE;\ncall main;\n\n__end__:\njmp rel 0;\n"
            },
            "instruction_locations": {
                "0": {
                    "accessible_scopes": ["__main__"],
                    "flow_tracking_data": {
                        "ap_tracking": {
                            "group": 0,
                            "offset": 0
                        },
                        "reference_ids": {}
                    },
                    "hints": [],
                    "inst": {
                        "end_col": 46,
                        "end_line": 2,
                        "input_file": {
                            "filename": "<start>"
                        },
                        "start_col": 1,
                        "start_line": 2
                    }
                }
            }
        }),
        hints: serde_json::Value::Array(Default::default()),
        identifiers: serde_json::json!({
            "__main__.__end__": {
                "pc": 4,
                "type": "label"
            },
            "__main__.__start__": {
                "pc": 0,
                "type": "label"
            }
        }),
        main_scope: "__main__".to_string(),
        prime: "0x800000000000011000000000000000000000000000000000000000000000001".to_string(),
        reference_manager: serde_json::json!({
            "references": []
        }),
    };

    assert_eq!(compiled_program, expected);

    Ok(())
}
#[test]
fn test_serialize_compiled_program() -> serde_json::Result<()> {
    let input = CompiledProgram {
        attributes: vec![],
        builtins: vec!["output".to_string(), "pedersen".to_string()],
        compiler_version: "0.13.1".to_string(),
        data: vec![
            "0x40780017fff7fff".to_string(),
            "0x4".to_string(),
            "0x800000000000011000000000000000000000000000000000000000000000000".to_string(),
            "0x1104800180018000".to_string(),
            "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff9".to_string(),
            "0x208b7fff7fff7ffe".to_string(),
        ],
        debug_info: serde_json::json!({
            "file_contents": {
                "<start>": "__start__:\nap += main.Args.SIZE + main.ImplicitArgs.SIZE;\ncall main;\n\n__end__:\njmp rel 0;\n"
            },
            "instruction_locations": {
                "0": {
                    "accessible_scopes": ["__main__"],
                    "flow_tracking_data": {
                        "ap_tracking": {
                            "group": 0,
                            "offset": 0
                        },
                        "reference_ids": {}
                    },
                    "hints": [],
                    "inst": {
                        "end_col": 46,
                        "end_line": 2,
                        "input_file": {
                            "filename": "<start>"
                        },
                        "start_col": 1,
                        "start_line": 2
                    }
                }
            }
        }),
        hints: serde_json::Value::Array(Default::default()),
        identifiers: serde_json::json!({
            "__main__.__end__": {
                "pc": 4,
                "type": "label"
            },
            "__main__.__start__": {
                "pc": 0,
                "type": "label"
            }
        }),
        main_scope: "__main__".to_string(),
        prime: "0x800000000000011000000000000000000000000000000000000000000000001".to_string(),
        reference_manager: serde_json::json!({
            "references": []
        }),
    };
    let serialized = serde_json::to_string(&input)?;
    let deserialized = serde_json::from_str(&serialized)?;
    assert_eq!(input, deserialized);
    Ok(())
}
#[test]
fn test_serialize_prove_input() -> serde_json::Result<()> {
    let compiled_program = CompiledProgram {
        attributes: vec![],
        builtins: vec!["output".to_string(), "pedersen".to_string()],
        compiler_version: "0.13.1".to_string(),
        data: vec![
            "0x40780017fff7fff".to_string(),
            "0x4".to_string(),
            "0x800000000000011000000000000000000000000000000000000000000000000".to_string(),
            "0x1104800180018000".to_string(),
            "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffff9".to_string(),
            "0x208b7fff7fff7ffe".to_string(),
        ],
        debug_info: serde_json::json!({
            "file_contents": {
                "<start>": "__start__:\nap += main.Args.SIZE + main.ImplicitArgs.SIZE;\ncall main;\n\n__end__:\njmp rel 0;\n"
            },
            "instruction_locations": {
                "0": {
                    "accessible_scopes": ["__main__"],
                    "flow_tracking_data": {
                        "ap_tracking": {
                            "group": 0,
                            "offset": 0
                        },
                        "reference_ids": {}
                    },
                    "hints": [],
                    "inst": {
                        "end_col": 46,
                        "end_line": 2,
                        "input_file": {
                            "filename": "<start>"
                        },
                        "start_col": 1,
                        "start_line": 2
                    }
                }
            }
        }),
        hints: serde_json::Value::Array(Default::default()),
        identifiers: serde_json::json!({
            "__main__.__end__": {
                "pc": 4,
                "type": "label"
            },
            "__main__.__start__": {
                "pc": 0,
                "type": "label"
            }
        }),
        main_scope: "__main__".to_string(),
        prime: "0x800000000000011000000000000000000000000000000000000000000000001".to_string(),
        reference_manager: serde_json::json!({
            "references": []
        }),
    };
    let input =r#"{
        "fibonacci_claim_index": 10
    }"#;
    let prove_input =  ProveInput{program:compiled_program,
    program_input: serde_json::to_value(&input).unwrap()};
    let serialized = &serde_json::to_string(&prove_input).unwrap(); 
    let deserialized:ProveInput = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized,prove_input);
    Ok(())
}
}