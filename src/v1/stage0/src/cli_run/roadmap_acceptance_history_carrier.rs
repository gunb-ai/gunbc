use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::v1_interpreter::{
    list_value, sorted_fields, str_value, InterpContext, InterpResult, Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RoadmapAcceptanceEventHistoryParse {
    Parsed { events: Vec<Value> },
    Refused { detail: String },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "variant", rename_all = "snake_case", deny_unknown_fields)]
enum JsonEvent {
    AcceptanceRecorded {
        receipt: JsonReceipt,
    },
    AcceptanceRevoked {
        exact_prior_receipt: JsonReceipt,
        disposition: JsonAcceptanceRevocationDisposition,
        reason: String,
        revoked_by: String,
        revoked_on: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "variant", rename_all = "snake_case", deny_unknown_fields)]
enum JsonAcceptanceRevocationDisposition {
    AcceptanceNodeReopensActiveFrontier,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct JsonReceipt {
    node: String,
    criteria_digest: String,
    red_control: JsonRedControl,
    handback: JsonHandback,
    accepted_by: String,
    accepted_on: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "variant", rename_all = "snake_case", deny_unknown_fields)]
enum JsonRedControl {
    RedControlNotRun,
    RedControlExecuted {
        witness_module: String,
        witness_fn: String,
        executed_on: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "variant", rename_all = "snake_case", deny_unknown_fields)]
enum JsonHandback {
    HandbackNotDelivered,
    HandbackDelivered {
        first_artifact: String,
        further_artifacts: Vec<String>,
    },
}

// Seed duplicate of std.content_hash lower_hex_16 / Fnv1a64StructuralDigestHex refinement.
// Dissolves when v1_compiler.cli_run roadmap_acceptance_history_carrier hollowing lands
// (roadmap_acceptance_event_history_jsonl_parser_seed_scaffold in gunbc.roadmap_acceptance_history_carrier).
fn validate_hex16(digest: &str, field: &str) -> Result<(), String> {
    if digest.len() != 16
        || !digest
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(format!("{field} must be 16 lowercase hex digits"));
    }
    Ok(())
}

// Seed bridge only: `validate_hex16` gates wire input before this call; the modeled
// `Fnv1a64StructuralDigestHex` refinement is not enforced at `Value` construction.
fn fnv1a64_structural_value(digest: String, ctx: &InterpContext) -> Value {
    Value::Record {
        type_name: ctx.sym("Fnv1a64Structural"),
        fields: Rc::new(sorted_fields(vec![(ctx.sym("digest"), str_value(digest))])),
    }
}

fn red_control_value(red_control: JsonRedControl, ctx: &InterpContext) -> Result<Value, String> {
    Ok(match red_control {
        JsonRedControl::RedControlNotRun => Value::Variant {
            type_name: ctx.sym("RedControlEvidence"),
            variant_name: ctx.sym("RedControlNotRun"),
            fields: Rc::new(Vec::new()),
        },
        JsonRedControl::RedControlExecuted {
            witness_module,
            witness_fn,
            executed_on,
        } => Value::Variant {
            type_name: ctx.sym("RedControlEvidence"),
            variant_name: ctx.sym("RedControlExecuted"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("witness_module"), str_value(witness_module)),
                (ctx.sym("witness_fn"), str_value(witness_fn)),
                (ctx.sym("executed_on"), str_value(executed_on)),
            ])),
        },
    })
}

fn handback_value(handback: JsonHandback, ctx: &InterpContext) -> Result<Value, String> {
    Ok(match handback {
        JsonHandback::HandbackNotDelivered => Value::Variant {
            type_name: ctx.sym("HandbackEvidence"),
            variant_name: ctx.sym("HandbackNotDelivered"),
            fields: Rc::new(Vec::new()),
        },
        JsonHandback::HandbackDelivered {
            first_artifact,
            further_artifacts,
        } => Value::Variant {
            type_name: ctx.sym("HandbackEvidence"),
            variant_name: ctx.sym("HandbackDelivered"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("first_artifact"), str_value(first_artifact)),
                (
                    ctx.sym("further_artifacts"),
                    list_value(
                        further_artifacts
                            .into_iter()
                            .map(str_value)
                            .collect::<Vec<_>>(),
                    ),
                ),
            ])),
        },
    })
}

