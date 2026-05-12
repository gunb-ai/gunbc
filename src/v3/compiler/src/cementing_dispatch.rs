//! Band-C cementing dispatch — [`crate::test_runner::TestPredicate::CementingDispatchMatchesProjection`].
//!
//! SG-0 / INVARIANTS P5: hand-authored `src/v3/compiler/src/` module — register the path in
//! `EXPECTED_HAND_AUTHORED_NON_TEST` inside `tests/integration/sg0_census_test.rs` when this
//! file ships or changes materially.
//!
//! Projects `LensRegistryEntry` rows from the bootstrapped `regen.dag` authority against
//! structured lens-capability rows declared in `cementing_dispatch.dag`, then validates
//! the Band-C receipt list and on-disk harness artifacts.

use std::collections::BTreeSet;
use std::path::Path as FsPath;

use crate::dag::{Dag, FieldValue, LiteralBits, ValueBody};
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

    let mut basenames = BTreeSet::new();
    for row in &capability_rows {
        let Some(fields) = record_fields(row) else {
            return Err(format!(
                "capability_register list: expected record row, got {row:?}"
            ));
        };
        let lens_basename = string_field(fields, "lens_basename")?;
        let behavioral = string_field(fields, "behavioral")?;
        let v2 = string_field(fields, "v2_counterpart")?;
        if behavioral == "Complete" && v2 == "RealV2" {
            basenames.insert(lens_basename);
        }
    }

    let registry_pairs = read_lens_registry_name_lens_file_pairs(dag)?;

    let mut expected_registry_names = BTreeSet::new();
    let mut matched_basenames = BTreeSet::new();
    for (name, lens_file) in &registry_pairs {
        let Some(basename) = FsPath::new(lens_file).file_name().and_then(|s| s.to_str()) else {
            return Err(format!(
                "registry entry `{name}` has lens_file without basename: {lens_file}"
            ));
        };
        if basenames.contains(basename) {
            expected_registry_names.insert(name.clone());
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

    let mut declared_names = BTreeSet::new();
    let mut receipt_triples = BTreeSet::new();
    let mut cost_receipts: BTreeSet<(String, String)> = BTreeSet::new();
    let mut cost_symbolic_receipts: BTreeSet<(String, String)> = BTreeSet::new();

    for row in &receipt_rows {
        let Some(fields) = record_fields(row) else {
            return Err(format!(
                "cementing_receipts list: expected record row, got {row:?}"
            ));
        };
        let registry_name = string_field(fields, "registry_name")?;
        let module_stem = string_field(fields, "module_stem")?;
        let kind_str = string_field(fields, "kind")?;
        if kind_str != "dag" && kind_str != "temporary-rust" {
            return Err(format!(
                "cementing receipt `kind` must be `dag` or `temporary-rust`, got `{kind_str}`"
            ));
        }
        let triple = (registry_name.clone(), module_stem.clone(), kind_str.clone());
        if !receipt_triples.insert(triple.clone()) {
            return Err(format!(
                "cementing_receipts contains duplicate receipt identity {triple:?}; each \
                 `(registry_name, module_stem, kind)` must be unique."
            ));
        }
        declared_names.insert(registry_name.clone());
        match registry_name.as_str() {
            "cost" => {
                cost_receipts.insert((module_stem.clone(), kind_str.clone()));
            }
            "cost_symbolic" => {
                cost_symbolic_receipts.insert((module_stem.clone(), kind_str.clone()));
            }
            _ => {}
        }
    }

    if declared_names != expected_registry_names {
        return Err(format!(
            "cementing_receipts registry `name` keys must equal the projection from capability_register ∩ regen.dag — \
             expected {expected_registry_names:?}, got {declared_names:?}"
        ));
    }

    let expected_cost = BTreeSet::from([
        (
            "t_r3_gate_87_cementing_regen_cost".to_string(),
            "dag".to_string(),
        ),
        (
            "complexity_lens_behavioral_completion".to_string(),
            "temporary-rust".to_string(),
        ),
    ]);
    if cost_receipts != expected_cost {
        return Err(format!(
            "`cost` registry key must keep both the gate #87 `.dag` harness and the temporary \
             `ComplexitySummary` Rust receipt until `.dag` TestClaims can express nested carriers; \
             expected {expected_cost:?}, got {cost_receipts:?}"
        ));
    }

    let expected_cost_symbolic = BTreeSet::from([(
        "cost_lens_symbolic_consumer_test".to_string(),
        "temporary-rust".to_string(),
    )]);
    if cost_symbolic_receipts != expected_cost_symbolic {
        return Err(format!(
            "`cost_symbolic` must keep an explicit Band-C Rust receipt while `.dag` TestClaims \
             cannot express nested `SymbolicCost` / `SizeVariable` expected values; \
             expected {expected_cost_symbolic:?}, got {cost_symbolic_receipts:?}"
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
        let kind_str = string_field(fields, "kind")?;
        match kind_str.as_str() {
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
                if !integration_rs_cementing_path_attr_binds_mod_stem(&integration_rs, &module_stem)
                {
                    let expected = format!(r#"#[path = "integration/cementing/{module_stem}.rs"]"#);
                    return Err(format!(
                        "registry lens `{registry_name}` lists temporary Rust cementing stem `{module_stem}` but \
                         tests/integration.rs does not bind `{expected}` to `mod {module_stem};` in the same item \
                         (Band-C dispatch)."
                    ));
                }
            }
            other => {
                return Err(format!(
                    "cementing receipt kind `{other}` is not supported for on-disk validation"
                ));
            }
        }
    }

    Ok(())
}
