// Lane 2 Stage 2c acceptance smoke: DB-15 R2 tests-as-declarations
// extensions to `src/v3/std/verification.dag`.
//
// The Stage 2c scope (per docs/design-test-infra.md R2 and
// docs/lane2-compile-time-proofs.md §Stage 2c) extends the existing
// `TestClaim`/`TestPredicate`/`TestSuite` authority with
//   - `TestClaim.requires: List<ResourceReference>`
//   - `TestPredicate::BehavioralObservation { subject, input_sample,
//      expected_output }`
//   - `TestPredicate::MockBackedInvariant { subject, mock_transport,
//      invariant }`
//   - `fn materialize_obligations(suite: TestSuite) -> List<TestClaim>`
//     as the documented dependency-walk entry point.
//
// This smoke test encodes R2's acceptance at the bootstrap level. The
// runtime runner that *consumes* obligations is out of scope for this
// PR per the brief's consumer-contract deferral.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    ArrowBody, Dag, DeclarationId, FieldValue, LiteralBits, PortState, TypeConnective,
};
use v3_compiler::lens_testgen::TestgenLens;
use v3_compiler::CompileError;

fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

fn find_named(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

fn record_fields(dag: &Dag, name: &str) -> Vec<String> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Conj { children } => {
            children.iter().map(|field| field.label.clone()).collect()
        }
        other => panic!("expected `{name}` to lower to a Conj, got {other:?}"),
    }
}

fn variant_labels(dag: &Dag, name: &str) -> Vec<String> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .map(|variant| variant.label.clone())
            .collect(),
        other => panic!("expected `{name}` to lower to a Disj, got {other:?}"),
    }
}

fn variant_payload_fields(dag: &Dag, sum_name: &str, variant_label: &str) -> Vec<String> {
    let id = find_named(dag, sum_name);
    match &dag.declaration(id).connective {
        TypeConnective::Disj { variants } => {
            let variant = variants
                .iter()
                .find(|v| v.label == variant_label)
                .unwrap_or_else(|| {
                    panic!(
                        "variant `{variant_label}` not found under `{sum_name}`, got {:?}",
                        variants.iter().map(|v| v.label.clone()).collect::<Vec<_>>()
                    )
                });
            match &dag.declaration(variant.ty).connective {
                TypeConnective::Conj { children } => {
                    children.iter().map(|field| field.label.clone()).collect()
                }
                other => panic!(
                    "expected variant `{variant_label}` under `{sum_name}` to carry a \
                     Conj payload, got {other:?}"
                ),
            }
        }
        other => panic!("expected `{sum_name}` to lower to a Disj, got {other:?}"),
    }
}

fn arrow_body(dag: &Dag, name: &str) -> ArrowBody {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("expected `{name}` declaration after bootstrap"));
    match &decl.connective {
        TypeConnective::Arrow { body, .. } => body.clone(),
        other => panic!("expected `{name}` to be Arrow, got {other:?}"),
    }
}

#[test]
fn verification_dag_bootstraps_without_diagnostics() {
    let dag = Dag::new();
    let verification_diags: Vec<_> = dag
        .diagnostics()
        .iter()
        .filter(|(_, diag)| format!("{diag:?}").contains("verification.dag"))
        .collect();
    assert!(
        verification_diags.is_empty(),
        "src/v3/std/verification.dag produced diagnostics during bootstrap: {verification_diags:?}"
    );
}

// R2 open-question #1 lock: `ResourceReference` is a typed sentinel at
// the compile-time declaration-reference layer, NOT a runtime handle.
// Shape: `{ identifier: String }` carrying the bootstrap-local
// scaffold surface while the `resource` grammar + v3 resources port
// land. Post-port, `identifier: String` dissolves into a narrowed
// `DeclarationRef` shape rejecting non-`resource` declarations.
#[test]
fn resource_reference_is_a_compile_time_declaration_sentinel() {
    let dag = Dag::new();
    assert_eq!(
        record_fields(&dag, "ResourceReference"),
        vec!["identifier"],
        "ResourceReference should carry the bootstrap-local `identifier` \
         scaffold field while the dsl/std/resources.dag → v3 port is \
         deferred (DB-15 R2 §Prerequisite)"
    );
}

