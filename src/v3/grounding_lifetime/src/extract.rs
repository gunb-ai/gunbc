//! Dag → `LifetimeProgram` projection (R2 extraction).
//!
//! Bootstrap fixtures (`Dag::new()`) embed a large declaration snapshot that is not yet
//! lowered into the bind/use graph this lane analyzes, so extraction returns an **empty**
//! program for fixture-only rows (the structural fold is validated via [`LifetimeProgram`]
//! fixtures).
//!
//! Any runtime-appended **`data` or `fn` declaration** with load-bearing surface (`data` body
//! or `fn` / `Arrow` connective) is treated as program surface we cannot project yet:
//! fail-closed per C-8 instead of returning `Ok(empty)` and dropping facts.
//!
//! ## Authority gate (structural boundary)
//!
//! Fixture vs user rows are distinguished by **Substrate-declared declaration identity**, not
//! by path-prefix matching on caller-controlled `span.file` strings on each declaration.
//! After **PR #1221**, [`Dag::post_bootstrap_declaration_append_begin`] and
//! [`Dag::is_runtime_appended_declaration`] stamp the first [`DeclarationId::raw`] allocated
//! after embedded bootstrap construction; runtime lowering (`compile_to_dag`, etc.) appends
//! rows at or above that bound. **Path-prefix staging** from #1218 / #1220 is **retired** here
//! in favor of that boundary (cross-manager request #1130 satisfied on the compiler side).

use v3_compiler::dag::{Dag, TypeConnective};

use crate::analyze;
use crate::axes::LanguageSpecAxes;
use crate::diagnostic::EmissionDiagnostic;
use crate::facts::LifetimeFacts;
use crate::program::{BindingId, LifetimeProgram};

use std::collections::BTreeMap;

fn first_runtime_appended_lifetime_surface_declaration(dag: &Dag) -> Option<(String, String)> {
    for decl in dag.declarations() {
        if !dag.is_runtime_appended_declaration(decl.id) {
            continue;
        }
        let surface =
            decl.value_body.is_some() || matches!(&decl.connective, TypeConnective::Arrow { .. });
        if !surface {
            continue;
        }
        let name = decl
            .name
            .clone()
            .unwrap_or_else(|| format!("declaration#{}", decl.id.raw()));
        return Some((name, decl.span.file.clone()));
    }
    None
}

/// Project the reflected `Dag` into the lifetime analyzer’s program slice.
///
/// Fail-closed on constructs the R2 analyzer does not model (once lowering surfaces them).
pub fn extract_lifetime_program(dag: &Dag) -> Result<LifetimeProgram, EmissionDiagnostic> {
    if let Some((name, file)) = first_runtime_appended_lifetime_surface_declaration(dag) {
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
