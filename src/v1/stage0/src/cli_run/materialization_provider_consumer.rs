use crate::resolved_graph_cache::{
    faithful_probe_unavailable_gap, is_union_part_absent, supports_faithful_probe,
    FaithfulResolvedGraphProbeParts,
};
use crate::std_content_hash::fnv1a64_structural_hex_digest;
use crate::v1_interpreter::{self, ExecutionMode, InterpContext, Value};
use std::cell::RefCell;
use std::rc::Rc;

// SCAFFOLD (§7 seed-retained HAND-RUST — authority: gunbc.materialization_provider_consumer_scaffold;
// witness: dag/test/claim/materialization_provider_consumer_hand_rust_witness_test.dag).
const MATERIALIZATION_PROVIDER_ENTRY: &str = "dag/std/materialization_provider.dag";
pub const OUTPUT_COMPILE_CLEAN_DIAGNOSTIC_UNION: &str = "compile_clean_diagnostic_union";

/// Typed mirror of `std.materialization_provider` consumer outcomes for the
/// resolved-graph disk-hit seam. Each contract refusal variant is represented
/// distinctly so downstream callers never collapse typed refusals into a string
/// bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedGraphProviderOutcome {
    Hit,
    Miss,
    RefusedIncomplete { missing: Vec<String> },
    RefusedWrongArtifact,
    RefusedKindMismatch,
    RefusedWrongContent,
    RefusedCrossFamilyContentHash,
    LookupUnclassified { label: String },
}

thread_local! {
    static MATERIALIZATION_PROVIDER_CTX: RefCell<Option<Rc<InterpContext>>> =
        RefCell::new(None);
}

static MATERIALIZATION_PROVIDER_CTX_BUILD_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

fn build_materialization_provider_ctx_cold() -> Result<InterpContext, String> {
    MATERIALIZATION_PROVIDER_CTX_BUILD_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let roots = super::default_source_roots();
    let entry = super::resolve_entry_file_under_roots(&roots, MATERIALIZATION_PROVIDER_ENTRY)?;
    // Bootstrap must not re-enter cross-process provider routing while loading
    // the provider authority itself (review 44268: unbounded recursion on cache hit).
    let (graph, indices) = super::with_cross_process_provider_routing_suppressed(|| {
        super::resolve_entry_graph_shared(&roots, &entry)
    })?;
    Ok(super::make_eval_context(
        &graph,
        indices,
        ExecutionMode::Hermetic,
    ))
}

fn materialization_provider_ctx() -> Result<Rc<InterpContext>, String> {
    MATERIALIZATION_PROVIDER_CTX.with(|slot| {
        if let Some(ctx) = slot.borrow().clone() {
            return Ok(ctx);
        }
        let ctx = Rc::new(build_materialization_provider_ctx_cold()?);
        *slot.borrow_mut() = Some(ctx.clone());
        Ok(ctx)
    })
}

fn wire_digest_str(hex: &str) -> Result<Value, String> {
    if fnv1a64_structural_hex_digest(hex.to_string()).is_none() {
        return Err(format!("invalid fnv1a64 structural wire digest `{hex}`"));
    }
    Ok(Value::Str(hex.to_string()))
}

fn wire_fnv1a64_content_hash_value(ctx: &InterpContext, hex: &str) -> Result<Value, String> {
    let wire = wire_digest_str(hex)?;
    let args = [(Some("hex".to_string()), wire)];
    v1_interpreter::run_in_context_with_args(ctx, "wire_fnv1a64_content_hash_hex", &args, false)
        .map_err(|e| format!("wire_fnv1a64_content_hash_hex({hex}): {e}"))
}

