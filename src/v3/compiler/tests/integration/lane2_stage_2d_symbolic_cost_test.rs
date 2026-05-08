// Lane 2 Stage 2d / DB-7 — symbolic-cost lens acceptance tests.
#![allow(clippy::needless_borrows_for_generic_args)]
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

use v3_compiler::dag::{
    dominates, iterate, max_path, normalize, sequential, Behavior, Dag, DegreeAtLeastTwo,
    NonSingletonList, PortId, SizeVariable, SymbolicCost, TransformTarget, TypeConnective,
};
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};

use crate::common::cached_compile_to_dag;

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
        SymbolicCostLookup::Hit(cost) => cost,
        SymbolicCostLookup::Miss => {
            panic!("symbolic_cost_of returned Miss for {port:?}")
        }
    }
}

fn bind_cost(source: &str, file: &str, name: &str) -> SymbolicCost {
    let dag = cached_compile_to_dag(source, file);
    expect_cost(&dag, find_bind_value(&dag, name))
}

fn is_constant(cost: &SymbolicCost) -> bool {
    matches!(cost, SymbolicCost::ConstantCost { .. })
}

fn mentions_linear(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::LinearCost { .. } => true,
        SymbolicCost::SumCost { _0: terms } | SymbolicCost::ProductCost { _0: terms } => {
            terms.iter().any(|term| mentions_linear(term.as_ref()))
        }
        _ => false,
    }
}

fn mentions_log(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::LogCost { .. } => true,
        SymbolicCost::SumCost { _0: terms } | SymbolicCost::ProductCost { _0: terms } => {
            terms.iter().any(|term| mentions_log(term.as_ref()))
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
    SizeVariable {
        source_port,
        display_name: None,
    }
}

fn linear(port: PortId) -> SymbolicCost {
    SymbolicCost::LinearCost { _0: size_var(port) }
}

fn polynomial(port: PortId, degree: i64) -> SymbolicCost {
    SymbolicCost::PolynomialCost {
        var: size_var(port),
        degree: DegreeAtLeastTwo::new(degree).expect("polynomial degree must be >= 2"),
    }
}

fn log_cost(port: PortId) -> SymbolicCost {
    SymbolicCost::LogCost { _0: size_var(port) }
}

fn constant(n: i64) -> SymbolicCost {
    SymbolicCost::ConstantCost { _0: n }
}

fn product_nsl(a: SymbolicCost, b: SymbolicCost, c: SymbolicCost) -> SymbolicCost {
    SymbolicCost::ProductCost {
        _0: NonSingletonList::from_vec(vec![Box::new(a), Box::new(b), Box::new(c)])
            .expect("product NSL requires at least two terms"),
    }
}

fn sum_nsl(a: SymbolicCost, b: SymbolicCost, c: SymbolicCost) -> SymbolicCost {
    SymbolicCost::SumCost {
        _0: NonSingletonList::from_vec(vec![Box::new(a), Box::new(b), Box::new(c)])
            .expect("sum NSL requires at least two terms"),
    }
}

/// `dominates` on `ProductCost` / `SumCost` matches the `.dag` composite branch
/// (`nsl_to_list` + `fold_or_dominate_scan` over `NonSingletonList`). The Rust
/// helper is still named `any_dominates` in `dag_cost_generated.rs`. Regression:
/// template / regen must not skip `second` or `rest` when checking child dominance.
#[test]
fn composite_dominance_considers_all_nsl_children() {
    let (p0, _) = two_distinct_ports();
    let rhs = linear(p0);

    let only_first = product_nsl(linear(p0), constant(1), constant(1));
    assert!(
        dominates(&only_first, &rhs),
        "ProductCost `first` must count toward dominance"
    );

    let only_second = product_nsl(constant(1), linear(p0), constant(1));
    assert!(
        dominates(&only_second, &rhs),
        "ProductCost `second` must count toward dominance"
    );

    let only_rest = product_nsl(constant(1), constant(1), linear(p0));
    assert!(
        dominates(&only_rest, &rhs),
        "ProductCost `rest` must count toward dominance"
    );

    assert!(
        dominates(&sum_nsl(linear(p0), constant(1), constant(1)), &rhs),
        "SumCost `first` must count toward dominance"
    );
    assert!(
        dominates(&sum_nsl(constant(1), linear(p0), constant(1)), &rhs),
        "SumCost `second` must count toward dominance"
    );
    assert!(
        dominates(&sum_nsl(constant(1), constant(1), linear(p0)), &rhs),
        "SumCost `rest` must count toward dominance"
    );
}

fn boxed_terms_to_vec(terms: &NonSingletonList<Box<SymbolicCost>>) -> Vec<SymbolicCost> {
    terms.iter().map(|term| term.as_ref().clone()).collect()
}

#[test]
fn degree_at_least_two_is_structural_in_bootstrap_substrate() {
    let dag = Dag::new();
    let decl = dag
        .declaration_by_name("DegreeAtLeastTwo")
        .expect("DegreeAtLeastTwo should bootstrap from std/algebra.dag");
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!(
            "DegreeAtLeastTwo should be a recursive sum, got {:?}",
            decl.connective
        );
    };
    assert_eq!(
        variants.len(),
        2,
        "degree carrier should expose 2 structural variants"
    );
    assert_eq!(variants[0].label, "DegreeTwo");
    assert_eq!(variants[1].label, "DegreeSuccessor");

    let succ = dag.declaration(variants[1].ty);
    let TypeConnective::Conj { children } = &succ.connective else {
        panic!(
            "DegreeSuccessor payload should be a record, got {:?}",
            succ.connective
        );
    };
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].label, "previous");
    assert_eq!(
        dag.declaration(children[0].ty).name.as_deref(),
        Some("DegreeAtLeastTwo"),
        "successor must recurse structurally to DegreeAtLeastTwo"
    );
}

