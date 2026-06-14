//! Coproduct-arm reflection — Phase 2a compile-time enumeration (R-reflect).
//!
//! Reflection path: resolve `type_name` to a closed `Disj` in the compiled module graph.
//! Path 3 (syntactic): scan the declaring source for `|`-separated arm labels before any
//! shared resolver (design-coproduct-arm-reflection.md §5.2).

use std::collections::HashMap;
use std::rc::Rc;

use crate::v2_compiler_infer_items::ItemKind;
use crate::v2_interpreter::{InterpContext, InterpError, InterpResult, Value};
use crate::v2_compiler_infer_types::child_type_node;
use crate::v2_std_core::{authored_name_at, Connective, Node};

fn expect_symbol<'a>(value: Option<&'a Value>, what: &str) -> InterpResult<&'a str> {
    match value {
        Some(Value::Str(s)) => Ok(s.as_str()),
        _ => Err(InterpError::TypeError {
            msg: format!("{what} requires a Symbol argument"),
        }),
    }
}

fn type_item_by_name<'a>(
    ctx: &'a InterpContext,
    type_name: &str,
) -> InterpResult<(&'a Rc<Node>, String)> {
    let si = ctx.source_indices();
    for module in ctx.modules.iter() {
        for item in module.items.iter() {
            let name = authored_name_at(si.clone(), item.clone());
            if name != type_name {
                continue;
            }
            let is_type = ctx
                .item_registry
                .get(&name)
                .map(|info| info.kind == ItemKind::TypeItem)
                .unwrap_or(false)
                || ctx
                    .item_registry
                    .get(&item.name)
                    .map(|info| info.kind == ItemKind::TypeItem)
                    .unwrap_or(false);
            if is_type {
                return Ok((item, item.span.file.clone()));
            }
        }
    }
    Err(InterpError::TypeError {
        msg: format!("coproduct reflection: unknown closed type `{type_name}`"),
    })
}

fn disj_variant_labels(item: &Rc<Node>, ctx: &InterpContext) -> InterpResult<Vec<String>> {
    if item.connective != Connective::Disj {
        return Err(InterpError::TypeError {
            msg: "coproduct reflection: type is not a closed coproduct (Disj)".to_string(),
        });
    }
    let si = ctx.source_indices();
    Ok(item
        .children
        .iter()
        .map(|child| authored_name_at(si.clone(), child.clone()))
        .collect())
}

fn symbol_list_value(labels: &[String]) -> Value {
    let items: Vec<Value> = labels.iter().map(|label| Value::Str(label.clone())).collect();
    crate::v2_interpreter::list_value(items)
}

fn variant_is_nullary(variant: &Rc<Node>, ctx: &InterpContext) -> bool {
    let payload = child_type_node(variant.clone());
    payload.connective == Connective::NoConnective && payload.children.is_empty()
}

