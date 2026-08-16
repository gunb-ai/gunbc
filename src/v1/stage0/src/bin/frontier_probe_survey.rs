#![allow(clippy::disallowed_macros)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use v1_compiler::cli_run::{
    discover_source_root_reads_for_entry, emit_source_ref_dag_for_path,
    emit_source_root_ingest_manifest, parse_source_root_entry_admission, resolve_entry_graph,
};
use v1_compiler::std_content_hash::{content_hash_atom, content_hash_combine_structural};
use v1_compiler::v1_interpreter::{
    self, free_monoid_symbol_value_to_dotted_string, ExecutionMode, InterpContext, Value,
};

const PROBE_ENTRY: &str = "src/v2/test/claim/long/compiler_frontier_probe_entry_test.dag";
const PROBE_RECEIPT_FN: &str = "frontier_probe_entry_receipt";
const WITNESS_LAYER_ROOTS: &[&str] = &["dag", "src/v2"];
const FRONTIER_SWEEP_ORDER_ENTRY: &str = "src/v2/compiler/self_host/frontier.dag";
const SWEEP_ORDER_DATA: &str = "compiler_frontier_sweep_order";
const HOST_REASON_SYMBOLS_ENTRY: &str = "src/v2/compiler/self_host/frontier_probe_types.dag";
const HOST_RUNTIME_ERROR_REASON_DATA: &str = "frontier_probe_survey_host_runtime_error_reason";
const HOST_OOM_OR_BUDGET_REASON_DATA: &str = "frontier_probe_survey_host_oom_or_budget_reason";

fn free_monoid_elems<'a>(value: &'a Value, ctx: &InterpContext) -> Result<Vec<&'a Value>, String> {
    let mut out = Vec::new();
    let mut cur = value;
    loop {
        match cur {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Cons") => {
                let head = ctx
                    .field(fields, "head")
                    .ok_or_else(|| "Cons without `head` field".to_string())?;
                out.push(head);
                cur = ctx
                    .field(fields, "tail")
                    .ok_or_else(|| "Cons without `tail` field".to_string())?;
            }
            Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Empty") => {
                return Ok(out);
            }
            Value::List(items) => {
                out.extend(items.iter());
                return Ok(out);
            }
            other => {
                return Err(format!(
                    "expected a List (Cons/Empty), got {}",
                    other.type_label_public()
                ));
            }
        }
    }
}

fn str_list_from_value(value: &Value, ctx: &InterpContext) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for elem in free_monoid_elems(value, ctx)? {
        match elem {
            Value::Str(s) => out.push(s.to_string()),
            other => {
                return Err(format!(
                    "expected a List<String> element, got {}",
                    other.type_label_public()
                ));
            }
        }
    }
    Ok(out)
}

fn load_compiler_frontier_sweep_order(source_roots: &[String]) -> Result<Vec<String>, String> {
    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    roots.extend(source_roots.iter().cloned());
    let (graph, source_indices) =
        resolve_entry_graph(&roots, FRONTIER_SWEEP_ORDER_ENTRY).map_err(|e| format!("{e}"))?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value = v1_interpreter::run_in_context(&ctx, SWEEP_ORDER_DATA, true)
        .map_err(|e| format!("read {SWEEP_ORDER_DATA}: {e}"))?;
    str_list_from_value(&value, &ctx)
}

struct ProbeReceiptRow {
    module_path: String,
    blocker_variant: String,
    located_stage: String,
    located_reason: String,
    detail: String,
    assemble_detail_manifest: String,
    rejection_chain: String,
    overlap_roster_detail: String,
    probe_error: Option<String>,
    subject: SurveySubjectPin,
}

fn dag_manifest_scalar_escape(s: &str) -> Result<String, String> {
    if s.contains('{') || s.contains('}') {
        return Err(format!(
            "manifest scalar escape: '{{' and '}}' forbidden in {:?}",
            s
        ));
    }
    Ok(s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn int_from_value(value: &Value) -> Result<i64, String> {
    match value {
        Value::Int(n) => Ok(*n),
        other => Err(format!("expected Int, got {}", other.type_label_public())),
    }
}

fn str_from_value(value: &Value) -> Result<String, String> {
    match value {
        Value::Str(s) => Ok(s.to_string()),
        other => Err(format!(
            "expected String, got {}",
            other.type_label_public()
        )),
    }
}

fn qualified_name_emit(value: &Value) -> Result<String, String> {
    let dotted = free_monoid_symbol_value_to_dotted_string(value);
    if dotted.is_empty() {
        return Err("empty QualifiedName".to_string());
    }
    let escaped = dag_manifest_scalar_escape(&dotted)?;
    Ok(format!(
        "qualified_name_from_dotted_string(dotted: \"{escaped}\")"
    ))
}

fn content_hash_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let digest = content_hash_digest_string(value, ctx)?;
    let escaped = dag_manifest_scalar_escape(&digest)?;
    Ok(format!(
        "Fnv1a64(Fnv1a64Structural {{ digest: \"{escaped}\" }})"
    ))
}

fn content_hash_digest_string(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let family = ctx.resolve(*variant_name);
            if family != "Fnv1a64" {
                return Err(format!(
                    "unsupported ContentHash family {family} in survey manifest"
                ));
            }
            if let Some(inner) = ctx.field(fields, "0") {
                return digest_from_structural(inner, ctx);
            }
            for (_, field_value) in fields.iter() {
                if let Ok(digest) = digest_from_structural(field_value, ctx) {
                    return Ok(digest);
                }
            }
            Err("Fnv1a64 variant missing structural payload".to_string())
        }
        Value::Record { type_name, .. } if ctx.resolve(*type_name) == "Fnv1a64Structural" => {
            digest_from_structural(value, ctx)
        }
        other => Err(format!(
            "expected ContentHash variant, got {}",
            other.type_label_public()
        )),
    }
}

fn digest_from_structural(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => {
            let digest = ctx
                .field(fields, "digest")
                .ok_or_else(|| "Fnv1a64Structural missing digest".to_string())?;
            str_from_value(digest)
        }
        other => Err(format!(
            "expected Fnv1a64Structural record, got {}",
            other.type_label_public()
        )),
    }
}

fn source_root_ref_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    variant_name(ctx, value)
}