fn receipt_value(receipt: JsonReceipt, ctx: &InterpContext) -> Result<Value, String> {
    validate_hex16(&receipt.criteria_digest, "criteria_digest")?;
    Ok(Value::Record {
        type_name: ctx.sym("RoadmapAcceptanceReceipt"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("node"), str_value(receipt.node)),
            (
                ctx.sym("criteria_digest"),
                fnv1a64_structural_value(receipt.criteria_digest, ctx),
            ),
            (
                ctx.sym("red_control"),
                red_control_value(receipt.red_control, ctx)?,
            ),
            (ctx.sym("handback"), handback_value(receipt.handback, ctx)?),
            (ctx.sym("accepted_by"), str_value(receipt.accepted_by)),
            (ctx.sym("accepted_on"), str_value(receipt.accepted_on)),
        ])),
    })
}

fn event_value(event: JsonEvent, ctx: &InterpContext) -> Result<Value, String> {
    Ok(match event {
        JsonEvent::AcceptanceRecorded { receipt } => Value::Variant {
            type_name: ctx.sym("RoadmapAcceptanceEvent"),
            variant_name: ctx.sym("AcceptanceRecorded"),
            fields: Rc::new(sorted_fields(vec![(
                ctx.sym("receipt"),
                receipt_value(receipt, ctx)?,
            )])),
        },
        JsonEvent::AcceptanceRevoked {
            exact_prior_receipt,
            disposition,
            reason,
            revoked_by,
            revoked_on,
        } => {
            let disposition_value = match disposition {
                JsonAcceptanceRevocationDisposition::AcceptanceNodeReopensActiveFrontier => {
                    Value::Variant {
                        type_name: ctx.sym("AcceptanceRevocationDisposition"),
                        variant_name: ctx.sym("AcceptanceNodeReopensActiveFrontier"),
                        fields: Rc::new(Vec::new()),
                    }
                }
            };
            Value::Variant {
                type_name: ctx.sym("RoadmapAcceptanceEvent"),
                variant_name: ctx.sym("AcceptanceRevoked"),
                fields: Rc::new(sorted_fields(vec![
                    (
                        ctx.sym("exact_prior_receipt"),
                        receipt_value(exact_prior_receipt, ctx)?,
                    ),
                    (ctx.sym("disposition"), disposition_value),
                    (ctx.sym("reason"), str_value(reason)),
                    (ctx.sym("revoked_by"), str_value(revoked_by)),
                    (ctx.sym("revoked_on"), str_value(revoked_on)),
                ])),
            }
        }
    })
}

fn field_value<'a>(
    value: &'a Value,
    field: &str,
    ctx: &InterpContext,
) -> Result<&'a Value, String> {
    let sym = ctx.sym(field);
    match value {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => fields
            .iter()
            .find(|(k, _)| *k == sym)
            .map(|(_, v)| v)
            .ok_or_else(|| format!("missing field `{field}`")),
        other => Err(format!(
            "expected Record or Variant, got {}",
            other.type_label_public()
        )),
    }
}

fn field_str(value: &Value, field: &str, ctx: &InterpContext) -> Result<String, String> {
    match field_value(value, field, ctx)? {
        Value::Str(s) => Ok(s.to_string()),
        other => Err(format!(
            "field `{field}` must be String, got {}",
            other.type_label_public()
        )),
    }
}

fn field_list(value: &Value, field: &str, ctx: &InterpContext) -> Result<Vec<Value>, String> {
    match field_value(value, field, ctx)? {
        Value::List(items) => Ok(items.iter().cloned().collect()),
        other => Err(format!(
            "field `{field}` must be List, got {}",
            other.type_label_public()
        )),
    }
}

fn variant_name(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant { variant_name, .. } => Ok(ctx.resolve(*variant_name)),
        other => Err(format!(
            "expected Variant, got {}",
            other.type_label_public()
        )),
    }
}

fn red_control_to_json(value: &Value, ctx: &InterpContext) -> Result<JsonRedControl, String> {
    match variant_name(value, ctx)?.as_str() {
        "RedControlNotRun" => Ok(JsonRedControl::RedControlNotRun),
        "RedControlExecuted" => Ok(JsonRedControl::RedControlExecuted {
            witness_module: field_str(value, "witness_module", ctx)?,
            witness_fn: field_str(value, "witness_fn", ctx)?,
            executed_on: field_str(value, "executed_on", ctx)?,
        }),
        name => Err(format!("unknown RedControlEvidence variant `{name}`")),
    }
}

