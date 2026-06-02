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
//! **W3:** substrate `run_emit_host_rust` stays `transport_not_wired` (fail-closed); real cargo+run
//! via `emit_host_bridge` / `emit_host_runner`. Rosters authored in #4046 (`rung_3_4`).
//! **W3.3 (+0 SG-0 paths):** go transport + rung-5 full law roster × rust/python/go
//! (`rung_5.dag`, `nat_semiring_rung5_eval.dag`); cross-target MVP-2 stdout parity via bridge.
//! **W3.4 (+0 SG-0 paths):** extends bridge with python transport + rung-6 post-emit law
//! preservation (`rung_6.dag`, `rung_5_6_common.dag`, `nat_semiring_rung56_eval.dag`).
//! Tranche-1 additive-Monoid + tranche-2 multiplicative-Monoid + annihilator (rust + python).
//! **Python L1/L2 (release-minimum):** rung-5 full-law roster python transport receipts,
//! worksheet-B falsification probes (runtime reject / parse fail / value mismatch), and
//! `scripts/v4-phase1-nat-semiring-python-runtime-gate.sh` for emitted-fixture execution.
//! **Go L1 (+0 paths, release-minimum):** `go_l1_nat_semiring_rung2` compiler-slice substrate
//! claim parse surface + `scripts/v4-phase1-nat-semiring-go-compiler-slice-gate.sh` (structured
//! JSON receipt; chained from rung gate after R2-go-compile). SG-0 + INVARIANTS §P5(b) in PR body.
//! Behavior receipts: MVP-2 emit-vs-eval `Pass` per law×target via `emit_host_bridge` (five-byte
//! stdout contract; not per-law emitted artifacts until emit pipeline wires law subjects).
//! Substrate rows stay `Deferred` until T-22 dispatch. Dissolution: **ROADMAP.md** T-PB-B /
//! **TASKS.md** T-22 T-38.
//!
//! **TESTING.md:** substrate `.dag` models receipt assembly; behavior tests exercise
//! `tools/emit_host_runner` / `emit_host_bridge` (real cargo + run).

use v3_compiler::emit_host_bridge;
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
const NAT_SEMIRING_RUNG_3_4_DAG: &str =
    include_str!("../../../../v4/test/claim/nat_semiring/rung_3_4.dag");
const NAT_SEMIRING_RUNG_3_4_PATH: &str = "src/v4/test/claim/nat_semiring/rung_3_4.dag";
const NAT_SEMIRING_RUNG_5_DAG: &str =
    include_str!("../../../../v4/test/claim/nat_semiring/rung_5.dag");
const NAT_SEMIRING_RUNG_5_PATH: &str = "src/v4/test/claim/nat_semiring/rung_5.dag";
const NAT_SEMIRING_RUNG_L1_PYTHON_RUNTIME_DAG: &str =
    include_str!("../../../../v4/test/claim/nat_semiring/rung_l1_python_runtime.dag");
const NAT_SEMIRING_RUNG_L1_PYTHON_RUNTIME_PATH: &str =
    "src/v4/test/claim/nat_semiring/rung_l1_python_runtime.dag";
const NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_DAG: &str =
    include_str!("../../../../v4/test/claim/nat_semiring/rung_l1_go_compiler_slice.dag");
const NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH: &str =
    "src/v4/test/claim/nat_semiring/rung_l1_go_compiler_slice.dag";
const NAT_SEMIRING_RUNG5_EVAL_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/nat_semiring_rung5_eval.dag");
const NAT_SEMIRING_RUNG5_EVAL_PATH: &str = "src/v4/test/claim/workflow/nat_semiring_rung5_eval.dag";
const NAT_SEMIRING_RUNG_6_DAG: &str =
    include_str!("../../../../v4/test/claim/nat_semiring/rung_6.dag");
const NAT_SEMIRING_RUNG_6_PATH: &str = "src/v4/test/claim/nat_semiring/rung_6.dag";
const NAT_SEMIRING_RUNG56_EVAL_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/nat_semiring_rung56_eval.dag");
const NAT_SEMIRING_RUNG56_EVAL_PATH: &str =
    "src/v4/test/claim/workflow/nat_semiring_rung56_eval.dag";
const RUNG_5_6_COMMON_DAG: &str = include_str!("../../../../v4/test/claim/rung_5_6_common.dag");
const RUNG_5_6_COMMON_PATH: &str = "src/v4/test/claim/rung_5_6_common.dag";
const NAT_SEMIRING_RUNG_8_DAG: &str =
    include_str!("../../../../v4/test/claim/nat_semiring/rung_8.dag");
const NAT_SEMIRING_RUNG_8_PATH: &str = "src/v4/test/claim/nat_semiring/rung_8.dag";
const NAT_SEMIRING_RUNG8_EVAL_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/nat_semiring_rung8_eval.dag");
const NAT_SEMIRING_RUNG8_EVAL_PATH: &str = "src/v4/test/claim/workflow/nat_semiring_rung8_eval.dag";
const LOOP_LINEAR_BOUND_RUNG_8_DAG: &str =
    include_str!("../../../../v4/test/claim/loop_linear_bound/rung_8.dag");
