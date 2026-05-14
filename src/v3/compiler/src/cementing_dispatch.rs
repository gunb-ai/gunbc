//! Band-C cementing dispatch — [`crate::test_runner::TestPredicate::CementingDispatchMatchesProjection`].
//!
//! SG-0 / INVARIANTS P5: hand-authored `src/v3/compiler/src/` module — register the path in
//! `EXPECTED_HAND_AUTHORED_NON_TEST` inside `tests/integration/sg0_census_test.rs` when this
//! file ships or changes materially.
//!
//! Projects `LensRegistryEntry` rows from the bootstrapped `regen.dag` authority against
//! `std.verification` `lens_capability_register_rows` (closed coproduct status axes per
//! `docs/design-tests-as-data-completeness.md` §8.3), then validates the Band-C receipt list
//! in `cementing_dispatch.dag` and on-disk harness artifacts.
//!
//! **Interim host coupling:** validating `temporary-rust` receipts scans `tests/integration.rs`
//! once per predicate evaluation (see `std::fs::read_to_string` below). Dissolution: consume
//! reflected wiring facts at the runner edge so Band-C dispatch does not depend on crate layout
//! + host FS for this check.
//!
//! **Interim projection bridge:** `expected_cementing_receipt_triples` is a fail-closed Rust match
//! from `(registry_name, regen lens_file basename)` to the canonical receipt triples the
//! `cementing_band_c_v2_complete_receipts` list must equal. Dissolution: move that expansion into
//! `.dag` data (fixture or `std.verification`) so the predicate reads a single structural receipt
//! roster keyed off the register ∩ `regen.dag` projection, with no parallel Rust roster.

use std::collections::{BTreeSet, HashSet};
use std::path::Path as FsPath;

use crate::dag::{Dag, FieldValue, LiteralBits, TypeConnective, ValueBody};
use crate::integration_rs_wiring_scan::integration_rs_cementing_path_attr_binds_mod_stem;
use crate::r3_gate_87_cementing_regen_runner_suites::r3_gate_87_cementing_regen_pb_b1_dag_module_stems;

fn record_field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|(candidate, _)| candidate == label)
        .map(|(_, value)| value)
}

fn string_field(fields: &[(String, FieldValue)], label: &str) -> Result<String, String> {
    match record_field(fields, label) {
        Some(FieldValue::Literal(LiteralBits::String(value))) => Ok(value.clone()),
        Some(other) => Err(format!("`{label}` must be a string literal, got {other:?}")),
        None => Err(format!("record is missing string field `{label}`")),
    }
}

fn disj_variants(
    dag: &Dag,
    type_name: &str,
) -> Result<Vec<(String, crate::dag::DeclarationId)>, String> {
    let sum_decl = dag.declaration_by_name(type_name).ok_or_else(|| {
        format!(
            "merged DAG missing nullary sum `{type_name}` — expected `std.verification` for lens \
             capability axes, or `tests/dag/cementing_dispatch.dag` for `CementingBandCReceiptKind`"
        )
    })?;
    let TypeConnective::Disj { variants } = &sum_decl.connective else {
        return Err(format!("`{type_name}` must be a Disj coproduct"));
    };
    Ok(variants.iter().map(|v| (v.label.clone(), v.ty)).collect())
}

fn decode_nullary_sum_variant(
    dag: &Dag,
    sum_type_name: &str,
    value: &FieldValue,
    field_label: &str,
    record_role: &str,
) -> Result<String, String> {
    let variants = disj_variants(dag, sum_type_name)?;
    let allowed: HashSet<crate::dag::DeclarationId> = variants.iter().map(|(_, id)| *id).collect();
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(format!(
            "{record_role} `{field_label}` must be a closed sum variant of `{sum_type_name}`; got {value:?}"
        ));
    };
    if !payload.is_empty() {
        return Err(format!(
            "{record_role} `{field_label}`: unexpected variant payload (expected nullary `{sum_type_name}` arm)"
        ));
    }
    if !allowed.contains(constructor) {
        return Err(format!(
            "{record_role} `{field_label}`: constructor id {:?} is not a declared variant of `{sum_type_name}`",
            constructor
        ));
    }
    variants
        .iter()
        .find(|(_, id)| id == constructor)
        .map(|(label, _)| label.clone())
        .ok_or_else(|| {
            format!(
                "{record_role} `{field_label}`: internal mismatch resolving `{sum_type_name}` variant"
            )
        })
}

