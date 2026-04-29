//! Dag → `LifetimeProgram` projection (R2 extraction).
//!
//! Bootstrap fixtures (`Dag::new()`) only carry declarations whose spans live under
//! checked-in authority prefixes; those are not yet lowered into the bind/use graph,
//! so extraction returns an **empty** program (the structural fold is validated via
//! [`LifetimeProgram`] fixtures).
//!
//! Any **`data` or `fn` declaration** minted **after** the embedded fixture snapshot
//! ([`Dag::declaration_append_boundary`]) is treated as load-bearing program surface we
//! cannot project yet — **even if** `span.file` looks like an authority path (callers
//! control the compile-time file string). Fail-closed per C-8 instead of returning
//! `Ok(empty)` and dropping facts.

use v3_compiler::dag::{Dag, TypeConnective};

use crate::analyze;
use crate::axes::LanguageSpecAxes;
use crate::diagnostic::EmissionDiagnostic;
use crate::facts::LifetimeFacts;
use crate::program::{BindingId, LifetimeProgram};

use std::collections::BTreeMap;

/// Span roots that appear on [`Dag::new()`] bootstrap fixtures (regenerated snapshots).
///
/// Used only as **defense in depth** for rows with `DeclarationId::raw` **below**
/// [`Dag::declaration_append_boundary`]: those rows are expected to be corpus-only.
///
/// Keep aligned with `bootstrap_generated.rs` / `bootstrap_generated_without_parse_surface.rs`
/// when new fixture corpora land; otherwise user/test modules may be misclassified.
///
/// Note: only **three** `src/v3/compiler/*.dag` stubs ship inside the fixture — not every path
/// under `src/v3/compiler/` (tests live there too).
fn is_bootstrap_fixture_authority_source_file(file: &str) -> bool {
    if file.starts_with("dsl/std/")
        || file.starts_with("dsl/extdeps/")
        || file.starts_with("src/v3/std/")
        || file.starts_with("src/v3/spec/")
    {
        return true;
    }
    matches!(
        file,
        "src/v3/compiler/operators.dag"
            | "src/v3/compiler/pipeline.dag"
            | "src/v3/compiler/regen.dag"
    )
}

fn first_unextracted_lifetime_surface_declaration(dag: &Dag) -> Option<(String, String)> {
    let boundary = dag.declaration_append_boundary();
    for decl in dag.declarations() {
        let surface =
            decl.value_body.is_some() || matches!(&decl.connective, TypeConnective::Arrow { .. });
        if !surface {
            continue;
        }
        let name = decl
            .name
            .clone()
            .unwrap_or_else(|| format!("declaration#{}", decl.id.raw()));
        let post_fixture = decl.id.raw() >= boundary;
        if post_fixture {
            return Some((name, decl.span.file.clone()));
        }
        if !is_bootstrap_fixture_authority_source_file(&decl.span.file) {
            // Fixture-id range but not stamped like the embedded corpus — do not trust.
            return Some((name, decl.span.file.clone()));
        }
    }
    None
}

/// Project the reflected `Dag` into the lifetime analyzer’s program slice.
///
/// Fail-closed on constructs the R2 analyzer does not model (once lowering surfaces them).
pub fn extract_lifetime_program(dag: &Dag) -> Result<LifetimeProgram, EmissionDiagnostic> {
    if let Some((name, file)) = first_unextracted_lifetime_surface_declaration(dag) {
        return Err(EmissionDiagnostic::LifetimeProgramExtractionPending {
            detail: format!("{name} ({file})"),
        });
    }
    Ok(LifetimeProgram::empty())
}

/// Public entry: **`Dag` + `LanguageSpecAxes` only** — no annotation table, no env sidecar
/// (`t-ground-lifetime-analyzer.md` test plan item 7).
pub fn analyze_lifetime_facts(
    dag: &Dag,
    axes: &LanguageSpecAxes,
) -> Result<BTreeMap<BindingId, LifetimeFacts>, EmissionDiagnostic> {
    let program = extract_lifetime_program(dag)?;
    analyze::analyze_lifetime_program(&program, axes)
}
