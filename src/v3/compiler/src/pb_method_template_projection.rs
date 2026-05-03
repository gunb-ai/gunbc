//! PB-Bootstrap-Process consumer hook: project `MethodTemplateContract` rows
//! from the full bootstrap `Dag`.
//!
//! Implements **R3 row 85 / PB #1560 Gap 4** per the decision in
//! `docs/decisions/r3-row85-method-template-read-surface.md`. The single
//! row authority remains `src/v3/std/{rust,python,go}_method_template_contracts.dag`;
//! this module exposes a target-keyed structural projection of those rows
//! over the lowered bootstrap `Dag` so PB-zero / v2-retirement consumers can
//! select rows by target + `MethodRef` without:
//!
//! - importing `v3.std.*` into v2 crates (Gap 4 acceptance A1),
//! - introducing a second `Map<String, String>` template-text authority (A2),
//! - copying any template strings into v2 sources (A3).
//!
//! All five `MethodTemplateContract` fields (`dag_method`, `runtime_template`,
//! `emit_template`, `wraps_result`, `placeholder_convention`) are preserved
//! by typed Rust mirrors below (A4). `emit_template` and
//! `placeholder_convention` are sum carriers; their substrate identity is
//! preserved as constructor-tagged enums, not flattened to strings.
//!
//! Gap 4 only — `LanguageSpec.method_templates` rewrite (Gap 5) and v2 leaf
//! emit migration are sequenced strictly after this hook lands. No v2
//! consumer is migrated by this module.

use crate::dag::{Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};

/// Target language whose `MethodTemplateContract` row list is being projected.
///
/// 🟢 TERMINAL at the per-target row-list scope. The three inhabitants
/// mirror the three closed list-declaration authorities under
/// `src/v3/std/{rust,python,go}_method_template_contracts.dag`. New
/// targets land by adding a row-list authority + a constructor here
/// together; no other dissolution path applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MethodTemplateTarget {
    Rust,
    Python,
    Go,
}

impl MethodTemplateTarget {
    /// Per-target list-declaration name in the bootstrap `Dag`. The strings
    /// here mirror the data declaration names in
    /// `src/v3/std/{rust,python,go}_method_template_contracts.dag` and are
    /// the *names of structural declarations*, not template text.
    fn list_declaration_name(self) -> &'static str {
        match self {
            MethodTemplateTarget::Rust => "rust_method_template_contracts",
            MethodTemplateTarget::Python => "python_method_template_contracts",
            MethodTemplateTarget::Go => "go_method_template_contracts",
        }
    }
}

/// Projected `MethodEmitTemplate` (substrate sum at
/// `src/v3/std/emit_model.dag:469-474`).
///
/// 🟢 TERMINAL at target method-emission-template scope, inheriting the
/// terminality of the substrate carrier (`MethodEmitTemplate` is marked
/// 🟢 TERMINAL at `emit_model.dag:453`). The two-variant split between
/// inline-lambda and function-reference renderings is the closed semantic
/// case the substrate already locks; new variants here would mean the
/// substrate carrier itself moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodEmitTemplateProjection {
    Single {
        template: String,
    },
    HigherOrder {
        inline_template: String,
        fn_ref_template: String,
    },
}

/// Projected `PlaceholderConvention` (substrate sum at
/// `src/v3/std/emit_model.dag:449-451`).
///
/// 🟢 TERMINAL: nullary two-inhabitant sum (`IndexedArgs | NamedArg`) that
/// mirrors the closed substrate carrier. New conventions land by editing
/// the substrate sum first; this projection follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderConventionProjection {
    IndexedArgs,
    NamedArg,
}

/// Projected single `MethodTemplateContract` row.
///
/// `dag_method` is the `DeclarationId` of the `MethodDeclaration` that the
/// row's `MethodRef.decl` field references (the typed, fail-closed substrate
/// identity for the method). Consumers select rows by target +
/// `dag_method` to recover the `(target, method)` template pair.
#[derive(Debug, Clone)]
pub struct MethodTemplateContractRow {
    pub dag_method: DeclarationId,
    pub runtime_template: String,
    pub emit_template: MethodEmitTemplateProjection,
    pub wraps_result: bool,
    pub placeholder_convention: PlaceholderConventionProjection,
}

