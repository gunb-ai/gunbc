use crate::dag::{
    ArrowBody, AtomPayload, Behavior, BranchPattern, CardinalityBound, Dag, DeclarationId,
    FieldValue, LiteralBits, NodeId, PortState, TransformTarget, TypeConnective, ValueBody,
};
use crate::diagnostics::{Diagnostic, SourceSpan};

#[derive(Debug, Clone)]
pub struct DagDifference {
    pub detail: String,
}

pub fn serialize_dag(dag: &Dag) -> Vec<u8> {
    let mut out = String::new();
    for (index, _) in dag.declarations().iter().enumerate() {
        out.push_str(&serialize_declaration(dag, index));
    }
    for node in dag.nodes() {
        out.push_str(&serialize_behavior(node));
    }
    for port in dag.ports() {
        out.push_str(&format!(
            "PORT {} state={} produced_by={}\n",
            port.id().raw(),
            render_port_state(port.state()),
            render_node_opt(port.produced_by)
        ));
    }
    let mut diagnostics: Vec<_> = dag.diagnostics().iter().collect();
    diagnostics.sort_by_key(|(port, _)| port.raw());
    for (port, diagnostic) in diagnostics {
        out.push_str(&format!(
            "DIAG port={} {}\n",
            port.raw(),
            render_diagnostic(diagnostic)
        ));
    }
    out.into_bytes()
}

pub fn first_difference(lhs: &Dag, rhs: &Dag) -> Option<DagDifference> {
    let lhs_decls = lhs.declarations();
    let rhs_decls = rhs.declarations();
    if lhs_decls.len() != rhs_decls.len() {
        return Some(DagDifference {
            detail: format!(
                "declaration count mismatch: pass1={}, pass2={}",
                lhs_decls.len(),
                rhs_decls.len()
            ),
        });
    }
    for idx in 0..lhs_decls.len() {
        let left = serialize_declaration(lhs, idx);
        let right = serialize_declaration(rhs, idx);
        if left != right {
            let name = lhs_decls[idx]
                .name
                .as_deref()
                .or(rhs_decls[idx].name.as_deref())
                .unwrap_or("<anonymous>");
            return Some(DagDifference {
                detail: format!(
                    "declaration {} `{}` diverged: pass1=`{}`, pass2=`{}`",
                    lhs_decls[idx].id.raw(),
                    name,
                    left.trim_end(),
                    right.trim_end()
                ),
            });
        }
    }

    let lhs_nodes = lhs.nodes();
    let rhs_nodes = rhs.nodes();
    if lhs_nodes.len() != rhs_nodes.len() {
        return Some(DagDifference {
            detail: format!(
                "behavior count mismatch: pass1={}, pass2={}",
                lhs_nodes.len(),
                rhs_nodes.len()
            ),
        });
    }
    for idx in 0..lhs_nodes.len() {
        let left = serialize_behavior(&lhs_nodes[idx]);
        let right = serialize_behavior(&rhs_nodes[idx]);
        if left != right {
            return Some(DagDifference {
                detail: format!(
                    "behavior {} diverged: pass1=`{}`, pass2=`{}`",
                    lhs_nodes[idx].id().raw(),
                    left.trim_end(),
                    right.trim_end()
                ),
            });
        }
    }

    let lhs_ports = lhs.ports();
    let rhs_ports = rhs.ports();
    if lhs_ports.len() != rhs_ports.len() {
        return Some(DagDifference {
            detail: format!(
                "port count mismatch: pass1={}, pass2={}",
                lhs_ports.len(),
                rhs_ports.len()
            ),
        });
    }
    for (left_port, right_port) in lhs_ports.iter().zip(rhs_ports.iter()) {
        let left = format!(
            "PORT {} state={} produced_by={}",
            left_port.id().raw(),
            render_port_state(left_port.state()),
            render_node_opt(left_port.produced_by)
        );
        let right = format!(
            "PORT {} state={} produced_by={}",
            right_port.id().raw(),
            render_port_state(right_port.state()),
            render_node_opt(right_port.produced_by)
        );
        if left != right {
            return Some(DagDifference {
                detail: format!("port diverged: pass1=`{left}`, pass2=`{right}`"),
            });
        }
    }

    let mut lhs_diags: Vec<_> = lhs.diagnostics().iter().collect();
    let mut rhs_diags: Vec<_> = rhs.diagnostics().iter().collect();
    lhs_diags.sort_by_key(|(port, _)| port.raw());
    rhs_diags.sort_by_key(|(port, _)| port.raw());
    if lhs_diags.len() != rhs_diags.len() {
        return Some(DagDifference {
            detail: format!(
                "diagnostic count mismatch: pass1={}, pass2={}",
                lhs_diags.len(),
                rhs_diags.len()
            ),
        });
    }
    for ((left_port, left_diag), (right_port, right_diag)) in lhs_diags.iter().zip(rhs_diags.iter())
    {
        let left = format!(
            "DIAG port={} {}",
            left_port.raw(),
            render_diagnostic(left_diag)
        );
        let right = format!(
            "DIAG port={} {}",
            right_port.raw(),
            render_diagnostic(right_diag)
        );
        if left != right {
            return Some(DagDifference {
                detail: format!("diagnostic diverged: pass1=`{left}`, pass2=`{right}`"),
            });
        }
    }

    None
}

