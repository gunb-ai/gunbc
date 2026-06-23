//! R-reflect Phase 2a — minimal dissolving bridges (substrate-native target).
//!
//! - `resolve_type_node`: compiler Disj type item → substrate `Node` Value (dissolves when v4
//!   gains compile-graph access).
//! - `syntactic_coproduct_arm_keys`: Path-3 raw-source text scan over in-memory compile source.

use std::collections::HashMap;
use std::rc::Rc;

use crate::v2_compiler_infer_items::ItemKind;
use crate::v2_interpreter::{InterpContext, InterpError, InterpResult, Value};
use crate::v2_std_core::{authored_name_at, Connective, Node};

fn expect_symbol<'a>(value: Option<&'a Value>, what: &str) -> InterpResult<&'a str> {
    match value {
        Some(Value::Str(s)) => Ok(s.as_str()),
        _ => Err(InterpError::TypeError {
            msg: format!("{what} requires a Symbol argument"),
        }),
    }
}

pub(crate) fn type_item_by_name<'a>(
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
        msg: format!("resolve_type_node: unknown closed type `{type_name}`"),
    })
}

fn nullary_connective_variant(ctx: &InterpContext, name: &str) -> Value {
    Value::Variant {
        type_name: ctx.sym("Connective"),
        variant_name: ctx.sym(name),
        fields: Rc::new(HashMap::new()),
    }
}

fn node_kind_type_node(ctx: &InterpContext, connective: Value) -> Value {
    Value::Variant {
        type_name: ctx.sym("NodeKind"),
        variant_name: ctx.sym("TypeNode"),
        fields: Rc::new(HashMap::from([(ctx.sym("connective"), connective)])),
    }
}

fn synthetic_occurrence(ctx: &InterpContext) -> Value {
    Value::Variant {
        type_name: ctx.sym("NodeOccurrenceId"),
        variant_name: ctx.sym("SyntheticOccurrence"),
        fields: Rc::new(HashMap::new()),
    }
}

fn node_record(ctx: &InterpContext, kind: Value, children: Vec<Value>) -> Value {
    Value::Record {
        type_name: ctx.sym("Node"),
        fields: Rc::new(HashMap::from([
            (ctx.sym("kind"), kind),
            (
                ctx.sym("children"),
                crate::v2_interpreter::list_value(children),
            ),
            (ctx.sym("occurrence_id"), synthetic_occurrence(ctx)),
        ])),
    }
}

fn edge_named(ctx: &InterpContext, name: &str, target: Value) -> Value {
    Value::Record {
        type_name: ctx.sym("Edge"),
        fields: Rc::new(HashMap::from([
            (
                ctx.sym("label"),
                Value::Variant {
                    type_name: ctx.sym("EdgeLabel"),
                    variant_name: ctx.sym("Named"),
                    fields: Rc::new(HashMap::from([(
                        ctx.sym("name"),
                        Value::Str(name.to_string()),
                    )])),
                },
            ),
            (ctx.sym("target"), target),
        ])),
    }
}

// 🟡 gated — feature:coproduct-reflection-bridge — bind gunbc#4863 — dissolve-on-2b: INERT placeholder
// target, valid ONLY while coproduct_arms is deferred and targets are NOT read (keys path reads labels
// only). When coproduct_arms is exposed in Phase-2b and arm TARGETS are read, the marshaler MUST emit
// the REAL arm target; a surviving placeholder here would be a latent B3 fabrication (type-incorrect
// payload). forbidden: exposing coproduct_arms while this placeholder is still the target.
fn placeholder_arm_target(ctx: &InterpContext) -> Value {
    node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Conj")),
        vec![],
    )
}

/// Marshal a resolved closed-coproduct (Disj) type item to substrate `Node` with Named arm edges.
pub fn marshal_disj_type_item(ctx: &InterpContext, item: &Rc<Node>) -> InterpResult<Value> {
    if item.connective != Connective::Disj {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: type is not a closed coproduct (Disj)".to_string(),
        });
    }
    let si = ctx.source_indices();
    let placeholder = placeholder_arm_target(ctx);
    let mut edges = Vec::with_capacity(item.children.len());
    for child in item.children.iter() {
        let label = authored_name_at(si.clone(), child.clone());
        edges.push(edge_named(ctx, &label, placeholder.clone()));
    }
    Ok(node_record(
        ctx,
        node_kind_type_node(ctx, nullary_connective_variant(ctx, "Disj")),
        edges,
    ))
}