const LOOP_LINEAR_BOUND_RUNG_8_PATH: &str = "src/v4/test/claim/loop_linear_bound/rung_8.dag";
const LOOP_LINEAR_BOUND_RUNG8_EVAL_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/loop_linear_bound_rung8_eval.dag");
const LOOP_LINEAR_BOUND_RUNG8_EVAL_PATH: &str =
    "src/v4/test/claim/workflow/loop_linear_bound_rung8_eval.dag";
const BRANCH_DISPATCH_RUNG_8_DAG: &str =
    include_str!("../../../../v4/test/claim/branch_dispatch/rung_8.dag");
const BRANCH_DISPATCH_RUNG_8_PATH: &str = "src/v4/test/claim/branch_dispatch/rung_8.dag";
const BRANCH_DISPATCH_RUNG8_EVAL_DAG: &str =
    include_str!("../../../../v4/test/claim/workflow/branch_dispatch_rung8_eval.dag");
const BRANCH_DISPATCH_RUNG8_EVAL_PATH: &str =
    "src/v4/test/claim/workflow/branch_dispatch_rung8_eval.dag";

/// Minimal python host fixture: five stdout bytes (MVP runtime value `5` alignment).
const EMIT_HOST_PYTHON_FIXTURE_SOURCE: &str = "import sys\nsys.stdout.buffer.write(b'\\x00' * 5)\n";

/// Worksheet-B F1: py_compile accepts; CPython rejects at execution (NameError).
const PYTHON_FIXTURE_RUNTIME_REJECTED: &str = "raise NameError(\"probe F1 runtime rejected\")\n";

/// Worksheet-B F2: exit 0 but stdout not parseable as MVP-2 runtime value.
const PYTHON_FIXTURE_UNPARSABLE_STDOUT: &str =
    "import sys\nsys.stdout.buffer.write(b'\\x00\\x00\\x00')\n";

/// Worksheet-B F3: exit 0, parseable length, wrong runtime bytes.
const PYTHON_FIXTURE_VALUE_MISMATCH: &str =
    "import sys\nsys.stdout.buffer.write(b'\\x01\\x02\\x03\\x04\\x05')\n";

/// Minimal go host fixture: five stdout bytes (MVP runtime value `5` alignment).
const EMIT_HOST_GO_FIXTURE_SOURCE: &str =
    "package main\nimport \"os\"\nfunc main() { _, _ = os.Stdout.Write(make([]byte, 5)) }\n";

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

fn surface_declares_data(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::Data { name: decl_name, .. } if decl_name == name
        )
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
            SurfaceItem::TypeRecord { name, fields, .. } if name == type_name => Some(
                fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect::<Vec<_>>(),
            ),
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

/// Nat-semiring / branch_dispatch rung-4 row uses the same MVP-2 eval pins; one transport proof
/// covers both fixtures until per-fixture emit lands.
#[test]
fn nat_semiring_rung4_emit_vs_eval_wired_transport_passes() {
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_nat_semiring_rung4_{}",
        std::process::id()
    ));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "phase1_nat_semiring_claim_input".to_string(),
        expected_eval_root: "phase1_nat_semiring_expected_eval".to_string(),
    };
    let verdict = emit_host_bridge::run_emit_vs_eval_mvp2_transport(
        EMIT_HOST_FIXTURE_SOURCE,
        &inputs,
        &work_dir,
        emit_host_bridge::MVP2_RUNTIME_VALUE_FIVE_BYTES,
    )
    .expect("run_emit_vs_eval_mvp2_transport");
    assert_eq!(verdict, emit_host_bridge::EmitHostEmitVsEvalVerdict::Pass);
}

#[test]
fn nat_semiring_rung4_emit_vs_eval_falsification_on_host_value_mismatch() {
    const MISMATCH_SOURCE: &str =
        "fn main() { let _ = std::io::Write::write_all(&mut std::io::stdout(), &[1,2,3,4,5]); }";
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_nat_semiring_rung4_fail_{}",
        std::process::id()
    ));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "phase1_nat_semiring_claim_input".to_string(),
        expected_eval_root: "phase1_nat_semiring_expected_eval".to_string(),
    };
    let verdict = emit_host_bridge::run_emit_vs_eval_mvp2_transport(
        MISMATCH_SOURCE,
        &inputs,
        &work_dir,
        emit_host_bridge::MVP2_RUNTIME_VALUE_FIVE_BYTES,
    )
    .expect("run_emit_vs_eval_mvp2_transport");
    assert!(
        matches!(
            verdict,
            emit_host_bridge::EmitHostEmitVsEvalVerdict::FailValueMismatch { .. }
        ),
        "expected structured Fail with Host receipt evidence, got {verdict:?}"
    );
}

