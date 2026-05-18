//! **Layer:** integration
//!
//! T-19 Wave-0: `src/v4/lens/testgen.dag` parses and exposes manual-anchor-key-driven
//! `Generator` wiring (`kind` + `t19_anchor` + `classification` + `slot: TestgenConcept`).
//! `AssertKind` selection for anchors lives in `v4.std.verification` (`assert_kind_for_manual_anchor`)
//! as the single substrate authority next to `TestClaim.kind`.
//! **Note:** `compile_to_dag` on this module alone does not resolve `import v4.std.*` peers
//! (Import lowering is still M2-scoped); full merge compile lands with cross-file M2 per TASKS T-19.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

#[test]
fn v4_lens_testgen_wave0_substrate_parses() {
    let testgen = parse_module(
        include_str!("../../../../v4/lens/testgen.dag"),
        "src/v4/lens/testgen.dag",
    );
    let verification = parse_module(
        include_str!("../../../../v4/std/verification.dag"),
        "src/v4/std/verification.dag",
    );

    assert_eq!(
        module_paths(&testgen),
        vec![vec!["v4", "lens", "testgen"]],
        "T-19 authority module should remain v4.lens.testgen"
    );
    assert_eq!(
        function_count(&testgen, "bootstrap_claim_generator_for_manual_anchor"),
        1,
        "T-19 Wave-0: single generator entrypoint keyed by T19ManualAnchorKey"
    );
    assert_eq!(function_count(&testgen, "testgen_concept_for_manual_anchor"), 1);

    assert_eq!(
        module_paths(&verification),
        vec![vec!["v4", "std", "verification"]],
        "`assert_kind_for_manual_anchor` substrate authority should remain v4.std.verification"
    );

    assert_eq!(
        function_count(&verification, "assert_kind_for_manual_anchor"),
        1,
        "T-19 manual-anchor AssertKind authority should remain `v4.std.verification`"
    );

    let bootstrap_rt = fn_return_type(&testgen, "bootstrap_claim_generator_for_manual_anchor")
        .expect("bootstrap_claim_generator_for_manual_anchor should have an explicit return type");
    assert!(
        type_is_outcome_generator_testgen_concept(bootstrap_rt),
        "T-19 Wave-0 bootstrap must fail-close `T19ManualAnchorAbsent` via `Outcome<Generator<TestgenConcept>>`; got {bootstrap_rt:?}"
    );
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn fn_return_type<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Option<&'a SurfaceType> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Fn {
            name: item_name,
            return_type,
            ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name,
            return_type,
            ..
        } => (item_name == name).then_some(return_type),
        _ => None,
    })
}

fn type_is_generator_testgen_concept(ty: &SurfaceType) -> bool {
    let SurfaceType::Parameterized { name, args, .. } = ty else {
        return false;
    };
    if name != "Generator" || args.len() != 1 {
        return false;
    }
    let TypeAngleArg::TypeExpr { ty: inner } = &args[0] else {
        return false;
    };
    matches!(
        inner.as_ref(),
        SurfaceType::Named { name: slot, .. } if slot == "TestgenConcept"
    )
}

fn type_is_outcome_generator_testgen_concept(ty: &SurfaceType) -> bool {
    let SurfaceType::Parameterized { name, args, .. } = ty else {
        return false;
    };
    if name != "Outcome" || args.len() != 1 {
        return false;
    }
    let TypeAngleArg::TypeExpr { ty: inner } = &args[0] else {
        return false;
    };
    type_is_generator_testgen_concept(inner.as_ref())
}

fn function_count(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> usize {
    module
        .items
        .iter()
        .filter(|item| match item {
            SurfaceItem::Fn {
                name: item_name, ..
            }
            | SurfaceItem::FnExternalBody {
                name: item_name, ..
            } => item_name == name,
            _ => false,
        })
        .count()
}