fn serialize_declaration(dag: &Dag, index: usize) -> String {
    let decl = &dag.declarations()[index];
    format!(
        "DECL {} name={} connective={} type_params=[{}] meta_tag={} inhabits={} value_body={} span={}\n",
        decl.id.raw(),
        decl.name.as_deref().unwrap_or("<anonymous>"),
        render_connective(&decl.connective),
        decl.type_params
            .iter()
            .map(|id| id.raw().to_string())
            .collect::<Vec<_>>()
            .join(","),
        render_decl_opt(decl.meta_tag),
        render_decl_opt(decl.inhabits),
        render_value_body(decl.value_body.as_ref()),
        render_span(&decl.span)
    )
}

fn serialize_behavior(node: &Behavior) -> String {
    match node {
        Behavior::Value(value) => format!(
            "BEHAV {} Value data={} output={} span={}\n",
            value.id.raw(),
            render_literal(&value.data),
            value.output.raw(),
            render_span(&value.span)
        ),
        Behavior::Transform(transform) => format!(
            "BEHAV {} Transform target={} inputs=[{}] output={} span={}\n",
            transform.id.raw(),
            render_transform_target(&transform.target),
            transform
                .inputs
                .iter()
                .map(|id| id.raw().to_string())
                .collect::<Vec<_>>()
                .join(","),
            transform.output.raw(),
            render_span(&transform.span)
        ),
        Behavior::Branch(branch) => format!(
            "BEHAV {} Branch input={} paths=[{}] output={} span={}\n",
            branch.id.raw(),
            branch.input.raw(),
            branch
                .paths
                .iter()
                .map(|path| format!(
                    "{{body={},output={},pattern={},binding={}}}",
                    path.body.raw(),
                    path.output.raw(),
                    render_branch_pattern(&path.pattern),
                    path.binding
                        .as_ref()
                        .map(|binding| format!(
                            "{}:{}",
                            binding.binding_name,
                            binding.payload_port.raw()
                        ))
                        .unwrap_or_else(|| "none".to_string())
                ))
                .collect::<Vec<_>>()
                .join(","),
            branch.output.raw(),
            render_span(&branch.span)
        ),
        Behavior::Loop(loop_node) => format!(
            "BEHAV {} Loop source={} init={} body={} bound={} output={} span={}\n",
            loop_node.id.raw(),
            loop_node.source.raw(),
            loop_node.init.raw(),
            loop_node.body.raw(),
            loop_node.bound.count.raw(),
            loop_node.output.raw(),
            render_span(&loop_node.span)
        ),
        Behavior::Bind(bind) => format!(
            "BEHAV {} Bind name={} value={} params=[{}] span={}\n",
            bind.id.raw(),
            bind.name,
            bind.value.raw(),
            bind.params
                .iter()
                .map(|id| id.raw().to_string())
                .collect::<Vec<_>>()
                .join(","),
            render_span(&bind.span)
        ),
    }
}

