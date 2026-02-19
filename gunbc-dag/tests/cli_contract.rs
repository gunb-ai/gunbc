use gunbc_cli::{parse, CliParam, ParamType};
use gunbc_dag::makegen::ToolRegistry;
use gunbc_dag::render_makefile;
use gunbc_ir::{to_bridge_json, Cardinality, Value};
use std::collections::{BTreeMap, HashMap};

#[test]
fn test_makefile_cli_args_match_entrypoints() {
    let registry = ToolRegistry::default_registry();
    let makefile = render_makefile(&registry);

    for tool in &registry.tools {
        for param in &tool.entrypoints {
            let expected = if param.repeatable {
                format!(
                    "$(if $({}),$(foreach v,$({}),{} $(v)))",
                    param.make_var, param.make_var, param.cli_flag
                )
            } else {
                format!(
                    "$(if $({}),{} $({}))",
                    param.make_var, param.cli_flag, param.make_var
                )
            };

            assert!(
                makefile.contains(&expected),
                "Makefile missing CLI arg wiring for {}.{}: expected `{}`",
                tool.short_name,
                param.port_name,
                expected
            );
        }
    }
}

#[test]
fn test_makefile_help_repeatable_params() {
    let registry = ToolRegistry::default_registry();
    let makefile = render_makefile(&registry);

    for tool in &registry.tools {
        for param in &tool.entrypoints {
            if !param.repeatable {
                continue;
            }

            let expected = match &param.default {
                Some(default) => format!("[{}={} ...]", param.make_var, default),
                None => format!("[{}=... ...]", param.make_var),
            };

            assert!(
                makefile.contains(&expected),
                "Makefile help missing repeatable param for {}.{}: expected `{}`",
                tool.short_name,
                param.port_name,
                expected
            );
        }
    }
}

fn param_type_from_hint(type_hint: &str) -> ParamType {
    ParamType::try_from(type_hint).unwrap_or(ParamType::Str)
}

fn parse_scalar_value(param_type: ParamType, raw: &str) -> Value {
    match param_type {
        ParamType::Str => Value::Str(raw.to_string()),
        ParamType::Int => Value::Int(
            raw.parse::<i64>()
                .expect("contract sample int should parse as i64"),
        ),
        ParamType::Bool => Value::Bool(raw == "true"),
    }
}

