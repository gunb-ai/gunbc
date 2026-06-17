// recorded_fixture.rs — Hermetic record/replay fixture store for service operations.
//
// Phase 2 hermetic rollout: fixtures are keyed by (operation, content_hash(inputs)).
// `--record` captures wet dispatch results; `--hermetic --fixture-store` replays them.
// Staleness is fail-closed: a fixture whose stored inputs_hash does not match the
// current call's recomputed hash is rejected with a loud diagnostic — never a stale value.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::v1_interpreter::{InterpContext, Value};
use crate::v1_rt;
use crate::v1_std_core::{authored_name_at, param_node_name_at, Node};

/// On-disk fixture row: one wet-captured service response keyed by inputs hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedFixture {
    pub operation: String,
    pub inputs_hash: String,
    pub response: serde_json::Value,
}

#[derive(Debug)]
pub enum FixtureError {
    Missing { operation: String, inputs_hash: String },
    Stale {
        operation: String,
        stored_hash: String,
        current_hash: String,
    },
    Io { path: PathBuf, source: std::io::Error },
    Json { path: PathBuf, source: serde_json::Error },
    InvalidDigest { digest: String },
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixtureError::Missing {
                operation,
                inputs_hash,
            } => write!(
                f,
                "missing recorded fixture for {} (inputs_hash={})",
                operation, inputs_hash
            ),
            FixtureError::Stale {
                operation,
                stored_hash,
                current_hash,
            } => write!(
                f,
                "stale recorded fixture for {}: stored inputs_hash={} but current inputs_hash={} — refusing to replay stale value",
                operation, stored_hash, current_hash
            ),
            FixtureError::Io { path, source } => {
                write!(f, "fixture I/O error at {}: {}", path.display(), source)
            }
            FixtureError::Json { path, source } => {
                write!(f, "fixture JSON error at {}: {}", path.display(), source)
            }
            FixtureError::InvalidDigest { digest } => {
                write!(f, "fixture inputs_hash must be 16-char hex, got {:?}", digest)
            }
        }
    }
}

impl std::error::Error for FixtureError {}

/// Append-only-on-record directory store: `{root}/{operation_slug}/{inputs_hash}.json`.
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

    fn fixture_path(&self, operation: &str, inputs_hash: &str) -> PathBuf {
        self.root
            .join(operation_slug(operation))
            .join(format!("{}.json", inputs_hash))
    }

    pub fn lookup(
        &self,
        operation: &str,
        inputs_hash: &str,
    ) -> Result<RecordedFixture, FixtureError> {
        expect_hash_digest(inputs_hash)?;
        let path = self.fixture_path(operation, inputs_hash);
        if !path.is_file() {
            return Err(FixtureError::Missing {
                operation: operation.to_string(),
                inputs_hash: inputs_hash.to_string(),
            });
        }
        let bytes = fs::read(&path).map_err(|e| FixtureError::Io {
            path: path.clone(),
            source: e,
        })?;
        let fixture: RecordedFixture =
            serde_json::from_slice(&bytes).map_err(|e| FixtureError::Json {
                path: path.clone(),
                source: e,
            })?;
        if fixture.operation != operation {
            return Err(FixtureError::Stale {
                operation: operation.to_string(),
                stored_hash: fixture.inputs_hash.clone(),
                current_hash: inputs_hash.to_string(),
            });
        }
        if fixture.inputs_hash != inputs_hash {
            return Err(FixtureError::Stale {
                operation: operation.to_string(),
                stored_hash: fixture.inputs_hash,
                current_hash: inputs_hash.to_string(),
            });
        }
        Ok(fixture)
    }

    pub fn record(
        &self,
        operation: &str,
        inputs_hash: &str,
        response: &Value,
        ctx: &InterpContext,
    ) -> Result<(), FixtureError> {
        expect_hash_digest(inputs_hash)?;
        let fixture = RecordedFixture {
            operation: operation.to_string(),
            inputs_hash: inputs_hash.to_string(),
            response: value_to_fixture_json(response, ctx),
        };
        let path = self.fixture_path(operation, inputs_hash);
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
        fs::write(&path, bytes).map_err(|e| FixtureError::Io {
            path,
            source: e,
        })
    }
}

