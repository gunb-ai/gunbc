use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::process::{Command, Stdio};

use crate::dag::{
    ArithmeticOp, ArrowBody, AtomPayload, Behavior, BindEmitParticipation, BindNode,
    BranchEmitParticipation, BranchNode, BranchPattern, CardinalityBound, Cluster, ClusterId,
    ComparisonOp, Dag, Declaration, DeclarationId, Field, FieldMap, FieldValue, IntraClusterCall,
    LiteralBits, LogicalOp, LoopBound, LoopNode, MemberDescent, NodeId, NominalOpacity,
    NonEmptyList, NonSingletonList, OperatorKind, Path, PayloadBinding, PhantomParameter, PortId,
    PortState, TemplateArgument, TransformNode, TransformTarget, TypeConnective, ValueBody,
    ValueNode,
};
use crate::diagnostics::{Diagnostic, DiagnosticAttribution};

pub fn render_bootstrap_generated_rs(
    dag: &Dag,
    authority_label: &str,
    function_name: &str,
) -> Result<String, String> {
    let header = format!(
        "// AUTO-GENERATED from `{authority_label}` via `regen_bootstrap`.\n\
         // Regenerate instead of hand-editing.\n\n"
    );
    let rust = emit_bootstrap_module(dag, function_name);
    rustfmt_stdout(&format!("{header}{rust}"))
}

pub fn render_bootstrap_std_generated_rs(dag: &Dag) -> Result<String, String> {
    render_bootstrap_generated_rs(dag, "dsl/std/*.dag", "bootstrapped_std_fixture_dag")
}

