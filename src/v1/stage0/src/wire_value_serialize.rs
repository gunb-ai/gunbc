use std::collections::HashMap;
use std::rc::Rc;

use crate::v1_compiler_emit_rust::{
    policy_is_string_variant, policy_is_untagged, policy_serde_tag_field,
    resolve_local_coproduct_wire_policy, rust_serde_tag_attr, rust_tagged_object_policy,
    wire_variant_tag_for_policy, RustEnumWireSerde,
};
use crate::v1_compiler_infer_items::TypedModule;
use crate::v1_interpreter::{InterpContext, Value};
use crate::v1_std_core::{module_imports, NewlineIndex};

type WireResult<T> = Result<T, String>;

pub fn resolve_coproduct_wire_policy(
    coproduct_name: &str,
    modules: &[Rc<TypedModule>],
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<Rc<RustEnumWireSerde>> {
    let si = Rc::new(source_indices.clone());
    let mut matches: Vec<Rc<RustEnumWireSerde>> = Vec::new();
    for tm in modules {
        let imports = module_imports(tm.module.clone());
        if let Some(local) = resolve_local_coproduct_wire_policy(
            coproduct_name.to_string(),
            false,
            tm.items.clone(),
            imports,
            si.clone(),
        ) {
            if local.error_message.is_none() {
                matches.push(local);
            }
        }
    }
    if matches.is_empty() {
        None
    } else if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        let first = &matches[0];
        if matches.iter().all(|m| m == first) {
            Some(first.clone())
        } else {
            None
        }
    }
}

fn resolve_sym(ctx: &InterpContext, sym: crate::v1_interpreter::Symbol) -> String {
    ctx.resolve(sym)
}

pub fn value_to_wire_json(val: &Value, ctx: &InterpContext) -> WireResult<serde_json::Value> {
    match val {
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => serialize_variant_to_wire_json(
            &resolve_sym(ctx, *type_name),
            &resolve_sym(ctx, *variant_name),
            fields,
            ctx,
        ),
        Value::Null => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(n) => Ok(serde_json::json!(*n)),
        Value::Float(f) => Ok(serde_json::json!(*f)),
        Value::Str(s) => {
            if s.starts_with('[') || s.starts_with('{') {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(s) {
                    return Ok(parsed);
                }
            }
            Ok(serde_json::Value::String(s.clone()))
        }
        Value::List(items) => {
            let mut arr = Vec::with_capacity(items.len());
            for item in items.iter() {
                arr.push(value_to_wire_json(item, ctx)?);
            }
            Ok(serde_json::Value::Array(arr))
        }
        Value::Set(members) => Ok(serde_json::Value::Array(
            members
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        )),
        Value::Map(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m.iter() {
                let key = match k.value_ref() {
                    Value::Str(s) => s.clone(),
                    other => {
                        return Err(format!(
                            "cannot serialize map with non-string key to JSON (got {other:?} key)"
                        ))
                    }
                };
                obj.insert(key, value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        Value::Record { fields, .. } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                if matches!(v, Value::Null) {
                    continue;
                }
                obj.insert(resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        Value::Unit => Ok(serde_json::Value::Null),
        Value::Closure { .. } => Ok(serde_json::Value::String("<closure>".to_string())),
        Value::Fn { node } => Ok(serde_json::Value::String(format!("<fn {}>", node.name))),
    }
}

fn serialize_variant_to_wire_json(
    type_name: &str,
    variant_name: &str,
    fields: &[(crate::v1_interpreter::Symbol, Value)],
    ctx: &InterpContext,
) -> WireResult<serde_json::Value> {
    let policy = resolve_coproduct_wire_policy(
        type_name,
        ctx.modules.iter().as_ref(),
        ctx.source_indices.as_ref(),
    )
    .unwrap_or_else(|| rust_tagged_object_policy());

    if policy.error_message.is_some() {
        return Err(policy
            .error_message
            .clone()
            .unwrap_or_else(|| format!("wire policy error for coproduct {type_name}")));
    }

    if policy_is_untagged(policy.clone()) {
        return serialize_untagged_variant(fields, ctx);
    }

    if policy_is_string_variant(policy.clone()) {
        let tag = wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .ok_or_else(|| format!("no wire tag for string variant {type_name}::{variant_name}"))?;
        return Ok(serde_json::Value::String(tag));
    }

    if let Some(tag_field) = policy_serde_tag_field(policy.clone()) {
        let wire_tag = wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .ok_or_else(|| {
                format!("no wire tag for internally-tagged variant {type_name}::{variant_name}")
            })?;
        let mut obj = serde_json::Map::new();
        obj.insert(tag_field, serde_json::Value::String(wire_tag));
        for (k, v) in fields.iter() {
            if matches!(v, Value::Null) {
                continue;
            }
            obj.insert(resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
        }
        return Ok(serde_json::Value::Object(obj));
    }

    let tag_key = policy_serde_tag_field(policy.clone()).unwrap_or_else(|| "_variant".to_string());
    let default_tag = if policy.enum_attr == rust_serde_tag_attr() {
        variant_name.to_string()
    } else {
        wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .unwrap_or_else(|| variant_name.to_string())
    };
    let mut obj = serde_json::Map::new();
    obj.insert(tag_key, serde_json::Value::String(default_tag));
    for (k, v) in fields.iter() {
        if matches!(v, Value::Null) {
            continue;
        }
        obj.insert(resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
    }
    Ok(serde_json::Value::Object(obj))
}

fn serialize_untagged_variant(
    fields: &[(crate::v1_interpreter::Symbol, Value)],
    ctx: &InterpContext,
) -> WireResult<serde_json::Value> {
    let mut values: Vec<serde_json::Value> = fields
        .iter()
        .map(|(_, v)| v)
        .filter(|v| !matches!(v, Value::Null))
        .map(|v| value_to_wire_json(v, ctx))
        .collect::<Result<Vec<_>, _>>()?;
    match values.len() {
        0 => Ok(serde_json::Value::Null),
        1 => Ok(values.remove(0)),
        _ => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                if matches!(v, Value::Null) {
                    continue;
                }
                obj.insert(resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
    }
}
