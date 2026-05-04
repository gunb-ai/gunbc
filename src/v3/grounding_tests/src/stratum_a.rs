//! Stratum A — name-keyed routing parity for `MethodTemplateContract` Phase 1 rows.
//!
//! Source: `docs/briefs/t-ground-tests.md` (test plan item 1). Generalizes the pilot
//! kernel-integer checks (`src/v3/grounding_pilot/src/lib.rs:408-460`) onto the
//! per-target row lists `rust_*` / `python_*` / `go_*` from T-Ground-LanguageSpec #1195.
//!
//! **Substrate read path:** Stratum A walks [`v3_compiler::generated_full_bootstrap_dag`] only.
//! This crate intentionally does **not** `include_str!` the `src/v3/std/*_method_template_contracts.dag`
//! sources from here: a path relative to `grounding_tests/src/` is easy to mis-write as `../std/…`
//! (which resolves under `src/v3/grounding_tests/std/`, not `src/v3/std/`). The compiler’s embedded
//! snapshot is the single structural authority for these rows.

use std::collections::{BTreeMap, BTreeSet};

use v3_compiler::dag::{Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};

use crate::diagnostic::GroundingTestsDiagnostic;

const RUST_LIST: &str = "rust_method_template_contracts";
const PYTHON_LIST: &str = "python_method_template_contracts";
const GO_LIST: &str = "go_method_template_contracts";

/// Closed schema for `MethodTemplateContract` rows (`emit_model.dag`) — Stratum A must reject
/// duplicate labels, missing keys, and unconsumed extra fields.
const METHOD_TEMPLATE_CONTRACT_FIELDS: &[&str] = &[
    "dag_method",
    "emit_template",
    "placeholder_convention",
    "runtime_template",
    "wraps_result",
];

/// `MethodRef` (`methods.dag`) — single `decl` field, no silent ignores.
const METHOD_REF_FIELDS: &[&str] = &["decl"];

/// Director-locked Phase 1 row counts (`t-ground-tests.md`; bump when `*_method_template_contracts.dag` grows).
///
/// **List scope only:** these counts cover `data *_method_template_contracts: List<MethodTemplateContract>`.
/// Per-target `CollectionOps.*_contract` carriers promoted to named
/// `MethodTemplateContract` declarations outside the per-target lists are
/// **not** list elements — `verify_stratum_a_lockstep_all_targets` closes that
/// receipt separately (`verify_language_spec_collection_ops_contract_wiring`).
pub const EXPECTED_STRATUM_A_ROW_COUNTS: &[(&str, usize)] =
    &[(RUST_LIST, 13), (PYTHON_LIST, 18), (GO_LIST, 14)];

/// (`CollectionOps` decl, field name, named `MethodTemplateContract` decl, expected registry method name).
const LANGUAGE_SPEC_COLLECTION_OPS_CONTRACT_WITNESSES: &[(&str, &str, &str, &str)] = &[
    (
        "rust_collection_ops",
        "concat_contract",
        "rust_language_spec_free_monoid_concat_contract",
        "concat",
    ),
    (
        "rust_collection_ops",
        "length_contract",
        "rust_language_spec_free_monoid_length_contract",
        "length",
    ),
    (
        "rust_collection_ops",
        "is_empty_contract",
        "rust_language_spec_free_monoid_emptiness_contract",
        "length",
    ),
    (
        "rust_collection_ops",
        "fold_contract",
        "rust_language_spec_free_monoid_fold_contract",
        "fold",
    ),
    (
        "python_collections",
        "concat_contract",
        "python_language_spec_free_monoid_concat_contract",
        "concat",
    ),
    (
        "python_collections",
        "length_contract",
        "python_language_spec_free_monoid_length_contract",
        "length",
    ),
    (
        "python_collections",
        "is_empty_contract",
        "python_language_spec_free_monoid_emptiness_contract",
        "length",
    ),
    (
        "python_collections",
        "fold_contract",
        "python_language_spec_free_monoid_fold_contract",
        "fold",
    ),
    (
        "go_collection_ops",
        "concat_contract",
        "go_language_spec_free_monoid_concat_contract",
        "concat",
    ),
    (
        "go_collection_ops",
        "length_contract",
        "go_language_spec_free_monoid_length_contract",
        "length",
    ),
    (
        "go_collection_ops",
        "is_empty_contract",
        "go_language_spec_free_monoid_emptiness_contract",
        "length",
    ),
    (
        "go_collection_ops",
        "fold_contract",
        "go_language_spec_free_monoid_fold_contract",
        "fold",
    ),
];

