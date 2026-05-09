//! Rust-side realization-cost table for T-CostLens-Composition's epsilon path.
//!
//! The `.dag` cost lens remains target-agnostic and produces `SymbolicCost`.
//! This module consumes target LanguageSpec realization rows and extracts the
//! per-primitive concrete costs that later composition slices combine with the
//! abstract symbolic shape.

use std::collections::HashMap;

use crate::dag::{Dag, DeclarationId, FieldValue, LiteralBits, ValueBody};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RealizationCostCategory {
    Type,
    Callable,
    Operator,
    Behavior,
    TypeInstantiation,
    Pattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RealizationCostKey {
    Type(DeclarationId),
    Callable(DeclarationId),
    Operator {
        target: DeclarationId,
        op: DeclarationId,
    },
    Behavior(DeclarationId),
    TypeInstantiation(DeclarationId),
    Pattern(DeclarationId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizationCostEntry {
    pub declaration: DeclarationId,
    pub language: DeclarationId,
    pub category: RealizationCostCategory,
    pub key: RealizationCostKey,
    pub cost: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizationCostTable {
    language: DeclarationId,
    entries: HashMap<RealizationCostKey, RealizationCostEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealizationCostError {
    MissingMeta(&'static str),
    MalformedRealization {
        declaration: DeclarationId,
        detail: &'static str,
    },
    DuplicateRealization {
        declaration: DeclarationId,
        key: RealizationCostKey,
    },
}

impl RealizationCostTable {
    pub fn for_language(dag: &Dag, language: DeclarationId) -> Result<Self, RealizationCostError> {
        let metas = RealizationMetas::read(dag)?;
        let mut entries = HashMap::new();

        for decl in dag.declarations() {
            let Some(meta_tag) = decl.meta_tag else {
                continue;
            };
            let Some(category) = metas.category_for(meta_tag) else {
                continue;
            };
            let Some(ValueBody::Structural { fields }) = &decl.value_body else {
                return Err(RealizationCostError::MalformedRealization {
                    declaration: decl.id,
                    detail: "realization data item has no Structural value_body",
                });
            };
            let row_language = require_decl_ref(fields, "language", decl.id)?;
            if row_language != language {
                continue;
            }
            let target = require_decl_ref(fields, "target", decl.id)?;
            let key = match category {
                RealizationCostCategory::Type => RealizationCostKey::Type(target),
                RealizationCostCategory::Callable => RealizationCostKey::Callable(target),
                RealizationCostCategory::Operator => RealizationCostKey::Operator {
                    target,
                    op: require_decl_ref(fields, "op", decl.id)?,
                },
                RealizationCostCategory::Behavior => RealizationCostKey::Behavior(target),
                RealizationCostCategory::TypeInstantiation => {
                    RealizationCostKey::TypeInstantiation(target)
                }
                RealizationCostCategory::Pattern => RealizationCostKey::Pattern(target),
            };
            let entry = RealizationCostEntry {
                declaration: decl.id,
                language,
                category,
                key,
                cost: require_int(fields, "cost", decl.id)?,
            };
            if entries.insert(key, entry).is_some() {
                return Err(RealizationCostError::DuplicateRealization {
                    declaration: decl.id,
                    key,
                });
            }
        }

        Ok(Self { language, entries })
    }

    pub fn language(&self) -> DeclarationId {
        self.language
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, key: &RealizationCostKey) -> Option<&RealizationCostEntry> {
        self.entries.get(key)
    }

    pub fn cost(&self, key: &RealizationCostKey) -> Option<i64> {
        self.get(key).map(|entry| entry.cost)
    }
}

struct RealizationMetas {
    type_meta: DeclarationId,
    callable_meta: DeclarationId,
    operator_meta: DeclarationId,
    behavior_meta: DeclarationId,
    type_instantiation_meta: DeclarationId,
    pattern_meta: DeclarationId,
}

impl RealizationMetas {
    fn read(dag: &Dag) -> Result<Self, RealizationCostError> {
        Ok(Self {
            type_meta: dag
                .type_realization_meta()
                .ok_or(RealizationCostError::MissingMeta("TypeRealization"))?,
            callable_meta: dag
                .callable_realization_meta()
                .ok_or(RealizationCostError::MissingMeta("CallableRealization"))?,
            operator_meta: dag
                .operator_realization_meta()
                .ok_or(RealizationCostError::MissingMeta("OperatorRealization"))?,
            behavior_meta: dag
                .behavior_realization_meta()
                .ok_or(RealizationCostError::MissingMeta("BehaviorRealization"))?,
            type_instantiation_meta: dag.type_instantiation_realization_meta().ok_or(
                RealizationCostError::MissingMeta("TypeInstantiationRealization"),
            )?,
            pattern_meta: dag
                .pattern_realization_meta()
                .ok_or(RealizationCostError::MissingMeta("PatternRealization"))?,
        })
    }

    fn category_for(&self, meta_tag: DeclarationId) -> Option<RealizationCostCategory> {
        if meta_tag == self.type_meta {
            Some(RealizationCostCategory::Type)
        } else if meta_tag == self.callable_meta {
            Some(RealizationCostCategory::Callable)
        } else if meta_tag == self.operator_meta {
            Some(RealizationCostCategory::Operator)
        } else if meta_tag == self.behavior_meta {
            Some(RealizationCostCategory::Behavior)
        } else if meta_tag == self.type_instantiation_meta {
            Some(RealizationCostCategory::TypeInstantiation)
        } else if meta_tag == self.pattern_meta {
            Some(RealizationCostCategory::Pattern)
        } else {
            None
        }
    }
}

fn require_field<'a>(
    fields: &'a [(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<&'a FieldValue, RealizationCostError> {
    fields
        .iter()
        .find_map(|(field_label, value)| (field_label == label).then_some(value))
        .ok_or(RealizationCostError::MalformedRealization {
            declaration,
            detail: "realization data item is missing a required field",
        })
}

fn require_decl_ref(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<DeclarationId, RealizationCostError> {
    match require_field(fields, label, declaration)? {
        FieldValue::Reference(id) => Ok(*id),
        _ => Err(RealizationCostError::MalformedRealization {
            declaration,
            detail: "realization data item field should be a DeclarationRef",
        }),
    }
}

fn require_int(
    fields: &[(String, FieldValue)],
    label: &str,
    declaration: DeclarationId,
) -> Result<i64, RealizationCostError> {
    match require_field(fields, label, declaration)? {
        FieldValue::Literal(LiteralBits::Int(value)) => Ok(*value),
        _ => Err(RealizationCostError::MalformedRealization {
            declaration,
            detail: "realization data item field should be an Int literal",
        }),
    }
}
