//! Receipt (a) — per-module resolve memo preserves byte-identity vs unmemoized path.

use im_rc::HashMap;
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{front_end_sources, normalize_graph, SourceFile};
use v1_compiler::v1_compiler_infer::{
    build_global_bare_census, build_global_bare_variant_locals, typecheck_module,
};
use v1_compiler::v1_compiler_infer_env::empty_symbol_index;
use v1_compiler::v1_compiler_infer_resolve::{
    per_module_resolve_memo_flush, per_module_resolve_memo_install,
    per_module_resolve_memo_global_snapshot, resolve_node,
    PerModuleResolveMemoEnabledGuard,
};
use v1_compiler::v1_compiler_resolve::ResolvedModule;
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{authored_name_at, InternTable, NewlineIndex, Node};

const DEFINER: &str = r#"module probe.def

type ProbeCurrency =
    ProbeEur
  | ProbeUsd

type MaybeCurrency = Optional<ProbeCurrency>

fn probe_minor_unit(c: ProbeCurrency) -> Int {
  match c {
    ProbeEur => 2
    ProbeUsd => 2
  }
}
"#;

fn src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

fn fixture() -> (
    Rc<ResolvedModule>,
    Rc<HashMap<String, Rc<NewlineIndex>>>,
    Rc<InternTable>,
    Rc<HashMap<String, Rc<v1_compiler::v1_compiler_infer_env::GlobalBareLookupState>>>,
    Rc<HashMap<String, Rc<v1_compiler::v1_compiler_infer_env::TypeBinding>>>,
) {
    let sources = Rc::new(
        vec![src("dag/probe_def.dag", DEFINER)]
            .into_iter()
            .collect::<im_rc::Vector<_>>(),
    );
    let frontend = front_end_sources(sources);
    let graph = frontend.graph.clone().expect("graph");
    let source_indices = frontend.newline_indices.iter().cloned().fold(
        v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
        |acc, si| v1_rt::rc_map_insert(acc, si.file.clone(), si),
    );
    let norm = normalize_graph(graph, source_indices.clone());
    let resolved = norm
        .graph
        .modules
        .iter()
        .next()
        .expect("single module")
        .clone();
    let intern_table = frontend.intern_table.clone();
    let global_bare = build_global_bare_census(norm.graph.modules.clone(), source_indices.clone());
    let global_bare_variant_locals =
        build_global_bare_variant_locals(global_bare.clone(), source_indices.clone());
    (
        resolved,
        source_indices,
        intern_table,
        global_bare,
        global_bare_variant_locals,
    )
}

fn typecheck_fixture() -> Rc<v1_compiler::v1_compiler_infer::TypecheckModuleResult> {
    let (resolved, source_indices, intern_table, global_bare, global_bare_variant_locals) =
        fixture();
    typecheck_module(
        resolved,
        v1_rt::rc_empty_map(),
        v1_rt::rc_empty_map(),
        source_indices,
        intern_table,
        global_bare,
        global_bare_variant_locals,
        empty_symbol_index(),
    )
}

#[test]
fn receipt_a_memo_path_byte_identical_to_unmemoized() {
    let _memo_off = PerModuleResolveMemoEnabledGuard::force(false);
    let cold = typecheck_fixture();
    drop(_memo_off);
    let hot = typecheck_fixture();
    assert_eq!(
        cold, hot,
        "memo path must be byte-identical to unmemoized typecheck"
    );
}

#[test]
fn receipt_a_bounded_memo_serves_repeat_resolve_node() {
    let (resolved, source_indices, intern_table, global_bare, _) = fixture();
    let tc = typecheck_module(
        resolved.clone(),
        v1_rt::rc_empty_map(),
        v1_rt::rc_empty_map(),
        source_indices.clone(),
        intern_table.clone(),
        global_bare.clone(),
        v1_rt::rc_empty_map(),
        empty_symbol_index(),
    );
    let env = tc.typed.type_env.clone();
    let module_name = authored_name_at(source_indices.clone(), resolved.module.clone());
    let probe_type: Rc<Node> = tc
        .typed
        .type_env
        .str_bindings
        .get("ProbeCurrency")
        .expect("ProbeCurrency binding")
        .resolved
        .clone();
    per_module_resolve_memo_install(&env);
    let _ = resolve_node(probe_type.clone(), env.clone(), module_name.clone());
    let _ = resolve_node(probe_type, env, module_name);
    let stats = per_module_resolve_memo_global_snapshot();
    per_module_resolve_memo_flush();
    assert!(
        stats.bounded_hits > 0,
        "repeat resolve_node must hit bounded memo: {:?}",
        stats
    );
}

#[test]
fn receipt_a_composite_resolve_reenters_memo_without_panic() {
    let (resolved, source_indices, intern_table, global_bare, _) = fixture();
    let tc = typecheck_module(
        resolved.clone(),
        v1_rt::rc_empty_map(),
        v1_rt::rc_empty_map(),
        source_indices.clone(),
        intern_table.clone(),
        global_bare.clone(),
        v1_rt::rc_empty_map(),
        empty_symbol_index(),
    );
    let env = tc.typed.type_env.clone();
    let module_name = authored_name_at(source_indices.clone(), resolved.module.clone());
    let maybe_currency: Rc<Node> = tc
        .typed
        .type_env
        .str_bindings
        .get("MaybeCurrency")
        .expect("MaybeCurrency binding")
        .resolved
        .clone();
    per_module_resolve_memo_install(&env);
    let _ = resolve_node(maybe_currency, env, module_name);
    per_module_resolve_memo_flush();
}
