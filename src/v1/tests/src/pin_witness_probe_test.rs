// PROBE ONLY (branch probe/pin-witness; never ships). Finds which in-tree seed construct reaches a
// minted TypeVariable, so the boundary guard's walk can be SHOWN to see one on main rather than being
// green because it looks at nothing (review 83183: "a check whose forbidden state is expressible
// nowhere it runs is a decoration").
//
// Candidate minting sites in main's own 04_infer.dag:
//   :11556 unbound_owner_param_variables  -> type_variable_node(id: g)   (record literal, name-keyed)
//   :8884  builtin empty_set element       -> type_variable_node(id: "set_elem")
// The guard's own fixture (time_measure(count: 10)) is the negative control: it must find NONE.
use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};
use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_std_core::Node;

fn walk(n: &Rc<Node>, depth: u32, out: &mut Vec<String>) {
    if depth > 12 || out.len() > 400 {
        return;
    }
    if let Some(inferred) = &n.inferred {
        if let v1_compiler::v1_std_core::InferredNode::TypeVariable { id } = &**inferred {
            out.push(id.clone());
        }
    }
    for c in n.children.iter() {
        walk(c, depth + 1, out);
    }
    if let Some(b) = &n.body {
        walk(b, depth + 1, out);
    }
    for p in n.params.iter() {
        walk(p, depth + 1, out);
    }
}

fn run(label: &str, source: &str, target: &str) {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources = resolve_imports_transitively_with_source_roots("probe.dag", source, &roots);
    let resolved: Rc<ResolvedPipelineResult> = compile_to_resolved(Rc::new(sources.into()));
    let mut ids: Vec<String> = Vec::new();
    let mut saw = false;
    if let Some(graph) = &resolved.graph {
        for m in graph.modules.iter() {
            for item in m.items.iter() {
                if item.name == target {
                    saw = true;
                    walk(item, 0, &mut ids);
                }
            }
        }
    }
    let hard: Vec<String> = resolved
        .diagnostics
        .iter()
        .filter(|d| {
            !matches!(
                d,
                v1_compiler::v1_compiler_diagnostic::Diagnostic::UnusedDeclaration { .. }
            )
        })
        .map(|d| format!("{d:?}"))
        .collect();
    let mut uniq: Vec<&String> = ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    uniq.sort();
    println!(
        "WITNESS {label} saw_fn={saw} positions={} distinct={:?} hard_diags={} sample_diags={:?}",
        ids.len(),
        uniq,
        hard.len(),
        hard.iter().take(2).collect::<Vec<_>>()
    );
}

#[test]
fn probe_witness_candidates() {
    run(
        "record_literal_unannotated_generic",
        r#"
module probe.a

import std.measure { Measure }

fn probe_target() -> Nat {
  let m = Measure { count: 0 }
  m.count
}
"#,
        "probe_target",
    );
    run(
        "record_literal_annotated_generic",
        r#"
module probe.b

import std.measure { Measure }

fn probe_target() -> Nat {
  let m: Measure<Nat, Nat, Nat> = Measure { count: 0 }
  m.count
}
"#,
        "probe_target",
    );
    run(
        "builtin_empty_set_then_insert",
        r#"
module probe.c

import std.set { empty_set, set_insert }

fn probe_target() -> Nat {
  let s = set_insert(empty_set(), 1)
  count(xs: s.items)
}
"#,
        "probe_target",
    );
    run(
        "pin_negative_control_time_measure",
        r#"
module probe.d

import std.measure { time_measure }

fn probe_target() -> Bool {
  let m = time_measure(count: 10)
  m.count == 0
}
"#,
        "probe_target",
    );
}