/// Maps `CementingBandCReceiptKind` variant labels to the stable wire tags used in
/// `expected_cementing_receipt_triples` and on-disk dispatch branches.
fn cementing_receipt_kind_wire_tag(sum_variant_label: &str) -> Result<&'static str, String> {
    match sum_variant_label {
        "DagHarness" => Ok("dag"),
        "TemporaryRustModule" => Ok("temporary-rust"),
        other => Err(format!(
            "cementing receipt `kind`: unknown `CementingBandCReceiptKind` variant `{other}`"
        )),
    }
}

fn decode_cementing_band_c_receipt_kind(
    dag: &Dag,
    value: &FieldValue,
    field_label: &str,
) -> Result<&'static str, String> {
    let label = decode_nullary_sum_variant(
        dag,
        "CementingBandCReceiptKind",
        value,
        field_label,
        "cementing receipt row",
    )?;
    cementing_receipt_kind_wire_tag(&label)
}

fn record_fields(value: &FieldValue) -> Option<&[(String, FieldValue)]> {
    match value {
        FieldValue::Record(fields) => Some(fields.as_slice()),
        _ => None,
    }
}

fn resolve_declaration_ref_id(
    value: &FieldValue,
    field_label: &str,
) -> Result<crate::dag::DeclarationId, String> {
    match value {
        FieldValue::Reference(id) => Ok(*id),
        FieldValue::Record(fields) if fields.is_empty() => Err(format!(
            "`{field_label}`: DeclarationRef is the empty record literal {{}} — use an identifier \
             so lowering emits FieldValue::Reference(DeclarationId), not an empty record",
        )),
        other => Err(format!(
            "`{field_label}`: expected FieldValue::Reference(DeclarationId) \
             for a DeclarationRef edge, got {other:?}"
        )),
    }
}

fn list_items_of_declaration(
    dag: &Dag,
    id: crate::dag::DeclarationId,
    role: &str,
) -> Result<Vec<FieldValue>, String> {
    let decl = dag.declaration(id);
    match &decl.value_body {
        Some(ValueBody::List(items)) => Ok(items.clone()),
        other => Err(format!(
            "`{role}` must resolve to a `List<…>` data declaration; got {:?} for `{}`",
            other,
            decl.name.as_deref().unwrap_or("<anonymous>")
        )),
    }
}

fn read_lens_registry_name_lens_file_pairs(dag: &Dag) -> Result<Vec<(String, String)>, String> {
    let entry_type_id = dag
        .declaration_by_name("LensRegistryEntry")
        .map(|decl| decl.id)
        .ok_or_else(|| "bootstrap must declare `LensRegistryEntry` (regen.dag)".to_string())?;

    let mut pairs = Vec::new();
    for decl in dag.declarations() {
        if decl.meta_tag != Some(entry_type_id) {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            return Err(format!(
                "lens registry entry `{}` must carry a structural value body",
                decl.name.as_deref().unwrap_or("<anonymous>")
            ));
        };
        let binding = decl
            .name
            .clone()
            .unwrap_or_else(|| "<anonymous>".to_string());
        pairs.push((
            string_field(fields, "name")
                .map_err(|e| format!("lens registry entry `{binding}`: {e}"))?,
            string_field(fields, "lens_file")
                .map_err(|e| format!("lens registry entry `{binding}`: {e}"))?,
        ));
    }
    Ok(pairs)
}

