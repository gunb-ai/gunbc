use std::collections::HashSet;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_compiler_infer_env::{lookup_binding, lookup_type_by_name};
use v1_compiler::v1_compiler_infer_items::{ResolvedGraph, TypedModule};
use v1_compiler::v1_std_core::{authored_name_at, intern_find, Connective};

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
    let def_binding =
        lookup_binding(def_mod.type_env.clone(), def_id).expect("definer local Shared");
    let use_ty = lookup_type_by_name(use_mod.type_env.clone(), "Shared".to_string())
        .expect("consumer lookup Shared");
    assert!(
        Rc::ptr_eq(&def_binding.resolved, &use_ty),
        "import chain must reach the defining module's resolved node Rc, not a fresh clone"
    );
}

fn collect_binding_ptrs(
    env: &v1_compiler::v1_compiler_infer_env::TypeEnv,
    out: &mut HashSet<*const v1_compiler::v1_compiler_infer_env::TypeBinding>,
) {
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
        def_mod
            .type_env
            .bindings
            .values()
            .any(|b| b.name == "Shared"),
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
        .filter(|m| m.type_env.bindings.values().any(|b| b.name == "Shared"))
        .count();
    assert_eq!(
        modules_with_local_shared, 1,
        "Shared must live in exactly one module's local bindings (flat closure would copy into importer)"
    );
}

#[test]
fn type_env_local_binding_shadows_imported_name() {
    let sources = vec![
        Rc::new(SourceFile {
            path: "definer.dag".to_string(),
            content: "module test.type_env_shadow_definer\ntype Marker = String\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "consumer.dag".to_string(),
            content: "module test.type_env_shadow_consumer\nimport test.type_env_shadow_definer\ntype Marker = Int\nfn pick() -> Marker { 0 }\n".to_string(),
        }),
    ];
    let resolved = compile_modules(sources);
    let graph = resolved.graph.as_ref().expect("graph");
    let definer = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.type_env_shadow_definer",
    );
    let consumer = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.type_env_shadow_consumer",
    );
    let imported = lookup_type_by_name(definer.type_env.clone(), "Marker".to_string())
        .expect("definer Marker");
    let visible = lookup_type_by_name(consumer.type_env.clone(), "Marker".to_string())
        .expect("consumer Marker must resolve to local shadow, not import");
    assert!(
        !Rc::ptr_eq(&imported, &visible),
        "local Marker must shadow imported Marker (distinct resolved nodes)"
    );
    assert_eq!(
        authored_name_at(consumer.type_env.source_indices.clone(), visible.clone()),
        "Int",
        "local Marker = Int shadow must resolve to Int body, not imported String alias"
    );
    assert!(
        consumer
            .type_env
            .bindings
            .values()
            .any(|b| b.name == "Marker"),
        "Marker must remain a local binding on the consumer module"
    );
}

#[test]
fn type_env_import_resolves_via_str_bindings_index() {
    let resolved = compile_modules(rc_identity_fixture_sources());
    let graph = resolved.graph.as_ref().expect("graph");
    let consumer = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.type_env_rc_consumer",
    );
    assert!(
        lookup_type_by_name(consumer.type_env.clone(), "Shared".to_string()).is_some(),
        "sanity: imported Shared must resolve via merged str_bindings index"
    );

    let stripped = Rc::new(v1_compiler::v1_compiler_infer_env::TypeEnv {
        bindings: consumer.type_env.bindings.clone(),
        str_bindings: Rc::new(std::collections::HashMap::new()),
        ancestry_str_bindings: Rc::new(std::collections::HashMap::new()),
        parents: consumer.type_env.parents.clone(),
        recursive_types: consumer.type_env.recursive_types.clone(),
        recursive_type_set: consumer.type_env.recursive_type_set.clone(),
        inductive_fields: consumer.type_env.inductive_fields.clone(),
        source_indices: consumer.type_env.source_indices.clone(),
        intern_table: consumer.type_env.intern_table.clone(),
    });
    assert!(
        lookup_type_by_name(stripped, "Shared".to_string()).is_none(),
        "perturbation: empty str_bindings must break imported type lookup even with parents intact"
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
        str_bindings: consumer.type_env.str_bindings.clone(),
        ancestry_str_bindings: consumer.type_env.ancestry_str_bindings.clone(),
        parents: Rc::new(vec![]),
        recursive_types: consumer.type_env.recursive_types.clone(),
        recursive_type_set: consumer.type_env.recursive_type_set.clone(),
        inductive_fields: consumer.type_env.inductive_fields.clone(),
        source_indices: consumer.type_env.source_indices.clone(),
        intern_table: consumer.type_env.intern_table.clone(),
    });
    assert!(
        lookup_type_by_name(stripped.clone(), "Shared".to_string()).is_some(),
        "parent chain drop must not break lookup when str_bindings index carries ancestry"
    );
    let stripped_index = Rc::new(v1_compiler::v1_compiler_infer_env::TypeEnv {
        bindings: consumer.type_env.bindings.clone(),
        str_bindings: Rc::new(std::collections::HashMap::new()),
        ancestry_str_bindings: Rc::new(std::collections::HashMap::new()),
        parents: Rc::new(vec![]),
        recursive_types: consumer.type_env.recursive_types.clone(),
        recursive_type_set: consumer.type_env.recursive_type_set.clone(),
        inductive_fields: consumer.type_env.inductive_fields.clone(),
        source_indices: consumer.type_env.source_indices.clone(),
        intern_table: consumer.type_env.intern_table.clone(),
    });
    assert!(
        lookup_type_by_name(stripped_index, "Shared".to_string()).is_none(),
        "perturbation: stripping str_bindings must break imported type lookup"
    );
}

