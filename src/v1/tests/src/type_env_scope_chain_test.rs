use std::collections::HashSet;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_compiler_infer_env::{lookup_binding, lookup_type_by_name};
use v1_compiler::v1_compiler_infer_items::{ResolvedGraph, TypedModule};
use v1_compiler::v1_std_core::{authored_name_at, intern_find};

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
        "gunbc-type-env-{label}-{}-{}",
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
            content: "module test.type_env_rc_definer\ntype Shared = Int\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "consumer.dag".to_string(),
            content: "module test.type_env_rc_consumer\nimport test.type_env_rc_definer\nfn use_shared() -> Shared { 7 }\n".to_string(),
        }),
    ]
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

fn assert_rc_identity_across_import_chain(
    graph: &ResolvedGraph,
    source_indices: &Rc<
        std::collections::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>,
    >,
) {
    let def_mod = typed_module_by_name(&graph.modules, source_indices, "test.type_env_rc_definer");
    let use_mod = typed_module_by_name(&graph.modules, source_indices, "test.type_env_rc_consumer");
    let def_id = intern_find(def_mod.type_env.intern_table.clone(), "Shared".to_string())
        .expect("definer intern Shared");
    let def_binding = lookup_binding(def_mod.type_env.clone(), def_id)
        .expect("definer local Shared");
    let use_ty = lookup_type_by_name(use_mod.type_env.clone(), "Shared".to_string())
        .expect("consumer lookup Shared");
    assert!(
        Rc::ptr_eq(&def_binding.resolved, &use_ty),
        "import chain must reach the defining module's resolved node Rc, not a fresh clone"
    );
}

fn collect_binding_ptrs(env: &v1_compiler::v1_compiler_infer_env::TypeEnv, out: &mut HashSet<*const v1_compiler::v1_compiler_infer_env::TypeBinding>) {
    for binding in env.bindings.values() {
        out.insert(Rc::as_ptr(binding));
    }
    for parent in env.parents.iter() {
        collect_binding_ptrs(parent, out);
    }
}

fn unique_binding_ptr_count_modules(modules: &[Rc<TypedModule>]) -> usize {
    let mut ptrs = HashSet::new();
    for m in modules.iter() {
        collect_binding_ptrs(&m.type_env, &mut ptrs);
    }
    ptrs.len()
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

#[test]
fn type_env_rc_identity_shared_across_import_chain() {
    let resolved = compile_modules(rc_identity_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    assert_rc_identity_across_import_chain(graph, &resolved.source_indices);
}

#[test]
fn type_env_rc_identity_holds_on_resolved_graph_cache_hit() {
    let dir = temp_dir("cache-hit");
    let definer = "module test.type_env_rc_definer\ntype Shared = Int\n";
    let consumer = "module test.type_env_rc_consumer\nimport test.type_env_rc_definer\nfn use_shared() -> Shared { 7 }\n";
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
        unique_binding_ptr_count_modules(cold_graph.modules.as_ref()),
        unique_binding_ptr_count_modules(warm_graph.modules.as_ref()),
        "cache hit must not re-materialize extra TypeBinding shells"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn type_env_import_does_not_materialize_binding_in_consumer_locals() {
    let resolved = compile_modules(rc_identity_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    let def_mod = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.type_env_rc_definer",
    );
    let consumer = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.type_env_rc_consumer",
    );
    assert!(
        def_mod.type_env.bindings.values().any(|b| b.name == "Shared"),
        "definer must carry Shared in local bindings"
    );
    assert!(
        !consumer
            .type_env
            .bindings
            .values()
            .any(|b| b.name == "Shared"),
        "consumer must reach Shared via parent chain, not a copied local binding"
    );
}

#[test]
fn type_env_shared_type_single_local_authority() {
    let resolved = compile_modules(rc_identity_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    let modules_with_local_shared = graph
        .modules
        .iter()
        .filter(|m| {
            m.type_env
                .bindings
                .values()
                .any(|b| b.name == "Shared")
        })
        .count();
    assert_eq!(
        modules_with_local_shared, 1,
        "Shared must live in exactly one module's local bindings (flat closure would copy into importer)"
    );
}

#[test]
fn type_env_dropped_parent_chain_fails_lookup() {
    let resolved = compile_modules(rc_identity_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    let consumer = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.type_env_rc_consumer",
    );
    assert!(
        lookup_type_by_name(consumer.type_env.clone(), "Shared".to_string()).is_some(),
        "sanity: imported Shared must resolve with intact parent chain"
    );

    let stripped = Rc::new(v1_compiler::v1_compiler_infer_env::TypeEnv {
        bindings: consumer.type_env.bindings.clone(),
        parents: Rc::new(vec![]),
        recursive_types: consumer.type_env.recursive_types.clone(),
        recursive_type_set: consumer.type_env.recursive_type_set.clone(),
        inductive_fields: consumer.type_env.inductive_fields.clone(),
        source_indices: consumer.type_env.source_indices.clone(),
        intern_table: consumer.type_env.intern_table.clone(),
    });
    assert!(
        lookup_type_by_name(stripped, "Shared".to_string()).is_none(),
        "perturbation: stripping parents must break imported type lookup"
    );
}
