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

use crate::dag::{Dag, DeclarationId, RuntimeBootstrapFixtureKind};
use crate::lower::{collect_symbols_phase, lower_bodies_phase, resolve_pending_identifiers};
use crate::parse::{parse, SurfaceModule};
use crate::tokenize::tokenize;
use std::collections::HashMap;

const LOGIC_DAG: &str = include_str!("../../../../dsl/std/logic.dag");
const BIT_DAG: &str = include_str!("../../../../dsl/std/bit.dag");
const ERROR_PRIMITIVES_DAG: &str = include_str!("../../../../dsl/std/error_primitives.dag");
const ALGEBRA_DAG: &str = include_str!("../../../../dsl/std/algebra.dag");
const MAGNITUDE_DAG: &str = include_str!("../../../../dsl/std/magnitude.dag");
const MACHINE_CONSTRAINTS_DAG: &str = include_str!("../../../../dsl/std/machine_constraints.dag");
const NAT_DAG: &str = include_str!("../../../../dsl/std/nat.dag");
const INTEGER_DAG: &str = include_str!("../../../../dsl/std/integer.dag");
const RATIONAL_DAG: &str = include_str!("../../../../dsl/std/rational.dag");
const APPROXIMATE_FIELD_DAG: &str = include_str!("../../../v3/std/approximate_field.dag");
const FLOAT_DAG: &str = include_str!("../../../../dsl/std/float.dag");
const STRING_TYPE_DAG: &str = include_str!("../../../../dsl/std/string_type.dag");
const TYPES_DAG: &str = include_str!("../../../../dsl/std/types.dag");
const UNICODE_DAG: &str = include_str!("../../../../dsl/std/unicode.dag");
const RENDER_REPEAT_STRING_BOOTSTRAP_DAG: &str =
    include_str!("../../../../dsl/std/render_repeat_string_bootstrap.dag");
const METHODS_DAG: &str = include_str!("../../../../dsl/std/methods.dag");

// Same `OUT_DIR` fixture arrays `build.rs` emits for staged/spec/compiler/extdeps sources;
// this module is their only remaining fresh-parse consumer.
include!(concat!(env!("OUT_DIR"), "/v3_staged_files.rs"));
include!(concat!(env!("OUT_DIR"), "/v3_specs.rs"));
include!(concat!(env!("OUT_DIR"), "/v3_compiler_files.rs"));
include!(concat!(env!("OUT_DIR"), "/v3_extdeps_files.rs"));

// `BOOTSTRAP_FIXTURE_PATH_KEYS` may resolve against either STAGED_FILES or
// EXTDEPS_FILES (mixed-tree fixture set per T-Ground-LanguageSpec scope E.1).
// STAGED_FILES are already loaded via `staged_iter` in
// `load_runtime_bootstrap_authorities`, so this iterator returns only the
// extdeps-tree subset of the keys to avoid duplicate-declaration loads.
fn extdeps_keyed_bootstrap_fixtures() -> impl Iterator<Item = (&'static str, &'static str)> {
    EXTDEPS_FILES
        .iter()
        .copied()
        .filter(|(path, _)| crate::bootstrap::BOOTSTRAP_FIXTURE_PATH_KEYS.contains(path))
}

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
    load_fixtures(&mut dag, std_fixtures());
    dag.finalize_runtime_bootstrap_from_generated_snapshot(
        RuntimeBootstrapFixtureKind::StdOnlySnapshot,
    );
    dag
}

pub fn compile_full_bootstrap_dag_from_std_seed(std_seed: Dag) -> Dag {
    let mut dag = std_seed;
    // `approximate_field.dag` is part of `std_fixtures()` so `dsl/std/float.dag` can
    // import `ApproximateField`; skip the staged copy to avoid duplicate declarations.
    load_runtime_bootstrap_authorities(&mut dag, &["src/v3/std/approximate_field.dag"], &[]);
    dag
}

pub fn compile_full_bootstrap_without_parse_surface_dag_from_std_seed(std_seed: Dag) -> Dag {
    let mut dag = std_seed;
    load_runtime_bootstrap_authorities(
        &mut dag,
        &[
            "src/v3/std/parse_surface.dag",
            "src/v3/std/approximate_field.dag",
        ],
        &[],
    );
    dag
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
    assert_bootstrap_fixture_keys_resolve();
    let fixtures: Vec<(&'static str, &'static str)> = staged_iter
        .chain(V3_SPECS.iter().copied())
        .chain(compiler_iter)
        .chain(extdeps_keyed_bootstrap_fixtures())
        .collect();
    load_fixtures(dag, &fixtures);
    crate::bootstrap::materialize_pipeline_realizations(dag);
    dag.finalize_runtime_bootstrap_from_generated_snapshot(
        RuntimeBootstrapFixtureKind::FullExtdepsPipelineSnapshot,
    );
}

fn assert_bootstrap_fixture_keys_resolve() {
    let missing: Vec<&str> = crate::bootstrap::BOOTSTRAP_FIXTURE_PATH_KEYS
        .iter()
        .copied()
        .filter(|key| {
            !STAGED_FILES.iter().any(|(path, _)| *path == *key)
                && !EXTDEPS_FILES.iter().any(|(path, _)| *path == *key)
        })
        .collect();
    assert!(
        missing.is_empty(),
        "BOOTSTRAP_FIXTURE_PATH_KEYS must each appear in STAGED_FILES or EXTDEPS_FILES \
         (from build.rs): {missing:?}"
    );
}