fn source_ref_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let path = str_from_value(record_field(ctx, value, "path")?)?;
    let source_root = source_root_ref_emit(record_field(ctx, value, "source_root")?, ctx)?;
    let content_hash = content_hash_emit(record_field(ctx, value, "content_hash")?, ctx)?;
    let escaped_path = dag_manifest_scalar_escape(&path)?;
    Ok(format!(
        "SourceRef {{ path: \"{escaped_path}\", source_root: {source_root}, content_hash: {content_hash} }}"
    ))
}

fn source_root_coverage_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "SourceRootCoverageComplete" || name == "SourceRootManifestAbsent" {
                return Ok(name);
            }
            if name == "SourceRootManifestElided" {
                let read_count = int_from_value(
                    ctx.field(fields, "read_count")
                        .ok_or_else(|| format!("{name} missing read_count"))?,
                )?;
                let cap = int_from_value(
                    ctx.field(fields, "cap")
                        .ok_or_else(|| format!("{name} missing cap"))?,
                )?;
                return Ok(format!("{name} {{ read_count: {read_count}, cap: {cap} }}"));
            }
            if name == "SourceRootRowsMissing" {
                let read_count = int_from_value(
                    ctx.field(fields, "read_count")
                        .ok_or_else(|| format!("{name} missing read_count"))?,
                )?;
                let produced_row_count = int_from_value(
                    ctx.field(fields, "produced_row_count")
                        .ok_or_else(|| format!("{name} missing produced_row_count"))?,
                )?;
                return Ok(format!(
                    "{name} {{ read_count: {read_count}, produced_row_count: {produced_row_count} }}"
                ));
            }
            Err(format!("unknown SourceRootCoverage variant {name}"))
        }
        _ => Err(format!(
            "expected SourceRootCoverage variant, got {}",
            value.type_label_public()
        )),
    }
}

fn detail_variant_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            match name.as_str() {
                "FrontierProbeDetailAbsent" => Ok(name),
                "MissingModule" => {
                    let requested = ctx
                        .field(fields, "requested")
                        .ok_or_else(|| format!("{name} missing requested"))?;
                    Ok(format!(
                        "{name} {{ requested: {} }}",
                        qualified_name_emit(requested)?
                    ))
                }
                "SourceRootManifestElidedProbe" => {
                    let read_count = int_from_value(
                        ctx.field(fields, "read_count")
                            .ok_or_else(|| format!("{name} missing read_count"))?,
                    )?;
                    let cap = int_from_value(
                        ctx.field(fields, "cap")
                            .ok_or_else(|| format!("{name} missing cap"))?,
                    )?;
                    Ok(format!("{name} {{ read_count: {read_count}, cap: {cap} }}"))
                }
                "FrontierProbeClosurePopulationRefused" => {
                    let cause = ctx
                        .field(fields, "cause")
                        .ok_or_else(|| format!("{name} missing cause"))?;
                    Ok(format!("{name} {{ cause: {} }}", value_symbol_name(cause)?))
                }
                "FrontierProbeCoverageRefused" => {
                    let coverage = ctx
                        .field(fields, "coverage")
                        .ok_or_else(|| format!("{name} missing coverage"))?;
                    Ok(format!(
                        "{name} {{ coverage: {} }}",
                        source_root_coverage_emit(coverage, ctx)?
                    ))
                }
                "SourceRefReadRefused" => {
                    let source_ref = ctx
                        .field(fields, "source_ref")
                        .ok_or_else(|| format!("{name} missing source_ref"))?;
                    let error = str_from_value(
                        ctx.field(fields, "error")
                            .ok_or_else(|| format!("{name} missing error"))?,
                    )?;
                    let escaped_error = dag_manifest_scalar_escape(&error)?;
                    Ok(format!(
                        "{name} {{ source_ref: {}, error: \"{escaped_error}\" }}",
                        source_ref_emit(source_ref, ctx)?
                    ))
                }
                "SourceRefContentHashMismatch" => {
                    let source_ref = ctx
                        .field(fields, "source_ref")
                        .ok_or_else(|| format!("{name} missing source_ref"))?;
                    let expected = ctx
                        .field(fields, "expected")
                        .ok_or_else(|| format!("{name} missing expected"))?;
                    let observed = ctx
                        .field(fields, "observed")
                        .ok_or_else(|| format!("{name} missing observed"))?;
                    Ok(format!(
                        "{name} {{ source_ref: {}, expected: {}, observed: {} }}",
                        source_ref_emit(source_ref, ctx)?,
                        content_hash_emit(expected, ctx)?,
                        content_hash_emit(observed, ctx)?
                    ))
                }
                other => Err(format!("unsupported FrontierProbeDetail variant {other}")),
            }
        }
        _ => Err(format!(
            "expected FrontierProbeDetail variant, got {}",
            value.type_label_public()
        )),
    }
}

fn symbol_list_csv(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let mut parts = Vec::new();
    for item in free_monoid_elems(value, ctx)? {
        parts.push(value_symbol_name(item)?);
    }
    Ok(parts.join(","))
}

fn non_empty_symbol_chain_csv_flat(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let head = record_field(ctx, value, "head")?;
    let tail = record_field(ctx, value, "tail")?;
    let mut parts = vec![value_symbol_name(head)?];
    for item in free_monoid_elems(tail, ctx)? {
        parts.push(value_symbol_name(item)?);
    }
    Ok(parts.join(","))
}

fn symbol_list_dag_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let mut parts = Vec::new();
    for item in free_monoid_elems(value, ctx)? {
        parts.push(value_symbol_name(item)?);
    }
    let mut list = String::from("Empty");
    while let Some(sym) = parts.pop() {
        list = format!("Cons {{\n  head: {sym},\n  tail: {list}\n}}");
    }
    Ok(list)
}

fn grammar_choice_ambiguity_row_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let production = value_symbol_name(record_field(ctx, value, "production")?)?;
    let left_list = symbol_list_dag_emit(record_field(ctx, value, "left_first_terminals")?, ctx)?;
    let right_list = symbol_list_dag_emit(record_field(ctx, value, "right_first_terminals")?, ctx)?;
    let overlap_list = symbol_list_dag_emit(record_field(ctx, value, "overlap_terminals")?, ctx)?;
    let both_nullable = match record_field(ctx, value, "both_nullable")? {
        Value::Bool(b) => {
            if *b {
                "true"
            } else {
                "false"
            }
        }
        other => {
            return Err(format!(
                "both_nullable not Bool: {}",
                other.type_label_public()
            ));
        }
    };
    Ok(format!(
        "GrammarChoiceAmbiguityRow {{\n  production: {production},\n  left_first_terminals: {left_list},\n  right_first_terminals: {right_list},\n  overlap_terminals: {overlap_list},\n  both_nullable: {both_nullable}\n}}"
    ))
}