/// Tranche-1 bridge receipt only: same MVP-2 fixture per target (five-byte stdout), not per-law emit.
/// `claim_input_root` pins the roster row's law subject symbol until emit wires law bodies; test names
/// mirror `.dag` row ids (`run_phase1_nat_semiring_rung6_*`) — they do not yet prove law semantics.
fn assert_rung6_mvp2_emit_vs_eval_pass(
    work_dir_prefix: &str,
    claim_input_root: &str,
    run_transport: fn(
        &str,
        &emit_host_runner::EmitHostFixtureInputs,
        &std::path::Path,
        [u8; 5],
    ) -> Result<
        emit_host_bridge::EmitHostEmitVsEvalVerdict,
        emit_host_runner::HostSetupFailure,
    >,
    emitted_source: &str,
) {
    let work_dir =
        emit_host_runner::default_work_dir(&format!("{work_dir_prefix}_{}", std::process::id()));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: claim_input_root.to_string(),
        expected_eval_root: "phase1_nat_semiring_rung6_expected_eval".to_string(),
    };
    let verdict = run_transport(
        emitted_source,
        &inputs,
        &work_dir,
        emit_host_bridge::MVP2_RUNTIME_VALUE_FIVE_BYTES,
    )
    .expect("emit_vs_eval transport");
    assert_eq!(verdict, emit_host_bridge::EmitHostEmitVsEvalVerdict::Pass);
}

/// Roster-row receipt (law pin + rust transport); MVP-2 stdout only — see `assert_rung6_mvp2_emit_vs_eval_pass`.
#[test]
fn nat_semiring_rung6_rust_add_left_identity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_rust_left_id",
        "nat_add_left_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_transport,
        EMIT_HOST_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_rust_add_right_identity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_rust_right_id",
        "nat_add_right_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_transport,
        EMIT_HOST_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_rust_add_associativity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_rust_assoc",
        "nat_add_associativity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_transport,
        EMIT_HOST_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_python_add_left_identity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_py_left_id",
        "nat_add_left_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_python_add_right_identity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_py_right_id",
        "nat_add_right_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_python_add_associativity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_py_assoc",
        "nat_add_associativity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_rust_mul_left_identity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_rust_mul_left_id",
        "nat_mul_left_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_transport,
        EMIT_HOST_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_rust_mul_associativity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_rust_mul_assoc",
        "nat_mul_associativity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_transport,
        EMIT_HOST_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_rust_mul_annihilator_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_rust_mul_ann",
        "nat_mul_annihilator_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_transport,
        EMIT_HOST_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_python_mul_left_identity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_py_mul_left_id",
        "nat_mul_left_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_python_mul_associativity_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_py_mul_assoc",
        "nat_mul_associativity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung6_python_mul_annihilator_emit_vs_eval_transport_passes() {
    assert_rung6_mvp2_emit_vs_eval_pass(
        "gunbc_rung6_py_mul_ann",
        "nat_mul_annihilator_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

/// Rung-5 full-law roster: same MVP-2 python transport contract as rung-6 rows.
fn assert_rung5_mvp2_emit_vs_eval_pass(
    work_dir_prefix: &str,
    claim_input_root: &str,
    run_transport: fn(
        &str,
        &emit_host_runner::EmitHostFixtureInputs,
        &std::path::Path,
        [u8; 5],
    ) -> Result<
        emit_host_bridge::EmitHostEmitVsEvalVerdict,
        emit_host_runner::HostSetupFailure,
    >,
    emitted_source: &str,
) {
    let work_dir =
        emit_host_runner::default_work_dir(&format!("{work_dir_prefix}_{}", std::process::id()));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: claim_input_root.to_string(),
        expected_eval_root: "phase1_nat_semiring_rung5_expected_eval".to_string(),
    };
    let verdict = run_transport(
        emitted_source,
        &inputs,
        &work_dir,
        emit_host_bridge::MVP2_RUNTIME_VALUE_FIVE_BYTES,
    )
    .expect("emit_vs_eval transport");
    assert_eq!(verdict, emit_host_bridge::EmitHostEmitVsEvalVerdict::Pass);
}