/// Fail-closed witness: each target `CollectionOps.*_contract` ref resolves to the
/// named `MethodTemplateContract` carrier with the expected `dag_method` registry name.
fn verify_language_spec_collection_ops_contract_wiring(
    dag: &Dag,
) -> Result<(), GroundingTestsDiagnostic> {
    let mtc_id = dag
        .declaration_by_name("MethodTemplateContract")
        .ok_or_else(|| GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "collection_ops_contract_witness.MethodTemplateContract",
            detail: "MethodTemplateContract missing from Dag".to_string(),
        })?
        .id;

    for &(collection_ops_name, field, contract_decl_name, expected_method) in
        LANGUAGE_SPEC_COLLECTION_OPS_CONTRACT_WITNESSES
    {
        let cs_decl = dag
            .declaration_by_name(collection_ops_name)
            .ok_or_else(|| GroundingTestsDiagnostic::StratumADagProjectionFailed {
                step: "collection_ops_contract_witness.collection_ops",
                detail: format!("missing `{collection_ops_name}`"),
            })?;
        let vb = cs_decl.value_body.as_ref().ok_or_else(|| {
            GroundingTestsDiagnostic::StratumADagProjectionFailed {
                step: "collection_ops_contract_witness.collection_ops.value_body",
                detail: format!("`{collection_ops_name}` has no value_body"),
            }
        })?;
        let ValueBody::Structural { fields: cs_fields } = vb else {
            return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                step: "collection_ops_contract_witness.collection_ops.shape",
                detail: format!("`{collection_ops_name}`: expected Structural value body"),
            });
        };
        let contract_ref = row_field(cs_fields, collection_ops_name, 0, field)?;
        let FieldValue::Reference(expected_id) = contract_ref else {
            return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: collection_ops_name.to_string(),
                row_index: 0,
                detail: format!("`{field}` must be DeclarationRef, got {contract_ref:?}"),
            });
        };

        let contract_decl = dag.declaration_by_name(contract_decl_name).ok_or_else(|| {
            GroundingTestsDiagnostic::StratumADagProjectionFailed {
                step: "collection_ops_contract_witness.named_decl",
                detail: format!(
                    "missing `{contract_decl_name}` (CollectionOps `{field}` ref {expected_id:?})"
                ),
            }
        })?;
        if contract_decl.id != *expected_id {
            return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: collection_ops_name.to_string(),
                row_index: 0,
                detail: format!(
                    "`{field}` ref {expected_id:?} does not match declaration `{contract_decl_name}` ({:?})",
                    contract_decl.id
                ),
            });
        }

        let template = match &contract_decl.connective {
            TypeConnective::Instantiation { template, .. } => *template,
            other => {
                return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                    list_name: contract_decl_name.to_string(),
                    row_index: 0,
                    detail: format!("expected MethodTemplateContract instantiation, got {other:?}"),
                });
            }
        };
        if template != mtc_id {
            return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: contract_decl_name.to_string(),
                row_index: 0,
                detail: format!(
                    "expected template MethodTemplateContract ({mtc_id:?}), got {template:?}"
                ),
            });
        }

        let contract_body = contract_decl.value_body.as_ref().ok_or_else(|| {
            GroundingTestsDiagnostic::StratumADagProjectionFailed {
                step: "collection_ops_contract_witness.named_decl.value_body",
                detail: format!("`{contract_decl_name}` missing value_body"),
            }
        })?;
        let ValueBody::Structural {
            fields: contract_fields,
        } = contract_body
        else {
            return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                step: "collection_ops_contract_witness.named_decl.shape",
                detail: format!(
                    "`{contract_decl_name}`: expected Structural MethodTemplateContract data"
                ),
            });
        };
        let row = FieldValue::Record(contract_fields.clone());
        let fp = row_fingerprint(dag, contract_decl_name, 0, &row)?;
        if fp.method_name != expected_method {
            return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: contract_decl_name.to_string(),
                row_index: 0,
                detail: format!(
                    "expected dag_method registry name `{expected_method}` for `{field}`, got `{}`",
                    fp.method_name
                ),
            });
        }
    }
    Ok(())
}