fn non_empty_grammar_rows_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let head = grammar_choice_ambiguity_row_emit(record_field(ctx, value, "head")?, ctx)?;
    let tail = record_field(ctx, value, "tail")?;
    let mut tail_nodes: Vec<String> = Vec::new();
    for item in free_monoid_elems(tail, ctx)? {
        tail_nodes.push(grammar_choice_ambiguity_row_emit(item, ctx)?);
    }
    let mut list = String::from("Empty");
    while let Some(node) = tail_nodes.pop() {
        list = format!("Cons {{\n  head: {node},\n  tail: {list}\n}}");
    }
    Ok(format!(
        "NonEmptyGrammarChoiceAmbiguityRows {{\n  head: {head},\n  tail: {list}\n}}"
    ))
}

fn non_empty_symbol_chain_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let head = value_symbol_name(record_field(ctx, value, "head")?)?;
    let tail = symbol_list_dag_emit(record_field(ctx, value, "tail")?, ctx)?;
    Ok(format!(
        "NonEmptySymbolChain {{\n  head: {head},\n  tail: {tail}\n}}"
    ))
}

// The cause location is emitted, not dropped, because a reason symbol alone does not say
// WHICH file refused. It sits on AssembleRejectionDetail beside the cause rather than
// inside it, because a position is not part of what the refusal IS. It is a probe-owned
// coproduct rather than a bare Locus precisely so this emitter can be TOTAL: a position
// with no serializable form becomes the closed CauseLocationNodeNotSerializable arm
// instead of aborting a 27-module survey, and no arm invents a position it did not observe.
fn cause_location_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "CauseLocationTextual" {
                let file = ctx
                    .field(fields, "file")
                    .ok_or_else(|| format!("{name} missing file"))?;
                let extent = ctx
                    .field(fields, "extent")
                    .ok_or_else(|| format!("{name} missing extent"))?;
                Ok(format!(
                    "CauseLocationTextual {{ file: \"{}\", extent: {} }}",
                    dag_manifest_scalar_escape(&str_from_value(file)?)?,
                    extent_emit(extent, ctx)?
                ))
            } else if name == "CauseLocationPort" {
                let port = ctx
                    .field(fields, "port")
                    .ok_or_else(|| format!("{name} missing port"))?;
                Ok(format!(
                    "CauseLocationPort {{ port: {} }}",
                    value_symbol_name(port)?
                ))
            } else if name == "CauseLocationNodeNotSerializable" {
                Ok("CauseLocationNodeNotSerializable".to_string())
            } else {
                Err(format!("unknown FrontierProbeCauseLocation variant {name}"))
            }
        }
        _ => Err("cause location not a variant".to_string()),
    }
}

fn extent_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "WholeFile" {
                Ok("WholeFile".to_string())
            } else if name == "ByteRange" {
                let start = ctx
                    .field(fields, "start")
                    .ok_or_else(|| format!("{name} missing start"))?;
                let end = ctx
                    .field(fields, "end")
                    .ok_or_else(|| format!("{name} missing end"))?;
                Ok(format!(
                    "ByteRange {{ start: {}, end: {} }}",
                    int_from_value(start)?,
                    int_from_value(end)?
                ))
            } else {
                Err(format!("unknown Extent variant {name}"))
            }
        }
        _ => Err("extent not a variant".to_string()),
    }
}

fn assemble_rejection_cause_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "GrammarChoiceOverlap" {
                let rows = ctx
                    .field(fields, "rows")
                    .ok_or_else(|| format!("{name} missing rows"))?;
                Ok(format!(
                    "GrammarChoiceOverlap {{ rows: {} }}",
                    non_empty_grammar_rows_emit(rows, ctx)?
                ))
            } else if name == "MaskedAssembleRefusal" {
                let overlap_evidence = ctx
                    .field(fields, "overlap_evidence")
                    .ok_or_else(|| format!("{name} missing overlap_evidence"))?;
                let remaining_chain = ctx
                    .field(fields, "remaining_chain")
                    .ok_or_else(|| format!("{name} missing remaining_chain"))?;
                Ok(format!(
                    "MaskedAssembleRefusal {{\n  overlap_evidence: {},\n  remaining_chain: {}\n}}",
                    non_empty_grammar_rows_emit(overlap_evidence, ctx)?,
                    non_empty_symbol_chain_emit(remaining_chain, ctx)?
                ))
            } else if name == "MaskedAssembleRefusalWithoutEvidence" {
                let remaining_chain = ctx
                    .field(fields, "remaining_chain")
                    .ok_or_else(|| format!("{name} missing remaining_chain"))?;
                Ok(format!(
                    "MaskedAssembleRefusalWithoutEvidence {{\n  remaining_chain: {}\n}}",
                    non_empty_symbol_chain_emit(remaining_chain, ctx)?
                ))
            } else if name == "AssembleMissingModule" {
                let requested = ctx
                    .field(fields, "requested")
                    .ok_or_else(|| format!("{name} missing requested"))?;
                Ok(format!(
                    "AssembleMissingModule {{ requested: {} }}",
                    qualified_name_emit(requested)?
                ))
            } else if name == "MissingModuleIdentityUnavailable" {
                Ok("MissingModuleIdentityUnavailable".to_string())
            } else if name == "OtherAssembleRefusal" {
                let reason = ctx
                    .field(fields, "reason")
                    .ok_or_else(|| format!("{name} missing reason"))?;
                Ok(format!(
                    "OtherAssembleRefusal {{ reason: {} }}",
                    value_symbol_name(reason)?
                ))
            } else {
                Err(format!("unknown AssembleRejectionCause variant {name}"))
            }
        }
        _ => Err("assemble cause not a variant".to_string()),
    }
}

fn assemble_detail_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "NoFrontierProbeDetail" {
                Ok("NoFrontierProbeDetail".to_string())
            } else if name == "AssembleRejectionDetail" {
                let chain = ctx
                    .field(fields, "rejection_chain")
                    .ok_or_else(|| format!("{name} missing rejection_chain"))?;
                let cause = ctx
                    .field(fields, "cause")
                    .ok_or_else(|| format!("{name} missing cause"))?;
                let at = ctx
                    .field(fields, "at")
                    .ok_or_else(|| format!("{name} missing at"))?;
                Ok(format!(
                    "AssembleRejectionDetail {{\n  rejection_chain: {},\n  cause: {},\n  at: {}\n}}",
                    non_empty_symbol_chain_emit(chain, ctx)?,
                    assemble_rejection_cause_emit(cause, ctx)?,
                    cause_location_emit(at, ctx)?
                ))
            } else {
                Err(format!("unknown assemble_detail variant {name}"))
            }
        }
        _ => Err("assemble_detail not a variant".to_string()),
    }
}

