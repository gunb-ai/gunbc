//! **Layer:** integration
//!
//! T-21 wire: `src/v4/lens/edit_locus.dag` — git diff name-only → substrate `Node`
//! edit_locus for `affected_set` via path-regex bindings (`EditLocusRead` injection).
//! Claims: `src/v4/test/claim/lens_affected_set/edit_locus_resolver.dag`.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (peer v4 smoke posture).
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-21 (edit-locus resolver dissolves path-regex in `detect-affected-components.sh`).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const EDIT_LOCUS_DAG: &str = include_str!("../../../../v4/lens/edit_locus.dag");
const EDIT_LOCUS_PATH: &str = "src/v4/lens/edit_locus.dag";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_affected_set/edit_locus_resolver.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/lens_affected_set/edit_locus_resolver.dag";

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

#[test]
fn v4_lens_edit_locus_dag_tokenizes_and_parses() {
    let _ = parse_module(EDIT_LOCUS_DAG, EDIT_LOCUS_PATH);
    let _ = parse_module(CLAIM_DAG, CLAIM_PATH);
}

#[test]
fn v4_lens_edit_locus_module_authority_and_entrypoints() {
    let module = parse_module(EDIT_LOCUS_DAG, EDIT_LOCUS_PATH);
    assert_eq!(
        module_path(&module),
        vec!["v4", "lens", "edit_locus"],
        "{EDIT_LOCUS_PATH}: module path"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "lens", "affected_set"],
            "affected_set_reading"
        ),
        "{EDIT_LOCUS_PATH}: must import affected_set_reading"
    );
    assert!(
        surface_declares_fn(&module, "resolve_edit_locus"),
        "{EDIT_LOCUS_PATH}: resolve_edit_locus"
    );
    assert!(
        surface_declares_fn(&module, "affected_set_reading_from_git_diff"),
        "{EDIT_LOCUS_PATH}: affected_set_reading_from_git_diff"
    );
    assert!(
        EDIT_LOCUS_DAG.contains(") -> Witness<AffectedSetReading>"),
        "{EDIT_LOCUS_PATH}: affected_set_reading_from_git_diff must return Witness (fail-closed boundary)"
    );
}

#[test]
fn v4_lens_edit_locus_claim_wiring() {
    assert!(
        CLAIM_DAG.contains("edit_locus_narrow_resolution_claim_passes")
            && CLAIM_DAG.contains("edit_locus_fail_closed_claim_passes")
            && CLAIM_DAG.contains("edit_locus_affected_set_wire_claim_passes")
            && CLAIM_DAG.contains("edit_locus_affected_set_wire_fail_closed_claim_passes")
            && CLAIM_DAG.contains("affected_set_reading_from_git_diff"),
        "{CLAIM_PATH}: resolver + affected_set wire claims (Witness propagation)"
    );
}
