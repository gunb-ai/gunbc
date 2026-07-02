use std::collections::{HashMap, HashSet};
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::{
    build_multi_entry_index, resolve_entry_with_index, whole_tree_resolved_ctx, WholeTreeCtx,
    FLOOR_DISCOVERY_EXCLUDES,
};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_compiler_infer::{infer_expr, InferScope};
use v1_compiler::v1_compiler_infer_items::{ResolvedGraph, TypedModule};
use v1_compiler::v1_compiler_infer_lookup::lookup_func_sig;
use v1_compiler::v1_compiler_infer_sigs::{lookup_resolved_sig, ResolvedFuncEnv, ResolvedFuncSig};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};
use v1_compiler::v1_std_core::{authored_name_at, diagnostic_to_message};

use crate::helpers::workspace_root;

static CACHE_ENV_MUTEX: Mutex<()> = Mutex::new(());

struct CacheEnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    prev: Option<std::ffi::OsString>,
}

impl CacheEnvGuard {
    fn set(cache_dir: &std::path::Path) -> Self {
        let lock = CACHE_ENV_MUTEX.lock().expect("cache env mutex");
        let prev = std::env::var_os("GUNBC_RESOLVED_GRAPH_CACHE_DIR");
        std::env::set_var("GUNBC_RESOLVED_GRAPH_CACHE_DIR", cache_dir);
        Self { _lock: lock, prev }
    }
}

impl Drop for CacheEnvGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(v) => std::env::set_var("GUNBC_RESOLVED_GRAPH_CACHE_DIR", v),
            None => std::env::remove_var("GUNBC_RESOLVED_GRAPH_CACHE_DIR"),
        }
    }
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "gunbc-func-env-{label}-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn rc_identity_fixture_sources() -> Vec<Rc<SourceFile>> {
    vec![
        Rc::new(SourceFile {
            path: "definer.dag".to_string(),
            content: "module test.func_env_rc_definer\nfn shared_fn() -> Int { 7 }\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "consumer.dag".to_string(),
            content: "module test.func_env_rc_consumer\nimport test.func_env_rc_definer\nfn call_shared() -> Int { shared_fn() }\n".to_string(),
        }),
    ]
}

fn assert_rc_identity_across_import_chain(
    graph: &ResolvedGraph,
    source_indices: &Rc<
        std::collections::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>,
    >,
) {
    let def_mod = typed_module_by_name(&graph.modules, source_indices, "test.func_env_rc_definer");
    let use_mod = typed_module_by_name(&graph.modules, source_indices, "test.func_env_rc_consumer");
    let def_sig = lookup_resolved_sig(def_mod.func_env.clone(), "shared_fn".to_string())
        .expect("definer local shared_fn");
    let use_sig = lookup_func_sig(use_mod.func_env.clone(), "shared_fn".to_string())
        .expect("consumer lookup shared_fn");
    assert!(
        Rc::ptr_eq(&def_sig, &use_sig),
        "import chain must reach the defining module's Rc, not a fresh clone"
    );
}

fn collect_func_sig_ptrs(env: &ResolvedFuncEnv, out: &mut HashSet<*const ResolvedFuncSig>) {
    for sig in env.local.iter() {
        out.insert(Rc::as_ptr(sig.1));
    }
    for parent in env.parents.iter() {
        collect_func_sig_ptrs(parent, out);
    }
}

fn unique_func_sig_ptr_count_modules(modules: &[Rc<TypedModule>]) -> usize {
    let mut ptrs = HashSet::new();
    for m in modules.iter() {
        collect_func_sig_ptrs(&m.func_env, &mut ptrs);
    }
    ptrs.len()
}

fn sum_local_func_sig_defs_modules(modules: &[Rc<TypedModule>]) -> usize {
    modules.iter().map(|m| m.func_env.local.len()).sum()
}

fn unique_func_sig_ptr_count(graph: &ResolvedGraph) -> usize {
    unique_func_sig_ptr_count_modules(graph.modules.as_ref())
}

