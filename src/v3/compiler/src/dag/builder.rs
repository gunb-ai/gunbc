use super::*;
use crate::operators::OperatorKind;
use crate::types::TypeShape;

const BUILDER_TYPE_WALK_DEPTH_LIMIT: usize = 32;

#[allow(dead_code)]
impl Dag {
    /// Test-facing builder: allocate a detached port that already carries a
    /// resolved shape. This keeps unit-style graph construction on the same
    /// Port-state invariant as post-infer dags.
    pub(crate) fn alloc_port_with_shape(&mut self, shape: TypeShape) -> PortId {
        let port = self.alloc_port(None);
        self.set_port_type(port, shape);
        port
    }

    /// Test-facing builder: append a `Value` node and return its output port.
    /// The output port is resolved to the corresponding primitive shape.
    pub(crate) fn push_value(&mut self, bits: LiteralBits, span: SourceSpan) -> PortId {
        let node_id = self.alloc_node_id();
        let output = self.alloc_port(Some(node_id));
        let output_shape = self.literal_shape(&bits);
        self.push_node(Behavior::Value(ValueNode {
            id: node_id,
            data: bits,
            output,
            span,
            lane2_workflow: None,
        }));
        self.set_port_type(output, output_shape);
        output
    }

    /// Test-facing builder: append a `Transform` node and return its output
    /// port. The builder validates referenced input ports and seeds the output
    /// port shape when the target carries enough structural information.
    pub(crate) fn push_transform(
        &mut self,
        target: TransformTarget,
        inputs: Vec<PortId>,
        span: SourceSpan,
    ) -> PortId {
        self.assert_transform_inputs(&target, &inputs);
        let node_id = self.alloc_node_id();
        let output = self.alloc_port(Some(node_id));
        let output_shape = self.transform_output_shape(&target, &inputs);
        self.push_node(Behavior::Transform(TransformNode {
            id: node_id,
            target,
            inputs,
            output,
            span,
        }));
        if let Some(shape) = output_shape {
            self.set_port_type(output, shape);
        }
        output
    }

    /// Test-facing builder: append a `Bind` node and return its `NodeId`.
    /// `Bind` reuses the supplied value port as its result port rather than
    /// synthesizing a parallel output edge.
    pub(crate) fn push_bind(
        &mut self,
        name: impl Into<String>,
        value: PortId,
        params: Vec<PortId>,
        span: SourceSpan,
    ) -> NodeId {
        self.assert_port_exists(value, "push_bind(value)");
        self.assert_ports_exist(&params, "push_bind(params)");
        let node_id = self.alloc_node_id();
        self.push_node(Behavior::Bind(BindNode {
            id: node_id,
            name: name.into(),
            value,
            params,
            span,
            lane2_workflow: None,
        }));
        node_id
    }

    /// Test-facing builder: append a `Branch` node and return its output port.
    /// Every path body/output reference is validated before the node is pushed.
    /// The output port is resolved when every arm output already has the same
    /// resolved shape.
    pub(crate) fn push_branch(
        &mut self,
        input: PortId,
        paths: Vec<Path>,
        span: SourceSpan,
    ) -> PortId {
        assert!(!paths.is_empty(), "push_branch requires at least one path");
        self.assert_port_exists(input, "push_branch(input)");
        for path in &paths {
            self.assert_node_exists(path.body, "push_branch(path.body)");
            self.assert_port_exists(path.output, "push_branch(path.output)");
            if let BranchPattern::ResolvedVariant(variant) = path.pattern {
                self.assert_declaration_exists(variant, "push_branch(path.pattern)");
            }
            if let Some(binding) = &path.binding {
                self.assert_port_exists(
                    binding.payload_port,
                    "push_branch(path.binding.payload_port)",
                );
            }
        }
        let node_id = self.alloc_node_id();
        let output = self.alloc_port(Some(node_id));
        let output_shape = self.common_resolved_shape(paths.iter().map(|path| path.output));
        self.push_node(Behavior::Branch(BranchNode {
            id: node_id,
            input,
            paths,
            output,
            span,
        }));
        if let Some(shape) = output_shape {
            self.set_port_type(output, shape);
        }
        output
    }