fn list_rows<'a>(
    dag: &'a Dag,
    list_name: &str,
) -> Result<&'a [FieldValue], GroundingTestsDiagnostic> {
    let decl = dag.declaration_by_name(list_name).ok_or_else(|| {
        GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "list_rows.declaration_by_name",
            detail: format!("missing declaration `{list_name}`"),
        }
    })?;
    let body = decl.value_body.as_ref().ok_or_else(|| {
        GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "list_rows.value_body",
            detail: format!("`{list_name}` has no value body"),
        }
    })?;
    match body {
        ValueBody::List(rows) => Ok(rows.as_slice()),
        other => Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "list_rows.value_body",
            detail: format!("`{list_name}`: expected List body, got {other:?}"),
        }),
    }
}

fn method_emit_template_variant_label(
    dag: &Dag,
    constructor: DeclarationId,
) -> Result<String, GroundingTestsDiagnostic> {
    let method_emit_template = dag
        .declaration_by_name("MethodEmitTemplate")
        .ok_or_else(|| GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "MethodEmitTemplate.declaration_by_name",
            detail: "MethodEmitTemplate missing from Dag".to_string(),
        })?;
    let TypeConnective::Disj { variants } = &method_emit_template.connective else {
        return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "MethodEmitTemplate.connective",
            detail: format!("expected Disj, got {:?}", method_emit_template.connective),
        });
    };
    variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .map(|v| v.label.clone())
        .ok_or_else(|| GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "MethodEmitTemplate.variant",
            detail: format!("unknown MethodEmitTemplate constructor {constructor:?}"),
        })
}

fn placeholder_variant_label(
    dag: &Dag,
    constructor: DeclarationId,
) -> Result<String, GroundingTestsDiagnostic> {
    let root = dag
        .declaration_by_name("PlaceholderConvention")
        .ok_or_else(|| GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "PlaceholderConvention.declaration_by_name",
            detail: "PlaceholderConvention missing from Dag".to_string(),
        })?;
    let TypeConnective::Disj { variants } = &root.connective else {
        return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "PlaceholderConvention.connective",
            detail: format!("expected Disj, got {:?}", root.connective),
        });
    };
    variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .map(|v| v.label.clone())
        .ok_or_else(|| GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "PlaceholderConvention.variant",
            detail: format!("unknown PlaceholderConvention constructor {constructor:?}"),
        })
}

fn method_declaration_template_id(dag: &Dag) -> Result<DeclarationId, String> {
    dag.declaration_by_name("MethodDeclaration")
        .map(|d| d.id)
        .ok_or_else(|| "MethodDeclaration missing from Dag".to_string())
}

/// Resolve a `MethodRef.decl` target to the method's registry `name` literal.
///
/// Fail-closed: the referenced declaration must **instantiate** `MethodDeclaration`
/// (same structural gate as `method_registry_covers_all_algebra_template_names`), not merely
/// carry a string `name` field on an arbitrary record. The value body must be a closed record
/// with exactly the `name` field (no duplicates, no extra keys).
fn method_registry_name(
    dag: &Dag,
    method_decl_id: DeclarationId,
    list_name: &str,
    row_index: usize,
) -> Result<String, GroundingTestsDiagnostic> {
    let method_decl_template = method_declaration_template_id(dag).map_err(|e| {
        GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: e,
        }
    })?;
    let decl = dag.declaration(method_decl_id);
    let template = match &decl.connective {
        TypeConnective::Instantiation { template, .. } => *template,
        other => {
            return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: format!(
                    "declaration {:?}: expected MethodDeclaration instantiation (Instantiation connective), got {other:?}",
                    decl.name
                ),
            });
        }
    };
    if template != method_decl_template {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!(
                "declaration {:?}: instantiates template {template:?}, expected MethodDeclaration ({method_decl_template:?})",
                decl.name
            ),
        });
    }
    let vb = decl.value_body.as_ref().ok_or_else(|| {
        GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("declaration {:?} has no value_body", decl.name),
        }
    })?;
    let ValueBody::Structural { fields } = vb else {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!(
                "declaration {:?}: value_body not Structural: {vb:?}",
                decl.name
            ),
        });
    };
    enforce_closed_record_schema(
        fields,
        list_name,
        row_index,
        "MethodDeclaration.value_body",
        &["name"],
    )?;
    let name_field = row_field(fields, list_name, row_index, "name")?;
    match name_field {
        FieldValue::Literal(LiteralBits::String(s)) => Ok(s.clone()),
        other => Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!(
                "declaration {:?}: `name` not a string literal: {other:?}",
                decl.name
            ),
        }),
    }
}

