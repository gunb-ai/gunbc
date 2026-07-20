//! Receipt 1 — byte-identity of `env_variant_locals` vs old `merge_global_bare_variant_locals`.
//!
//! Discriminating case: owning module, same-authority variant (globally-unique Disj arm
//! present in BOTH module `variant_fold.locals` and shared `global_bare_variant_locals`).

use im::HashMap;
use std::sync::Arc;

use v1_compiler::v1_compiler_compile::{front_end_sources, normalize_graph, SourceFile};
use v1_compiler::v1_compiler_infer::{
    build_global_bare_census, build_global_bare_variant_locals, build_local_variants,
    build_type_env, insert_variant_owner_checked, typecheck_module, VariantFoldState,
};
use v1_compiler::v1_compiler_infer_env::{GlobalBareLookupState, TypeBinding};
use v1_compiler::v1_compiler_resolve::ResolvedModule;
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{
    authored_name_at, has_child_named, Connective, InternTable, NewlineIndex,
};

const DEFINER: &str = r#"module probe.def

type ProbeCurrency =
    ProbeEur
  | ProbeUsd

fn probe_minor_unit(c: ProbeCurrency) -> Int {
  match c {
    ProbeEur => 2
    ProbeUsd => 2
  }
}
"#;

fn src(path: &str, content: &str) -> Arc<SourceFile> {
    Arc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

fn binding_byte_identical(
    a: &Arc<TypeBinding>,
    b: &Arc<TypeBinding>,
    source_indices: &Arc<HashMap<String, Arc<NewlineIndex>>>,
) -> bool {
    if Arc::ptr_eq(a, b) {
        return true;
    }
    if a.name != b.name {
        return false;
    }
    let a_auth = authored_name_at(source_indices.clone(), a.resolved.clone());
    let b_auth = authored_name_at(source_indices.clone(), b.resolved.clone());
    a_auth == b_auth
        && a.resolved.span.file == b.resolved.span.file
        && a.resolved.span.start == b.resolved.span.start
        && a.resolved.connective == b.resolved.connective
}

fn maps_byte_identical(
    got: &Arc<HashMap<String, Arc<TypeBinding>>>,
    expected: &Arc<HashMap<String, Arc<TypeBinding>>>,
    source_indices: &Arc<HashMap<String, Arc<NewlineIndex>>>,
) -> bool {
    if got.len() != expected.len() {
        return false;
    }
    got.iter().all(|(k, v)| {
        expected
            .get(k)
            .is_some_and(|ev| binding_byte_identical(v, ev, source_indices))
    })
}

/// Old `merge_global_bare_variant_locals` semantics (still-owl pre-fix).
fn simulate_old_merge_global_bare_variant_locals(
    global_bare: &Arc<HashMap<String, Arc<GlobalBareLookupState>>>,
    state: Arc<VariantFoldState>,
    source_indices: &Arc<HashMap<String, Arc<NewlineIndex>>>,
    module_name: &str,
) -> Arc<VariantFoldState> {
    global_bare
        .iter()
        .fold(state, |acc, (name, lookup)| match lookup.as_ref() {
            GlobalBareLookupState::GlobalBareUniqueBinding {
                module_path: _,
                binding,
            } => {
                let owner = binding.resolved.clone();
                if owner.connective == Connective::Disj
                    && has_child_named(owner.clone(), name.clone(), source_indices.clone())
                {
                    if acc.locals.contains_key(name) {
                        acc
                    } else {
                        insert_variant_owner_checked(
                            acc,
                            name.clone(),
                            owner,
                            source_indices.clone(),
                            module_name.to_string(),
                        )
                    }
                } else {
                    acc
                }
            }
            GlobalBareLookupState::GlobalBareAmbiguousBinding { .. } => acc,
        })
}

fn expected_variant_locals_old_path(
    resolved: &Arc<ResolvedModule>,
    source_indices: &Arc<HashMap<String, Arc<NewlineIndex>>>,
    intern_table: &Arc<InternTable>,
    global_bare: &Arc<HashMap<String, Arc<GlobalBareLookupState>>>,
) -> Arc<HashMap<String, Arc<TypeBinding>>> {
    let bte = build_type_env(
        resolved.clone(),
        v1_rt::rc_empty_map(),
        source_indices.clone(),
        intern_table.clone(),
        v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
    );
    let env = bte.env.clone();
    let module_name = authored_name_at(source_indices.clone(), resolved.module.clone());
    let local_fold = build_local_variants(
        resolved.module.children.clone(),
        source_indices.clone(),
        module_name.clone(),
        Arc::new(VariantFoldState {
            locals: v1_rt::rc_empty_map(),
            collision_errors: Arc::new(im::Vector::new()),
        }),
    );
    let after_old_merge = simulate_old_merge_global_bare_variant_locals(
        global_bare,
        local_fold,
        source_indices,
        &module_name,
    );
    v1_compiler::v1_compiler_infer::merge_kernel_variant_locals_low_priority(
        env,
        after_old_merge.locals.clone(),
    )
}

#[test]
fn receipt1_owning_module_same_authority_variant_locals_byte_identical() {
    let sources = Arc::new(
        vec![src("dag/probe_def.dag", DEFINER)]
            .into_iter()
            .collect::<im::Vector<_>>(),
    );
    let frontend = front_end_sources(sources);
    let graph = frontend.graph.clone().expect("graph");
    let source_indices = frontend.newline_indices.iter().cloned().fold(
        v1_rt::rc_empty_map::<String, Arc<NewlineIndex>>(),
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
    let module_path = authored_name_at(source_indices.clone(), resolved.module.clone());
    let intern_table = frontend.intern_table.clone();

    let global_bare = build_global_bare_census(norm.graph.modules.clone(), source_indices.clone());
    let global_bare_variant_locals =
        build_global_bare_variant_locals(global_bare.clone(), source_indices.clone());

    assert!(
        global_bare_variant_locals.contains_key("ProbeEur"),
        "shared census must include owning-module variant ProbeEur"
    );
    assert!(
        global_bare_variant_locals.contains_key("ProbeUsd"),
        "shared census must include owning-module variant ProbeUsd"
    );

    let tc = typecheck_module(
        resolved.clone(),
        v1_rt::rc_empty_map(),
        v1_rt::rc_empty_map(),
        source_indices.clone(),
        intern_table.clone(),
        v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
    );
    assert!(
        tc.diagnostics.is_empty(),
        "owning-module typecheck must be clean: {:?}",
        tc.diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );

    let got = tc.typed.type_env_cache.variant_locals.clone();
    let expected_old =
        expected_variant_locals_old_path(&resolved, &source_indices, &intern_table, &global_bare);

    assert!(
        maps_byte_identical(&got, &expected_old, &source_indices),
        "new shared-layer path must be byte-identical to old per-module merge for owning module \
         (same-authority variants ProbeEur/ProbeUsd); got keys={:?} expected keys={:?}",
        got.keys().collect::<Vec<_>>(),
        expected_old.keys().collect::<Vec<_>>()
    );

    // Discriminator: module overlay must win — ProbeEur binding must come from module fold,
    // not the census-built shared map (Rc identity differs when paths diverged pre-fix).
    let module_fold = build_local_variants(
        resolved.module.children.clone(),
        source_indices.clone(),
        module_path.clone(),
        Arc::new(VariantFoldState {
            locals: v1_rt::rc_empty_map(),
            collision_errors: Arc::new(im::Vector::new()),
        }),
    );
    let probe_eur_module = module_fold
        .locals
        .get("ProbeEur")
        .expect("module defines ProbeEur");
    let probe_eur_got = got
        .get("ProbeEur")
        .expect("env_variant_locals has ProbeEur");
    assert!(
        binding_byte_identical(probe_eur_got, probe_eur_module, &source_indices),
        "overlay-wins: env_variant_locals[ProbeEur] must be the module-fold binding"
    );
}
