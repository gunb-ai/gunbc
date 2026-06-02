//! Boundary emit-gate check functions shared by hand-Rust `#[test]` harnesses and
//! the `boundary_emit_gates` `bin` (`ExecuteCommand` logical child for `.dag`
//! `TestClaim` wrappers under `tests/dag/`).
//!
//! Each `check_*` returns `Ok(())` or `Err(String)`. The bin maps to exit 0/1;
//! `ExecuteCommand` reads only the exit code.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::compile_to_dag;
use crate::emit::emit_python_text;
use crate::emit_rust::emit_rust_module;

const M2_PROBE_DAG: &str = "\
module test.emit.multi_field_struct_variant

import std.substrate { Declaration, TypeConnective, ArrowBody }

fn is_arrow_pending(decl: Declaration) -> Bool =
  match decl.connective {
    Atom(payload) => false
    Conj(c) => false
    Disj(d) => false
    Arrow(a) => is_pending(a.body)
    Cardinality(c) => false
    Instantiation(i) => false
  }

fn is_pending(body: ArrowBody) -> Bool =
  match body {
    UserDefined(n) => false
    ExternalRealization(e) => false
    Pending => true
    NoBody => false
    Unparsed(s) => false
  }
";

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);

fn next_roundtrip_dir() -> PathBuf {
    let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "v3_boundary_emit_roundtrip_{}_{}",
        std::process::id(),
        id
    ))
}

fn emit_m2_probe_rust() -> Result<String, String> {
    let dag = compile_to_dag(M2_PROBE_DAG, "probe.dag")
        .map_err(|e| format!("probe .dag compile failed: {e:?}"))?;
    if !dag.diagnostics().is_empty() {
        return Err(format!(
            "probe .dag must compile cleanly, got {:?}",
            dag.diagnostics()
        ));
    }
    emit_rust_module(&dag).map_err(|e| format!("emit Rust module failed: {e:?}"))
}

/// Host receipt: `m2_emit_multi_field_struct_variant_test::multi_field_struct_variant_match_emits_aliased_field_destructure`.
pub fn check_m2_multi_field_struct_variant_destructure_alias() -> Result<(), String> {
    let rust = emit_m2_probe_rust()?;
    if !rust.contains("body: __a_body") {
        return Err(format!(
            "expected `body: __a_body` destructure in emitted Rust; got:\n{rust}"
        ));
    }
    if rust.contains("body: _,") || rust.contains("body: _ }") {
        return Err(format!(
            "wildcard `body: _` would drop the binding the fix routes through; got:\n{rust}"
        ));
    }
    Ok(())
}

/// Host receipt: `m2_emit_multi_field_struct_variant_test::multi_field_struct_variant_arm_body_uses_aliased_reference`.
pub fn check_m2_multi_field_struct_variant_arm_aliased_ref() -> Result<(), String> {
    let rust = emit_m2_probe_rust()?;
    if !rust.contains("__a_body") {
        return Err(format!(
            "emitted arm body must reference the aliased `__a_body`; got:\n{rust}"
        ));
    }
    if rust.contains("(a).body") || rust.contains("((a).body)") {
        return Err(format!(
            "pre-fix `(a).body` access must not appear in emitted Rust; got:\n{rust}"
        ));
    }
    Ok(())
}

/// Host receipt: `m2_emit_multi_field_struct_variant_test::multi_field_struct_variant_emitted_rust_is_valid_syntax`.
pub fn check_m2_multi_field_struct_variant_rustfmt_valid() -> Result<(), String> {
    let rust = emit_m2_probe_rust()?;
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn rustfmt: {e}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "rustfmt stdin".to_string())?
        .write_all(rust.as_bytes())
        .map_err(|e| format!("write to rustfmt: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait rustfmt: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "emitted Rust did not parse through rustfmt; stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn python_stdout(source: &str, file_name: &str) -> Result<String, String> {
    let dag = compile_to_dag(source, file_name)
        .map_err(|e| format!("compiled python program source: {e:?}"))?;
    let rendered = emit_python_text(&dag).map_err(|e| format!("emit python program: {e:?}"))?;
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("create tmp dir: {e}"))?;
    let src_path = tmp_dir.join("main.py");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .map_err(|e| format!("write emitted python source: {e}"))?;

    let run = Command::new("python3")
        .arg(&src_path)
        .output()
        .map_err(|e| format!("run python3: {e}"))?;
    if !run.status.success() {
        return Err(format!(
            "python3 failed on emitted source:\n{rendered}\nstderr:\n{}",
            String::from_utf8_lossy(&run.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

/// Host receipt: `m1_4_emit_python_test::emit_python_checked_division_roundtrips_ok_and_errors`.
///
/// Structural migration target for `t_pb_b_1_execute_command_boundary.dag` (shell smoke only);
/// this check exercises real CPython on emitted division programs.
pub fn check_python_checked_division_roundtrips() -> Result<(), String> {
    let ok = python_stdout("let x = 6 / 2\n", "python_div_ok.v3")?;
    if ok != "('Ok', 3)" {
        return Err(format!("6/2 roundtrip: expected `('Ok', 3)`, got `{ok}`"));
    }
    let zero = python_stdout("let x = 6 / 0\n", "python_div_zero.v3")?;
    if zero != "('Err', <DivError.DivideByZero: 0>)" {
        return Err(format!(
            "6/0 roundtrip: expected `('Err', <DivError.DivideByZero: 0>)`, got `{zero}`"
        ));
    }
    Ok(())
}