fn rustfmt_stdout(combined: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn rustfmt: {e}"))?;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(combined.as_bytes())
        .map_err(|e| format!("write rustfmt stdin: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait rustfmt: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("rustfmt stdout utf-8: {e}"))
}

fn emit_bootstrap_module(dag: &Dag, function_name: &str) -> String {
    let nodes_fn = format!("{function_name}_nodes");
    let declarations_fn = format!("{function_name}_declarations");
    let ports_fn = format!("{function_name}_ports");
    let diagnostics_fn = format!("{function_name}_diagnostics");
    let clusters_fn = format!("{function_name}_clusters");
    let optional_match_disjs_fn = format!("{function_name}_optional_match_disjs");

    let mut out = String::new();
    out.push_str(&format!("pub(crate) fn {function_name}() -> Dag {{\n"));
    out.push_str("    Dag {\n");
    push_field(&mut out, "nodes", &format!("{nodes_fn}()"), 2);
    push_field(&mut out, "declarations", &format!("{declarations_fn}()"), 2);
    push_field(&mut out, "ports", &format!("{ports_fn}()"), 2);
    push_field(&mut out, "diagnostics", &format!("{diagnostics_fn}()"), 2);
    push_field(&mut out, "next_node_id", &dag.nodes().len().to_string(), 2);
    push_field(
        &mut out,
        "next_declaration_id",
        &dag.declarations().len().to_string(),
        2,
    );
    push_field(&mut out, "next_port_id", &dag.ports().len().to_string(), 2);
    out.push_str("        primitives: PrimitiveCache::default(),\n");
    out.push_str("        substrate_markers: SubstrateMarkers::default(),\n");
    out.push_str("        realization_metas: RealizationMetaCache::default(),\n");
    out.push_str("        target_syntax: TargetSyntaxCache::default(),\n");
    out.push_str("        stdlib_types: StdlibTypeCache::default(),\n");
    out.push_str("        emit_anchors: EmitAnchorCache::default(),\n");
    out.push_str("        pattern_binding_rule_variants: PatternBindingRuleVariants::default(),\n");
    out.push_str("        variant_payload_field_access_rule_variants: VariantPayloadFieldAccessRuleVariants::default(),\n");
    out.push_str(
        "        verifier_output_policy_variants: VerifierOutputPolicyVariants::default(),\n",
    );
    out.push_str("        callable_strategy_variants: CallableStrategyVariants::default(),\n");
    out.push_str("        emit_model_variants: EmitModelVariants::default(),\n");
    push_field(&mut out, "clusters", &format!("{clusters_fn}()"), 2);
    push_field(
        &mut out,
        "optional_match_disjs",
        &format!("{optional_match_disjs_fn}()"),
        2,
    );
    push_field(
        &mut out,
        "declaration_append_begin_after_bootstrap",
        &dag.declarations().len().to_string(),
        2,
    );
    out.push_str("    }\n");
    out.push_str("}\n");
    out.push_str(&format!(
        "\n#[allow(clippy::vec_init_then_push)]\nfn {nodes_fn}() -> Vec<Behavior> {{\n    {}\n}}\n",
        render_behaviors(dag.nodes())
    ));
    out.push_str(&format!(
        "\n#[allow(clippy::vec_init_then_push)]\nfn {declarations_fn}() -> Vec<Declaration> {{\n    {}\n}}\n",
        render_declarations(dag.declarations())
    ));
    out.push_str(&format!(
        "\nfn {ports_fn}() -> HashMap<PortId, Port> {{\n    {}\n}}\n",
        render_ports(dag)
    ));
    out.push_str(&format!(
        "\nfn {diagnostics_fn}() -> DiagnosticTable {{\n    {}\n}}\n",
        render_diagnostics(dag)
    ));
    out.push_str(&format!(
        "\nfn {clusters_fn}() -> Vec<Cluster> {{\n    {}\n}}\n",
        render_clusters(dag)
    ));
    out.push_str(&format!(
        "\nfn {optional_match_disjs_fn}() -> HashMap<DeclarationId, DeclarationId> {{\n    {}\n}}\n",
        render_optional_match_disjs(dag)
    ));
    out
}

fn push_field(out: &mut String, name: &str, value: &str, indent: usize) {
    let padding = " ".repeat(indent * 4);
    let _ = writeln!(out, "{padding}{name}: {value},");
}

fn render_declarations(declarations: &[Declaration]) -> String {
    if declarations.is_empty() {
        return "Vec::new()".to_string();
    }
    let mut out = format!(
        "{{ let mut declarations = Vec::with_capacity({});\n",
        declarations.len()
    );
    for declaration in declarations {
        let _ = writeln!(
            out,
            "        declarations.push({});",
            render_declaration(declaration)
        );
    }
    out.push_str("        declarations }\n");
    out
}

fn render_declaration(declaration: &Declaration) -> String {
    format!(
        "Declaration {{ id: {}, name: {}, connective: {}, type_params: {}, phantom_params: {}, meta_tag: {}, specialization_parent: {}, inhabits: {}, value_body: {}, refinement: {}, nominal_opacity: {}, span: {} }}",
        render_declaration_id(declaration.id),
        render_opt_string(declaration.name.as_deref()),
        render_type_connective(&declaration.connective),
        render_declaration_id_vec(&declaration.type_params),
        render_phantom_params(&declaration.phantom_params),
        render_opt_declaration_id(declaration.meta_tag),
        render_opt_declaration_id(declaration.specialization_parent),
        render_opt_declaration_id(declaration.inhabits),
        render_opt_value_body(declaration.value_body.as_ref()),
        render_opt_declaration_id(declaration.refinement),
        render_opt_nominal_opacity(declaration.nominal_opacity.as_ref()),
        render_source_span(&declaration.span),
    )
}

fn render_opt_nominal_opacity(opacity: Option<&NominalOpacity>) -> String {
    match opacity {
        None => "None".to_string(),
        Some(o) => {
            let ids: Vec<String> = o
                .permitted_accessors
                .iter()
                .map(|id| render_declaration_id(*id))
                .collect();
            format!(
                "Some(NominalOpacity {{ permitted_accessors: vec![{}] }})",
                ids.join(", ")
            )
        }
    }
}

fn render_phantom_params(params: &[PhantomParameter]) -> String {
    if params.is_empty() {
        return "Vec::new()".to_string();
    }
    let rendered: Vec<String> = params
        .iter()
        .map(|param| {
            format!(
                "PhantomParameter {{ parameter: {}, algebra: {} }}",
                render_declaration_id(param.parameter),
                render_declaration_id(param.algebra)
            )
        })
        .collect();
    format!("vec![{}]", rendered.join(", "))
}

fn render_type_connective(connective: &TypeConnective) -> String {
    match connective {
        TypeConnective::Atom(payload) => {
            format!("TypeConnective::Atom({})", render_atom_payload(payload))
        }
        TypeConnective::Conj { children } => {
            format!(
                "TypeConnective::Conj {{ children: {} }}",
                render_fields(children)
            )
        }
        TypeConnective::Disj { variants } => {
            format!(
                "TypeConnective::Disj {{ variants: {} }}",
                render_fields(variants)
            )
        }
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => format!(
            "TypeConnective::Arrow {{ inputs: {}, output: {}, body: {} }}",
            render_declaration_id_vec(inputs),
            render_declaration_id(*output),
            render_arrow_body(body),
        ),
        TypeConnective::Cardinality(payload) => format!(
            "TypeConnective::Cardinality(CardinalityPayload::new_unchecked_bypassing_idempotence({}, {}))",
            render_declaration_id(payload.element()),
            render_cardinality_bound(&payload.bound()),
        ),
        TypeConnective::Instantiation {
            template,
            arguments,
        } => format!(
            "TypeConnective::Instantiation {{ template: {}, arguments: {} }}",
            render_declaration_id(*template),
            render_template_arguments(arguments),
        ),
    }
}

fn render_fields(fields: &[Field]) -> String {
    let values: Vec<String> = fields
        .iter()
        .map(|field| {
            format!(
                "Field {{ label: {:?}.to_string(), ty: {} }}",
                field.label,
                render_declaration_id(field.ty)
            )
        })
        .collect();
    render_vec(&values)
}

fn render_atom_payload(payload: &AtomPayload) -> String {
    match payload {
        AtomPayload::Literal(bits) => {
            format!("AtomPayload::Literal({})", render_literal_bits(bits))
        }
        AtomPayload::UnresolvedIdentifier(name) => {
            format!("AtomPayload::UnresolvedIdentifier({name:?}.to_string())")
        }
        AtomPayload::ResolvedByStructure(id) => {
            format!(
                "AtomPayload::ResolvedByStructure({})",
                render_declaration_id(*id)
            )
        }
        AtomPayload::ResolvedByName(id) => {
            format!(
                "AtomPayload::ResolvedByName({})",
                render_declaration_id(*id)
            )
        }
        AtomPayload::TypeParam(name) => format!("AtomPayload::TypeParam({name:?}.to_string())"),
    }
}

fn render_arrow_body(body: &ArrowBody) -> String {
    match body {
        ArrowBody::UserDefined(id) => format!(
            "ArrowBody::UserDefined(BindNodeId::new_unchecked({}))",
            render_node_id(id.node_id())
        ),
        ArrowBody::ExternalRealization(id) => {
            format!(
                "ArrowBody::ExternalRealization({})",
                render_declaration_id(*id)
            )
        }
        ArrowBody::Pending => "ArrowBody::Pending".to_string(),
        ArrowBody::NoBody => "ArrowBody::NoBody".to_string(),
        ArrowBody::Unparsed(span) => format!("ArrowBody::Unparsed({})", render_source_span(span)),
    }
}

fn render_cardinality_bound(bound: &CardinalityBound) -> String {
    match bound {
        CardinalityBound::Exact(value) => format!("CardinalityBound::Exact({value})"),
        CardinalityBound::AtMostOne => "CardinalityBound::AtMostOne".to_string(),
        CardinalityBound::Unbounded => "CardinalityBound::Unbounded".to_string(),
    }
}

fn render_template_arguments(arguments: &[TemplateArgument]) -> String {
    let values: Vec<String> = arguments
        .iter()
        .map(|arg| {
            format!(
                "TemplateArgument {{ parameter: {}, value: {} }}",
                render_declaration_id(arg.parameter),
                render_declaration_id(arg.value)
            )
        })
        .collect();
    render_vec(&values)
}

fn render_opt_value_body(value_body: Option<&ValueBody>) -> String {
    match value_body {
        Some(value_body) => format!("Some({})", render_value_body(value_body)),
        None => "None".to_string(),
    }
}

fn render_value_body(value_body: &ValueBody) -> String {
    match value_body {
        ValueBody::Unparsed(span) => format!("ValueBody::Unparsed({})", render_source_span(span)),
        ValueBody::Structural { fields } => {
            format!(
                "ValueBody::Structural {{ fields: {} }}",
                render_named_field_values(fields)
            )
        }
        ValueBody::Scalar(bits) => format!("ValueBody::Scalar({})", render_literal_bits(bits)),
        ValueBody::List(values) => {
            let rendered: Vec<String> = values.iter().map(render_field_value).collect();
            format!("ValueBody::List({})", render_vec(&rendered))
        }
        ValueBody::Map(entries) => {
            format!(
                "ValueBody::Map({})",
                render_field_map("ValueBody::Map", entries)
            )
        }
    }
}

fn render_field_map(context: &str, map: &FieldMap) -> String {
    format!(
        "FieldMap::from_entries({}).expect({context:?})",
        render_named_field_values(map.entries())
    )
}

fn render_named_field_values(fields: &[(String, FieldValue)]) -> String {
    let values: Vec<String> = fields
        .iter()
        .map(|(label, value)| format!("({label:?}.to_string(), {})", render_field_value(value)))
        .collect();
    render_vec(&values)
}

fn render_field_value(value: &FieldValue) -> String {
    match value {
        FieldValue::Literal(bits) => format!("FieldValue::Literal({})", render_literal_bits(bits)),
        FieldValue::Reference(id) => {
            format!("FieldValue::Reference({})", render_declaration_id(*id))
        }
        FieldValue::Record(fields) => {
            format!("FieldValue::Record({})", render_named_field_values(fields))
        }
        FieldValue::List(values) => {
            let rendered: Vec<String> = values.iter().map(render_field_value).collect();
            format!("FieldValue::List({})", render_vec(&rendered))
        }
        FieldValue::Map(entries) => {
            format!(
                "FieldValue::Map({})",
                render_field_map("FieldValue::Map", entries)
            )
        }
        FieldValue::Variant {
            constructor,
            payload,
        } => {
            let rendered: Vec<String> = payload.iter().map(render_field_value).collect();
            format!(
                "FieldValue::Variant {{ constructor: {}, payload: {} }}",
                render_declaration_id(*constructor),
                render_vec(&rendered)
            )
        }
    }
}

fn render_behaviors(behaviors: &[Behavior]) -> String {
    if behaviors.is_empty() {
        return "Vec::new()".to_string();
    }
    let mut out = format!(
        "{{ let mut nodes = Vec::with_capacity({});\n",
        behaviors.len()
    );
    for behavior in behaviors {
        let _ = writeln!(out, "        nodes.push({});", render_behavior(behavior));
    }
    out.push_str("        nodes }\n");
    out
}

fn render_behavior(behavior: &Behavior) -> String {
    match behavior {
        Behavior::Value(node) => format!("Behavior::Value({})", render_value_node(node)),
        Behavior::Transform(node) => {
            format!("Behavior::Transform({})", render_transform_node(node))
        }
        Behavior::Branch(node) => format!("Behavior::Branch({})", render_branch_node(node)),
        Behavior::Loop(node) => format!("Behavior::Loop({})", render_loop_node(node)),
        Behavior::Bind(node) => format!("Behavior::Bind({})", render_bind_node(node)),
    }
}

fn render_value_node(node: &ValueNode) -> String {
    format!(
        "ValueNode {{ id: {}, data: {}, output: {}, span: {}, lane2_workflow: {} }}",
        render_node_id(node.id),
        render_literal_bits(&node.data),
        render_port_id(node.output),
        render_source_span(&node.span),
        render_lane2_workflow("ValueNode", node.id, node.lane2_workflow()),
    )
}

fn render_transform_node(node: &TransformNode) -> String {
    format!(
        "TransformNode {{ id: {}, target: {}, inputs: {}, output: {}, span: {} }}",
        render_node_id(node.id),
        render_transform_target(&node.target),
        render_port_id_vec(&node.inputs),
        render_port_id(node.output),
        render_source_span(&node.span),
    )
}

fn render_branch_node(node: &BranchNode) -> String {
    format!(
        "BranchNode {{ id: {}, input: {}, paths: {}, output: {}, span: {}, emit_participation: {} }}",
        render_node_id(node.id),
        render_port_id(node.input),
        render_paths(&node.paths),
        render_port_id(node.output),
        render_source_span(&node.span),
        render_opt_branch_emit_participation(node.emit_participation()),
    )
}

fn render_loop_node(node: &LoopNode) -> String {
    format!(
        "LoopNode {{ id: {}, source: {}, init: {}, body: {}, bound: {}, output: {}, span: {} }}",
        render_node_id(node.id),
        render_port_id(node.source),
        render_port_id(node.init),
        render_node_id(node.body),
        render_loop_bound(&node.bound),
        render_port_id(node.output),
        render_source_span(&node.span),
    )
}

fn render_bind_node(node: &BindNode) -> String {
    format!(
        "BindNode {{ id: {}, name: {:?}.to_string(), value: {}, params: {}, span: {}, lane2_workflow: {}, emit_participation: {} }}",
        render_node_id(node.id),
        node.name,
        render_port_id(node.value),
        render_port_id_vec(&node.params),
        render_source_span(&node.span),
        render_lane2_workflow("BindNode", node.id, node.lane2_workflow()),
        render_opt_bind_emit_participation(node.emit_participation()),
    )
}

fn render_opt_bind_emit_participation(p: Option<BindEmitParticipation>) -> String {
    match p {
        None => "None".to_string(),
        Some(BindEmitParticipation::UserCallable) => {
            "Some(BindEmitParticipation::UserCallable)".to_string()
        }
    }
}

fn render_opt_branch_emit_participation(p: Option<BranchEmitParticipation>) -> String {
    match p {
        None => "None".to_string(),
        Some(BranchEmitParticipation::UserMatch) => {
            "Some(BranchEmitParticipation::UserMatch)".to_string()
        }
    }
}

fn render_lane2_workflow(
    behavior: &str,
    node_id: NodeId,
    workflow: Option<&crate::dag::WorkflowEffect>,
) -> String {
    match workflow {
        None => "None".to_string(),
        Some(_) => panic!(
            "regen_bootstrap does not yet support serializing lane2_workflow on {behavior} {:?}",
            node_id
        ),
    }
}

fn render_transform_target(target: &TransformTarget) -> String {
    match target {
        TransformTarget::Callable(id) => {
            format!("TransformTarget::Callable({})", render_declaration_id(*id))
        }
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => format!(
            "TransformTarget::FieldProject {{ field_label: {field_label:?}.to_string(), field_child: {} }}",
            render_opt_declaration_id(*field_child),
        ),
        TransformTarget::Operator(kind) => {
            format!("TransformTarget::Operator({})", render_operator_kind(kind))
        }
    }
}

fn render_operator_kind(kind: &OperatorKind) -> String {
    match kind {
        OperatorKind::Arithmetic(op) => {
            format!("OperatorKind::Arithmetic({})", render_arithmetic_op(op))
        }
        OperatorKind::Comparison(op) => {
            format!("OperatorKind::Comparison({})", render_comparison_op(op))
        }
        OperatorKind::Logical(op) => format!("OperatorKind::Logical({})", render_logical_op(op)),
    }
}

fn render_arithmetic_op(op: &ArithmeticOp) -> String {
    match op {
        ArithmeticOp::Add => "ArithmeticOp::Add".to_string(),
        ArithmeticOp::Sub => "ArithmeticOp::Sub".to_string(),
        ArithmeticOp::Mul => "ArithmeticOp::Mul".to_string(),
        ArithmeticOp::Div => "ArithmeticOp::Div".to_string(),
    }
}

fn render_comparison_op(op: &ComparisonOp) -> String {
    match op {
        ComparisonOp::Eq => "ComparisonOp::Eq".to_string(),
        ComparisonOp::Ne => "ComparisonOp::Ne".to_string(),
        ComparisonOp::Lt => "ComparisonOp::Lt".to_string(),
        ComparisonOp::Le => "ComparisonOp::Le".to_string(),
        ComparisonOp::Gt => "ComparisonOp::Gt".to_string(),
        ComparisonOp::Ge => "ComparisonOp::Ge".to_string(),
    }
}

fn render_logical_op(op: &LogicalOp) -> String {
    match op {
        LogicalOp::And => "LogicalOp::And".to_string(),
        LogicalOp::Or => "LogicalOp::Or".to_string(),
    }
}

fn render_paths(paths: &[Path]) -> String {
    let values: Vec<String> = paths
        .iter()
        .map(|path| {
            format!(
                "Path {{ body: {}, output: {}, pattern: {}, binding: {} }}",
                render_node_id(path.body),
                render_port_id(path.output),
                render_branch_pattern(&path.pattern),
                render_opt_payload_binding(path.binding.as_ref()),
            )
        })
        .collect();
    render_vec(&values)
}

fn render_branch_pattern(pattern: &BranchPattern) -> String {
    match pattern {
        BranchPattern::UnresolvedVariant { name, span } => format!(
            "BranchPattern::UnresolvedVariant {{ name: {name:?}.to_string(), span: {} }}",
            render_source_span(span)
        ),
        BranchPattern::ResolvedVariant(id) => {
            format!(
                "BranchPattern::ResolvedVariant({})",
                render_declaration_id(*id)
            )
        }
    }
}

fn render_opt_payload_binding(binding: Option<&PayloadBinding>) -> String {
    match binding {
        Some(binding) => format!(
            "Some(PayloadBinding {{ binding_name: {:?}.to_string(), payload_port: {} }})",
            binding.binding_name,
            render_port_id(binding.payload_port)
        ),
        None => "None".to_string(),
    }
}

fn render_loop_bound(bound: &LoopBound) -> String {
    match bound {
        LoopBound::Cardinality { count } => {
            format!(
                "LoopBound::Cardinality {{ count: {} }}",
                render_port_id(*count)
            )
        }
        LoopBound::Descent { cluster, measure } => {
            format!(
                "LoopBound::Descent {{ cluster: {}, measure: {} }}",
                render_cluster_id(*cluster),
                render_port_id(*measure)
            )
        }
    }
}

fn render_ports(dag: &Dag) -> String {
    let ports = dag.ports();
    if ports.is_empty() {
        return "HashMap::new()".to_string();
    }
    let mut out = String::from("{ let mut ports = HashMap::new();\n");
    for port in &ports {
        let _ = writeln!(
            out,
            "        ports.insert({}, Port {{ id: {}, state: {}, produced_by: {} }});",
            render_port_id(port.id()),
            render_port_id(port.id()),
            render_port_state(port.state()),
            render_opt_node_id(port.produced_by),
        );
    }
    out.push_str("        ports }\n");
    out
}

fn render_port_state(state: &PortState) -> String {
    match state {
        PortState::Uninferred => "PortState::Uninferred".to_string(),
        PortState::Resolved(shape) => {
            format!(
                "PortState::Resolved(TypeShape::new({}))",
                render_declaration_id(shape.declaration)
            )
        }
        PortState::Unresolved => "PortState::Unresolved".to_string(),
    }
}

fn render_diagnostic_attribution(attribution: &DiagnosticAttribution) -> String {
    match attribution {
        DiagnosticAttribution::Unattributed => "DiagnosticAttribution::Unattributed".to_string(),
        DiagnosticAttribution::BootstrapAuthority(key) => format!(
            "DiagnosticAttribution::BootstrapAuthority(crate::diagnostics::BootstrapAuthorityKey::new({:?}))",
            key.path()
        ),
    }
}

fn render_diagnostics(dag: &Dag) -> String {
    let mut entries: Vec<_> = dag.diagnostics().iter_attributed().collect();
    entries.sort_by_key(|(port, _, _)| port.raw());
    if entries.is_empty() {
        return "DiagnosticTable::new()".to_string();
    }
    let mut out = String::from("{ let mut table = DiagnosticTable::new();\n");
    for (port, diagnostic, attribution) in entries {
        let _ = writeln!(
            out,
            "        table.insert({}, {}, {});",
            render_port_id(port),
            render_diagnostic(diagnostic),
            render_diagnostic_attribution(attribution)
        );
    }
    out.push_str("        table }\n");
    out
}

fn render_diagnostic(diagnostic: &Diagnostic) -> String {
    match diagnostic {
        Diagnostic::TokenizerError {
            message,
            span,
            fixes,
        } => format!(
            "Diagnostic::TokenizerError {{ message: {message:?}.to_string(), span: {}, fixes: {} }}",
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::ParseError {
            message,
            span,
            fixes,
        } => format!(
            "Diagnostic::ParseError {{ message: {message:?}.to_string(), span: {}, fixes: {} }}",
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::TypeMismatch {
            expected,
            actual,
            span,
            fixes,
        } => format!(
            "Diagnostic::TypeMismatch {{ expected: TypeShape::new({}), actual: TypeShape::new({}), span: {}, fixes: {} }}",
            render_declaration_id(expected.declaration),
            render_declaration_id(actual.declaration),
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::ArityMismatch {
            function,
            expected,
            actual,
            span,
            fixes,
        } => format!(
            "Diagnostic::ArityMismatch {{ function: {function:?}.to_string(), expected: {expected}, actual: {actual}, span: {}, fixes: {} }}",
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::ResolveError { name, span, fixes } => format!(
            "Diagnostic::ResolveError {{ name: {name:?}.to_string(), span: {}, fixes: {} }}",
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::UnitMismatch {
            operator,
            parameter,
            expected,
            actual,
            span,
            fixes,
        } => format!(
            "Diagnostic::UnitMismatch {{ operator: {operator:?}.to_string(), parameter: {parameter:?}.to_string(), expected: TypeShape::new({}), actual: TypeShape::new({}), span: {}, fixes: {} }}",
            render_declaration_id(expected.declaration),
            render_declaration_id(actual.declaration),
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::BranchConditionNotBool {
            port,
            actual_type,
            span,
            fixes,
        } => format!(
            "Diagnostic::BranchConditionNotBool {{ port: {}, actual_type: {}, span: {}, fixes: {} }}",
            render_port_id(*port),
            render_opt_type_shape(*actual_type),
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::MagnitudeOutOfRange {
            literal,
            target,
            range_min_inclusive,
            range_max_inclusive,
            expected,
            span,
            fixes,
        } => format!(
            "Diagnostic::MagnitudeOutOfRange {{ literal: {literal:?}.to_string(), target: {target:?}.to_string(), range_min_inclusive: {range_min_inclusive:?}.to_string(), range_max_inclusive: {range_max_inclusive:?}.to_string(), expected: TypeShape::new({}), span: {}, fixes: {} }}",
            render_declaration_id(expected.declaration),
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::MalformedIntegerRangeFact {
            message,
            span,
            fixes,
        } => format!(
            "Diagnostic::MalformedIntegerRangeFact {{ message: {message:?}.to_string(), span: {}, fixes: {} }}",
            render_source_span(span),
            render_corrections(fixes),
        ),
        Diagnostic::NominalOpacityViolation {
            declaration,
            accessor,
            span,
            fixes,
        } => format!(
            "Diagnostic::NominalOpacityViolation {{ declaration: {}, accessor: {}, span: {}, fixes: {} }}",
            render_declaration_id(*declaration),
            render_opt_declaration_id(*accessor),
            render_source_span(span),
            render_corrections(fixes),
        ),
    }
}

fn render_corrections(corrections: &[crate::diagnostics::Correction]) -> String {
    let values: Vec<String> = corrections
        .iter()
        .map(|correction| {
            format!(
                "crate::diagnostics::Correction {{ description: {:?}.to_string(), span: {}, new_source: {:?}.to_string() }}",
                correction.description,
                render_source_span(&correction.span),
                correction.new_source,
            )
        })
        .collect();
    render_vec(&values)
}

fn render_clusters(dag: &Dag) -> String {
    let values: Vec<String> = dag.clusters().iter().map(render_cluster).collect();
    render_vec(&values)
}

fn render_cluster(cluster: &Cluster) -> String {
    format!(
        "Cluster {{ members: {}, intra_cluster_calls: {} }}",
        render_non_singleton_member_descents(&cluster.members),
        render_non_empty_intra_cluster_calls(&cluster.intra_cluster_calls),
    )
}

fn render_non_singleton_member_descents(values: &NonSingletonList<MemberDescent>) -> String {
    format!(
        "NonSingletonList {{ first: {}, second: {}, rest: {} }}",
        render_member_descent(&values.first),
        render_member_descent(&values.second),
        render_vec(
            &values
                .rest
                .iter()
                .map(render_member_descent)
                .collect::<Vec<_>>()
        ),
    )
}

fn render_member_descent(value: &MemberDescent) -> String {
    format!(
        "MemberDescent {{ param: ParamRef {{ member: {}, slot: {} }} }}",
        render_node_id(value.param.member_of()),
        value.param.slot_of(),
    )
}

fn render_non_empty_intra_cluster_calls(values: &NonEmptyList<IntraClusterCall>) -> String {
    format!(
        "NonEmptyList {{ first: {}, rest: {} }}",
        render_intra_cluster_call(&values.first),
        render_vec(
            &values
                .rest
                .iter()
                .map(render_intra_cluster_call)
                .collect::<Vec<_>>()
        ),
    )
}

fn render_intra_cluster_call(value: &IntraClusterCall) -> String {
    format!(
        "IntraClusterCall {{ transform: TransformRef({}) }}",
        render_node_id(value.transform.node_id())
    )
}

fn render_optional_match_disjs(dag: &Dag) -> String {
    let mut entries: Vec<_> = dag.optional_match_disjs().iter().collect();
    entries.sort_by_key(|(key, _)| key.raw());
    if entries.is_empty() {
        return "HashMap::new()".to_string();
    }
    let mut out = String::from("{ let mut map = HashMap::new();\n");
    for (key, value) in entries {
        let _ = writeln!(
            out,
            "        map.insert({}, {});",
            render_declaration_id(*key),
            render_declaration_id(*value)
        );
    }
    out.push_str("        map }\n");
    out
}

fn render_literal_bits(bits: &LiteralBits) -> String {
    match bits {
        LiteralBits::Int(value) => format!("LiteralBits::Int({value})"),
        LiteralBits::Bool(value) => format!("LiteralBits::Bool({value})"),
        LiteralBits::String(value) => format!("LiteralBits::String({value:?}.to_string())"),
    }
}

fn render_source_span(span: &crate::diagnostics::SourceSpan) -> String {
    format!(
        "SourceSpan::new({:?}, {}, {})",
        span.file, span.byte_start, span.byte_end
    )
}

fn render_opt_type_shape(shape: Option<crate::types::TypeShape>) -> String {
    match shape {
        Some(shape) => format!(
            "Some(TypeShape::new({}))",
            render_declaration_id(shape.declaration)
        ),
        None => "None".to_string(),
    }
}

fn render_declaration_id_vec(values: &[DeclarationId]) -> String {
    let rendered: Vec<String> = values.iter().map(|id| render_declaration_id(*id)).collect();
    render_vec(&rendered)
}

fn render_port_id_vec(values: &[PortId]) -> String {
    let rendered: Vec<String> = values.iter().map(|id| render_port_id(*id)).collect();
    render_vec(&rendered)
}

fn render_opt_declaration_id(id: Option<DeclarationId>) -> String {
    match id {
        Some(id) => format!("Some({})", render_declaration_id(id)),
        None => "None".to_string(),
    }
}

fn render_opt_node_id(id: Option<NodeId>) -> String {
    match id {
        Some(id) => format!("Some({})", render_node_id(id)),
        None => "None".to_string(),
    }
}

fn render_opt_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("Some({value:?}.to_string())"),
        None => "None".to_string(),
    }
}

fn render_declaration_id(id: DeclarationId) -> String {
    format!("DeclarationId({})", id.raw())
}

fn render_port_id(id: PortId) -> String {
    format!("PortId({})", id.raw())
}

fn render_node_id(id: NodeId) -> String {
    format!("NodeId({})", id.raw())
}

fn render_cluster_id(id: ClusterId) -> String {
    format!("ClusterId({})", id.raw())
}

fn render_vec(values: &[String]) -> String {
    if values.is_empty() {
        "vec![]".to_string()
    } else {
        format!("vec![{}]", values.join(", "))
    }
}