#[test]
fn symbolic_cost_semiring_inhabitance_is_bootstrap_substrate_fact() {
    let dag = Dag::new();
    let semiring = dag
        .declaration_by_name("Semiring")
        .expect("Semiring should bootstrap from std/algebra.dag")
        .id;
    let symbolic_cost = dag
        .declaration_by_name("SymbolicCost")
        .expect("SymbolicCost should bootstrap from v3 std algebra")
        .clone();
    let inhabitance = symbolic_cost
        .inhabits
        .expect("SymbolicCost should carry an inhabits edge");
    match &dag.declaration(inhabitance).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            assert_eq!(*template, semiring);
            assert_eq!(
                arguments.iter().map(|arg| arg.value).collect::<Vec<_>>(),
                vec![symbolic_cost.id],
                "SymbolicCost must inhabit Semiring<SymbolicCost>"
            );
        }
        other => panic!("expected Semiring<SymbolicCost> instantiation, got {other:?}"),
    }
}

#[test]
fn symbolic_cost_product_identity_stage_is_bootstrap_substrate_fact() {
    let dag = Dag::new();
    let reduce_product = dag
        .declaration_by_name("reduce_product")
        .expect("reduce_product should bootstrap from std/algebra.dag")
        .id;
    let drop_multiplicative_one = dag
        .declaration_by_name("drop_multiplicative_one")
        .expect("drop_multiplicative_one should bootstrap from std/algebra.dag")
        .id;
    let collapse_on_multiplicative_zero = dag
        .declaration_by_name("collapse_on_multiplicative_zero")
        .expect("collapse_on_multiplicative_zero should bootstrap from std/algebra.dag")
        .id;

    let found_product_identity_chain = dag.nodes().iter().any(|node| {
        let Behavior::Transform(reduce) = node else {
            return false;
        };
        if reduce.target != TransformTarget::Callable(reduce_product) {
            return false;
        }
        let Some(drop_one) = reduce
            .inputs
            .iter()
            .filter_map(|input| dag.resolve_producer_opt(input))
            .find_map(Behavior::as_transform)
        else {
            return false;
        };
        if drop_one.target != TransformTarget::Callable(drop_multiplicative_one) {
            return false;
        }
        drop_one
            .inputs
            .iter()
            .filter_map(|input| dag.resolve_producer_opt(input))
            .filter_map(Behavior::as_transform)
            .any(|collapse_zero| {
                collapse_zero.target == TransformTarget::Callable(collapse_on_multiplicative_zero)
            })
    });

    assert!(
        found_product_identity_chain,
        "bootstrap normalize(ProductCost) path must preserve collapse-zero -> drop-one -> \
         reduce-product so Semiring<SymbolicCost> has substrate-visible multiplicative identity"
    );
}