/// Closed roster of Band-C `(registry_name, module_stem, kind)` receipts implied by the
/// current `regen.dag` × cementing register projection. When a new projected lens lands
/// in `regen.dag`, extend this expansion in the same PR as `cementing_dispatch.dag`
/// receipt rows — otherwise the dispatch predicate fail-closes with an explicit error.
fn expected_cementing_receipt_triples(
    registry_pairs: &[(String, String)],
    basenames: &BTreeSet<String>,
) -> Result<BTreeSet<(String, String, String)>, String> {
    let mut out = BTreeSet::new();
    for (name, lens_file) in registry_pairs {
        let basename = FsPath::new(lens_file)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                format!("registry entry `{name}` has lens_file without basename: {lens_file}")
            })?
            .to_string();
        if !basenames.contains(&basename) {
            continue;
        }
        match (name.as_str(), basename.as_str()) {
            ("cost", "complexity.dag") => {
                out.insert((
                    name.clone(),
                    "t_r3_gate_87_cementing_regen_cost".to_string(),
                    "dag".to_string(),
                ));
                out.insert((
                    name.clone(),
                    "complexity_lens_behavioral_completion".to_string(),
                    "temporary-rust".to_string(),
                ));
            }
            ("cost_symbolic", "cost.dag") => {
                out.insert((
                    name.clone(),
                    "t_r3_gate_87_cementing_regen_cost_symbolic".to_string(),
                    "dag".to_string(),
                ));
                out.insert((
                    name.clone(),
                    "cost_lens_symbolic_consumer_test".to_string(),
                    "temporary-rust".to_string(),
                ));
            }
            ("effect_enumeration", "effect_enumeration.dag") => {
                out.insert((
                    name.clone(),
                    "t_r3_gate_87_cementing_regen_effect_enumeration".to_string(),
                    "dag".to_string(),
                ));
            }
            _ => {
                return Err(format!(
                    "Band-C cementing projection includes `LensRegistryEntry` \
                     (`name={name}`, `lens_file` basename `{basename}`) but \
                     `expected_cementing_receipt_triples` has no receipt expansion — \
                     land matching `cementing_band_c_v2_complete_receipts` rows in \
                     `cementing_dispatch.dag` and extend the expansion table in \
                     `cementing_dispatch.rs` in the same PR."
                ));
            }
        }
    }
    Ok(out)
}

fn v2_cementing_basenames_from_capability_rows(
    dag: &Dag,
    capability_rows: &[FieldValue],
) -> Result<BTreeSet<String>, String> {
    let mut basenames = BTreeSet::new();
    for row in capability_rows {
        let Some(fields) = record_fields(row) else {
            return Err(format!(
                "capability_register list: expected record row, got {row:?}"
            ));
        };
        let lens_basename = string_field(fields, "lens_basename")?;
        let structural = record_field(fields, "structural").ok_or_else(|| {
            "capability_register row: missing `structural` field (expected \
             `LensCapabilityStructuralStatus` variant)"
                .to_string()
        })?;
        let behavioral = record_field(fields, "behavioral").ok_or_else(|| {
            "capability_register row: missing `behavioral` field (expected \
             `LensCapabilityBehavioralStatus` variant)"
                .to_string()
        })?;
        let v2 = record_field(fields, "v2_counterpart").ok_or_else(|| {
            "capability_register row: missing `v2_counterpart` field (expected \
             `LensCapabilityV2Counterpart` variant)"
                .to_string()
        })?;
        decode_nullary_sum_variant(
            dag,
            "LensCapabilityStructuralStatus",
            structural,
            "structural",
            "capability_register row",
        )?;
        let behavioral_label = decode_nullary_sum_variant(
            dag,
            "LensCapabilityBehavioralStatus",
            behavioral,
            "behavioral",
            "capability_register row",
        )?;
        let v2_label = decode_nullary_sum_variant(
            dag,
            "LensCapabilityV2Counterpart",
            v2,
            "v2_counterpart",
            "capability_register row",
        )?;
        let is_v2_complete = behavioral_label == "LensCapabilityBehavioralComplete"
            && v2_label == "LensCapabilityV2RealV2";
        let is_effect_enumeration_native_complete = behavioral_label
            == "LensCapabilityBehavioralComplete"
            && v2_label == "LensCapabilityV2NoneV3Native"
            && lens_basename == "effect_enumeration.dag";
        if is_v2_complete || is_effect_enumeration_native_complete {
            basenames.insert(lens_basename);
        }
    }
    Ok(basenames)
}

