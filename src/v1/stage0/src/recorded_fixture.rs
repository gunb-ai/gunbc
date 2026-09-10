use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::v1_interpreter::{sorted_fields, str_value, InterpContext, Value};
use crate::v1_rt;
use crate::v1_std_core::{authored_name_at, param_node_name_at, Node};

pub const FIXTURE_FRESHNESS_SECS: u64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedFixture {
    pub operation: String,
    pub input_hash: String,
    pub inputs: serde_json::Value,
    pub response: serde_json::Value,
    pub recorded_at: u64,
}

#[derive(Debug)]
pub enum FixtureError {
    Missing {
        operation: String,
        input_hash: String,
    },
    Stale {
        operation: String,
        stored_hash: String,
        current_hash: String,
    },
    InputMismatch {
        operation: String,
        input_hash: String,
    },
    Expired {
        operation: String,
        input_hash: String,
        recorded_at: u64,
        age_secs: u64,
        max_age_secs: u64,
    },
    ResponseDrift {
        operation: String,
        input_hash: String,
    },
    DeserializationMismatch {
        reason: String,
    },
    UnknownTag {
        tag: String,
    },
    UnreplayableValue {
        kind: String,
    },
    ClockUnavailable,
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    InvalidDigest {
        digest: String,
    },
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixtureError::Missing {
                operation,
                input_hash,
            } => write!(
                f,
                "missing recorded fixture for {} (input_hash={})",
                operation, input_hash
            ),
            FixtureError::Stale {
                operation,
                stored_hash,
                current_hash,
            } => write!(
                f,
                "stale recorded fixture for {}: stored input_hash={} but current input_hash={} — refusing to replay stale value",
                operation, stored_hash, current_hash
            ),
            FixtureError::InputMismatch {
                operation,
                input_hash,
            } => write!(
                f,
                "recorded fixture input mismatch for {} (input_hash={}): stored inputs do not match current call — refusing to replay (possible hash collision)",
                operation, input_hash
            ),
            FixtureError::Expired {
                operation,
                input_hash,
                recorded_at,
                age_secs,
                max_age_secs,
            } => write!(
                f,
                "expired recorded fixture for {} (input_hash={}): recorded_at={} age={}s > max={}s — refusing to replay stale value",
                operation, input_hash, recorded_at, age_secs, max_age_secs
            ),
            FixtureError::ResponseDrift {
                operation,
                input_hash,
            } => write!(
                f,
                "recorded fixture response drift for {} (input_hash={}): wet capture returned a different response for the same input_hash — refusing to overwrite (cache-purity oracle)",
                operation, input_hash
            ),
            FixtureError::DeserializationMismatch { reason } => {
                write!(f, "fixture deserialization mismatch: {reason}")
            }
            FixtureError::UnknownTag { tag } => {
                write!(f, "fixture unknown tag {:?} — refusing to fabricate a value", tag)
            }
            FixtureError::UnreplayableValue { kind } => {
                write!(
                    f,
                    "fixture cannot record unreplayable value kind {kind:?} — refusing to write an unfaithful fixture"
                )
            }
            FixtureError::ClockUnavailable => {
                write!(f, "fixture clock unavailable — refusing to guess recorded_at or freshness")
            }
            FixtureError::Io { path, source } => {
                write!(f, "fixture I/O error at {}: {}", path.display(), source)
            }
            FixtureError::Json { path, source } => {
                write!(f, "fixture JSON error at {}: {}", path.display(), source)
            }
            FixtureError::InvalidDigest { digest } => {
                write!(f, "fixture input_hash must be 16-char hex, got {:?}", digest)
            }
        }
    }
}

impl std::error::Error for FixtureError {}

#[derive(Debug, Clone)]
pub struct RecordedFixtureStore {
    root: PathBuf,
}

impl RecordedFixtureStore {
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn fixture_path(&self, operation: &str, input_hash: &str) -> PathBuf {
        self.root
            .join(operation_slug(operation))
            .join(format!("{}.json", input_hash))
    }