// R2 open-question #2 lock: `requires` is declared on `TestClaim`, NOT
// duplicated onto each predicate variant. A single subject exercised by
// multiple predicate variants has one resource declaration set.
#[test]
fn requires_is_per_claim_not_per_predicate_variant() {
    let dag = Dag::new();

    assert_eq!(
        record_fields(&dag, "TestClaim"),
        vec!["name", "source", "file_name", "predicate", "requires"],
        "TestClaim should carry `requires` as its own field"
    );

    // The runtime-backed predicate variants take their own typed
    // references, but NOT a `requires` list — that would duplicate
    // TestClaim.requires.
    assert_eq!(
        variant_payload_fields(&dag, "TestPredicate", "BehavioralObservation"),
        vec!["subject", "input_sample", "expected_output"],
        "BehavioralObservation should not carry its own `requires` list"
    );
    assert_eq!(
        variant_payload_fields(&dag, "TestPredicate", "MockBackedInvariant"),
        vec!["subject", "mock_transport", "invariant"],
        "MockBackedInvariant should not carry its own `requires` list"
    );
}

// R2 open-question #3 lock: tautology avoidance is structural. The
// coproduct does not admit a variant whose verification oracle is the
// same lens that produced the fact. `BehavioralObservation` takes a
// separately-declared `expected_output`; `MockBackedInvariant` takes a
// separately-declared `invariant`. A hypothetical
// `LensSays { lens, subject, expected }` variant is NOT in the
// coproduct.
#[test]
fn test_predicate_coproduct_blocks_lens_rerun_tautology() {
    let dag = Dag::new();
    let variants = variant_labels(&dag, "TestPredicate");

    for required in [
        "Compiles",
        "FailsWithDiagnostic",
        "OutputEquals",
        "PortHasState",
        "CostBounded",
        "BehavioralObservation",
        "MockBackedInvariant",
    ] {
        assert!(
            variants.iter().any(|v| v == required),
            "TestPredicate should admit `{required}`, got {variants:?}"
        );
    }

    for forbidden in ["LensSays", "LensEquals", "LensOutput", "RunLens"] {
        assert!(
            !variants.iter().any(|v| v == forbidden),
            "TestPredicate must NOT admit `{forbidden}` — that would \
             re-open DB-15 R2 tautology avoidance (see \
             src/v3/std/verification.dag §TestPredicate docblock)"
        );
    }
}

// Materialization entry point: `fn materialize_obligations(suite:
// TestSuite) -> List<TestClaim>` — documented in verification.dag.
// Under-the-hood this is a structural projection: the compiler's
// dependency walk does the sharing / caching / scoping; the function
// names what a runner consumes, nothing more.
#[test]
fn materialize_obligations_is_declared_as_dependency_walk_entry_point() {
    let dag = Dag::new();
    let _body = arrow_body(&dag, "materialize_obligations");
}

