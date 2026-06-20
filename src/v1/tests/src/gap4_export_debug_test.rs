//! Temporary: debug v2.compiler.parse exports for gap-4 lane.
use std::collections::HashMap;
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_compiler_resolve::get_exported_names;
use v1_compiler::v1_std_core::build_newline_index;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

#[test]
fn gap4_v2_parse_module_export_debug() {
    let roots = [workspace_root().join("src/v2"), workspace_root().join("dsl")];
    let path = "src/v2/compiler/02_parse.dag";
    let content = std::fs::read_to_string(workspace_root().join(path)).unwrap();
    let sources = resolve_imports_transitively_with_source_roots(path, &content, &roots);
    let files: Vec<Rc<SourceFile>> = sources
        .iter()
        .map(|s| {
            Rc::new(SourceFile {
                path: s.path.clone(),
                content: s.content.clone(),
            })
        })
        .collect();
    let result = compile_to_resolved(Rc::new(files));
    let blocking: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    eprintln!("blocking diagnostics: {blocking:?}");
    let si: HashMap<String, Rc<_>> = result
        .newline_indices
        .iter()
        .map(|idx| (idx.file.clone(), build_newline_index(idx.file.clone(), "".to_string())))
        .collect();
    // rebuild proper si from sources
    let mut si2 = HashMap::new();
    for s in &sources {
        si2.insert(
            s.path.clone(),
            build_newline_index(s.path.clone(), s.content.clone()),
        );
    }
    let si = Rc::new(si2);
    if let Some(graph) = &result.graph {
        for m in graph.modules.iter() {
            let name = v1_compiler::v1_compiler_emit_rust::authored_name_at(
                si.clone(),
                m.module.clone(),
            );
            if name == "v2.compiler.parse" {
                let exports = get_exported_names(m.module.clone(), si.clone());
                eprintln!("export count: {}", exports.len());
                eprintln!("has parse_module: {}", exports.iter().any(|n| n == "parse_module"));
                let parse_exports: Vec<_> = exports
                    .iter()
                    .filter(|n| n.contains("parse"))
                    .collect();
                eprintln!("parse* exports ({})", parse_exports.len());
                for n in parse_exports.iter().rev().take(15) {
                    eprintln!("  {n}");
                }
            }
        }
    }
}
