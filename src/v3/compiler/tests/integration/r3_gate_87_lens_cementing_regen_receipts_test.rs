//! **Layer:** integration
//!
//! R3 gate #87 (`lens_cementing_test_discipline_complete`) — paired **Rust** receipts for
//! `src/v3/compiler/regen.dag` registry lenses while the `.dag`
//! `tests/dag/t_r3_gate_87_cementing_regen_*.dag` harnesses either use narrow behavioral
//! `LensOutputEquals` / `DifferentialEquals` witnesses or remain explicit temporary `Compiles`
//! placeholders where no public behavior carrier is authorable yet.
//!
//! **Lane-E + symbolic-cost** `.dag` receipts are exercised by `t_pb_b_1_dag_runner_test`.
//! `unused_parameters` and `structural_resolution` use Int-projection `.dag` claims until strict
//! user modules can freeze the corresponding list carriers without M1(2.8) opaque-body diagnostics;
//! Rust receipts below keep covering `UnusedParametersLens` / `lens_structural_resolution::check`.
//! Helper-only rows (`infer_helpers`, `lower_helpers`, `variant_payload`) stay explicit `Compiles`
//! placeholders with per-file dissolution triggers in their `.dag` harness comments.
//!
//! **Frozen-oracle witness discipline (`cost` / `cost_symbolic`):** the `.dag` harnesses
//! `tests/dag/t_r3_gate_87_cementing_regen_cost*.dag` pin `LensOutputEquals` / `DifferentialEquals` /
//! `SymbolicCostExprEquals*` witnesses. The Rust tests below keep the same program text, the same
//! `compile_to_dag` `file_name` markers, and the same structural witnesses so a drift-only edit to
//! either side fails CI (paired receipt; no live v2 oracle in tests).
//!
//! **INVARIANTS P5(b):** Gate-#87 work is **merge-visible** as this module,
//! `r3_gate_87_cementing_regen_runner_suites` plus `t_pb_b_1_dag_runner_test` wiring, and the
//! harness files under `tests/dag/t_r3_gate_87_cementing_regen_*.dag` (confirm with
//! `git diff origin/main...HEAD --stat` and path grep). Registry `name` inventory matches
//! `r3_gate_87_cementing_regen_runner_suites::r3_gate_87_cementing_regen_lens_names_for_runner_table`
//! derived from `R3_GATE_87_CEMENTING_REGEN_SUITES` (single authority, no parallel hand list).
//!
//! **Cementing-test discipline ratchet (`TESTING.md` §4 "One claim per test"):** every new
//! `#[test]` / `data foo: TestClaim` in this lane makes **one** structural claim; cross-suite
//! drive tests assert `ClaimResult` by shape (`== ClaimResult::Pass` or `matches!(_, Pass)`),
//! never by stringified message. When porting any Rust receipt below to a `.dag` `TestClaim`,
//! the same PR removes its row from `sg0_census_test::EXPECTED_HAND_AUTHORED_TEST` — no
//! parallel cementing inventory is allowed to track the Rust→`.dag` migration separately.
//! Per `INVARIANTS.md` §P5(b), the **single checkable net paydown receipt** (delete path, SG-0
//! census shrink with counts, or cited `ROADMAP.md` deferral) must live in **PR #2639’s
//! description**; module comments must not assert deletes for paths that never existed on
//! `origin/main`. §1.8 gate-#87 **PASSING** is indexed in `docs/r3-program-plan.md` (row 87);
//! the canonical Pass-condition body is `r3-structure.md` §"Acceptance"
//! (`lens_cementing_test_discipline_complete`). Broader Band-C work for lenses outside
//! `regen.dag` continues through `docs/v3-lens-capability-register.md` +
//! `cementing_lens_registry_dispatch_test.rs` + `ROADMAP.md` honesty pass.

use std::collections::BTreeSet;
use std::path::PathBuf;

use v3_compiler::r3_gate_87_cementing_regen_runner_suites::r3_gate_87_cementing_regen_lens_names_for_runner_table;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, Declaration, FieldValue, LiteralBits, SymbolicCost, ValueBody,
};
use v3_compiler::lens_cost::{cost_of, CostLookup};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
use v3_compiler::lens_cost_target_realization::type_realization_meta;
use v3_compiler::lens_effect_enumeration::{enumerate_effects, TransactionalPattern};
use v3_compiler::lens_provenance::{origin_of, Origin};
use v3_compiler::lens_structural_resolution;
use v3_compiler::lens_unused_parameters::{UnusedParametersConfig, UnusedParametersLens};
use v3_compiler::{analyze_symbolic_cost_dimension, DimensionReport, Dag, Witness};

