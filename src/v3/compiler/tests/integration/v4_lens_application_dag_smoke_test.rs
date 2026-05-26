//! **Layer:** integration
//!
//! T-23 wire: `src/v4/lens/application.dag` — lens application surface carriers,
//! `apply_lens`, D1 `subterm_at` / `apply_diff`, advisory→fail-closed bridge.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (peer v4 smoke posture).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem};
use v3_compiler::tokenize_for_test;

const APPLICATION_DAG: &str = include_str!("../../../../v4/lens/application.dag");
const APPLICATION_PATH: &str = "src/v4/lens/application.dag";
const REPORT_DAG: &str = include_str!("../../../../v4/std/report.dag");
const REPORT_PATH: &str = "src/v4/std/report.dag";
const INTROSPECT_ADVISORY_CLAIM_DAG: &str = include_str!(
    "../../../../v4/test/claim/lens_application/apply_lens_introspect_rejection_is_advisory.dag"
);
const INTROSPECT_ADVISORY_CLAIM_PATH: &str =
    "src/v4/test/claim/lens_application/apply_lens_introspect_rejection_is_advisory.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn module_path(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<&str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .unwrap_or_default()
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

fn expr_contains_match(expr: &SurfaceExpr) -> bool {
    match expr {
        SurfaceExpr::Match { .. } => true,
        SurfaceExpr::Call { args, .. } | SurfaceExpr::PathCall { args, .. } => {
            args.iter().any(expr_contains_match)
        }
        SurfaceExpr::Operator { args, .. } => args.iter().any(expr_contains_match),
        SurfaceExpr::Lambda { body, .. } => expr_contains_match(body),
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_contains_match(cond)
                || expr_contains_match(then_branch)
                || expr_contains_match(else_branch)
        }
        SurfaceExpr::Record { fields, .. } | SurfaceExpr::VariantRecord { fields, .. } => {
            fields.iter().any(|field| expr_contains_match(&field.value))
        }
        SurfaceExpr::List { elements, .. } => elements.iter().any(expr_contains_match),
        SurfaceExpr::Map { entries, .. } => {
            entries.iter().any(|entry| expr_contains_match(&entry.value))
        }
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } | SurfaceExpr::Path { .. } => false,
    }
}

fn expr_has_match_arm_body_match(expr: &SurfaceExpr) -> bool {
    match expr {
        SurfaceExpr::Match { scrutinee, arms, .. } => {
            expr_contains_match(scrutinee)
                || arms.iter().any(|arm| expr_contains_match(&arm.body))
                || arms
                    .iter()
                    .any(|arm| expr_has_match_arm_body_match(&arm.body))
        }
        SurfaceExpr::Call { args, .. } | SurfaceExpr::PathCall { args, .. } => {
            args.iter().any(expr_has_match_arm_body_match)
        }
        SurfaceExpr::Operator { args, .. } => args.iter().any(expr_has_match_arm_body_match),
        SurfaceExpr::Lambda { body, .. } => expr_has_match_arm_body_match(body),
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_has_match_arm_body_match(cond)
                || expr_has_match_arm_body_match(then_branch)
                || expr_has_match_arm_body_match(else_branch)
        }
        SurfaceExpr::Record { fields, .. } | SurfaceExpr::VariantRecord { fields, .. } => fields
            .iter()
            .any(|field| expr_has_match_arm_body_match(&field.value)),
        SurfaceExpr::List { elements, .. } => elements.iter().any(expr_has_match_arm_body_match),
        SurfaceExpr::Map { entries, .. } => entries
            .iter()
            .any(|entry| expr_has_match_arm_body_match(&entry.value)),
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } | SurfaceExpr::Path { .. } => false,
    }
}

fn module_has_match_arm_body_match(module: &v3_compiler::parse_surface::SurfaceModule) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn { body, .. } => expr_has_match_arm_body_match(body),
        SurfaceItem::Data { body, .. } => body
            .as_ref()
            .is_some_and(|expr| expr_has_match_arm_body_match(expr)),
        _ => false,
    })
}