/// Phase-1 `PlaceholderConvention` (`emit_model.dag`): nullary variants only; payload must be
/// empty so digest/fingerprint cannot collapse distinct malformed shapes.
fn placeholder_convention_canonical(
    dag: &Dag,
    ph: &FieldValue,
) -> Result<String, GroundingTestsDiagnostic> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = ph
    else {
        return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "placeholder_convention_canonical.shape",
            detail: format!("placeholder_convention not a variant: {ph:?}"),
        });
    };
    let label = placeholder_variant_label(dag, *constructor)?;
    match label.as_str() {
        "IndexedArgs" | "NamedArg" => {
            if !payload.is_empty() {
                return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                    step: "placeholder_convention_canonical.arity",
                    detail: format!(
                        "variant `{label}` is nullary in Phase 1; got {} payload field(s): {payload:?}",
                        payload.len()
                    ),
                });
            }
            Ok(label)
        }
        _ => Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "placeholder_convention_canonical.variant",
            detail: format!("unknown PlaceholderConvention variant `{label}`"),
        }),
    }
}

fn emit_template_canonical(
    dag: &Dag,
    emit: &FieldValue,
) -> Result<String, GroundingTestsDiagnostic> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = emit
    else {
        return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "emit_template_canonical.shape",
            detail: format!("emit_template not a variant: {emit:?}"),
        });
    };
    let label = method_emit_template_variant_label(dag, *constructor)?;
    match label.as_str() {
        "SingleTemplate" => {
            if payload.len() != 1 {
                return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                    step: "emit_template_canonical.SingleTemplate",
                    detail: format!(
                        "expected 1 payload field, got {}: {payload:?}",
                        payload.len()
                    ),
                });
            }
            let FieldValue::Literal(LiteralBits::String(s)) = &payload[0] else {
                return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                    step: "emit_template_canonical.SingleTemplate",
                    detail: format!("expected String literal payload: {:?}", payload[0]),
                });
            };
            Ok(format!("SingleTemplate({s})"))
        }
        "HigherOrderTemplates" => {
            if payload.len() != 2 {
                return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                    step: "emit_template_canonical.HigherOrderTemplates",
                    detail: format!("expected 2 payload fields, got {payload:?}"),
                });
            }
            let FieldValue::Literal(LiteralBits::String(inline)) = &payload[0] else {
                return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                    step: "emit_template_canonical.inline_template",
                    detail: format!("{:?}", payload[0]),
                });
            };
            let FieldValue::Literal(LiteralBits::String(fn_ref)) = &payload[1] else {
                return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                    step: "emit_template_canonical.fn_ref_template",
                    detail: format!("{:?}", payload[1]),
                });
            };
            Ok(format!("HigherOrderTemplates({inline}|{fn_ref})"))
        }
        _ => Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "emit_template_canonical.variant",
            detail: format!("unknown MethodEmitTemplate variant `{label}`"),
        }),
    }
}

/// One `MethodTemplateContract` row projected to deterministic strings for parity / digests.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RowFingerprint {
    pub method_name: String,
    pub runtime_template: String,
    pub emit_canonical: String,
    pub wraps_result: bool,
    pub placeholder: String,
}

impl RowFingerprint {
    pub fn canonical_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}",
            self.method_name,
            self.runtime_template,
            self.emit_canonical,
            self.wraps_result,
            self.placeholder
        )
    }
}

fn row_record<'a>(
    row: &'a FieldValue,
    list_name: &str,
    row_index: usize,
) -> Result<&'a [(String, FieldValue)], GroundingTestsDiagnostic> {
    let FieldValue::Record(fields) = row else {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("row is not a record: {row:?}"),
        });
    };
    Ok(fields.as_slice())
}