fn synthetic_import_chain_sources(depth: usize) -> Vec<Rc<SourceFile>> {
    (0..depth)
        .map(|i| {
            let content = if i == 0 {
                format!("module test.chain_{i}\ntype T{i} = Int\n")
            } else {
                format!(
                    "module test.chain_{i}\nimport test.chain_{}\ntype T{i} = T{}\n",
                    i - 1,
                    i - 1
                )
            };
            Rc::new(SourceFile {
                path: format!("chain_{i}.dag"),
                content,
            })
        })
        .collect()
}

fn time_import_chain_resolve(depth: usize) -> Duration {
    let sources = synthetic_import_chain_sources(depth);
    let mut samples = Vec::new();
    for _ in 0..7 {
        let start = Instant::now();
        compile_modules(sources.clone());
        samples.push(start.elapsed());
    }
    samples.sort();
    samples[samples.len() / 2]
}

#[test]
fn type_env_import_chain_scaling_not_quadratic() {
    let d16 = time_import_chain_resolve(16);
    let d32 = time_import_chain_resolve(32);
    let d64 = time_import_chain_resolve(64);
    let d128 = time_import_chain_resolve(128);

    let ratio_64_32 = d64.as_secs_f64() / d32.as_secs_f64().max(1e-9);
    let ratio_128_64 = d128.as_secs_f64() / d64.as_secs_f64().max(1e-9);

    eprintln!(
        "import-chain scaling: d16={d16:?} d32={d32:?} d64={d64:?} d128={d128:?} ratio_64/32={ratio_64_32:.2} ratio_128/64={ratio_128_64:.2}"
    );

    assert!(
        ratio_128_64 < 3.0,
        "import-chain resolve must stay sub-quadratic: time(128)/time(64)={ratio_128_64:.2} (budget <3.0)"
    );
    assert!(
        ratio_64_32 < 3.0,
        "import-chain resolve must stay sub-quadratic: time(64)/time(32)={ratio_64_32:.2} (budget <3.0)"
    );
}

#[test]
fn type_env_import_chain_flatten_parent_recurses_zero() {
    v1_compiler::v1_compiler_infer_env::reset_flatten_visible_profile();
    compile_modules(synthetic_import_chain_sources(128));
    assert_eq!(
        v1_compiler::v1_compiler_infer_env::flatten_visible_parent_recurses(),
        0,
        "flatten_visible_bindings must use ancestry index, not recursive parent flatten"
    );
}

#[test]
fn type_env_dual_import_later_overlay_wins() {
    use v1_compiler::v1_compiler_infer_env::lookup_binding_by_name;

    let sources = vec![
        Rc::new(SourceFile {
            path: "dual_a.dag".to_string(),
            content: "module test.dual_import_a\ntype Collider = String\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "dual_b.dag".to_string(),
            content: "module test.dual_import_b\ntype Collider = Int\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "consumer.dag".to_string(),
            content: "module test.dual_import_consumer\nimport test.dual_import_a\nimport test.dual_import_b\nfn pick() -> Collider { 0 }\n".to_string(),
        }),
    ];
    let resolved = compile_modules(sources);
    let graph = resolved.graph.as_ref().expect("graph");
    let mod_a = typed_module_by_name(&graph.modules, &resolved.source_indices, "test.dual_import_a");
    let mod_b = typed_module_by_name(&graph.modules, &resolved.source_indices, "test.dual_import_b");
    let consumer = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.dual_import_consumer",
    );
    let cache_a = mod_a
        .type_env_cache
        .str_bindings
        .get("Collider")
        .expect("dual_import_a exports Collider");
    let cache_b = mod_b
        .type_env_cache
        .str_bindings
        .get("Collider")
        .expect("dual_import_b exports Collider");
    let visible_cache = consumer
        .type_env_cache
        .str_bindings
        .get("Collider")
        .expect("consumer visible cache must export Collider");
    assert!(
        Rc::ptr_eq(visible_cache, cache_b),
        "merge_type_env_cache overlay-wins: later import (dual_import_b) must win in type_env_cache.str_bindings"
    );
    assert!(
        !Rc::ptr_eq(visible_cache, cache_a),
        "earlier import (dual_import_a) must not win when same visible name is exported twice"
    );
    let visible_binding = lookup_binding_by_name(consumer.type_env.clone(), "Collider".to_string())
        .expect("consumer Collider binding");
    assert!(
        Rc::ptr_eq(&visible_binding.resolved, &cache_b.resolved),
        "visible Collider must share dual_import_b's resolved Int alias node"
    );
    assert!(
        !Rc::ptr_eq(&visible_binding.resolved, &cache_a.resolved),
        "visible Collider must not share dual_import_a's resolved String alias node"
    );
}

