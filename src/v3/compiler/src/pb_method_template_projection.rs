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

use std::collections::HashMap;

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
    /// Row is a declaration reference, but the referenced declaration is not
    /// present in the bootstrap `Dag`.
    RowReferenceMissing {
        list: &'static str,
        row_index: usize,
        decl_id: DeclarationId,
    },
    /// Row is a declaration reference, but the referenced declaration is not a
    /// `data` declaration carrying a structural `MethodTemplateContract` body.
    RowReferenceNotStructural {
        list: &'static str,
        row_index: usize,
        decl_id: DeclarationId,
    },
    /// Row is missing one of the five required fields.
    RowMissingField {
        list: &'static str,
        row_index: usize,
        field: &'static str,
    },
    /// Row carries a duplicate field — two entries share a label. The
    /// substrate-side `MethodTemplateContract` (and `MethodRef`) carriers
    /// declare each field exactly once; duplicates would mean two
    /// authorities for the same row coordinate (P2 single-authority).
    RowDuplicateField {
        list: &'static str,
        row_index: usize,
        record: &'static str,
        field: &'static str,
        first_field_index: usize,
        duplicate_field_index: usize,
    },
    /// Row carries a field whose label is not one of the closed schema
    /// (`MethodTemplateContract` = five fields; `MethodRef` = one field).
    /// Extra substrate fields land via the substrate carrier, not via
    /// projection-side acceptance, so unknown labels surface here rather
    /// than being silently ignored (P2 / C-8).
    RowUnknownField {
        list: &'static str,
        row_index: usize,
        record: &'static str,
        field: String,
        field_index: usize,
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
    /// `MethodRef.decl` references a declaration that does not satisfy
    /// the substrate-side `MethodDeclaration` *data-binding* contract
    /// (`src/v3/std/methods.dag` + `dsl/std/methods.dag`). Mirrors the
    /// Stratum A registry gate
    /// (`src/v3/grounding_tests/src/stratum_a.rs::method_registry_name`):
    /// the binding must (a) instantiate `MethodDeclaration`, (b) carry
    /// a `Structural` value body, (c) have a closed single-`name` field
    /// schema, (d) and the `name` value must be a string literal. The
    /// `reason` payload identifies which sub-check failed.
    MethodRefDeclNotMethodDeclaration {
        list: &'static str,
        row_index: usize,
        decl_id: DeclarationId,
        reason: MethodDeclarationBindingViolation,
    },
    /// `method_template_contract_row` was called with a `dag_method`
    /// `DeclarationId` that does not satisfy the
    /// `MethodDeclaration` data-binding contract (same shape and
    /// `reason` taxonomy as `MethodRefDeclNotMethodDeclaration`). The
    /// helper validates at entry so callers cannot conflate
    /// "valid method, no row" with "caller handed in a non-method
    /// declaration."
    LookupKeyNotMethodDeclaration {
        decl_id: DeclarationId,
        reason: MethodDeclarationBindingViolation,
    },
    /// `MethodDeclaration` itself is missing from the bootstrap `Dag`.
    /// Should be impossible if the std fixtures lower cleanly; surfaced
    /// rather than panicked so consumers can act on it.
    MethodDeclarationCarrierMissing,
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

/// Granular reason for a `MethodDeclaration` data-binding contract failure.
///
/// 🟢 TERMINAL at the substrate-side data-binding contract scope
/// (`src/v3/std/methods.dag` + Stratum A `method_registry_name`). Each
/// variant maps one of the four sub-checks the contract requires; the
/// substrate-side contract is closed (`MethodDeclaration` carries a
/// single `name: String` field), so this enumeration is closed too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodDeclarationBindingViolation {
    /// Connective is not `TypeConnective::Instantiation { template, .. }`.
    /// Rules out type / sum / record declarations.
    ConnectiveNotInstantiation,
    /// `Instantiation` connective, but `template` does not match
    /// `MethodDeclaration`'s declaration id. Rules out instantiations of
    /// unrelated targets.
    InstantiationTemplateNotMethodDeclaration,
    /// `value_body` is `None`. Rules out type aliases of the form
    /// `type Foo = MethodDeclaration`.
    ValueBodyMissing,
    /// `value_body` is `Some(_)` but not `ValueBody::Structural`. Rules
    /// out `List` / `Map` / scalar bodies that would be wrong-shaped for
    /// a `MethodDeclaration` data binding.
    ValueBodyNotStructural,
    /// `Structural` body, but its field set does not match the closed
    /// `MethodDeclaration` schema (single `name` field). Mirrors
    /// `enforce_closed_record_schema` in Stratum A.
    StructuralFieldsNotClosedNameOnly { observed_labels: Vec<String> },
    /// Closed-schema match, but the `name` field's `FieldValue` is not
    /// `Literal(LiteralBits::String(_))`.
    NameNotStringLiteral,
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

    let projected: Vec<MethodTemplateContractRow> = rows
        .iter()
        .enumerate()
        .map(|(row_index, row)| project_row(dag, list_name, row_index, row))
        .collect::<Result<_, _>>()?;

    // Per-target uniqueness by `dag_method` is the substrate-side claim
    // (`method_template_contract_per_target_dag_method_unique`). Enforce it
    // at the canonical row-list boundary so v2-retirement consumers that
    // build their own keyed tables from the projection cannot silently
    // observe two authorities for the same `(target, MethodRef)` pair.
    let mut first_seen: HashMap<DeclarationId, usize> = HashMap::with_capacity(projected.len());
    for (index, row) in projected.iter().enumerate() {
        if let Some(prior) = first_seen.get(&row.dag_method) {
            return Err(MethodTemplateProjectionError::DuplicateMethodTemplateRow {
                list: list_name,
                first_row_index: *prior,
                duplicate_row_index: index,
            });
        }
        first_seen.insert(row.dag_method, index);
    }

    Ok(projected)
}

