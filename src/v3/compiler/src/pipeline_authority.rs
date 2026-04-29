use std::path::Path;

use crate::dag::{
    ArrowBody, Dag, Declaration, DeclarationId, FieldValue, TypeConnective, ValueBody,
};

pub(crate) const PIPELINE_AUTHORITY_FILE: &str = "src/v3/compiler/pipeline.dag";

const PIPELINE_STAGE_BINDING_TYPE: &str = "PipelineStageBinding";
const PIPELINE_COMPILE_FN: &str = "compile";

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
/// `pipeline.dag` is the ordering authority.
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

    reconcile_with_compile_body(dag, &ordered)?;

    Ok(ordered)
}

/// Fail-closed cross-check: the `fn compile { ... }` body in
/// `pipeline.dag` must list the same stages, in the same order, as the
/// `PipelineStageBinding` declarations. The bindings are the runtime
/// ordering authority; this check ensures the `compile` orchestrator
/// surface cannot silently drift from that authority. Any divergence is
/// surfaced as a bootstrap diagnostic (via the caller).
///
/// **T-Bridge-Retirement (`include_str!` side channel):** stage names in
/// the `compile` body are read using the `ArrowBody::Unparsed` span already
/// carried on the lowered `compile` declaration (DB-16 case 2c), then
/// sliced from the authoritative on-disk `pipeline.dag` next to this crate.
/// That retires `include_str!("../pipeline.dag")` while case 2c still
/// keeps two authored carriers in sync until derivation collapses them.
fn reconcile_with_compile_body(
    dag: &Dag,
    ordered: &[PipelineStageAuthority],
) -> Result<(), String> {
    let body_names = compile_body_stage_names_from_dag(dag)?;
    let binding_names: Vec<&str> = ordered
        .iter()
        .map(|stage| stage.stage_name.as_str())
        .collect();
    let body_refs: Vec<&str> = body_names.iter().map(|name| name.as_str()).collect();
    if binding_names != body_refs {
        return Err(format!(
            "pipeline authority drift: `fn compile` body lists [{}] but \
             `PipelineStageBinding` declaration order is [{}]. The bindings \
             are the runtime authority — update `fn compile` to match, or \
             reorder the bindings to match `fn compile`.",
            body_refs.join(", "),
            binding_names.join(", ")
        ));
    }
    Ok(())
}

fn pipeline_compile_declaration(dag: &Dag) -> Result<&Declaration, String> {
    dag.declarations()
        .iter()
        .filter(|decl| {
            decl.name.as_deref() == Some(PIPELINE_COMPILE_FN)
                && decl.span.file == PIPELINE_AUTHORITY_FILE
        })
        .max_by_key(|decl| decl.id.raw())
        .ok_or_else(|| {
            format!(
                "pipeline authority `{}` is missing fn `{PIPELINE_COMPILE_FN}`",
                PIPELINE_AUTHORITY_FILE
            )
        })
}

fn read_pipeline_authority_text() -> Result<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("pipeline.dag");
    std::fs::read_to_string(&path).map_err(|err| {
        format!(
            "failed to read pipeline authority substrate `{}`: {err}",
            path.display()
        )
    })
}

fn compile_body_stage_names_from_dag(dag: &Dag) -> Result<Vec<String>, String> {
    let compile = pipeline_compile_declaration(dag)?;
    let TypeConnective::Arrow { body, .. } = &compile.connective else {
        return Err(format!(
            "pipeline `{PIPELINE_COMPILE_FN}` must be an arrow declaration"
        ));
    };
    let ArrowBody::Unparsed(span) = body else {
        return Err(format!(
            "pipeline `{PIPELINE_COMPILE_FN}` must carry Unparsed body until case 2c derivation; found non-Unparsed body"
        ));
    };
    if span.file != PIPELINE_AUTHORITY_FILE {
        return Err(format!(
            "pipeline `{PIPELINE_COMPILE_FN}` Unparsed span file `{}` does not match `{}`",
            span.file, PIPELINE_AUTHORITY_FILE
        ));
    }

    let source = read_pipeline_authority_text()?;
    let body = source
        .get(span.byte_start as usize..span.byte_end as usize)
        .ok_or_else(|| {
            "pipeline compile body span is out of bounds for on-disk pipeline.dag".to_string()
        })?;

    compile_body_stage_names_from_braced_block(body)
}

fn compile_body_stage_names_from_braced_block(body: &str) -> Result<Vec<String>, String> {
    let body = body.trim();
    let body = body
        .strip_prefix('{')
        .and_then(|body| body.strip_suffix('}'))
        .ok_or_else(|| "pipeline compile body must be a braced block".to_string())?;

    let mut stages = Vec::new();
    for line in body.lines() {
        let candidate = line.trim();
        if candidate.is_empty() || candidate.starts_with("//") {
            continue;
        }
        if !candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return Err(format!(
                "pipeline compile body contains unsupported stage expression `{candidate}`"
            ));
        }
        stages.push(candidate.to_string());
    }

    if stages.is_empty() {
        return Err("pipeline compile body does not list any stages".to_string());
    }

    Ok(stages)
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
    use std::fs;

    use super::*;

    fn bootstrapped_stages() -> (Dag, Vec<PipelineStageAuthority>) {
        // Bootstrapping includes reconcile_with_compile_body, so any Dag
        // we build by hand would already match. Start from a Dag that
        // passes reconciliation and mutate the returned vector.
        let dag = Dag::new();
        let stages = ordered_pipeline_stages(&dag).expect("bootstrap pipeline authority is clean");
        (dag, stages)
    }

    #[test]
    fn pipeline_authority_does_not_use_include_str_for_pipeline_dag() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("pipeline_authority.rs");
        let source = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("read {} for include_str ratchet: {err}", path.display());
        });
        assert!(
            !source.contains("include_str!"),
            "T-Bridge-Retirement: pipeline_authority.rs must not embed pipeline.dag via include_str!; \
             use Dag-anchored Unparsed spans + on-disk substrate read instead ({})",
            path.display()
        );
    }

    #[test]
    fn reconcile_rejects_reordered_binding_list() {
        let (dag, mut stages) = bootstrapped_stages();
        assert!(stages.len() >= 2, "need >=2 stages to reorder");
        stages.swap(0, 1);
        let err = reconcile_with_compile_body(&dag, &stages)
            .expect_err("reordered bindings must be rejected");
        assert!(
            err.contains("pipeline authority drift"),
            "expected drift diagnostic, got: {err}"
        );
    }

    #[test]
    fn reconcile_accepts_matching_order() {
        let (dag, stages) = bootstrapped_stages();
        reconcile_with_compile_body(&dag, &stages).expect("matching order must reconcile");
    }
}