    fn read_fixture_file(&self, path: &Path) -> Result<RecordedFixture, FixtureError> {
        let bytes = fs::read(path).map_err(|e| FixtureError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        serde_json::from_slice(&bytes).map_err(|e| FixtureError::Json {
            path: path.to_path_buf(),
            source: e,
        })
    }

    pub fn assert_fresh(
        fixture: &RecordedFixture,
        operation: &str,
        input_hash: &str,
        now_secs: u64,
    ) -> Result<(), FixtureError> {
        let age = now_secs.saturating_sub(fixture.recorded_at);
        if age > FIXTURE_FRESHNESS_SECS {
            return Err(FixtureError::Expired {
                operation: operation.to_string(),
                input_hash: input_hash.to_string(),
                recorded_at: fixture.recorded_at,
                age_secs: age,
                max_age_secs: FIXTURE_FRESHNESS_SECS,
            });
        }
        Ok(())
    }

    pub fn lookup(
        &self,
        operation: &str,
        input_hash: &str,
        inputs: &serde_json::Value,
        now_secs: u64,
    ) -> Result<RecordedFixture, FixtureError> {
        expect_hash_digest(input_hash)?;
        let path = self.fixture_path(operation, input_hash);
        if !path.is_file() {
            return Err(FixtureError::Missing {
                operation: operation.to_string(),
                input_hash: input_hash.to_string(),
            });
        }
        let fixture = self.read_fixture_file(&path)?;
        if fixture.operation != operation {
            return Err(FixtureError::Stale {
                operation: operation.to_string(),
                stored_hash: fixture.input_hash.clone(),
                current_hash: input_hash.to_string(),
            });
        }
        if fixture.input_hash != input_hash {
            return Err(FixtureError::Stale {
                operation: operation.to_string(),
                stored_hash: fixture.input_hash,
                current_hash: input_hash.to_string(),
            });
        }
        if fixture.inputs != *inputs {
            return Err(FixtureError::InputMismatch {
                operation: operation.to_string(),
                input_hash: input_hash.to_string(),
            });
        }
        Self::assert_fresh(&fixture, operation, input_hash, now_secs)?;
        Ok(fixture)
    }

    pub fn record(
        &self,
        operation: &str,
        input_hash: &str,
        inputs: &serde_json::Value,
        response: &Value,
        ctx: &InterpContext,
        now_secs: u64,
    ) -> Result<(), FixtureError> {
        expect_hash_digest(input_hash)?;
        let response_json = value_to_fixture_json(response, ctx)?;
        let path = self.fixture_path(operation, input_hash);
        if path.is_file() {
            let existing = self.read_fixture_file(&path)?;
            if existing.response != response_json {
                return Err(FixtureError::ResponseDrift {
                    operation: operation.to_string(),
                    input_hash: input_hash.to_string(),
                });
            }
        }
        let fixture = RecordedFixture {
            operation: operation.to_string(),
            input_hash: input_hash.to_string(),
            inputs: inputs.clone(),
            response: response_json,
            recorded_at: now_secs,
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| FixtureError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let bytes = serde_json::to_vec_pretty(&fixture).map_err(|e| FixtureError::Json {
            path: path.clone(),
            source: e,
        })?;
        fs::write(&path, bytes).map_err(|e| FixtureError::Io { path, source: e })
    }
}

pub fn service_inputs_fixture_json(
    op_node: &Rc<Node>,
    param_env: &crate::v1_interpreter::Env,
    ctx: &InterpContext,
) -> Result<serde_json::Value, FixtureError> {
    let si = ctx.source_indices();
    let mut rows = Vec::new();
    for param in op_node.params.iter() {
        let name = param_node_name_at(param.clone(), si.clone());
        let key = ctx.sym(&name);
        let value = match param_env.lookup(key) {
            Some(val) => value_to_fixture_json(val, ctx)?,
            None => serde_json::Value::Null,
        };
        rows.push(json!({ "name": name, "value": value }));
    }
    Ok(serde_json::Value::Array(rows))
}

pub fn content_hash_service_inputs(
    op_node: &Rc<Node>,
    param_env: &crate::v1_interpreter::Env,
    ctx: &InterpContext,
) -> String {
    let si = ctx.source_indices();
    let mut digest = "0000000000000000".to_string();
    for param in op_node.params.iter() {
        let name = param_node_name_at(param.clone(), si.clone());
        let key = ctx.sym(&name);
        let limb = match param_env.lookup(key) {
            Some(val) => format!("{:016x}", crate::v1_interpreter::value_hash_public(val)),
            None => "0000000000000000".to_string(),
        };
        digest = v1_rt::hash_combine(digest, limb);
    }
    digest
}

fn expect_hash_digest(digest: &str) -> Result<(), FixtureError> {
    if v1_rt::is_hash_digest(digest) {
        Ok(())
    } else {
        Err(FixtureError::InvalidDigest {
            digest: digest.to_string(),
        })
    }
}

fn operation_slug(operation: &str) -> String {
    operation.replace('.', "__")
}

fn require_object(
    json: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, FixtureError> {
    json.as_object()
        .ok_or_else(|| FixtureError::DeserializationMismatch {
            reason: "expected tagged object".to_string(),
        })
}

fn require_tag(obj: &serde_json::Map<String, serde_json::Value>) -> Result<&str, FixtureError> {
    obj.get("__tag")
        .and_then(|v| v.as_str())
        .ok_or_else(|| FixtureError::DeserializationMismatch {
            reason: "missing __tag".to_string(),
        })
}

fn require_bool(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<bool, FixtureError> {
    obj.get(key)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| FixtureError::DeserializationMismatch {
            reason: format!("missing or ill-typed bool field {key}"),
        })
}

fn require_i64(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<i64, FixtureError> {
    obj.get(key)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| FixtureError::DeserializationMismatch {
            reason: format!("missing or ill-typed int field {key}"),
        })
}

fn require_f64(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<f64, FixtureError> {
    obj.get(key)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| FixtureError::DeserializationMismatch {
            reason: format!("missing or ill-typed float field {key}"),
        })
}

fn require_str(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<String, FixtureError> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| FixtureError::DeserializationMismatch {
            reason: format!("missing or ill-typed string field {key}"),
        })
}

fn require_fields_obj(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<&serde_json::Map<String, serde_json::Value>, FixtureError> {
    obj.get("fields")
        .and_then(|v| v.as_object())
        .ok_or_else(|| FixtureError::DeserializationMismatch {
            reason: "missing or ill-typed fields object".to_string(),
        })
}

pub fn value_to_fixture_json(
    val: &Value,
    ctx: &InterpContext,
) -> Result<serde_json::Value, FixtureError> {
    match val {
        Value::Null => Ok(serde_json::Value::Null),
        Value::Unit => Ok(json!({ "__tag": "Unit" })),
        Value::Bool(b) => Ok(json!({ "__tag": "Bool", "value": b })),
        Value::Int(n) => Ok(json!({ "__tag": "Int", "value": n })),
        Value::Float(f) => Ok(json!({ "__tag": "Float", "value": f })),
        Value::Str(s) => Ok(json!({ "__tag": "Str", "value": s.as_str() })),
        Value::List(items) => {
            let arr: Result<Vec<_>, _> = items
                .iter()
                .map(|v| value_to_fixture_json(v, ctx))
                .collect();
            Ok(json!({ "__tag": "List", "items": arr? }))
        }
        Value::Record { type_name, fields } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                obj.insert(ctx.resolve(*k), value_to_fixture_json(v, ctx)?);
            }
            Ok(json!({
                "__tag": "Record",
                "__type": ctx.resolve(*type_name),
                "fields": obj,
            }))
        }
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                obj.insert(ctx.resolve(*k), value_to_fixture_json(v, ctx)?);
            }
            Ok(json!({
                "__tag": "Variant",
                "__type": ctx.resolve(*type_name),
                "__variant": ctx.resolve(*variant_name),
                "fields": obj,
            }))
        }
        Value::Map(entries) => {
            let mut encoded: Vec<(String, serde_json::Value, serde_json::Value)> =
                Vec::with_capacity(entries.len());
            for (k, v) in entries.iter() {
                let key_json = value_to_fixture_json(k.value_ref(), ctx)?;
                let value_json = value_to_fixture_json(v, ctx)?;
                let sort_key = canonical_json_sort_key(&key_json)?;
                encoded.push((sort_key, key_json, value_json));
            }
            encoded.sort_by(|a, b| a.0.cmp(&b.0));
            let items: Vec<serde_json::Value> = encoded
                .into_iter()
                .map(|(_, key, value)| json!({ "key": key, "value": value }))
                .collect();
            Ok(json!({ "__tag": "Map", "entries": items }))
        }
        Value::Set(members) => {
            let mut encoded: Vec<(String, serde_json::Value)> = Vec::with_capacity(members.len());
            for m in members.iter() {
                let member_json = value_to_fixture_json(&str_value(m.clone()), ctx)?;
                let sort_key = canonical_json_sort_key(&member_json)?;
                encoded.push((sort_key, member_json));
            }
            encoded.sort_by(|a, b| a.0.cmp(&b.0));
            let items: Vec<serde_json::Value> = encoded.into_iter().map(|(_, m)| m).collect();
            Ok(json!({ "__tag": "Set", "members": items }))
        }
        Value::Closure { .. } | Value::Fn { .. } => Err(FixtureError::UnreplayableValue {
            kind: val.type_label_public().to_string(),
        }),
    }
}