/// Fail-closed projection error. Every observable mismatch between the
/// bootstrap `Dag` shape and the `MethodTemplateContract` substrate carrier
/// surfaces here as a typed value (per `INVARIANTS.md` C-8).
///
/// 🟢 TERMINAL at the projection's bootstrap-mismatch scope. Each variant
/// names a specific structural class the row authorities cannot satisfy;
/// new variants only land if substrate shape grows a new sub-fact (which
/// would itself follow §P1). The carrier is therefore a closed
/// substrate-mismatch enumeration, not a transitional grab-bag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodTemplateProjectionError {
    /// Per-target row-list declaration is absent from the bootstrap `Dag`.
    /// Should be impossible if `BOOTSTRAP_FIXTURE_PATH_KEYS` (`bootstrap.rs`)
    /// stays in lockstep with `bootstrap_fixture_authority` and
    /// `regen_bootstrap --verify` is green; surfaced rather than panicked
    /// so consumers can act on it.
    ListDeclarationMissing { name: &'static str },
    /// List declaration has no value body (i.e., not a `data` declaration).
    ListDeclarationLacksValueBody { name: &'static str },
    /// List declaration's value body is not `ValueBody::List` — violates the
    /// `bootstrap_method_template_contracts_lower_to_list_with_empty_diagnostics`
    /// gate per the row-85 decision doc.
    ListDeclarationValueBodyNotList { name: &'static str },
    /// Row at the given index in the named list is not a record literal.
    RowNotRecord {
        list: &'static str,
        row_index: usize,
    },
    /// Row is missing one of the five required fields.
    RowMissingField {
        list: &'static str,
        row_index: usize,
        field: &'static str,
    },
    /// `dag_method` field is not the `MethodRef { decl }` record shape.
    DagMethodNotMethodRefRecord {
        list: &'static str,
        row_index: usize,
    },
    /// `MethodRef.decl` is not a typed substrate reference.
    MethodRefDeclNotReference {
        list: &'static str,
        row_index: usize,
    },
    /// `runtime_template` field is not a string literal.
    RuntimeTemplateNotString {
        list: &'static str,
        row_index: usize,
    },
    /// `wraps_result` field is not a bool literal.
    WrapsResultNotBool {
        list: &'static str,
        row_index: usize,
    },
    /// `placeholder_convention` field is not a substrate variant constructor.
    PlaceholderConventionNotVariant {
        list: &'static str,
        row_index: usize,
    },
    /// `placeholder_convention` constructor name is unknown.
    PlaceholderConventionUnknown {
        list: &'static str,
        row_index: usize,
        constructor: Option<String>,
    },
    /// `placeholder_convention` is a nullary sum (`IndexedArgs | NamedArg`),
    /// but the variant payload is non-empty. Mirrors the substrate-side
    /// canonical-row check
    /// `placeholder_convention_canonical_rejects_payload_for_nullary_indexed_args`
    /// in `src/v3/grounding_tests/src/stratum_a.rs`.
    PlaceholderConventionPayloadNotEmpty {
        list: &'static str,
        row_index: usize,
        constructor: &'static str,
        payload_len: usize,
    },
    /// `emit_template` field is not a substrate variant constructor.
    EmitTemplateNotVariant {
        list: &'static str,
        row_index: usize,
    },
    /// `emit_template` constructor name is unknown.
    EmitTemplateUnknownConstructor {
        list: &'static str,
        row_index: usize,
        constructor: Option<String>,
    },
    /// `emit_template` `SingleTemplate` payload shape mismatch (expected one
    /// string-typed `template` field).
    EmitTemplateSinglePayloadShape {
        list: &'static str,
        row_index: usize,
    },
    /// `emit_template` `HigherOrderTemplates` payload shape mismatch
    /// (expected two string-typed fields: `inline_template`, `fn_ref_template`).
    EmitTemplateHigherOrderPayloadShape {
        list: &'static str,
        row_index: usize,
    },
    /// The `(target, dag_method)` direct lookup helper found more than one row
    /// for the same `MethodDeclaration` in the per-target list. Per-target
    /// uniqueness by `dag_method` is the substrate-side claim
    /// (`method_template_contract_per_target_dag_method_unique`); the helper
    /// surfaces a violation rather than silently selecting the first row
    /// (P2 single-authority + P3 fail-closed at the public lookup boundary).
    DuplicateMethodTemplateRow {
        list: &'static str,
        first_row_index: usize,
        duplicate_row_index: usize,
    },
}

/// Project the `MethodTemplateContract` rows for `target` from the full
/// bootstrap `Dag` snapshot.
///
/// The expected source for `dag` is `generated_full_bootstrap_dag()` (or its
/// equivalent under PB-1-e dissolution-first per #1558 — see the
/// row-85 decision doc §"Non-Fork Ratchet"). Calling on a partially-built
/// `Dag` that is missing the per-target list authorities surfaces
/// [`MethodTemplateProjectionError::ListDeclarationMissing`] rather than
/// panicking.
///
/// **Boundary contract:** this is the canonical PB-zero consumer hook for
/// row text. v2-side migrations consume `Vec<MethodTemplateContractRow>` and
/// must not store or carry parallel `Map<String, String>` template tables;
/// the structural carrier is the typed projection above.
pub fn method_template_contract_rows(
    dag: &Dag,
    target: MethodTemplateTarget,
) -> Result<Vec<MethodTemplateContractRow>, MethodTemplateProjectionError> {
    let list_name = target.list_declaration_name();
    let decl = dag
        .declaration_by_name(list_name)
        .ok_or(MethodTemplateProjectionError::ListDeclarationMissing { name: list_name })?;
    let body = decl
        .value_body
        .as_ref()
        .ok_or(MethodTemplateProjectionError::ListDeclarationLacksValueBody { name: list_name })?;
    let ValueBody::List(rows) = body else {
        return Err(
            MethodTemplateProjectionError::ListDeclarationValueBodyNotList { name: list_name },
        );
    };

    rows.iter()
        .enumerate()
        .map(|(row_index, row)| project_row(dag, list_name, row_index, row))
        .collect()
}

/// Direct `(target, dag_method)` lookup helper for Gap-5 / leaf-emit
/// consumers. Returns the row whose `dag_method` matches `dag_method`, or
/// `None` if the per-target list does not contain a row keyed by that
/// `MethodDeclaration`. Projection failures (typed
/// [`MethodTemplateProjectionError`]) bubble through.
///
/// **Fail-closed on duplicate authorities.** Per-target uniqueness by
/// `dag_method` is the substrate-side claim
/// (`method_template_contract_per_target_dag_method_unique`); if two rows
/// share a `dag_method`, the helper surfaces
/// [`MethodTemplateProjectionError::DuplicateMethodTemplateRow`] rather
/// than silently selecting the first match by iteration order. Gap 5 /
/// leaf-emit consumers depend on the helper preserving "one row per
/// `(target, MethodRef)`" as a public boundary.
pub fn method_template_contract_row(
    dag: &Dag,
    target: MethodTemplateTarget,
    dag_method: DeclarationId,
) -> Result<Option<MethodTemplateContractRow>, MethodTemplateProjectionError> {
    let rows = method_template_contract_rows(dag, target)?;
    let list_name = target.list_declaration_name();
    let mut found: Option<(usize, MethodTemplateContractRow)> = None;
    for (index, row) in rows.into_iter().enumerate() {
        if row.dag_method != dag_method {
            continue;
        }
        if let Some((first_row_index, _)) = &found {
            return Err(MethodTemplateProjectionError::DuplicateMethodTemplateRow {
                list: list_name,
                first_row_index: *first_row_index,
                duplicate_row_index: index,
            });
        }
        found = Some((index, row));
    }
    Ok(found.map(|(_, row)| row))
}

fn project_row(
    dag: &Dag,
    list: &'static str,
    row_index: usize,
    row: &FieldValue,
) -> Result<MethodTemplateContractRow, MethodTemplateProjectionError> {
    let FieldValue::Record(fields) = row else {
        return Err(MethodTemplateProjectionError::RowNotRecord { list, row_index });
    };

    let dag_method_value = field_by_label(list, row_index, fields, "dag_method")?;
    let dag_method = project_dag_method(list, row_index, dag_method_value)?;

    let runtime_template_value = field_by_label(list, row_index, fields, "runtime_template")?;
    let runtime_template = match runtime_template_value {
        FieldValue::Literal(LiteralBits::String(s)) => s.clone(),
        _ => {
            return Err(MethodTemplateProjectionError::RuntimeTemplateNotString {
                list,
                row_index,
            });
        }
    };

    let emit_template_value = field_by_label(list, row_index, fields, "emit_template")?;
    let emit_template = project_emit_template(dag, list, row_index, emit_template_value)?;

    let wraps_result_value = field_by_label(list, row_index, fields, "wraps_result")?;
    let wraps_result = match wraps_result_value {
        FieldValue::Literal(LiteralBits::Bool(b)) => *b,
        _ => return Err(MethodTemplateProjectionError::WrapsResultNotBool { list, row_index }),
    };

    let placeholder_value = field_by_label(list, row_index, fields, "placeholder_convention")?;
    let placeholder_convention = project_placeholder(dag, list, row_index, placeholder_value)?;

    Ok(MethodTemplateContractRow {
        dag_method,
        runtime_template,
        emit_template,
        wraps_result,
        placeholder_convention,
    })
}

fn field_by_label<'a>(
    list: &'static str,
    row_index: usize,
    fields: &'a [(String, FieldValue)],
    field: &'static str,
) -> Result<&'a FieldValue, MethodTemplateProjectionError> {
    fields
        .iter()
        .find(|(label, _)| label == field)
        .map(|(_, value)| value)
        .ok_or(MethodTemplateProjectionError::RowMissingField {
            list,
            row_index,
            field,
        })
}

