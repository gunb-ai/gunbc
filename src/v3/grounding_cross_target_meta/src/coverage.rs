//! Project landed LanguageSpec-adjacent rows onto L6 cross-product cells.
//!
//! Authority at HEAD: `src/v3/std/{rust,python,go}_method_template_contracts.dag`
//! (`List<MethodTemplateContract>` per target). Each row declares runtime + emit
//! templates for one `std.methods` registry method — collection-algebra **method
//! templates** on cardinality-bearing receivers.
//!
//! **Audit mapping (HEAD):** every Phase 1 row attaches to the same structural L6
//! bucket — **[`FormAxis::Cardinality`] × [`BehaviorAxis::Transform`] × target** —
//! because these rows are the Shape A emission paths for **Transform-shaped method
//! calls** on structured collection carriers (the connective axis for list-like
//! shapes is [`TypeConnective::Cardinality`] per substrate; invocation is L1
//! **Transform**, not Value/Branch/Loop/Bind). Targets split only along
//! [`ShapeATarget`].
//!
//! Future LanguageSpec tables should extend [`language_spec_emission_cells_covered`]
//! to union additional `(connective × behavior × target)` cells without changing
//! the walker's public surface.

use std::collections::HashSet;

use v3_compiler::dag::{Dag, ValueBody};

use crate::cells::{BehaviorAxis, Cell, FormAxis, ShapeATarget};

/// Bootstrap list declarations that carry `MethodTemplateContract` rows per Shape A
/// target (`emit_model.dag` + `extdeps_bootstrap_fixtures.dag`).
const TARGET_LISTS: &[(&str, ShapeATarget)] = &[
    ("rust_method_template_contracts", ShapeATarget::Rust),
    ("python_method_template_contracts", ShapeATarget::Python),
    ("go_method_template_contracts", ShapeATarget::Go),
];

/// Returns the set of L6 cells for which landed LanguageSpec rows declare at least
/// one emission-path template at substrate load time.
///
/// Fail-closed: missing declarations, wrong `value_body` shapes, or empty lists
/// contribute **no** coverage (those targets stay absent from the returned set).
///
/// TODO: When rows span multiple L6 cells per target (e.g. Branch-shaped templates),
/// replace list-non-empty checks with a per-row projection that unions each row's
/// target `Cell`s instead of a single bucket per list.
pub(crate) fn language_spec_emission_cells_covered(dag: &Dag) -> HashSet<Cell> {
    let mut covered = HashSet::new();
    for &(list_name, target) in TARGET_LISTS {
        if method_template_list_covers_bucket(dag, list_name) {
            covered.insert(Cell {
                connective: FormAxis::Cardinality,
                behavior: BehaviorAxis::Transform,
                target,
            });
        }
    }
    covered
}

fn method_template_list_covers_bucket(dag: &Dag, list_name: &str) -> bool {
    let Some(decl) = dag.declaration_by_name(list_name) else {
        return false;
    };
    let Some(body) = decl.value_body.as_ref() else {
        return false;
    };
    let ValueBody::List(rows) = body else {
        return false;
    };
    !rows.is_empty()
}
