use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::coproduct_reflection;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CONFORMANCE_ENTRY: &str =
    "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag";
const WITNESS_FN: &str = "coproduct_reflection_conformance_holds";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn cert_sources() -> Vec<Rc<SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(CONFORMANCE_ENTRY))
        .unwrap_or_else(|e| panic!("read {CONFORMANCE_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(
        CONFORMANCE_ENTRY,
        &entry_content,
        &v2_source_roots(),
    )
    .iter()
    .map(|s| {
        Rc::new(SourceFile {
            path: s.path.clone(),
            content: s.content.clone(),
        })
    })
    .collect()
}

fn assert_resolved_ok(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved graph for {CONFORMANCE_ENTRY}, got diagnostics {msgs:?}"
    );
}

fn run_witness(resolved: &ResolvedPipelineResult, function: &str) -> Value {
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    v1_interpreter::run(graph, resolved.source_indices.clone(), function)
        .unwrap_or_else(|e| panic!("run {function}: {e}"))
}

fn symbol_list_strings(val: &Value) -> Vec<String> {
    let items = match val {
        Value::List(xs) => xs.iter().cloned().collect::<Vec<_>>(),
        _ => panic!("expected list, got {val:?}"),
    };
    items
        .iter()
        .map(|v| match v {
            Value::Str(s) => s.clone(),
            other => panic!("expected Str in symbol list, got {other:?}"),
        })
        .collect()
}

#[test]
fn coproduct_reflection_std_node_bridge_fns_are_intercept_wired() {
    let source = include_str!("../../stage0/src/v1_interpreter.rs");
    for name in v1_interpreter::std_node_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.node bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_node_query_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.node_query bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_concept_index_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.concept_index bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_fn_index_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.fn_index bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_qualified_name_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.qualified_name bridge `{name}`"
        );
    }
    assert!(
        source.contains("is_v4_std_node_bridge_call"),
        "eval_call must gate v2.std.node bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_node_query_bridge_call"),
        "eval_call must gate v2.std.node_query bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_concept_index_bridge_call"),
        "eval_call must gate v2.std.concept_index bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_fn_index_bridge_call"),
        "eval_call must gate v2.std.fn_index bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_qualified_name_bridge_call"),
        "eval_call must gate v2.std.qualified_name bridge dispatch"
    );
}

#[test]
fn coproduct_reflection_path3_connective_behavior_conformance_holds() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    for (name, expect) in [
        ("witness_connective_arm_key_set_path3", true),
        ("witness_behavior_arm_key_set_path3", true),
        ("witness_connective_arm_payload_pair_path3", true),
        ("witness_behavior_arm_payload_pair_path3", true),
        ("witness_connective_behavior_arm_sets_are_distinct", true),
        (
            "witness_behavior_nullary_inhabitant_set_matches_arm_keys",
            true,
        ),
        ("witness_connective_nullary_inhabitants_fail_closed", true),
    ] {
        match run_witness(&resolved, name) {
            Value::Bool(v) if v == expect => {}
            other => panic!("witness {name}: expected Bool({expect}), got {other:?}"),
        }
    }
    match run_witness(&resolved, WITNESS_FN) {
        Value::Bool(true) => {}
        other => {
            panic!("expected Bool(true) from {WITNESS_FN} (Path-3 conformance), got {other:?}")
        }
    }
}

#[test]
fn coproduct_reflection_connective_behavior_arm_sets_are_distinct() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    match run_witness(
        &resolved,
        "witness_connective_behavior_arm_sets_are_distinct",
    ) {
        Value::Bool(true) => {}
        other => panic!("expected distinct arm sets, got {other:?}"),
    }
}

