//! **Layer:** integration
//!
//! **W2 / joint spec** `compiler-spine-runtime-rung34-min-runner-interface-2026-05-30.md` §4.2:
//! emit-host harness substrate (`emit_host.dag`, `host_run.dag`, `test_claim_falsification.dag`,
//! `nat_semiring_rung34_eval.dag`) + executable boundary `tools/emit_host_runner`.
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-38 / rung-4 host receipt path.
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` land in the same PR. **This PR (+1 census path):**
//! `v4_emit_host_harness_test.rs` — behavior-driven `run_emit_host_rust` (compile + run fixture,
//! `HostExit` Holds witness + five-byte stdout parse) plus **tokenize/parse** surface receipts
//! for W2 `.dag` modules (not `str::contains` source probes per TESTING.md).
//!
//! **Dissolution trigger:** W3 populates `nat_semiring_rung34_runtime_value_rows`
//! and wires `run_emit_host_rust` in `emit_host.dag` to invoke `tools/emit_host_runner` (removes
//! `emit_host_transport_not_wired` on the Rust row); delete this file when rung-3/4 claims +
//! generated harness replace hand-Rust probes (see `nat_semiring_rung34_eval.dag` roster comment).
//!
//! **TESTING.md:** substrate `.dag` eval remains hermetic (`run_emit_host_rust` → `Rejected` until
//! W3/CI wiring); this test exercises the Rust transport the `.dag` row models, not substrate eval.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceVariant};
use v3_compiler::tokenize_for_test;

const EMIT_HOST_DAG: &str = include_str!("../../../../v4/compiler/emit_host.dag");
const EMIT_HOST_PATH: &str = "src/v4/compiler/emit_host.dag";
const HOST_RUN_DAG: &str = include_str!("../../../../v4/std/host_run.dag");
const HOST_RUN_PATH: &str = "src/v4/std/host_run.dag";
const FALSIFICATION_DAG: &str = include_str!("../../../../v4/std/test_claim_falsification.dag");
const FALSIFICATION_PATH: &str = "src/v4/std/test_claim_falsification.dag";
const NAT_SEMIRING_RUNG34_EVAL_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/nat_semiring_rung34_eval.dag");
const NAT_SEMIRING_RUNG34_EVAL_PATH: &str =
    "src/v4/test/claim/workflow/nat_semiring_rung34_eval.dag";

/// Minimal fixture: five stdout bytes (MVP runtime value `5` alignment).
const EMIT_HOST_FIXTURE_SOURCE: &str =
    "fn main() { let _ = std::io::Write::write_all(&mut std::io::stdout(), &[0u8; 5]); }";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: item_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum { name: decl_name, .. }
                | SurfaceItem::TypeRecord { name: decl_name, .. }
                | SurfaceItem::TypeAlias { name: decl_name, .. }
                | SurfaceItem::TypeAtom { name: decl_name, .. }
                if decl_name == name
        )
    })
}

fn type_record_field_names(
    module: &v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
) -> Vec<String> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord { name, fields, .. } if name == type_name => {
                Some(
                    fields
                        .iter()
                        .map(|field| field.name.clone())
                        .collect::<Vec<_>>(),
                )
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{type_name}: missing type record"))
}

fn type_sum_variant<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
    variant_name: &str,
) -> &'a SurfaceVariant {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum { name, variants, .. } if name == type_name => {
                variants.iter().find(|variant| variant.name == variant_name)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing `{type_name}.{variant_name}` variant"))
}

#[test]
fn emit_host_runner_rust_row_builds_runs_and_parses_stdout() {
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_v4_emit_host_harness_{}",
        std::process::id()
    ));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "w2_harness_claim_input".to_string(),
        expected_eval_root: "w2_harness_expected_eval".to_string(),
    };
    let receipt =
        emit_host_runner::run_emit_host_rust(EMIT_HOST_FIXTURE_SOURCE, &inputs, &work_dir)
            .expect("run_emit_host_rust");
    assert!(
        receipt.exit.exit_holds(),
        "expected successful host exit (Holds witness), got {:?}",
        receipt.exit
    );
    assert!(
        emit_host_runner::host_logical_run_from_exit(&receipt.exit, receipt.stdout_bytes.clone())
            .is_some(),
        "logical_run projection requires Holds exit"
    );
    emit_host_runner::runtime_value_parse_rust(&receipt.stdout_bytes)
        .expect("runtime_value_parse_rust on fixture stdout");
}

#[test]
fn v4_host_run_dag_tokenizes_and_parses_logical_run_carrier() {
    let module = parse_module(HOST_RUN_DAG, HOST_RUN_PATH);
    assert!(
        surface_declares_type(&module, "HostRunStdout"),
        "{HOST_RUN_PATH}: HostRunStdout carrier"
    );
    assert!(
        surface_declares_type(&module, "HostLogicalRun"),
        "{HOST_RUN_PATH}: HostLogicalRun carrier"
    );
    assert_eq!(
        type_record_field_names(&module, "HostLogicalRun"),
        vec!["stdout".to_string()],
        "{HOST_RUN_PATH}: success-only logical run (no nested Witness)"
    );
    assert!(
        surface_declares_fn(&module, "host_logical_run_from_exit"),
        "{HOST_RUN_PATH}: host_logical_run_from_exit"
    );
}

#[test]
fn v4_falsification_dag_tokenizes_and_parses_execution_evidence() {
    let module = parse_module(FALSIFICATION_DAG, FALSIFICATION_PATH);
    assert!(
        surface_declares_type(&module, "FalsificationReceipt"),
        "{FALSIFICATION_PATH}: FalsificationReceipt"
    );
    let receipt_fields = type_record_field_names(&module, "FalsificationReceipt");
    assert!(
        receipt_fields.contains(&"subject".to_string()),
        "{FALSIFICATION_PATH}: FalsificationReceipt.subject"
    );
    for variant in ["Host", "Interpreter", "EvidenceNone"] {
        let _ = type_sum_variant(&module, "ExecutionEvidence", variant);
    }
}

#[test]
fn v4_emit_host_dag_tokenizes_and_parses_fail_closed_surface() {
    let module = parse_module(EMIT_HOST_DAG, EMIT_HOST_PATH);
    for name in [
        "run_emit_host_rust",
        "run_emit_host",
        "host_exit_failure_outcome",
        "run_test_claim_emit_vs_eval_for_claim",
        "run_test_claim_emit_vs_eval",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{EMIT_HOST_PATH}: missing fn {name}"
        );
    }
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::Data { name, .. } if name == "emit_host_transport_not_wired"
        )),
        "{EMIT_HOST_PATH}: transport_not_wired symbol"
    );
}

#[test]
fn v4_nat_semiring_rung_gate_dag_tokenizes_and_parses_empty_roster_gates() {
    let module = parse_module(NAT_SEMIRING_RUNG34_EVAL_DAG, NAT_SEMIRING_RUNG34_EVAL_PATH);
    for name in [
        "nat_semiring_rung34_report_has_evidence",
        "nat_semiring_rung3_gate",
        "nat_semiring_rung4_gate",
        "run_nat_semiring_rung34_eval",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{NAT_SEMIRING_RUNG34_EVAL_PATH}: missing fn {name}"
        );
    }
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::Data { name, .. } if name == "nat_semiring_rung34_runtime_value_rows"
        )),
        "{NAT_SEMIRING_RUNG34_EVAL_PATH}: empty runtime roster"
    );
}
