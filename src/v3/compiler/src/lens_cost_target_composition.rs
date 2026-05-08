//! Slice 1a of T-CostLens-Composition follow-on (gunb-ai/gunbc#2141 ε scope).
//!
//! Provides [`RealizationCostTable`]: per-language HashMap of per-primitive
//! realization-cost facts extracted from `data X: TypeRealization` /
//! `CallableRealization` / `OperatorRealization` / `BehaviorRealization`
//! rows in `src/v3/spec/{rust,python,go}.dag`.
//!
//! Mirrors `emit/rust_target.rs:705+` HashMap-build precedent, scoped to the
//! `cost: Int` field consumption per ε ratification at gunbc#2181
//! #issuecomment-4401584012 (canvas
//! `docs/proposals/q-cost-composition-layering-canvas.md` Layer 2). Does NOT
//! consume the realization rows for emit (no carrier strings); solely
//! captures the per-primitive cost contribution that Slice 1b's
//! cost-composition consumer reads to specialize abstract `SymbolicCost`
//! shape into concrete cost.

use crate::dag::{Dag, DeclarationId, FieldValue, LiteralBits, ValueBody};
use std::collections::HashMap;

/// Per-language table of per-primitive realization costs.
///
/// Built once per (Dag, language_spec_id) pair via
/// [`RealizationCostTable::build_for_language`]. Categories mirror
/// `src/v3/std/emit_model.dag` realization meta-types.
#[derive(Debug, Clone, Default)]
pub struct RealizationCostTable {
    types: HashMap<DeclarationId, i64>,
    callables: HashMap<DeclarationId, i64>,
    operators: HashMap<(DeclarationId, DeclarationId), i64>,
    behaviors: HashMap<DeclarationId, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    MissingRealizationMeta(&'static str),
    MalformedRealization {
        declaration: DeclarationId,
        detail: &'static str,
    },
    DuplicateRealization {
        declaration: DeclarationId,
        detail: &'static str,
    },
}

#[derive(Copy, Clone)]
enum Category {
    Type,
    Callable,
    Operator,
    Behavior,
}

impl RealizationCostTable {
    /// Build a cost table for the given language spec by scanning the dag's
    /// `data` declarations for realization rows whose `language` field
    /// resolves to `language_id`.
    pub fn build_for_language(dag: &Dag, language_id: DeclarationId) -> Result<Self, BuildError> {
        let type_meta = dag
            .type_realization_meta()
            .ok_or(BuildError::MissingRealizationMeta("TypeRealization"))?;
        let callable_meta = dag
            .callable_realization_meta()
            .ok_or(BuildError::MissingRealizationMeta("CallableRealization"))?;
        let operator_meta = dag
            .operator_realization_meta()
            .ok_or(BuildError::MissingRealizationMeta("OperatorRealization"))?;
        let behavior_meta = dag
            .behavior_realization_meta()
            .ok_or(BuildError::MissingRealizationMeta("BehaviorRealization"))?;

        let mut table = Self::default();

        for decl in dag.declarations() {
            let Some(meta_tag) = decl.meta_tag else {
                continue;
            };
            let category = if meta_tag == type_meta {
                Category::Type
            } else if meta_tag == callable_meta {
                Category::Callable
            } else if meta_tag == operator_meta {
                Category::Operator
            } else if meta_tag == behavior_meta {
                Category::Behavior
            } else {
                continue;
            };

            let Some(ValueBody::Structural { fields }) = &decl.value_body else {
                return Err(BuildError::MalformedRealization {
                    declaration: decl.id,
                    detail: "realization data item has no Structural value_body",
                });
            };

            let language_ref = require_field_decl_ref(fields, "language", decl.id)?;
            if language_ref != language_id {
                continue;
            }

            let target = require_field_decl_ref(fields, "target", decl.id)?;
            let cost = require_field_int(fields, "cost", decl.id)?;

            match category {
                Category::Type => {
                    if table.types.insert(target, cost).is_some() {
                        return Err(BuildError::DuplicateRealization {
                            declaration: decl.id,
                            detail: "two TypeRealization rows target the same primitive",
                        });
                    }
                }
                Category::Callable => {
                    if table.callables.insert(target, cost).is_some() {
                        return Err(BuildError::DuplicateRealization {
                            declaration: decl.id,
                            detail: "two CallableRealization rows target the same callable",
                        });
                    }
                }
                Category::Operator => {
                    let op = require_field_decl_ref(fields, "op", decl.id)?;
                    if table.operators.insert((target, op), cost).is_some() {
                        return Err(BuildError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two OperatorRealization rows target the same (operand_type, op_field) pair",
                        });
                    }
                }
                Category::Behavior => {
                    if table.behaviors.insert(target, cost).is_some() {
                        return Err(BuildError::DuplicateRealization {
                            declaration: decl.id,
                            detail: "two BehaviorRealization rows target the same behavior marker",
                        });
                    }
                }
            }
        }