fn source_text_from_ctx(ctx: &InterpContext, file: &str) -> InterpResult<String> {
    let index = ctx.source_indices().get(file).ok_or_else(|| InterpError::TypeError {
        msg: format!("syntactic coproduct arm keys: no in-memory source for `{file}`"),
    })?;
    let len = index.char_codes.len() as i64;
    Ok(crate::v2_rt::chars_to_string(&index.char_codes, 0, len))
}

/// Path 3: grammar-level arm labels from `type Name = Arm | Arm | ...` in source text.
pub fn syntactic_coproduct_arm_labels(
    ctx: &InterpContext,
    file: &str,
    type_name: &str,
) -> InterpResult<Vec<String>> {
    let content = source_text_from_ctx(ctx, file)?;
    extract_type_sum_arm_labels(&content, type_name).map_err(|msg| InterpError::TypeError { msg })
}

fn extract_type_sum_arm_labels(source: &str, type_name: &str) -> Result<Vec<String>, String> {
    let start = find_type_decl_start(source, type_name).ok_or_else(|| {
        format!("syntactic coproduct arm keys: `{type_name}` not found in source")
    })?;
    let needle = format!("type {type_name}");
    let after_type = &source[start + needle.len()..];
    let eq_rel = after_type
        .find('=')
        .ok_or_else(|| format!("syntactic coproduct arm keys: `{type_name}` missing `=`"))?;
    let mut rest = after_type[eq_rel + 1..].trim_start();
    let mut arms = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with("//") {
            return Err(format!(
                "syntactic coproduct arm keys: unexpected `//` comment mid-declaration for `{type_name}`"
            ));
        }
        if rest.starts_with('|') {
            rest = rest[1..].trim_start();
        }
        let arm_name = read_identifier_prefix(rest).ok_or_else(|| {
            format!("syntactic coproduct arm keys: expected arm identifier for `{type_name}`")
        })?;
        arms.push(arm_name.0);
        rest = arm_name.1.trim_start();
        if rest.starts_with('{') {
            rest = skip_braced(rest).ok_or_else(|| {
                format!("syntactic coproduct arm keys: unclosed `{{` in `{type_name}` arm payload")
            })?;
        }
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if !rest.starts_with('|') {
            let peek = rest.chars().next().unwrap_or('\0');
            return Err(format!(
                "syntactic coproduct arm keys: unexpected token `{peek}` mid-declaration for `{type_name}`"
            ));
        }
    }
    if arms.is_empty() {
        return Err(format!(
            "syntactic coproduct arm keys: `{type_name}` has no arms"
        ));
    }
    Ok(arms)
}

fn find_type_decl_start(source: &str, type_name: &str) -> Option<usize> {
    let needle = format!("type {type_name}");
    let mut search_from = 0usize;
    while let Some(rel) = source[search_from..].find(&needle) {
        let start = search_from + rel;
        let after = start + needle.len();
        let boundary_ok = source[after..]
            .chars()
            .next()
            .is_none_or(|c| c.is_whitespace() || c == '=' || c == '{');
        let prefix_ok = start == 0
            || source
                .as_bytes()
                .get(start.saturating_sub(1))
                .is_some_and(|b| b.is_ascii_whitespace());
        if boundary_ok && prefix_ok {
            return Some(start);
        }
        search_from = start + 1;
    }
    None
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

pub fn eval_resolve_type_node(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "resolve_type_node")?;
    let (item, _) = type_item_by_name(ctx, type_name)?;
    marshal_disj_type_item(ctx, item)
}

pub fn eval_syntactic_coproduct_arm_keys(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "syntactic_coproduct_arm_keys")?;
    let (_, file) = type_item_by_name(ctx, type_name)?;
    let labels = syntactic_coproduct_arm_labels(ctx, &file, type_name)?;
    let items: Vec<Value> = labels
        .iter()
        .map(|label| Value::Str(label.clone()))
        .collect();
    Ok(crate::v2_interpreter::list_value(items))
}