#[test]
fn nat_semiring_rung5_python_add_left_identity_emit_vs_eval_transport_passes() {
    assert_rung5_mvp2_emit_vs_eval_pass(
        "gunbc_rung5_py_left_id",
        "nat_add_left_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung5_python_add_right_identity_emit_vs_eval_transport_passes() {
    assert_rung5_mvp2_emit_vs_eval_pass(
        "gunbc_rung5_py_right_id",
        "nat_add_right_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung5_python_add_associativity_emit_vs_eval_transport_passes() {
    assert_rung5_mvp2_emit_vs_eval_pass(
        "gunbc_rung5_py_assoc",
        "nat_add_associativity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung5_python_mul_left_identity_emit_vs_eval_transport_passes() {
    assert_rung5_mvp2_emit_vs_eval_pass(
        "gunbc_rung5_py_mul_left_id",
        "nat_mul_left_identity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung5_python_mul_associativity_emit_vs_eval_transport_passes() {
    assert_rung5_mvp2_emit_vs_eval_pass(
        "gunbc_rung5_py_mul_assoc",
        "nat_mul_associativity_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn nat_semiring_rung5_python_mul_annihilator_emit_vs_eval_transport_passes() {
    assert_rung5_mvp2_emit_vs_eval_pass(
        "gunbc_rung5_py_mul_ann",
        "nat_mul_annihilator_input",
        emit_host_bridge::run_emit_vs_eval_mvp2_python_transport,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
    );
}

#[test]
fn python_emit_vs_eval_transport_fails_host_exit_on_runtime_rejection() {
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_py_runtime_reject_{}",
        std::process::id()
    ));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "probe_f1_claim_input".to_string(),
        expected_eval_root: "probe_f1_expected_eval".to_string(),
    };
    let verdict = emit_host_bridge::run_emit_vs_eval_mvp2_python_transport(
        PYTHON_FIXTURE_RUNTIME_REJECTED,
        &inputs,
        &work_dir,
        emit_host_bridge::MVP2_RUNTIME_VALUE_FIVE_BYTES,
    )
    .expect("transport setup");
    assert!(
        matches!(
            verdict,
            emit_host_bridge::EmitHostEmitVsEvalVerdict::FailHostExit { .. }
        ),
        "F1: compile-ok runtime NameError must yield FailHostExit with receipt, got {verdict:?}"
    );
}

#[test]
fn python_emit_vs_eval_transport_fails_parse_on_unparsable_stdout() {
    let work_dir =
        emit_host_runner::default_work_dir(&format!("gunbc_py_parse_fail_{}", std::process::id()));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "probe_f2_claim_input".to_string(),
        expected_eval_root: "probe_f2_expected_eval".to_string(),
    };
    let verdict = emit_host_bridge::run_emit_vs_eval_mvp2_python_transport(
        PYTHON_FIXTURE_UNPARSABLE_STDOUT,
        &inputs,
        &work_dir,
        emit_host_bridge::MVP2_RUNTIME_VALUE_FIVE_BYTES,
    )
    .expect("transport setup");
    assert!(
        matches!(
            verdict,
            emit_host_bridge::EmitHostEmitVsEvalVerdict::FailParse { .. }
        ),
        "F2: exit-0 unparsable stdout must yield FailParse, got {verdict:?}"
    );
}

#[test]
fn python_emit_vs_eval_transport_fails_on_runtime_value_mismatch() {
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_py_value_mismatch_{}",
        std::process::id()
    ));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "probe_f3_claim_input".to_string(),
        expected_eval_root: "probe_f3_expected_eval".to_string(),
    };
    let verdict = emit_host_bridge::run_emit_vs_eval_mvp2_python_transport(
        PYTHON_FIXTURE_VALUE_MISMATCH,
        &inputs,
        &work_dir,
        emit_host_bridge::MVP2_RUNTIME_VALUE_FIVE_BYTES,
    )
    .expect("transport setup");
    match verdict {
        emit_host_bridge::EmitHostEmitVsEvalVerdict::FailValueMismatch {
            host_receipt,
            host_stdout,
            expected_bytes,
        } => {
            assert!(emit_host_bridge::host_exit_holds(&host_receipt.exit));
            assert_eq!(host_stdout, [1, 2, 3, 4, 5]);
            assert_eq!(
                expected_bytes,
                emit_host_bridge::MVP2_RUNTIME_VALUE_FIVE_BYTES
            );
        }
        other => panic!("F3: expected FailValueMismatch with Host evidence, got {other:?}"),
    }
}

#[test]
fn v4_nat_semiring_rung_l1_python_runtime_dag_tokenizes_and_parses_claim_row() {
    let module = parse_module(
        NAT_SEMIRING_RUNG_L1_PYTHON_RUNTIME_DAG,
        NAT_SEMIRING_RUNG_L1_PYTHON_RUNTIME_PATH,
    );
    assert!(
        surface_declares_data(&module, "claim_phase1_nat_semiring_l1_python_runtime_exec"),
        "{NAT_SEMIRING_RUNG_L1_PYTHON_RUNTIME_PATH}: L1 python runtime claim"
    );
    assert!(
        surface_declares_data(&module, "phase1_nat_semiring_l1_python_runtime_claim_rows"),
        "{NAT_SEMIRING_RUNG_L1_PYTHON_RUNTIME_PATH}: L1 claim roster"
    );
}

#[test]
fn v4_nat_semiring_rung_l1_go_compiler_slice_dag_tokenizes_and_parses_claim_row() {
    let module = parse_module(
        NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_DAG,
        NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH,
    );
    assert!(
        surface_declares_data(&module, "go_l1_nat_semiring_rung2"),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: canonical slice id symbol"
    );
    assert!(
        surface_declares_data(&module, "go_l1_nat_semiring_rung2_l1_host_gate"),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: L1 host gate path"
    );
    assert!(
        surface_declares_data(&module, "go_l1_nat_semiring_rung2_l1_receipt_schema"),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: L1 JSON receipt schema"
    );
    assert!(
        surface_declares_data(&module, "phase1_l1_go_compiler_slice_subject_slice_binding"),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: slice id bound on receipt subject"
    );
    assert!(
        NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_DAG.contains(
            "scripts/v4-phase1-nat-semiring-go-compiler-slice-gate.sh::go_l1_compiler_slice_receipt_v1"
        ),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: receipt schema must match host transport"
    );
    assert!(
        surface_declares_data(&module, "phase1_nat_semiring_l1_go_compiler_slice_subject"),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: Conj receipt subject carrier"
    );
    assert!(
        surface_declares_data(
            &module,
            "claim_phase1_nat_semiring_l1_go_compiler_slice_compile"
        ),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: L1 go compiler-slice claim"
    );
    assert!(
        surface_declares_data(
            &module,
            "phase1_nat_semiring_l1_go_compiler_slice_claim_rows"
        ),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: L1 claim roster"
    );
    assert!(
        NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_DAG.contains("input: phase1_nat_semiring_l1_go_compiler_slice_subject")
            && NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_DAG
                .contains("target: phase1_l1_go_compiler_slice_subject_slice_binding"),
        "{NAT_SEMIRING_RUNG_L1_GO_COMPILER_SLICE_PATH}: CompilesClaim must reference structural slice binding"
    );
}

