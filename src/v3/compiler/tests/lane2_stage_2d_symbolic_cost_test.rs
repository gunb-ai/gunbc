// Lane 2 Stage 2d / DB-7 — symbolic-cost lens acceptance tests.
//
// Authority: `src/v3/lenses/cost.dag` (projection:
// `src/v3/compiler/src/lens_cost_symbolic_generated.rs`).
//
// These tests pin the per-Behavior lowering contract DB-7 specifies
// — every Behavior variant produces an honest asymptotic bound, and
// the composition algebra in `src/v3/std/algebra.dag` (+ its Rust
// mirror in `dag.rs`) normalizes correctly.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    dominates, iterate, max_path, normalize, sequential, Behavior, Dag, PortId, SizeVariable,
    SymbolicCost,
};
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};

fn find_bind_value(dag: &Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn expect_cost(dag: &Dag, port: PortId) -> SymbolicCost {
    match symbolic_cost_of(dag, &port) {
        SymbolicCostLookup::FoundCost { _0: cost } => cost,
        SymbolicCostLookup::MissingCost => {
            panic!("symbolic_cost_of returned MissingCost for {port:?}")
        }
    }
}

fn bind_cost(source: &str, file: &str, name: &str) -> SymbolicCost {
    let dag = compile_to_dag(source, file).expect("fixture compiles");
    expect_cost(&dag, find_bind_value(&dag, name))
}

fn is_constant(cost: &SymbolicCost) -> bool {
    matches!(cost, SymbolicCost::ConstantCost { .. })
}

fn mentions_linear(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::LinearCost { .. } => true,
        SymbolicCost::SumCost { _0: terms } | SymbolicCost::ProductCost { _0: terms } => {
            terms.iter().any(mentions_linear)
        }
        _ => false,
    }
}

// Two real PortIds from the bootstrap Dag — just need distinct
// handles for the structural-equality checks the composition tests
// exercise. `Dag::new()` ports are stable across runs.
fn two_distinct_ports() -> (PortId, PortId) {
    let dag = Dag::new();
    let ports: Vec<PortId> = dag.ports().iter().map(|p| p.id()).collect();
    assert!(
        ports.len() >= 2,
        "bootstrap Dag should allocate at least two ports"
    );
    (ports[0], ports[1])
}

fn size_var(source_port: PortId) -> SizeVariable {
    SizeVariable { source_port }
}

fn linear(port: PortId) -> SymbolicCost {
    SymbolicCost::LinearCost { _0: size_var(port) }
}

fn polynomial(port: PortId, degree: i64) -> SymbolicCost {
    SymbolicCost::PolynomialCost {
        var: size_var(port),
        degree,
    }
}

fn log_cost(port: PortId) -> SymbolicCost {
    SymbolicCost::LogCost { _0: size_var(port) }
}

fn constant(n: i64) -> SymbolicCost {
    SymbolicCost::ConstantCost { _0: n }
}

// ── Per-Behavior lowering ────────────────────────────────────────

#[test]
fn value_reports_constant() {
    // `let x = 1` → ConstantCost(0) (leaf literal, no work).
    let dag = compile_to_dag("let x = 1", "test.v3").expect("compiles");
    let cost = expect_cost(&dag, find_bind_value(&dag, "x"));
    assert!(
        is_constant(&cost),
        "literal value should report Constant, got {cost:?}"
    );
}

#[test]
fn transform_single_op_reports_constant() {
    // `let x = 1 + 2` → sequential(Constant(1), sum(Constant(0), Constant(0)))
    // → Constant wrapper. Asymptotically O(1).
    let cost = bind_cost("let x = 1 + 2", "test.v3", "x");
    assert!(
        is_constant(&cost),
        "single scalar op should report Constant, got {cost:?}"
    );
}

#[test]
fn branch_reports_constant_when_both_arms_constant() {
    // `if 1 > 0 then 10 else 20` — both arms are leaf literals, so
    // max_path over two Constants stays Constant.
    let cost = bind_cost("let r = if 1 > 0 then 10 else 20", "test.v3", "r");
    assert!(
        is_constant(&cost),
        "branch over constant arms should report Constant, got {cost:?}"
    );
}

