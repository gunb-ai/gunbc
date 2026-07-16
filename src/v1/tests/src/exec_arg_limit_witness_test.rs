//! Wet demonstration of the host argv-size wall (Deploy E2BIG, Part B).
//!
//! Pairs with the floor witness `dag/test/claim/exec_arg_limit_witness_test.dag` by
//! exact stem (`exec_arg_limit_witness`) so the v1 test-debt ratchet counts this
//! module as covered, not new debt.
//!
//! Runs the REAL `extdeps.shell` service through the interpreter in WET mode
//! (`v1_interpreter::run` defaults to `ExecutionMode::Wet`, so shell transports
//! dispatch to a real `sh` spawn — no mock). `shell.Exec.Check` embeds its
//! command in argv (`sh -c "{command}"`), so it is the surface on which an
//! oversized single argument would otherwise hit `execve`'s E2BIG (`os error 7`).
//!
//! The wall (v1_interpreter::dispatch_shell) turns that opaque failure into a
//! typed, located `InterpError::ArgvExceedsHostArgMax` refusal, decided from the
//! model before the spawn. Discriminating pair (DESIGN §5): an oversized command
//! refuses (RED path proven), a small command still runs wet (GREEN control).
//!
//! Routing a large payload through stdin instead of argv (so it never reaches the
//! wall) is the separate deploy fix in `shell.Exec.Run` (PR #6711) — not exercised
//! here; this file proves only the general host-boundary guard.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, InterpError, Value, HOST_ARG_MAX_STRLEN_BYTES};

use crate::helpers::resolve_imports_transitively;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

fn run_entry(src: &str, entry_fn: &str) -> v1_interpreter::InterpResult<Value> {
    let sources = resolve_imports_transitively("test/exec_arg_limit_wet.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    v1_interpreter::run(graph, resolved.source_indices.clone(), entry_fn)
}

/// RED path: a single argv element over the host per-argument ceiling refuses with
/// the typed diagnostic — carrying the offending byte count and the modeled limit —
/// rather than letting execve die with an opaque os error 7 (E2BIG). This is the
/// demonstration the operator asked to see, on the real shell service, wet.
#[test]
fn oversized_argv_command_refuses_typed_not_os_error_7() {
    let command = "x".repeat(HOST_ARG_MAX_STRLEN_BYTES + 1);
    let src = format!(
        r#"module test.exec_arg_limit_wet

import extdeps.shell

test fn check_oversized_command() -> Bool {{
  shell.Exec.Check(command: "{command}").exists
}}
"#
    );
    match run_entry(&src, "check_oversized_command") {
        Err(InterpError::ArgvExceedsHostArgMax {
            actual_bytes,
            limit_bytes,
            argv0,
        }) => {
            assert_eq!(actual_bytes, HOST_ARG_MAX_STRLEN_BYTES + 1);
            assert_eq!(limit_bytes, HOST_ARG_MAX_STRLEN_BYTES);
            assert_eq!(argv0, "sh");
        }
        other => {
            panic!("expected typed ArgvExceedsHostArgMax refusal (not os error 7), got {other:?}")
        }
    }
}

/// GREEN control: a small argv command passes the wall and executes wet — the wall
/// does not false-positive on ordinary probes, and the shell dispatch still works.
#[test]
fn small_argv_command_still_runs_wet() {
    let src = r#"module test.exec_arg_limit_wet

import extdeps.shell

test fn check_true() -> Bool {
  shell.Exec.Check(command: "true").exists
}
"#;
    match run_entry(src, "check_true") {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected small argv command to run wet and succeed, got {other:?}"),
    }
}