fn assert_resolved_no_hard_errors(
    resolved: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved graph, got diagnostics {:?}",
        msgs
    );
}

fn compile_modules(
    sources: Vec<Rc<SourceFile>>,
) -> Rc<v1_compiler::v1_compiler_compile::ResolvedPipelineResult> {
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    resolved
}

fn typed_module_by_name<'a>(
    modules: &'a [Rc<TypedModule>],
    source_indices: &Rc<
        std::collections::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>,
    >,
    name: &str,
) -> &'a Rc<TypedModule> {
    modules
        .iter()
        .find(|m| authored_name_at(source_indices.clone(), m.module.clone()) == name)
        .unwrap_or_else(|| panic!("module {name} not found"))
}

fn shadow_fixture_sources() -> Vec<Rc<SourceFile>> {
    vec![
        Rc::new(SourceFile {
            path: "shadow_first.dag".to_string(),
            content: "module test.func_env_shadow_first\nfn marker() -> Int { 1 }\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "shadow_second.dag".to_string(),
            content: "module test.func_env_shadow_second\nfn marker() -> Int { 2 }\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "shadow_consumer.dag".to_string(),
            content: "module test.func_env_shadow_consumer\nimport test.func_env_shadow_first\nimport test.func_env_shadow_second\nfn use_marker() -> Int { marker() }\n".to_string(),
        }),
    ]
}

