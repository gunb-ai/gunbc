//! **Layer:** integration
//!
//! T-23 wire: `src/v4/lens/application.dag` — lens application surface carriers,
//! `apply_lens`, D1 `subterm_at` / `apply_diff`, advisory→fail-closed bridge.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (peer v4 smoke posture).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const APPLICATION_DAG: &str = include_str!("../../../../v4/lens/application.dag");
const APPLICATION_PATH: &str = "src/v4/lens/application.dag";
const REPORT_DAG: &str = include_str!("../../../../v4/std/report.dag");
const REPORT_PATH: &str = "src/v4/std/report.dag";

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

fn surface_declares_type_sum(
    module: &v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeSum { name, .. } => name == type_name,
        _ => false,
    })
}

#[test]
fn v4_lens_application_dag_tokenizes_and_parses() {
    let _ = parse_module(REPORT_DAG, REPORT_PATH);
    let _ = parse_module(APPLICATION_DAG, APPLICATION_PATH);
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
