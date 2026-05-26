//! **Layer:** integration
//!
//! **Wave-5-A / P3 commitment 6:** `00_compile.dag` public terminal `validate_then_compile` +
//! `Validated<CompileOutput>` + `07_target_carriers.dag` acyclic target carriers.
//!
//! **TESTING.md:** M1(2.7) `.dag` brace-bodied `fn` items surface as `FnExternalBody` (no
//! expression AST), so call-site contracts are checked via **import rows** and **declared `fn`
//! inventory** — not raw `str::contains` probes. Semantic substantiation deferred to T-22/T-14.
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** Wave-5-A (`src/v4/compiler/00_compile.dag`).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS §SG-0 hand-authored integration test receipts row
//! land in the same PR.
//!
//! **Dissolution:** remove when `validate_then_compile` / compile-core surfaces are exercised
//! only by `.dag` `TestClaim` rows / a generated harness without this per-file Rust probe (or when
//! `compile_to_dag` over v4 compiler modules resolves imports without substrate collision).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const COMPILE_DAG: &str = include_str!("../../../../v4/compiler/00_compile.dag");
const COMPILE_PATH: &str = "src/v4/compiler/00_compile.dag";
const EMIT_DAG: &str = include_str!("../../../../v4/compiler/05_emit.dag");
const EMIT_PATH: &str = "src/v4/compiler/05_emit.dag";
const TARGET_CARRIERS_DAG: &str = include_str!("../../../../v4/compiler/07_target_carriers.dag");
const TARGET_CARRIERS_PATH: &str = "src/v4/compiler/07_target_carriers.dag";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/validate_then_compile_public_terminal.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/manual/validate_then_compile_public_terminal.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

#[test]
fn v4_compile_dag_tokenizes_and_parses() {
    let _module = parse_module(COMPILE_DAG, COMPILE_PATH);
}

#[test]
fn v4_compile_dag_module_path_is_compiler_compile() {
    let module = parse_module(COMPILE_DAG, COMPILE_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "compiler", "compile"]],
        "{COMPILE_PATH}: module authority path"
    );
}

#[test]
fn v4_compile_dag_declares_public_validate_then_compile() {
    let module = parse_module(COMPILE_DAG, COMPILE_PATH);
    assert!(
        surface_declares_fn(&module, "validate_then_compile"),
        "{COMPILE_PATH}: must declare validate_then_compile public terminal"
    );
}

#[test]
fn v4_compile_dag_declares_ratified_compile_core() {
    let module = parse_module(COMPILE_DAG, COMPILE_PATH);
    assert!(
        surface_declares_fn(&module, "compile"),
        "{COMPILE_PATH}: must declare internal compile-core entry"
    );
    assert!(
        surface_declares_fn(&module, "compile_inferred"),
        "{COMPILE_PATH}: must declare compile_inferred (post-infer compile-core)"
    );
    assert!(
        surface_declares_fn(&module, "apply_lens"),
        "{COMPILE_PATH}: must declare apply_lens gate combinator (T-23 forward home)"
    );
    assert!(
        surface_declares_type(&module, "Validated"),
        "{COMPILE_PATH}: must declare Validated carrier"
    );
}

#[test]
fn v4_compile_dag_imports_target_carriers_not_emit_cycle() {
    let module = parse_module(COMPILE_DAG, COMPILE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "target_carriers"],
            "TargetModel"
        ),
        "{COMPILE_PATH}: TargetModel authority must be target_carriers (acyclic with emit)"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "target_carriers"],
            "TargetSource"
        ),
        "{COMPILE_PATH}: TargetSource authority must be target_carriers (acyclic with emit)"
    );
}

#[test]
fn v4_compile_dag_does_not_import_specific_lens_modules() {
    let module = parse_module(COMPILE_DAG, COMPILE_PATH);
    // v4.lens.fact_density is the always-required hollow-alias gate (T-30); it is exempted.
    // All other domain lens modules remain blocked (P3 commitment 4).
    let lens_imports: Vec<_> = import_paths(&module)
        .into_iter()
        .filter(|path| path.first().copied() == Some("v4") && path.get(1).copied() == Some("lens"))
        .filter(|path| path.get(2).copied() != Some("fact_density"))
        .collect();
    assert!(
        lens_imports.is_empty(),
        "{COMPILE_PATH}: P3 commitment 4 — compile-core must not import v4.lens.* except fact_density ({lens_imports:?})"
    );
}

#[test]
fn v4_emit_dag_does_not_import_compile_module() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        !import_paths(&module)
            .iter()
            .any(|path| path.as_slice() == ["v4", "compiler", "compile"]),
        "{EMIT_PATH}: emit must not import compile (breaks compile→emit cycle)"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "target_carriers"],
            "TargetModel"
        ),
        "{EMIT_PATH}: emit must import TargetModel from target_carriers"
    );
}

#[test]
fn v4_target_carriers_dag_tokenizes_and_parses() {
    let _module = parse_module(TARGET_CARRIERS_DAG, TARGET_CARRIERS_PATH);
}

#[test]
fn v4_validate_then_compile_claim_tokenizes_and_parses() {
    let _module = parse_module(CLAIM_DAG, CLAIM_PATH);
}

#[test]
fn v4_validate_then_compile_claim_imports_public_terminal_helpers() {
    let module = parse_module(CLAIM_DAG, CLAIM_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "compile"],
            "validate_then_compile"
        ),
        "{CLAIM_PATH}: claim must import validate_then_compile public terminal"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "compile"],
            "compile_lens_gate_rejected_diagnostic"
        ),
        "{CLAIM_PATH}: claim must import compile_lens_gate_rejected_diagnostic"
    );
    assert!(
        import_includes_name(&module, &["v4", "compiler", "compile"], "TranslateTo"),
        "{CLAIM_PATH}: claim must import TranslateTo constructor"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "compile"],
            "compile_lens_gate_rejected"
        ),
        "{CLAIM_PATH}: claim must import compile_lens_gate_rejected reason symbol"
    );
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

fn import_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Import { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
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

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeSum {
            name: item_name, ..
        }
        | SurfaceItem::TypeRecord {
            name: item_name, ..
        }
        | SurfaceItem::TypeAlias {
            name: item_name, ..
        }
        | SurfaceItem::TypeAtom {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}
