//! Fresh tokenize/parse/lower bootstrap from on-disk `.dag` authorities.
//!
//! This module is compiled **only** with Cargo feature `bootstrap-regen-fresh`
//! (the `regen_bootstrap` binary enables it via `required-features`). It is not
//! a second production bootstrap authority: `Dag::new()` loads the committed
//! `bootstrap_generated.rs` snapshot. Per PB-1-e mechanism (ii), the
//! fresh-compile-vs-snapshot acid test runs at `regen_bootstrap` time
//! (`--verify`), not in the default integration test suite.
//!
//! **Classification:** permanent PB-1-e **regen verification** substrate (not
//! the old “delete once the drift harness stops needing runtime std” scaffold).
//! **Dissolution:** when PB-Bootstrap-Process replaces this hand-Rust regen
//! host with a declared `bootstrap.dag` / generated producer path, delete this
//! module in favor of that single authority — same cost-of-change rule as the
//! retired `bootstrap_std_fixtures_only` dissolution trigger, reframed for
//! “regen host goes data-native” rather than “in-tree tests stop diffing.”

use crate::dag::{Dag, DeclarationId};
use crate::lower::{collect_symbols_phase, lower_bodies_phase, resolve_pending_identifiers};
use crate::parse::{parse, SurfaceModule};
use crate::tokenize::tokenize;
use std::collections::HashMap;

const LOGIC_DAG: &str = include_str!("../../../../dsl/std/logic.dag");
const BIT_DAG: &str = include_str!("../../../../dsl/std/bit.dag");
const ALGEBRA_DAG: &str = include_str!("../../../../dsl/std/algebra.dag");
const INTEGER_DAG: &str = include_str!("../../../../dsl/std/integer.dag");
const FLOAT_DAG: &str = include_str!("../../../../dsl/std/float.dag");
const STRING_TYPE_DAG: &str = include_str!("../../../../dsl/std/string_type.dag");
const TYPES_DAG: &str = include_str!("../../../../dsl/std/types.dag");

const EXTDEPS_RUST_PRIMITIVES_DAG: &str =
    include_str!("../../../../dsl/extdeps/languages/rust/primitives.dag");

const EXTDEPS_BOOTSTRAP_FIXTURES: &[(&str, &str)] = &[(
    "dsl/extdeps/languages/rust/primitives.dag",
    EXTDEPS_RUST_PRIMITIVES_DAG,
)];

// Same `OUT_DIR` fixture arrays `build.rs` emits for staged/spec/compiler sources;
// this module is their only remaining fresh-parse consumer.
include!(concat!(env!("OUT_DIR"), "/v3_staged_files.rs"));
include!(concat!(env!("OUT_DIR"), "/v3_specs.rs"));
include!(concat!(env!("OUT_DIR"), "/v3_compiler_files.rs"));

// Single authority for staged-vs-`dsl/` name collision resolution during regen.
// The committed `bootstrap_generated.rs` snapshot was baked with this policy; any
// future runtime path that re-parses the same fixture set must keep this logic
// in lockstep (cost of change: one module — extend here only).
fn declaration_name_preference_rank(file: &str) -> usize {
    if file.starts_with("src/v3/") {
        2
    } else if file.starts_with("dsl/") {
        0
    } else {
        1
    }
}

pub fn compile_std_bootstrap_dag() -> Dag {
    let mut dag = Dag::empty();
    bootstrap_std_fixtures_only(&mut dag);
    dag
}

pub fn compile_full_bootstrap_dag_from_std_seed(std_seed: Dag) -> Dag {
    let mut dag = std_seed;
    bootstrap_runtime_authorities_on(&mut dag, &[], &[]);
    dag
}

pub fn compile_full_bootstrap_without_parse_surface_dag_from_std_seed(std_seed: Dag) -> Dag {
    let mut dag = std_seed;
    bootstrap_runtime_authorities_on(&mut dag, &["src/v3/std/parse_surface.dag"], &[]);
    dag
}

fn bootstrap_std_fixtures_only(dag: &mut Dag) {
    *dag = Dag::empty();
    load_fixtures(dag, std_fixtures());
    dag.populate_primitive_cache();
}

fn bootstrap_runtime_authorities_on(
    dag: &mut Dag,
    excluded_staged_paths: &[&str],
    excluded_compiler_paths: &[&str],
) {
    load_runtime_bootstrap_authorities(dag, excluded_staged_paths, excluded_compiler_paths);
}

fn load_runtime_bootstrap_authorities(
    dag: &mut Dag,
    excluded_staged_paths: &[&str],
    excluded_compiler_paths: &[&str],
) {
    let staged_iter = STAGED_FILES
        .iter()
        .copied()
        .filter(|(path, _)| !excluded_staged_paths.contains(path));
    let compiler_iter = COMPILER_FILES
        .iter()
        .copied()
        .filter(|(path, _)| !excluded_compiler_paths.contains(path));
    let fixtures: Vec<(&str, &str)> = staged_iter
        .chain(V3_SPECS.iter().copied())
        .chain(compiler_iter)
        .chain(EXTDEPS_BOOTSTRAP_FIXTURES.iter().copied())
        .collect();
    load_fixtures(dag, &fixtures);
    crate::bootstrap::materialize_pipeline_realizations(dag);
    dag.populate_primitive_cache();
}

fn std_fixtures() -> &'static [(&'static str, &'static str)] {
    &[
        ("dsl/std/logic.dag", LOGIC_DAG),
        ("dsl/std/bit.dag", BIT_DAG),
        ("dsl/std/algebra.dag", ALGEBRA_DAG),
        ("dsl/std/integer.dag", INTEGER_DAG),
        ("dsl/std/float.dag", FLOAT_DAG),
        ("dsl/std/types.dag", TYPES_DAG),
        ("dsl/std/string_type.dag", STRING_TYPE_DAG),
    ]
}

fn load_fixtures(dag: &mut Dag, fixtures: &[(&str, &str)]) {
    let mut parsed: Vec<(SurfaceModule, Vec<bool>)> = Vec::with_capacity(fixtures.len());
    for (file, source) in fixtures.iter() {
        let Some(module) = parse_fixture(dag, source, file) else {
            continue;
        };
        let (_stale_symbols, is_first) = collect_symbols_phase(dag, &module.items);
        parsed.push((module, is_first));
    }

    let mut shared_symbols: HashMap<String, DeclarationId> = HashMap::new();
    for d in dag.declarations() {
        if let Some(name) = &d.name {
            match shared_symbols.get(name).copied() {
                None => {
                    shared_symbols.insert(name.clone(), d.id);
                }
                Some(existing_id) => {
                    let existing = dag.declaration(existing_id);
                    let new_rank = declaration_name_preference_rank(&d.span.file);
                    let existing_rank = declaration_name_preference_rank(&existing.span.file);
                    if new_rank > existing_rank {
                        shared_symbols.insert(name.clone(), d.id);
                    }
                }
            }
        }
    }

    for (module, is_first) in parsed.iter() {
        lower_bodies_phase(dag, module, &shared_symbols, is_first);
    }

    resolve_pending_identifiers(dag);
    crate::bootstrap::patch_kernel_bool_boolean_algebra_inhabits(dag);
}

fn parse_fixture(dag: &mut Dag, source: &str, file: &str) -> Option<SurfaceModule> {
    let tokens = match tokenize(source, file) {
        Ok(t) => t,
        Err(diag) => {
            dag.attach_diagnostic(diag);
            return None;
        }
    };
    match parse(&tokens, file) {
        Ok(m) => Some(m),
        Err(diag) => {
            dag.attach_diagnostic(diag);
            None
        }
    }
}