// End-to-end: a declared TestSuite with claims whose `requires` and
// predicate payloads compile cleanly. Exercises the R2 schema from
// user code (not just the bootstrap shape).
#[test]
fn r2_schema_accepts_user_authored_suite_with_requires() {
    let src = r#"
let pred_compiles: TestPredicate = Compiles

let claim_with_runtime_resources: TestClaim = {
  name: "idempotency_gcp_sts_exchange",
  source: "let x: Int = 1",
  file_name: "idempotency_gcp_sts_exchange.v3",
  predicate: pred_compiles,
  requires: [
    { identifier: "test_runner" },
    { identifier: "mock_gcp_sts_transport" }
  ]
}

let claim_compile_time_only: TestClaim = {
  name: "workflow_compiles",
  source: "let x: Int = 1",
  file_name: "workflow_compiles.v3",
  predicate: pred_compiles,
  requires: [{ identifier: "compile_time" }]
}

let suite: TestSuite = {
  name: "stage_2c_r2_smoke",
  claims: [claim_with_runtime_resources, claim_compile_time_only]
}
"#;

    let dag = compile_any(src, "stage_2c_r2_smoke.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "DB-15 R2 schema should accept a user-authored TestSuite with \
         per-claim `requires` references, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    // Both claims should resolve to the same TestClaim type.
    let test_claim = find_named(&dag, "TestClaim");
    let test_suite = find_named(&dag, "TestSuite");

    for bind_name in ["claim_with_runtime_resources", "claim_compile_time_only"] {
        let value_port = dag
            .nodes()
            .iter()
            .find_map(|node| match node {
                v3_compiler::dag::Behavior::Bind(bind) if bind.name == bind_name => {
                    Some(bind.value)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("bind `{bind_name}` not found"));
        match dag.port(value_port).state() {
            PortState::Resolved(ty) => {
                assert_eq!(
                    ty.declaration, test_claim,
                    "bind `{bind_name}` should resolve to TestClaim"
                );
            }
            other => panic!("bind `{bind_name}` did not resolve, got {other:?}"),
        }
    }

    let suite_port = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            v3_compiler::dag::Behavior::Bind(bind) if bind.name == "suite" => Some(bind.value),
            _ => None,
        })
        .unwrap_or_else(|| panic!("bind `suite` not found"));
    match dag.port(suite_port).state() {
        PortState::Resolved(ty) => {
            assert_eq!(
                ty.declaration, test_suite,
                "bind `suite` should resolve to TestSuite"
            );
        }
        other => panic!("bind `suite` did not resolve, got {other:?}"),
    }
}

// Regression guard for a DB-15 R2 consumer hole flagged in PR review:
// `TestgenLens::push_claim` used to emit `requires: []` for every
// generated claim, but `src/v3/std/verification.dag` documents the
// bootstrap-local convention that compile-time predicates carry
// `requires: [{ identifier: "compile_time" }]`. With the earlier
// empty-list emission, a downstream runner consuming
// `materialize_obligations(suite).requires` would read generated
// compile-time claims as "no resources to acquire" and skip the
// intended resource-classification path. Every predicate variant
// this lens emits today (`Compiles`, `FailsWithDiagnostic`,
// `PortHasState`, `CostBounded`) is compile-time, so all generated
// claims must carry the single-element sentinel list.
#[test]
fn testgen_claims_carry_compile_time_sentinel_in_requires() {
    let dag = Dag::new();
    let claims = TestgenLens::new(&dag).query();
    assert!(
        !claims.is_empty(),
        "testgen lens should emit at least one claim against the \
         bootstrapped stdlib"
    );

    for claim in &claims {
        let requires_value = claim
            .fields()
            .iter()
            .find(|(label, _)| label == "requires")
            .map(|(_, value)| value)
            .unwrap_or_else(|| {
                panic!(
                    "generated claim `{}` missing `requires`",
                    claim.declaration_name()
                )
            });

        let FieldValue::List(entries) = requires_value else {
            panic!(
                "generated claim `{}` `requires` should lower to a list, got {requires_value:?}",
                claim.declaration_name()
            );
        };

        assert_eq!(
            entries.len(),
            1,
            "generated claim `{}` should carry a single compile-time \
             sentinel; got {} entries — empty list silently re-opens the \
             runner classification hole flagged in PR review",
            claim.declaration_name(),
            entries.len()
        );

        let FieldValue::Record(fields) = &entries[0] else {
            panic!(
                "generated claim `{}` `requires[0]` should be a \
                 ResourceReference record, got {:?}",
                claim.declaration_name(),
                entries[0]
            );
        };
        let identifier = fields
            .iter()
            .find(|(label, _)| label == "identifier")
            .map(|(_, value)| value)
            .unwrap_or_else(|| {
                panic!(
                    "generated claim `{}` `requires[0]` record missing `identifier`",
                    claim.declaration_name()
                )
            });
        assert_eq!(
            identifier,
            &FieldValue::Literal(LiteralBits::String("compile_time".to_string())),
            "generated claim `{}` `requires[0].identifier` should be \
             the compile-time sentinel",
            claim.declaration_name()
        );
    }
}
