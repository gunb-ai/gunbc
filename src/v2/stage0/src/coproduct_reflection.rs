//! Coproduct-arm reflection — Phase 2a compile-time key enumeration (R-reflect).
//!
//! Bootstrap seam: v2 interpreter intercept over the resolved compile graph (Disj walk).
//! Path 3 (syntactic): raw source-text scan of `|`-separated arm labels — independent of the
//! Disj resolver (design-coproduct-arm-reflection.md §5.2).
//!
//! Dissolve-on-arrival: substrate compiler builtin once type-name → resolved Disj Node is
//! expressible in v4 `.dag` (analogous to node_labeled_child_edges for value-level queries).

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
    let items: Vec<Value> = labels
        .iter()
        .map(|label| Value::Str(label.clone()))
        .collect();
    crate::v2_interpreter::list_value(items)
}

/// Bootstrap only: resolve declaring source path via cwd walk + optional GUNBC_ROOT.
/// Dissolve-on-arrival: compile-graph span.file is the sole authority once claim-run
/// transport no longer needs off-disk reads for Path-3 witnesses.
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
    let start = find_type_decl_start(source, type_name)?;
    let needle = format!("type {type_name}");
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

/// Word-boundary match: `type Connective` must not match `type ConnectiveCoproductVariant`.
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

pub fn eval_coproduct_arm_keys(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(args.first().map(|(_, v)| v), "coproduct_arm_keys")?;
    let (item, _) = type_item_by_name(ctx, type_name)?;
    let labels = disj_variant_labels(item, ctx)?;
    Ok(symbol_list_value(&labels))
}

pub fn eval_syntactic_coproduct_arm_keys(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> InterpResult<Value> {
    let type_name = expect_symbol(
        args.first().map(|(_, v)| v),
        "syntactic_coproduct_arm_keys",
    )?;
    let (_, file) = type_item_by_name(ctx, type_name)?;
    let labels = syntactic_coproduct_arm_labels(&file, type_name)?;
    Ok(symbol_list_value(&labels))
}

/// Mechanism-drift probe: drop the last Disj child and compare key sets (test-only).
pub fn eval_coproduct_arm_keys_with_dropped_last_arm(
    ctx: &InterpContext,
    type_name: &str,
) -> InterpResult<Value> {
    let (item, _) = type_item_by_name(ctx, type_name)?;
    let mut labels = disj_variant_labels(item, ctx)?;
    labels.pop();
    Ok(symbol_list_value(&labels))
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
        let source =
            "type ConnectiveCoproductVariant = Foo | Bar\ntype Connective = Atom | Conj\n";
        assert_eq!(
            extract_type_sum_arm_labels(source, "ConnectiveCoproductVariant").expect("variant"),
            vec!["Foo", "Bar"]
        );
        assert_eq!(
            extract_type_sum_arm_labels(source, "Connective").expect("Connective"),
            vec!["Atom", "Conj"]
        );
    }
}