#[test]
fn emit_host_bridge_rust_transport_builds_runs_and_parses_stdout() {
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_emit_host_bridge_{}",
        std::process::id()
    ));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "w3_bridge_claim_input".to_string(),
        expected_eval_root: "w3_bridge_expected_eval".to_string(),
    };
    let receipt = emit_host_bridge::run_emit_host_rust_transport(
        EMIT_HOST_FIXTURE_SOURCE,
        &inputs,
        &work_dir,
    )
    .expect("run_emit_host_rust_transport");
    assert!(
        emit_host_bridge::host_exit_holds(&receipt.exit),
        "expected Holds exit, got {:?}",
        receipt.exit
    );
    let stdout = emit_host_bridge::host_stdout_bytes(&receipt.exit, receipt.stdout_bytes.clone())
        .expect("logical stdout");
    emit_host_runner::runtime_value_parse_rust(&stdout).expect("parse five-byte stdout");
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
        "run_emit_host_python",
        "run_emit_host_go",
        "run_emit_host",
        "runtime_value_parse_python",
        "runtime_value_parse_go",
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
        surface_declares_fn(&module, "emit_host_transport_not_wired_diagnostic"),
        "{EMIT_HOST_PATH}: transport_not_wired diagnostic (fail-closed substrate row)"
    );
    assert!(
        surface_declares_data(&module, "emit_host_transport_not_wired"),
        "{EMIT_HOST_PATH}: transport_not_wired reason symbol"
    );
    assert!(
        surface_declares_data(&module, "emit_host_python_authority_pin"),
        "{EMIT_HOST_PATH}: python authority pin (W3.4)"
    );
    assert!(
        surface_declares_data(&module, "emit_host_go_authority_pin"),
        "{EMIT_HOST_PATH}: go authority pin (W3.3)"
    );
}

#[test]
fn v4_nat_semiring_rung_3_4_dag_tokenizes_and_parses_emit_vs_eval_row() {
    let module = parse_module(NAT_SEMIRING_RUNG_3_4_DAG, NAT_SEMIRING_RUNG_3_4_PATH);
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::Data { name, .. }
                if name == "run_phase1_nat_semiring_rung4_rust_emit_equals_eval"
        )),
        "{NAT_SEMIRING_RUNG_3_4_PATH}: rung-4 TestClaimRun row (#4046 roster)"
    );
}

#[test]
fn emit_host_runtime_value_parse_rust_falsification_on_wrong_stdout_len() {
    let err = emit_host_runner::runtime_value_parse_rust(&[0u8; 3])
        .expect_err("three-byte stdout must fail parse");
    assert!(
        err.to_string().contains("expected 5"),
        "structured parse failure: {err}"
    );
}

#[test]
fn v4_nat_semiring_rung_gate_dag_tokenizes_and_parses_populated_roster_gates() {
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
        surface_declares_data(&module, "nat_semiring_rung34_runtime_value_rows"),
        "{NAT_SEMIRING_RUNG34_EVAL_PATH}: runtime roster carrier"
    );
    let rung_3_4 = parse_module(NAT_SEMIRING_RUNG_3_4_DAG, NAT_SEMIRING_RUNG_3_4_PATH);
    assert!(
        surface_declares_data(&rung_3_4, "run_phase1_nat_semiring_rung3_module_roundtrip"),
        "{NAT_SEMIRING_RUNG_3_4_PATH}: rung-3 roster row (#4046)"
    );
    assert!(
        surface_declares_data(
            &rung_3_4,
            "run_phase1_nat_semiring_rung4_rust_emit_equals_eval"
        ),
        "{NAT_SEMIRING_RUNG_3_4_PATH}: rung-4 roster row (#4046)"
    );
}

