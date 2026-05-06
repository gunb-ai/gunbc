//! Project landed LanguageSpec-adjacent rows onto L6 cross-product cells.
//!
//! Authority at HEAD: `src/v3/std/cross_target_coverage.dag`
//! (`List<EmissionPathProjection>`) joined back to
//! `src/v3/std/{rust,python,go}_method_template_contracts.dag`
//! (`List<MethodTemplateContract>` per target) by `(target, dag_method)`.
//!
//! **Audit mapping (HEAD):** every Phase 1 row attaches to the same structural L6
//! bucket — **[`FormAxis::Cardinality`] × [`BehaviorAxis::Transform`] × target** —
//! because these rows are the Shape A emission paths for **Transform-shaped method
//! calls** on structured collection carriers (the connective axis for list-like
//! shapes is [`TypeConnective::Cardinality`] per substrate; invocation is L1
//! **Transform**, not Value/Branch/Loop/Bind). Targets split only along
//! [`ShapeATarget`].
//!
//! Future LanguageSpec tables should add row-local `EmissionCell` entries; the
//! walker unions only cells carried by rows that bijectively join to source
//! `MethodTemplateContract` identities.

use std::collections::{HashMap, HashSet};

use v3_compiler::dag::{Dag, DeclarationId, FieldValue, ValueBody};
use v3_compiler::pb_method_template_projection::{
    method_template_contract_rows, MethodTemplateTarget,
};

use crate::cells::{BehaviorAxis, Cell, FormAxis, ShapeATarget};