/// Label-set outcome for a closed record row (duplicate labels vs allowed set mismatch vs OK).
#[derive(Debug, PartialEq, Eq)]
enum ClosedRecordLabelsOutcome {
    Ok,
    Duplicate {
        label: String,
    },
    Mismatch {
        missing: Vec<String>,
        extra: Vec<String>,
    },
}

/// Single source of truth for closed-record label discipline (see `enforce_closed_record_schema`).
fn analyze_closed_record_labels(
    fields: &[(String, FieldValue)],
    allowed: &[&str],
) -> ClosedRecordLabelsOutcome {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for (label, _) in fields {
        if !seen.insert(label.as_str()) {
            return ClosedRecordLabelsOutcome::Duplicate {
                label: label.clone(),
            };
        }
    }
    let expected: BTreeSet<&str> = allowed.iter().copied().collect();
    if seen == expected {
        return ClosedRecordLabelsOutcome::Ok;
    }
    let missing: Vec<String> = expected
        .difference(&seen)
        .copied()
        .map(str::to_string)
        .collect();
    let extra: Vec<String> = seen
        .difference(&expected)
        .copied()
        .map(str::to_string)
        .collect();
    ClosedRecordLabelsOutcome::Mismatch { missing, extra }
}

/// Fail-closed record shape: no duplicate labels, no unknown fields — every substrate key in
/// `allowed` appears exactly once (order-independent).
fn enforce_closed_record_schema(
    fields: &[(String, FieldValue)],
    list_name: &str,
    row_index: usize,
    record_kind: &'static str,
    allowed: &[&str],
) -> Result<(), GroundingTestsDiagnostic> {
    match analyze_closed_record_labels(fields, allowed) {
        ClosedRecordLabelsOutcome::Ok => Ok(()),
        ClosedRecordLabelsOutcome::Duplicate { label } => {
            Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: format!("{record_kind}: duplicate field `{label}`"),
            })
        }
        ClosedRecordLabelsOutcome::Mismatch { missing, extra } => {
            let expected: BTreeSet<&str> = allowed.iter().copied().collect();
            Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: format!(
                    "{record_kind}: field set mismatch — missing {missing:?}, extra {extra:?} (expected exactly {:?})",
                    expected.iter().collect::<Vec<_>>()
                ),
            })
        }
    }
}

fn row_field<'a>(
    fields: &'a [(String, FieldValue)],
    list_name: &str,
    row_index: usize,
    label: &'static str,
) -> Result<&'a FieldValue, GroundingTestsDiagnostic> {
    let mut matches = fields.iter().filter(|(l, _)| l == label);
    let (_, v) = matches.next().ok_or_else(|| {
        GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("missing `{label}`"),
        }
    })?;
    if matches.next().is_some() {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("duplicate field `{label}`"),
        });
    }
    Ok(v)
}

fn method_name_from_dag_method(
    dag: &Dag,
    dag_method: &FieldValue,
    list_name: &str,
    row_index: usize,
) -> Result<String, GroundingTestsDiagnostic> {
    let FieldValue::Record(method_ref_fields) = dag_method else {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("`dag_method` not MethodRef record: {dag_method:?}"),
        });
    };
    enforce_closed_record_schema(
        method_ref_fields,
        list_name,
        row_index,
        "MethodRef",
        METHOD_REF_FIELDS,
    )?;
    let decl_field = row_field(method_ref_fields, list_name, row_index, "decl")?;
    let FieldValue::Reference(method_decl_id) = decl_field else {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("MethodRef.decl not a reference: {decl_field:?}"),
        });
    };
    method_registry_name(dag, *method_decl_id, list_name, row_index)
}

