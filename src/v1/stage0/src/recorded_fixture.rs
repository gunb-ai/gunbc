use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::v1_interpreter::{InterpContext, Value};
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
        Value::Str(s) => Ok(json!({ "__tag": "Str", "value": s })),
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
        Value::Map(_) | Value::Set(_) | Value::Closure { .. } | Value::Fn { .. } => {
            Err(FixtureError::UnreplayableValue {
                kind: val.type_label_public().to_string(),
            })
        }
    }
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
        "Str" => Ok(Value::Str(require_str(obj, "value")?)),
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
            fields.sort_unstable_by_key(|(k, _)| k.0);
            Ok(Value::Record {
                type_name,
                fields: Rc::new(fields),
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
            fields.sort_unstable_by_key(|(k, _)| k.0);
            Ok(Value::Variant {
                type_name,
                variant_name,
                fields: Rc::new(fields),
            })
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