/// Direct `(target, dag_method)` lookup helper for Gap-5 / leaf-emit
/// consumers. Returns the row whose `dag_method` matches `dag_method`, or
/// `None` if the per-target list does not contain a row keyed by that
/// `MethodDeclaration`.
///
/// **Fail-closed on non-method keys.** `dag_method` must instantiate
/// `MethodDeclaration` — the helper validates this at entry so a caller
/// cannot conflate two distinct states: "valid `MethodDeclaration` with
/// no row for this target" (returns `Ok(None)`) versus "caller supplied
/// a declaration that is not a `MethodDeclaration` at all" (returns
/// [`MethodTemplateProjectionError::LookupKeyNotMethodDeclaration`]).
/// This mirrors the per-row `MethodRef.decl` check the projection
/// already enforces; until `DeclarationRef<MethodDeclaration>`
/// refinement-typing lands, this is the only place the boundary can
/// hold.
///
/// Other typed projection failures — including
/// [`MethodTemplateProjectionError::DuplicateMethodTemplateRow`], which is
/// enforced at the canonical row-list boundary in
/// [`method_template_contract_rows`] — bubble through.
pub fn method_template_contract_row(
    dag: &Dag,
    target: MethodTemplateTarget,
    dag_method: DeclarationId,
) -> Result<Option<MethodTemplateContractRow>, MethodTemplateProjectionError> {
    let method_declaration_id = method_declaration_carrier_id(dag)?;
    if let Err(reason) =
        validate_method_declaration_data_binding(dag, dag_method, method_declaration_id)
    {
        return Err(
            MethodTemplateProjectionError::LookupKeyNotMethodDeclaration {
                decl_id: dag_method,
                reason,
            },
        );
    }
    let rows = method_template_contract_rows(dag, target)?;
    Ok(rows.into_iter().find(|row| row.dag_method == dag_method))
}

/// Locate the `MethodDeclaration` carrier declaration in the bootstrap
/// `Dag` once. Surfaces [`MethodTemplateProjectionError::MethodDeclarationCarrierMissing`]
/// when the carrier is absent rather than panicking.
pub(crate) fn method_declaration_carrier_id(
    dag: &Dag,
) -> Result<DeclarationId, MethodTemplateProjectionError> {
    Ok(dag
        .declaration_by_name("MethodDeclaration")
        .ok_or(MethodTemplateProjectionError::MethodDeclarationCarrierMissing)?
        .id)
}