fn project_dag_method(
    list: &'static str,
    row_index: usize,
    value: &FieldValue,
) -> Result<DeclarationId, MethodTemplateProjectionError> {
    let FieldValue::Record(method_ref_fields) = value else {
        return Err(MethodTemplateProjectionError::DagMethodNotMethodRefRecord { list, row_index });
    };
    let decl_value = method_ref_fields
        .iter()
        .find(|(label, _)| label == "decl")
        .map(|(_, v)| v)
        .ok_or(MethodTemplateProjectionError::DagMethodNotMethodRefRecord { list, row_index })?;
    match decl_value {
        FieldValue::Reference(decl_id) => Ok(*decl_id),
        _ => Err(MethodTemplateProjectionError::MethodRefDeclNotReference { list, row_index }),
    }
}

fn project_emit_template(
    dag: &Dag,
    list: &'static str,
    row_index: usize,
    value: &FieldValue,
) -> Result<MethodEmitTemplateProjection, MethodTemplateProjectionError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(MethodTemplateProjectionError::EmitTemplateNotVariant { list, row_index });
    };
    let constructor_label = disj_variant_label(dag, "MethodEmitTemplate", *constructor);
    match constructor_label {
        Some("SingleTemplate") => {
            // Payload shape (positional, per FieldValue::Variant): one
            // `template: String` field. Lowering produces a single
            // record-shaped payload entry whose only field is `template`.
            let template = single_template_payload(list, row_index, payload)?;
            Ok(MethodEmitTemplateProjection::Single { template })
        }
        Some("HigherOrderTemplates") => {
            let (inline_template, fn_ref_template) =
                higher_order_payload(list, row_index, payload)?;
            Ok(MethodEmitTemplateProjection::HigherOrder {
                inline_template,
                fn_ref_template,
            })
        }
        other => Err(
            MethodTemplateProjectionError::EmitTemplateUnknownConstructor {
                list,
                row_index,
                constructor: other.map(str::to_string),
            },
        ),
    }
}