use crate::common::assert_recursive_countdown_linear_semantics;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("expected src/v3/compiler -> workspace root")
        .to_path_buf()
}

fn structural_fields(decl: &Declaration) -> &[(String, FieldValue)] {
    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        panic!(
            "lens registry entry `{}` must carry a structural value body",
            decl.name.as_deref().unwrap_or("<anonymous>")
        );
    };
    fields.as_slice()
}

fn string_field(fields: &[(String, FieldValue)], label: &str, binding: &str) -> String {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("lens registry entry `{binding}` is missing a String `{label}` field")
        })
}

fn regen_lens_registry_names() -> BTreeSet<String> {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load `src/v3/compiler/regen.dag` cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let entry_type_id = dag
        .declaration_by_name("LensRegistryEntry")
        .map(|decl| decl.id)
        .expect("regen.dag must declare `LensRegistryEntry`");
    dag.declarations()
        .iter()
        .filter(|decl| decl.meta_tag == Some(entry_type_id))
        .map(|decl| {
            let binding = decl
                .name
                .clone()
                .unwrap_or_else(|| "<anonymous>".to_string());
            let fields = structural_fields(decl);
            string_field(fields, "name", &binding)
        })
        .collect()
}

fn read_lens_source(rel: &str) -> String {
    let path = workspace_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn assert_lens_dag_compiles(rel: &str) {
    let source = read_lens_source(rel);
    let dag = compile_to_dag(&source, rel).unwrap_or_else(|diag| {
        panic!("{rel} should compile cleanly, got {diag:?}");
    });
    assert!(
        dag.diagnostics().is_empty(),
        "{rel} should have no module diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

fn find_bind_value_port(dag: &Dag, name: &str) -> v3_compiler::dag::PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn find_bind_node(dag: &Dag, name: &str) -> v3_compiler::dag::NodeId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .id
}

/// Escape a V3 program fragment the way `TestClaim.source` string literals appear in `.dag` files.
fn escape_dag_test_claim_source(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}

/// Same stack sizing as `cementing/cost_lens_symbolic_consumer_test` — recursive countdown under full
/// bootstrap can overflow the default test thread.
fn run_gate87_cost_symbolic_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("r3-gate-87-cost-symbolic-cementing".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn gate-87 cost_symbolic cementing thread")
        .join()
        .expect("gate-87 cost_symbolic cementing thread should not panic");
}

fn linear_size_ports(cost: &SymbolicCost, out: &mut Vec<v3_compiler::dag::PortId>) {
    match cost {
        SymbolicCost::LinearCost { _0: var } | SymbolicCost::LogCost { _0: var } => {
            out.push(var.source_port);
        }
        SymbolicCost::PolynomialCost { var, .. } => {
            out.push(var.source_port);
        }
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            for term in terms.iter() {
                linear_size_ports(term.as_ref(), out);
            }
        }
        SymbolicCost::ConstantCost { .. } | SymbolicCost::UnknownCost { .. } => {}
    }
}

/// Program sources and `compile_to_dag` file markers paired with
/// `tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag`.
const GATE87_SYMBOLIC_LIT_SOURCE: &str = "let lit: Int = 7";
const GATE87_SYMBOLIC_LIT_FILE: &str = "r3_gate_87_symbolic_lit.v3";
const GATE87_SYMBOLIC_COUNTDOWN_SOURCE: &str =
    "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)";
const GATE87_SYMBOLIC_COUNTDOWN_FILE: &str = "r3_gate_87_symbolic_countdown.v3";

#[test]
fn r3_gate_87_regen_lens_registry_names_match_fixture_inventory() {
    let actual = regen_lens_registry_names();
    let expected = r3_gate_87_cementing_regen_lens_names_for_runner_table();
    assert_eq!(
        actual, expected,
        "`src/v3/compiler/regen.dag` registry drift vs \
         `v3_compiler::r3_gate_87_cementing_regen_runner_suites::R3_GATE_87_CEMENTING_REGEN_SUITES`: extend the runner table + \
         `tests/dag/t_r3_gate_87_cementing_regen_*.dag` in the same PR as any new registry row."
    );
}

#[test]
fn r3_gate_87_effect_enumeration_rust_receipt_on_minimal_program() {
    let dag =
        compile_to_dag("let lit: Int = 7", "r3_gate_87_effect_enum_receipt.v3").expect("compile");
    let report = enumerate_effects(&dag);
    assert!(
        matches!(report.transaction, TransactionalPattern::NoTransaction),
        "effect enumeration transaction scaffold must remain explicit"
    );
    assert!(
        report.facts.len() <= dag.nodes().len(),
        "effect facts should not exceed walked node count"
    );
}

#[test]
fn r3_gate_87_provenance_origin_rust_receipt_on_literal_bind() {
    let dag =
        compile_to_dag("let lit: Int = 7", "r3_gate_87_provenance_receipt.v3").expect("compile");
    let port = find_bind_value_port(&dag, "lit");
    let got = origin_of(&dag, &port);
    assert!(
        matches!(got, Origin::Source { .. }),
        "literal bind should classify as Source(..), got {got:?}"
    );
}

#[test]
fn r3_gate_87_cost_target_realization_rust_receipt_resolves_type_realization_row() {
    let dag = compile_to_dag(
        "let lit: Int = 7",
        "r3_gate_87_cost_target_realization_receipt.v3",
    )
    .expect("compile");
    let resolved_name = type_realization_meta(&dag).and_then(|d| d.name);
    assert_eq!(
        resolved_name.as_deref(),
        Some("TypeRealization"),
        "type_realization_meta must resolve the substrate `TypeRealization` declaration \
         (declaration_by_name contract used by cost_target_realization.dag)"
    );
}

#[test]
fn r3_gate_87_infer_helpers_lens_source_compiles() {
    assert_lens_dag_compiles("src/v3/lenses/infer_helpers.dag");
}

#[test]
fn r3_gate_87_variant_payload_lens_source_compiles() {
    assert_lens_dag_compiles("src/v3/lenses/variant_payload.dag");
}

#[test]
fn r3_gate_87_lower_helpers_lens_source_compiles() {
    assert_lens_dag_compiles("src/v3/lenses/lower_helpers.dag");
}

#[test]
fn r3_gate_87_structural_resolution_rust_receipt_on_literal_program() {
    let dag = compile_to_dag(
        "let lit: Int = 7",
        "r3_gate_87_structural_resolution_receipt.v3",
    )
    .expect("compile");
    assert!(
        lens_structural_resolution::check(&dag).is_empty(),
        "clean literal program should surface zero Pending-arrow violations"
    );
}

#[test]
fn r3_gate_87_unused_parameters_rust_receipt_on_literal_program() {
    let dag = compile_to_dag(
        "let lit: Int = 7",
        "r3_gate_87_unused_parameters_receipt.v3",
    )
    .expect("compile");
    assert!(
        UnusedParametersLens::new(&dag)
            .query(&UnusedParametersConfig::default())
            .is_empty(),
        "literal bind should not surface unused-parameter findings"
    );
}

#[test]
fn r3_gate_87_parallelism_analyze_lit_frozen_no_workflow_witness() {
    const FILE: &str = "r3_gate_87_parallelism_lit.v3";
    let dag = compile_to_dag("let lit: Int = 7", FILE).expect("compile");
    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "lit" && b.span.file == FILE)
        .expect("lit bind");
    let report = v3_compiler::analyze_parallelism(&dag, bind.id);
    assert!(
        matches!(
            report,
            v3_compiler::dag::WorkflowParallelismReport::ParallelismUnsupported(ref d)
                if d.kind == v3_compiler::dag::ParallelismUnsupportedKind::NoWorkflowProjection
        ),
        "frozen witness: literal program without lane2 workflow must classify as NoWorkflowProjection; \
         got {report:?} (paired with `t_r3_gate_87_cementing_regen_parallelism.dag`)"
    );
}

