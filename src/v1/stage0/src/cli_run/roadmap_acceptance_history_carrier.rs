use std::rc::Rc;

use serde::Deserialize;

use crate::v1_interpreter::{
    list_value, sorted_fields, InterpContext, InterpError, InterpResult, Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RoadmapAcceptanceEventHistoryParse {
    Parsed { events: Vec<Value> },
    Refused { detail: String },
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(tag = "variant", rename_all = "snake_case")]
enum JsonAcceptanceRevocationDisposition {
    AcceptanceNodeReopensActiveFrontier,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonReceipt {
    node: String,
    criteria_digest: String,
    red_control: JsonRedControl,
    handback: JsonHandback,
    accepted_by: String,
    accepted_on: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "variant", rename_all = "snake_case", deny_unknown_fields)]
enum JsonRedControl {
    RedControlNotRun,
    RedControlExecuted {
        witness_module: String,
        witness_fn: String,
        executed_on: String,
    },
}

#[derive(Debug, Deserialize)]
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

fn fnv1a64_structural_value(digest: String, ctx: &InterpContext) -> Value {
    Value::Record {
        type_name: ctx.sym("Fnv1a64Structural"),
        fields: Rc::new(sorted_fields(vec![(ctx.sym("digest"), Value::Str(digest))])),
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
                (ctx.sym("witness_module"), Value::Str(witness_module)),
                (ctx.sym("witness_fn"), Value::Str(witness_fn)),
                (ctx.sym("executed_on"), Value::Str(executed_on)),
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
                (ctx.sym("first_artifact"), Value::Str(first_artifact)),
                (
                    ctx.sym("further_artifacts"),
                    list_value(
                        further_artifacts
                            .into_iter()
                            .map(Value::Str)
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
            (ctx.sym("node"), Value::Str(receipt.node)),
            (
                ctx.sym("criteria_digest"),
                fnv1a64_structural_value(receipt.criteria_digest, ctx),
            ),
            (
                ctx.sym("red_control"),
                red_control_value(receipt.red_control, ctx)?,
            ),
            (ctx.sym("handback"), handback_value(receipt.handback, ctx)?),
            (ctx.sym("accepted_by"), Value::Str(receipt.accepted_by)),
            (ctx.sym("accepted_on"), Value::Str(receipt.accepted_on)),
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
                    (ctx.sym("reason"), Value::Str(reason)),
                    (ctx.sym("revoked_by"), Value::Str(revoked_by)),
                    (ctx.sym("revoked_on"), Value::Str(revoked_on)),
                ])),
            }
        }
    })
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
            fields: Rc::new(sorted_fields(vec![(ctx.sym("detail"), Value::Str(detail))])),
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