/// Structural content_hash of a service operation's bound inputs (param order).
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

pub fn value_to_fixture_json(val: &Value, ctx: &InterpContext) -> serde_json::Value {
    use serde_json::json;
    match val {
        Value::Null => serde_json::Value::Null,
        Value::Unit => json!({ "__tag": "Unit" }),
        Value::Bool(b) => json!({ "__tag": "Bool", "value": b }),
        Value::Int(n) => json!({ "__tag": "Int", "value": n }),
        Value::Float(f) => json!({ "__tag": "Float", "value": f }),
        Value::Str(s) => json!({ "__tag": "Str", "value": s }),
        Value::List(items) => {
            let arr: Vec<_> = items
                .iter()
                .map(|v| value_to_fixture_json(v, ctx))
                .collect();
            json!({ "__tag": "List", "items": arr })
        }
        Value::Record { type_name, fields } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                obj.insert(ctx.resolve(*k), value_to_fixture_json(v, ctx));
            }
            json!({
                "__tag": "Record",
                "__type": ctx.resolve(*type_name),
                "fields": obj,
            })
        }
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                obj.insert(ctx.resolve(*k), value_to_fixture_json(v, ctx));
            }
            json!({
                "__tag": "Variant",
                "__type": ctx.resolve(*type_name),
                "__variant": ctx.resolve(*variant_name),
                "fields": obj,
            })
        }
        other => json!({
            "__tag": "Opaque",
            "repr": format!("{:?}", other),
        }),
    }
}

pub fn value_from_fixture_json(json: &serde_json::Value, ctx: &InterpContext) -> Value {
    use crate::v1_interpreter::list_value;
    let Some(obj) = json.as_object() else {
        return Value::Null;
    };
    let tag = obj
        .get("__tag")
        .and_then(|v| v.as_str())
        .unwrap_or("Null");
    match tag {
        "Unit" => Value::Unit,
        "Bool" => Value::Bool(obj.get("value").and_then(|v| v.as_bool()).unwrap_or(false)),
        "Int" => Value::Int(obj.get("value").and_then(|v| v.as_i64()).unwrap_or(0)),
        "Float" => Value::Float(obj.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0)),
        "Str" => Value::Str(
            obj.get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ),
        "List" => {
            let items = obj
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|v| value_from_fixture_json(v, ctx))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            list_value(items)
        }
        "Record" => {
            let type_name = ctx.sym(
                obj.get("__type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Record"),
            );
            let mut fields = HashMap::new();
            if let Some(fields_obj) = obj.get("fields").and_then(|v| v.as_object()) {
                for (k, v) in fields_obj {
                    fields.insert(ctx.sym(k), value_from_fixture_json(v, ctx));
                }
            }
            Value::Record {
                type_name,
                fields: Rc::new(fields),
            }
        }
        "Variant" => {
            let type_name = ctx.sym(
                obj.get("__type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Variant"),
            );
            let variant_name = ctx.sym(
                obj.get("__variant")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown"),
            );
            let mut fields = HashMap::new();
            if let Some(fields_obj) = obj.get("fields").and_then(|v| v.as_object()) {
                for (k, v) in fields_obj {
                    fields.insert(ctx.sym(k), value_from_fixture_json(v, ctx));
                }
            }
            Value::Variant {
                type_name,
                variant_name,
                fields: Rc::new(fields),
            }
        }
        _ => Value::Null,
    }
}

/// Resolve the return type name for an operation (for fixture round-trip checks).
pub fn operation_result_type_name(op_node: &Rc<Node>, ctx: &InterpContext) -> String {
    match op_node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => {
            authored_name_at(ctx.source_indices(), node.clone())
        }
        _ => String::new(),
    }
}
