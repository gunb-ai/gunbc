use std::collections::HashMap;

use crate::dag::{Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};
use crate::parse::{self, SurfaceItem};
use crate::tokenize;

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
    pub(crate) stage: DeclarationId,
    pub(crate) stage_name: String,
    pub(crate) realization: DeclarationId,
    pub(crate) realization_name: String,
    pub(crate) snapshot_kind: PipelineSnapshotKind,
}

pub(crate) fn ordered_pipeline_stages(dag: &Dag) -> Result<Vec<PipelineStageAuthority>, String> {
    let mut bindings = pipeline_stage_bindings(dag)?;
    let stage_names = pipeline_compile_order_names()?;
    let mut ordered = Vec::with_capacity(stage_names.len());

    for stage_name in stage_names {
        let stage_id = dag
            .declaration_by_name(&stage_name)
            .map(|decl| decl.id)
            .ok_or_else(|| {
                format!("pipeline compile body references missing stage declaration `{stage_name}`")
            })?;
        let authority = bindings.remove(&stage_id).ok_or_else(|| {
            format!("pipeline stage `{stage_name}` is used by compile but has no stage binding")
        })?;
        ordered.push(authority);
    }

    if !bindings.is_empty() {
        let mut extras: Vec<_> = bindings
            .into_values()
            .map(|authority| authority.stage_name)
            .collect();
        extras.sort();
        return Err(format!(
            "pipeline stage bindings not used by compile: {}",
            extras.join(", ")
        ));
    }

    Ok(ordered)
}

fn pipeline_stage_bindings(
    dag: &Dag,
) -> Result<HashMap<DeclarationId, PipelineStageAuthority>, String> {
    let binding_type_id = dag
        .declaration_by_name(PIPELINE_STAGE_BINDING_TYPE)
        .map(|decl| decl.id)
        .ok_or_else(|| {
            format!("missing pipeline authority type `{PIPELINE_STAGE_BINDING_TYPE}`")
        })?;

    let mut bindings = HashMap::new();
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

        if bindings.contains_key(&stage) {
            return Err(format!(
                "pipeline stage `{stage_name}` has multiple stage bindings"
            ));
        }

        bindings.insert(
            stage,
            PipelineStageAuthority {
                stage,
                stage_name,
                realization,
                realization_name,
                snapshot_kind,
            },
        );
    }

    if bindings.is_empty() {
        return Err("pipeline authority contains no stage bindings".to_string());
    }

    Ok(bindings)
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

fn pipeline_compile_order_names() -> Result<Vec<String>, String> {
    let source = include_str!("../pipeline.dag");
    let tokens = tokenize::tokenize(source, PIPELINE_AUTHORITY_FILE)
        .map_err(|diag| format!("failed to tokenize pipeline authority: {diag:?}"))?;
    let module = parse::parse(&tokens, PIPELINE_AUTHORITY_FILE)
        .map_err(|diag| format!("failed to parse pipeline authority: {diag:?}"))?;

    let body_span = module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::FnExternalBody {
                name, body_span, ..
            } if name == PIPELINE_COMPILE_FN => Some(body_span.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            format!("pipeline authority is missing external-body fn `{PIPELINE_COMPILE_FN}`")
        })?;

    let body = source
        .get(body_span.byte_start as usize..body_span.byte_end as usize)
        .ok_or_else(|| "pipeline compile body span is out of bounds".to_string())?;
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
