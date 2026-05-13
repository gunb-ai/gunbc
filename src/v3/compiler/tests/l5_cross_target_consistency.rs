//! **Layer:** boundary
//!
//! R3 §1.8 gate **#15** `l5_cross_target_consistency`: certification corpus with N>0 programs, each
//! exercised under `ForAllTargets` so emitted Rust, Python, and Go agree on the observed `Int`
//! stdout carve-out (`docs/r3-actual-close-plan.md` §Gap 2 close criterion). The full-suite test
//! requires `rustc`, `python3`, and `go` on `PATH` (same host contract as the prior L5 skeleton).

#[path = "integration/common/mod.rs"]
mod common;

use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, FieldValue, LiteralBits};
use v3_compiler::test_runner::{ClaimResult, TestClaimValue, TestRunner};
use v3_compiler::CompileError;

use common::run_on_larger_stack;

const L5_FIXTURE: &str = include_str!("fixtures/r3_verification_l5_corpus.dag");
const L5_FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag";
const L5_SUITE: &str = "l5_cross_target_consistency";

/// `(fixture declaration id, canonical TestClaim.name, authority .v3 bytes)`.
const L5_CORPUS_AUTHORITY: &[(&str, &str, &str)] = &[
    (
        "l5_cross_target_consistency_add_then",
        "l5_cross_target_consistency",
        include_str!("fixtures/r3_l5_corpus/add_then_branch_seed.v3"),
    ),
    (
        "r3_l5_cert_branch_literal_true",
        "r3_l5_cert_branch_literal_true",
        include_str!("fixtures/r3_l5_corpus/branch_literal_true_seed.v3"),
    ),
    (
        "r3_l5_cert_branch_literal_false",
        "r3_l5_cert_branch_literal_false",
        include_str!("fixtures/r3_l5_corpus/branch_literal_false_seed.v3"),
    ),
    (
        "r3_l5_cert_nested_branch",
        "r3_l5_cert_nested_branch",
        include_str!("fixtures/r3_l5_corpus/nested_branch_seed.v3"),
    ),
];

static L5_DAG: OnceLock<Dag> = OnceLock::new();

fn cached_l5_dag() -> &'static Dag {
    L5_DAG.get_or_init(|| match compile_to_dag(L5_FIXTURE, L5_FIXTURE_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{L5_FIXTURE_PATH}: expected empty module diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{L5_FIXTURE_PATH} should lower without module diagnostics. Got `Err(Semantic)`: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("unexpected compile error for {L5_FIXTURE_PATH}: {other:?}"),
    })
}

#[test]
fn l5_certification_corpus_sources_match_authority_files() {
    let dag = cached_l5_dag();
    for &(decl_name, claim_name, authority) in L5_CORPUS_AUTHORITY {
        let claim_decl = dag.declaration_by_name(decl_name).unwrap_or_else(|| {
            panic!("missing `{decl_name}` in {L5_FIXTURE_PATH}");
        });
        let claim = TestClaimValue::from_declaration(claim_decl).unwrap_or_else(|reason| {
            panic!("`{decl_name}` should lower to a structural TestClaim: {reason}");
        });
        assert_eq!(
            claim.claim_name, claim_name,
            "`TestClaim.name` must match corpus table for `{decl_name}`"
        );
        assert_eq!(
            claim.source, authority,
            "`TestClaim.source` must equal `{decl_name}` authority .v3 bytes"
        );
    }
}

#[test]
fn l5_cross_target_consistency_gate_claim_predicate_shape() {
    run_on_larger_stack(|| {
        let dag = cached_l5_dag();
        let claim_decl = dag
            .declaration_by_name("l5_cross_target_consistency_add_then")
            .expect("missing l5_cross_target_consistency_add_then");
        let claim = TestClaimValue::from_declaration(claim_decl).expect("structural TestClaim");
        assert_eq!(claim.claim_name, "l5_cross_target_consistency");
        let FieldValue::Variant { payload, .. } = &claim.predicate else {
            panic!("L5 predicate must lower to a structural ForAllTargets variant");
        };
        let [FieldValue::Literal(LiteralBits::String(command)), FieldValue::List(args), FieldValue::Literal(LiteralBits::Int(expect_exit_code)), FieldValue::Reference(input_ref)] =
            payload.as_slice()
        else {
            panic!(
                "L5 ForAllTargets must carry inert command triple plus ProgramOutputBind input_ref, got {payload:?}"
            );
        };
        assert_eq!(command, "true");
        assert!(args.is_empty());
        assert_eq!(expect_exit_code, "0");
        assert_eq!(
            dag.declaration(*input_ref).name.as_deref(),
            Some("r3_l5_gate_program_output"),
            "L5 ForAllTargets must structurally select the gate ProgramOutputBind"
        );
        let required_toolchains: Vec<_> = claim
            .requires
            .iter()
            .map(|requirement| match requirement {
                FieldValue::Record(fields) => match fields.as_slice() {
                    [(label, FieldValue::Reference(id))] if label == "target" => dag
                        .declaration(*id)
                        .name
                        .as_deref()
                        .unwrap_or("<anonymous>")
                        .to_string(),
                    other => panic!(
                        "L5 `requires` entry must be ResourceReference {{ target }}, got {other:?}"
                    ),
                },
                other => {
                    panic!("L5 `requires` entry must be a ResourceReference record, got {other:?}")
                }
            })
            .collect();
        assert_eq!(
            required_toolchains,
            ["L5RustcToolchain", "L5Python3Toolchain", "L5GoToolchain"],
            "L5 ForAllTargets must declare host toolchain requirements structurally on TestClaim.requires"
        );
    });
}

#[test]
fn l5_cross_target_consistency_suite_passes_all_corpus_rows() {
    run_on_larger_stack(|| {
        let dag = cached_l5_dag();
        let results = TestRunner::new(dag).run_suite(L5_SUITE);
        assert_eq!(
            results.len(),
            L5_CORPUS_AUTHORITY.len(),
            "`{L5_SUITE}` should run one receipt per certification program"
        );
        let expected_names: Vec<_> = L5_CORPUS_AUTHORITY
            .iter()
            .map(|(_, name, _)| *name)
            .collect();
        let actual_names: Vec<_> = results.iter().map(|r| r.claim_name.as_str()).collect();
        assert_eq!(
            actual_names, expected_names,
            "suite claim order / naming drift — update L5_CORPUS_AUTHORITY (fixture is authority)"
        );
        assert!(
            results
                .iter()
                .all(|r| matches!(r.result, ClaimResult::Pass)),
            "every corpus row should Pass ForAllTargets Rust/Python/Go Int parity; got {results:?}"
        );
    });
}