#[test]
fn coproduct_reflection_connective_reflection_pairs_match_syntactic() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let connective = (None, Value::Str("Connective".to_string()));
    let node =
        coproduct_reflection::eval_resolve_type_node(&ctx, std::slice::from_ref(&connective))
            .expect("node");
    let reflected =
        coproduct_reflection::arm_payload_pairs_from_marshaled_node(&ctx, &node).expect("pairs");
    let syntactic =
        coproduct_reflection::eval_syntactic_coproduct_arm_pairs(&ctx, &[connective]).expect("syn");
    let syntactic_pairs: Vec<(String, String)> = match &syntactic {
        Value::List(items) => items
            .iter()
            .map(|v| {
                let Value::Record { fields, .. } = v else {
                    panic!("pair record");
                };
                let label = match ctx.field(fields, "label") {
                    Some(Value::Str(s)) => s.clone(),
                    _ => panic!("label"),
                };
                let payload = match ctx.field(fields, "payload_type_name") {
                    Some(Value::Str(s)) => s.clone(),
                    _ => panic!("payload"),
                };
                (label, payload)
            })
            .collect(),
        _ => panic!("expected list"),
    };
    let reflected_pairs: Vec<(String, String)> = reflected
        .iter()
        .map(|p| (p.label.clone(), p.payload_type_name.clone()))
        .collect();
    assert_eq!(reflected_pairs, syntactic_pairs, "rust-side pair sets");
}

#[test]
fn coproduct_reflection_path3_pair_witness_fails_on_perturbed_atom_payload_type() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let connective = (None, Value::Str("Connective".to_string()));
    let node =
        coproduct_reflection::eval_resolve_type_node(&ctx, std::slice::from_ref(&connective))
            .expect("node");
    let mut pairs =
        coproduct_reflection::arm_payload_pairs_from_marshaled_node(&ctx, &node).expect("pairs");
    pairs[0].payload_type_name = "{ identity: Int }".to_string();
    let syntactic =
        coproduct_reflection::eval_syntactic_coproduct_arm_pairs(&ctx, &[connective]).expect("syn");
    let syntactic_pairs: Vec<(String, String)> = match &syntactic {
        Value::List(items) => items
            .iter()
            .map(|v| {
                let Value::Record { fields, .. } = v else {
                    panic!("pair");
                };
                (
                    match ctx.field(fields, "label") {
                        Some(Value::Str(s)) => s.clone(),
                        _ => panic!("label"),
                    },
                    match ctx.field(fields, "payload_type_name") {
                        Some(Value::Str(s)) => s.clone(),
                        _ => panic!("payload"),
                    },
                )
            })
            .collect(),
        _ => panic!("list"),
    };
    let reflected_pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|p| (p.label.clone(), p.payload_type_name.clone()))
        .collect();
    assert_ne!(
        reflected_pairs, syntactic_pairs,
        "perturbed Atom payload-type must break pair bag_eq"
    );
}

#[test]
fn coproduct_reflection_path3_witness_fails_on_dropped_disj_arm() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let connective = (None, Value::Str("Connective".to_string()));

    let reflection_node =
        coproduct_reflection::eval_resolve_type_node(&ctx, std::slice::from_ref(&connective))
            .expect("resolved node");
    let corrupted =
        coproduct_reflection::eval_resolve_type_node_with_dropped_last_arm(&ctx, "Connective")
            .expect("corrupted node");
    let syntactic = coproduct_reflection::eval_syntactic_coproduct_arm_keys(&ctx, &[connective])
        .expect("syntactic keys");

    let good_keys = coproduct_reflection::arm_labels_from_marshaled_node(&ctx, &reflection_node)
        .expect("good keys");
    let bad_keys =
        coproduct_reflection::arm_labels_from_marshaled_node(&ctx, &corrupted).expect("bad keys");
    let syntactic_keys = symbol_list_strings(&syntactic);

    assert_eq!(
        good_keys, syntactic_keys,
        "baseline keys must match syntactic"
    );
    assert_ne!(
        bad_keys, syntactic_keys,
        "dropped Disj arm must break Path-3 bag_eq witness"
    );
}

const CONCEPT_INDEX_ENTRY: &str = "src/v2/test/claim/concept_index_enumeration_test.dag";

#[test]
fn concept_index_parse_only_perturb_witnesses_hold() {
    let roots: Vec<String> = v2_source_roots()
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let entry = workspace_root()
        .join(CONCEPT_INDEX_ENTRY)
        .to_string_lossy()
        .into_owned();
    let (graph, si) = cli_run::resolve_entry_graph(&roots, &entry).expect("resolve entry");
    let ctx = cli_run::make_eval_context(&graph, si, ExecutionMode::Wet);
    let outcome = cli_run::run_claim(&ctx, "concept_index_enumeration_witnesses");
    assert_eq!(
        outcome,
        cli_run::ClaimOutcome::Pass,
        "concept_index_enumeration_witnesses must pass (includes parse-only perturb-RED)"
    );
}