/// Validate that `decl_id` references a `MethodDeclaration` *data binding*
/// per the substrate-side contract (`src/v3/std/methods.dag` +
/// `dsl/std/methods.dag`).
///
/// Mirrors `src/v3/grounding_tests/src/stratum_a.rs::method_registry_name`
/// so projection-side and Stratum A-side enforcement of the same gate
/// stay in lockstep. Four sub-checks must all pass:
///
///  1. **Connective is `Instantiation`.** Rules out type / sum / record
///     declarations.
///  2. **`Instantiation.template == MethodDeclaration`.** Rules out
///     instantiations of unrelated targets.
///  3. **`value_body` is `Some(ValueBody::Structural { .. })`.** Rules
///     out type aliases (`type Foo = MethodDeclaration`, no value body)
///     and wrong-shaped data bindings (`List` / `Map` / scalar bodies).
///  4. **`Structural` fields are exactly `[name]` with a string literal
///     value.** Closed schema check — mirrors
///     `enforce_closed_record_schema` in Stratum A. Rules out extra
///     fields, missing `name`, duplicate `name`, and non-string-literal
///     `name` values.
///
/// On success returns the projected method name (`name` field's string
/// literal). On failure returns the typed
/// [`MethodDeclarationBindingViolation`] reason.
///
/// Both `project_dag_method` (per-row) and `method_template_contract_row`
/// (lookup helper) consume this helper so the data-binding identity is
/// factored once.
pub(crate) fn validate_method_declaration_data_binding(
    dag: &Dag,
    decl_id: DeclarationId,
    method_declaration_id: DeclarationId,
) -> Result<String, MethodDeclarationBindingViolation> {
    let referenced = dag.declaration(decl_id);

    // (1) + (2) Connective + template.
    let template = match &referenced.connective {
        TypeConnective::Instantiation { template, .. } => *template,
        _ => return Err(MethodDeclarationBindingViolation::ConnectiveNotInstantiation),
    };
    if template != method_declaration_id {
        return Err(MethodDeclarationBindingViolation::InstantiationTemplateNotMethodDeclaration);
    }

    // (3) Value body present and Structural.
    let value_body = referenced
        .value_body
        .as_ref()
        .ok_or(MethodDeclarationBindingViolation::ValueBodyMissing)?;
    let ValueBody::Structural { fields } = value_body else {
        return Err(MethodDeclarationBindingViolation::ValueBodyNotStructural);
    };

    // (4) Closed `[name]` schema with string-literal value. Reject any
    // duplicate or unknown label, require `name` to be present exactly
    // once. Mirrors Stratum A's enforce_closed_record_schema.
    const EXPECTED: &[&str] = &["name"];
    let name_count = fields.iter().filter(|(label, _)| label == "name").count();
    let any_unknown = fields
        .iter()
        .any(|(label, _)| !EXPECTED.contains(&label.as_str()));
    if fields.len() != EXPECTED.len() || name_count != 1 || any_unknown {
        let observed_labels: Vec<String> = fields.iter().map(|(label, _)| label.clone()).collect();
        return Err(
            MethodDeclarationBindingViolation::StructuralFieldsNotClosedNameOnly {
                observed_labels,
            },
        );
    }
    let (_, name_value) = fields
        .iter()
        .find(|(label, _)| label == "name")
        .expect("closed-schema check above guarantees a single `name` field");
    match name_value {
        FieldValue::Literal(LiteralBits::String(s)) => Ok(s.clone()),
        _ => Err(MethodDeclarationBindingViolation::NameNotStringLiteral),
    }
}