fn handback_to_json(value: &Value, ctx: &InterpContext) -> Result<JsonHandback, String> {
    match variant_name(value, ctx)?.as_str() {
        "HandbackNotDelivered" => Ok(JsonHandback::HandbackNotDelivered),
        "HandbackDelivered" => {
            let further = field_list(value, "further_artifacts", ctx)?
                .into_iter()
                .map(|item| match item {
                    Value::Str(s) => Ok(s.to_string()),
                    other => Err(format!(
                        "further_artifacts element must be String, got {}",
                        other.type_label_public()
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(JsonHandback::HandbackDelivered {
                first_artifact: field_str(value, "first_artifact", ctx)?,
                further_artifacts: further,
            })
        }
        name => Err(format!("unknown HandbackEvidence variant `{name}`")),
    }
}

fn receipt_to_json(value: &Value, ctx: &InterpContext) -> Result<JsonReceipt, String> {
    let criteria_digest = match field_value(value, "criteria_digest", ctx)? {
        Value::Record { fields, .. } => {
            let sym = ctx.sym("digest");
            fields
                .iter()
                .find(|(k, _)| *k == sym)
                .and_then(|(_, v)| match v {
                    Value::Str(s) => Some(s.to_string()),
                    _ => None,
                })
                .ok_or_else(|| "criteria_digest.digest must be String".to_string())?
        }
        other => {
            return Err(format!(
                "criteria_digest must be Fnv1a64Structural record, got {}",
                other.type_label_public()
            ));
        }
    };
    Ok(JsonReceipt {
        node: field_str(value, "node", ctx)?,
        criteria_digest,
        red_control: red_control_to_json(field_value(value, "red_control", ctx)?, ctx)?,
        handback: handback_to_json(field_value(value, "handback", ctx)?, ctx)?,
        accepted_by: field_str(value, "accepted_by", ctx)?,
        accepted_on: field_str(value, "accepted_on", ctx)?,
    })
}

fn event_to_json(value: &Value, ctx: &InterpContext) -> Result<JsonEvent, String> {
    match variant_name(value, ctx)?.as_str() {
        "AcceptanceRecorded" => Ok(JsonEvent::AcceptanceRecorded {
            receipt: receipt_to_json(field_value(value, "receipt", ctx)?, ctx)?,
        }),
        "AcceptanceRevoked" => {
            let disposition =
                match variant_name(field_value(value, "disposition", ctx)?, ctx)?.as_str() {
                    "AcceptanceNodeReopensActiveFrontier" => {
                        JsonAcceptanceRevocationDisposition::AcceptanceNodeReopensActiveFrontier
                    }
                    name => {
                        return Err(format!(
                            "unknown AcceptanceRevocationDisposition variant `{name}`"
                        ));
                    }
                };
            Ok(JsonEvent::AcceptanceRevoked {
                exact_prior_receipt: receipt_to_json(
                    field_value(value, "exact_prior_receipt", ctx)?,
                    ctx,
                )?,
                disposition,
                reason: field_str(value, "reason", ctx)?,
                revoked_by: field_str(value, "revoked_by", ctx)?,
                revoked_on: field_str(value, "revoked_on", ctx)?,
            })
        }
        name => Err(format!("unknown RoadmapAcceptanceEvent variant `{name}`")),
    }
}

/// Serialize interpreted events to the JSONL carrier wire form. Used to remap bootstrap
/// projection values from an isolated overlay context into the caller's symbol table.
pub fn serialize_roadmap_acceptance_events_to_jsonl(
    events: &[Value],
    ctx: &InterpContext,
) -> Result<String, String> {
    let mut lines = Vec::with_capacity(events.len());
    for (index, event) in events.iter().enumerate() {
        let json = event_to_json(event, ctx)?;
        let line = serde_json::to_string(&json)
            .map_err(|error| format!("event {index}: json encode failed: {error}"))?;
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

pub fn parse_roadmap_acceptance_event_history_jsonl(
    content: &str,
    ctx: &InterpContext,
) -> RoadmapAcceptanceEventHistoryParse {
    let mut events = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let json: JsonEvent = match serde_json::from_str(trimmed) {
            Ok(event) => event,
            Err(error) => {
                return RoadmapAcceptanceEventHistoryParse::Refused {
                    detail: format!("line {}: {error}", line_no + 1),
                };
            }
        };
        match event_value(json, ctx) {
            Ok(value) => events.push(value),
            Err(detail) => {
                return RoadmapAcceptanceEventHistoryParse::Refused {
                    detail: format!("line {}: {detail}", line_no + 1),
                };
            }
        }
    }
    RoadmapAcceptanceEventHistoryParse::Parsed { events }
}

pub fn parse_roadmap_acceptance_event_history_jsonl_builtin(
    content: &str,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let parsed = parse_roadmap_acceptance_event_history_jsonl(content, ctx);
    Ok(match parsed {
        RoadmapAcceptanceEventHistoryParse::Parsed { events } => Value::Variant {
            type_name: ctx.sym("RoadmapAcceptanceEventHistoryParse"),
            variant_name: ctx.sym("RoadmapAcceptanceEventHistoryParsed"),
            fields: Rc::new(sorted_fields(vec![(ctx.sym("events"), list_value(events))])),
        },
        RoadmapAcceptanceEventHistoryParse::Refused { detail } => Value::Variant {
            type_name: ctx.sym("RoadmapAcceptanceEventHistoryParse"),
            variant_name: ctx.sym("RoadmapAcceptanceEventHistoryParseRefused"),
            fields: Rc::new(sorted_fields(vec![(ctx.sym("detail"), str_value(detail))])),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;
    use crate::v1_interpreter::ExecutionMode;
    use im::HashMap;
    use std::rc::Rc;

    fn empty_ctx() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im::Vector::new()),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im::Vector::new()),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
    }

    #[test]
    fn empty_jsonl_parses_to_empty_list() {
        let ctx = empty_ctx();
        match parse_roadmap_acceptance_event_history_jsonl("", &ctx) {
            RoadmapAcceptanceEventHistoryParse::Parsed { events } => assert!(events.is_empty()),
            RoadmapAcceptanceEventHistoryParse::Refused { detail } => {
                panic!("unexpected refusal: {detail}")
            }
        }
    }

    #[test]
    fn malformed_jsonl_refuses_with_line_number() {
        let ctx = empty_ctx();
        match parse_roadmap_acceptance_event_history_jsonl("{ not json", &ctx) {
            RoadmapAcceptanceEventHistoryParse::Refused { detail } => {
                assert!(detail.starts_with("line 1:"));
            }
            RoadmapAcceptanceEventHistoryParse::Parsed { .. } => panic!("expected refusal"),
        }
    }

    #[test]
    fn invalid_digest_refuses() {
        let ctx = empty_ctx();
        let jsonl = r#"{"variant":"acceptance_recorded","receipt":{"node":"n","criteria_digest":"short","red_control":{"variant":"red_control_not_run"},"handback":{"variant":"handback_not_delivered"},"accepted_by":"op","accepted_on":"2026-01-01"}}"#;
        match parse_roadmap_acceptance_event_history_jsonl(jsonl, &ctx) {
            RoadmapAcceptanceEventHistoryParse::Refused { detail } => {
                assert!(detail.contains("criteria_digest"));
            }
            RoadmapAcceptanceEventHistoryParse::Parsed { .. } => panic!("expected refusal"),
        }
    }

    #[test]
    fn unknown_jsonl_field_refuses() {
        let ctx = empty_ctx();
        let jsonl = r#"{"variant":"acceptance_recorded","receipt":{"node":"n","criteria_digest":"0123456789abcdef","extra":"x","red_control":{"variant":"red_control_not_run"},"handback":{"variant":"handback_not_delivered"},"accepted_by":"op","accepted_on":"2026-01-01"}}"#;
        match parse_roadmap_acceptance_event_history_jsonl(jsonl, &ctx) {
            RoadmapAcceptanceEventHistoryParse::Refused { detail } => {
                assert!(detail.starts_with("line 1:"));
            }
            RoadmapAcceptanceEventHistoryParse::Parsed { .. } => panic!("expected refusal"),
        }
    }
}