#[test]
fn type_env_std_types_type_variable_filtered_from_import() {
    use v1_compiler::v1_compiler_infer::type_env_for_import;
    use v1_compiler::v1_compiler_infer_env::TypeBinding;
    use v1_compiler::v1_std_core::{empty_intern_table, intern, leaf_node_with_span, make_span};

    fn stub_leaf(name: &str) -> Rc<v1_compiler::v1_std_core::Node> {
        leaf_node_with_span(name.to_string(), make_span(0, 0))
    }

    let intern_table = empty_intern_table();
    let t_binding = Rc::new(TypeBinding {
        name: "T".to_string(),
        resolved: stub_leaf("T"),
        provenance: Rc::new(v1_compiler::v1_std_core::SubValueRelation::SubValueUnknown),
    });
    let int_binding = Rc::new(TypeBinding {
        name: "Int".to_string(),
        resolved: stub_leaf("Int"),
        provenance: Rc::new(v1_compiler::v1_std_core::SubValueRelation::SubValueUnknown),
    });
    let t_id = intern(intern_table.clone(), "T".to_string()).id;
    let int_id = intern(intern_table.clone(), "Int".to_string()).id;
    let parent = Rc::new(v1_compiler::v1_compiler_infer_env::TypeEnv {
        bindings: Rc::new(std::collections::HashMap::from([
            (t_id, t_binding.clone()),
            (int_id, int_binding.clone()),
        ])),
        str_bindings: Rc::new(std::collections::HashMap::from([
            ("T".to_string(), t_binding),
            ("Int".to_string(), int_binding),
        ])),
        ancestry_str_bindings: Rc::new(std::collections::HashMap::new()),
        parents: Rc::new(vec![]),
        recursive_types: Rc::new(vec![]),
        recursive_type_set: Rc::new(std::collections::HashMap::new()),
        inductive_fields: Rc::new(std::collections::HashMap::new()),
        source_indices: Rc::new(std::collections::HashMap::from([(
            "stub.dag".to_string(),
            Rc::new(v1_compiler::v1_std_core::NewlineIndex {
                file: "stub.dag".to_string(),
                offsets: Rc::new(vec![0]),
                char_codes: Rc::new(vec![]),
            }),
        )])),
        intern_table: intern_table.clone(),
    });
    let filtered = type_env_for_import("std.types".to_string(), parent);
    for tv in ["T", "K", "V", "MappedElement", "FoldAccumulator"] {
        assert!(
            !filtered.str_bindings.contains_key(tv),
            "type_env_for_import(std.types) must strip type variable {tv} from str_bindings"
        );
        assert!(
            !filtered.ancestry_str_bindings.contains_key(tv),
            "type_env_for_import(std.types) must strip type variable {tv} from ancestry_str_bindings"
        );
    }
    assert!(
        filtered.str_bindings.contains_key("Int"),
        "non-type-variable kernel bindings must survive std.types import filtering"
    );
}

#[test]
fn type_env_local_variant_shadows_imported_variant_local() {
    let sources = vec![
        Rc::new(SourceFile {
            path: "parent.dag".to_string(),
            content: "module test.variant_shadow_parent\ntype E = Alpha { x: Int } | Beta { y: String }\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "consumer.dag".to_string(),
            content: "module test.variant_shadow_consumer\nimport test.variant_shadow_parent\ntype F = Alpha { x: String } | Gamma { z: Int }\n".to_string(),
        }),
    ];
    let resolved = compile_modules(sources);
    let graph = resolved.graph.as_ref().expect("graph");
    let parent = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.variant_shadow_parent",
    );
    let consumer = typed_module_by_name(
        &graph.modules,
        &resolved.source_indices,
        "test.variant_shadow_consumer",
    );
    let parent_alpha = parent
        .type_env_cache
        .variant_locals
        .get("Alpha")
        .expect("parent Alpha variant local");
    let consumer_alpha = consumer
        .type_env_cache
        .variant_locals
        .get("Alpha")
        .expect("consumer Alpha variant local");
    assert_eq!(
        authored_name_at(parent.type_env.source_indices.clone(), parent_alpha.resolved.clone()),
        "E",
        "parent Alpha must point at imported enum E"
    );
    assert_eq!(
        authored_name_at(
            consumer.type_env.source_indices.clone(),
            consumer_alpha.resolved.clone()
        ),
        "F",
        "consumer Alpha must shadow parent variant with local enum F"
    );
    assert!(
        parent_alpha.resolved.connective == Connective::Disj
            && consumer_alpha.resolved.connective == Connective::Disj,
        "variant locals must reference coproduct carriers"
    );
    assert!(
        !Rc::ptr_eq(&parent_alpha.resolved, &consumer_alpha.resolved),
        "parent variant locals must not replace consumer-local variant bindings"
    );
}