fn surface_declares_type_sum(
    module: &v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeSum { name, .. } => name == type_name,
        _ => false,
    })
}

// P5 receipt: smoke-test expansion for Finding #2 (CompileLensIntrospect advisory semantics).
// Verifies the .dag shape of lens/application.dag structural claims added in this PR.
// Deferral: retired when .dag-native assertions replace Rust smoke tests.
fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        let SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } = item
        else {
            return false;
        };
        item_path.len() == path.len()
            && item_path
                .iter()
                .zip(path.iter())
                .all(|(a, &b)| a.as_str() == b)
            && names.iter().any(|n| n == name)
    })
}

#[test]
fn v4_lens_application_dag_tokenizes_and_parses() {
    let _ = parse_module(REPORT_DAG, REPORT_PATH);
    let _ = parse_module(APPLICATION_DAG, APPLICATION_PATH);
}

#[test]
fn v4_lens_application_introspect_advisory_claim_tokenizes_and_parses() {
    let module = parse_module(
        INTROSPECT_ADVISORY_CLAIM_DAG,
        INTROSPECT_ADVISORY_CLAIM_PATH,
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "compile"],
            "CompileLensIntrospect"
        ),
        "{INTROSPECT_ADVISORY_CLAIM_PATH}: claim must exercise CompileLensIntrospect"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "witness"], "Violates"),
        "{INTROSPECT_ADVISORY_CLAIM_PATH}: claim must assert advisory Violates witness"
    );
}

#[test]
fn v4_lens_application_introspect_advisory_claim_has_no_match_arm_body_match() {
    let module = parse_module(
        INTROSPECT_ADVISORY_CLAIM_DAG,
        INTROSPECT_ADVISORY_CLAIM_PATH,
    );
    assert!(
        !module_has_match_arm_body_match(&module),
        "{INTROSPECT_ADVISORY_CLAIM_PATH}: claim must keep match-arm bodies helper-shaped for the v2 compile surface"
    );
}

#[test]
fn v4_lens_application_module_authority_and_entrypoints() {
    let module = parse_module(APPLICATION_DAG, APPLICATION_PATH);
    assert_eq!(
        module_path(&module),
        vec!["v4", "lens", "application"],
        "{APPLICATION_PATH}: module path"
    );
    assert!(
        surface_declares_type_sum(&module, "SectionRef"),
        "{APPLICATION_PATH}: SectionRef disjoint sum"
    );
    assert!(
        APPLICATION_DAG.contains("type EnforcedApplication"),
        "{APPLICATION_PATH}: EnforcedApplication carrier"
    );
    assert!(
        APPLICATION_DAG.contains("type IntrospectApplication"),
        "{APPLICATION_PATH}: IntrospectApplication carrier"
    );
    assert!(
        surface_declares_fn(&module, "section_subject"),
        "{APPLICATION_PATH}: section_subject (SectionRef projection hook)"
    );
    assert!(
        surface_declares_fn(&module, "apply_lens"),
        "{APPLICATION_PATH}: apply_lens (introspect)"
    );
    assert!(
        surface_declares_fn(&module, "apply_lens_enforce"),
        "{APPLICATION_PATH}: apply_lens_enforce"
    );
    assert!(
        surface_declares_fn(&module, "subterm_at"),
        "{APPLICATION_PATH}: subterm_at (D1)"
    );
    assert!(
        surface_declares_fn(&module, "apply_diff"),
        "{APPLICATION_PATH}: apply_diff (D1)"
    );
    assert!(
        surface_declares_fn(&module, "apply_advisory_lens"),
        "{APPLICATION_PATH}: apply_advisory_lens (advisory→fail-closed bridge)"
    );
    assert!(
        APPLICATION_DAG.contains("type LensApplicationConfig"),
        "{APPLICATION_PATH}: LensApplicationConfig enforce/introspect mode authority"
    );
}