fn canonical_json_sort_key(json: &serde_json::Value) -> Result<String, FixtureError> {
    serde_json::to_string(json).map_err(|e| FixtureError::Json {
        path: PathBuf::from("<fixture-canonical-key>"),
        source: e,
    })
}

pub fn value_from_fixture_json(
    json: &serde_json::Value,
    ctx: &InterpContext,
) -> Result<Value, FixtureError> {
    use crate::v1_interpreter::list_value;

    if json.is_null() {
        return Ok(Value::Null);
    }

    let obj = require_object(json)?;
    let tag = require_tag(obj)?;
    match tag {
        "Unit" => Ok(Value::Unit),
        "Bool" => Ok(Value::Bool(require_bool(obj, "value")?)),
        "Int" => Ok(Value::Int(require_i64(obj, "value")?)),
        "Float" => Ok(Value::Float(require_f64(obj, "value")?)),
        "Str" => Ok(str_value(require_str(obj, "value")?)),
        "List" => {
            let items = obj.get("items").and_then(|v| v.as_array()).ok_or_else(|| {
                FixtureError::DeserializationMismatch {
                    reason: "List missing items array".to_string(),
                }
            })?;
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(value_from_fixture_json(item, ctx)?);
            }
            Ok(list_value(out))
        }
        "Record" => {
            let type_name = ctx.sym(&require_str(obj, "__type")?);
            let fields_obj = require_fields_obj(obj)?;
            let mut fields = Vec::with_capacity(fields_obj.len());
            for (k, v) in fields_obj {
                fields.push((ctx.sym(k), value_from_fixture_json(v, ctx)?));
            }
            Ok(Value::Record {
                type_name,
                fields: Rc::new(sorted_fields(fields)),
            })
        }
        "Variant" => {
            let type_name = ctx.sym(&require_str(obj, "__type")?);
            let variant_name = ctx.sym(&require_str(obj, "__variant")?);
            let fields_obj = require_fields_obj(obj)?;
            let mut fields = Vec::with_capacity(fields_obj.len());
            for (k, v) in fields_obj {
                fields.push((ctx.sym(k), value_from_fixture_json(v, ctx)?));
            }
            Ok(Value::Variant {
                type_name,
                variant_name,
                fields: Rc::new(sorted_fields(fields)),
            })
        }
        "Map" => {
            let entries = obj
                .get("entries")
                .and_then(|v| v.as_array())
                .ok_or_else(|| FixtureError::DeserializationMismatch {
                    reason: "Map missing entries array".to_string(),
                })?;
            let mut out = im::HashMap::new();
            for entry in entries {
                let pair = require_object(entry)?;
                let key_json =
                    pair.get("key")
                        .ok_or_else(|| FixtureError::DeserializationMismatch {
                            reason: "Map entry missing key".to_string(),
                        })?;
                let value_json =
                    pair.get("value")
                        .ok_or_else(|| FixtureError::DeserializationMismatch {
                            reason: "Map entry missing value".to_string(),
                        })?;
                let key = value_from_fixture_json(key_json, ctx)?;
                let ck = crate::v1_interpreter::CanonKey::new(key).ok_or_else(|| {
                    FixtureError::DeserializationMismatch {
                        reason: "Map key has no decidable identity".to_string(),
                    }
                })?;
                if out.contains_key(&ck) {
                    return Err(FixtureError::DeserializationMismatch {
                        reason: "duplicate Map key".to_string(),
                    });
                }
                out = out.update(ck, value_from_fixture_json(value_json, ctx)?);
            }
            Ok(crate::v1_interpreter::map_value(out))
        }
        "Set" => {
            let members = obj
                .get("members")
                .and_then(|v| v.as_array())
                .ok_or_else(|| FixtureError::DeserializationMismatch {
                    reason: "Set missing members array".to_string(),
                })?;
            let mut out = im::OrdSet::new();
            for member in members {
                let value = value_from_fixture_json(member, ctx)?;
                let text = match value {
                    Value::Str(s) => s.to_string(),
                    other => {
                        return Err(FixtureError::DeserializationMismatch {
                            reason: format!(
                                "Set member must be Str, got {}",
                                other.type_label_public()
                            ),
                        })
                    }
                };
                if out.contains(&text) {
                    return Err(FixtureError::DeserializationMismatch {
                        reason: "duplicate Set member".to_string(),
                    });
                }
                out.insert(text);
            }
            Ok(Value::Set(Rc::new(out)))
        }
        "Opaque" => Err(FixtureError::UnknownTag {
            tag: "Opaque".to_string(),
        }),
        other => Err(FixtureError::UnknownTag {
            tag: other.to_string(),
        }),
    }
}

