//! **Layer:** boundary
//!
//! R3 gate #71 — `v3_self_host_demonstration` (T-V2-Retirement).
//!
//! [`compile_to_dag`] + [`TestRunner`] evaluates a [`TestClaim`] whose predicate is
//! [`ExecuteCommand`] over the `self_host_fixed_point` binary — the PB-Runtime bounded
//! host-spawn trampoline (`docs/r3-structure.md` demonstration principle).
//!
//! The `.dag` template carries `__SELF_HOST_FIXED_POINT_BIN__`; substitution uses
//! `env!("CARGO_BIN_EXE_self_host_fixed_point")` at integration-test compile time (R1
//! Closure `#973` discipline, parallel to `r1c_e_emit_gates_dag_test` / `r1c_e_emit_gates.template.dag`).
//!
//! **Gate strength:** `self_host_fixed_point` exits 0 on DB-8 staged paths when
//! `compiler.dag` fails to parse; this test therefore **also** parses
//! `target/self_host/receipt.json` and requires `compiler_dag_v3_parse == ok`,
//! `fixed_point_diff == ok`, and `status == completed` so the demonstration cannot pass
//! without the full self-host fixed-point slice.
//!
//! **Toolchain:** the logical child runs DB-8’s staged ratchet and invokes `rustc` on the
//! emitted stage1 when `dsl/gunbc/compiler.dag` parses — full Rust toolchain required.

use std::path::PathBuf;

use serde_json::Value;
use v3_compiler::compile_to_dag;
use v3_compiler::self_host_receipt_p0 as receipt_p0;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::CompileError;

const TEMPLATE: &str = include_str!("../dag/r3_v3_self_host_demonstration.template.dag");
const TEMPLATE_PATH: &str = "src/v3/compiler/tests/dag/r3_v3_self_host_demonstration.template.dag";
const BIN_PATH: &str = env!("CARGO_BIN_EXE_self_host_fixed_point");
const BIN_PLACEHOLDER: &str = "__SELF_HOST_FIXED_POINT_BIN__";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn substituted_dag_source() -> String {
    assert!(
        TEMPLATE.contains(BIN_PLACEHOLDER),
        "template must contain `{BIN_PLACEHOLDER}` placeholder for bin substitution \
         (see `r1c_e_emit_gates.template.dag` discipline): {TEMPLATE_PATH}"
    );
    TEMPLATE.replace(BIN_PLACEHOLDER, BIN_PATH)
}

/// R3 §1.8 gate #71 — receipt must prove the full DB-8 slice, not probe-only exit 0.
fn assert_receipt_proves_self_host_fixed_point_slice() {
    let receipt_path = workspace_root().join("target/self_host/receipt.json");
    let body = std::fs::read_to_string(&receipt_path).unwrap_or_else(|e| {
        panic!(
            "gate #71: read receipt {} (run self_host_fixed_point first): {e}",
            receipt_path.display()
        );
    });
    let v: Value = serde_json::from_str(&body).unwrap_or_else(|e| {
        panic!(
            "gate #71: receipt {} is not valid JSON: {e}",
            receipt_path.display()
        );
    });

    assert_eq!(
        v.get(receipt_p0::K_COMPILER_DAG_V3_PARSE)
            .and_then(Value::as_str),
        Some("ok"),
        "gate #71 requires v3 parse of `dsl/gunbc/compiler.dag` (receipt {:?})",
        v.get(receipt_p0::K_COMPILER_DAG_V3_PARSE)
    );
    assert_eq!(
        v.get("fixed_point_diff").and_then(Value::as_str),
        Some("ok"),
        "gate #71 requires emit→rustc→run→byte-identical fixed-point receipt field fixed_point_diff==ok; got {:?}",
        v.get("fixed_point_diff")
    );
    assert_eq!(
        v.get(receipt_p0::K_STATUS).and_then(Value::as_str),
        Some("completed"),
        "gate #71 requires status completed (no failed_self_host_slice); got {:?}",
        v.get(receipt_p0::K_STATUS)
    );
}

#[test]
fn r3_v3_self_host_demonstration_suite_passes_through_runner() {
    let source = substituted_dag_source();
    let dag = match compile_to_dag(&source, TEMPLATE_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{TEMPLATE_PATH} (substituted): expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "{TEMPLATE_PATH} (substituted) should lower without module diagnostics. \
                 Got `Err(Semantic)`: {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
        }
        Err(other) => panic!("unexpected compile error for {TEMPLATE_PATH}: {other:?}"),
    };

    let results = TestRunner::new(&dag).run_suite("suite_v3_self_host_demonstration");
    assert!(
        !results.is_empty(),
        "suite `suite_v3_self_host_demonstration` should contain at least one claim"
    );
    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.result != ClaimResult::Pass)
        .collect();
    assert!(
        failures.is_empty(),
        "suite_v3_self_host_demonstration: {} claim(s) did not Pass:\n{:#?}",
        failures.len(),
        failures
    );

    assert_receipt_proves_self_host_fixed_point_slice();
}