    /// Test-facing builder: append a `Loop` node and return its output port.
    /// The output port inherits the init-port shape when that shape is already
    /// resolved on the supplied graph fragment.
    pub(crate) fn push_loop(
        &mut self,
        source: PortId,
        init: PortId,
        body: NodeId,
        bound: LoopBound,
        span: SourceSpan,
    ) -> PortId {
        self.assert_port_exists(source, "push_loop(source)");
        self.assert_port_exists(init, "push_loop(init)");
        self.assert_node_exists(body, "push_loop(body)");
        match bound {
            LoopBound::Cardinality { count } => {
                self.assert_port_exists(count, "push_loop(bound.count)");
            }
            LoopBound::Descent { cluster } => {
                assert!(
                    self.clusters.get(cluster.index()).is_some(),
                    "push_loop(bound.cluster): unknown cluster {:?}",
                    cluster
                );
            }
        }
        let node_id = self.alloc_node_id();
        let output = self.alloc_port(Some(node_id));
        let output_shape = self.resolved_port_shape(init);
        self.push_node(Behavior::Loop(LoopNode {
            id: node_id,
            source,
            init,
            body,
            bound,
            output,
            span,
        }));
        if let Some(shape) = output_shape {
            self.set_port_type(output, shape);
        }
        output
    }

    /// Test-facing builder: append a `Conj` declaration and return its
    /// `DeclarationId`.
    pub(crate) fn push_conj(
        &mut self,
        name: Option<String>,
        children: Vec<Field>,
        span: SourceSpan,
    ) -> DeclarationId {
        self.assert_fields_exist(&children, "push_conj(children)");
        let id = self.alloc_declaration_id();
        self.push_declaration(Declaration {
            id,
            name,
            connective: TypeConnective::Conj { children },
            type_params: Vec::new(),
            meta_tag: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            span,
        });
        id
    }

    /// Test-facing builder: append an `Atom` declaration and return its
    /// `DeclarationId`.
    pub(crate) fn push_atom(
        &mut self,
        name: Option<String>,
        payload: AtomPayload,
        span: SourceSpan,
    ) -> DeclarationId {
        if let Some(target) = payload.resolved_id() {
            self.assert_declaration_exists(target, "push_atom(payload)");
        }
        let id = self.alloc_declaration_id();
        self.push_declaration(Declaration {
            id,
            name,
            connective: TypeConnective::Atom(payload),
            type_params: Vec::new(),
            meta_tag: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            span,
        });
        id
    }

    fn literal_shape(&self, bits: &LiteralBits) -> TypeShape {
        match bits {
            LiteralBits::Int(_) => self
                .int_shape()
                .expect("push_value requires bootstrap Int shape"),
            LiteralBits::Bool(_) => self
                .bool_shape()
                .expect("push_value requires bootstrap Bool shape"),
            LiteralBits::String(_) => self
                .string_shape()
                .expect("push_value requires bootstrap String shape"),
        }
    }

    fn transform_output_shape(
        &self,
        target: &TransformTarget,
        inputs: &[PortId],
    ) -> Option<TypeShape> {
        match target {
            TransformTarget::Callable(target) => self.callable_output_shape(*target),
            TransformTarget::FieldProject { field_child, .. } => field_child.map(|child| {
                self.assert_declaration_exists(child, "push_transform(target.field_child)");
                TypeShape::new(child)
            }),
            TransformTarget::Operator(kind) => self.operator_output_shape(*kind, inputs),
        }
    }

    fn callable_output_shape(&self, decl_id: DeclarationId) -> Option<TypeShape> {
        let mut subst = Vec::new();
        self.callable_output_shape_with_subst(decl_id, &mut subst, 0)
    }