fn project_row(
    dag: &Dag,
    list: &'static str,
    row_index: usize,
    row: &FieldValue,
) -> Result<MethodTemplateContractRow, MethodTemplateProjectionError> {
    let fields = match row {
        FieldValue::Record(fields) => fields,
        FieldValue::Reference(decl_id) => {
            let decl = dag.declaration_opt(decl_id).ok_or(
                MethodTemplateProjectionError::RowReferenceMissing {
                    list,
                    row_index,
                    decl_id: *decl_id,
                },
            )?;
            match decl.value_body.as_ref() {
                Some(ValueBody::Structural { fields }) => fields,
                _ => {
                    return Err(MethodTemplateProjectionError::RowReferenceNotStructural {
                        list,
                        row_index,
                        decl_id: *decl_id,
                    });
                }
            }
        }
        _ => return Err(MethodTemplateProjectionError::RowNotRecord { list, row_index }),
    };

    const EXPECTED: &[&str] = &[
        "dag_method",
        "runtime_template",
        "emit_template",
        "wraps_result",
        "placeholder_convention",
    ];
    let lookup = closed_record_lookup(list, row_index, "MethodTemplateContract", fields, EXPECTED)?;

    let dag_method_value = lookup_required(list, row_index, &lookup, "dag_method")?;
    let dag_method = project_dag_method(dag, list, row_index, dag_method_value)?;

    let runtime_template_value = lookup_required(list, row_index, &lookup, "runtime_template")?;
    let runtime_template = match runtime_template_value {
        FieldValue::Literal(LiteralBits::String(s)) => s.clone(),
        _ => {
            return Err(MethodTemplateProjectionError::RuntimeTemplateNotString {
                list,
                row_index,
            });
        }
    };

    let emit_template_value = lookup_required(list, row_index, &lookup, "emit_template")?;
    let emit_template = project_emit_template(dag, list, row_index, emit_template_value)?;

    let wraps_result_value = lookup_required(list, row_index, &lookup, "wraps_result")?;
    let wraps_result = match wraps_result_value {
        FieldValue::Literal(LiteralBits::Bool(b)) => *b,
        _ => return Err(MethodTemplateProjectionError::WrapsResultNotBool { list, row_index }),
    };

    let placeholder_value = lookup_required(list, row_index, &lookup, "placeholder_convention")?;
    let placeholder_convention = project_placeholder(dag, list, row_index, placeholder_value)?;

    Ok(MethodTemplateContractRow {
        dag_method,
        runtime_template,
        emit_template,
        wraps_result,
        placeholder_convention,
    })
}

/// Validate that `fields` carries exactly the closed set of `expected`
/// labels — no duplicates, no unknowns. Missing labels are not reported
/// here (the per-field `lookup_required` calls below surface those as
/// `RowMissingField` so the error names *which* field is missing).
///
/// Returns a label-to-`FieldValue` map for the present fields; collisions
/// surface as `RowDuplicateField` and unknown labels surface as
/// `RowUnknownField`. `record` names the carrier (e.g.
/// `"MethodTemplateContract"` or `"MethodRef"`) for diagnostic context.
fn closed_record_lookup<'a>(
    list: &'static str,
    row_index: usize,
    record: &'static str,
    fields: &'a [(String, FieldValue)],
    expected: &[&'static str],
) -> Result<HashMap<&'a str, &'a FieldValue>, MethodTemplateProjectionError> {
    let mut lookup: HashMap<&'static str, &FieldValue> = HashMap::with_capacity(expected.len());
    let mut first_seen_index: HashMap<&'static str, usize> = HashMap::with_capacity(expected.len());
    for (field_index, (label, value)) in fields.iter().enumerate() {
        // Resolve the row-side `String` label to its `&'static str` in
        // `expected`. Holding the static reference avoids re-scanning the
        // expected slice (and the `.expect()` re-borrow) on the duplicate
        // path; non-membership surfaces as `RowUnknownField`.
        let Some(static_label) = expected.iter().copied().find(|exp| *exp == label.as_str()) else {
            return Err(MethodTemplateProjectionError::RowUnknownField {
                list,
                row_index,
                record,
                field: label.clone(),
                field_index,
            });
        };
        if let Some(prior_index) = first_seen_index.get(static_label) {
            return Err(MethodTemplateProjectionError::RowDuplicateField {
                list,
                row_index,
                record,
                field: static_label,
                first_field_index: *prior_index,
                duplicate_field_index: field_index,
            });
        }
        lookup.insert(static_label, value);
        first_seen_index.insert(static_label, field_index);
    }
    Ok(lookup)
}

fn lookup_required<'a>(
    list: &'static str,
    row_index: usize,
    lookup: &HashMap<&'a str, &'a FieldValue>,
    field: &'static str,
) -> Result<&'a FieldValue, MethodTemplateProjectionError> {
    lookup
        .get(field)
        .copied()
        .ok_or(MethodTemplateProjectionError::RowMissingField {
            list,
            row_index,
            field,
        })
}