// ── Per-Behavior lowering ────────────────────────────────────────

budgeted_test! {
    value_reports_constant,
    {
        // `let x = 1` → ConstantCost(0) (leaf literal, no work).
        let dag = cached_compile_to_dag("let x = 1", "test.v3");
        let cost = expect_cost(&dag, find_bind_value(&dag, "x"));
        assert!(
            is_constant(&cost),
            "literal value should report Constant, got {cost:?}"
        );
    }
}

budgeted_test! {
    transform_single_op_reports_constant,
    {
        // `let x = 1 + 2` → sequential(Constant(1), sum(Constant(0), Constant(0)))
        // → Constant wrapper. Asymptotically O(1).
        let cost = bind_cost("let x = 1 + 2", "test.v3", "x");
        assert!(
            is_constant(&cost),
            "single scalar op should report Constant, got {cost:?}"
        );
    }
}

#[test]
fn branch_reports_constant_when_both_arms_constant() {
    // `if 1 > 0 then 10 else 20` — both arms are leaf literals, so
    // max_path over two Constants stays Constant.
    //
    // Ratchet exception (unchanged after #546): #546's cache consolidation
    // keys on `(source, file)`, and this fixture's pair is unique across the
    // suite — the shared cache never warms it. On the CI narrow Stage 2d gate
    // this test runs first in alphabetical order and pays a cold
    // bootstrap + pipeline compile (measured 2.515s; past the 2s
    // `budgeted_test!` cap). Restoring under `budgeted_test!` was attempted
    // and rolled back when CI confirmed the cold cost. Honest dissolution
    // trigger: cache bootstrap Dag state as input to `compile_to_dag` (so
    // cold-pipeline cost drops, not just the per-source cold-compile cost),
    // OR change this fixture to share a `(source, file)` key with an existing
    // cached test in this module. #546's caching pattern alone does NOT
    // address this test's cold path.
    //
    // **Stack-budget bump (PR #2164 / S5 carrier landing):** the cold
    // bootstrap compile here traverses every substrate carrier declaration
    // including the new `v3.std.coproduct_projection` carrier; the typed
    // `DeclarationRef` + `Map<VariantId, CoproductVariantProjection>` shape
    // pushed the static-initializer + bootstrap-walk past the default
    // 2MB test-thread stack on Linux debug builds. Wrapped in an 8MB
    // thread per the `with_full_bootstrap_stack` precedent at
    // `m2_substrate_inhabitance_test.rs:23-35` (same fix applied to a
    // sibling substrate-bootstrap-heavy test). The cold-bootstrap cache
    // dissolution trigger above remains the load-bearing fix — bumping
    // stack is the cliff-edge workaround until that lands.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let cost = bind_cost("let r = if 1 > 0 then 10 else 20", "test.v3", "r");
            assert!(
                is_constant(&cost),
                "branch over constant arms should report Constant, got {cost:?}"
            );
        })
        .expect("spawn bootstrap-stack-bumped thread")
        .join()
        .expect("bootstrap-stack-bumped thread panicked");
}