fn row_fingerprint(
    dag: &Dag,
    list_name: &str,
    row_index: usize,
    row: &FieldValue,
) -> Result<RowFingerprint, GroundingTestsDiagnostic> {
    let fields = row_record(row, list_name, row_index)?;
    enforce_closed_record_schema(
        fields,
        list_name,
        row_index,
        "MethodTemplateContract",
        METHOD_TEMPLATE_CONTRACT_FIELDS,
    )?;
    let dag_method = row_field(fields, list_name, row_index, "dag_method")?;
    let method_name = method_name_from_dag_method(dag, dag_method, list_name, row_index)?;

    let runtime_field = row_field(fields, list_name, row_index, "runtime_template")?;
    let runtime_template = match runtime_field {
        FieldValue::Literal(LiteralBits::String(s)) => s.clone(),
        other => {
            return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: format!("runtime_template not string: {other:?}"),
            });
        }
    };

    let emit_field = row_field(fields, list_name, row_index, "emit_template")?;
    let emit_canonical = emit_template_canonical(dag, emit_field)?;

    let wraps_field = row_field(fields, list_name, row_index, "wraps_result")?;
    let wraps_result = match wraps_field {
        FieldValue::Literal(LiteralBits::Bool(b)) => *b,
        other => {
            return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: format!("wraps_result not bool: {other:?}"),
            });
        }
    };

    let ph_field = row_field(fields, list_name, row_index, "placeholder_convention")?;
    let placeholder = placeholder_convention_canonical(dag, ph_field).map_err(|e| match e {
        GroundingTestsDiagnostic::StratumADagProjectionFailed { step, detail } => {
            GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: format!("{step}: {detail}"),
            }
        }
        other => other,
    })?;

    Ok(RowFingerprint {
        method_name,
        runtime_template,
        emit_canonical,
        wraps_result,
        placeholder,
    })
}

fn fingerprints_for_list(
    dag: &Dag,
    list_name: &str,
) -> Result<BTreeMap<String, RowFingerprint>, GroundingTestsDiagnostic> {
    let rows = list_rows(dag, list_name)?;
    let mut out: BTreeMap<String, RowFingerprint> = BTreeMap::new();
    for (idx, row) in rows.iter().enumerate() {
        let fp = row_fingerprint(dag, list_name, idx, row)?;
        if out.insert(fp.method_name.clone(), fp).is_some() {
            return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index: idx,
                detail: format!("duplicate method key in `{list_name}` rows"),
            });
        }
    }
    Ok(out)
}

