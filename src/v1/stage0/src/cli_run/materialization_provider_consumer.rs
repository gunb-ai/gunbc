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

fn byte_size_value_in_ctx(ctx: &InterpContext, count: u64) -> Result<Value, String> {
    let count_i64 = i64::try_from(count)
        .map_err(|_| "payload byte count exceeds i64 range for ByteSize carrier".to_string())?;
    let args = [(Some("count".to_string()), Value::Int(count_i64))];
    v1_interpreter::run_in_context_with_args(ctx, "byte_size", &args, false)
        .map_err(|e| format!("byte_size: {e}"))
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

fn lookup_outcome_label(ctx: &InterpContext, lookup: &Value) -> Result<String, String> {
    if call_lookup_bool(ctx, "lookup_is_hit", lookup)? {
        return Ok("ProviderHit".to_string());
    }
    if call_lookup_bool(ctx, "lookup_is_miss", lookup)? {
        return Ok("ProviderMiss".to_string());
    }
    if call_lookup_bool(ctx, "lookup_is_refused_kind_mismatch", lookup)? {
        return Ok("ProviderRefusedKindMismatch".to_string());
    }
    if call_lookup_bool(ctx, "lookup_is_refused_wrong_artifact", lookup)? {
        return Ok("ProviderRefusedWrongArtifact".to_string());
    }
    if call_lookup_bool(ctx, "lookup_is_refused_wrong_content", lookup)? {
        return Ok("ProviderRefusedWrongContent".to_string());
    }
    let missing = lookup_missing_outputs(ctx, lookup)?;
    if !missing.is_empty() {
        return Ok(format!("ProviderRefusedIncomplete({missing:?})"));
    }
    Ok(format!("ProviderLookup({})", ctx.format_value(lookup)))
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
    if call_lookup_bool(ctx, "lookup_is_refused_kind_mismatch", &lookup)? {
        return Ok(ResolvedGraphProviderOutcome::RefusedKindMismatch);
    }
    if call_lookup_bool(ctx, "lookup_is_refused_wrong_artifact", &lookup)? {
        return Ok(ResolvedGraphProviderOutcome::RefusedWrongArtifact);
    }
    if call_lookup_bool(ctx, "lookup_is_refused_wrong_content", &lookup)? {
        return Ok(ResolvedGraphProviderOutcome::RefusedWrongContent);
    }
    let missing = lookup_missing_outputs(ctx, &lookup)?;
    if !missing.is_empty() {
        return Ok(ResolvedGraphProviderOutcome::RefusedIncomplete { missing });
    }
    Ok(ResolvedGraphProviderOutcome::LookupUnclassified {
        label: lookup_outcome_label(ctx, &lookup)?,
    })
}

fn serve_resolved_graph_v1_disk_probe_in_ctx(
    ctx: &InterpContext,
    closure_digest: &str,
    compiler_digest: &str,
    content_digest: &str,
    payload_byte_count: u64,
) -> Result<ResolvedGraphProviderOutcome, String> {
    let payload_bytes = byte_size_value_in_ctx(ctx, payload_byte_count)?;
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
        (Some("payload_bytes".to_string()), payload_bytes),
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
    let ctx = materialization_provider_ctx()?;
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

#[doc(hidden)]
pub fn materialization_provider_ctx_build_count_for_test() -> usize {
    MATERIALIZATION_PROVIDER_CTX_BUILD_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

#[doc(hidden)]
pub fn reset_materialization_provider_ctx_for_test() {
    MATERIALIZATION_PROVIDER_CTX.with(|slot| *slot.borrow_mut() = None);
    MATERIALIZATION_PROVIDER_CTX_BUILD_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}