#[test]
fn v4_nat_semiring_rung8_dag_tokenizes_and_parses_full_law_roster() {
    let rung_8 = parse_module(NAT_SEMIRING_RUNG_8_DAG, NAT_SEMIRING_RUNG_8_PATH);
    for name in [
        "run_claim_nat_add_left_identity",
        "run_claim_nat_add_right_identity",
        "run_claim_nat_add_associativity",
        "run_claim_nat_mul_left_identity",
        "run_claim_nat_mul_annihilator",
        "run_claim_nat_mul_associativity",
        "run_claim_nat_add_wrong_identity_falsifies_law",
    ] {
        assert!(
            surface_declares_data(&rung_8, name),
            "{NAT_SEMIRING_RUNG_8_PATH}: missing run row {name}"
        );
    }
    assert!(
        surface_declares_data(&rung_8, "phase1_nat_semiring_rung8_runtime_value_rows"),
        "{NAT_SEMIRING_RUNG_8_PATH}: runtime roster carrier (7 rows)"
    );
    assert!(
        surface_declares_fn(&rung_8, "rung8_tier1_eval_run"),
        "{NAT_SEMIRING_RUNG_8_PATH}: T-22 eval constructor must call run_test_claim"
    );
    assert!(
        NAT_SEMIRING_RUNG_8_DAG.contains("run_test_claim("),
        "{NAT_SEMIRING_RUNG_8_PATH}: roster rows must thread run_test_claim verdicts (not fabricated Pass)"
    );

    let eval_module = parse_module(NAT_SEMIRING_RUNG8_EVAL_DAG, NAT_SEMIRING_RUNG8_EVAL_PATH);
    for name in [
        "run_nat_semiring_rung8_eval",
        "nat_semiring_rung8_gate",
        "nat_semiring_rung8_zero_deferred",
    ] {
        assert!(
            surface_declares_fn(&eval_module, name),
            "{NAT_SEMIRING_RUNG8_EVAL_PATH}: missing fn {name}"
        );
    }
    assert!(
        surface_declares_data(
            &eval_module,
            "witness_nat_semiring_rung8_zero_deferred_closed"
        ),
        "{NAT_SEMIRING_RUNG8_EVAL_PATH}: authoring-time zero-deferred witness (data binding)"
    );
    assert!(
        NAT_SEMIRING_RUNG8_EVAL_DAG.contains("phase1_nat_semiring_rung8_runtime_value_rows"),
        "{NAT_SEMIRING_RUNG8_EVAL_PATH}: CorpusEvalReport must consume rung-8 roster"
    );
    assert!(
        NAT_SEMIRING_RUNG8_EVAL_DAG.contains("corpus_report_tally"),
        "{NAT_SEMIRING_RUNG8_EVAL_PATH}: VerdictTally via corpus_report_tally"
    );
}

#[test]
fn v4_loop_linear_bound_rung8_dag_tokenizes_and_parses_full_fixture_roster() {
    let rung_8 = parse_module(LOOP_LINEAR_BOUND_RUNG_8_DAG, LOOP_LINEAR_BOUND_RUNG_8_PATH);
    for name in [
        "run_claim_loop_linear_bound_well_formed",
        "run_claim_loop_linear_bound_equals_refl",
        "run_claim_loop_linear_bound_content_hash_stable",
    ] {
        assert!(
            surface_declares_data(&rung_8, name),
            "{LOOP_LINEAR_BOUND_RUNG_8_PATH}: missing run row {name}"
        );
    }
    assert!(
        surface_declares_data(&rung_8, "phase4_loop_linear_bound_rung8_runtime_value_rows"),
        "{LOOP_LINEAR_BOUND_RUNG_8_PATH}: runtime roster carrier (3 rows)"
    );
    assert!(
        surface_declares_fn(&rung_8, "rung8_tier1_eval_run"),
        "{LOOP_LINEAR_BOUND_RUNG_8_PATH}: T-22 eval constructor must call run_test_claim"
    );
    assert!(
        LOOP_LINEAR_BOUND_RUNG_8_DAG.contains("run_test_claim("),
        "{LOOP_LINEAR_BOUND_RUNG_8_PATH}: roster rows must thread run_test_claim verdicts (not fabricated Pass)"
    );

    let eval_module = parse_module(
        LOOP_LINEAR_BOUND_RUNG8_EVAL_DAG,
        LOOP_LINEAR_BOUND_RUNG8_EVAL_PATH,
    );
    for name in [
        "run_loop_linear_bound_rung8_eval",
        "loop_linear_bound_rung8_gate",
        "loop_linear_bound_rung8_zero_deferred",
    ] {
        assert!(
            surface_declares_fn(&eval_module, name),
            "{LOOP_LINEAR_BOUND_RUNG8_EVAL_PATH}: missing fn {name}"
        );
    }
    assert!(
        surface_declares_data(
            &eval_module,
            "witness_loop_linear_bound_rung8_zero_deferred_closed"
        ),
        "{LOOP_LINEAR_BOUND_RUNG8_EVAL_PATH}: authoring-time closed witness (data binding)"
    );
    assert!(
        LOOP_LINEAR_BOUND_RUNG8_EVAL_DAG.contains(
            "witness_loop_linear_bound_rung8_zero_deferred_closed: Bool = loop_linear_bound_rung8_gate(report: run_loop_linear_bound_rung8_eval())"
        ),
        "{LOOP_LINEAR_BOUND_RUNG8_EVAL_PATH}: closed witness must bind full gate (non-empty roster + zero deferred)"
    );
    assert!(
        !LOOP_LINEAR_BOUND_RUNG8_EVAL_DAG.contains(
            "witness_loop_linear_bound_rung8_zero_deferred_closed: Bool = loop_linear_bound_rung8_zero_deferred"
        ),
        "{LOOP_LINEAR_BOUND_RUNG8_EVAL_PATH}: witness must not bypass gate via zero_deferred alone"
    );
    assert!(
        LOOP_LINEAR_BOUND_RUNG8_EVAL_DAG.contains(
            "corpus_entries_from_node_runtime_value_runs(\n      runs: phase4_loop_linear_bound_rung8_runtime_value_rows"
        ),
        "{LOOP_LINEAR_BOUND_RUNG8_EVAL_PATH}: CorpusEvalReport must consume rung-8 roster in eval body"
    );
    assert!(
        LOOP_LINEAR_BOUND_RUNG8_EVAL_DAG
            .contains("corpus_report_tally(r: report).deferred == Zero"),
        "{LOOP_LINEAR_BOUND_RUNG8_EVAL_PATH}: VerdictTally deferred check in zero_deferred helper"
    );
}