fn list_digest_from_fingerprints(map: &BTreeMap<String, RowFingerprint>) -> String {
    map.values()
        .map(|fp| fp.canonical_line())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sorted multiset digest — invariant under row iteration order (Stratum A determinism).
pub fn stratum_a_list_digest(
    dag: &Dag,
    list_name: &str,
) -> Result<String, GroundingTestsDiagnostic> {
    let fps = fingerprints_for_list(dag, list_name)?;
    Ok(list_digest_from_fingerprints(&fps))
}

fn assert_expected_row_count(
    list_name: &str,
    actual: usize,
) -> Result<(), GroundingTestsDiagnostic> {
    let expected = EXPECTED_STRATUM_A_ROW_COUNTS
        .iter()
        .find(|(n, _)| *n == list_name)
        .map(|(_, c)| *c)
        .ok_or_else(|| GroundingTestsDiagnostic::StratumADagProjectionFailed {
            step: "assert_expected_row_count.unknown_list",
            detail: format!(
                "list `{list_name}` has no Director-locked expected row count; known: {}",
                EXPECTED_STRATUM_A_ROW_COUNTS
                    .iter()
                    .map(|(n, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        })?;
    if actual != expected {
        return Err(GroundingTestsDiagnostic::StratumARowCountMismatch {
            list_name: list_name.to_string(),
            expected,
            actual,
        });
    }
    Ok(())
}

/// Stratum A: Phase 1 `MethodTemplateContract` rows in the full bootstrap Dag.
///
/// Asserts Director-locked row counts, resolves every row’s `dag_method` through the
/// `MethodDeclaration` registry, and verifies two independent
/// [`v3_compiler::generated_full_bootstrap_dag`] runs produce **bit-identical** digests per
/// list (routing projection is a pure function of the embedded snapshot).
///
/// Also asserts Rust/Python/Go `CollectionOps.{concat,length,is_empty,fold}_contract` wiring to the
/// named `MethodTemplateContract` carriers (list-external authority; `is_empty_contract` pins `length_method`).
pub fn verify_stratum_a_lockstep_all_targets() -> Result<(), GroundingTestsDiagnostic> {
    let bootstrap_a = v3_compiler::generated_full_bootstrap_dag();
    let bootstrap_b = v3_compiler::generated_full_bootstrap_dag();
    for list_name in [RUST_LIST, PYTHON_LIST, GO_LIST] {
        let n = list_rows(&bootstrap_a, list_name)?.len();
        assert_expected_row_count(list_name, n)?;
        let n_b = list_rows(&bootstrap_b, list_name)?.len();
        if n_b != n {
            return Err(GroundingTestsDiagnostic::StratumARowCountMismatch {
                list_name: list_name.to_string(),
                expected: n,
                actual: n_b,
            });
        }
        let digest_a = stratum_a_list_digest(&bootstrap_a, list_name)?;
        let digest_b = stratum_a_list_digest(&bootstrap_b, list_name)?;
        if digest_a != digest_b {
            return Err(GroundingTestsDiagnostic::StratumALockstepListDigestMismatch {
                list_name: list_name.to_string(),
                detail:
                    "two `generated_full_bootstrap_dag()` runs produced different Stratum-A digests"
                        .to_string(),
            });
        }
    }
    verify_language_spec_collection_ops_contract_wiring(&bootstrap_a)?;
    Ok(())
}

#[cfg(test)]
mod stratum_a_tests {
    use v3_compiler::generated_full_bootstrap_dag;

    use super::*;

    fn method_template_contract_row_fields_with_extra_surprise() -> Vec<(String, FieldValue)> {
        let dummy = FieldValue::Literal(LiteralBits::Bool(false));
        let mut fields: Vec<(String, FieldValue)> = METHOD_TEMPLATE_CONTRACT_FIELDS
            .iter()
            .map(|&label| (label.to_string(), dummy.clone()))
            .collect();
        fields.push(("surprise".to_string(), dummy));
        fields
    }

    #[test]
    fn determinism_forward_vs_reverse_row_walk_before_btree_keying() {
        let dag = generated_full_bootstrap_dag();
        let rows = list_rows(&dag, RUST_LIST).expect("rust list");
        let mut forward = BTreeMap::new();
        for (idx, row) in rows.iter().enumerate() {
            let fp = row_fingerprint(&dag, RUST_LIST, idx, row).expect("fp");
            forward.insert(fp.method_name.clone(), fp);
        }
        let mut backward = BTreeMap::new();
        for (idx, row) in rows.iter().enumerate().rev() {
            let fp = row_fingerprint(&dag, RUST_LIST, idx, row).expect("fp");
            backward.insert(fp.method_name.clone(), fp);
        }
        assert_eq!(forward, backward);
        assert_eq!(
            list_digest_from_fingerprints(&forward),
            stratum_a_list_digest(&dag, RUST_LIST).expect("digest")
        );
    }

    #[test]
    fn assert_expected_row_count_unknown_list_is_not_silent_zero_expected() {
        let err = super::assert_expected_row_count("typo_method_template_contracts", 0)
            .expect_err("unknown list must not default to expected=0");
        assert!(
            matches!(
                &err,
                GroundingTestsDiagnostic::StratumADagProjectionFailed { step, .. }
                    if *step == "assert_expected_row_count.unknown_list"
            ),
            "unexpected diagnostic: {err}"
        );
    }

    #[test]
    fn enforce_closed_record_schema_rejects_duplicate_method_ref_field() {
        let fields = vec![
            (
                "decl".to_string(),
                FieldValue::Literal(LiteralBits::Bool(false)),
            ),
            (
                "decl".to_string(),
                FieldValue::Literal(LiteralBits::Bool(true)),
            ),
        ];
        assert_eq!(
            analyze_closed_record_labels(&fields, METHOD_REF_FIELDS),
            ClosedRecordLabelsOutcome::Duplicate {
                label: "decl".to_string(),
            }
        );
        let err =
            enforce_closed_record_schema(&fields, RUST_LIST, 0, "MethodRef", METHOD_REF_FIELDS)
                .expect_err("duplicate decl");
        assert!(
            matches!(
                &err,
                GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                    list_name,
                    row_index,
                    ..
                } if list_name == RUST_LIST && *row_index == 0
            ),
            "unexpected diagnostic: {err:?}"
        );
    }

    #[test]
    fn analyze_closed_record_labels_flags_extra_surprise_with_empty_missing() {
        let fields = method_template_contract_row_fields_with_extra_surprise();
        assert_eq!(
            analyze_closed_record_labels(&fields, METHOD_TEMPLATE_CONTRACT_FIELDS),
            ClosedRecordLabelsOutcome::Mismatch {
                missing: vec![],
                extra: vec!["surprise".to_string()],
            }
        );
    }

    #[test]
    fn enforce_closed_record_schema_rejects_extra_method_template_contract_field() {
        let fields = method_template_contract_row_fields_with_extra_surprise();
        let err = enforce_closed_record_schema(
            &fields,
            RUST_LIST,
            0,
            "MethodTemplateContract",
            METHOD_TEMPLATE_CONTRACT_FIELDS,
        )
        .expect_err("extra field");
        assert!(
            matches!(
                &err,
                GroundingTestsDiagnostic::StratumARegistryResolutionFailed { .. }
            ),
            "unexpected diagnostic: {err:?}"
        );
    }

    #[test]
    fn placeholder_convention_canonical_rejects_payload_for_nullary_indexed_args() {
        let dag = generated_full_bootstrap_dag();
        let root = dag
            .declaration_by_name("PlaceholderConvention")
            .expect("PlaceholderConvention");
        let TypeConnective::Disj { variants } = &root.connective else {
            panic!("expected Disj, got {:?}", root.connective);
        };
        let indexed = variants
            .iter()
            .find(|v| v.label == "IndexedArgs")
            .expect("IndexedArgs variant");
        let ph = FieldValue::Variant {
            constructor: indexed.ty,
            payload: vec![FieldValue::Literal(LiteralBits::Bool(false))],
        };
        let err = placeholder_convention_canonical(&dag, &ph).expect_err("non-empty payload");
        assert!(
            matches!(
                &err,
                GroundingTestsDiagnostic::StratumADagProjectionFailed { step, .. }
                    if *step == "placeholder_convention_canonical.arity"
            ),
            "unexpected diagnostic: {err:?}"
        );
    }

    #[test]
    fn method_registry_name_rejects_non_method_declaration_instantiation() {
        let dag = generated_full_bootstrap_dag();
        let decl = dag.declaration_by_name("Int").expect("Int");
        let method_decl_template =
            method_declaration_template_id(&dag).expect("MethodDeclaration template");
        let int_is_not_method_declaration_instantiation = match &decl.connective {
            TypeConnective::Instantiation { template, .. } => *template != method_decl_template,
            _ => true,
        };
        assert!(
            int_is_not_method_declaration_instantiation,
            "regression witness: Int must not instantiate MethodDeclaration; connective={:?}",
            decl.connective
        );
        let err = method_registry_name(&dag, decl.id, RUST_LIST, 0)
            .expect_err("Int is not MethodDeclaration");
        assert!(
            matches!(
                &err,
                GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                    list_name,
                    row_index,
                    ..
                } if list_name == RUST_LIST && *row_index == 0
            ),
            "unexpected diagnostic: {err:?}"
        );
    }

    #[test]
    fn stratum_a_language_spec_collection_ops_contract_wiring_ok() {
        let dag = generated_full_bootstrap_dag();
        super::verify_language_spec_collection_ops_contract_wiring(&dag)
            .unwrap_or_else(|e| panic!("CollectionOps contract witness: {e}"));
    }

    #[test]
    fn stratum_a_phase1_bootstrap_verification() {
        verify_stratum_a_lockstep_all_targets().unwrap_or_else(|e| panic!("{e}"));
    }

    #[test]
    fn stratum_a_row_counts_match_director_phase1() {
        let dag = generated_full_bootstrap_dag();
        for &(name, expected) in EXPECTED_STRATUM_A_ROW_COUNTS {
            let n = list_rows(&dag, name).expect("list").len();
            assert_eq!(
                n, expected,
                "`{name}` row count drift — update EXPECTED_STRATUM_A_ROW_COUNTS or substrate"
            );
        }
    }

    #[test]
    fn stratum_a_digest_idempotent_on_full_bootstrap() {
        let dag = generated_full_bootstrap_dag();
        let d1 = stratum_a_list_digest(&dag, RUST_LIST).expect("d1");
        let d2 = stratum_a_list_digest(&dag, RUST_LIST).expect("d2");
        assert_eq!(d1, d2);
    }
}
