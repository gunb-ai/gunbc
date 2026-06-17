//! Ignored perf harness: per-module typecheck breakdown for body_producer closure.
//! Run: cargo test -p v1-tests profile_body_producer_per_module_typecheck -- --ignored --nocapture

use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use v1_compiler::v1_compiler_compile::SourceFile;
use v1_compiler::v1_compiler_infer;
use v1_compiler::v1_compiler_normalize;
use v1_compiler::v1_compiler_resolve;
use v1_compiler::v1_std_core::{authored_name_at, build_newline_index, NewlineIndex};
use v1_compiler::v1_rt;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const ENTRY: &str = "src/v2/compiler/03_body_producer.dag";

#[test]
#[ignore]
fn profile_body_producer_per_module_typecheck() {
    let ws = workspace_root();
    let entry_content = std::fs::read_to_string(ws.join(ENTRY)).expect("read entry");
    let sources = resolve_imports_transitively_with_source_roots(
        ENTRY,
        &entry_content,
        &[ws.join("src/v2")],
    );
    eprintln!("\n=== body_producer closure: {} modules ===", sources.len());

    let mut modules = Vec::new();
    let mut intern_table = v1_compiler::v1_std_core::empty_intern_table();
    let mut si_map: HashMap<String, Rc<NewlineIndex>> = HashMap::new();

    let t_parse = Instant::now();
    for source in &sources {
        let tokens =
            v1_compiler::v1_compiler_tokenize::tokenize(source.content.clone(), source.path.clone());
        let nl_index = build_newline_index(source.path.clone(), source.content.clone());
        let single_si = v1_rt::rc_map_insert(
            v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
            nl_index.file.clone(),
            nl_index.clone(),
        );
        let parsed =
            v1_compiler::v1_compiler_parse::parse_with_table(tokens, single_si, intern_table.clone());
        intern_table = parsed.intern_table.clone();
        si_map.insert(nl_index.file.clone(), nl_index);
        let m = parsed
            .result
            .module
            .clone()
            .unwrap_or_else(|| panic!("parse failed: {}", source.path));
        modules.push(m);
    }
    eprintln!("parse all: {:?}", t_parse.elapsed());

    let source_indices = Rc::new(si_map);
    let t_resolve = Instant::now();
    let graph = v1_compiler_resolve::resolve_modules(Rc::new(modules), source_indices.clone());
    eprintln!("resolve_modules: {:?}", t_resolve.elapsed());

    let t_norm = Instant::now();
    let norm = v1_compiler_normalize::normalize_graph(graph, source_indices.clone());
    eprintln!("normalize_graph: {:?}", t_norm.elapsed());

    let mut module_index = v1_rt::rc_empty_map();
    let t_tc = Instant::now();
    for resolved in norm.graph.modules.iter() {
        let mod_name = authored_name_at(source_indices.clone(), resolved.module.clone());
        let t0 = Instant::now();
        let _tc = v1_compiler_infer::typecheck_module(
            resolved.clone(),
            module_index.clone(),
            source_indices.clone(),
            intern_table.clone(),
        );
        eprintln!("  typecheck {mod_name}: {:?}", t0.elapsed());
        module_index = v1_rt::rc_map_insert(module_index, mod_name, _tc.typed.clone());
    }
    eprintln!("typecheck all: {:?}", t_tc.elapsed());
}