    fn callable_output_shape_with_subst(
        &self,
        decl_id: DeclarationId,
        subst: &mut Vec<Vec<TemplateArgument>>,
        depth: usize,
    ) -> Option<TypeShape> {
        if depth >= BUILDER_TYPE_WALK_DEPTH_LIMIT {
            return None;
        }
        match &self.declaration(decl_id).connective {
            TypeConnective::Arrow { output, .. } => self.signature_type_shape(*output, subst, depth + 1),
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                subst.push(arguments.clone());
                let result = self.callable_output_shape_with_subst(*template, subst, depth + 1);
                subst.pop();
                result
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.callable_output_shape_with_subst(*next, subst, depth + 1)
            }
            TypeConnective::Atom(AtomPayload::Literal(_))
            | TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
            | TypeConnective::Atom(AtomPayload::TypeParam(_))
            | TypeConnective::Conj { .. }
            | TypeConnective::Disj { .. }
            | TypeConnective::Cardinality { .. } => None,
        }
    }

    fn operator_output_shape(&self, kind: OperatorKind, inputs: &[PortId]) -> Option<TypeShape> {
        match kind {
            OperatorKind::Arithmetic(_) => inputs
                .first()
                .copied()
                .and_then(|port| self.resolved_port_shape(port)),
            OperatorKind::Comparison(_) | OperatorKind::Logical(_) => self.bool_shape(),
        }
    }

    fn resolved_port_shape(&self, port: PortId) -> Option<TypeShape> {
        match self.port(port).state() {
            PortState::Resolved(shape) => Some(*shape),
            PortState::Uninferred | PortState::Unresolved => None,
        }
    }

    fn signature_type_shape(
        &self,
        current: DeclarationId,
        subst: &[Vec<TemplateArgument>],
        depth: usize,
    ) -> Option<TypeShape> {
        if depth >= BUILDER_TYPE_WALK_DEPTH_LIMIT {
            return None;
        }
        let decl = self.declaration(current);
        if decl.name.is_some() || decl.refinement.is_some() {
            return Some(TypeShape::new(current));
        }
        match &decl.connective {
            TypeConnective::Instantiation { .. } => self
                .resolve_decl_with_subst(current, subst, depth + 1)
                .map(TypeShape::new)
                .or_else(|| Some(TypeShape::new(current))),
            TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
                if let Some(bound) = self.lookup_template_argument(current, subst) {
                    self.signature_type_shape(bound, subst, depth + 1)
                } else {
                    Some(TypeShape::new(current))
                }
            }
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.signature_type_shape(*next, subst, depth + 1)
            }
            TypeConnective::Cardinality { .. } => Some(TypeShape::new(current)),
            TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
            | TypeConnective::Atom(AtomPayload::Literal(_))
            | TypeConnective::Conj { .. }
            | TypeConnective::Disj { .. }
            | TypeConnective::Arrow { .. } => None,
        }
    }

    fn resolve_decl_with_subst(
        &self,
        current: DeclarationId,
        subst: &[Vec<TemplateArgument>],
        depth: usize,
    ) -> Option<DeclarationId> {
        if depth >= BUILDER_TYPE_WALK_DEPTH_LIMIT {
            return None;
        }
        let decl = self.declaration(current);
        match &decl.connective {
            TypeConnective::Atom(AtomPayload::TypeParam(_)) => self
                .lookup_template_argument(current, subst)
                .and_then(|bound| self.resolve_decl_with_subst(bound, subst, depth + 1))
                .or(Some(current)),
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
                self.resolve_decl_with_subst(*next, subst, depth + 1)
            }
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                let specialized_arguments: Vec<TemplateArgument> = arguments
                    .iter()
                    .map(|arg| {
                        Some(TemplateArgument {
                            parameter: arg.parameter,
                            value: self.resolve_decl_with_subst(arg.value, subst, depth + 1)?,
                        })
                    })
                    .collect::<Option<_>>()?;
                if specialized_arguments
                    .iter()
                    .zip(arguments.iter())
                    .all(|(lhs, rhs)| lhs.parameter == rhs.parameter && lhs.value == rhs.value)
                {
                    return Some(current);
                }
                self.find_equivalent_decl_instantiation(*template, &specialized_arguments)
                    .or(Some(current))
            }
            TypeConnective::Cardinality { element, bound } => {
                let specialized_element =
                    self.resolve_decl_with_subst(*element, subst, depth + 1)?;
                if specialized_element == *element {
                    return Some(current);
                }
                self.find_equivalent_decl_cardinality(specialized_element, bound.clone())
                    .or(Some(current))
            }
            _ => Some(current),
        }
    }

    fn lookup_template_argument(
        &self,
        parameter: DeclarationId,
        subst: &[Vec<TemplateArgument>],
    ) -> Option<DeclarationId> {
        for frame in subst.iter().rev() {
            for arg in frame {
                if arg.parameter == parameter {
                    if arg.value == parameter {
                        return None;
                    }
                    return Some(arg.value);
                }
            }
        }
        None
    }

    fn find_equivalent_decl_instantiation(
        &self,
        template: DeclarationId,
        arguments: &[TemplateArgument],
    ) -> Option<DeclarationId> {
        self.declarations().iter().find_map(|decl| {
            let TypeConnective::Instantiation {
                template: existing_template,
                arguments: existing_arguments,
            } = &decl.connective
            else {
                return None;
            };
            (template == *existing_template
                && existing_arguments.len() == arguments.len()
                && existing_arguments
                    .iter()
                    .zip(arguments.iter())
                    .all(|(lhs, rhs)| lhs.parameter == rhs.parameter && lhs.value == rhs.value))
            .then_some(decl.id)
        })
    }

    fn find_equivalent_decl_cardinality(
        &self,
        element: DeclarationId,
        bound: CardinalityBound,
    ) -> Option<DeclarationId> {
        self.declarations().iter().find_map(|decl| {
            let TypeConnective::Cardinality {
                element: existing_element,
                bound: existing_bound,
            } = &decl.connective
            else {
                return None;
            };
            (element == *existing_element && bound == *existing_bound).then_some(decl.id)
        })
    }

    fn common_resolved_shape(&self, ports: impl IntoIterator<Item = PortId>) -> Option<TypeShape> {
        let mut iter = ports.into_iter();
        let first = self.resolved_port_shape(iter.next()?)?;
        for port in iter {
            if self.resolved_port_shape(port)? != first {
                return None;
            }
        }
        Some(first)
    }

    fn assert_transform_inputs(&self, target: &TransformTarget, inputs: &[PortId]) {
        self.assert_ports_exist(inputs, "push_transform(inputs)");
        match target {
            TransformTarget::Callable(target_decl) => {
                self.assert_declaration_exists(*target_decl, "push_transform(target)");
            }
            TransformTarget::FieldProject { .. } => {
                assert!(
                    inputs.len() == 1,
                    "push_transform(FieldProject) requires exactly one input port"
                );
            }
            TransformTarget::Operator(_) => {
                assert!(
                    inputs.len() == 2,
                    "push_transform(Operator) requires exactly two input ports"
                );
            }
        }
    }

    fn assert_fields_exist(&self, fields: &[Field], context: &str) {
        for field in fields {
            self.assert_declaration_exists(field.ty, context);
        }
    }

    fn assert_ports_exist(&self, ports: &[PortId], context: &str) {
        for port in ports {
            self.assert_port_exists(*port, context);
        }
    }

    fn assert_port_exists(&self, port: PortId, context: &str) {
        assert!(
            self.port_opt(&port).is_some(),
            "{context}: unknown port {:?}",
            port
        );
    }

    fn assert_node_exists(&self, node: NodeId, context: &str) {
        assert!(
            self.node_opt(&node).is_some(),
            "{context}: unknown node {:?}",
            node
        );
    }

    fn assert_declaration_exists(&self, decl: DeclarationId, context: &str) {
        assert!(
            self.declarations.get(decl.index()).is_some(),
            "{context}: unknown declaration {:?}",
            decl
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operators::{ArithmeticOp, ComparisonOp};

    fn span() -> SourceSpan {
        SourceSpan::new("<builder-test>", 0, 0)
    }

    fn push_test_declaration(
        dag: &mut Dag,
        name: Option<&str>,
        connective: TypeConnective,
        type_params: Vec<DeclarationId>,
    ) -> DeclarationId {
        let id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id,
            name: name.map(str::to_string),
            connective,
            type_params,
            meta_tag: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            span: span(),
        });
        id
    }

    #[test]
    fn alloc_port_with_shape_marks_port_resolved() {
        let mut dag = Dag::new();
        let int_shape = dag.int_shape().expect("bootstrap Int");
        let port = dag.alloc_port_with_shape(int_shape);
        let port_ref = dag.port(port);
        assert_eq!(port_ref.produced_by, None);
        assert_eq!(port_ref.state(), &PortState::Resolved(int_shape));
    }

    #[test]
    fn push_value_sets_output_shape_and_producer() {
        let mut dag = Dag::new();
        let output = dag.push_value(LiteralBits::Int(7), span());
        let producer = dag.port(output).produced_by.expect("value producer");
        let value = dag.node(producer).as_value().expect("value node");
        assert_eq!(value.output, output);
        assert_eq!(value.data, LiteralBits::Int(7));
        assert_eq!(
            dag.port(output).state(),
            &PortState::Resolved(dag.int_shape().expect("bootstrap Int"))
        );
    }

    #[test]
    fn push_transform_seeds_callable_output_shape() {
        let mut dag = Dag::new();
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("bootstrap Int declaration")
            .id;
        let input = dag.alloc_port_with_shape(TypeShape::new(int_decl));
        let arrow = push_test_declaration(
            &mut dag,
            Some("test_arrow"),
            TypeConnective::Arrow {
                inputs: vec![int_decl],
                output: int_decl,
                body: ArrowBody::NoBody,
            },
            Vec::new(),
        );
        let callable = dag.push_atom(
            Some("test_callable".to_string()),
            AtomPayload::ResolvedByStructure(arrow),
            span(),
        );

        let output = dag.push_transform(TransformTarget::Callable(callable), vec![input], span());
        let producer = dag.port(output).produced_by.expect("transform producer");
        let transform = dag.node(producer).as_transform().expect("transform node");
        assert_eq!(transform.output, output);
        assert_eq!(transform.inputs, vec![input]);
        assert_eq!(
            dag.port(output).state(),
            &PortState::Resolved(TypeShape::new(int_decl))
        );
    }

    #[test]
    fn push_transform_specializes_instantiated_callable_output_shape() {
        let mut dag = Dag::new();
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("bootstrap Int declaration")
            .id;
        let input = dag.alloc_port_with_shape(TypeShape::new(int_decl));
        let type_param = push_test_declaration(
            &mut dag,
            None,
            TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            Vec::new(),
        );
        let generic_callable = push_test_declaration(
            &mut dag,
            Some("generic_identity"),
            TypeConnective::Arrow {
                inputs: vec![type_param],
                output: type_param,
                body: ArrowBody::NoBody,
            },
            vec![type_param],
        );
        let specialized_callable = push_test_declaration(
            &mut dag,
            None,
            TypeConnective::Instantiation {
                template: generic_callable,
                arguments: vec![TemplateArgument {
                    parameter: type_param,
                    value: int_decl,
                }],
            },
            Vec::new(),
        );

        let output = dag.push_transform(
            TransformTarget::Callable(specialized_callable),
            vec![input],
            span(),
        );

        assert_eq!(
            dag.port(output).state(),
            &PortState::Resolved(TypeShape::new(int_decl))
        );
    }

    #[test]
    fn push_transform_specializes_nested_instantiated_output_shape() {
        let mut dag = Dag::new();
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("bootstrap Int declaration")
            .id;
        let input = dag.alloc_port_with_shape(TypeShape::new(int_decl));
        let type_param = push_test_declaration(
            &mut dag,
            None,
            TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            Vec::new(),
        );
        let list_template = push_test_declaration(
            &mut dag,
            Some("TestList"),
            TypeConnective::Cardinality {
                element: type_param,
                bound: CardinalityBound::Unbounded,
            },
            vec![type_param],
        );
        let list_of_t = push_test_declaration(
            &mut dag,
            None,
            TypeConnective::Instantiation {
                template: list_template,
                arguments: vec![TemplateArgument {
                    parameter: type_param,
                    value: type_param,
                }],
            },
            Vec::new(),
        );
        let list_of_int = push_test_declaration(
            &mut dag,
            None,
            TypeConnective::Instantiation {
                template: list_template,
                arguments: vec![TemplateArgument {
                    parameter: type_param,
                    value: int_decl,
                }],
            },
            Vec::new(),
        );
        let generic_callable = push_test_declaration(
            &mut dag,
            Some("generic_wrap"),
            TypeConnective::Arrow {
                inputs: vec![type_param],
                output: list_of_t,
                body: ArrowBody::NoBody,
            },
            vec![type_param],
        );
        let specialized_callable = push_test_declaration(
            &mut dag,
            None,
            TypeConnective::Instantiation {
                template: generic_callable,
                arguments: vec![TemplateArgument {
                    parameter: type_param,
                    value: int_decl,
                }],
            },
            Vec::new(),
        );

        let output = dag.push_transform(
            TransformTarget::Callable(specialized_callable),
            vec![input],
            span(),
        );

        assert_eq!(
            dag.port(output).state(),
            &PortState::Resolved(TypeShape::new(list_of_int))
        );
    }

    #[test]
    fn push_bind_reuses_supplied_value_port() {
        let mut dag = Dag::new();
        let value = dag.push_value(LiteralBits::Bool(true), span());
        let param = dag.alloc_port_with_shape(dag.bool_shape().expect("bootstrap Bool"));

        let bind_id = dag.push_bind("flag", value, vec![param], span());
        let bind = dag.node(bind_id).as_bind().expect("bind node");
        assert_eq!(bind.name, "flag");
        assert_eq!(bind.value, value);
        assert_eq!(bind.params, vec![param]);
        assert_eq!(bind.result_port(), value);
    }

    #[test]
    fn push_branch_reuses_arm_shapes_for_output() {
        let mut dag = Dag::new();
        let cond = dag.push_value(LiteralBits::Bool(true), span());
        let lhs = dag.push_value(LiteralBits::Int(1), span());
        let rhs = dag.push_value(LiteralBits::Int(2), span());
        let lhs_body = dag.push_bind("lhs", lhs, Vec::new(), span());
        let rhs_body = dag.push_bind("rhs", rhs, Vec::new(), span());

        let output = dag.push_branch(
            cond,
            vec![
                Path {
                    body: lhs_body,
                    output: lhs,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Left".to_string(),
                        span: span(),
                    },
                    binding: None,
                },
                Path {
                    body: rhs_body,
                    output: rhs,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Right".to_string(),
                        span: span(),
                    },
                    binding: None,
                },
            ],
            span(),
        );

        let producer = dag.port(output).produced_by.expect("branch producer");
        let branch = dag.node(producer).as_branch().expect("branch node");
        assert_eq!(branch.input, cond);
        assert_eq!(branch.output, output);
        assert_eq!(branch.paths.len(), 2);
        assert_eq!(
            dag.port(output).state(),
            &PortState::Resolved(dag.int_shape().expect("bootstrap Int"))
        );
    }

    #[test]
    fn push_loop_reuses_init_shape_for_output() {
        let mut dag = Dag::new();
        let source = dag.push_value(LiteralBits::Int(4), span());
        let init = dag.push_value(LiteralBits::Int(0), span());
        let body = dag.push_bind("loop_body", init, Vec::new(), span());
        let bound = LoopBound::Cardinality { count: source };

        let output = dag.push_loop(source, init, body, bound, span());

        let producer = dag.port(output).produced_by.expect("loop producer");
        let loop_node = dag.node(producer).as_loop().expect("loop node");
        assert_eq!(loop_node.source, source);
        assert_eq!(loop_node.init, init);
        assert_eq!(loop_node.body, body);
        assert_eq!(
            dag.port(output).state(),
            &PortState::Resolved(dag.int_shape().expect("bootstrap Int"))
        );
    }

    #[test]
    fn push_conj_allocates_named_record_declaration() {
        let mut dag = Dag::new();
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("bootstrap Int declaration")
            .id;
        let decl_id = dag.push_conj(
            Some("Point".to_string()),
            vec![
                Field {
                    label: "x".to_string(),
                    ty: int_decl,
                },
                Field {
                    label: "y".to_string(),
                    ty: int_decl,
                },
            ],
            span(),
        );

        let decl = dag.declaration(decl_id);
        assert_eq!(decl.name.as_deref(), Some("Point"));
        let TypeConnective::Conj { children } = &decl.connective else {
            panic!("expected Conj declaration");
        };
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].label, "x");
        assert_eq!(children[1].label, "y");
    }

    #[test]
    fn push_atom_allocates_named_atom_declaration() {
        let mut dag = Dag::new();
        let int_decl = dag
            .declaration_by_name("Int")
            .expect("bootstrap Int declaration")
            .id;
        let decl_id = dag.push_atom(
            Some("IntAlias".to_string()),
            AtomPayload::ResolvedByStructure(int_decl),
            span(),
        );

        let decl = dag.declaration(decl_id);
        assert_eq!(decl.name.as_deref(), Some("IntAlias"));
        let TypeConnective::Atom(AtomPayload::ResolvedByStructure(target)) = &decl.connective
        else {
            panic!("expected Atom(ResolvedByStructure(_)) declaration");
        };
        assert_eq!(*target, int_decl);
    }

    #[test]
    #[should_panic(expected = "push_transform(Operator) requires exactly two input ports")]
    fn push_transform_rejects_wrong_operator_arity() {
        let mut dag = Dag::new();
        let lhs = dag.push_value(LiteralBits::Int(1), span());
        let _ = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs],
            span(),
        );
    }

    #[test]
    fn push_transform_comparison_uses_bool_shape() {
        let mut dag = Dag::new();
        let lhs = dag.push_value(LiteralBits::Int(1), span());
        let rhs = dag.push_value(LiteralBits::Int(2), span());

        let output = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Lt)),
            vec![lhs, rhs],
            span(),
        );

        assert_eq!(
            dag.port(output).state(),
            &PortState::Resolved(dag.bool_shape().expect("bootstrap Bool"))
        );
    }

    #[test]
    #[should_panic(expected = "push_transform(target.field_child): unknown declaration")]
    fn push_transform_rejects_unknown_field_project_child() {
        let mut dag = Dag::new();
        let parent = dag.alloc_port_with_shape(
            dag.declaration_by_name("Dag")
                .expect("bootstrap Dag declaration")
                .id
                .into(),
        );

        let _ = dag.push_transform(
            TransformTarget::FieldProject {
                field_label: "nodes".to_string(),
                field_child: Some(DeclarationId(u32::MAX)),
            },
            vec![parent],
            span(),
        );
    }

    #[test]
    #[should_panic(expected = "push_branch(path.pattern): unknown declaration")]
    fn push_branch_rejects_unknown_resolved_variant() {
        let mut dag = Dag::new();
        let cond = dag.push_value(LiteralBits::Bool(true), span());
        let arm_output = dag.push_value(LiteralBits::Int(1), span());
        let arm_body = dag.push_bind("arm", arm_output, Vec::new(), span());

        let _ = dag.push_branch(
            cond,
            vec![Path {
                body: arm_body,
                output: arm_output,
                pattern: BranchPattern::ResolvedVariant(DeclarationId(u32::MAX)),
                binding: None,
            }],
            span(),
        );
    }
}