#[test]
fn v4_branch_dispatch_rung8_dag_tokenizes_and_parses_full_fixture_roster() {
    let rung_8 = parse_module(BRANCH_DISPATCH_RUNG_8_DAG, BRANCH_DISPATCH_RUNG_8_PATH);
    for name in [
        "run_claim_branch_dispatch_well_formed",
        "run_claim_branch_dispatch_equals_refl",
        "run_claim_branch_dispatch_content_hash_stable",
    ] {
        assert!(
            surface_declares_data(&rung_8, name),
            "{BRANCH_DISPATCH_RUNG_8_PATH}: missing run row {name}"
        );
    }
    assert!(
        surface_declares_data(&rung_8, "phase4_branch_dispatch_rung8_runtime_value_rows"),
        "{BRANCH_DISPATCH_RUNG_8_PATH}: runtime roster carrier (3 rows)"
    );
    assert!(
        surface_declares_fn(&rung_8, "rung8_tier1_eval_run"),
        "{BRANCH_DISPATCH_RUNG_8_PATH}: T-22 eval constructor must call run_test_claim"
    );
    assert!(
        BRANCH_DISPATCH_RUNG_8_DAG.contains("run_test_claim("),
        "{BRANCH_DISPATCH_RUNG_8_PATH}: roster rows must thread run_test_claim verdicts (not fabricated Pass)"
    );

    let eval_module = parse_module(
        BRANCH_DISPATCH_RUNG8_EVAL_DAG,
        BRANCH_DISPATCH_RUNG8_EVAL_PATH,
    );
    for name in [
        "run_branch_dispatch_rung8_eval",
        "branch_dispatch_rung8_gate",
        "branch_dispatch_rung8_zero_deferred",
    ] {
        assert!(
            surface_declares_fn(&eval_module, name),
            "{BRANCH_DISPATCH_RUNG8_EVAL_PATH}: missing fn {name}"
        );
    }
    assert!(
        surface_declares_data(&eval_module, "witness_branch_dispatch_rung8_gate_closed"),
        "{BRANCH_DISPATCH_RUNG8_EVAL_PATH}: authoring-time F5 gate witness (data binding)"
    );
    assert!(
        BRANCH_DISPATCH_RUNG8_EVAL_DAG
            .contains("branch_dispatch_rung8_gate(report: run_branch_dispatch_rung8_eval())"),
        "{BRANCH_DISPATCH_RUNG8_EVAL_PATH}: closed witness must consume full gate (non-empty roster + zero Deferred)"
    );
    assert!(
        BRANCH_DISPATCH_RUNG8_EVAL_DAG.contains("phase4_branch_dispatch_rung8_runtime_value_rows"),
        "{BRANCH_DISPATCH_RUNG8_EVAL_PATH}: CorpusEvalReport must consume rung-8 roster"
    );
    assert!(
        BRANCH_DISPATCH_RUNG8_EVAL_DAG.contains("corpus_report_tally"),
        "{BRANCH_DISPATCH_RUNG8_EVAL_PATH}: VerdictTally via corpus_report_tally"
    );
}

#[test]
fn emit_host_runner_go_row_runs_and_parses_stdout() {
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_v4_emit_host_go_{}",
        std::process::id()
    ));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "w5_go_claim_input".to_string(),
        expected_eval_root: "w5_go_expected_eval".to_string(),
    };
    let receipt =
        emit_host_runner::run_emit_host_go(EMIT_HOST_GO_FIXTURE_SOURCE, &inputs, &work_dir)
            .expect("run_emit_host_go");
    assert!(receipt.exit.exit_holds());
    emit_host_runner::runtime_value_parse_go(&receipt.stdout_bytes).expect("parse");
}

#[test]
fn cross_target_mvp2_stdout_parity_rust_python_go() {
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "cross_target_claim_input".to_string(),
        expected_eval_root: "cross_target_expected_eval".to_string(),
    };
    let pid = std::process::id();
    let rust_dir = emit_host_runner::default_work_dir(&format!("gunbc_xct_rust_{pid}"));
    let py_dir = emit_host_runner::default_work_dir(&format!("gunbc_xct_py_{pid}"));
    let go_dir = emit_host_runner::default_work_dir(&format!("gunbc_xct_go_{pid}"));
    let verdict = emit_host_bridge::run_cross_target_mvp2_python_parity_transport(
        EMIT_HOST_FIXTURE_SOURCE,
        EMIT_HOST_PYTHON_FIXTURE_SOURCE,
        EMIT_HOST_GO_FIXTURE_SOURCE,
        &inputs,
        &rust_dir,
        &py_dir,
        &go_dir,
    );
    assert_eq!(
        verdict,
        emit_host_bridge::EmitHostCrossTargetParityVerdict::Pass
    );
}

