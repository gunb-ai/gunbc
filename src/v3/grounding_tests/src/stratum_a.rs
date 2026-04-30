//! Stratum A — name-keyed routing parity for `MethodTemplateContract` Phase 1 rows.
//!
//! Source: `docs/briefs/t-ground-tests.md` (test plan item 1). Generalizes the pilot
//! kernel-integer checks (`src/v3/grounding_pilot/src/lib.rs:408-460`) onto the
//! per-target row lists `rust_*` / `python_*` / `go_*` from T-Ground-LanguageSpec #1195.

use std::collections::BTreeMap;

use v3_compiler::dag::{Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};

use crate::diagnostic::GroundingTestsDiagnostic;

const RUST_LIST: &str = "rust_method_template_contracts";
const PYTHON_LIST: &str = "python_method_template_contracts";
const GO_LIST: &str = "go_method_template_contracts";

const RUST_SOURCE: &str = include_str!("../std/rust_method_template_contracts.dag");
const RUST_PATH: &str = "src/v3/std/rust_method_template_contracts.dag";
const PYTHON_SOURCE: &str = include_str!("../std/python_method_template_contracts.dag");
const PYTHON_PATH: &str = "src/v3/std/python_method_template_contracts.dag";
const GO_SOURCE: &str = include_str!("../std/go_method_template_contracts.dag");
const GO_PATH: &str = "src/v3/std/go_method_template_contracts.dag";

/// Director-locked Phase 1 row counts (`t-ground-tests.md`; Go is 14 rows on main — `chars` deferred per file header).
pub const EXPECTED_STRATUM_A_ROW_COUNTS: &[(&str, usize)] =
    &[(RUST_LIST, 9), (PYTHON_LIST, 18), (GO_LIST, 14)];

fn list_rows<'a>(dag: &'a Dag, list_name: &str) -> &'a [FieldValue] {
    let decl = dag
        .declaration_by_name(list_name)
        .unwrap_or_else(|| panic!("`{list_name}` missing from Dag"));
    let body = decl
        .value_body
        .as_ref()
        .unwrap_or_else(|| panic!("`{list_name}` has no value body"));
    let ValueBody::List(rows) = body else {
        panic!("`{list_name}`: expected List body, got {body:?}");
    };
    rows
}

fn method_emit_template_variant_label(dag: &Dag, constructor: DeclarationId) -> String {
    let method_emit_template = dag
        .declaration_by_name("MethodEmitTemplate")
        .expect("MethodEmitTemplate");
    let TypeConnective::Disj { variants } = &method_emit_template.connective else {
        panic!("MethodEmitTemplate must be a Disj");
    };
    variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .unwrap_or_else(|| panic!("unknown MethodEmitTemplate constructor {constructor:?}"))
        .label
        .clone()
}

fn placeholder_variant_label(dag: &Dag, constructor: DeclarationId) -> String {
    let root = dag
        .declaration_by_name("PlaceholderConvention")
        .expect("PlaceholderConvention");
    let TypeConnective::Disj { variants } = &root.connective else {
        panic!("PlaceholderConvention must be a Disj");
    };
    variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .unwrap_or_else(|| panic!("unknown PlaceholderConvention constructor {constructor:?}"))
        .label
        .clone()
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

fn emit_template_canonical(dag: &Dag, emit: &FieldValue) -> Result<String, String> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = emit
    else {
        return Err(format!("emit_template not a variant: {emit:?}"));
    };
    let label = method_emit_template_variant_label(dag, *constructor);
    match label.as_str() {
        "SingleTemplate" => {
            let Some(FieldValue::Literal(LiteralBits::String(s))) = payload.first() else {
                return Err(format!("SingleTemplate payload: {payload:?}"));
            };
            Ok(format!("SingleTemplate({s})"))
        }
        "HigherOrderTemplates" => {
            if payload.len() != 2 {
                return Err(format!("HigherOrderTemplates arity: {payload:?}"));
            }
            let FieldValue::Literal(LiteralBits::String(inline)) = &payload[0] else {
                return Err(format!("inline_template: {:?}", payload[0]));
            };
            let FieldValue::Literal(LiteralBits::String(fn_ref)) = &payload[1] else {
                return Err(format!("fn_ref_template: {:?}", payload[1]));
            };
            Ok(format!("HigherOrderTemplates({inline}|{fn_ref})"))
        }
        _ => Err(format!("unknown MethodEmitTemplate variant `{label}`")),
    }
}

/// One `MethodTemplateContract` row projected to deterministic strings for parity / digests.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
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
    let emit_canonical = emit_template_canonical(dag, emit_field).map_err(|e| {
        GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
            list_name: list_name.to_string(),
            row_index,
            detail: e,
        }
    })?;

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
    let placeholder = placeholder_variant_label(dag, *ph_ctor);

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
    let rows = list_rows(dag, list_name);
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

fn compare_fingerprint_maps(
    list_name: &str,
    bootstrap: &BTreeMap<String, RowFingerprint>,
    standalone: &BTreeMap<String, RowFingerprint>,
) -> Result<(), GroundingTestsDiagnostic> {
    for (k, b) in bootstrap {
        let Some(s) = standalone.get(k) else {
            return Err(GroundingTestsDiagnostic::StratumALockstepMismatch {
                list_name: list_name.to_string(),
                method_name: k.clone(),
                detail: "present in full-bootstrap Dag but missing in standalone compile"
                    .to_string(),
            });
        };
        if b != s {
            return Err(GroundingTestsDiagnostic::StratumALockstepMismatch {
                list_name: list_name.to_string(),
                method_name: k.clone(),
                detail: format!("bootstrap={b:?} standalone={s:?}"),
            });
        }
    }
    for k in standalone.keys() {
        if !bootstrap.contains_key(k) {
            return Err(GroundingTestsDiagnostic::StratumALockstepMismatch {
                list_name: list_name.to_string(),
                method_name: k.clone(),
                detail: "present in standalone compile but missing in full-bootstrap Dag"
                    .to_string(),
            });
        }
    }
    Ok(())
}

/// Stratum A: each Phase 1 row list matches between [`v3_compiler::generated_full_bootstrap_dag`]
/// and a fresh [`v3_compiler::compile_to_dag`] of the checked-in `.dag` authority (lockstep).
pub fn verify_stratum_a_lockstep_all_targets() -> Result<(), GroundingTestsDiagnostic> {
    let bootstrap = v3_compiler::generated_full_bootstrap_dag();
    let cases: &[(&str, &str, &str)] = &[
        (RUST_LIST, RUST_SOURCE, RUST_PATH),
        (PYTHON_LIST, PYTHON_SOURCE, PYTHON_PATH),
        (GO_LIST, GO_SOURCE, GO_PATH),
    ];
    for &(list_name, source, path) in cases {
        let n = list_rows(&bootstrap, list_name).len();
        assert_expected_row_count(list_name, n)?;
        let standalone = v3_compiler::compile_to_dag(source, path).map_err(|e| {
            GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name: list_name.to_string(),
                row_index: 0,
                detail: format!("standalone compile `{path}` failed: {e:?}"),
            }
        })?;
        let n2 = list_rows(&standalone, list_name).len();
        assert_expected_row_count(list_name, n2)?;
        let b_map = fingerprints_for_list(&bootstrap, list_name)?;
        let s_map = fingerprints_for_list(&standalone, list_name)?;
        compare_fingerprint_maps(list_name, &b_map, &s_map)?;
    }
    Ok(())
}