/// Recover a sum-variant label by walking the parent `Disj` declaration's
/// variant list. Variant constructor declarations themselves do not carry
/// `name`; identity flows through the parent's `variants[i].label`. Mirrors
/// the pattern used in `tests/integration/method_template_contract_test.rs`
/// (`method_emit_template_variant_label`).
fn disj_variant_label<'a>(
    dag: &'a Dag,
    parent_name: &str,
    constructor: DeclarationId,
) -> Option<&'a str> {
    let parent = dag.declaration_by_name(parent_name)?;
    let TypeConnective::Disj { variants } = &parent.connective else {
        return None;
    };
    variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .map(|variant| variant.label.as_str())
}

fn single_template_payload(
    list: &'static str,
    row_index: usize,
    payload: &[FieldValue],
) -> Result<String, MethodTemplateProjectionError> {
    // `Variant.payload` is positional per the substrate carrier
    // (`FieldValue::Variant` doc-comment). `SingleTemplate { template: String }`
    // therefore lowers to a single-element payload carrying the string literal.
    match payload {
        [FieldValue::Literal(LiteralBits::String(s))] => Ok(s.clone()),
        _ => Err(MethodTemplateProjectionError::EmitTemplateSinglePayloadShape { list, row_index }),
    }
}

fn higher_order_payload(
    list: &'static str,
    row_index: usize,
    payload: &[FieldValue],
) -> Result<(String, String), MethodTemplateProjectionError> {
    // `HigherOrderTemplates { inline_template: String, fn_ref_template: String }`
    // lowers to a two-element positional payload carrying the two string
    // literals in declaration order.
    match payload {
        [FieldValue::Literal(LiteralBits::String(inline)), FieldValue::Literal(LiteralBits::String(fn_ref))] => {
            Ok((inline.clone(), fn_ref.clone()))
        }
        _ => Err(
            MethodTemplateProjectionError::EmitTemplateHigherOrderPayloadShape { list, row_index },
        ),
    }
}

