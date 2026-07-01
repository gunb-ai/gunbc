use crate::v1_compiler_dag_collect_support::{
    dag_node_key_collision_error, dag_node_surface_fingerprint, is_synthetic_span, DagCollectAcc,
};
use crate::v1_compiler_infer_items::{ResolvedGraph, TypedModule};
use crate::v1_rt;
use crate::v1_std_core::ExprData::NoExprData;
use crate::v1_std_core::InferredNode::Resolved;
use crate::v1_std_core::{import_is_all, Connective, ExprData, InferredNode, MatchPattern, Node};
use std::rc::Rc;

fn is_import_slot_node(n: Rc<Node>) -> bool {
    import_is_all(n.clone())
        || ((((n.params.clone().len() as i64) == 0) && (n.ident_span.is_some()))
            && (n.body.is_none())
            && (*n.expr_data == ExprData::NoExprData))
}

fn is_module_shell_node(n: Rc<Node>) -> bool {
    n.inferred.is_none()
        && (*n.expr_data == ExprData::NoExprData)
        && (n.connective == Connective::NoConnective)
        && n.body.is_none()
        && n.transport.is_none()
        && ((n.uses.clone().len() as i64) == 0)
        && {
            let mut all_import_slots = true;
            for p in n.params.clone().iter().cloned() {
                if !is_import_slot_node(p) {
                    all_import_slots = false;
                    break;
                }
            }
            all_import_slots
        }
}

pub fn dag_node_is_resolved_identity_shell(node: Rc<Node>) -> bool {
    match (*node.expr_data).clone() {
        NoExprData => match node.inferred.clone().as_deref().cloned() {
            Some(Resolved { node: _, .. }) => {
                node.body.is_none()
                    && node.transport.is_none()
                    && node.children.is_empty()
                    && node.params.is_empty()
            }
            _ => false,
        },
        _ => false,
    }
}

pub fn dag_node_collection_anchor(mut node: Rc<Node>) -> Rc<Node> {
    loop {
        if !dag_node_is_resolved_identity_shell(node.clone()) {
            break node;
        }
        match node.inferred.clone().as_deref().cloned() {
            Some(Resolved { node: target, .. }) => {
                node = target.clone();
                continue;
            }
            _ => break node,
        }
    }
}

pub fn dag_node_key(node: Rc<Node>) -> String {
    let anchor = dag_node_collection_anchor(node);
    if is_synthetic_span(&anchor.span) {
        return v1_rt::concat(":0..0:".to_string(), dag_node_fingerprint(anchor.clone()));
    }
    v1_rt::concat(
        v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat(anchor.span.clone().file.clone(), ":".to_string()),
                    (anchor.span.clone().start.clone()).to_string(),
                ),
                "..".to_string(),
            ),
            (anchor.span.clone().end.clone()).to_string(),
        ),
        match anchor.ident.clone() {
            Some(id) => v1_rt::concat(":".to_string(), (id.clone()).to_string()),
            None => "".to_string(),
        },
    )
}

pub fn dag_node_fingerprint(node: Rc<Node>) -> String {
    dag_node_surface_fingerprint(dag_node_collection_anchor(node))
}

pub fn dag_collect_nodes_list(
    nodes: Rc<Vec<Rc<Node>>>,
    acc: Rc<DagCollectAcc>,
) -> Rc<DagCollectAcc> {
    nodes
        .iter()
        .cloned()
        .fold(acc, |a: Rc<DagCollectAcc>, n: Rc<Node>| {
            dag_collect_insert(n.clone(), a)
        })
}

pub fn dag_collect_optional_node(
    value: Option<Rc<Node>>,
    acc: Rc<DagCollectAcc>,
) -> Rc<DagCollectAcc> {
    match value {
        Some(inner) => dag_collect_insert(inner.clone(), acc),
        None => acc,
    }
}

pub fn dag_collect_inferred(
    value: Option<Rc<InferredNode>>,
    acc: Rc<DagCollectAcc>,
) -> Rc<DagCollectAcc> {
    match value.as_deref().cloned() {
        Some(Resolved { node: n, .. }) => dag_collect_insert(n.clone(), acc),
        _ => acc,
    }
}

