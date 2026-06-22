use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::v1_std_core::{
    authored_name_at, find_property, find_property_string, is_rest_transport, transport_method_key,
    transport_path_template_key, ExprData, NewlineIndex, Node,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredRestTransportOp {
    pub service: String,
    pub name: String,
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestTransportFactError {
    MissingServiceScope { operation: String },
    MissingMethodProperty { service: String, operation: String },
    MissingPathProperty { service: String, operation: String },
}

impl fmt::Display for RestTransportFactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestTransportFactError::MissingServiceScope { operation } => {
                write!(
                    f,
                    "REST transport without enclosing service scope (operation={operation})"
                )
            }
            RestTransportFactError::MissingMethodProperty { service, operation } => {
                write!(
                    f,
                    "missing method on rest transport for {service}::{operation}"
                )
            }
            RestTransportFactError::MissingPathProperty { service, operation } => {
                write!(
                    f,
                    "missing path on rest transport for {service}::{operation}"
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestTransportCollectResult {
    pub ops: Vec<DeclaredRestTransportOp>,
    pub errors: Vec<RestTransportFactError>,
}

fn transport_field_string(
    props: Rc<Vec<Rc<Node>>>,
    prop_name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    find_property_string(props.clone(), prop_name.clone(), source_indices.clone()).or_else(|| {
        let n = find_property(props, prop_name, source_indices.clone())?;
        match (*n.expr_data).clone() {
            ExprData::ExprVar { .. } => {
                let s = authored_name_at(source_indices, n);
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

pub fn collect_rest_transport_operations(
    module: &Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> RestTransportCollectResult {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    fn walk(
        n: &Rc<Node>,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        service_ctx: Option<String>,
        out: &mut Vec<DeclaredRestTransportOp>,
        errors: &mut Vec<RestTransportFactError>,
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
                let Some(svc) = service_ctx.clone() else {
                    errors.push(RestTransportFactError::MissingServiceScope {
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                let method = transport_field_string(
                    t.properties.clone(),
                    transport_method_key(),
                    source_indices.clone(),
                );
                let Some(method) = method else {
                    errors.push(RestTransportFactError::MissingMethodProperty {
                        service: svc.clone(),
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                let path = transport_field_string(
                    t.properties.clone(),
                    transport_path_template_key(),
                    source_indices.clone(),
                );
                let Some(path) = path else {
                    errors.push(RestTransportFactError::MissingPathProperty {
                        service: svc.clone(),
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                out.push(DeclaredRestTransportOp {
                    service: svc,
                    name: n.name.clone(),
                    method,
                    path,
                });
            }
        }

        for c in n.children.iter() {
            walk(
                c,
                source_indices.clone(),
                ctx_for_children.clone(),
                out,
                errors,
            );
        }
    }
    walk(module, source_indices, None, &mut out, &mut errors);
    RestTransportCollectResult { ops: out, errors }
}