fn project_placeholder(
    dag: &Dag,
    list: &'static str,
    row_index: usize,
    value: &FieldValue,
) -> Result<PlaceholderConventionProjection, MethodTemplateProjectionError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(
            MethodTemplateProjectionError::PlaceholderConventionNotVariant { list, row_index },
        );
    };
    let constructor_label = disj_variant_label(dag, "PlaceholderConvention", *constructor);
    // `PlaceholderConvention` is a nullary sum (`IndexedArgs | NamedArg`)
    // per `src/v3/std/emit_model.dag:449-451`. A non-empty payload would
    // mean a malformed bootstrap row (e.g., `IndexedArgs(false)`) that the
    // projection must surface as a typed error rather than silently
    // discarding. Mirrors the substrate-side canonical-row check in
    // `src/v3/grounding_tests/src/stratum_a.rs`.
    let (label, projection) = match constructor_label {
        Some("IndexedArgs") => ("IndexedArgs", PlaceholderConventionProjection::IndexedArgs),
        Some("NamedArg") => ("NamedArg", PlaceholderConventionProjection::NamedArg),
        other => {
            return Err(
                MethodTemplateProjectionError::PlaceholderConventionUnknown {
                    list,
                    row_index,
                    constructor: other.map(str::to_string),
                },
            );
        }
    };
    if !payload.is_empty() {
        return Err(
            MethodTemplateProjectionError::PlaceholderConventionPayloadNotEmpty {
                list,
                row_index,
                constructor: label,
                payload_len: payload.len(),
            },
        );
    }
    Ok(projection)
}

#[cfg(test)]
mod tests {
    //! Unit tests for projection-internal failure modes that require
    //! crate-private `Dag` mutation to construct (e.g., injecting a
    //! malformed value body that the on-disk row authorities cannot
    //! produce). Behavior tests over the green bootstrap Dag live in
    //! `tests/integration/pb_method_template_projection_test.rs`.

    use super::*;
    use crate::generated_full_bootstrap_dag;

    /// Inject a non-empty payload onto the `placeholder_convention` variant
    /// of the row at `row_index` of the named per-target list. Mutates only
    /// the in-memory `Dag`; the row authority sources on disk are
    /// untouched.
    fn inject_extra_placeholder_payload(dag: &mut Dag, list_name: &str, row_index: usize) {
        let decl_id = dag
            .declaration_by_name(list_name)
            .unwrap_or_else(|| panic!("`{list_name}` missing from bootstrap"))
            .id;
        let decl = dag.declaration_mut(decl_id);
        let body = decl
            .value_body
            .as_mut()
            .unwrap_or_else(|| panic!("`{list_name}` has no value body"));
        let ValueBody::List(rows) = body else {
            panic!("`{list_name}` value body is not a List");
        };
        let row = rows
            .get_mut(row_index)
            .unwrap_or_else(|| panic!("row {row_index} missing from `{list_name}`"));
        let FieldValue::Record(fields) = row else {
            panic!("row {row_index} is not a Record");
        };
        let placeholder_value = fields
            .iter_mut()
            .find(|(label, _)| label == "placeholder_convention")
            .map(|(_, value)| value)
            .expect("placeholder_convention field");
        let FieldValue::Variant { payload, .. } = placeholder_value else {
            panic!("placeholder_convention must be a Variant");
        };
        payload.push(FieldValue::Literal(LiteralBits::Bool(false)));
    }