#[test]
fn func_env_import_shadowing_last_import_wins() {
    let resolved = compile_modules(shadow_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    match v1_interpreter::run(graph, resolved.source_indices.clone(), "use_marker") {
        Ok(Value::Int(2)) => {}
        other => panic!("last import must shadow first (expected 2): {other:?}"),
    }
}

#[test]
fn func_env_local_shadow_beats_imports() {
    let sources = vec![
        Rc::new(SourceFile {
            path: "shadow_first.dag".to_string(),
            content: "module test.func_env_shadow_first\nfn marker() -> Int { 1 }\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "shadow_second.dag".to_string(),
            content: "module test.func_env_shadow_second\nfn marker() -> Int { 2 }\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "shadow_local_consumer.dag".to_string(),
            content: "module test.func_env_shadow_local_consumer\nimport test.func_env_shadow_first\nimport test.func_env_shadow_second\nfn marker() -> Int { 3 }\nfn use_marker() -> Int { marker() }\n".to_string(),
        }),
    ];
    let resolved = compile_modules(sources);
    let graph = resolved.graph.as_ref().expect("graph");
    match v1_interpreter::run(graph, resolved.source_indices.clone(), "use_marker") {
        Ok(Value::Int(3)) => {}
        other => panic!("local marker must shadow imports (expected 3): {other:?}"),
    }
}

#[test]
fn func_env_rc_identity_shared_across_import_chain() {
    let resolved = compile_modules(rc_identity_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    assert_rc_identity_across_import_chain(graph, &resolved.source_indices);
}

#[test]
fn func_env_rc_identity_holds_on_resolved_graph_cache_hit() {
    let dir = temp_dir("cache-hit");
    let definer = "module test.func_env_rc_definer\nfn shared_fn() -> Int { 7 }\n";
    let consumer = "module test.func_env_rc_consumer\nimport test.func_env_rc_definer\nfn call_shared() -> Int { shared_fn() }\n";
    fs::write(dir.join("definer.dag"), definer).expect("write definer");
    fs::write(dir.join("consumer.dag"), consumer).expect("write consumer");
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");
    let _guard = CacheEnvGuard::set(&cache_dir);
    let roots = vec![dir.to_string_lossy().into_owned()];
    let entry = dir.join("consumer.dag").to_string_lossy().into_owned();

    let cold_index = build_multi_entry_index(&roots);
    let (cold_graph, cold_si) =
        resolve_entry_with_index(&cold_index, &entry).expect("cold resolve");
    assert_rc_identity_across_import_chain(&cold_graph, &cold_si);

    let warm_index = build_multi_entry_index(&roots);
    let (warm_graph, warm_si) =
        resolve_entry_with_index(&warm_index, &entry).expect("warm cache hit");
    assert_rc_identity_across_import_chain(&warm_graph, &warm_si);

    assert_eq!(
        unique_func_sig_ptr_count(&cold_graph),
        unique_func_sig_ptr_count(&warm_graph),
        "cache hit must not re-materialize extra ResolvedFuncSig shells"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn func_env_unique_sig_ptr_count_matches_defined_functions() {
    let resolved = compile_modules(rc_identity_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    let count = unique_func_sig_ptr_count(graph);
    assert_eq!(
        count, 2,
        "fixture defines shared_fn + call_shared; scope-chain must not duplicate shared_fn"
    );
}

#[test]
fn func_env_whole_tree_unique_ptr_count_equals_local_definitions() {
    let mut exclude_subpaths: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .map(|sub| (*sub).to_string())
        .collect();
    exclude_subpaths.extend([
        "test/fixture/".to_string(),
        "/test/".to_string(),
        "nat_semiring_rung".to_string(),
        "lens/application/empty_required_lenses_skip_gate.dag".to_string(),
        "lens/application/rejecting_lens_blocks_before_compile.dag".to_string(),
    ]);
    let roots = vec![
        workspace_root().join("dag").to_string_lossy().into_owned(),
        workspace_root()
            .join("src/v1")
            .to_string_lossy()
            .into_owned(),
    ];
    let WholeTreeCtx {
        ctx,
        modules_resolved,
        ..
    } = whole_tree_resolved_ctx(&roots, &exclude_subpaths, ExecutionMode::Wet)
        .expect("whole-tree resolve");
    let unique = unique_func_sig_ptr_count_modules(ctx.modules.as_ref());
    let defined = sum_local_func_sig_defs_modules(ctx.modules.as_ref());
    assert_eq!(
        unique, defined,
        "scope-chain: {unique} unique ResolvedFuncSig ptrs must equal {defined} local defs \
         across {modules_resolved} modules (flat closure would inflate unique >> defined)"
    );
}

#[test]
fn func_env_dropped_parent_chain_fails_lookup() {
    let resolved = compile_modules(rc_identity_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    let consumer = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.func_env_rc_consumer",
    );
    assert!(
        lookup_func_sig(consumer.func_env.clone(), "shared_fn".to_string()).is_some(),
        "sanity: imported shared_fn must resolve with intact parent chain"
    );

    let stripped = Rc::new(ResolvedFuncEnv {
        local: consumer.func_env.local.clone(),
        parents: Rc::new(vec![]),
    });
    assert!(
        lookup_func_sig(stripped.clone(), "shared_fn".to_string()).is_none(),
        "perturbation: stripping parents from a real import consumer must break \
         imported name lookup (chain-walk is load-bearing, not decorative)"
    );

    let call_shared = consumer
        .items
        .iter()
        .find(|item| {
            authored_name_at(resolved.source_indices.clone(), (*item).clone()) == "call_shared"
        })
        .expect("call_shared item in rc_identity consumer fixture");
    let body = call_shared.body.clone().expect("call_shared body expr");
    let stripped_scope = Rc::new(InferScope {
        type_env: consumer.type_env.clone(),
        func_env: stripped,
        locals: Rc::new(HashMap::new()),
        match_bound_names: Rc::new(HashMap::new()),
        module_name: "test.func_env_rc_consumer".to_string(),
        service_registry: Rc::new(HashMap::new()),
        item_registry: consumer.item_registry.clone(),
        lambda_param_provenance: Rc::new(HashMap::new()),
    });
    let reinfer = infer_expr(body, stripped_scope, None);
    let diag_msgs: Vec<String> = reinfer
        .diagnostics
        .iter()
        .map(|diag| diagnostic_to_message(diag.diagnostic.clone()))
        .collect();
    assert!(
        diag_msgs.iter().any(|msg| {
            msg.contains("shared_fn")
                && (msg.contains("undefined variable") || msg.contains("not found in scope"))
        }),
        "perturbation must surface lookup failure diagnostic on reinfer, got {diag_msgs:?}"
    );
}
