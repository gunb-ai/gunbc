//! Emitter-gap witness: a generic fn's `fold` over `List<List<Measure<Time, S, Nat>>>`
//! must thread the fn's type-param environment into nested element-type rendering so `S`
//! survives at arbitrary nesting depth — the same generic, one container level apart from
//! `List<Measure<Time, S, Nat>>` (which already worked via `time_measure_list_par`).
//!
//! Pre-fix: the fold lambda's `batch` parameter rendered as
//! `compile_error!("UNRESOLVED_CompilerError")` inside `Measure<Time, …, Nat>`.
//! Post-fix: `S` renders as the fn type param. Genuinely-unresolved slots must still refuse.

use crate::helpers::{assert_no_diagnostics, compile_dag_named, diagnostic_messages};
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module nestedfold.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Quantity = Time | Memory\n",
    "type Scale = One | Micro\n\n",
    "type Measure<Q, S, M> {\n",
    "  count: M\n",
    "}\n\n",
    "fn time_measure<S>(count: Nat) -> Measure<Time, S, Nat> {\n",
    "  Measure { count: count }\n",
    "}\n\n",
    "fn schedule_critical_path<S>(batch_times: List<List<Measure<Time, S, Nat>>>) -> Measure<Time, S, Nat> {\n",
    "  fold(\n",
    "    batch_times,\n",
    "    init: time_measure(count: 0),\n",
    "    f: (acc, batch) => fold(\n",
    "      batch,\n",
    "      init: time_measure(count: 0),\n",
    "      f: (a, t) => a\n",
    "    )\n",
    "  )\n",
    "}\n\n",
    "fn flat_control<S>(times: List<Measure<Time, S, Nat>>) -> Measure<Time, S, Nat> {\n",
    "  fold(times, init: time_measure(count: 0), f: (acc, t) => acc)\n",
    "}\n"
);

const REFUSAL_FIXTURE: &str = concat!(
    "module nestedfold.refusal\n",
    "import std.nat { Nat }\n\n",
    "type Quantity = Time\n",
    "type Measure<Q, S, M> {\n",
    "  count: M\n",
    "}\n\n",
    "fn unresolved_inner(items: List<List<Measure<Time, Ghost, Nat>>>) -> Int {\n",
    "  fold(items, init: 0, f: (acc, batch) => acc)\n",
    "}\n"
);

fn emit_host() -> String {
    let result = compile_dag_named(
        "src/v1/nested_fold_generic_param_fixture.dag",
        FIXTURE,
        RenderTarget::Rust,
    );
    assert_no_diagnostics(&result);
    result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn fn_body(emitted: &str, name: &str) -> String {
    let needle = format!("fn {name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("fn `{name}` not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest[needle.len()..]
        .find("\npub fn ")
        .map(|i| i + needle.len())
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn nested_fold_threads_generic_param_into_element_type() {
    let emitted = emit_host();
    assert!(
        !emitted.contains("UNRESOLVED_CompilerError"),
        "nested fold must not emit UNRESOLVED_CompilerError after generic env threading:\n{emitted}"
    );
    let nested = fn_body(&emitted, "schedule_critical_path");
    assert!(
        nested.contains(", S, Nat>") || nested.contains(",S,Nat>"),
        "nested fold lambda must render fn type param S in inner Measure, got:\n{nested}"
    );
    assert!(
        !nested.contains("UNRESOLVED_CompilerError"),
        "nested fold batch param must not refuse S:\n{nested}"
    );
    let flat = fn_body(&emitted, "flat_control");
    assert!(
        flat.contains(", S, Nat>") || flat.contains(",S,Nat>"),
        "flat fold control must still render S, got:\n{flat}"
    );
}

#[test]
fn genuinely_unresolved_type_arg_still_refuses() {
    let result = compile_dag_named(
        "src/v1/nested_fold_generic_param_refusal_fixture.dag",
        REFUSAL_FIXTURE,
        RenderTarget::Rust,
    );
    let emitted = result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !emitted.is_empty() || !diagnostic_messages(&result).is_empty(),
        "refusal fixture should emit or diagnose, got neither"
    );
    if emitted.contains("unresolved_inner") {
        let broken = fn_body(&emitted, "unresolved_inner");
        assert!(
            broken.contains("UNRESOLVED_CompilerError"),
            "undeclared `Ghost` type arg must still emit UNRESOLVED_CompilerError (no widen), got:\n{broken}"
        );
    } else {
        assert!(
            diagnostic_messages(&result)
                .iter()
                .any(|m| m.contains("Ghost") || m.contains("not found") || m.contains("unknown")),
            "undeclared Ghost must fail closed at infer, got: {:?}",
            diagnostic_messages(&result)
        );
    }
}
