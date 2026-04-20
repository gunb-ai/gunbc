// rest_transport_facts.rs — Hand-maintained REST transport introspection.
// Survives stage0 regeneration (see scripts/check-stage0-freshness.sh exclusions).
//
// Surfaces declared `transport rest { method, path }` facts for service
// operations using the same `v2_std_core` accessors as the compiler — not a
// parallel text parser over raw `.dag` source.

use std::collections::HashMap;
use std::rc::Rc;

use crate::v2_std_core::{
    authored_name_at, find_property, find_property_string, is_rest_transport, transport_method_key,
    transport_path_template_key, ExprData, NewlineIndex, Node,
};

/// One REST operation under a service scope (`service.operation` in `.dag`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredRestTransportOp {
    pub service: String,
    pub name: String,
    pub method: String,
    pub path: String,
}

/// String literal or keyword (`GET`, `POST`, …) for a transport property.
fn transport_field_string(
    props: Rc<Vec<Rc<Node>>>,
    prop_name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    find_property_string(props.clone(), prop_name.clone(), source_indices.clone()).or_else(|| {
        let n = find_property(props, prop_name, source_indices.clone())?;
        match (*n.expr_data).clone() {
            ExprData::ExprVar { .. } => {
                let s = authored_name_at(source_indices, &n);
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            }
            _ => None,
        }
    })
}

/// Collect every `transport rest` operation in a parsed module tree, with
/// enclosing service name (e.g. `github.Pulls`, `oauth2.Google`).
pub fn collect_rest_transport_operations(
    module: &Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Vec<DeclaredRestTransportOp> {
    let mut out = Vec::new();
    fn walk(
        n: &Rc<Node>,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        service_ctx: Option<String>,
        out: &mut Vec<DeclaredRestTransportOp>,
    ) {
        let ctx_for_children = match &n.transport {
            Some(t)
                if !is_rest_transport(t.clone(), source_indices.clone()) && !n.name.is_empty() =>
            {
                Some(n.name.clone())
            }
            _ => service_ctx.clone(),
        };

        if let Some(t) = &n.transport {
            if is_rest_transport(t.clone(), source_indices.clone()) {
                let svc = service_ctx
                    .clone()
                    .expect("REST operation without enclosing service scope");
                let method = transport_field_string(
                    t.properties.clone(),
                    transport_method_key(),
                    source_indices.clone(),
                )
                .unwrap_or_else(|| {
                    panic!(
                        "missing method value on rest transport for {}::{}",
                        svc, n.name
                    )
                });
                let path = transport_field_string(
                    t.properties.clone(),
                    transport_path_template_key(),
                    source_indices.clone(),
                )
                .unwrap_or_else(|| {
                    panic!(
                        "missing path value on rest transport for {}::{}",
                        svc, n.name
                    )
                });
                out.push(DeclaredRestTransportOp {
                    service: svc,
                    name: n.name.clone(),
                    method,
                    path,
                });
            }
        }

        for c in n.children.iter() {
            walk(c, source_indices.clone(), ctx_for_children.clone(), out);
        }
    }
    walk(module, source_indices, None, &mut out);
    out
}
