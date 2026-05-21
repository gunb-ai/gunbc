//! **Layer:** integration
//!
//! **P2 / Practice 5 (single authority):** This harness proves **parse cleanliness**
//! for `src/v4/lens/registry.dag` — it does **not** claim a **generated** substrate consumer
//! for `LensRegistryEntryV0` / `LensIdV0` / `LensModulePathV0` (INVARIANTS §P2: declaration
//! without generated consumer = staging; see `STRUCTURE.md` and
//! `docs/briefs/r4-lane-a-lens-interface-freeze-pin.md` §3). The operator pin’s §3 markdown
//! table remains a human mirror until a mechanical reader lands.
//!
//! **Note:** After P9 `#3503`, `registry.dag` imports `Symbol` and `List` from `v4.std.*` and
//! carries self-referential `Symbol` data rows. Isolated `compile_to_dag` does not resolve those
//! imports under M1(2.7) (same posture as `v4_bin_main_dag_smoke_test` / `v4_lens_testgen_dag_smoke_test`).
//!
//! **INVARIANTS §P5 Dispatch-Discipline Mechanism (b):** this path’s SG-0 census line + matching
//! `INVARIANTS.md` table row land in the same PR as the harness (home-of-record for the
//! hand-Rust receipt).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const REGISTRY_DAG: &str = include_str!("../../../../v4/lens/registry.dag");
const REGISTRY_PATH: &str = "src/v4/lens/registry.dag";

#[test]
fn v4_lens_registry_dag_tokenizes_and_parses() {
    let tokens = tokenize_for_test(REGISTRY_DAG, REGISTRY_PATH)
        .unwrap_or_else(|e| panic!("{REGISTRY_PATH}: tokenize: {e:?}"));
    let module = parse_for_test(&tokens, REGISTRY_PATH)
        .unwrap_or_else(|e| panic!("{REGISTRY_PATH}: parse: {e:?}"));
    assert_eq!(
        module_path(&module),
        vec!["v4", "lens", "registry"],
        "{REGISTRY_PATH}: module path"
    );
    assert!(
        module_declares_type_sum_named(&module, "LensIdV0"),
        "{REGISTRY_PATH}: must declare LensIdV0 closed sum"
    );
    assert!(
        module_declares_type_sum_named(&module, "LensModulePathV0"),
        "{REGISTRY_PATH}: must declare LensModulePathV0"
    );
}

#[test]
fn v4_lens_registry_t23_closed_lens_ids_present() {
    for id in [
        "Complexity",
        "Cost",
        "Parallelism",
        "EffectEnumeration",
        "Idempotency",
        "Provenance",
        "UnusedParameters",
        "StructuralResolution",
    ] {
        assert!(
            REGISTRY_DAG.contains(id),
            "{REGISTRY_PATH}: LensIdV0 arm `{id}` must appear in closed registry source"
        );
    }
}

#[test]
fn v4_lens_registry_structural_resolution_bound_to_module() {
    assert!(
        REGISTRY_DAG.contains("lens_id: StructuralResolution")
            && REGISTRY_DAG.contains(r#"module_path: Bound { path: "v4.lens.structural_resolution" }"#),
        "{REGISTRY_PATH}: StructuralResolution must be Bound to v4.lens.structural_resolution (T-13 registry fill)"
    );
}

#[test]
fn v4_lens_registry_p9_owned_fn_surface_present() {
    assert!(
        REGISTRY_DAG.contains("type LensOwnedFnV0")
            && REGISTRY_DAG.contains("data lens_owned_fn_registry_v0")
            && REGISTRY_DAG.contains("data symbol_llvm_instruction_cost")
            && REGISTRY_DAG.contains("owner_module_path: \"v4.lens.cost\""),
        "{REGISTRY_PATH}: P9 single-owner registry rows must remain in substrate home-of-record"
    );
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

fn module_declares_type_sum_named(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum {
                name: item_name, ..
            } if item_name == name
        )
    })
}
