// Non-hermetic corpus test: reads real `dsl/std/*.dag` files from the source tree.
// Lives in `tests/` (integration test harness) per the testing invariant that
// non-hermetic tests must not reside in `src/` unit test modules.
#![allow(clippy::disallowed_methods)]

use std::collections::HashMap;
use std::path::PathBuf;

use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_typecheck::{typecheck_module_graph_with_options, TypecheckOptions};

fn read_dsl_std(name: &str) -> (PathBuf, String) {
    let path = PathBuf::from(format!(
        "{}/../../../../dsl/std/{name}",
        env!("CARGO_MANIFEST_DIR")
    ));
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    (path, source)
}

fn module_graph_from_files(files: &[&str]) -> ModuleGraph {
    let file_data: Vec<_> = files.iter().map(|name| read_dsl_std(name)).collect();
    let modules: Vec<_> = file_data
        .iter()
        .map(|(path, source)| {
            let ast = daglang_syntax::parser::parse(source)
                .unwrap_or_else(|e| panic!("failed to parse {}: {e:?}", path.display()));
            let module_path = ast
                .module_path
                .as_ref()
                .map(|m| m.node.clone())
                .unwrap_or_else(|| panic!("missing module declaration in {}", path.display()));
            ResolvedModule {
                path: path.clone(),
                ast,
                module_path,
                dependencies: Vec::new(),
                source: source.clone(),
            }
        })
        .collect();
    let module_lookup: HashMap<_, _> = modules
        .iter()
        .enumerate()
        .map(|(i, m)| (m.module_path.as_dotted(), i))
        .collect();
    let mut modules = modules;
    for module in &mut modules {
        module.dependencies = module
            .ast
            .imports
            .iter()
            .filter_map(|imp| module_lookup.get(&imp.node.path.as_dotted()).copied())
            .collect();
    }
    ModuleGraph { modules }
}

/// Strict-typechecks the real `dsl/std/types.dag` and `dsl/std/resources.dag`.
/// Catches regressions where the stdlib drifts out of sync with typecheck
/// (import resolution, type structure, refinements).
#[test]
fn strict_typecheck_dsl_std_types_and_resources() {
    let graph = module_graph_from_files(&["types.dag", "resources.dag"]);

    let typed = typecheck_module_graph_with_options(
        &graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .expect("dsl/std types.dag and resources.dag should strict-typecheck");

    assert_eq!(typed.module_count(), 2);

    let module_names: Vec<_> = typed.modules().map(|m| m.module_path.as_dotted()).collect();
    assert!(
        module_names.contains(&"std.types".to_string()),
        "expected std.types in typechecked modules"
    );
    assert!(
        module_names.contains(&"std.resources".to_string()),
        "expected std.resources in typechecked modules"
    );
}