/// Lens basenames that participate in Band-C cementing: `LensCapabilityBehavioralComplete`
/// plus `LensCapabilityV2RealV2`, plus the v3-native effect-enumeration behavioral receipt,
/// in the canonical `std.verification` `lens_capability_register_rows` list (same projection
/// `CementingDispatchMatchesProjection` uses before intersecting `regen.dag`).
///
/// Exposed for integration tests that mechanically ratchet the markdown capability table against
/// this structural authority (`lens_register_correspondence_test`).
pub fn lens_capability_register_v2_cementing_basenames(
    dag: &Dag,
) -> Result<BTreeSet<String>, String> {
    let reg_decl = dag
        .declaration_by_name("lens_capability_register_rows")
        .ok_or_else(|| {
            "bootstrap must declare `lens_capability_register_rows` (std.verification)".to_string()
        })?;
    let capability_rows =
        list_items_of_declaration(dag, reg_decl.id, "lens_capability_register_rows")?;
    v2_cementing_basenames_from_capability_rows(dag, &capability_rows)
}

fn cementing_dispatch_two_refs(
    payload: &[FieldValue],
) -> Result<(&FieldValue, &FieldValue), String> {
    match payload {
        [a, b] => Ok((a, b)),
        [FieldValue::Record(fields)] => {
            let a = record_field(fields, "capability_register").ok_or_else(|| {
                "CementingDispatchMatchesProjection record missing `capability_register`"
                    .to_string()
            })?;
            let b = record_field(fields, "cementing_receipts").ok_or_else(|| {
                "CementingDispatchMatchesProjection record missing `cementing_receipts`".to_string()
            })?;
            Ok((a, b))
        }
        _ => Err(format!(
            "CementingDispatchMatchesProjection payload should be two DeclarationRef edges \
             or a record {{ capability_register, cementing_receipts }}; got {} slot(s)",
            payload.len()
        )),
    }
}