pub fn dag_collect_match_pattern(
    pattern: Rc<MatchPattern>,
    acc: Rc<DagCollectAcc>,
) -> Rc<DagCollectAcc> {
    match (*pattern).clone() {
        MatchPattern::Bind { name: _, .. } => acc,
        MatchPattern::LitPattern { value: _, .. } => acc,
        MatchPattern::VariantPattern { field_bindings, .. } => field_bindings
            .clone()
            .iter()
            .cloned()
            .fold(acc, |a: Rc<DagCollectAcc>, fb: Rc<Node>| {
                dag_collect_insert(fb.clone(), a)
            }),
        MatchPattern::Wildcard => acc,
    }
}

pub fn dag_collect_node_tree(node: Rc<Node>, acc: Rc<DagCollectAcc>) -> Rc<DagCollectAcc> {
    let acc = dag_collect_nodes_list(node.children.clone(), acc);
    let acc = if is_module_shell_node(node.clone()) {
        acc
    } else {
        dag_collect_nodes_list(node.params.clone(), acc)
    };
    let acc = dag_collect_nodes_list(node.uses.clone(), acc);
    let acc = dag_collect_optional_node(node.body.clone(), acc);
    let acc = dag_collect_optional_node(node.transport.clone(), acc);
    let acc = dag_collect_nodes_list(node.properties.clone(), acc);
    let acc = dag_collect_optional_node(node.type_annotation.clone(), acc);
    let acc = dag_collect_inferred(node.inferred.clone(), acc);
    match node.match_pattern.clone() {
        Some(p) => dag_collect_match_pattern(p.clone(), acc),
        None => acc,
    }
}

pub fn dag_collect_insert(node: Rc<Node>, acc: Rc<DagCollectAcc>) -> Rc<DagCollectAcc> {
    let anchor = dag_node_collection_anchor(node.clone());
    let key = dag_node_key(anchor.clone());
    let fp = dag_node_fingerprint(anchor.clone());
    let lookup = v1_rt::map_get(&*acc.seen, key.clone());
    match lookup {
        Some(prior) => {
            if prior.as_str() == fp.as_str() {
                acc
            } else if is_synthetic_span(&anchor.span) {
                Rc::new(DagCollectAcc {
                    seen: acc.seen.clone(),
                    order: acc.order.clone(),
                    collision_errors: v1_rt::rc_list_push(
                        acc.collision_errors.clone(),
                        dag_node_key_collision_error(key.clone(), anchor.span.clone()),
                    ),
                })
            } else {
                acc
            }
        }
        None => {
            let inner = match Rc::try_unwrap(acc) {
                Ok(i) => i,
                Err(rc) => (*rc).clone(),
            };
            let acc1 = Rc::new(DagCollectAcc {
                seen: v1_rt::rc_map_insert(inner.seen, key.clone(), fp),
                order: v1_rt::rc_list_push(inner.order, anchor.clone()),
                collision_errors: inner.collision_errors,
            });
            dag_collect_node_tree(anchor.clone(), acc1)
        }
    }
}

pub fn dag_collect_from_module(
    module: Rc<TypedModule>,
    acc: Rc<DagCollectAcc>,
) -> Rc<DagCollectAcc> {
    let acc = dag_collect_insert(module.module.clone(), acc);
    module
        .items
        .clone()
        .iter()
        .cloned()
        .fold(acc, |a: Rc<DagCollectAcc>, item: Rc<Node>| {
            dag_collect_insert(item.clone(), a)
        })
}

pub fn collect_dag_nodes(typed: Rc<ResolvedGraph>) -> Rc<DagCollectAcc> {
    typed.modules.clone().iter().cloned().fold(
        Rc::new(DagCollectAcc {
            seen: v1_rt::rc_empty_map::<String, String>(),
            order: Rc::new(vec![]),
            collision_errors: Rc::new(vec![]),
        }),
        |acc: Rc<DagCollectAcc>, m: Rc<TypedModule>| dag_collect_from_module(m.clone(), acc),
    )
}