/// Extract Named arm labels from a marshaled substrate Node Value (test / drift probe).
pub fn arm_labels_from_marshaled_node(
    ctx: &InterpContext,
    node: &Value,
) -> InterpResult<Vec<String>> {
    let Value::Record { fields, .. } = node else {
        return Err(InterpError::TypeError {
            msg: "expected Node record".to_string(),
        });
    };
    let children = fields
        .get(&ctx.sym("children"))
        .ok_or_else(|| InterpError::TypeError {
            msg: "Node missing children".to_string(),
        })?;
    let edges = crate::v2_interpreter::free_monoid_to_vec(children).ok_or_else(|| {
        InterpError::TypeError {
            msg: "children not a list".to_string(),
        }
    })?;
    let mut labels = Vec::with_capacity(edges.len());
    for edge in edges {
        let Value::Record { fields: ef, .. } = edge else {
            return Err(InterpError::TypeError {
                msg: "expected Edge record".to_string(),
            });
        };
        let label = ef
            .get(&ctx.sym("label"))
            .ok_or_else(|| InterpError::TypeError {
                msg: "Edge missing label".to_string(),
            })?;
        let Value::Variant {
            variant_name,
            fields: lf,
            ..
        } = label
        else {
            continue;
        };
        if ctx.resolve(*variant_name) != "Named" {
            continue;
        }
        let Some(Value::Str(name)) = lf.get(&ctx.sym("name")) else {
            continue;
        };
        labels.push(name.clone());
    }
    Ok(labels)
}

/// Mechanism-drift probe: drop the last Named arm edge from a marshaled Disj node.
pub fn eval_resolve_type_node_with_dropped_last_arm(
    ctx: &InterpContext,
    type_name: &str,
) -> InterpResult<Value> {
    let (item, _) = type_item_by_name(ctx, type_name)?;
    let node = marshal_disj_type_item(ctx, item)?;
    let Value::Record { fields, .. } = node else {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: expected Node record".to_string(),
        });
    };
    let children = fields
        .get(&ctx.sym("children"))
        .ok_or_else(|| InterpError::TypeError {
            msg: "resolve_type_node: Node missing children".to_string(),
        })?;
    let Some(items) = crate::v2_interpreter::free_monoid_to_vec(children) else {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: children not a list".to_string(),
        });
    };
    if items.is_empty() {
        return Err(InterpError::TypeError {
            msg: "resolve_type_node: Disj has no arms".to_string(),
        });
    }
    let mut trimmed = items[..items.len() - 1].to_vec();
    if trimmed.is_empty() {
        trimmed = vec![];
    }
    Ok(Value::Record {
        type_name: ctx.sym("Node"),
        fields: Rc::new(HashMap::from([
            (
                ctx.sym("kind"),
                fields
                    .get(&ctx.sym("kind"))
                    .cloned()
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "resolve_type_node: Node missing kind".to_string(),
                    })?,
            ),
            (
                ctx.sym("children"),
                crate::v2_interpreter::list_value(trimmed),
            ),
            (
                ctx.sym("occurrence_id"),
                fields
                    .get(&ctx.sym("occurrence_id"))
                    .cloned()
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "resolve_type_node: Node missing occurrence_id".to_string(),
                    })?,
            ),
        ])),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntactic_extractor_finds_connective_arms() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../src/v4/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let arms = extract_type_sum_arm_labels(&source, "Connective").expect("Connective arms");
        assert_eq!(
            arms,
            vec![
                "Atom",
                "Conj",
                "Disj",
                "Arrow",
                "Cardinality",
                "Instantiation"
            ]
        );
    }

    #[test]
    fn syntactic_extractor_finds_behavior_arms() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../src/v4/std/node.dag");
        let source = std::fs::read_to_string(&path).expect("read node.dag");
        let arms = extract_type_sum_arm_labels(&source, "Behavior").expect("Behavior arms");
        assert_eq!(arms, vec!["Value", "Transform", "Branch", "Loop", "Bind"]);
    }

    #[test]
    fn syntactic_extractor_rejects_connective_prefix_of_longer_type_name() {
        let source = "type ConnectiveCoproductVariant = Foo | Bar\ntype Connective = Atom | Conj\n";
        assert_eq!(
            extract_type_sum_arm_labels(source, "ConnectiveCoproductVariant").expect("variant"),
            vec!["Foo", "Bar"]
        );
        assert_eq!(
            extract_type_sum_arm_labels(source, "Connective").expect("Connective"),
            vec!["Atom", "Conj"]
        );
    }

    #[test]
    fn syntactic_extractor_fails_loud_on_mid_decl_comment() {
        let source = "type Connective = Atom | Conj // trailing\n";
        let err = extract_type_sum_arm_labels(source, "Connective").unwrap_err();
        assert!(
            err.contains("//"),
            "expected mid-decl comment diagnostic, got: {err}"
        );
    }

    #[test]
    fn syntactic_extractor_fails_loud_on_unexpected_token_mid_decl() {
        let source = "type Connective = Atom , Conj\n";
        let err = extract_type_sum_arm_labels(source, "Connective").unwrap_err();
        assert!(
            err.contains("unexpected token"),
            "expected unexpected-token diagnostic, got: {err}"
        );
    }
}