const EMISSION_PATH_PROJECTIONS: &str = "emission_path_projections";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProjectionCoverageError {
    ProjectionDeclarationMissing,
    ProjectionDeclarationLacksValueBody,
    ProjectionDeclarationValueBodyNotList,
    ProjectionListEmpty,
    ProjectionRowNotRecord { row_index: usize },
    ProjectionRowMissingField { row_index: usize, field: &'static str },
    ProjectionRowUnknownField { row_index: usize, field: String },
    ProjectionRowDuplicateField { row_index: usize, field: String },
    RowIdentityNotRecord { row_index: usize },
    RowIdentityTargetNotVariant { row_index: usize },
    UnknownTargetLabel { row_index: usize, label: String },
    DagMethodNotMethodRefRecord { row_index: usize },
    MethodRefDeclNotReference { row_index: usize },
    CellsNotList { row_index: usize },
    CellsEmpty { row_index: usize },
    CellNotRecord { row_index: usize, cell_index: usize },
    CellUnknownField {
        row_index: usize,
        cell_index: usize,
        field: String,
    },
    CellDuplicateField {
        row_index: usize,
        cell_index: usize,
        field: String,
    },
    CellMissingField {
        row_index: usize,
        cell_index: usize,
        field: &'static str,
    },
    CellAxisNotVariant {
        row_index: usize,
        cell_index: usize,
        field: &'static str,
    },
    UnknownFormLabel {
        row_index: usize,
        cell_index: usize,
        label: String,
    },
    UnknownBehaviorLabel {
        row_index: usize,
        cell_index: usize,
        label: String,
    },
    SourceRowsUnavailable,
    ProjectionWithoutSourceRow { row_index: usize },
    MissingProjectionForSourceRow {
        target: ShapeATarget,
        dag_method: DeclarationId,
    },
    DuplicateProjectionKeyConflictingCells {
        row_index: usize,
        target: ShapeATarget,
        dag_method: DeclarationId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProjectionKey {
    target: ShapeATarget,
    dag_method: DeclarationId,
}

#[derive(Debug, Clone)]
struct ProjectionRow {
    key: ProjectionKey,
    cells: HashSet<Cell>,
}

/// Returns the set of L6 cells for which landed per-row projections declare
/// at least one emission-path template at substrate load time.
///
/// Fail-closed: any malformed projection surface, empty projection list, empty
/// row-local cell list, projection/source mismatch, or duplicate key with
/// conflicting cells contributes **no** coverage. The walker then reports the
/// cells as typed missing coverage rather than fabricating a partial answer.
pub(crate) fn language_spec_emission_cells_covered(dag: &Dag) -> HashSet<Cell> {
    language_spec_emission_cells_covered_checked(dag).unwrap_or_default()
}

fn language_spec_emission_cells_covered_checked(
    dag: &Dag,
) -> Result<HashSet<Cell>, ProjectionCoverageError> {
    let source_keys = source_method_template_keys(dag)?;
    let projection_rows = projection_rows(dag)?;
    let projection_by_key = projection_rows_by_key(projection_rows, &source_keys)?;

    for key in &source_keys {
        if !projection_by_key.contains_key(key) {
            return Err(ProjectionCoverageError::MissingProjectionForSourceRow {
                target: key.target,
                dag_method: key.dag_method,
            });
        }
    }

    let mut covered = HashSet::new();
    for cells in projection_by_key.values() {
        covered.extend(cells.iter().copied());
    }
    Ok(covered)
}

fn source_method_template_keys(dag: &Dag) -> Result<HashSet<ProjectionKey>, ProjectionCoverageError> {
    let mut keys = HashSet::new();
    for (target, projection_target) in [
        (MethodTemplateTarget::Rust, ShapeATarget::Rust),
        (MethodTemplateTarget::Python, ShapeATarget::Python),
        (MethodTemplateTarget::Go, ShapeATarget::Go),
    ] {
        let rows = method_template_contract_rows(dag, target)
            .map_err(|_| ProjectionCoverageError::SourceRowsUnavailable)?;
        for row in rows {
            keys.insert(ProjectionKey {
                target: projection_target,
                dag_method: row.dag_method,
            });
        }
    }
    Ok(keys)
}

fn projection_rows(dag: &Dag) -> Result<Vec<ProjectionRow>, ProjectionCoverageError> {
    let decl = dag
        .declaration_by_name(EMISSION_PATH_PROJECTIONS)
        .ok_or(ProjectionCoverageError::ProjectionDeclarationMissing)?;
    let body = decl
        .value_body
        .as_ref()
        .ok_or(ProjectionCoverageError::ProjectionDeclarationLacksValueBody)?;
    let ValueBody::List(rows) = body else {
        return Err(ProjectionCoverageError::ProjectionDeclarationValueBodyNotList);
    };
    if rows.is_empty() {
        return Err(ProjectionCoverageError::ProjectionListEmpty);
    }
    rows.iter()
        .enumerate()
        .map(|(row_index, row)| projection_row(dag, row_index, row))
        .collect()
}

fn projection_rows_by_key(
    rows: Vec<ProjectionRow>,
    source_keys: &HashSet<ProjectionKey>,
) -> Result<HashMap<ProjectionKey, HashSet<Cell>>, ProjectionCoverageError> {
    let mut by_key: HashMap<ProjectionKey, HashSet<Cell>> = HashMap::new();
    for (row_index, row) in rows.into_iter().enumerate() {
        if !source_keys.contains(&row.key) {
            return Err(ProjectionCoverageError::ProjectionWithoutSourceRow { row_index });
        }
        if let Some(existing) = by_key.get(&row.key) {
            if existing != &row.cells {
                return Err(
                    ProjectionCoverageError::DuplicateProjectionKeyConflictingCells {
                        row_index,
                        target: row.key.target,
                        dag_method: row.key.dag_method,
                    },
                );
            }
            continue;
        }
        by_key.insert(row.key, row.cells);
    }
    Ok(by_key)
}

fn projection_row(
    dag: &Dag,
    row_index: usize,
    row: &FieldValue,
) -> Result<ProjectionRow, ProjectionCoverageError> {
    let FieldValue::Record(fields) = row else {
        return Err(ProjectionCoverageError::ProjectionRowNotRecord { row_index });
    };
    let lookup = field_lookup(row_index, fields, &["row_identity", "cells"])?;
    let row_identity = lookup
        .get("row_identity")
        .ok_or(ProjectionCoverageError::ProjectionRowMissingField {
            row_index,
            field: "row_identity",
        })?;
    let key = projection_key(dag, row_index, row_identity)?;
    let cells_value = lookup
        .get("cells")
        .ok_or(ProjectionCoverageError::ProjectionRowMissingField {
            row_index,
            field: "cells",
        })?;
    let cells = projection_cells(dag, row_index, key.target, cells_value)?;
    Ok(ProjectionRow { key, cells })
}

fn projection_key(
    dag: &Dag,
    row_index: usize,
    value: &FieldValue,
) -> Result<ProjectionKey, ProjectionCoverageError> {
    let FieldValue::Record(fields) = value else {
        return Err(ProjectionCoverageError::RowIdentityNotRecord { row_index });
    };
    let lookup = field_lookup(row_index, fields, &["target", "dag_method"])?;
    let target_value =
        lookup
            .get("target")
            .ok_or(ProjectionCoverageError::ProjectionRowMissingField {
                row_index,
                field: "target",
            })?;
    let target = shape_target(dag, row_index, target_value)?;
    let dag_method_value =
        lookup
            .get("dag_method")
            .ok_or(ProjectionCoverageError::ProjectionRowMissingField {
                row_index,
                field: "dag_method",
            })?;
    let dag_method = dag_method(row_index, dag_method_value)?;
    Ok(ProjectionKey { target, dag_method })
}

fn projection_cells(
    dag: &Dag,
    row_index: usize,
    target: ShapeATarget,
    value: &FieldValue,
) -> Result<HashSet<Cell>, ProjectionCoverageError> {
    let FieldValue::List(cells) = value else {
        return Err(ProjectionCoverageError::CellsNotList { row_index });
    };
    if cells.is_empty() {
        return Err(ProjectionCoverageError::CellsEmpty { row_index });
    }
    cells
        .iter()
        .enumerate()
        .map(|(cell_index, value)| projection_cell(dag, row_index, cell_index, target, value))
        .collect()
}

fn projection_cell(
    dag: &Dag,
    row_index: usize,
    cell_index: usize,
    target: ShapeATarget,
    value: &FieldValue,
) -> Result<Cell, ProjectionCoverageError> {
    let FieldValue::Record(fields) = value else {
        return Err(ProjectionCoverageError::CellNotRecord {
            row_index,
            cell_index,
        });
    };
    let lookup = cell_field_lookup(row_index, cell_index, fields, &["connective", "behavior"])?;
    let connective_value =
        lookup
            .get("connective")
            .ok_or(ProjectionCoverageError::CellMissingField {
                row_index,
                cell_index,
                field: "connective",
            })?;
    let behavior_value =
        lookup
            .get("behavior")
            .ok_or(ProjectionCoverageError::CellMissingField {
                row_index,
                cell_index,
                field: "behavior",
            })?;
    Ok(Cell {
        connective: form_axis(dag, row_index, cell_index, connective_value)?,
        behavior: behavior_axis(dag, row_index, cell_index, behavior_value)?,
        target,
    })
}

fn dag_method(row_index: usize, value: &FieldValue) -> Result<DeclarationId, ProjectionCoverageError> {
    let FieldValue::Record(fields) = value else {
        return Err(ProjectionCoverageError::DagMethodNotMethodRefRecord { row_index });
    };
    let lookup = field_lookup(row_index, fields, &["decl"])?;
    let decl = lookup
        .get("decl")
        .ok_or(ProjectionCoverageError::ProjectionRowMissingField {
            row_index,
            field: "decl",
        })?;
    let FieldValue::Reference(decl_id) = decl else {
        return Err(ProjectionCoverageError::MethodRefDeclNotReference { row_index });
    };
    Ok(*decl_id)
}

fn shape_target(
    dag: &Dag,
    row_index: usize,
    value: &FieldValue,
) -> Result<ShapeATarget, ProjectionCoverageError> {
    let label = variant_label(dag, value).ok_or(ProjectionCoverageError::RowIdentityTargetNotVariant {
        row_index,
    })?;
    match label {
        "Rust" => Ok(ShapeATarget::Rust),
        "Python" => Ok(ShapeATarget::Python),
        "Go" => Ok(ShapeATarget::Go),
        other => Err(ProjectionCoverageError::UnknownTargetLabel {
            row_index,
            label: other.to_string(),
        }),
    }
}

fn form_axis(
    dag: &Dag,
    row_index: usize,
    cell_index: usize,
    value: &FieldValue,
) -> Result<FormAxis, ProjectionCoverageError> {
    let label = variant_label(dag, value).ok_or(ProjectionCoverageError::CellAxisNotVariant {
        row_index,
        cell_index,
        field: "connective",
    })?;
    match label {
        "Atom" => Ok(FormAxis::Atom),
        "Conj" => Ok(FormAxis::Conj),
        "Disj" => Ok(FormAxis::Disj),
        "Arrow" => Ok(FormAxis::Arrow),
        "Cardinality" => Ok(FormAxis::Cardinality),
        "Instantiation" => Ok(FormAxis::Instantiation),
        other => Err(ProjectionCoverageError::UnknownFormLabel {
            row_index,
            cell_index,
            label: other.to_string(),
        }),
    }
}

fn behavior_axis(
    dag: &Dag,
    row_index: usize,
    cell_index: usize,
    value: &FieldValue,
) -> Result<BehaviorAxis, ProjectionCoverageError> {
    let label = variant_label(dag, value).ok_or(ProjectionCoverageError::CellAxisNotVariant {
        row_index,
        cell_index,
        field: "behavior",
    })?;
    match label {
        "Value" => Ok(BehaviorAxis::Value),
        "Transform" => Ok(BehaviorAxis::Transform),
        "Branch" => Ok(BehaviorAxis::Branch),
        "Loop" => Ok(BehaviorAxis::Loop),
        "Bind" => Ok(BehaviorAxis::Bind),
        other => Err(ProjectionCoverageError::UnknownBehaviorLabel {
            row_index,
            cell_index,
            label: other.to_string(),
        }),
    }
}

fn variant_label<'a>(dag: &'a Dag, value: &FieldValue) -> Option<&'a str> {
    let FieldValue::Variant { constructor, .. } = value else {
        return None;
    };
    dag.declaration_opt(constructor)
        .and_then(|decl| decl.name.as_deref())
}

fn field_lookup<'a>(
    row_index: usize,
    fields: &'a [(String, FieldValue)],
    expected: &[&'static str],
) -> Result<HashMap<&'static str, &'a FieldValue>, ProjectionCoverageError> {
    let mut lookup = HashMap::new();
    let mut seen = HashSet::new();
    for (label, value) in fields {
        let Some(field) = expected.iter().copied().find(|field| field == label) else {
            return Err(ProjectionCoverageError::ProjectionRowUnknownField {
                row_index,
                field: label.clone(),
            });
        };
        if !seen.insert(field) {
            return Err(ProjectionCoverageError::ProjectionRowDuplicateField {
                row_index,
                field: label.clone(),
            });
        }
        lookup.insert(field, value);
    }
    Ok(lookup)
}

fn cell_field_lookup<'a>(
    row_index: usize,
    cell_index: usize,
    fields: &'a [(String, FieldValue)],
    expected: &[&'static str],
) -> Result<HashMap<&'static str, &'a FieldValue>, ProjectionCoverageError> {
    let mut lookup = HashMap::new();
    let mut seen = HashSet::new();
    for (label, value) in fields {
        let Some(field) = expected.iter().copied().find(|field| field == label) else {
            return Err(ProjectionCoverageError::CellUnknownField {
                row_index,
                cell_index,
                field: label.clone(),
            });
        };
        if !seen.insert(field) {
            return Err(ProjectionCoverageError::CellDuplicateField {
                row_index,
                cell_index,
                field: label.clone(),
            });
        }
        lookup.insert(field, value);
    }
    Ok(lookup)
}
