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
#[derive(Debug, Clone)]
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
    /// `language_id` matched zero realization rows across all four
    /// categories. Either the id doesn't reference a valid LanguageSpec
    /// declaration, or the spec exists but has no realization rows
    /// authored. Either is a bug-like state for a downstream consumer
    /// expecting per-primitive cost facts; fail-closed at the
    /// substrate-consumer boundary.
    NoRealizationCostsForLanguage {
        language: DeclarationId,
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
    /// Private internal initializer. Public construction goes through
    /// [`RealizationCostTable::build_for_language`] so the substrate-derived
    /// table cannot be fabricated empty/ungrounded by a downstream consumer
    /// (fail-closed at the substrate-consumer boundary per
    /// `INVARIANTS.md` LAYER MODEL — gpt-5-5-pro review on PR #2194 sha
    /// 6548ccf4 BLOCKING finding).
    fn empty() -> Self {
        Self {
            types: HashMap::new(),
            callables: HashMap::new(),
            operators: HashMap::new(),
            behaviors: HashMap::new(),
        }
    }

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

        let mut table = Self::empty();

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

            // Mirror the coarse-grained field-presence acceptance criteria
            // from `emit/rust_target.rs:779+` so this consumer accepts only
            // rows the existing realization authority also accepts (single
            // row-validity contract — INVARIANTS P2; gpt-5-5-pro REQUEST_CHANGES
            // on PR #2194 sha 9ed08bc3 BLOCKING).
            match category {
                Category::Type => {
                    require_field_string(fields, "carrier", decl.id)?;
                    require_field_bool(fields, "is_copy", decl.id)?;
                    require_field_present(fields, "fields", decl.id)?;
                    if table.types.insert(target, cost).is_some() {
                        return Err(BuildError::DuplicateRealization {
                            declaration: decl.id,
                            detail: "two TypeRealization rows target the same primitive",
                        });
                    }
                }
                Category::Callable => {
                    require_field_present(fields, "strategy", decl.id)?;
                    require_field_present(fields, "parameters", decl.id)?;
                    if table.callables.insert(target, cost).is_some() {
                        return Err(BuildError::DuplicateRealization {
                            declaration: decl.id,
                            detail: "two CallableRealization rows target the same callable",
                        });
                    }
                }
                Category::Operator => {
                    let op = require_field_decl_ref(fields, "op", decl.id)?;
                    require_field_string(fields, "carrier", decl.id)?;
                    if table.operators.insert((target, op), cost).is_some() {
                        return Err(BuildError::DuplicateRealization {
                            declaration: decl.id,
                            detail:
                                "two OperatorRealization rows target the same (operand_type, op_field) pair",
                        });
                    }
                }
                Category::Behavior => {
                    require_field_string(fields, "carrier", decl.id)?;
                    if table.behaviors.insert(target, cost).is_some() {
                        return Err(BuildError::DuplicateRealization {
                            declaration: decl.id,
                            detail: "two BehaviorRealization rows target the same behavior marker",
                        });
                    }
                }
            }
        }

        // Fail-closed: a language_id that matched zero realization rows is
        // either bogus (not a LanguageSpec) or a spec with no realization
        // rows authored — neither is a state a downstream consumer should
        // see as a "successful build". Surface as typed error rather than
        // returning a fabricated-empty table.
        if table.types.is_empty()
            && table.callables.is_empty()
            && table.operators.is_empty()
            && table.behaviors.is_empty()
        {
            return Err(BuildError::NoRealizationCostsForLanguage {
                language: language_id,
            });
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

fn require_field_string(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<(), BuildError> {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Literal(LiteralBits::String(_)) => Some(()),
            _ => None,
        })
        .ok_or(BuildError::MalformedRealization {
            declaration,
            detail: "realization data item is missing a required String field",
        })
}

fn require_field_bool(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<(), BuildError> {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .and_then(|(_, v)| match v {
            FieldValue::Literal(LiteralBits::Bool(_)) => Some(()),
            _ => None,
        })
        .ok_or(BuildError::MalformedRealization {
            declaration,
            detail: "realization data item is missing a required Bool field",
        })
}

/// Coarse-grained presence check for fields whose internal shape is
/// validated by the bootstrap-inhabitance check (e.g., `fields:
/// List<FieldBinding>`, `strategy: CallableStrategy`, `parameters:
/// List<CallableParameter>`). This consumer only needs to confirm the
/// field is authored on the row; full shape validation lives at the
/// substrate-inhabitance boundary.
fn require_field_present(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<(), BuildError> {
    if fields.iter().any(|(l, _)| l == label) {
        Ok(())
    } else {
        Err(BuildError::MalformedRealization {
            declaration,
            detail: "realization data item is missing a required field",
        })
    }
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

        // At-least-one known-key cost lookup: the `Int` primitive has a
        // `rust_int: TypeRealization { cost: 1, ... }` row in
        // src/v3/spec/rust.dag (line ~118). A successful Some(1) lookup
        // implicitly asserts the table is non-empty + the bootstrap row
        // landed correctly.
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

    /// Negative coverage of the fail-closed boundary: a `language_id`
    /// that doesn't reference a LanguageSpec (here: the `Int` primitive
    /// declaration itself) must yield `BuildError::NoRealizationCostsForLanguage`,
    /// NOT a successful empty table. Per gpt-5-5-pro REQUEST_CHANGES on
    /// PR #2194 sha 36e63d22 (LAYER MODEL + Fail-Closed BLOCKING).
    #[test]
    fn build_for_non_language_id_fails_closed() {
        let dag = generated_full_bootstrap_dag();
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("bootstrap dag has Int declaration");
        let bogus_language = int_decl.id;

        let result = RealizationCostTable::build_for_language(&dag, bogus_language);
        match result {
            Err(BuildError::NoRealizationCostsForLanguage { language }) => {
                assert_eq!(
                    language, bogus_language,
                    "error should carry the bogus language id back"
                );
            }
            Err(other) => panic!(
                "expected NoRealizationCostsForLanguage for bogus language id; got {other:?}"
            ),
            Ok(_) => panic!(
                "expected fail-closed error for bogus language id; got Ok(table) instead — \
                 fabricated-empty-table fail-closed contract violated"
            ),
        }
    }
}