fn overlap_rows_detail_from_rows_value(
    value: &Value,
    ctx: &InterpContext,
) -> Result<String, String> {
    let mut rows = Vec::new();
    let head = record_field(ctx, value, "head")?;
    rows.push(head);
    for item in free_monoid_elems(record_field(ctx, value, "tail")?, ctx)? {
        rows.push(item);
    }
    let mut out = Vec::new();
    for row in rows {
        let production = value_symbol_name(record_field(ctx, row, "production")?)?;
        let left = symbol_list_csv(record_field(ctx, row, "left_first_terminals")?, ctx)?;
        let right = symbol_list_csv(record_field(ctx, row, "right_first_terminals")?, ctx)?;
        let overlap = symbol_list_csv(record_field(ctx, row, "overlap_terminals")?, ctx)?;
        out.push(format!(
            "{production}|left={left}|right={right}|overlap={overlap}"
        ));
    }
    Ok(out.join(";"))
}

fn overlap_roster_detail_from_assemble_detail(
    value: &Value,
    ctx: &InterpContext,
) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "AssembleRejectionDetail") => {
            let cause = ctx
                .field(fields, "cause")
                .ok_or_else(|| "AssembleRejectionDetail missing cause".to_string())?;
            match cause {
                Value::Variant {
                    variant_name: cause_name,
                    fields: cause_fields,
                    ..
                } if ctx.sym_eq(*cause_name, "GrammarChoiceOverlap") => {
                    let rows = ctx
                        .field(cause_fields, "rows")
                        .ok_or_else(|| "GrammarChoiceOverlap missing rows".to_string())?;
                    overlap_rows_detail_from_rows_value(rows, ctx)
                }
                Value::Variant {
                    variant_name: cause_name,
                    fields: cause_fields,
                    ..
                } if ctx.sym_eq(*cause_name, "MaskedAssembleRefusal") => {
                    let overlap_evidence =
                        ctx.field(cause_fields, "overlap_evidence").ok_or_else(|| {
                            "MaskedAssembleRefusal missing overlap_evidence".to_string()
                        })?;
                    overlap_rows_detail_from_rows_value(overlap_evidence, ctx)
                }
                _ => Ok(String::new()),
            }
        }
        _ => Ok(String::new()),
    }
}

fn rejection_chain_from_assemble_detail(
    value: &Value,
    ctx: &InterpContext,
) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "AssembleRejectionDetail") => {
            let chain = ctx
                .field(fields, "rejection_chain")
                .ok_or_else(|| "AssembleRejectionDetail missing rejection_chain".to_string())?;
            non_empty_symbol_chain_csv_flat(chain, ctx)
        }
        _ => Ok(String::new()),
    }
}

fn survey_manifest_needs_detail_imports(rows: &[ProbeReceiptRow]) -> bool {
    rows.iter().any(|row| {
        row.detail != "FrontierProbeDetailAbsent"
            || row.assemble_detail_manifest != "NoFrontierProbeDetail"
    })
}

fn survey_manifest_import_block(rows: &[ProbeReceiptRow]) -> String {
    let mut imports = String::from(
        "import extdeps.git.object_store { GitSha1ObjectId }\n\
         import std.content_hash { Fnv1a64Structural, Sha1Digest, Sha256Digest, as_content_hash_structural }\n\
         import v2.compiler.self_host.frontier_probe_types {\n\
           FrontierProbeReceipt,\n\
           FrontierSurveyManifest,\n\
           FrontierSurveySubject\n\
         }\n\
         import v2.std.algebra { Cons, Empty }\n\
         import v2.std.collection { List }\n",
    );
    if survey_manifest_needs_detail_imports(rows) {
        imports.push_str(
            "import v2.compiler.source_authority { SourceRef, SourceRootCoverage }\n\
             import std.content_hash { ContentHash, Fnv1a64 }\n\
             import v2.std.cross_tree.import_model { SourceRootRef, V2Tree, DagTree }\n\
             import v2.std.grammar_choice_ambiguity { GrammarChoiceAmbiguityRow, NonEmptyGrammarChoiceAmbiguityRows }\n\
             import v2.std.qualified_name { qualified_name_from_dotted_string }\n",
        );
    }
    imports.push('\n');
    imports
}

struct HostFailureReasons {
    runtime_error: String,
    oom_or_budget: String,
}

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("frontier_probe_survey: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn symbol_to_manifest(sym: &str) -> String {
    if sym.starts_with('^') {
        sym.to_string()
    } else {
        format!("^{sym}")
    }
}

fn value_symbol_name(value: &Value) -> Result<String, String> {
    match value {
        Value::Str(s) => Ok(symbol_to_manifest(s.as_ref())),
        other => Err(format!(
            "expected symbol String, got {}",
            other.type_label_public()
        )),
    }
}

fn load_symbol_data_from_entry(
    source_roots: &[String],
    entry: &str,
    data_name: &str,
) -> Result<String, String> {
    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    roots.extend(source_roots.iter().cloned());
    let (graph, source_indices) = resolve_entry_graph(&roots, entry).map_err(|e| format!("{e}"))?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value = v1_interpreter::run_in_context(&ctx, data_name, true)
        .map_err(|e| format!("read {data_name}: {e}"))?;
    let sym = value_symbol_name(&value)?;
    Ok(sym.trim_start_matches('^').to_string())
}

fn load_host_failure_reasons(source_roots: &[String]) -> Result<HostFailureReasons, String> {
    Ok(HostFailureReasons {
        runtime_error: load_symbol_data_from_entry(
            source_roots,
            HOST_REASON_SYMBOLS_ENTRY,
            HOST_RUNTIME_ERROR_REASON_DATA,
        )?,
        oom_or_budget: load_symbol_data_from_entry(
            source_roots,
            HOST_REASON_SYMBOLS_ENTRY,
            HOST_OOM_OR_BUDGET_REASON_DATA,
        )?,
    })
}