#[test]
fn v4_nat_semiring_rung_5_dag_tokenizes_and_parses_full_law_roster_three_targets() {
    let module = parse_module(NAT_SEMIRING_RUNG_5_DAG, NAT_SEMIRING_RUNG_5_PATH);
    for name in [
        "run_phase1_nat_semiring_rung5_rust_add_left_identity_emit_equals_eval",
        "run_phase1_nat_semiring_rung5_python_mul_associativity_emit_equals_eval",
        "run_phase1_nat_semiring_rung5_go_mul_annihilator_emit_equals_eval",
    ] {
        assert!(
            surface_declares_data(&module, name),
            "{NAT_SEMIRING_RUNG_5_PATH}: sample rung-5 row {name}"
        );
    }
    assert!(
        surface_declares_data(
            &module,
            "phase1_nat_semiring_rung5_full_law_roster_runtime_value_rows"
        ),
        "{NAT_SEMIRING_RUNG_5_PATH}: 6 laws × 3 targets roster"
    );
    let common = parse_module(RUNG_5_6_COMMON_DAG, RUNG_5_6_COMMON_PATH);
    assert!(
        surface_declares_data(&common, "rung56_emit_host_go_target_model"),
        "{RUNG_5_6_COMMON_PATH}: go TargetModel (W3.3)"
    );

    let eval_module = parse_module(NAT_SEMIRING_RUNG5_EVAL_DAG, NAT_SEMIRING_RUNG5_EVAL_PATH);
    for name in [
        "run_nat_semiring_rung5_eval",
        "nat_semiring_rung5_gate",
        "nat_semiring_rung5_report_has_evidence",
    ] {
        assert!(
            surface_declares_fn(&eval_module, name),
            "{NAT_SEMIRING_RUNG5_EVAL_PATH}: missing fn {name}"
        );
    }
    assert!(
        NAT_SEMIRING_RUNG5_EVAL_DAG
            .contains("phase1_nat_semiring_rung5_full_law_roster_runtime_value_rows"),
        "{NAT_SEMIRING_RUNG5_EVAL_PATH}: CorpusEvalReport must consume rung-5 roster"
    );
}

#[test]
fn emit_host_runner_python_row_runs_and_parses_stdout() {
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_v4_emit_host_py_{}",
        std::process::id()
    ));
    let inputs = emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: "w34_py_claim_input".to_string(),
        expected_eval_root: "w34_py_expected_eval".to_string(),
    };
    let receipt =
        emit_host_runner::run_emit_host_python(EMIT_HOST_PYTHON_FIXTURE_SOURCE, &inputs, &work_dir)
            .expect("run_emit_host_python");
    assert!(receipt.exit.exit_holds());
    emit_host_runner::runtime_value_parse_python(&receipt.stdout_bytes).expect("parse");
}

#[test]
fn v4_nat_semiring_rung_6_dag_tokenizes_and_parses_post_emit_law_preservation_emit_rows() {
    let module = parse_module(NAT_SEMIRING_RUNG_6_DAG, NAT_SEMIRING_RUNG_6_PATH);
    for name in [
        "run_phase1_nat_semiring_rung6_rust_add_left_identity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_rust_add_right_identity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_rust_add_associativity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_rust_mul_left_identity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_rust_mul_associativity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_rust_mul_annihilator_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_python_add_left_identity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_python_add_right_identity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_python_add_associativity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_python_mul_left_identity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_python_mul_associativity_emit_equals_eval",
        "run_phase1_nat_semiring_rung6_python_mul_annihilator_emit_equals_eval",
    ] {
        assert!(
            surface_declares_data(&module, name),
            "{NAT_SEMIRING_RUNG_6_PATH}: missing rung-6 row {name}"
        );
    }
    let common = parse_module(RUNG_5_6_COMMON_DAG, RUNG_5_6_COMMON_PATH);
    for name in [
        "rung56_emit_host_python_target_model",
        "rung56_emit_host_go_target_model",
    ] {
        assert!(
            surface_declares_data(&common, name),
            "{RUNG_5_6_COMMON_PATH}: TargetModel {name}"
        );
    }
}

#[test]
fn v4_nat_semiring_rung56_eval_dag_tokenizes_and_parses_rung6_gate() {
    let module = parse_module(NAT_SEMIRING_RUNG56_EVAL_DAG, NAT_SEMIRING_RUNG56_EVAL_PATH);
    for name in [
        "nat_semiring_rung56_report_has_evidence",
        "nat_semiring_rung6_gate",
        "run_nat_semiring_rung56_eval",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{NAT_SEMIRING_RUNG56_EVAL_PATH}: missing fn {name}"
        );
    }
    assert!(
        surface_declares_data(
            &module,
            "nat_semiring_rung6_additive_monoid_runtime_value_rows"
        ),
        "{NAT_SEMIRING_RUNG56_EVAL_PATH}: tranche-1 additive-Monoid roster"
    );
    assert!(
        surface_declares_data(
            &module,
            "nat_semiring_rung6_mul_monoid_annihilator_runtime_value_rows"
        ),
        "{NAT_SEMIRING_RUNG56_EVAL_PATH}: tranche-2 mul-Monoid + annihilator roster"
    );
    assert!(
        surface_declares_data(
            &module,
            "nat_semiring_rung6_post_emit_law_preservation_runtime_value_rows"
        ),
        "{NAT_SEMIRING_RUNG56_EVAL_PATH}: combined tranche-1+2 CorpusEval roster"
    );
    assert!(
        NAT_SEMIRING_RUNG56_EVAL_DAG
            .contains("nat_semiring_rung6_post_emit_law_preservation_runtime_value_rows"),
        "{NAT_SEMIRING_RUNG56_EVAL_PATH}: CorpusEvalReport must consume full rung-6 roster"
    );
}