budgeted_test! {
    recursive_fn_body_contributes_to_loop_cost,
    {
        // PR #537 reviewer call-out (briansrls, BLOCKING): the prior
        // `loop_cost(acc, l.source)` shape fed the loop its source
        // port as the body cost — the SAME port as the bound — so
        // body work contributed `ConstantCost(0)` (the pre-seeded
        // param-port cost) and `normalize` dropped it from the
        // iteration. A recursive fn `fn f(n) = if ... else f(n - 1)`
        // then flattened to a bare `LinearCost` instead of a
        // `ProductCost([LinearCost, body-cost])`.
        //
        // Post-fix: `l.body: NodeId` is resolved through
        // `node(d, body_id)` → result_port, and THAT port's cost
        // enters the iterate composition. The recursive fn's Branch
        // body has a `ConstantCost(1)` (the comparison op). With
        // `SymbolicCost` now honestly inhabiting `Semiring<SymbolicCost>`,
        // `ConstantCost(1)` is the multiplicative identity, so the final
        // normalized result is a bare `LinearCost`. The regression guard here
        // is therefore the semantic value: the loop must remain linear rather
        // than collapsing to constant/missing cost.
        let dag = cached_compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
            "loop_body_countdown.v3",
        );
        let cost = expect_cost(&dag, find_bind_value(&dag, "countdown"));
        assert!(
            matches!(cost, SymbolicCost::LinearCost { .. }),
            "recursive fn with O(1) body should normalize Linear * 1 to Linear, got {cost:?}"
        );
    }
}

#[test]
fn iterate_keeps_non_identity_body_cost_in_product() {
    let (bound_port, body_port) = two_distinct_ports();
    let cost = iterate(linear(bound_port), log_cost(body_port));
    let SymbolicCost::ProductCost { _0: terms } = &cost else {
        panic!("Linear-bound loop with non-identity body should stay ProductCost, got {cost:?}");
    };
    assert_eq!(
        terms.iter().count(),
        2,
        "iterate should compose bound and body costs exactly once"
    );
    assert!(
        mentions_linear(&cost) && mentions_log(&cost),
        "iterate must retain both the loop-bound and body-cost terms, got {cost:?}"
    );
}

budgeted_test! {
    recursive_fn_reports_linear_via_loop_lowering,
    {
        // `fn countdown(n) = if n == 0 then 0 else countdown(n - 1)`
        // lowers to a Loop whose source is the `n` param port. The
        // lens's `loop_cost` fires LinearCost(size_var_of(source)),
        // so the bound carries a LinearCost term.
        let dag = cached_compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
            "countdown.v3",
        );
        let cost = expect_cost(&dag, find_bind_value(&dag, "countdown"));
        assert!(
            mentions_linear(&cost),
            "recursive fn should surface a LinearCost term, got {cost:?}"
        );
    }
}

// ── Composition algebra (std.algebra Rust mirror) ────────────────