fn wire_content_hash_to_hex(ctx: &InterpContext, hash: &Value) -> Result<String, String> {
    let args = [(Some("hash".to_string()), hash.clone())];
    match v1_interpreter::run_in_context_with_args(ctx, "wire_content_hash_to_hex", &args, false) {
        Ok(Value::Str(hex)) => Ok(hex),
        Ok(other) => Err(format!(
            "wire_content_hash_to_hex returned `{}`, expected String",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("wire_content_hash_to_hex: {e}")),
    }
}

fn wire_byte_size_value(ctx: &InterpContext, bytes: u64) -> Result<Value, String> {
    let args = [(Some("count".to_string()), Value::Int(bytes as i64))];
    v1_interpreter::run_in_context_with_args(ctx, "wire_byte_size", &args, false)
        .map_err(|e| format!("wire_byte_size({bytes}): {e}"))
}

fn lookup_fold_outcome(
    ctx: &InterpContext,
    lookup: &Value,
) -> Result<ResolvedGraphProviderOutcome, String> {
    let args = [(Some("l".to_string()), lookup.clone())];
    let outcome =
        v1_interpreter::run_in_context_with_args(ctx, "provider_lookup_outcome_tag", &args, false)
            .map_err(|e| format!("provider_lookup_outcome_tag: {e}"))?;
    match outcome {
        Value::Str(tag) => match tag.as_str() {
            "hit" => Ok(ResolvedGraphProviderOutcome::Hit),
            "miss" => Ok(ResolvedGraphProviderOutcome::Miss),
            "kind_mismatch" => Ok(ResolvedGraphProviderOutcome::RefusedKindMismatch),
            "wrong_artifact" => Ok(ResolvedGraphProviderOutcome::RefusedWrongArtifact),
            "wrong_content" => Ok(ResolvedGraphProviderOutcome::RefusedWrongContent),
            "cross_family_hash" => Ok(ResolvedGraphProviderOutcome::RefusedCrossFamilyContentHash),
            "incomplete" => {
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
                let missing = items
                    .iter()
                    .map(|item| match item {
                        Value::Str(s) => Ok(s.clone()),
                        other => Err(format!(
                            "lookup_refused_incomplete_missing element `{}` is not String",
                            ctx.format_value(other)
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ResolvedGraphProviderOutcome::RefusedIncomplete { missing })
            }
            other => Ok(ResolvedGraphProviderOutcome::LookupUnclassified {
                label: other.to_string(),
            }),
        },
        other => Err(format!(
            "provider_lookup_outcome_tag returned `{}`, expected String tag",
            ctx.format_value(&other)
        )),
    }
}

fn serve_resolved_graph_stored_disk_probe_in_ctx(
    ctx: &InterpContext,
    closure_digest: &str,
    compiler_digest: &str,
    stored_request_key: &str,
    stored_semantic_digest: &str,
    parts: &FaithfulResolvedGraphProbeParts,
) -> Result<ResolvedGraphProviderOutcome, String> {
    let args = [
        (
            Some("closure_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, closure_digest)?,
        ),
        (
            Some("compiler_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, compiler_digest)?,
        ),
        (
            Some("stored_request_key".to_string()),
            wire_fnv1a64_content_hash_value(ctx, stored_request_key)?,
        ),
        (
            Some("stored_semantic_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, stored_semantic_digest)?,
        ),
        (
            Some("graph_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, &parts.graph_digest)?,
        ),
        (
            Some("graph_bytes".to_string()),
            wire_byte_size_value(ctx, parts.graph_bytes)?,
        ),
        (
            Some("indices_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, &parts.indices_digest)?,
        ),
        (
            Some("indices_bytes".to_string()),
            wire_byte_size_value(ctx, parts.indices_bytes)?,
        ),
        (
            Some("union_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, &parts.union_digest)?,
        ),
        (
            Some("union_bytes".to_string()),
            wire_byte_size_value(ctx, parts.union_bytes)?,
        ),
    ];
    let lookup = v1_interpreter::run_in_context_with_args(
        ctx,
        "serve_resolved_graph_stored_disk_probe",
        &args,
        false,
    )
    .map_err(|e| format!("serve_resolved_graph_stored_disk_probe: {e}"))?;
    lookup_fold_outcome(ctx, &lookup)
}

fn serve_resolved_graph_incomplete_stored_disk_probe_in_ctx(
    ctx: &InterpContext,
    closure_digest: &str,
    compiler_digest: &str,
    stored_request_key: &str,
    graph_digest: &str,
    graph_bytes: u64,
    indices_digest: &str,
    indices_bytes: u64,
) -> Result<ResolvedGraphProviderOutcome, String> {
    let args = [
        (
            Some("closure_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, closure_digest)?,
        ),
        (
            Some("compiler_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, compiler_digest)?,
        ),
        (
            Some("stored_request_key".to_string()),
            wire_fnv1a64_content_hash_value(ctx, stored_request_key)?,
        ),
        (
            Some("graph_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, graph_digest)?,
        ),
        (
            Some("graph_bytes".to_string()),
            wire_byte_size_value(ctx, graph_bytes)?,
        ),
        (
            Some("indices_digest".to_string()),
            wire_fnv1a64_content_hash_value(ctx, indices_digest)?,
        ),
        (
            Some("indices_bytes".to_string()),
            wire_byte_size_value(ctx, indices_bytes)?,
        ),
    ];
    let lookup = v1_interpreter::run_in_context_with_args(
        ctx,
        "serve_resolved_graph_incomplete_stored_disk_probe",
        &args,
        false,
    )
    .map_err(|e| format!("serve_resolved_graph_incomplete_stored_disk_probe: {e}"))?;
    lookup_fold_outcome(ctx, &lookup)
}

fn provider_lookup_refusal_err(ctx: &InterpContext, lookup: &Value, prefix: &str) -> String {
    match lookup_fold_outcome(ctx, lookup) {
        Ok(outcome) => super::provider_integrity_refusal_message(outcome)
            .unwrap_or_else(|| format!("{prefix}: unexpected provider hit")),
        Err(e) => format!("{prefix}: {e}"),
    }
}

fn decode_content_hash_outcome_or_lookup_refusal(
    ctx: &InterpContext,
    value: &Value,
    outcome_type: &str,
    ready_variant: &str,
    ready_field: &str,
    refused_variant: &str,
    err_prefix: &str,
) -> Result<String, String> {
    match value {
        Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } => {
            if !ctx.sym_eq(*type_name, outcome_type) {
                return Err(format!(
                    "{} returned `{}`, expected {} outcome",
                    err_prefix,
                    ctx.format_value(value),
                    outcome_type
                ));
            }
            if ctx.sym_eq(*variant_name, ready_variant) {
                let ready = ctx
                    .field(fields, ready_field)
                    .unwrap_or_else(|| panic!("{err_prefix}: missing `{ready_field}` field"));
                wire_content_hash_to_hex(ctx, &ready)
            } else if ctx.sym_eq(*variant_name, refused_variant) {
                let lookup = ctx
                    .field(fields, "lookup")
                    .unwrap_or_else(|| panic!("{err_prefix}: missing `lookup` field"));
                Err(provider_lookup_refusal_err(ctx, &lookup, err_prefix))
            } else {
                Err(format!(
                    "{} returned `{}`, expected {} or {} outcome",
                    err_prefix,
                    ctx.format_value(value),
                    ready_variant,
                    refused_variant
                ))
            }
        }
        other => Err(format!(
            "{} returned `{}`, expected variant",
            err_prefix,
            ctx.format_value(other)
        )),
    }
}

pub fn resolve_closure_request_key_from_digests(
    closure_digest: &str,
    compiler_digest: &str,
) -> Result<String, String> {
    let ctx = materialization_provider_ctx()?;
    let args = [
        (
            Some("closure_digest".to_string()),
            wire_fnv1a64_content_hash_value(&ctx, closure_digest)?,
        ),
        (
            Some("compiler_digest".to_string()),
            wire_fnv1a64_content_hash_value(&ctx, compiler_digest)?,
        ),
    ];
    let outcome = v1_interpreter::run_in_context_with_args(
        &ctx,
        "resolve_closure_request_key_outcome",
        &args,
        false,
    )
    .map_err(|e| format!("resolve_closure_request_key_outcome: {e}"))?;
    decode_content_hash_outcome_or_lookup_refusal(
        &ctx,
        &outcome,
        "ResolveClosureRequestKeyOutcome",
        "ResolveClosureRequestKeyReady",
        "key",
        "ResolveClosureRequestKeyRefused",
        "resolve_closure_request_key_from_digests",
    )
}

pub fn resolved_graph_parts_semantic_digest(
    graph_digest: &str,
    graph_bytes: u64,
    indices_digest: &str,
    indices_bytes: u64,
    union_digest: &str,
    union_bytes: u64,
) -> Result<String, String> {
    let ctx = materialization_provider_ctx()?;
    let args = [
        (
            Some("graph_digest".to_string()),
            wire_fnv1a64_content_hash_value(&ctx, graph_digest)?,
        ),
        (
            Some("graph_bytes".to_string()),
            wire_byte_size_value(&ctx, graph_bytes)?,
        ),
        (
            Some("indices_digest".to_string()),
            wire_fnv1a64_content_hash_value(&ctx, indices_digest)?,
        ),
        (
            Some("indices_bytes".to_string()),
            wire_byte_size_value(&ctx, indices_bytes)?,
        ),
        (
            Some("union_digest".to_string()),
            wire_fnv1a64_content_hash_value(&ctx, union_digest)?,
        ),
        (
            Some("union_bytes".to_string()),
            wire_byte_size_value(&ctx, union_bytes)?,
        ),
    ];
    let outcome = v1_interpreter::run_in_context_with_args(
        &ctx,
        "resolved_graph_parts_semantic_digest_outcome",
        &args,
        false,
    )
    .map_err(|e| format!("resolved_graph_parts_semantic_digest_outcome: {e}"))?;
    decode_content_hash_outcome_or_lookup_refusal(
        &ctx,
        &outcome,
        "ResolvedGraphPartsSemanticDigestOutcome",
        "ResolvedGraphPartsSemanticDigestReady",
        "digest",
        "ResolvedGraphPartsSemanticDigestRefused",
        "resolved_graph_parts_semantic_digest",
    )
}

pub fn serve_resolved_graph_stored_disk_probe(
    closure_digest: &str,
    compiler_digest: &str,
    stored_request_key: &str,
    stored_semantic_digest: &str,
    parts: &FaithfulResolvedGraphProbeParts,
) -> Result<ResolvedGraphProviderOutcome, String> {
    if !supports_faithful_probe() {
        return Err(format!(
            "resolved-graph-cache provider refused faithful probe: {}",
            faithful_probe_unavailable_gap()
        ));
    }
    let ctx = materialization_provider_ctx()?;
    if is_union_part_absent(parts) {
        return serve_resolved_graph_incomplete_stored_disk_probe_in_ctx(
            &ctx,
            closure_digest,
            compiler_digest,
            stored_request_key,
            &parts.graph_digest,
            parts.graph_bytes,
            &parts.indices_digest,
            parts.indices_bytes,
        );
    }
    serve_resolved_graph_stored_disk_probe_in_ctx(
        &ctx,
        closure_digest,
        compiler_digest,
        stored_request_key,
        stored_semantic_digest,
        parts,
    )
}

pub fn serve_resolved_graph_stored_disk_probe_for_test(
    closure_digest: &str,
    compiler_digest: &str,
    stored_request_key: &str,
    stored_semantic_digest: &str,
    parts: &FaithfulResolvedGraphProbeParts,
) -> Result<ResolvedGraphProviderOutcome, String> {
    serve_resolved_graph_stored_disk_probe(
        closure_digest,
        compiler_digest,
        stored_request_key,
        stored_semantic_digest,
        parts,
    )
}

#[doc(hidden)]
pub fn materialization_provider_ctx_build_count_for_test() -> usize {
    MATERIALIZATION_PROVIDER_CTX_BUILD_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

#[doc(hidden)]
pub fn reset_materialization_provider_ctx_for_test() {
    MATERIALIZATION_PROVIDER_CTX.with(|slot| *slot.borrow_mut() = None);
    MATERIALIZATION_PROVIDER_CTX_BUILD_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}
