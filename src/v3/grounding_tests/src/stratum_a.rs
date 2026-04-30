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

use std::collections::BTreeMap;

use v3_compiler::dag::{Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};

use crate::diagnostic::GroundingTestsDiagnostic;

const RUST_LIST: &str = "rust_method_template_contracts";
const PYTHON_LIST: &str = "python_method_template_contracts";
const GO_LIST: &str = "go_method_template_contracts";

/// Director-locked Phase 1 row counts (`t-ground-tests.md`; bump when `*_method_template_contracts.dag` grows).
pub const EXPECTED_STRATUM_A_ROW_COUNTS: &[(&str, usize)] =
    &[(RUST_LIST, 13), (PYTHON_LIST, 18), (GO_LIST, 14)];

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

fn method_registry_name(dag: &Dag, method_decl_id: DeclarationId) -> Result<String, String> {
    let decl = dag.declaration(method_decl_id);
    let vb = decl
        .value_body
        .as_ref()
        .ok_or_else(|| format!("declaration {:?} has no value_body", decl.name))?;
    let ValueBody::Structural { fields } = vb else {
        return Err(format!(
            "declaration {:?}: value_body not Structural: {vb:?}",
            decl.name
        ));
    };
    let name_field = fields
        .iter()
        .find(|(label, _)| label == "name")
        .ok_or_else(|| format!("declaration {:?}: missing `name` field", decl.name))?;
    match &name_field.1 {
        FieldValue::Literal(LiteralBits::String(s)) => Ok(s.clone()),
        other => Err(format!(
            "declaration {:?}: `name` not a string literal: {other:?}",
            decl.name
        )),
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
            let Some(FieldValue::Literal(LiteralBits::String(s))) = payload.first() else {
                return Err(GroundingTestsDiagnostic::StratumADagProjectionFailed {
                    step: "emit_template_canonical.SingleTemplate",
                    detail: format!("payload: {payload:?}"),
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

fn row_fingerprint(
    dag: &Dag,
    list_name: &str,
    row_index: usize,
    row: &FieldValue,
) -> Result<RowFingerprint, GroundingTestsDiagnostic> {
    let list = list_name.to_string();
    let FieldValue::Record(fields) = row else {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list,
            row_index,
            detail: format!("row is not a record: {row:?}"),
        });
    };
    let (_, dag_method) = fields
        .iter()
        .find(|(label, _)| label == "dag_method")
        .ok_or_else(
            || GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: "missing `dag_method`".to_string(),
            },
        )?;
    let FieldValue::Record(method_ref_fields) = dag_method else {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("`dag_method` not MethodRef record: {dag_method:?}"),
        });
    };
    let (_, decl_field) = method_ref_fields
        .iter()
        .find(|(label, _)| label == "decl")
        .ok_or_else(
            || GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: "MethodRef missing `decl`".to_string(),
            },
        )?;
    let FieldValue::Reference(method_decl_id) = decl_field else {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("MethodRef.decl not a reference: {decl_field:?}"),
        });
    };
    let method_name = method_registry_name(dag, *method_decl_id).map_err(|e| {
        GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: e,
        }
    })?;

    let (_, runtime_field) = fields
        .iter()
        .find(|(label, _)| label == "runtime_template")
        .ok_or_else(
            || GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: "missing `runtime_template`".to_string(),
            },
        )?;
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

    let (_, emit_field) = fields
        .iter()
        .find(|(label, _)| label == "emit_template")
        .ok_or_else(
            || GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: "missing `emit_template`".to_string(),
            },
        )?;
    let emit_canonical = emit_template_canonical(dag, emit_field)?;

    let (_, wraps_field) = fields
        .iter()
        .find(|(label, _)| label == "wraps_result")
        .ok_or_else(
            || GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: "missing `wraps_result`".to_string(),
            },
        )?;
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

    let (_, ph_field) = fields
        .iter()
        .find(|(label, _)| label == "placeholder_convention")
        .ok_or_else(
            || GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index,
                detail: "missing `placeholder_convention`".to_string(),
            },
        )?;
    let FieldValue::Variant {
        constructor: ph_ctor,
        ..
    } = ph_field
    else {
        return Err(GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: format!("placeholder_convention not variant: {ph_field:?}"),
        });
    };
    let placeholder = placeholder_variant_label(dag, *ph_ctor).map_err(|e| {
        GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: e.to_string(),
        }
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
        .unwrap_or(0);
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
            return Err(GroundingTestsDiagnostic::StratumALockstepMismatch {
                list_name: list_name.to_string(),
                method_name: "<bootstrap-run>".to_string(),
                detail:
                    "two `generated_full_bootstrap_dag()` runs produced different Stratum-A digests"
                        .to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod stratum_a_tests {
    use v3_compiler::generated_full_bootstrap_dag;

    use super::*;

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