fn variant_name(ctx: &InterpContext, value: &Value) -> Result<String, String> {
    match value {
        Value::Variant { variant_name, .. } => Ok(ctx.resolve(*variant_name)),
        _ => Err(format!(
            "expected variant, got {}",
            value.type_label_public()
        )),
    }
}

fn record_field<'a>(
    ctx: &InterpContext,
    value: &'a Value,
    name: &str,
) -> Result<&'a Value, String> {
    match value {
        Value::Record { fields, .. } => ctx
            .field(fields, name)
            .ok_or_else(|| format!("record missing field {name}")),
        _ => Err(format!(
            "expected record, got {}",
            value.type_label_public()
        )),
    }
}

fn blocker_variant_emit(blocker: &Value, ctx: &InterpContext) -> Result<String, String> {
    match blocker {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "UnknownProbeCause" {
                let reason = ctx
                    .field(fields, "reason")
                    .or_else(|| ctx.field(fields, "cause"))
                    .ok_or_else(|| format!("{name} missing reason/cause"))?;
                let reason_sym = value_symbol_name(reason)?;
                Ok(format!("{name} {{ reason: {reason_sym} }}"))
            } else if name == "UpstreamSemanticRefusal" {
                // HAND-RUST GATE (+12 LOC): mechanical serialize arm for new coproduct variant —
                // see v2.compiler.self_host.frontier_probe_survey::frontier_probe_survey_blocker_variant_emit_rust_gate_note
                let diagnostic_class = ctx
                    .field(fields, "diagnostic_class")
                    .ok_or_else(|| format!("{name} missing diagnostic_class"))?;
                let diagnostic_name = ctx
                    .field(fields, "diagnostic_name")
                    .ok_or_else(|| format!("{name} missing diagnostic_name"))?;
                let class_sym = value_symbol_name(diagnostic_class)?;
                let name_sym = value_symbol_name(diagnostic_name)?;
                Ok(format!(
                    "{name} {{ diagnostic_class: {class_sym}, diagnostic_name: {name_sym} }}"
                ))
            } else {
                Ok(name)
            }
        }
        _ => Err("blocker_class not a variant".to_string()),
    }
}

fn extract_probe_receipt(value: &Value, ctx: &InterpContext) -> Result<ProbeReceiptRow, String> {
    let module_path = match record_field(ctx, value, "module_path")? {
        Value::Str(s) => s.to_string(),
        other => {
            return Err(format!(
                "module_path not String: {}",
                other.type_label_public()
            ));
        }
    };
    let blocker = record_field(ctx, value, "blocker_class")?;
    let stage = record_field(ctx, value, "located_stage")?;
    let reason = record_field(ctx, value, "located_reason")?;
    let detail = record_field(ctx, value, "detail")?;
    let assemble_detail = record_field(ctx, value, "assemble_detail")?;
    let assemble_detail_manifest = assemble_detail_emit(assemble_detail, ctx)?;
    let rejection_chain = rejection_chain_from_assemble_detail(assemble_detail, ctx)?;
    let overlap_roster_detail = overlap_roster_detail_from_assemble_detail(assemble_detail, ctx)?;
    Ok(ProbeReceiptRow {
        module_path,
        blocker_variant: blocker_variant_emit(blocker, ctx)?,
        located_stage: variant_name(ctx, stage)?,
        located_reason: value_symbol_name(reason)?,
        detail: detail_variant_emit(detail, ctx)?,
        assemble_detail_manifest,
        rejection_chain,
        overlap_roster_detail,
        probe_error: None,
        subject: SurveySubjectPin {
            source_commit: String::new(),
            source_tree: String::new(),
            survey_executable: String::new(),
            source_roots_digest: String::new(),
            probe_policy_revision: String::new(),
        },
    })
}

fn unknown_probe_row(
    module_path: &str,
    cause: &str,
    subject: &SurveySubjectPin,
) -> ProbeReceiptRow {
    ProbeReceiptRow {
        module_path: module_path.replace('\\', "/"),
        blocker_variant: format!("UnknownProbeCause {{ reason: ^{cause} }}"),
        located_stage: "ProbeStageAssemble".to_string(),
        located_reason: format!("^{cause}"),
        detail: "FrontierProbeDetailAbsent".to_string(),
        assemble_detail_manifest: "NoFrontierProbeDetail".to_string(),
        rejection_chain: String::new(),
        overlap_roster_detail: String::new(),
        probe_error: Some(cause.to_string()),
        subject: subject.clone(),
    }
}

fn write_probe_overlay_manifest(
    module_path: &str,
    ingest_manifest: &Path,
    entry_source_ref: &str,
) -> Result<(), String> {
    let escaped = module_path.replace('\\', "/");
    let append = format!(
        "\ndata frontier_probe_entry_module_path: String = \"{escaped}\"\n\
         data frontier_probe_entry_source_ref: SourceRef = {entry_source_ref}\n"
    );
    let mut body =
        fs::read_to_string(ingest_manifest).map_err(|e| format!("read ingest manifest: {e}"))?;
    if body.contains("frontier_probe_entry_module_path") {
        return Ok(());
    }
    body.push_str(&append);
    fs::write(ingest_manifest, body).map_err(|e| format!("append module_path overlay: {e}"))
}

fn per_module_overlay_dir(survey_dir: &Path, module_path: &str) -> PathBuf {
    let slug = module_path
        .replace('\\', "/")
        .trim_start_matches("src/v2/")
        .replace('/', "__");
    survey_dir.join(slug)
}

fn run_probe_for_module(
    source_roots: &[String],
    survey_dir: &Path,
    module_path: &str,
) -> Result<ProbeReceiptRow, String> {
    let exclude = vec!["host_source_root_ingest_manifest.dag".to_string()];
    let overlay_dir = per_module_overlay_dir(survey_dir, module_path);
    fs::create_dir_all(&overlay_dir)
        .map_err(|e| format!("mkdir overlay {:?}: {e}", overlay_dir))?;

    let mut discover_roots: Vec<String> =
        WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    for root in source_roots {
        if !discover_roots.iter().any(|existing| existing == root) {
            discover_roots.push(root.clone());
        }
    }
    let records = discover_source_root_reads_for_entry(&discover_roots, module_path, &exclude)?;
    let entry_source =
        fs::read_to_string(module_path).map_err(|e| format!("read entry {module_path}: {e}"))?;
    let admission = parse_source_root_entry_admission(&entry_source)?;
    let ingest_manifest = overlay_dir.join("host_source_root_ingest_manifest.dag");
    emit_source_root_ingest_manifest(&ingest_manifest, &records, Some(&admission))?;
    let entry_source_ref = emit_source_ref_dag_for_path(&records, module_path)?;
    write_probe_overlay_manifest(module_path, &ingest_manifest, &entry_source_ref)?;

    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    roots.push(overlay_dir.to_string_lossy().into_owned());

    let (graph, source_indices) = resolve_entry_graph(&roots, PROBE_ENTRY)?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value =
        v1_interpreter::run_in_context(&ctx, PROBE_RECEIPT_FN, true).map_err(|e| format!("{e}"))?;
    extract_probe_receipt(&value, &ctx)
}