fn std_fixtures() -> &'static [(&'static str, &'static str)] {
    &[
        ("dsl/std/logic.dag", LOGIC_DAG),
        ("dsl/std/bit.dag", BIT_DAG),
        ("dsl/std/error_primitives.dag", ERROR_PRIMITIVES_DAG),
        ("dsl/std/algebra.dag", ALGEBRA_DAG),
        ("dsl/std/magnitude.dag", MAGNITUDE_DAG),
        ("dsl/std/machine_constraints.dag", MACHINE_CONSTRAINTS_DAG),
        ("dsl/std/nat.dag", NAT_DAG),
        ("dsl/std/integer.dag", INTEGER_DAG),
        ("dsl/std/rational.dag", RATIONAL_DAG),
        ("src/v3/std/approximate_field.dag", APPROXIMATE_FIELD_DAG),
        ("dsl/std/float.dag", FLOAT_DAG),
        ("dsl/std/types.dag", TYPES_DAG),
        ("dsl/std/string_type.dag", STRING_TYPE_DAG),
        ("dsl/std/unicode.dag", UNICODE_DAG),
        (
            "dsl/std/render_repeat_string_bootstrap.dag",
            RENDER_REPEAT_STRING_BOOTSTRAP_DAG,
        ),
        ("dsl/std/methods.dag", METHODS_DAG),
    ]
}

fn load_fixtures(dag: &mut Dag, fixtures: &[(&'static str, &'static str)]) {
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

fn parse_fixture(dag: &mut Dag, source: &str, file: &'static str) -> Option<SurfaceModule> {
    let authority = crate::diagnostics::BootstrapAuthorityKey::new(file);
    let tokens = match tokenize(source, file) {
        Ok(t) => t,
        Err(diag) => {
            dag.attach_bootstrap_diagnostic(authority, diag);
            return None;
        }
    };
    match parse(&tokens, file) {
        Ok(m) => Some(m),
        Err(diag) => {
            dag.attach_bootstrap_diagnostic(authority, diag);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    //! Bootstrap-side coverage of `BootstrapAuthorityKey` /
    //! `DiagnosticAttribution::BootstrapAuthority` for the
    //! parse/tokenize attach path. Sibling unit tests in
    //! `crate::diagnostics` cover the type-level invariants;
    //! `crate::bootstrap::tests` covers the kernel-Bool detached
    //! phantom-port path. This test exercises the third bootstrap
    //! attach path (`parse_fixture`'s tokenize/parse-failure branches).
    //!
    //! Verification (PR #1572 Worker B') consumes the resulting
    //! attribution via [`DiagnosticTable::iter_attributed`] /
    //! [`DiagnosticTable::attribution`] without ever asking
    //! `Diagnostic.span().file`.

    use super::*;
    use crate::diagnostics::{BootstrapAuthorityKey, DiagnosticAttribution};

    #[test]
    fn parse_fixture_tokenize_failure_carries_bootstrap_authority() {
        // Arbitrary illegal byte makes `tokenize` fail closed.
        let source = "\u{0}";
        let file: &'static str = "src/v3/std/bootstrap_regen_fresh_tokenize_smoke.dag";
        let mut dag = Dag::empty();
        let outcome = parse_fixture(&mut dag, source, file);
        assert!(outcome.is_none(), "tokenize must fail for this fixture");
        let expected = BootstrapAuthorityKey::new(file);
        let bootstrap_count = dag
            .diagnostics()
            .iter_attributed()
            .filter(|(_, _, attribution)| attribution.as_bootstrap_authority() == Some(&expected))
            .count();
        assert_eq!(
            bootstrap_count,
            1,
            "tokenize-failure diagnostic must carry BootstrapAuthority({file:?}); table: {:?}",
            dag.diagnostics().iter_attributed().collect::<Vec<_>>()
        );
        for (_, _, attribution) in dag.diagnostics().iter_attributed() {
            assert!(
                matches!(attribution, DiagnosticAttribution::BootstrapAuthority(_)),
                "every diagnostic from parse_fixture must be BootstrapAuthority-attributed, got {attribution:?}"
            );
        }
    }

    #[test]
    fn parse_fixture_parse_failure_carries_bootstrap_authority() {
        // Tokenizes cleanly but trips parse (unmatched closing brace at
        // top level — not a valid module item).
        let source = "}\n";
        let file: &'static str = "src/v3/std/bootstrap_regen_fresh_parse_smoke.dag";
        let mut dag = Dag::empty();
        let outcome = parse_fixture(&mut dag, source, file);
        assert!(outcome.is_none(), "parse must fail for this fixture");
        let expected = BootstrapAuthorityKey::new(file);
        assert!(
            dag.diagnostics()
                .iter_attributed()
                .any(|(_, _, attribution)| attribution.as_bootstrap_authority() == Some(&expected)),
            "parse-failure diagnostic must carry BootstrapAuthority({file:?}); table: {:?}",
            dag.diagnostics().iter_attributed().collect::<Vec<_>>()
        );
    }
}