/// Evaluate `CementingDispatchMatchesProjection` for a claim declared in `declaration_file`.
///
/// Returns `Ok(())` on pass; `Err(reason)` maps to [`crate::test_runner::ClaimResult::Fail`].
pub(crate) fn evaluate_cementing_dispatch_projection(
    dag: &Dag,
    declaration_file: &str,
    payload: &[FieldValue],
) -> Result<(), String> {
    const CEMENTING_DISPATCH_DECL_PATH: &str = "cementing_dispatch.dag";
    if !declaration_file.ends_with(CEMENTING_DISPATCH_DECL_PATH) {
        return Err(format!(
            "CementingDispatchMatchesProjection is only accepted for claims declared in `**/{CEMENTING_DISPATCH_DECL_PATH}`; got `{declaration_file}`"
        ));
    }

    let (reg_ref, recv_ref) = cementing_dispatch_two_refs(payload)?;
    let reg_id = resolve_declaration_ref_id(reg_ref, "capability_register")?;
    let recv_id = resolve_declaration_ref_id(recv_ref, "cementing_receipts")?;

    let capability_rows = list_items_of_declaration(dag, reg_id, "capability_register")?;
    let receipt_rows = list_items_of_declaration(dag, recv_id, "cementing_receipts")?;

    let basenames = v2_cementing_basenames_from_capability_rows(dag, &capability_rows)?;

    let registry_pairs = read_lens_registry_name_lens_file_pairs(dag)?;

    let mut matched_basenames = BTreeSet::new();
    for (name, lens_file) in &registry_pairs {
        let Some(basename) = FsPath::new(lens_file).file_name().and_then(|s| s.to_str()) else {
            return Err(format!(
                "registry entry `{name}` has lens_file without basename: {lens_file}"
            ));
        };
        if basenames.contains(basename) {
            matched_basenames.insert(basename.to_string());
        }
    }

    let missing: Vec<_> = basenames.difference(&matched_basenames).cloned().collect();
    if !missing.is_empty() {
        return Err(format!(
            "`lens_capability_register` escalates v2 cementing for lens basenames {missing:?}, \
             but no `LensRegistryEntry` in bootstrap names those files — fix the register data or add registry entries."
        ));
    }

    let mut receipt_triples = BTreeSet::new();

    for row in &receipt_rows {
        let Some(fields) = record_fields(row) else {
            return Err(format!(
                "cementing_receipts list: expected record row, got {row:?}"
            ));
        };
        let registry_name = string_field(fields, "registry_name")?;
        let module_stem = string_field(fields, "module_stem")?;
        let kind_field = record_field(fields, "kind").ok_or_else(|| {
            "cementing receipt row: missing `kind` field (expected `CementingBandCReceiptKind` \
             variant)"
                .to_string()
        })?;
        let kind_str = decode_cementing_band_c_receipt_kind(dag, kind_field, "kind")?;
        let triple = (
            registry_name.clone(),
            module_stem.clone(),
            kind_str.to_string(),
        );
        if !receipt_triples.insert(triple.clone()) {
            return Err(format!(
                "cementing_receipts contains duplicate receipt identity {triple:?}; each \
                 `(registry_name, module_stem, kind)` must be unique."
            ));
        }
    }

    let expected_triples = expected_cementing_receipt_triples(&registry_pairs, &basenames)?;
    if receipt_triples != expected_triples {
        return Err(format!(
            "cementing_receipts `(registry_name, module_stem, kind)` triples must exactly match \
             the Band-C expansion of the register ∩ regen projection — expected {expected_triples:?}, \
             got {receipt_triples:?}"
        ));
    }

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let pb_b1_gate87_dag_stems = r3_gate_87_cementing_regen_pb_b1_dag_module_stems();
    let integration_rs = match std::fs::read_to_string(manifest_dir.join("tests/integration.rs")) {
        Ok(text) => text,
        Err(err) => {
            return Err(format!(
                "read tests/integration.rs for cementing wiring ratchet: {err}"
            ));
        }
    };

    for row in &receipt_rows {
        let Some(fields) = record_fields(row) else {
            return Err(format!(
                "cementing_receipts list: expected record row, got {row:?}"
            ));
        };
        let registry_name = string_field(fields, "registry_name")?;
        let module_stem = string_field(fields, "module_stem")?;
        let kind_field = record_field(fields, "kind").ok_or_else(|| {
            "cementing receipt row: missing `kind` field (expected `CementingBandCReceiptKind` \
             variant)"
                .to_string()
        })?;
        let kind_str = decode_cementing_band_c_receipt_kind(dag, kind_field, "kind")?;
        match kind_str {
            "dag" => {
                if !pb_b1_gate87_dag_stems.contains(&module_stem) {
                    return Err(format!(
                        "registry lens `{registry_name}` lists v2-complete cementing receipt `{module_stem}` \
                         as a `.dag` harness, but that stem is not wired in \
                         `r3_gate_87_cementing_regen_runner_suites::R3_GATE_87_CEMENTING_REGEN_SUITES` \
                         (T-PB-B-1 runner) — extend the shared table or fix the receipt list."
                    ));
                }
                let path = manifest_dir
                    .join("tests")
                    .join("dag")
                    .join(format!("{module_stem}.dag"));
                if !path.is_file() {
                    return Err(format!(
                        "registry lens `{registry_name}` lists v2-complete cementing receipt `{module_stem}` \
                         as a `.dag` harness; expected file {}",
                        path.display()
                    ));
                }
            }
            "temporary-rust" => {
                let path = manifest_dir
                    .join("tests")
                    .join("integration")
                    .join("cementing")
                    .join(format!("{module_stem}.rs"));
                if !path.is_file() {
                    return Err(format!(
                        "registry lens `{registry_name}` lists temporary Rust cementing receipt `{module_stem}`; \
                         expected file {}",
                        path.display()
                    ));
                }
                if !integration_rs_cementing_path_attr_binds_mod_stem(
                    &integration_rs,
                    &module_stem,
                )? {
                    let expected = format!(r#"#[path = "integration/cementing/{module_stem}.rs"]"#);
                    return Err(format!(
                        "registry lens `{registry_name}` lists temporary Rust cementing stem `{module_stem}` but \
                         tests/integration.rs does not bind `{expected}` to `mod {module_stem};` in the same item \
                         (Band-C dispatch)."
                    ));
                }
            }
            _ => {
                return Err(
                    "cementing receipt `kind`: internal error — wire tag not `dag` or `temporary-rust`"
                        .to_string(),
                );
            }
        }
    }

    Ok(())
}