fn emit_receipt_row(row: &ProbeReceiptRow) -> String {
    format!(
        "FrontierProbeReceipt {{\n  module_path: \"{}\",\n  blocker_class: {},\n  located_stage: {},\n  located_reason: {},\n  detail: {},\n  assemble_detail: {}\n}}",
        row.module_path.replace('\\', "/"),
        row.blocker_variant,
        row.located_stage,
        row.located_reason,
        row.detail,
        row.assemble_detail_manifest,
    )
}

fn emit_survey_manifest(
    path: &Path,
    rows: &[ProbeReceiptRow],
    subject: &SurveySubjectPin,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {:?}: {e}", parent))?;
    }
    let mut receipt_nodes: Vec<String> = rows.iter().map(emit_receipt_row).collect();
    let mut list = String::from("Empty");
    while let Some(head) = receipt_nodes.pop() {
        list = format!("Cons {{\n  head: {head},\n  tail: {list}\n}}");
    }
    let imports = survey_manifest_import_block(rows);
    let body = format!(
        "// GENERATED by frontier_probe_survey — execution receipt table. Regenerate via frontier_probe_survey bin.\n\
         module v2.test.workflow.host_frontier_probe_survey_manifest\n\n\
         {imports}\
         data host_frontier_probe_survey_manifest: FrontierSurveyManifest = FrontierSurveyManifest {{\n\
           subject: FrontierSurveySubject {{\n\
             source_commit: \"{source_commit}\",\n\
             source_tree: GitSha1ObjectId {{\n\
               digest: Sha1Digest {{ hex: \"{source_tree}\" }}\n\
             }},\n\
             survey_executable: Sha256Digest {{ hex: \"{survey_executable}\" }},\n\
             source_roots_digest: Fnv1a64Structural {{ digest: \"{source_roots_digest}\" }},\n\
             probe_policy_revision: as_content_hash_structural(structural: Fnv1a64Structural {{ digest: \"{probe_policy_revision}\" }})\n\
           }},\n\
           receipts: {list}\n\
         }}\n",
        source_commit = subject.source_commit,
        source_tree = subject.source_tree,
        survey_executable = subject.survey_executable,
        source_roots_digest = subject.source_roots_digest,
        probe_policy_revision = subject.probe_policy_revision,
    );
    fs::write(path, body).map_err(|e| format!("write survey manifest: {e}"))
}

fn emit_tsv(path: &Path, rows: &[ProbeReceiptRow]) -> Result<(), String> {
    let mut out = fs::File::create(path).map_err(|e| format!("create tsv: {e}"))?;
    writeln!(
        out,
        "module\tsource_commit\tsource_tree\tsurvey_executable\tsource_roots_digest\tprobe_policy_revision\tself_emit_ready\tblocker_class\tlocated_stage\tlocated_reason\trejection_chain\toverlap_roster_detail\tprobe_error"
    )
    .map_err(|e| format!("write tsv header: {e}"))?;
    for row in rows {
        let self_emit = if row.blocker_variant == "SelfEmitReady" {
            "yes"
        } else {
            "no"
        };
        writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.module_path,
            row.subject.source_commit,
            row.subject.source_tree,
            row.subject.survey_executable,
            row.subject.source_roots_digest,
            row.subject.probe_policy_revision,
            self_emit,
            row.blocker_variant,
            row.located_stage,
            row.located_reason,
            row.rejection_chain,
            row.overlap_roster_detail,
            row.probe_error.as_deref().unwrap_or("")
        )
        .map_err(|e| format!("write tsv row: {e}"))?;
    }
    Ok(())
}

// Diagnostic stderr histogram only — NOT the Wave-2 sizing verdict authority.
// Canonical clustering: `frontier_gap_clustering_from_receipts` in
// `v2.compiler.self_host.frontier_probe_survey` (witness:
// `compiler_frontier_gap_clustering_common_root_at_blocker_class_holds`).
// Deletes with the seed `frontier_probe_survey` bin (P5 scaffold).
fn cluster_summary(rows: &[ProbeReceiptRow]) -> String {
    use std::collections::BTreeMap;
    let mut by_reason: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_blocker: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        *by_reason.entry(row.located_reason.clone()).or_insert(0) += 1;
        *by_blocker.entry(row.blocker_variant.clone()).or_insert(0) += 1;
    }
    let mut lines = vec![
        "=== frontier_probe_survey cluster summary ===".to_string(),
        format!("modules: {}", rows.len()),
        "--- by blocker_class ---".to_string(),
    ];
    for (k, v) in &by_blocker {
        lines.push(format!("  {v}\t{k}"));
    }
    lines.push("--- by located_reason (root clustering) ---".to_string());
    for (k, v) in &by_reason {
        lines.push(format!("  {v}\t{k}"));
    }
    lines.push(format!("distinct located_reasons: {}", by_reason.len()));
    lines.push(
        "verdict: (see frontier_gap_clustering_from_receipts on emitted manifest)".to_string(),
    );
    lines.join("\n")
}

struct SurveySubjectPin {
    source_commit: String,
    source_tree: String,
    survey_executable: String,
    source_roots_digest: String,
    probe_policy_revision: String,
}

impl Clone for SurveySubjectPin {
    fn clone(&self) -> Self {
        Self {
            source_commit: self.source_commit.clone(),
            source_tree: self.source_tree.clone(),
            survey_executable: self.survey_executable.clone(),
            source_roots_digest: self.source_roots_digest.clone(),
            probe_policy_revision: self.probe_policy_revision.clone(),
        }
    }
}

fn subjects_equal(left: &SurveySubjectPin, right: &SurveySubjectPin) -> bool {
    left.source_commit == right.source_commit
        && left.source_tree == right.source_tree
        && left.survey_executable == right.survey_executable
        && left.source_roots_digest == right.source_roots_digest
        && left.probe_policy_revision == right.probe_policy_revision
}

