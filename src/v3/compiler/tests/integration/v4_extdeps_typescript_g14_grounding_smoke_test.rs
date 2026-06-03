//! **Layer:** integration
//!
//! G.1.4 receipt: `typescript.dag` populates `PerLanguageFactBundleRegistry` for wave-2b
//! primitives (number, bigint, boolean, string) via fail-closed
//! `insert_per_language_fact_bundle_entry`. Parse-surface smoke only; full-tree M1 emit
//! exercises cross-module resolution (same posture as `v4_std_grounding_dag_smoke_test`).
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:62); dissolves when `.dag` TestClaim
//! coverage asserts G.1.4 registry rows directly.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const TYPESCRIPT_LANGUAGE_DAG: &str =
    include_str!("../../../../v4/extdeps/languages/typescript.dag");
const TYPESCRIPT_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/typescript.dag";

fn typescript_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(TYPESCRIPT_LANGUAGE_DAG, TYPESCRIPT_LANGUAGE_PATH)
        .unwrap_or_else(|e| panic!("{TYPESCRIPT_LANGUAGE_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, TYPESCRIPT_LANGUAGE_PATH)
        .unwrap_or_else(|e| panic!("{TYPESCRIPT_LANGUAGE_PATH}: parse: {e:?}"))
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: decl_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: decl_name, ..
        } => decl_name == name,
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

#[test]
fn v4_typescript_g14_grounding_registry_parses() {
    let module = typescript_surface_or_panic();
    assert_eq!(
        module
            .items
            .iter()
            .find_map(|item| match item {
                SurfaceItem::Module { path, .. } => {
                    Some(path.iter().map(String::as_str).collect::<Vec<_>>())
                }
                _ => None,
            })
            .expect("typescript.dag module path"),
        vec!["v4", "extdeps", "languages", "typescript"]
    );
}

#[test]
fn v4_typescript_g14_declares_per_language_fact_bundle_registry() {
    assert!(
        TYPESCRIPT_LANGUAGE_DAG.contains("import v4.std.grounding"),
        "G.1.4 must import v4.std.grounding (PerLanguageFactBundleRegistry authority)"
    );
    let module = typescript_surface_or_panic();
    assert!(
        surface_declares_fn(&module, "ts_build_per_language_fact_bundle_registry")
            || TYPESCRIPT_LANGUAGE_DAG.contains("fn ts_build_per_language_fact_bundle_registry("),
        "typescript.dag must export ts_build_per_language_fact_bundle_registry()"
    );
    assert!(
        surface_declares_data(&module, "ts_per_language_fact_bundle_registry_outcome")
            || TYPESCRIPT_LANGUAGE_DAG
                .contains("data ts_per_language_fact_bundle_registry_outcome:"),
        "typescript.dag must export ts_per_language_fact_bundle_registry_outcome data"
    );
    assert!(
        TYPESCRIPT_LANGUAGE_DAG.contains("insert_per_language_fact_bundle_entry("),
        "registry must use fail-closed insert_per_language_fact_bundle_entry"
    );
    assert!(
        TYPESCRIPT_LANGUAGE_DAG.contains("ModelCoreFactAxisSurfaceSpelling")
            && TYPESCRIPT_LANGUAGE_DAG.contains("ModelCoreFactAxisWidth")
            && TYPESCRIPT_LANGUAGE_DAG.contains("ModelCoreFactAxisEncoding"),
        "wave-2b registry must key ModelCorePrimitiveFactAxis coproduct arms"
    );
}