    /// Duplicate the row at `row_index` of the named per-target list by
    /// pushing a clone onto the end of the list. The two rows share a
    /// `dag_method`, breaking per-target uniqueness — the substrate-side
    /// claim `method_template_contract_per_target_dag_method_unique`.
    /// Mutates only the in-memory `Dag`.
    fn duplicate_row(dag: &mut Dag, list_name: &str, row_index: usize) {
        let decl_id = dag
            .declaration_by_name(list_name)
            .unwrap_or_else(|| panic!("`{list_name}` missing from bootstrap"))
            .id;
        let decl = dag.declaration_mut(decl_id);
        let body = decl
            .value_body
            .as_mut()
            .unwrap_or_else(|| panic!("`{list_name}` has no value body"));
        let ValueBody::List(rows) = body else {
            panic!("`{list_name}` value body is not a List");
        };
        let clone = rows
            .get(row_index)
            .unwrap_or_else(|| panic!("row {row_index} missing from `{list_name}`"))
            .clone();
        rows.push(clone);
    }

    #[test]
    fn duplicate_row_in_lookup_helper_surfaces_typed_error() {
        // Per-target uniqueness by `dag_method` is the substrate-side claim
        // `method_template_contract_per_target_dag_method_unique`. The
        // direct `(target, dag_method)` lookup helper must fail-closed on
        // duplicate rows rather than silently selecting the first match by
        // iteration order — Gap 5 / leaf-emit consumers depend on the
        // helper preserving "one row per `(target, MethodRef)`" as a
        // public boundary.
        let mut dag = generated_full_bootstrap_dag();
        duplicate_row(&mut dag, "rust_method_template_contracts", 0);

        let count_method_id = dag
            .declaration_by_name("count_method")
            .expect("count_method MethodDeclaration in bootstrap Dag")
            .id;

        let result =
            method_template_contract_row(&dag, MethodTemplateTarget::Rust, count_method_id);
        match result {
            Err(MethodTemplateProjectionError::DuplicateMethodTemplateRow {
                list,
                first_row_index,
                duplicate_row_index,
            }) => {
                assert_eq!(list, "rust_method_template_contracts");
                assert_ne!(first_row_index, duplicate_row_index);
            }
            other => panic!(
                "expected DuplicateMethodTemplateRow for duplicate (target, dag_method), got {other:?}"
            ),
        }
    }

    #[test]
    fn malformed_placeholder_payload_surfaces_typed_error() {
        // PlaceholderConvention is a nullary sum (`IndexedArgs | NamedArg`).
        // A malformed bootstrap row carrying a non-empty placeholder payload
        // must surface as `PlaceholderConventionPayloadNotEmpty`, not
        // silently project to `IndexedArgs` / `NamedArg` with the payload
        // discarded. Mirrors the substrate-side canonical-row check
        // `placeholder_convention_canonical_rejects_payload_for_nullary_indexed_args`
        // in `src/v3/grounding_tests/src/stratum_a.rs`.
        let mut dag = generated_full_bootstrap_dag();
        inject_extra_placeholder_payload(&mut dag, "rust_method_template_contracts", 0);

        let result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match result {
            Err(MethodTemplateProjectionError::PlaceholderConventionPayloadNotEmpty {
                list,
                row_index,
                payload_len,
                ..
            }) => {
                assert_eq!(list, "rust_method_template_contracts");
                assert_eq!(row_index, 0);
                assert!(payload_len >= 1, "expected non-empty payload to be reported");
            }
            other => panic!(
                "expected PlaceholderConventionPayloadNotEmpty for malformed placeholder, got {other:?}"
            ),
        }
    }
}
