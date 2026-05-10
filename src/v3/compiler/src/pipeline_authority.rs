use crate::dag::{Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};

#[cfg(test)]
pub(crate) const PIPELINE_AUTHORITY_FILE: &str = "src/v3/compiler/pipeline.dag";

const PIPELINE_STAGE_BINDING_TYPE: &str = "PipelineStageBinding";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PipelineSnapshotKind {
    Surface,
    Dag,
    Text,
}

#[derive(Debug, Clone)]
pub(crate) struct PipelineStageAuthority {
    // Read by `bootstrap::materialize_pipeline_realizations` when feature
    // `bootstrap-regen-fresh` is enabled.
    #[cfg_attr(not(feature = "bootstrap-regen-fresh"), allow(dead_code))]
    pub(crate) stage: DeclarationId,
    pub(crate) stage_name: String,
    #[cfg_attr(not(feature = "bootstrap-regen-fresh"), allow(dead_code))]
    pub(crate) realization: DeclarationId,
    #[cfg_attr(not(feature = "bootstrap-regen-fresh"), allow(dead_code))]
    pub(crate) realization_name: String,
    pub(crate) snapshot_kind: PipelineSnapshotKind,
}

/// Pipeline stage order, read structurally from `PipelineStageBinding`
/// declarations in the Dag. The declaration order of the bindings in
/// `pipeline.dag` is the **sole runtime ordering authority** on this path.
///
/// **DB-16 case 2c / T-Bridge-Retirement disposition (PB review, PR #1171,
/// 2026-04-29):** `fn compile { … }` still lowers to `ArrowBody::Unparsed` — the
/// lowered Dag does **not** carry an ordered list of stage names inside the
/// compile arrow. Without that structural witness, a fail-closed cross-check
/// against the human-readable `compile` body cannot use the substrate query
/// surface alone. Prior attempts used either compile-time embedding of
/// `../pipeline.dag`
/// (compile-time embed) or `std::fs::read_to_string` + span slicing (runtime
/// source-text side channel); both are rejected for `bridge_include_str_side_channels_retired`
/// in this lane. **Compile-body vs binding drift detection is therefore
/// suspended** until derivation provides a single authored carrier or a
/// lowered structural compile-body representation. Callers must keep
/// `PipelineStageBinding` declaration order and the `compile` orchestrator body
/// in sync by review/regen discipline until then.
pub(crate) fn ordered_pipeline_stages(dag: &Dag) -> Result<Vec<PipelineStageAuthority>, String> {
    let binding_type_id = dag
        .declaration_by_name(PIPELINE_STAGE_BINDING_TYPE)
        .map(|decl| decl.id)
        .ok_or_else(|| {
            format!("missing pipeline authority type `{PIPELINE_STAGE_BINDING_TYPE}`")
        })?;

    let mut ordered: Vec<PipelineStageAuthority> = Vec::new();
    let mut seen_stages: Vec<DeclarationId> = Vec::new();

    for declaration in dag.declarations() {
        if declaration.meta_tag != Some(binding_type_id) {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &declaration.value_body else {
            let binding_name = declaration
                .name
                .as_deref()
                .unwrap_or("<anonymous pipeline stage binding>");
            return Err(format!(
                "pipeline stage binding `{binding_name}` must carry a structural value body"
            ));
        };

        let stage = require_decl_ref(fields, "stage", declaration.name.as_deref())?;
        let realization = require_decl_ref(fields, "realization", declaration.name.as_deref())?;
        let snapshot_kind = require_snapshot_kind(dag, fields, declaration.name.as_deref())?;

        let stage_name = dag.declaration(stage).name.clone().ok_or_else(|| {
            format!(
                "pipeline stage binding `{}` points at unnamed stage declaration `{}`",
                declaration
                    .name
                    .as_deref()
                    .unwrap_or("<anonymous pipeline stage binding>"),
                stage.raw()
            )
        })?;
        let realization_name = dag.declaration(realization).name.clone().ok_or_else(|| {
            format!(
                "pipeline stage binding `{}` points at unnamed realization declaration `{}`",
                declaration
                    .name
                    .as_deref()
                    .unwrap_or("<anonymous pipeline stage binding>"),
                realization.raw()
            )
        })?;

        if seen_stages.contains(&stage) {
            return Err(format!(
                "pipeline stage `{stage_name}` has multiple stage bindings"
            ));
        }
        seen_stages.push(stage);

        ordered.push(PipelineStageAuthority {
            stage,
            stage_name,
            realization,
            realization_name,
            snapshot_kind,
        });
    }

    if ordered.is_empty() {
        return Err("pipeline authority contains no stage bindings".to_string());
    }

    Ok(ordered)
}

/// Pipeline stage names in authority order. Thin wrapper over
/// `ordered_pipeline_stages` for callers that only need the names.
pub(crate) fn pipeline_compile_order_names(dag: &Dag) -> Result<Vec<String>, String> {
    Ok(ordered_pipeline_stages(dag)?
        .into_iter()
        .map(|stage| stage.stage_name)
        .collect())
}

fn require_decl_ref(
    fields: &[(String, FieldValue)],
    label: &str,
    binding_name: Option<&str>,
) -> Result<DeclarationId, String> {
    fields
        .iter()
        .find(|(field, _)| field == label)
        .and_then(|(_, value)| match value {
            FieldValue::Reference(id) => Some(*id),
            _ => None,
        })
        .ok_or_else(|| {
            format!(
                "pipeline stage binding `{}` is missing required DeclarationRef field `{label}`",
                binding_name.unwrap_or("<anonymous pipeline stage binding>")
            )
        })
}

fn require_snapshot_kind(
    dag: &Dag,
    fields: &[(String, FieldValue)],
    binding_name: Option<&str>,
) -> Result<PipelineSnapshotKind, String> {
    let snapshot_type_id = dag
        .declaration_by_name("PipelineSnapshotKind")
        .map(|decl| decl.id)
        .ok_or_else(|| "missing pipeline authority type `PipelineSnapshotKind`".to_string())?;
    let value = fields
        .iter()
        .find(|(field, _)| field == "snapshot")
        .map(|(_, value)| value)
        .ok_or_else(|| {
            format!(
                "pipeline stage binding `{}` is missing required `snapshot` field",
                binding_name.unwrap_or("<anonymous pipeline stage binding>")
            )
        })?;

    let constructor = match value {
        FieldValue::Variant {
            constructor,
            payload,
        } => {
            if !payload.is_empty() {
                return Err(format!(
                    "pipeline stage binding `{}` has non-empty snapshot payload",
                    binding_name.unwrap_or("<anonymous pipeline stage binding>")
                ));
            }
            *constructor
        }
        _ => {
            return Err(format!(
            "pipeline stage binding `{}` must use a PipelineSnapshotKind variant for `snapshot`",
            binding_name.unwrap_or("<anonymous pipeline stage binding>")
        ))
        }
    };

    let TypeConnective::Disj { variants } = &dag.declaration(snapshot_type_id).connective else {
        return Err(
            "pipeline authority type `PipelineSnapshotKind` must lower to a sum".to_string(),
        );
    };
    let label = variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .map(|variant| variant.label.as_str())
        .ok_or_else(|| {
            format!(
                "pipeline stage binding `{}` uses snapshot constructor `{}` that does not belong to `PipelineSnapshotKind`",
                binding_name.unwrap_or("<anonymous pipeline stage binding>"),
                constructor.raw()
            )
        })?;

    match label {
        "SnapshotSurface" => Ok(PipelineSnapshotKind::Surface),
        "SnapshotDag" => Ok(PipelineSnapshotKind::Dag),
        "SnapshotText" => Ok(PipelineSnapshotKind::Text),
        other => Err(format!(
            "pipeline stage binding `{}` uses unknown snapshot constructor `{other}`",
            binding_name.unwrap_or("<anonymous pipeline stage binding>")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::ArrowBody;

    const PIPELINE_COMPILE_FN: &str = "compile";

    #[test]
    fn ordered_pipeline_stages_authority_is_pipeline_stage_binding_only() {
        let dag = Dag::new();
        let names = pipeline_compile_order_names(&dag).expect("bootstrap pipeline stages");
        assert_eq!(
            names,
            vec![
                "parse".to_string(),
                "lower".to_string(),
                "infer".to_string(),
                "compute_ownership".to_string(),
                "lens_complexity".to_string(),
                "emit".to_string(),
            ],
            "ordering is structural declaration order of PipelineStageBinding rows; update this test when pipeline.dag stage set changes"
        );
    }

    #[test]
    fn pipeline_compile_body_remains_unparsed_blocking_structural_retirement() {
        let dag = Dag::new();
        let compile = dag
            .declarations()
            .iter()
            .find(|decl| {
                decl.name.as_deref() == Some(PIPELINE_COMPILE_FN)
                    && decl.span.file == PIPELINE_AUTHORITY_FILE
            })
            .expect("pipeline `compile` declaration");
        let TypeConnective::Arrow { body, .. } = &compile.connective else {
            panic!("`compile` must be an arrow declaration");
        };
        assert!(
            matches!(body, ArrowBody::Unparsed(_)),
            "case 2c: `compile` has no lowered ordered stage list — R3 bridge_include_str_side_channels_retired cannot be satisfied here without a new substrate fact; see module doc on ordered_pipeline_stages"
        );
    }
}