#[test]
fn r3_gate_87_cost_cementing_dag_merge_sort_program_locksteps_merge_sort_fixture() {
    let program = include_str!("../fixtures/r1_merge_sort_pair.v3");
    let dag_text = include_str!("../dag/t_r3_gate_87_cementing_regen_cost.dag");
    let needle = escape_dag_test_claim_source(program);
    assert!(
        dag_text.contains(&needle),
        "`tests/dag/t_r3_gate_87_cementing_regen_cost.dag` must embed the same merge-sort program \
         text as `tests/fixtures/r1_merge_sort_pair.v3` so the gate-87 `.dag` receipt and the Rust \
         pin share one frozen-oracle authority"
    );
}

#[test]
fn r3_gate_87_cost_merge_sort_lens_pins_frozen_v2_int_depth_oracle_witness() {
    const FILE: &str = "r3_gate_87_merge_sort_pair.v3";
    let dag = compile_to_dag(include_str!("../fixtures/r1_merge_sort_pair.v3"), FILE)
        .unwrap_or_else(|e| panic!("{FILE}: expected clean compile, got {e:?}"));
    assert!(
        dag.diagnostics().is_empty(),
        "{FILE}: expected no diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let port = find_bind_value_port(&dag, "merge_sort_out");
    assert_eq!(
        cost_of(&dag, &port),
        CostLookup::Hit(3),
        "Lane-E frozen int-depth oracle for merge-sort (matches \
         `t_r3_gate_87_cementing_regen_cost.dag` `gate87_merge_sort_expected_cost: Int = 3`)"
    );
}

#[test]
fn r3_gate_87_cost_symbolic_cementing_dag_sources_lockstep_rust_authority() {
    let dag_text = include_str!("../dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag");
    assert!(
        dag_text.contains(&format!(
            "source: \"{}\"",
            escape_dag_test_claim_source(GATE87_SYMBOLIC_LIT_SOURCE)
        )),
        "cost_symbolic harness must keep the literal `TestClaim.source` in lockstep with \
         `GATE87_SYMBOLIC_LIT_SOURCE`"
    );
    assert!(
        dag_text.contains(&format!(
            "source: \"{}\"",
            escape_dag_test_claim_source(GATE87_SYMBOLIC_COUNTDOWN_SOURCE)
        )),
        "cost_symbolic harness must keep the countdown `TestClaim.source` in lockstep with \
         `GATE87_SYMBOLIC_COUNTDOWN_SOURCE`"
    );
}

#[test]
fn r3_gate_87_cost_symbolic_literal_pins_frozen_constant_witness() {
    run_gate87_cost_symbolic_stack(|| {
        let dag = compile_to_dag(GATE87_SYMBOLIC_LIT_SOURCE, GATE87_SYMBOLIC_LIT_FILE)
            .unwrap_or_else(|e| panic!("{}: {e:?}", GATE87_SYMBOLIC_LIT_FILE));
        assert!(
            dag.diagnostics().is_empty(),
            "{}: expected no diagnostics",
            GATE87_SYMBOLIC_LIT_FILE
        );
        let port = find_bind_value_port(&dag, "lit");
        let cost = match symbolic_cost_of(&dag, &port) {
            SymbolicCostLookup::Hit(c) => c,
            SymbolicCostLookup::Miss => panic!("symbolic_cost_of Miss for `lit`"),
        };
        assert!(
            matches!(cost, SymbolicCost::ConstantCost { _0: 0 }),
            "frozen witness: literal symbolic cost must stay ConstantCost(0) (matches \
             `t_r3_gate_87_cementing_regen_cost_symbolic.dag` `gate87_symbolic_cost_expected`); \
             got {cost:?}"
        );

        let report = analyze_symbolic_cost_dimension(&dag, find_bind_node(&dag, "lit"));
        let DimensionReport::DimensionOk {
            dimension_name,
            composed,
            witnesses,
        } = report
        else {
            panic!("analyze_symbolic_cost_dimension should return DimensionOk for `lit`");
        };
        assert_eq!(dimension_name, "symbolic_cost");
        assert_eq!(
            composed, cost,
            "dimension spine must agree with symbolic_cost_of for the same frozen witness"
        );
        assert!(
            witnesses.iter().all(|w| matches!(w, Witness::Inhabits(_))),
            "symbolic cost dimension should only emit Inhabits witnesses for gate-87 literal \
             receipt, got {witnesses:?}"
        );
    });
}

#[test]
fn r3_gate_87_cost_symbolic_countdown_pins_linear_bind_param_witness_discipline() {
    run_gate87_cost_symbolic_stack(|| {
        let dag = compile_to_dag(GATE87_SYMBOLIC_COUNTDOWN_SOURCE, GATE87_SYMBOLIC_COUNTDOWN_FILE)
            .unwrap_or_else(|e| panic!("{}: {e:?}", GATE87_SYMBOLIC_COUNTDOWN_FILE));
        assert!(
            dag.diagnostics().is_empty(),
            "{}: expected no diagnostics",
            GATE87_SYMBOLIC_COUNTDOWN_FILE
        );
        let countdown = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|bind| bind.name == "countdown")
            .expect("countdown bind");
        let parameter = countdown
            .params
            .first()
            .copied()
            .expect("countdown should expose one parameter port for bind-param witness");

        let port = find_bind_value_port(&dag, "countdown");
        let cost = match symbolic_cost_of(&dag, &port) {
            SymbolicCostLookup::Hit(c) => c,
            SymbolicCostLookup::Miss => panic!("symbolic_cost_of Miss for `countdown`"),
        };
        assert_recursive_countdown_linear_semantics(&cost);

        let mut ports = Vec::new();
        linear_size_ports(&cost, &mut ports);
        assert!(
            ports.contains(&parameter),
            "frozen witness: linear cost must key the unary parameter port {parameter:?} (matches \
             `SymbolicCostExprEqualsForBindParam` / `LinearCostForBindParam` in \
             `t_r3_gate_87_cementing_regen_cost_symbolic.dag`); got cost={cost:?}"
        );
    });
}