fn scalar_sample(port_name: &str, param_type: ParamType, idx: usize) -> String {
    match param_type {
        ParamType::Str => format!("{port_name}_value_{idx}"),
        ParamType::Int => (10 + idx as i64).to_string(),
        ParamType::Bool => {
            if idx == 0 {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
    }
}

#[test]
fn test_per_tool_dry_run_cli_contracts_match_registry_entrypoints() {
    let registry = ToolRegistry::default_registry();

    for tool in &registry.tools {
        let mut schema = Vec::new();
        let mut argv = vec![tool.short_name.clone(), "--dry-run".to_string()];
        let mut expected: HashMap<String, Value> = HashMap::new();

        for entry in &tool.entrypoints {
            let param_type = param_type_from_hint(&entry.type_hint);
            let mut cli = CliParam::new(entry.port_name.clone(), param_type);
            if entry.repeatable {
                cli = cli.with_cardinality(Cardinality::ZERO_OR_MORE);
            }
            if let Some(default) = &entry.default {
                cli = cli.default(default.clone());
            }
            schema.push(cli);

            if entry.repeatable {
                let v1 = scalar_sample(&entry.port_name, param_type, 0);
                let v2 = scalar_sample(&entry.port_name, param_type, 1);
                argv.push(entry.cli_flag.clone());
                argv.push(v1.clone());
                argv.push(entry.cli_flag.clone());
                argv.push(v2.clone());
                expected.insert(
                    entry.port_name.clone(),
                    Value::List(vec![
                        parse_scalar_value(param_type, &v1),
                        parse_scalar_value(param_type, &v2),
                    ]),
                );
                continue;
            }

            if param_type == ParamType::Bool {
                argv.push(entry.cli_flag.clone());
                expected.insert(entry.port_name.clone(), Value::Bool(true));
                continue;
            }

            let value = scalar_sample(&entry.port_name, param_type, 0);
            argv.push(entry.cli_flag.clone());
            argv.push(value.clone());
            expected.insert(
                entry.port_name.clone(),
                parse_scalar_value(param_type, &value),
            );
        }

        let result = parse(&argv, &schema).unwrap_or_else(|err| {
            panic!(
                "CLI parse contract failed for tool '{}' with argv {:?}: {}",
                tool.short_name, argv, err
            )
        });
        assert!(
            result.dry_run,
            "tool '{}' should set dry_run=true when --dry-run is present",
            tool.short_name
        );

        for (port_name, expected_value) in expected {
            assert_eq!(
                result.values.get(&port_name),
                Some(&expected_value),
                "tool '{}' parsed value mismatch for entrypoint '{}'",
                tool.short_name,
                port_name
            );
        }
    }
}

/// CT2: Validate that `--print-inputs json` produces valid JSON that round-trips
/// parsed CLI inputs. This simulates the generated code's behavior:
///   1. Strip `--print-inputs json` from argv before parse
///   2. Parse remaining args
///   3. Serialize cli_inputs via `to_bridge_json(Value::Map(BTreeMap))`
///   4. Verify JSON keys/values match expected entrypoint inputs
#[test]
fn test_per_tool_print_inputs_json_round_trip() {
    let registry = ToolRegistry::default_registry();

    for tool in &registry.tools {
        let mut schema = Vec::new();
        // Build argv with --print-inputs json interleaved
        let mut full_argv = vec![
            tool.short_name.clone(),
            "--print-inputs".to_string(),
            "json".to_string(),
        ];
        let mut expected_json = serde_json::Map::new();

        for entry in &tool.entrypoints {
            let param_type = param_type_from_hint(&entry.type_hint);
            let mut cli = CliParam::new(entry.port_name.clone(), param_type);
            if entry.repeatable {
                cli = cli.with_cardinality(Cardinality::ZERO_OR_MORE);
            }
            if let Some(default) = &entry.default {
                cli = cli.default(default.clone());
            }
            schema.push(cli);

            if entry.repeatable {
                let v1 = scalar_sample(&entry.port_name, param_type, 0);
                let v2 = scalar_sample(&entry.port_name, param_type, 1);
                full_argv.push(entry.cli_flag.clone());
                full_argv.push(v1.clone());
                full_argv.push(entry.cli_flag.clone());
                full_argv.push(v2.clone());
                let expected_list: Vec<serde_json::Value> = vec![
                    scalar_to_json(param_type, &v1),
                    scalar_to_json(param_type, &v2),
                ];
                expected_json.insert(
                    entry.port_name.clone(),
                    serde_json::Value::Array(expected_list),
                );
                continue;
            }

            if param_type == ParamType::Bool {
                full_argv.push(entry.cli_flag.clone());
                expected_json.insert(entry.port_name.clone(), serde_json::Value::Bool(true));
                continue;
            }

            let value = scalar_sample(&entry.port_name, param_type, 0);
            full_argv.push(entry.cli_flag.clone());
            full_argv.push(value.clone());
            expected_json.insert(entry.port_name.clone(), scalar_to_json(param_type, &value));
        }

        // Step 1: Strip --print-inputs json from argv (simulating generated code)
        let mut parse_args: Vec<String> = Vec::with_capacity(full_argv.len());
        if let Some(program) = full_argv.first() {
            parse_args.push(program.clone());
        }
        let mut print_inputs_json = false;
        let mut raw_idx = 1;
        while raw_idx < full_argv.len() {
            let arg = &full_argv[raw_idx];
            if arg == "--print-inputs" {
                raw_idx += 1;
                assert!(
                    raw_idx < full_argv.len() && full_argv[raw_idx] == "json",
                    "tool '{}': --print-inputs should be followed by 'json'",
                    tool.short_name
                );
                print_inputs_json = true;
            } else if let Some(format) = arg.strip_prefix("--print-inputs=") {
                assert_eq!(format, "json");
                print_inputs_json = true;
            } else {
                parse_args.push(arg.clone());
            }
            raw_idx += 1;
        }
        assert!(
            print_inputs_json,
            "tool '{}': --print-inputs json flag should have been detected",
            tool.short_name
        );

        // Step 2: Parse remaining args
        let result = parse(&parse_args, &schema).unwrap_or_else(|err| {
            panic!(
                "CLI parse contract failed for tool '{}' with argv {:?}: {}",
                tool.short_name, parse_args, err
            )
        });

        // Step 3: Serialize via to_bridge_json (matching generated code)
        let mut ordered_inputs = BTreeMap::new();
        for (port, value) in &result.values {
            ordered_inputs.insert(port.clone(), value.clone());
        }
        let json = to_bridge_json(&Value::Map(ordered_inputs))
            .unwrap_or_else(|| panic!("tool '{}': to_bridge_json returned None", tool.short_name));

        // Step 4: Verify JSON is an object with expected keys
        let obj = json.as_object().unwrap_or_else(|| {
            panic!(
                "tool '{}': JSON output should be an object",
                tool.short_name
            )
        });

        for (key, expected_val) in &expected_json {
            assert_eq!(
                obj.get(key),
                Some(expected_val),
                "tool '{}': --print-inputs json mismatch for entrypoint '{}'",
                tool.short_name,
                key
            );
        }

        // Verify no unexpected keys
        for key in obj.keys() {
            assert!(
                expected_json.contains_key(key),
                "tool '{}': --print-inputs json has unexpected key '{}'",
                tool.short_name,
                key
            );
        }
    }
}

/// Also test the `--print-inputs=json` form (equals-separated)
#[test]
fn test_print_inputs_equals_form_parses() {
    let registry = ToolRegistry::default_registry();

    // Pick first tool with entrypoints for a focused test
    let tool = registry
        .tools
        .iter()
        .find(|t| !t.entrypoints.is_empty())
        .expect("registry should have at least one tool with entrypoints");

    let mut schema = Vec::new();
    let mut full_argv = vec![tool.short_name.clone(), "--print-inputs=json".to_string()];

    for entry in &tool.entrypoints {
        let param_type = param_type_from_hint(&entry.type_hint);
        let mut cli = CliParam::new(entry.port_name.clone(), param_type);
        if entry.repeatable {
            cli = cli.with_cardinality(Cardinality::ZERO_OR_MORE);
        }
        if let Some(default) = &entry.default {
            cli = cli.default(default.clone());
        }
        schema.push(cli);

        if entry.repeatable || param_type == ParamType::Bool {
            full_argv.push(entry.cli_flag.clone());
            if param_type != ParamType::Bool {
                full_argv.push(scalar_sample(&entry.port_name, param_type, 0));
            }
        } else {
            full_argv.push(entry.cli_flag.clone());
            full_argv.push(scalar_sample(&entry.port_name, param_type, 0));
        }
    }

    // Strip --print-inputs=json
    let mut parse_args: Vec<String> = Vec::with_capacity(full_argv.len());
    if let Some(program) = full_argv.first() {
        parse_args.push(program.clone());
    }
    let mut found = false;
    for arg in full_argv.iter().skip(1) {
        if arg.strip_prefix("--print-inputs=").is_some() {
            found = true;
        } else {
            parse_args.push(arg.clone());
        }
    }
    assert!(found, "should have found --print-inputs=json");

    let result = parse(&parse_args, &schema).unwrap_or_else(|err| {
        panic!(
            "CLI parse failed for tool '{}' with --print-inputs=json: {}",
            tool.short_name, err
        )
    });

    let mut ordered = BTreeMap::new();
    for (port, value) in &result.values {
        ordered.insert(port.clone(), value.clone());
    }
    let json = to_bridge_json(&Value::Map(ordered)).expect("serialization should succeed");
    assert!(
        json.is_object(),
        "JSON output should be an object for tool '{}'",
        tool.short_name
    );
}

fn scalar_to_json(param_type: ParamType, raw: &str) -> serde_json::Value {
    match param_type {
        ParamType::Str => serde_json::Value::String(raw.to_string()),
        ParamType::Int => serde_json::json!(raw.parse::<i64>().unwrap()),
        ParamType::Bool => serde_json::Value::Bool(raw == "true"),
    }
}