fn resolve_source_file_path(file: &str) -> InterpResult<String> {
    let path = std::path::Path::new(file);
    if path.is_absolute() && path.exists() {
        return Ok(file.to_string());
    }
    if path.exists() {
        return Ok(file.to_string());
    }
    if let Ok(mut dir) = std::env::current_dir() {
        loop {
            let candidate = dir.join(file);
            if candidate.exists() {
                return Ok(candidate.to_string_lossy().into_owned());
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if let Ok(root) = std::env::var("GUNBC_ROOT") {
        let candidate = std::path::Path::new(&root).join(file);
        if candidate.exists() {
            return Ok(candidate.to_string_lossy().into_owned());
        }
    }
    Err(InterpError::TypeError {
        msg: format!("coproduct reflection: cannot resolve source path `{file}`"),
    })
}

/// Path 3: grammar-level arm labels from `type Name = Arm | Arm | ...` in source text.
pub fn syntactic_coproduct_arm_labels(file: &str, type_name: &str) -> InterpResult<Vec<String>> {
    let resolved = resolve_source_file_path(file)?;
    let content = std::fs::read_to_string(&resolved).map_err(|e| InterpError::TypeError {
        msg: format!("syntactic coproduct arm keys: cannot read `{resolved}`: {e}"),
    })?;
    extract_type_sum_arm_labels(&content, type_name).ok_or_else(|| InterpError::TypeError {
        msg: format!("syntactic coproduct arm keys: `{type_name}` not found in `{file}`"),
    })
}

/// Scan source for `type <name> = <arm> (| <arm>)*` without invoking the type resolver.
fn extract_type_sum_arm_labels(source: &str, type_name: &str) -> Option<Vec<String>> {
    let needle = format!("type {type_name}");
    let start = source.find(&needle)?;
    let after_type = &source[start + needle.len()..];
    let eq_rel = after_type.find('=')?;
    let mut rest = after_type[eq_rel + 1..].trim_start();
    let mut arms = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.is_empty() || rest.starts_with("//") {
            break;
        }
        if rest.starts_with('|') {
            rest = rest[1..].trim_start();
        }
        let arm_name = read_identifier_prefix(rest)?;
        arms.push(arm_name.0);
        rest = arm_name.1.trim_start();
        if rest.starts_with('{') {
            rest = skip_braced(rest)?;
        }
        rest = rest.trim_start();
        if !rest.starts_with('|') {
            break;
        }
    }
    if arms.is_empty() {
        None
    } else {
        Some(arms)
    }
}

fn read_identifier_prefix(s: &str) -> Option<(String, &str)> {
    let mut end = 0;
    for (i, ch) in s.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    Some((s[..end].to_string(), &s[end..]))
}

fn skip_braced(s: &str) -> Option<&str> {
    let mut depth = 0usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(&s[i + 1..]);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn eval_coproduct_arm_keys(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Option<Value>> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "coproduct_arm_keys")?;
    let (item, _) = type_item_by_name(ctx, type_name)?;
    let labels = disj_variant_labels(item, ctx)?;
    Ok(Some(symbol_list_value(&labels)))
}

pub fn eval_syntactic_coproduct_arm_keys(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Option<Value>> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "syntactic_coproduct_arm_keys")?;
    let (_, file) = type_item_by_name(ctx, type_name)?;
    let labels = syntactic_coproduct_arm_labels(&file, type_name)?;
    Ok(Some(symbol_list_value(&labels)))
}

pub fn eval_coproduct_arms(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Option<Value>> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "coproduct_arms")?;
    let (item, _) = type_item_by_name(ctx, type_name)?;
    let si = ctx.source_indices();
    let labels = disj_variant_labels(item, ctx)?;
    let mut arms = Vec::with_capacity(labels.len());
    for (variant, label) in item.children.iter().zip(labels.iter()) {
        let payload_shape = if variant_is_nullary(variant, ctx) {
            Value::Variant {
                type_name: ctx.sym("NodeKind"),
                variant_name: ctx.sym("ComputationNode"),
                fields: Rc::new(HashMap::from([(
                    ctx.sym("behavior"),
                    Value::Variant {
                        type_name: ctx.sym("Behavior"),
                        variant_name: ctx.sym(label),
                        fields: Rc::new(HashMap::new()),
                    },
                )])),
            }
        } else {
            let payload = child_type_node(variant.clone());
            Value::Variant {
                type_name: ctx.sym("NodeKind"),
                variant_name: ctx.sym("TypeNode"),
                fields: Rc::new(HashMap::from([(
                    ctx.sym("connective"),
                    Value::Variant {
                        type_name: ctx.sym("Connective"),
                        variant_name: ctx.sym(&authored_name_at(si.clone(), payload.clone())),
                        fields: Rc::new(HashMap::new()),
                    },
                )])),
            }
        };
        arms.push(Value::Record {
            type_name: ctx.sym("CoproductArm"),
            fields: Rc::new(HashMap::from([
                (ctx.sym("label"), Value::Str(label.clone())),
                (
                    ctx.sym("payload"),
                    Value::Record {
                        type_name: ctx.sym("NodeShape"),
                        fields: Rc::new(HashMap::from([(ctx.sym("kind"), payload_shape)])),
                    },
                ),
            ])),
        });
    }
    Ok(Some(crate::v2_interpreter::list_value(arms)))
}

pub fn eval_coproduct_nullary_inhabitants(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Option<Value>> {
    let type_name = expect_symbol(
        args.first().map(|(_, v)| v),
        "coproduct_nullary_inhabitants",
    )?;
    let (item, _) = type_item_by_name(ctx, type_name)?;
    let si = ctx.source_indices();
    for variant in item.children.iter() {
        if !variant_is_nullary(variant, ctx) {
            return Err(InterpError::TypeError {
                msg: format!(
                    "coproduct_nullary_inhabitants: arm `{}` carries a payload — fail closed",
                    authored_name_at(si.clone(), variant.clone())
                ),
            });
        }
    }
    let inhabitants: Vec<Value> = item
        .children
        .iter()
        .map(|variant| Value::Str(authored_name_at(si.clone(), variant.clone())))
        .collect();
    Ok(Some(crate::v2_interpreter::list_value(inhabitants)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntactic_extractor_finds_connective_arms() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../src/v4/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let arms = extract_type_sum_arm_labels(&source, "Connective").expect("Connective arms");
        assert_eq!(
            arms,
            vec![
                "Atom", "Conj", "Disj", "Arrow", "Cardinality", "Instantiation"
            ]
        );
    }

    #[test]
    fn syntactic_extractor_finds_behavior_arms() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../src/v4/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let arms = extract_type_sum_arm_labels(&source, "Behavior").expect("Behavior arms");
        assert_eq!(
            arms,
            vec!["Value", "Transform", "Branch", "Loop", "Bind"]
        );
    }
}
