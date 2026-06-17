//! Ignored harness: localize interface-elaboration cost for languages.dag direct import.
//! Run:
//!   GUNBC_INSTRUMENT_INTERFACE_ELABORATION=1 \
//!   cargo test -p v1-compiler-tests profile_languages_direct_import_elaboration -- --ignored --nocapture

use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use v1_compiler::interface_elaboration_instrument;
use v1_compiler::v1_compiler_infer;
use v1_compiler::v1_compiler_normalize;
use v1_compiler::v1_compiler_resolve;
use v1_compiler::v1_std_core::{authored_name_at, build_newline_index, NewlineIndex};
use v1_compiler::v1_rt;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const LANGUAGES_ENTRY: &str = "src/v2/extdeps/languages/dag.dag";

const LANGUAGES_DIRECT_IMPORTS: &[&str] = &[
    "v2.std.host_transport",
    "v2.std.collection",
    "v2.std.grammar",
    "v2.std.witness",
    "v2.std.compilers.lexing",
    "v2.std.node",
    "v2.std.node_query",
    "v2.std.compilers.target_model",
    "v2.std.logic",
    "v2.std.algebra",
    "v2.std.diagnostic",
    "v2.std.qualified_name",
    "v2.extdeps.languages.fidelity",
];

fn parse_closure(
    entry: &str,
    content: &str,
) -> (
    Vec<Rc<v1_compiler::v1_std_core::Node>>,
    Rc<HashMap<String, Rc<NewlineIndex>>>,
    Rc<v1_compiler::v1_std_core::InternTable>,
) {
    let ws = workspace_root();
    let sources =
        resolve_imports_transitively_with_source_roots(entry, content, &[ws.join("src/v2")]);
    let mut modules = Vec::new();
    let mut intern_table = v1_compiler::v1_std_core::empty_intern_table();
    let mut si_map: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    for source in &sources {
        let tokens =
            v1_compiler::v1_compiler_tokenize::tokenize(source.content.clone(), source.path.clone());
        let nl_index = build_newline_index(source.path.clone(), source.content.clone());
        let single_si = v1_rt::rc_map_insert(
            v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
            nl_index.file.clone(),
            nl_index.clone(),
        );
        let parsed = v1_compiler::v1_compiler_parse::parse_with_table(
            tokens,
            single_si,
            intern_table.clone(),
        );
        intern_table = parsed.intern_table.clone();
        si_map.insert(nl_index.file.clone(), nl_index);
        if let Some(m) = &parsed.result.module {
            modules.push(m.clone());
        }
    }
    (modules, Rc::new(si_map), intern_table)
}

fn typecheck_closure(
    modules: Vec<Rc<v1_compiler::v1_std_core::Node>>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    intern_table: Rc<v1_compiler::v1_std_core::InternTable>,
) {
    let graph = v1_compiler_resolve::resolve_modules(Rc::new(modules), source_indices.clone());
    let norm = v1_compiler_normalize::normalize_graph(graph, source_indices.clone());
    let mut module_index = v1_rt::rc_empty_map();
    for resolved in norm.graph.modules.iter() {
        let tc = v1_compiler_infer::typecheck_module(
            resolved.clone(),
            module_index.clone(),
            source_indices.clone(),
            intern_table.clone(),
        );
        let mod_name = authored_name_at(source_indices.clone(), resolved.module.clone());
        module_index = v1_rt::rc_map_insert(module_index, mod_name, tc.typed.clone());
    }
}

#[test]
#[ignore]
fn profile_languages_direct_import_elaboration() {
    interface_elaboration_instrument::reset();
    let content = std::fs::read_to_string(workspace_root().join(LANGUAGES_ENTRY)).expect("read");
    let t0 = Instant::now();
    let (modules, si, intern) = parse_closure(LANGUAGES_ENTRY, &content);
    eprintln!("languages closure: {} modules", modules.len());
    typecheck_closure(modules, si, intern);
    eprintln!("languages direct-import probe total: {:?}", t0.elapsed());
    interface_elaboration_instrument::eprint_report("languages.dag direct import");
}

#[test]
#[ignore]
fn profile_languages_direct_import_bisect() {
    interface_elaboration_instrument::reset();
    let mut rows: Vec<(&str, std::time::Duration)> = Vec::new();

    for import_path in LANGUAGES_DIRECT_IMPORTS {
        let stub = format!(
            "module v2.compiler.manual.languages_import_probe\nimport {import_path} {{}}\n"
        );
        let path = format!(
            "src/v2/compiler/manual/languages_import_probe_{}.dag",
            import_path.replace('.', "_")
        );
        let t0 = Instant::now();
        let (modules, si, intern) = parse_closure(&path, &stub);
        let module_count = modules.len();
        typecheck_closure(modules, si, intern);
        let elapsed = t0.elapsed();
        eprintln!("probe {import_path}: {elapsed:?} ({module_count} modules)");
        rows.push((import_path, elapsed));
    }

    rows.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!("\n=== languages.dag direct-import bisect (slowest first) ===");
    for (import_path, elapsed) in &rows {
        eprintln!("  {import_path}: {elapsed:?}");
    }
    interface_elaboration_instrument::eprint_report("per-direct-import probes");
}