fn render_connective(connective: &TypeConnective) -> String {
    match connective {
        TypeConnective::Atom(atom) => format!("Atom({})", render_atom(atom)),
        TypeConnective::Conj { children } => format!(
            "Conj({})",
            children
                .iter()
                .map(|field| format!("{}:{}", field.label, field.ty.raw()))
                .collect::<Vec<_>>()
                .join(",")
        ),
        TypeConnective::Disj { variants } => format!(
            "Disj({})",
            variants
                .iter()
                .map(|field| format!("{}:{}", field.label, field.ty.raw()))
                .collect::<Vec<_>>()
                .join(",")
        ),
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => format!(
            "Arrow(inputs=[{}],output={},body={})",
            inputs
                .iter()
                .map(|id| id.raw().to_string())
                .collect::<Vec<_>>()
                .join(","),
            output.raw(),
            render_arrow_body(body)
        ),
        TypeConnective::Cardinality { element, bound } => {
            format!(
                "Cardinality(element={},bound={})",
                element.raw(),
                render_bound(bound)
            )
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => format!(
            "Instantiation(template={},arguments=[{}])",
            template.raw(),
            arguments
                .iter()
                .map(|arg| format!("{}->{}", arg.parameter.raw(), arg.value.raw()))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn render_atom(atom: &AtomPayload) -> String {
    match atom {
        AtomPayload::Literal(bits) => format!("Literal({})", render_literal(bits)),
        AtomPayload::UnresolvedIdentifier(name) => format!("UnresolvedIdentifier({name})"),
        AtomPayload::ResolvedIdentifier(id) => format!("ResolvedIdentifier({})", id.raw()),
        AtomPayload::TypeParam(name) => format!("TypeParam({name})"),
    }
}

fn render_literal(literal: &LiteralBits) -> String {
    match literal {
        LiteralBits::Int(value) => format!("Int({value})"),
        LiteralBits::Bool(value) => format!("Bool({value})"),
        LiteralBits::String(value) => format!("String({value:?})"),
    }
}

fn render_arrow_body(body: &ArrowBody) -> String {
    match body {
        ArrowBody::UserDefined(id) => format!("UserDefined({})", id.raw()),
        ArrowBody::ExternalRealization(id) => format!("ExternalRealization({})", id.raw()),
        ArrowBody::Pending => "Pending".to_string(),
        ArrowBody::Unparsed(span) => format!("Unparsed({})", render_span(span)),
    }
}

fn render_bound(bound: &CardinalityBound) -> String {
    match bound {
        CardinalityBound::Exact(value) => format!("Exact({value})"),
        CardinalityBound::AtMostOne => "AtMostOne".to_string(),
        CardinalityBound::Unbounded => "Unbounded".to_string(),
    }
}

fn render_value_body(body: Option<&ValueBody>) -> String {
    match body {
        None => "None".to_string(),
        Some(ValueBody::Unparsed(span)) => format!("Unparsed({})", render_span(span)),
        Some(ValueBody::Structural { fields }) => format!(
            "Structural({})",
            fields
                .iter()
                .map(|(label, value)| format!("{label}:{}", render_field_value(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn render_field_value(value: &FieldValue) -> String {
    match value {
        FieldValue::Literal(bits) => format!("Literal({})", render_literal(bits)),
        FieldValue::Reference(id) => format!("Reference({})", id.raw()),
        FieldValue::Record(fields) => format!(
            "Record({})",
            fields
                .iter()
                .map(|(label, value)| format!("{label}:{}", render_field_value(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        FieldValue::List(values) => format!(
            "List([{}])",
            values
                .iter()
                .map(render_field_value)
                .collect::<Vec<_>>()
                .join(",")
        ),
        FieldValue::Variant {
            constructor,
            payload,
        } => format!(
            "Variant(constructor={},payload=[{}])",
            constructor.raw(),
            payload
                .iter()
                .map(render_field_value)
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn render_port_state(state: &PortState) -> String {
    match state {
        PortState::Uninferred => "Uninferred".to_string(),
        PortState::Resolved(shape) => format!("Resolved({})", shape.declaration.raw()),
        PortState::Unresolved => "Unresolved".to_string(),
    }
}

fn render_transform_target(target: &TransformTarget) -> String {
    match target {
        TransformTarget::Callable(id) => format!("Callable({})", id.raw()),
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => format!(
            "FieldProject(label={},child={})",
            field_label,
            render_decl_opt(*field_child)
        ),
        TransformTarget::Operator(kind) => format!("Operator({kind:?})"),
    }
}

fn render_branch_pattern(pattern: &BranchPattern) -> String {
    match pattern {
        BranchPattern::UnresolvedVariant { name, span } => {
            format!("UnresolvedVariant(name={name},span={})", render_span(span))
        }
        BranchPattern::ResolvedVariant(id) => format!("ResolvedVariant({})", id.raw()),
    }
}

fn render_diagnostic(diagnostic: &Diagnostic) -> String {
    match diagnostic {
        Diagnostic::TokenizerError { message, span } => {
            format!(
                "TokenizerError(message={message:?},span={})",
                render_span(span)
            )
        }
        Diagnostic::ParseError { message, span } => {
            format!("ParseError(message={message:?},span={})", render_span(span))
        }
        Diagnostic::TypeMismatch {
            expected,
            actual,
            span,
        } => format!(
            "TypeMismatch(expected={},actual={},span={})",
            expected.declaration.raw(),
            actual.declaration.raw(),
            render_span(span)
        ),
        Diagnostic::ArityMismatch {
            function,
            expected,
            actual,
            span,
        } => format!(
            "ArityMismatch(function={function:?},expected={expected},actual={actual},span={})",
            render_span(span)
        ),
        Diagnostic::ResolveError { name, span } => {
            format!("ResolveError(name={name:?},span={})", render_span(span))
        }
    }
}

fn render_span(span: &SourceSpan) -> String {
    format!("{}:{}..{}", span.file, span.byte_start, span.byte_end)
}

fn render_decl_opt(id: Option<DeclarationId>) -> String {
    id.map(|id| id.raw().to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn render_node_opt(id: Option<NodeId>) -> String {
    id.map(|id| id.raw().to_string())
        .unwrap_or_else(|| "none".to_string())
}