fn attach_subject(row: ProbeReceiptRow, subject: &SurveySubjectPin) -> ProbeReceiptRow {
    ProbeReceiptRow {
        subject: subject.clone(),
        ..row
    }
}

fn validate_lowercase_hex(label: &str, value: &str, expected_len: usize) -> Result<(), String> {
    if value.len() != expected_len {
        return Err(format!(
            "SurveyPinRefused: {label} length {} != {}",
            value.len(),
            expected_len
        ));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    {
        return Err(format!("SurveyPinRefused: {label} is not lowercase hex"));
    }
    Ok(())
}

fn git_output(args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("SurveyPinRefused: git {args:?} spawn failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "SurveyPinRefused: git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn ensure_clean_tree() -> Result<(), String> {
    let porcelain = git_output(&["status", "--porcelain"])?;
    if !porcelain.is_empty() {
        return Err(
            "SurveyPinRefused: dirty working tree — run from a clean detached worktree at the selected commit"
                .to_string(),
        );
    }
    Ok(())
}

fn frontier_probe_source_roots_digest_rust(source_roots: &[String]) -> String {
    // SCAFFOLD — duplicates v2.compiler.self_host.frontier_probe_types
    // frontier_probe_source_roots_digest (same structural fold). Dissolve-on: frontier_probe_survey
    // self-emits from the .dag authority once emit crosses this module.
    source_roots
        .iter()
        .fold(
            content_hash_atom("frontier-probe-source-roots".to_string()),
            |acc, root| content_hash_combine_structural(acc, content_hash_atom(root.clone())),
        )
        .digest
        .clone()
}

fn frontier_probe_policy_revision_digest_rust() -> String {
    // SCAFFOLD — duplicates frontier_probe_policy_revision() in frontier_probe_types.dag.
    // Dissolve-on: same emit-crosses-module trigger as frontier_probe_source_roots_digest_rust.
    content_hash_atom("frontier-probe-survey-policy-v1".to_string())
        .digest
        .clone()
}

fn verify_build_provenance(subject: &SurveySubjectPin) -> Result<(), String> {
    const BUILD_DIRTY: Option<&str> = option_env!("FRONTIER_PROBE_SURVEY_BUILD_DIRTY");
    if BUILD_DIRTY == Some("1") {
        return Err(
            "SurveyPinRefused: binary built from dirty worktree — rebuild from clean sources"
                .to_string(),
        );
    }
    const BUILD_COMMIT: Option<&str> = option_env!("FRONTIER_PROBE_SURVEY_BUILD_COMMIT");
    const BUILD_TREE: Option<&str> = option_env!("FRONTIER_PROBE_SURVEY_BUILD_TREE");
    match (BUILD_COMMIT, BUILD_TREE) {
        (Some(build_commit), Some(build_tree))
            if !build_commit.is_empty() && !build_tree.is_empty() =>
        {
            if subject.source_commit != build_commit {
                return Err(format!(
                    "SurveyPinRefused: binary built at commit {build_commit} but survey subject is {}",
                    subject.source_commit
                ));
            }
            if subject.source_tree != build_tree {
                return Err(format!(
                    "SurveyPinRefused: binary built at tree {build_tree} but survey subject tree is {}",
                    subject.source_tree
                ));
            }
            Ok(())
        }
        _ => Err(
            "SurveyPinRefused: frontier_probe_survey missing embedded build provenance — rebuild with cargo build"
                .to_string(),
        ),
    }
}

fn resolve_survey_subject(source_roots: &[String]) -> Result<SurveySubjectPin, String> {
    ensure_clean_tree()?;
    let source_commit = git_output(&["rev-parse", "HEAD"])?;
    validate_lowercase_hex("source_commit", &source_commit, 40)?;
    let source_tree = git_output(&["rev-parse", "HEAD^{tree}"])?;
    validate_lowercase_hex("source_tree", &source_tree, 40)?;
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("SurveyPinRefused: current_exe unavailable: {e}"))?;
    let bytes = fs::read(&exe_path).map_err(|e| {
        format!(
            "SurveyPinRefused: read survey executable {:?}: {e}",
            exe_path
        )
    })?;
    use sha2::{Digest, Sha256};
    let survey_executable = format!("{:x}", Sha256::digest(bytes));
    validate_lowercase_hex("survey_executable", &survey_executable, 64)?;
    let source_roots_digest = frontier_probe_source_roots_digest_rust(source_roots);
    validate_lowercase_hex("source_roots_digest", &source_roots_digest, 16)?;
    let probe_policy_revision = frontier_probe_policy_revision_digest_rust();
    validate_lowercase_hex("probe_policy_revision", &probe_policy_revision, 16)?;
    Ok(SurveySubjectPin {
        source_commit,
        source_tree,
        survey_executable,
        source_roots_digest,
        probe_policy_revision,
    })
}

fn resolve_and_verify_survey_subject(source_roots: &[String]) -> Result<SurveySubjectPin, String> {
    let subject = resolve_survey_subject(source_roots)?;
    verify_build_provenance(&subject)?;
    Ok(subject)
}

fn load_probe_rows_from_tsv(path: &Path) -> Result<Vec<ProbeReceiptRow>, String> {
    let body = fs::read_to_string(path).map_err(|e| format!("read tsv {:?}: {e}", path))?;
    let mut rows = Vec::new();
    for (lineno, line) in body.lines().enumerate() {
        if lineno == 0 {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 13 {
            return Err(format!(
                "malformed tsv line {}: expected 13 subject/receipt columns, got {line}",
                lineno + 1
            ));
        }
        let subject = SurveySubjectPin {
            source_commit: parts[1].to_string(),
            source_tree: parts[2].to_string(),
            survey_executable: parts[3].to_string(),
            source_roots_digest: parts[4].to_string(),
            probe_policy_revision: parts[5].to_string(),
        };
        rows.push(ProbeReceiptRow {
            module_path: parts[0].to_string(),
            subject,
            blocker_variant: parts[7].to_string(),
            located_stage: parts[8].to_string(),
            located_reason: parts[9].to_string(),
            rejection_chain: parts[10].to_string(),
            overlap_roster_detail: parts[11].to_string(),
            detail: "FrontierProbeDetailAbsent".to_string(),
            assemble_detail_manifest: "NoFrontierProbeDetail".to_string(),
            probe_error: if parts[12].is_empty() {
                None
            } else {
                Some(parts[12].to_string())
            },
        });
    }
    if rows.is_empty() {
        return Err("receipt tsv has zero data rows".to_string());
    }
    let first = &rows[0].subject;
    for (idx, row) in rows.iter().enumerate().skip(1) {
        if !subjects_equal(first, &row.subject) {
            return Err(format!(
                "SurveySubjectMismatch: row {} subject {} != row 1 subject {}",
                idx + 1,
                row.subject.source_commit,
                first.source_commit
            ));
        }
    }
    Ok(rows)
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut survey_manifest: Option<PathBuf> = None;
    let mut survey_tsv: Option<PathBuf> = None;
    let mut survey_dir: Option<PathBuf> = None;
    let mut only_modules: Vec<String> = Vec::new();
    let mut used_module_filter = false;
    let mut emit_manifest_only = false;
    let mut receipt_tsv: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--module" => {
                i += 1;
                only_modules.push(require_value(&args, i, "--module")?);
                used_module_filter = true;
            }
            "--emit-survey-manifest" => {
                i += 1;
                survey_manifest = Some(PathBuf::from(require_value(
                    &args,
                    i,
                    "--emit-survey-manifest",
                )?));
            }
            "--emit-tsv" => {
                i += 1;
                survey_tsv = Some(PathBuf::from(require_value(&args, i, "--emit-tsv")?));
            }
            "--survey-dir" => {
                i += 1;
                survey_dir = Some(PathBuf::from(require_value(&args, i, "--survey-dir")?));
            }
            "--emit-manifest-only" => {
                emit_manifest_only = true;
            }
            "--receipt-tsv" => {
                i += 1;
                receipt_tsv = Some(PathBuf::from(require_value(&args, i, "--receipt-tsv")?));
            }
            other => {
                eprintln!("frontier_probe_survey: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("frontier_probe_survey: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    let subject = resolve_and_verify_survey_subject(&source_roots).map_err(|msg| {
        eprintln!("frontier_probe_survey: {msg}");
        ExitCode::from(1)
    })?;
    let survey_manifest = survey_manifest.ok_or_else(|| {
        eprintln!("frontier_probe_survey: --emit-survey-manifest required");
        ExitCode::from(2)
    })?;

    if emit_manifest_only {
        let tsv_path = receipt_tsv.ok_or_else(|| {
            eprintln!("frontier_probe_survey: --emit-manifest-only requires --receipt-tsv");
            ExitCode::from(2)
        })?;
        let rows = load_probe_rows_from_tsv(&tsv_path).map_err(|msg| {
            eprintln!("frontier_probe_survey: {msg}");
            ExitCode::from(1)
        })?;
        if !subjects_equal(&rows[0].subject, &subject) {
            eprintln!(
                "frontier_probe_survey: SurveySubjectMismatch: receipt tsv subject {} != current subject {}",
                rows[0].subject.source_commit, subject.source_commit
            );
            return Err(ExitCode::from(1));
        }
        emit_survey_manifest(&survey_manifest, &rows, &rows[0].subject).map_err(|msg| {
            eprintln!("frontier_probe_survey: {msg}");
            ExitCode::from(1)
        })?;
        eprintln!(
            "frontier_probe_survey: wrote {} ({} receipt(s), manifest-only)",
            survey_manifest.display(),
            rows.len()
        );
        return Ok(ExitCode::from(0));
    }

    let survey_dir = survey_dir.unwrap_or_else(|| {
        survey_manifest
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("target/frontier-probe-survey"))
    });
    fs::create_dir_all(&survey_dir).map_err(|e| {
        eprintln!("frontier_probe_survey: mkdir {:?}: {e}", survey_dir);
        ExitCode::from(1)
    })?;

    let mut module_paths = load_compiler_frontier_sweep_order(&source_roots).map_err(|msg| {
        eprintln!("frontier_probe_survey: {msg}");
        ExitCode::from(1)
    })?;
    if !only_modules.is_empty() {
        module_paths = only_modules;
    }
    if module_paths.is_empty() {
        eprintln!("frontier_probe_survey: no module paths to probe");
        return Err(ExitCode::from(1));
    }
    if used_module_filter {
        eprintln!(
            "frontier_probe_survey: probing {} module path(s) (--module filter)",
            module_paths.len()
        );
    } else {
        eprintln!(
            "frontier_probe_survey: probing {} module path(s) from {SWEEP_ORDER_DATA}",
            module_paths.len()
        );
    }

    let host_failure_reasons = load_host_failure_reasons(&source_roots).map_err(|msg| {
        eprintln!("frontier_probe_survey: {msg}");
        ExitCode::from(1)
    })?;

    let mut rows: Vec<ProbeReceiptRow> = Vec::new();
    for module_path in &module_paths {
        eprintln!("frontier_probe_survey: probing {module_path}");
        match run_probe_for_module(&source_roots, &survey_dir, module_path) {
            Ok(row) => {
                eprintln!(
                    "  blocker={} stage={} reason={} chain={} overlap={}",
                    row.blocker_variant,
                    row.located_stage,
                    row.located_reason,
                    row.rejection_chain,
                    row.overlap_roster_detail
                );
                rows.push(attach_subject(row, &subject));
            }
            Err(msg) => {
                let cause = if msg.contains("EvalBudgetExceeded") || msg.contains("OOM") {
                    host_failure_reasons.oom_or_budget.as_str()
                } else {
                    host_failure_reasons.runtime_error.as_str()
                };
                eprintln!("frontier_probe_survey: {module_path}: {msg} -> Unknown({cause})");
                rows.push(unknown_probe_row(module_path, cause, &subject));
            }
        }
    }

    emit_survey_manifest(&survey_manifest, &rows, &subject).map_err(|msg| {
        eprintln!("frontier_probe_survey: {msg}");
        ExitCode::from(1)
    })?;
    if let Some(tsv_path) = survey_tsv {
        emit_tsv(&tsv_path, &rows).map_err(|msg| {
            eprintln!("frontier_probe_survey: {msg}");
            ExitCode::from(1)
        })?;
        eprintln!("frontier_probe_survey: wrote {}", tsv_path.display());
    }
    let summary = cluster_summary(&rows);
    eprintln!("{summary}");
    if let Some(parent) = survey_manifest.parent() {
        let summary_path = parent.join("frontier_probe_survey_summary.txt");
        let _ = fs::write(&summary_path, &summary);
    }
    eprintln!(
        "frontier_probe_survey: wrote {} ({} receipt(s))",
        survey_manifest.display(),
        rows.len()
    );
    Ok(ExitCode::from(0))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