#[test]
fn recursive_fn_reports_linear_via_loop_lowering() {
    // `fn countdown(n) = if n == 0 then 0 else countdown(n - 1)`
    // lowers to a Loop whose source is the `n` param port. The
    // lens's `loop_cost` fires LinearCost(size_var_of(source)),
    // so the bound carries a LinearCost term.
    let dag = compile_to_dag(
        "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
        "countdown.v3",
    )
    .expect("compiles");
    let cost = expect_cost(&dag, find_bind_value(&dag, "countdown"));
    assert!(
        mentions_linear(&cost),
        "recursive fn should surface a LinearCost term, got {cost:?}"
    );
}

// ── Composition algebra (std.algebra Rust mirror) ────────────────

#[test]
fn sum_of_linear_and_constant_zero_normalizes_to_linear() {
    // DB-7 acceptance gate: `SumCost([LinearCost, ConstantCost(0)])`
    // normalizes to `LinearCost` (zero drops out).
    let (port, _) = two_distinct_ports();
    let l = linear(port);
    let result = sequential(l.clone(), constant(0));
    assert_eq!(
        result, l,
        "Linear + Constant(0) should normalize to Linear, got {result:?}"
    );
}

#[test]
fn sum_of_constant_and_linear_normalizes_to_linear() {
    // Dominance is order-independent: Constant + Linear also
    // normalizes to Linear (Constant is dominated).
    let (port, _) = two_distinct_ports();
    let l = linear(port);
    let result = sequential(constant(5), l);
    assert!(
        matches!(result, SymbolicCost::LinearCost { .. }),
        "Constant + Linear should normalize to Linear, got {result:?}"
    );
}

#[test]
fn product_of_two_linears_over_same_var_folds_to_polynomial_squared() {
    // DB-7 §"Nested fold detection (O(n²) diagnostic)":
    // `iterate(Linear(n), Linear(n))` folds to `Polynomial(n, 2)`.
    let (port, _) = two_distinct_ports();
    let l = linear(port);
    let result = iterate(l.clone(), l);
    match result {
        SymbolicCost::PolynomialCost { var, degree } => {
            assert_eq!(var, size_var(port));
            assert_eq!(degree, 2);
        }
        other => panic!("expected PolynomialCost(n, 2), got {other:?}"),
    }
}

#[test]
fn product_of_linears_over_different_vars_stays_product() {
    // Two folds over DISTINCT lists stay as ProductCost — the
    // two size variables aren't the same, so the polynomial
    // collapse doesn't fire.
    let (port_a, port_b) = two_distinct_ports();
    let result = iterate(linear(port_a), linear(port_b));
    assert!(
        matches!(result, SymbolicCost::ProductCost { .. }),
        "Linear(a) * Linear(b) over different ports should stay Product, got {result:?}"
    );
}

#[test]
fn dominance_unknown_dominates_everything() {
    // DB-7 §"Dominance" — Unknown is the safest over-approximation.
    let (port, _) = two_distinct_ports();
    let unknown = SymbolicCost::UnknownCost {
        _0: "reflection".to_string(),
    };
    assert!(dominates(&unknown, &linear(port)));
    assert!(dominates(&unknown, &constant(0)));
}

#[test]
fn dominance_linear_dominates_log_and_constant() {
    let (port, _) = two_distinct_ports();
    assert!(dominates(&linear(port), &log_cost(port)));
    assert!(dominates(&linear(port), &constant(3)));
    assert!(!dominates(&constant(3), &linear(port)));
}

#[test]
fn dominance_polynomial_degree_ordering() {
    let (port, _) = two_distinct_ports();
    assert!(dominates(&polynomial(port, 3), &polynomial(port, 2)));
    assert!(!dominates(&polynomial(port, 2), &polynomial(port, 3)));
}

#[test]
fn composite_dominance_reads_children_not_outer_variant() {
    // PR #537 review (codex P2): Product / Sum dominance must
    // derive from children, not hardcode "any Product dominates
    // Linear". A product whose strongest child is `Log(n)`
    // should NOT dominate `Linear(n)`.
    let (port, _) = two_distinct_ports();
    let product_of_logs = SymbolicCost::ProductCost {
        _0: vec![log_cost(port), log_cost(port)],
    };
    let linear_cost = linear(port);
    assert!(
        !dominates(&product_of_logs, &linear_cost),
        "Product([Log, Log]) must NOT dominate Linear — children don't reach Linear, \
         got dominates=true (regression of codex P2 on PR #537)"
    );

    // Symmetric positive case: a product WITH a Linear child DOES
    // dominate Log, because the dominant-child summary walks the
    // children.
    let product_with_linear = SymbolicCost::ProductCost {
        _0: vec![log_cost(port), linear(port)],
    };
    assert!(
        dominates(&product_with_linear, &log_cost(port)),
        "Product([Log, Linear]) must dominate Log via its Linear child"
    );
}