fn project_dag_method(
    dag: &Dag,
    list: &'static str,
    row_index: usize,
    value: &FieldValue,
) -> Result<DeclarationId, MethodTemplateProjectionError> {
    let FieldValue::Record(method_ref_fields) = value else {
        return Err(MethodTemplateProjectionError::DagMethodNotMethodRefRecord { list, row_index });
    };
    // `MethodRef` is a single-field carrier (`{ decl: DeclarationRef }`)
    // per `src/v3/std/methods.dag`. Apply the same closed-schema check
    // as the outer row so duplicates (`{ decl: a, decl: b }`) and
    // unknown extras (`{ decl: ..., extra: ... }`) surface as typed
    // errors instead of being silently ignored.
    const METHOD_REF_FIELDS: &[&str] = &["decl"];
    let lookup = closed_record_lookup(
        list,
        row_index,
        "MethodRef",
        method_ref_fields,
        METHOD_REF_FIELDS,
    )?;
    let decl_value = lookup
        .get("decl")
        .copied()
        .ok_or(MethodTemplateProjectionError::DagMethodNotMethodRefRecord { list, row_index })?;
    let decl_id = match decl_value {
        FieldValue::Reference(decl_id) => *decl_id,
        _ => {
            return Err(MethodTemplateProjectionError::MethodRefDeclNotReference {
                list,
                row_index,
            });
        }
    };
    // `src/v3/std/methods.dag` calls out the boundary contract: today's
    // substrate grammar can't express `DeclarationRef<MethodDeclaration>`,
    // so the projection enforces fail-closed at the boundary that
    // `decl` references a `MethodDeclaration`-instantiating data binding.
    // Pattern mirrors `method_registry_test.rs::method_registry_covers_*`.
    let method_declaration_id = method_declaration_carrier_id(dag)?;
    if let Err(reason) =
        validate_method_declaration_data_binding(dag, decl_id, method_declaration_id)
    {
        return Err(
            MethodTemplateProjectionError::MethodRefDeclNotMethodDeclaration {
                list,
                row_index,
                decl_id,
                reason,
            },
        );
    }
    Ok(decl_id)
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

    /// Mutate the row at `row_index` of `list_name` by adding a second
    /// field with `field_label` (cloned from the first occurrence). Used
    /// by the closed-schema duplicate-field tests below.
    fn duplicate_field_on_row(dag: &mut Dag, list_name: &str, row_index: usize, field_label: &str) {
        let decl_id = dag.declaration_by_name(list_name).expect("list").id;
        let decl = dag.declaration_mut(decl_id);
        let body = decl.value_body.as_mut().expect("value body");
        let ValueBody::List(rows) = body else {
            panic!("not a list");
        };
        let row = rows.get_mut(row_index).expect("row");
        let FieldValue::Record(fields) = row else {
            panic!("row not record");
        };
        let (orig_label, orig_value) = fields
            .iter()
            .find(|(label, _)| label == field_label)
            .map(|(label, value)| (label.clone(), value.clone()))
            .expect("field to clone");
        fields.push((orig_label, orig_value));
    }

    /// Replace the row at `row_index` of `list_name` so that, after the
    /// existing fields, an unknown-labelled field is appended (used by
    /// the closed-schema unknown-field tests below).
    fn append_unknown_field_on_row(
        dag: &mut Dag,
        list_name: &str,
        row_index: usize,
        unknown_label: &str,
    ) {
        let decl_id = dag.declaration_by_name(list_name).expect("list").id;
        let decl = dag.declaration_mut(decl_id);
        let body = decl.value_body.as_mut().expect("value body");
        let ValueBody::List(rows) = body else {
            panic!("not a list");
        };
        let row = rows.get_mut(row_index).expect("row");
        let FieldValue::Record(fields) = row else {
            panic!("row not record");
        };
        fields.push((
            unknown_label.to_string(),
            FieldValue::Literal(LiteralBits::Bool(false)),
        ));
    }

    #[test]
    fn duplicate_field_on_row_surfaces_typed_error() {
        let mut dag = generated_full_bootstrap_dag();
        duplicate_field_on_row(
            &mut dag,
            "rust_method_template_contracts",
            0,
            "runtime_template",
        );
        let result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match result {
            Err(MethodTemplateProjectionError::RowDuplicateField {
                record,
                field,
                first_field_index,
                duplicate_field_index,
                ..
            }) => {
                assert_eq!(record, "MethodTemplateContract");
                assert_eq!(field, "runtime_template");
                assert_ne!(first_field_index, duplicate_field_index);
            }
            other => panic!("expected RowDuplicateField, got {other:?}"),
        }
    }

    #[test]
    fn unknown_field_on_row_surfaces_typed_error() {
        let mut dag = generated_full_bootstrap_dag();
        append_unknown_field_on_row(
            &mut dag,
            "rust_method_template_contracts",
            0,
            "renamed_field",
        );
        let result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match result {
            Err(MethodTemplateProjectionError::RowUnknownField { record, field, .. }) => {
                assert_eq!(record, "MethodTemplateContract");
                assert_eq!(field, "renamed_field");
            }
            other => panic!("expected RowUnknownField, got {other:?}"),
        }
    }

    /// Inject a duplicate or unknown field into the inner `MethodRef`
    /// record at `dag_method` for the row at `row_index`.
    fn mutate_method_ref(
        dag: &mut Dag,
        list_name: &str,
        row_index: usize,
        kind: MethodRefMutation<'_>,
    ) {
        let decl_id = dag.declaration_by_name(list_name).expect("list").id;
        let decl = dag.declaration_mut(decl_id);
        let body = decl.value_body.as_mut().expect("value body");
        let ValueBody::List(rows) = body else {
            panic!("not a list");
        };
        let row = rows.get_mut(row_index).expect("row");
        let FieldValue::Record(fields) = row else {
            panic!("row not record");
        };
        let (_, dag_method_value) = fields
            .iter_mut()
            .find(|(label, _)| label == "dag_method")
            .expect("dag_method field");
        let FieldValue::Record(method_ref_fields) = dag_method_value else {
            panic!("dag_method not record");
        };
        match kind {
            MethodRefMutation::Duplicate => {
                let (label, value) = method_ref_fields
                    .iter()
                    .find(|(label, _)| label == "decl")
                    .map(|(label, value)| (label.clone(), value.clone()))
                    .expect("decl field");
                method_ref_fields.push((label, value));
            }
            MethodRefMutation::AppendUnknown(label) => {
                method_ref_fields.push((
                    label.to_string(),
                    FieldValue::Literal(LiteralBits::Bool(false)),
                ));
            }
        }
    }

    /// 🟢 TERMINAL at the test-local `MethodRef` mutation scope. Names the
    /// two ways an in-memory `Dag` row can break the closed `MethodRef`
    /// schema: duplicate the existing `decl` field, or append a field with
    /// an unknown label. Both modes drive the closed-schema check in
    /// `closed_record_lookup`. Test-only; not part of the projection's
    /// public surface.
    enum MethodRefMutation<'a> {
        Duplicate,
        AppendUnknown(&'a str),
    }

    #[test]
    fn method_ref_decl_alias_without_value_body_surfaces_typed_error() {
        // `src/v3/std/methods.dag` requires a *data binding*
        // (`data <name>_method: MethodDeclaration = { name: "..." }`),
        // not just any declaration whose connective resolves to
        // `MethodDeclaration`. A type alias like
        // `type Foo = MethodDeclaration` lowers to an `Instantiation`
        // connective with **no `value_body`**; without the value-body
        // guard, that shape would pass the connective check alone.
        //
        // Simulate the alias-shape by stripping `value_body` from a real
        // `MethodDeclaration` data binding (`count_method`) and asserting
        // the typed error fires for both the per-row check
        // (`MethodRefDeclNotMethodDeclaration`) and the lookup helper
        // (`LookupKeyNotMethodDeclaration`).
        let mut dag = generated_full_bootstrap_dag();
        let count_method_id = dag
            .declaration_by_name("count_method")
            .expect("count_method MethodDeclaration in bootstrap")
            .id;
        dag.declaration_mut(count_method_id).value_body = None;

        // Per-row path: row 0 of `rust_method_template_contracts` is the
        // `count_method` row, so projecting it should now surface
        // `MethodRefDeclNotMethodDeclaration` with `ValueBodyMissing`
        // reason instead of returning a row.
        let row_result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match row_result {
            Err(MethodTemplateProjectionError::MethodRefDeclNotMethodDeclaration {
                decl_id,
                reason,
                ..
            }) => {
                assert_eq!(decl_id, count_method_id);
                assert_eq!(reason, MethodDeclarationBindingViolation::ValueBodyMissing);
            }
            other => panic!(
                "expected MethodRefDeclNotMethodDeclaration for alias-shaped target, got {other:?}"
            ),
        }

        // Lookup-helper path: same key, same shape; the helper validates
        // at entry and surfaces `LookupKeyNotMethodDeclaration` with the
        // same reason.
        let helper_result =
            method_template_contract_row(&dag, MethodTemplateTarget::Rust, count_method_id);
        match helper_result {
            Err(MethodTemplateProjectionError::LookupKeyNotMethodDeclaration {
                decl_id,
                reason,
            }) => {
                assert_eq!(decl_id, count_method_id);
                assert_eq!(reason, MethodDeclarationBindingViolation::ValueBodyMissing);
            }
            other => {
                panic!("expected LookupKeyNotMethodDeclaration for alias-shaped key, got {other:?}")
            }
        }
    }

    #[test]
    fn method_ref_decl_not_method_declaration_surfaces_typed_error() {
        // `src/v3/std/methods.dag` enforces fail-closed at the projection
        // boundary that `MethodRef.decl` instantiates `MethodDeclaration`,
        // because today's substrate grammar can't express
        // `DeclarationRef<MethodDeclaration>`. Mutate the first row's
        // `MethodRef.decl` to point at a declaration that is not a
        // `MethodDeclaration` instance (`MethodTemplateContract` itself
        // is a type declaration, not a `MethodDeclaration` data binding)
        // and assert the typed error.
        let mut dag = generated_full_bootstrap_dag();
        let non_method_id = dag
            .declaration_by_name("MethodTemplateContract")
            .expect("MethodTemplateContract type")
            .id;

        let list_decl_id = dag
            .declaration_by_name("rust_method_template_contracts")
            .expect("list")
            .id;
        let decl = dag.declaration_mut(list_decl_id);
        let body = decl.value_body.as_mut().expect("value body");
        let ValueBody::List(rows) = body else {
            panic!("not a list");
        };
        let row = rows.get_mut(0).expect("row 0");
        let FieldValue::Record(fields) = row else {
            panic!("row not record");
        };
        let dag_method_value = fields
            .iter_mut()
            .find(|(label, _)| label == "dag_method")
            .map(|(_, value)| value)
            .expect("dag_method field");
        let FieldValue::Record(method_ref_fields) = dag_method_value else {
            panic!("dag_method not record");
        };
        let decl_field_value = method_ref_fields
            .iter_mut()
            .find(|(label, _)| label == "decl")
            .map(|(_, value)| value)
            .expect("decl field");
        *decl_field_value = FieldValue::Reference(non_method_id);

        let result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match result {
            Err(MethodTemplateProjectionError::MethodRefDeclNotMethodDeclaration {
                list,
                row_index,
                decl_id,
                reason,
            }) => {
                assert_eq!(list, "rust_method_template_contracts");
                assert_eq!(row_index, 0);
                assert_eq!(decl_id, non_method_id);
                // `MethodTemplateContract` is a type declaration (Conj),
                // not an Instantiation — connective sub-check fails first.
                assert_eq!(
                    reason,
                    MethodDeclarationBindingViolation::ConnectiveNotInstantiation
                );
            }
            other => panic!(
                "expected MethodRefDeclNotMethodDeclaration for non-MethodDeclaration target, got {other:?}"
            ),
        }
    }

    #[test]
    fn method_ref_decl_extra_field_surfaces_closed_schema_violation() {
        // A MethodDeclaration data binding with an extra field (beyond the
        // closed `[name]` schema) must surface
        // `StructuralFieldsNotClosedNameOnly`. Mirrors Stratum A's
        // `enforce_closed_record_schema` gate.
        let mut dag = generated_full_bootstrap_dag();
        let count_method_id = dag
            .declaration_by_name("count_method")
            .expect("count_method")
            .id;
        let decl = dag.declaration_mut(count_method_id);
        let body = decl.value_body.as_mut().expect("value body");
        let ValueBody::Structural { fields } = body else {
            panic!("expected Structural body");
        };
        fields.push((
            "alias".to_string(),
            FieldValue::Literal(LiteralBits::String("count_alias".to_string())),
        ));

        let result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match result {
            Err(MethodTemplateProjectionError::MethodRefDeclNotMethodDeclaration {
                reason:
                    MethodDeclarationBindingViolation::StructuralFieldsNotClosedNameOnly {
                        observed_labels,
                    },
                ..
            }) => {
                assert!(observed_labels.contains(&"name".to_string()));
                assert!(observed_labels.contains(&"alias".to_string()));
            }
            other => {
                panic!("expected StructuralFieldsNotClosedNameOnly for extra field, got {other:?}")
            }
        }
    }

    #[test]
    fn method_ref_decl_name_not_string_literal_surfaces_typed_error() {
        // A MethodDeclaration data binding whose `name` value is not a
        // string literal must surface
        // `MethodDeclarationBindingViolation::NameNotStringLiteral`.
        let mut dag = generated_full_bootstrap_dag();
        let count_method_id = dag
            .declaration_by_name("count_method")
            .expect("count_method")
            .id;
        let decl = dag.declaration_mut(count_method_id);
        let body = decl.value_body.as_mut().expect("value body");
        let ValueBody::Structural { fields } = body else {
            panic!("expected Structural body");
        };
        let (_, name_value) = fields
            .iter_mut()
            .find(|(label, _)| label == "name")
            .expect("name field");
        // Replace with a Bool literal — wrong shape for the contract.
        *name_value = FieldValue::Literal(LiteralBits::Bool(true));

        let result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match result {
            Err(MethodTemplateProjectionError::MethodRefDeclNotMethodDeclaration {
                reason: MethodDeclarationBindingViolation::NameNotStringLiteral,
                ..
            }) => {}
            other => panic!("expected NameNotStringLiteral for non-string name, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_field_on_method_ref_surfaces_typed_error() {
        let mut dag = generated_full_bootstrap_dag();
        mutate_method_ref(
            &mut dag,
            "rust_method_template_contracts",
            0,
            MethodRefMutation::Duplicate,
        );
        let result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match result {
            Err(MethodTemplateProjectionError::RowDuplicateField { record, field, .. }) => {
                assert_eq!(record, "MethodRef");
                assert_eq!(field, "decl");
            }
            other => panic!("expected RowDuplicateField for MethodRef, got {other:?}"),
        }
    }

    #[test]
    fn unknown_field_on_method_ref_surfaces_typed_error() {
        let mut dag = generated_full_bootstrap_dag();
        mutate_method_ref(
            &mut dag,
            "rust_method_template_contracts",
            0,
            MethodRefMutation::AppendUnknown("nickname"),
        );
        let result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match result {
            Err(MethodTemplateProjectionError::RowUnknownField { record, field, .. }) => {
                assert_eq!(record, "MethodRef");
                assert_eq!(field, "nickname");
            }
            other => panic!("expected RowUnknownField for MethodRef, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_row_surfaces_typed_error_at_canonical_row_list_api() {
        // Per-target uniqueness by `dag_method` is the substrate-side claim
        // `method_template_contract_per_target_dag_method_unique`. The
        // canonical row-list API enforces this at the public boundary so
        // every consumer (direct row-list consumer or `(target, dag_method)`
        // lookup helper) observes the same fail-closed behavior. Gap 5 /
        // leaf-emit consumers building their own keyed tables from the
        // projection cannot silently observe two authorities for the same
        // `(target, MethodRef)` pair.
        let mut dag = generated_full_bootstrap_dag();
        duplicate_row(&mut dag, "rust_method_template_contracts", 0);

        // Canonical row-list API surfaces the duplicate.
        let list_result = method_template_contract_rows(&dag, MethodTemplateTarget::Rust);
        match list_result {
            Err(MethodTemplateProjectionError::DuplicateMethodTemplateRow {
                list,
                first_row_index,
                duplicate_row_index,
            }) => {
                assert_eq!(list, "rust_method_template_contracts");
                assert_ne!(first_row_index, duplicate_row_index);
            }
            other => panic!(
                "expected DuplicateMethodTemplateRow at row-list API for duplicate (target, dag_method), got {other:?}"
            ),
        }

        // Lookup helper inherits the fail-closed shape via the row-list
        // API; no separate uniqueness path on the helper.
        let count_method_id = dag
            .declaration_by_name("count_method")
            .expect("count_method MethodDeclaration in bootstrap Dag")
            .id;
        let helper_result =
            method_template_contract_row(&dag, MethodTemplateTarget::Rust, count_method_id);
        assert!(
            matches!(
                helper_result,
                Err(MethodTemplateProjectionError::DuplicateMethodTemplateRow { .. })
            ),
            "lookup helper must inherit duplicate-row fail-closed via row-list API; got {helper_result:?}"
        );
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
