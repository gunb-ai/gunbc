use std::collections::HashSet;
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_compiler_infer_items::TypedModule;
use v1_compiler::v1_compiler_infer_lookup::lookup_func_sig;
use v1_compiler::v1_compiler_infer_sigs::{lookup_resolved_sig, ResolvedFuncEnv, ResolvedFuncSig};
use v1_compiler::v1_interpreter::{self, Value};
use v1_compiler::v1_std_core::authored_name_at;

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

fn compile_modules(sources: Vec<Rc<SourceFile>>) -> Rc<v1_compiler::v1_compiler_compile::ResolvedPipelineResult> {
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    resolved
}

fn typed_module_by_name<'a>(
    modules: &'a [Rc<TypedModule>],
    source_indices: &Rc<std::collections::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>>,
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
    let sources = vec![
        Rc::new(SourceFile {
            path: "definer.dag".to_string(),
            content: "module test.func_env_rc_definer\nfn shared_fn() -> Int { 7 }\n".to_string(),
        }),
        Rc::new(SourceFile {
            path: "consumer.dag".to_string(),
            content: "module test.func_env_rc_consumer\nimport test.func_env_rc_definer\nfn call_shared() -> Int { shared_fn() }\n".to_string(),
        }),
    ];
    let resolved = compile_modules(sources);
    let graph = resolved.graph.as_ref().expect("graph");
    let def_mod = typed_module_by_name(&graph.modules, &resolved.source_indices, "test.func_env_rc_definer");
    let use_mod = typed_module_by_name(&graph.modules, &resolved.source_indices, "test.func_env_rc_consumer");
    let def_sig = lookup_resolved_sig(def_mod.func_env.clone(), "shared_fn".to_string())
        .expect("definer local shared_fn");
    let use_sig = lookup_func_sig(use_mod.func_env.clone(), "shared_fn".to_string())
        .expect("consumer lookup shared_fn");
    assert!(
        Rc::ptr_eq(&def_sig, &use_sig),
        "import chain must reach the defining module's Rc, not a fresh clone"
    );
}

#[test]
fn func_env_dropped_parent_chain_fails_lookup() {
    let env = Rc::new(ResolvedFuncEnv {
        local: Rc::new(std::collections::HashMap::new()),
        parents: Rc::new(vec![]),
    });
    assert!(
        lookup_func_sig(env, "missing_fn".to_string()).is_none(),
        "empty parent chain must not resolve imported names"
    );
}

#[allow(dead_code)]
fn collect_func_sig_ptrs(env: &ResolvedFuncEnv, out: &mut HashSet<*const ResolvedFuncSig>) {
    for sig in env.local.iter() {
        out.insert(Rc::as_ptr(sig.1));
    }
    for parent in env.parents.iter() {
        collect_func_sig_ptrs(parent, out);
    }
}