pub fn operation_result_type_name(op_node: &Rc<Node>, ctx: &InterpContext) -> String {
    match op_node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => {
            authored_name_at(ctx.source_indices(), node.clone())
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod map_set_fixture_encoding_tests {
    use super::*;
    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;
    use crate::v1_interpreter::{map_value, CanonKey, Env, ExecutionMode, Value};
    use crate::v1_std_core::{make_expr_node, ExprData, SourceSpan};
    use im::vector as im_vec;
    use im::HashMap as HamtMap;
    use im::OrdSet;
    use std::rc::Rc;

    fn fresh_ctx() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HamtMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HamtMap::new()), ExecutionMode::Hermetic)
    }

    fn str_map(pairs: &[(&str, &str)]) -> Value {
        let mut entries = HamtMap::new();
        for (k, v) in pairs {
            let ck = CanonKey::new(str_value(*k)).expect("string key");
            entries = entries.update(ck, str_value(*v));
        }
        map_value(entries)
    }

    fn dummy_node() -> Rc<Node> {
        make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            Rc::new(SourceSpan {
                file: "fixture-encoding-test.dag".to_string(),
                start: 0,
                end: 0,
            }),
        )
    }

    #[test]
    fn map_roundtrip_equal_when_encoded_entry_order_differs() {
        let ctx = fresh_ctx();
        let map = str_map(&[
            ("com.cloudflare.api.account", "*"),
            ("com.cloudflare.edge.spectrum", "read"),
        ]);
        let encoded = value_to_fixture_json(&map, &ctx).expect("Map is recordable");
        let entries = encoded
            .get("entries")
            .and_then(|v| v.as_array())
            .expect("canonical Map encoding carries entries")
            .clone();
        assert!(
            entries.len() >= 2,
            "need at least two entries to permute order"
        );
        let mut reversed = encoded.clone();
        let mut rev_entries = entries.clone();
        rev_entries.reverse();
        assert_ne!(
            entries, rev_entries,
            "the two encodings must actually differ in array order"
        );
        reversed["entries"] = serde_json::Value::Array(rev_entries);
        let a = value_from_fixture_json(&encoded, &ctx).expect("canonical decode");
        let b = value_from_fixture_json(&reversed, &ctx).expect("reversed decode");
        assert_eq!(a, map);
        assert_eq!(b, map);
        assert_eq!(a, b);
    }

    #[test]
    fn map_whose_value_differs_compares_unequal() {
        let ctx = fresh_ctx();
        let a = str_map(&[("k", "v1")]);
        let b = str_map(&[("k", "v2")]);
        let aj = value_to_fixture_json(&a, &ctx).expect("record a");
        let bj = value_to_fixture_json(&b, &ctx).expect("record b");
        let ar = value_from_fixture_json(&aj, &ctx).expect("replay a");
        let br = value_from_fixture_json(&bj, &ctx).expect("replay b");
        assert_ne!(ar, br);
        assert_eq!(ar, a);
        assert_eq!(br, b);
    }

    #[test]
    fn duplicate_map_key_in_encoding_is_refused() {
        let ctx = fresh_ctx();
        let key = value_to_fixture_json(&str_value("k"), &ctx).expect("key");
        let val = value_to_fixture_json(&str_value("v"), &ctx).expect("val");
        let json = json!({
            "__tag": "Map",
            "entries": [
                { "key": key, "value": val },
                { "key": key, "value": val },
            ],
        });
        match value_from_fixture_json(&json, &ctx) {
            Err(FixtureError::DeserializationMismatch { reason }) => {
                assert!(
                    reason.contains("duplicate"),
                    "expected duplicate-key refusal, got {reason}"
                );
            }
            other => panic!("expected DeserializationMismatch, got {other:?}"),
        }
    }

    #[test]
    fn set_roundtrip_equal_when_encoded_member_order_differs() {
        let ctx = fresh_ctx();
        let mut members = OrdSet::new();
        members.insert("zeta".to_string());
        members.insert("alpha".to_string());
        let set = Value::Set(Rc::new(members));
        let encoded = value_to_fixture_json(&set, &ctx).expect("Set is recordable");
        let members_json = encoded
            .get("members")
            .and_then(|v| v.as_array())
            .expect("canonical Set encoding carries members")
            .clone();
        assert!(members_json.len() >= 2);
        let mut reversed = encoded.clone();
        let mut rev = members_json.clone();
        rev.reverse();
        assert_ne!(members_json, rev);
        reversed["members"] = serde_json::Value::Array(rev);
        let a = value_from_fixture_json(&encoded, &ctx).expect("canonical decode");
        let b = value_from_fixture_json(&reversed, &ctx).expect("reversed decode");
        assert_eq!(a, set);
        assert_eq!(b, set);
        assert_eq!(a, b);
    }

    #[test]
    fn duplicate_set_member_in_encoding_is_refused() {
        let ctx = fresh_ctx();
        let member = value_to_fixture_json(&str_value("x"), &ctx).expect("member");
        let json = json!({
            "__tag": "Set",
            "members": [member.clone(), member],
        });
        match value_from_fixture_json(&json, &ctx) {
            Err(FixtureError::DeserializationMismatch { reason }) => {
                assert!(
                    reason.contains("duplicate"),
                    "expected duplicate-member refusal, got {reason}"
                );
            }
            other => panic!("expected DeserializationMismatch, got {other:?}"),
        }
    }

    #[test]
    fn closure_and_fn_still_refuse_on_the_record_path() {
        let ctx = fresh_ctx();
        let node = dummy_node();
        let closure = Value::Closure {
            params: vec![],
            body: node.clone(),
            env: Env::empty(),
        };
        let fun = Value::Fn { node };
        for (label, value) in [("Closure", closure), ("Fn", fun)] {
            match value_to_fixture_json(&value, &ctx) {
                Err(FixtureError::UnreplayableValue { kind }) => {
                    assert_eq!(kind, label);
                    let msg = FixtureError::UnreplayableValue { kind: kind.clone() }.to_string();
                    assert!(
                        !msg.contains("Map") && !msg.contains("Set"),
                        "refusal must not implicate collection kinds: {msg}"
                    );
                }
                other => panic!("{label} must refuse on value_to_fixture_json, got {other:?}"),
            }
        }
    }

    #[test]
    fn store_record_accepts_map_and_refuses_closure() {
        let ctx = fresh_ctx();
        let dir = std::env::temp_dir().join(format!("gunbc-fixture-map-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let store = RecordedFixtureStore::open(&dir);
        let inputs = json!([]);
        let map = str_map(&[("a", "1"), ("b", "2")]);
        store
            .record("Test.Op", "0123456789abcdef", &inputs, &map, &ctx, 1)
            .expect("Map-carrying record must succeed");
        let fixture = store
            .lookup("Test.Op", "0123456789abcdef", &inputs, 1)
            .expect("lookup");
        let back = value_from_fixture_json(&fixture.response, &ctx).expect("replay");
        assert_eq!(back, map);

        let closure = Value::Closure {
            params: vec![],
            body: dummy_node(),
            env: Env::empty(),
        };
        match store.record(
            "Test.OpClosure",
            "0123456789abcdef",
            &inputs,
            &closure,
            &ctx,
            1,
        ) {
            Err(FixtureError::UnreplayableValue { kind }) => assert_eq!(kind, "Closure"),
            other => panic!("Closure must refuse on RecordedFixtureStore::record, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