#[test]
fn sum_dominance_reads_children_not_outer_variant() {
    // Same fix as ProductCost: Sum's dominance is a dominant-child
    // summary, not a hardcoded "any Sum dominates scalars".
    let (port, _) = two_distinct_ports();
    let sum_of_logs = SymbolicCost::SumCost {
        _0: vec![log_cost(port), log_cost(port)],
    };
    assert!(
        !dominates(&sum_of_logs, &linear(port)),
        "Sum([Log, Log]) must NOT dominate Linear — no child reaches Linear"
    );

    let sum_with_polynomial = SymbolicCost::SumCost {
        _0: vec![constant(0), polynomial(port, 2)],
    };
    assert!(
        dominates(&sum_with_polynomial, &linear(port)),
        "Sum containing Polynomial(n, 2) must dominate Linear(n) via the poly child"
    );
}

#[test]
fn max_path_returns_dominant_term() {
    // `max_path([Constant(0), Linear(n), Polynomial(n, 2)])` → Polynomial(n, 2).
    let (port, _) = two_distinct_ports();
    let paths = vec![constant(0), linear(port), polynomial(port, 2)];
    let result = max_path(&paths);
    match result {
        SymbolicCost::PolynomialCost { degree, .. } => assert_eq!(degree, 2),
        other => panic!("expected PolynomialCost, got {other:?}"),
    }
}

#[test]
fn normalize_keeps_singleton_costs_unchanged() {
    // `normalize` is structural — non-Sum/Product variants pass
    // through unmodified.
    let (port, _) = two_distinct_ports();
    for cost in [
        constant(7),
        linear(port),
        log_cost(port),
        SymbolicCost::UnknownCost {
            _0: "opaque".to_string(),
        },
    ] {
        let normalized = normalize(cost.clone());
        assert_eq!(normalized, cost, "normalize should pass through leaf costs");
    }
}

// ── Generated-module staleness guard ─────────────────────────────

const GENERATED_LENS_HEADER: &str = "// AUTO-GENERATED from `src/v3/lenses/cost.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("cost.dag")
}

fn emit_lens_module() -> String {
    let source = std::fs::read_to_string(lens_path()).expect("read cost.dag");
    let dag = compile_to_dag(&source, lens_path().to_string_lossy().as_ref())
        .expect("cost.dag should compile cleanly");
    assert!(
        dag.diagnostics().is_empty(),
        "cost.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit compiled lens module");
    format_rust_source(&format!("{GENERATED_LENS_HEADER}{raw}"))
}

fn format_rust_source(source: &str) -> String {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn rustfmt");
    child
        .stdin
        .as_mut()
        .expect("rustfmt stdin")
        .write_all(source.as_bytes())
        .expect("write source to rustfmt");
    let output = child.wait_with_output().expect("wait for rustfmt");
    assert!(
        output.status.success(),
        "rustfmt failed on emitted lens module"
    );
    String::from_utf8(output.stdout).expect("rustfmt output should be utf-8")
}

#[test]
fn cost_dag_compiles_cleanly() {
    let source = std::fs::read_to_string(lens_path()).expect("read cost.dag");
    let dag = compile_to_dag(&source, lens_path().to_string_lossy().as_ref())
        .expect("cost.dag should compile cleanly");
    assert!(
        dag.diagnostics().is_empty(),
        "cost.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn cost_generated_module_matches_checked_in_snapshot() {
    let fresh = emit_lens_module();
    let checked_in = include_str!("../src/lens_cost_symbolic_generated.rs");
    assert_eq!(
        fresh.trim(),
        checked_in.trim(),
        "checked-in `lens_cost_symbolic_generated.rs` is stale; run `cargo run --bin regen_lens_cost_symbolic`"
    );
}
