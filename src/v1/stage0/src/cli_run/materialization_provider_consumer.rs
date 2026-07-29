use crate::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

const MATERIALIZATION_PROVIDER_ENTRY: &str = "dag/std/materialization_provider.dag";
pub const OUTPUT_COMPILE_CLEAN_DIAGNOSTIC_UNION: &str = "compile_clean_diagnostic_union";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedGraphProviderOutcome {
    Hit,
    Miss,
    RefusedIncomplete { missing: Vec<String> },
    RefusedWrongArtifact,
    OtherRefusal { label: String },
}

fn build_materialization_provider_ctx() -> Result<InterpContext, String> {
    let roots = super::default_source_roots();
    let entry = super::resolve_entry_file_under_roots(&roots, MATERIALIZATION_PROVIDER_ENTRY)?;
    let (graph, indices) = super::resolve_entry_graph_shared(&roots, &entry)?;
    Ok(super::make_eval_context(
        &graph,
        indices,
        ExecutionMode::Hermetic,
    ))
}

fn call_lookup_bool(ctx: &InterpContext, fn_name: &str, lookup: &Value) -> Result<bool, String> {
    let args = [(Some("l".to_string()), lookup.clone())];
    match v1_interpreter::run_in_context_with_args(ctx, fn_name, &args, false) {
        Ok(Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "{fn_name} returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("{fn_name}: {e}")),
    }
}

fn lookup_missing_outputs(ctx: &InterpContext, lookup: &Value) -> Result<Vec<String>, String> {
    let args = [(Some("l".to_string()), lookup.clone())];
    let missing = v1_interpreter::run_in_context_with_args(
        ctx,
        "lookup_refused_incomplete_missing",
        &args,
        false,
    )
    .map_err(|e| format!("lookup_refused_incomplete_missing: {e}"))?;
    let Value::List(items) = missing else {
        return Err(format!(
            "lookup_refused_incomplete_missing returned `{}`, expected List",
            ctx.format_value(&missing)
        ));
    };
    Ok(items
        .iter()
        .map(|item| match item {
            Value::Str(s) => Ok(s.clone()),
            other => Err(format!(
                "lookup_refused_incomplete_missing element `{}` is not String",
                ctx.format_value(other)
            )),
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn lookup_outcome_label(ctx: &InterpContext, lookup: &Value) -> String {
    if call_lookup_bool(ctx, "lookup_is_hit", lookup).unwrap_or(false) {
        return "ProviderHit".to_string();
    }
    if call_lookup_bool(ctx, "lookup_is_miss", lookup).unwrap_or(false) {
        return "ProviderMiss".to_string();
    }
    if call_lookup_bool(ctx, "lookup_is_refused_kind_mismatch", lookup).unwrap_or(false) {
        return "ProviderRefusedKindMismatch".to_string();
    }
    if call_lookup_bool(ctx, "lookup_is_refused_wrong_artifact", lookup).unwrap_or(false) {
        return "ProviderRefusedWrongArtifact".to_string();
    }
    if call_lookup_bool(ctx, "lookup_is_refused_wrong_content", lookup).unwrap_or(false) {
        return "ProviderRefusedWrongContent".to_string();
    }
    if call_lookup_bool(ctx, "lookup_is_refused_identity_unshareable", lookup).unwrap_or(false) {
        return "ProviderRefusedIdentityUnshareable".to_string();
    }
    if call_lookup_bool(ctx, "lookup_is_refused_identity_grade_unsupported", lookup)
        .unwrap_or(false)
    {
        return "ProviderRefusedIdentityGradeUnsupported".to_string();
    }
    if let Ok(missing) = lookup_missing_outputs(ctx, lookup) {
        if !missing.is_empty() {
            return format!("ProviderRefusedIncomplete({missing:?})");
        }
    }
    format!("ProviderLookup({})", ctx.format_value(lookup))
}

pub fn interpret_provider_lookup(
    ctx: &InterpContext,
    lookup: Value,
) -> Result<ResolvedGraphProviderOutcome, String> {
    if call_lookup_bool(ctx, "lookup_is_hit", &lookup)? {
        return Ok(ResolvedGraphProviderOutcome::Hit);
    }
    if call_lookup_bool(ctx, "lookup_is_miss", &lookup)? {
        return Ok(ResolvedGraphProviderOutcome::Miss);
    }
    if call_lookup_bool(ctx, "lookup_is_refused_wrong_artifact", &lookup)? {
        return Ok(ResolvedGraphProviderOutcome::RefusedWrongArtifact);
    }
    let missing = lookup_missing_outputs(ctx, &lookup)?;
    if !missing.is_empty() {
        return Ok(ResolvedGraphProviderOutcome::RefusedIncomplete { missing });
    }
    Ok(ResolvedGraphProviderOutcome::OtherRefusal {
        label: lookup_outcome_label(ctx, &lookup),
    })
}

fn serve_resolved_graph_v1_disk_probe_in_ctx(
    ctx: &InterpContext,
    closure_digest: &str,
    compiler_digest: &str,
    content_digest: &str,
    payload_byte_count: u64,
) -> Result<ResolvedGraphProviderOutcome, String> {
    let args = [
        (
            Some("closure_digest".to_string()),
            Value::Str(closure_digest.to_string()),
        ),
        (
            Some("compiler_digest".to_string()),
            Value::Str(compiler_digest.to_string()),
        ),
        (
            Some("content_digest".to_string()),
            Value::Str(content_digest.to_string()),
        ),
        (
            Some("payload_byte_count".to_string()),
            Value::Int(i64::try_from(payload_byte_count).map_err(|_| {
                "payload_byte_count exceeds i64 range for provider probe".to_string()
            })?),
        ),
    ];
    let lookup = v1_interpreter::run_in_context_with_args(
        ctx,
        "serve_resolved_graph_v1_disk_probe",
        &args,
        false,
    )
    .map_err(|e| format!("serve_resolved_graph_v1_disk_probe: {e}"))?;
    interpret_provider_lookup(ctx, lookup)
}

pub fn serve_resolved_graph_v1_disk_probe(
    closure_digest: &str,
    compiler_digest: &str,
    content_digest: &str,
    payload_byte_count: u64,
) -> Result<ResolvedGraphProviderOutcome, String> {
    let ctx = build_materialization_provider_ctx()?;
    serve_resolved_graph_v1_disk_probe_in_ctx(
        &ctx,
        closure_digest,
        compiler_digest,
        content_digest,
        payload_byte_count,
    )
}

pub fn serve_resolved_graph_v1_disk_probe_for_test(
    closure_digest: &str,
    compiler_digest: &str,
    content_digest: &str,
    payload_byte_count: u64,
) -> Result<ResolvedGraphProviderOutcome, String> {
    serve_resolved_graph_v1_disk_probe(
        closure_digest,
        compiler_digest,
        content_digest,
        payload_byte_count,
    )
}
