//! #6777 construction-residue reproducer: four-case interpreted-vs-emitted agreement.
//! Authored `classify_length` distinguishes 0/1/2+ elements via nested Cons tail patterns.
//! Main's FreeMonoid lowering collapses sibling Cons arms; this test fails at n=2 and n=3
//! until the emitter groups arms sharing an outer constructor into a nested match.

use v1_compiler::cli_run::{make_eval_context, resolve_entry_graph, run_claim, ClaimOutcome};
use v1_compiler::v1_gunbc_nested_match_6777_length_classifier::{
    classify_length, mk_list, LengthClass,
};
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

use crate::helpers::workspace_root;

fn classifier_entry() -> String {
    workspace_root()
        .join("src/v1/gunbc/nested_match_6777_length_classifier.dag")
        .to_string_lossy()
        .into_owned()
}

fn classifier_roots() -> Vec<String> {
    vec![
        workspace_root()
            .join("src/v1")
            .to_string_lossy()
            .into_owned(),
        workspace_root().join("dag").to_string_lossy().into_owned(),
    ]
}

fn length_class_tag(class: &LengthClass) -> &'static str {
    match class {
        LengthClass::Zero => "Zero",
        LengthClass::One => "One",
        LengthClass::Many => "Many",
    }
}

fn interpret_length_class_tag(ctx: &InterpContext, result: Value) -> String {
    match result {
        Value::Variant { variant_name, .. } => ctx.resolve(variant_name),
        Value::Int(0) => "Zero".to_string(),
        other => panic!("unexpected interpreted classify_length result: {other:?}"),
    }
}

fn interpret_classify_length(ctx: &InterpContext, n: i64) -> String {
    let list = v1_compiler::v1_interpreter::run_in_context_with_args(
        ctx,
        "mk_list",
        &[(Some("n".to_string()), Value::Int(n))],
        true,
    )
    .expect("interpret mk_list");
    let result = v1_compiler::v1_interpreter::run_in_context_with_args(
        ctx,
        "classify_length",
        &[(Some("xs".to_string()), list)],
        true,
    )
    .expect("interpret classify_length");
    interpret_length_class_tag(ctx, result)
}

#[test]
fn nested_match_6777_authored_fixture_interpreter_holds() {
    let (graph, si) =
        resolve_entry_graph(&classifier_roots(), &classifier_entry()).expect("resolve classifier");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Hermetic);
    assert!(
        matches!(
            run_claim(&ctx, "classify_length_witness_holds"),
            ClaimOutcome::Pass
        ),
        "authored fixture must classify 0->Zero, 1->One, 2/3->Many under the interpreter"
    );
}

#[test]
fn nested_match_6777_emitted_matches_interpreted() {
    let (graph, si) =
        resolve_entry_graph(&classifier_roots(), &classifier_entry()).expect("resolve classifier");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Hermetic);
    for n in 0..=3 {
        let emitted = length_class_tag(&classify_length(mk_list(n)));
        let interpreted = interpret_classify_length(&ctx, n);
        assert_eq!(
            emitted, interpreted,
            "#6777 reproducer: emitted vs interpreted mismatch for n={n} (expected agreement after nested-match construction fix)"
        );
    }
}
