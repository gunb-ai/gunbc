//! **Layer:** integration
//!
//! **Wave-5-A:** `00_compile.dag` public terminal `validate_then_compile` + ratified `compile` surface.
//!
//! **TESTING.md:** M1(2.7) parse/tokenize receipt until T-22 evaluates TestClaim rows.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const COMPILE_DAG: &str = include_str!("../../../../v4/compiler/00_compile.dag");
const COMPILE_PATH: &str = "src/v4/compiler/00_compile.dag";
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
        surface_declares_fn(&module, "apply_lens"),
        "{COMPILE_PATH}: must declare apply_lens gate combinator (T-23 forward home)"
    );
    assert!(
        surface_declares_type(&module, "Validated"),
        "{COMPILE_PATH}: must declare Validated carrier"
    );
}

#[test]
fn v4_compile_dag_does_not_import_specific_lens_modules() {
    let module = parse_module(COMPILE_DAG, COMPILE_PATH);
    let lens_imports: Vec<_> = import_paths(&module)
        .into_iter()
        .filter(|path| path.first().copied() == Some("v4") && path.get(1).copied() == Some("lens"))
        .collect();
    assert!(
        lens_imports.is_empty(),
        "{COMPILE_PATH}: P3 commitment 4 — compile-core must not import v4.lens.* ({lens_imports:?})"
    );
}

#[test]
fn v4_validate_then_compile_claim_tokenizes_and_parses() {
    let _module = parse_module(CLAIM_DAG, CLAIM_PATH);
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
        SurfaceItem::Type { name: item_name, .. } => item_name == name,
        _ => false,
    })
}