budgeted_test! {
    sum_of_linear_and_constant_zero_normalizes_to_linear,
    {
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
}

budgeted_test! {
    sum_of_constant_and_linear_normalizes_to_linear,
    {
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
}

budgeted_test! {
    product_of_two_linears_over_same_var_folds_to_polynomial_squared,
    {
        // DB-7 §"Nested fold detection (O(n²) diagnostic)":
        // `iterate(Linear(n), Linear(n))` folds to `Polynomial(n, 2)`.
        let (port, _) = two_distinct_ports();
        let l = linear(port);
        let result = iterate(l.clone(), l);
        match result {
            SymbolicCost::PolynomialCost { var, degree } => {
                assert_eq!(var, size_var(port));
                assert_eq!(degree.raw(), 2);
            }
            other => panic!("expected PolynomialCost(n, 2), got {other:?}"),
        }
    }
}

budgeted_test! {
    product_of_linears_over_different_vars_stays_product,
    {
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
}

budgeted_test! {
    product_with_constant_zero_collapses_to_zero,
    {
        let (port, _) = two_distinct_ports();
        let result = iterate(linear(port), constant(0));
        assert_eq!(
            result,
            constant(0),
            "Semiring<SymbolicCost> multiplication must treat ConstantCost(0) as an annihilator"
        );
    }
}

budgeted_test! {
    product_with_constant_one_normalizes_to_other_factor,
    {
        let (port, _) = two_distinct_ports();
        let l = linear(port);
        assert_eq!(
            iterate(l.clone(), constant(1)),
            l,
            "Semiring<SymbolicCost> multiplication must treat trailing ConstantCost(1) as identity"
        );
        assert_eq!(
            iterate(constant(1), l.clone()),
            l,
            "Semiring<SymbolicCost> multiplication must treat leading ConstantCost(1) as identity"
        );
        assert_eq!(
            iterate(constant(1), constant(1)),
            constant(1),
            "product of multiplicative identities should remain ConstantCost(1)"
        );
    }
}

budgeted_test! {
    dominance_unknown_dominates_everything,
    {
        // DB-7 §"Dominance" — Unknown is the safest over-approximation.
        let (port, _) = two_distinct_ports();
        let unknown = SymbolicCost::UnknownCost {
            _0: "reflection".to_string(),
        };
        assert!(dominates(&unknown, &linear(port)));
        assert!(dominates(&unknown, &constant(0)));
    }
}

budgeted_test! {
    dominance_linear_dominates_log_and_constant,
    {
        let (port, _) = two_distinct_ports();
        assert!(dominates(&linear(port), &log_cost(port)));
        assert!(dominates(&linear(port), &constant(3)));
        assert!(!dominates(&constant(3), &linear(port)));
    }
}

budgeted_test! {
    dominance_polynomial_degree_ordering,
    {
        let (port, _) = two_distinct_ports();
        assert!(dominates(&polynomial(port, 3), &polynomial(port, 2)));
        assert!(!dominates(&polynomial(port, 2), &polynomial(port, 3)));
    }
}

budgeted_test! {
    composite_dominance_reads_children_not_outer_variant,
    {
        // PR #537 review (codex P2): Product / Sum dominance must
        // derive from children, not hardcode "any Product dominates
        // Linear". A product whose strongest child is `Log(n)`
        // should NOT dominate `Linear(n)`.
        let (port, _) = two_distinct_ports();
        let product_of_logs = SymbolicCost::ProductCost {
            _0: NonSingletonList::from_vec(vec![Box::new(log_cost(port)), Box::new(log_cost(port))])
                .unwrap(),
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
            _0: NonSingletonList::from_vec(vec![Box::new(log_cost(port)), Box::new(linear(port))])
                .unwrap(),
        };
        assert!(
            dominates(&product_with_linear, &log_cost(port)),
            "Product([Log, Linear]) must dominate Log via its Linear child"
        );
    }
}

budgeted_test! {
    sum_dominance_reads_children_not_outer_variant,
    {
        // Same fix as ProductCost: Sum's dominance is a dominant-child
        // summary, not a hardcoded "any Sum dominates scalars".
        let (port, _) = two_distinct_ports();
        let sum_of_logs = SymbolicCost::SumCost {
            _0: NonSingletonList::from_vec(vec![Box::new(log_cost(port)), Box::new(log_cost(port))])
                .unwrap(),
        };
        assert!(
            !dominates(&sum_of_logs, &linear(port)),
            "Sum([Log, Log]) must NOT dominate Linear — no child reaches Linear"
        );

        let sum_with_polynomial = SymbolicCost::SumCost {
            _0: NonSingletonList::from_vec(vec![
                Box::new(constant(0)),
                Box::new(polynomial(port, 2)),
            ])
            .unwrap(),
        };
        assert!(
            dominates(&sum_with_polynomial, &linear(port)),
            "Sum containing Polynomial(n, 2) must dominate Linear(n) via the poly child"
        );
    }
}

budgeted_test! {
    max_path_returns_dominant_term,
    {
        // `max_path([Constant(0), Linear(n), Polynomial(n, 2)])` → Polynomial(n, 2).
        let (port, _) = two_distinct_ports();
        let paths = vec![constant(0), linear(port), polynomial(port, 2)];
        let result = max_path(&paths);
        match result {
            SymbolicCost::PolynomialCost { degree, .. } => assert_eq!(degree.raw(), 2),
            other => panic!("expected PolynomialCost, got {other:?}"),
        }
    }
}

budgeted_test! {
    max_path_preserves_both_incomparable_branches,
    {
        // PR #537 review (briansrls, BLOCKING): two incomparable
        // branches — `Linear(n)` over port_a vs `Linear(m)` over
        // port_b — must NOT drop one silently. `Big-O(f + g) =
        // Big-O(max(f, g))` for non-negative asymptotic terms, so the
        // preserved-as-sum shape is the honest worst case.
        let (port_a, port_b) = two_distinct_ports();
        let paths = vec![linear(port_a), linear(port_b)];
        let result = max_path(&paths);
        match &result {
            SymbolicCost::SumCost { _0: terms } => {
                assert_eq!(
                    terms.len(),
                    2,
                    "incomparable Linear(n) + Linear(m) must preserve both as a two-element Sum, \
                     got {result:?}"
                );
                assert!(
                    terms.iter().any(|term| term.as_ref() == &linear(port_a))
                        && terms.iter().any(|term| term.as_ref() == &linear(port_b)),
                    "both branch costs must remain in the Sum, got {result:?}"
                );
            }
            other => panic!("expected SumCost preserving both branches, got {other:?}"),
        }
    }
}

budgeted_test! {
    max_path_order_independence_on_incomparable_branches,
    {
        // Reversing the input order must yield the same asymptotic
        // shape — the previous two-way dominance fold depended on
        // fold order and dropped whichever branch landed later when
        // both were incomparable.
        let (port_a, port_b) = two_distinct_ports();
        let forward = max_path(&[linear(port_a), linear(port_b)]);
        let reversed = max_path(&[linear(port_b), linear(port_a)]);
        let (forward_terms, reversed_terms) = match (&forward, &reversed) {
            (SymbolicCost::SumCost { _0: fwd }, SymbolicCost::SumCost { _0: rev }) => {
                (boxed_terms_to_vec(fwd), boxed_terms_to_vec(rev))
            }
            _ => panic!(
                "both orderings should produce a SumCost, got forward={forward:?} reversed={reversed:?}"
            ),
        };
        assert_eq!(
            forward_terms.len(),
            reversed_terms.len(),
            "element count must not depend on fold order"
        );
        assert!(
            forward_terms.iter().all(|t| reversed_terms.contains(t))
                && reversed_terms.iter().all(|t| forward_terms.contains(t)),
            "the preserved set of terms must be the same regardless of input order; \
             got forward={forward_terms:?} reversed={reversed_terms:?}"
        );
    }
}

budgeted_test! {
    normalize_keeps_singleton_costs_unchanged,
    {
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
    let dag = cached_compile_to_dag(&source, lens_path().to_string_lossy().as_ref());
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

// Cold-init path for the `cost.dag` OnceLock cache key. Sibling
// `cost_generated_module_matches_checked_in_snapshot` also compiles
// cost.dag and reuses the cached Dag, so this test legitimately
// bears the first `compile_to_dag(cost.dag)` cost on CI (~2.5s on typical cold
// runners; busy shared runners or heavier bootstrap — `parse_surface.dag`
// in the bundle and substrate growth — can exceed ~7s wall on integration
// binaries). `15_000`ms keeps headroom under this ratchet without matching the
// sibling's 45s snapshot-compare budget below.
budgeted_test! {
    15_000,
    cost_dag_compiles_cleanly,
    {
        let source = std::fs::read_to_string(lens_path()).expect("read cost.dag");
        let dag = cached_compile_to_dag(&source, lens_path().to_string_lossy().as_ref());
        assert!(
            dag.diagnostics().is_empty(),
            "cost.dag should compile without diagnostics, got {:?}",
            dag.diagnostics()
        );
    }
}

// rustfmt + snapshot compare can spike on cold CI runners. Bootstrap includes
// `parse_surface.dag` (regen_parse authority lives in the same bundle), so
// `Dag::new()` clones and the first `compile_to_dag(cost.dag)` pay more than
// the old 15s cap; a cold integration-binary run can also spend tens of seconds
// in link + first `LazyLock` bootstrap before this test's body starts.
budgeted_test! {
    45_000,
    cost_generated_module_matches_checked_in_snapshot,
    {
        let fresh = emit_lens_module();
        let checked_in = include_str!("../../src/lens_cost_symbolic_generated.rs");
        assert_eq!(
            fresh.trim(),
            checked_in.trim(),
            "checked-in `lens_cost_symbolic_generated.rs` is stale; run `cargo run -p v3-compiler --bin regen_lens -- --lens cost_symbolic`"
        );
    }
}