        Ok(table)
    }

    pub fn type_cost(&self, primitive: DeclarationId) -> Option<i64> {
        self.types.get(&primitive).copied()
    }

    pub fn callable_cost(&self, callable: DeclarationId) -> Option<i64> {
        self.callables.get(&callable).copied()
    }

    pub fn operator_cost(&self, operand: DeclarationId, op: DeclarationId) -> Option<i64> {
        self.operators.get(&(operand, op)).copied()
    }

    pub fn behavior_cost(&self, behavior: DeclarationId) -> Option<i64> {
        self.behaviors.get(&behavior).copied()
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    pub fn callable_count(&self) -> usize {
        self.callables.len()
    }

    pub fn operator_count(&self) -> usize {
        self.operators.len()
    }

    pub fn behavior_count(&self) -> usize {
        self.behaviors.len()
    }
}

fn require_field_decl_ref(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<DeclarationId, BuildError> {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Reference(id) => Some(*id),
            _ => None,
        })
        .ok_or(BuildError::MalformedRealization {
            declaration,
            detail: "realization data item is missing a required Reference field",
        })
}

fn require_field_int(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<i64, BuildError> {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Literal(LiteralBits::Int(n)) => Some(*n),
            _ => None,
        })
        .ok_or(BuildError::MalformedRealization {
            declaration,
            detail: "realization data item is missing a required Int field (`cost`)",
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated_full_bootstrap_dag;

    /// Slice 1a smoke: builder against bootstrap rust.dag spec data
    /// produces a non-empty `RealizationCostTable` with at-least-one
    /// known-key cost lookup. Per Substrate Mgr disposition at gunbc#2068
    /// #issuecomment-4402516739: minimal 1a self-test at the boundary;
    /// exhaustive variant coverage lives in Slice 1b's parameterized-fold
    /// tests.
    #[test]
    fn build_for_rust_language_produces_nonempty_table() {
        let dag = generated_full_bootstrap_dag();
        let rust_id = dag
            .rust_language_spec()
            .expect("bootstrap dag has rust_language");
        let table = RealizationCostTable::build_for_language(&dag, rust_id)
            .expect("build_for_language(rust) succeeds against bootstrap dag");

        // Smoke: at least one TypeRealization landed in the type table
        // (bootstrap rust.dag has rust_int / rust_uint8 / rust_i32 / etc.,
        // all with `cost: 1`).
        assert!(
            table.type_count() > 0,
            "RealizationCostTable types map is empty for Rust; expected at least one \
             TypeRealization row from bootstrap rust.dag"
        );

        // At-least-one known-key cost lookup: the `Int` primitive has a
        // `rust_int: TypeRealization { cost: 1, ... }` row in
        // src/v3/spec/rust.dag (line ~118).
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("bootstrap dag has Int declaration");
        let int_cost = table.type_cost(int_decl.id);
        assert_eq!(
            int_cost,
            Some(1),
            "type_cost(Int) for Rust target should be 1 per src/v3/spec/rust.dag rust_int row; \
             got {int_cost:?}"
        );
    }
}
