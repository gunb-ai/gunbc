// AUTO-GENERATED from `dsl/std/*.dag` via `regen_bootstrap`.
// Regenerate instead of hand-editing.

pub(crate) fn bootstrapped_std_fixture_dag() -> Dag {
    Dag {
        nodes: bootstrapped_std_fixture_dag_nodes(),
        declarations: bootstrapped_std_fixture_dag_declarations(),
        ports: bootstrapped_std_fixture_dag_ports(),
        diagnostics: bootstrapped_std_fixture_dag_diagnostics(),
        next_node_id: 254,
        next_declaration_id: 693,
        next_port_id: 255,
        primitives: PrimitiveCache::default(),
        substrate_markers: SubstrateMarkers::default(),
        realization_metas: RealizationMetaCache::default(),
        target_syntax: TargetSyntaxCache::default(),
        stdlib_types: StdlibTypeCache::default(),
        emit_anchors: EmitAnchorCache::default(),
        pattern_binding_rule_variants: PatternBindingRuleVariants::default(),
        variant_payload_field_access_rule_variants: VariantPayloadFieldAccessRuleVariants::default(
        ),
        verifier_output_policy_variants: VerifierOutputPolicyVariants::default(),
        callable_strategy_variants: CallableStrategyVariants::default(),
        emit_model_variants: EmitModelVariants::default(),
        clusters: bootstrapped_std_fixture_dag_clusters(),
        optional_match_disjs: bootstrapped_std_fixture_dag_optional_match_disjs(),
        declaration_append_begin_after_bootstrap: 693,
    }
}

#[allow(clippy::vec_init_then_push)]
fn bootstrapped_std_fixture_dag_nodes() -> Vec<Behavior> {
    {
        let mut nodes = Vec::with_capacity(254);
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(0),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(1),
            span: SourceSpan::new("dsl/std/integer.dag", 9242, 9249),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(1),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Gt)),
            inputs: vec![PortId(0), PortId(1)],
            output: PortId(2),
            span: SourceSpan::new("dsl/std/integer.dag", 9242, 9249),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(2),
            name: "<registry-refinement:PositiveInt>".to_string(),
            value: PortId(2),
            params: vec![PortId(0)],
            span: SourceSpan::new("dsl/std/integer.dag", 9242, 9249),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(3),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(4),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(4),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(3), PortId(4)],
            output: PortId(5),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(5),
            data: LiteralBits::Int("55295".to_string()),
            output: PortId(6),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(6),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(3), PortId(6)],
            output: PortId(7),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(7),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(5), PortId(7)],
            output: PortId(8),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(8),
            data: LiteralBits::Int("57344".to_string()),
            output: PortId(9),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(9),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(3), PortId(9)],
            output: PortId(10),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(10),
            data: LiteralBits::Int("1114111".to_string()),
            output: PortId(11),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(11),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(3), PortId(11)],
            output: PortId(12),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(12),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(10), PortId(12)],
            output: PortId(13),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(13),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(8), PortId(13)],
            output: PortId(14),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8476),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(14),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(3), PortId(3)],
            output: PortId(15),
            span: SourceSpan::new("dsl/std/types.dag", 8478, 8491),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(15),
            data: LiteralBits::String("Char".to_string()),
            output: PortId(16),
            span: SourceSpan::new("dsl/std/types.dag", 8478, 8491),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(16),
            data: LiteralBits::String("Char".to_string()),
            output: PortId(17),
            span: SourceSpan::new("dsl/std/types.dag", 8478, 8491),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(17),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(16), PortId(17)],
            output: PortId(18),
            span: SourceSpan::new("dsl/std/types.dag", 8478, 8491),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(18),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(15), PortId(18)],
            output: PortId(19),
            span: SourceSpan::new("dsl/std/types.dag", 8478, 8491),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(19),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(14), PortId(19)],
            output: PortId(20),
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8491),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(20),
            name: "<registry-refinement:Char>".to_string(),
            value: PortId(20),
            params: vec![PortId(3)],
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8491),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(21),
            data: LiteralBits::Int("1".to_string()),
            output: PortId(22),
            span: SourceSpan::new("dsl/std/types.dag", 10887, 10908),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(22),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(21), PortId(22)],
            output: PortId(23),
            span: SourceSpan::new("dsl/std/types.dag", 10887, 10908),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(23),
            data: LiteralBits::Int("5".to_string()),
            output: PortId(24),
            span: SourceSpan::new("dsl/std/types.dag", 10887, 10908),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(24),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(21), PortId(24)],
            output: PortId(25),
            span: SourceSpan::new("dsl/std/types.dag", 10887, 10908),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(25),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(23), PortId(25)],
            output: PortId(26),
            span: SourceSpan::new("dsl/std/types.dag", 10887, 10908),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(26),
            name: "<registry-refinement:RetryCount>".to_string(),
            value: PortId(26),
            params: vec![PortId(21)],
            span: SourceSpan::new("dsl/std/types.dag", 10887, 10908),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(27),
            data: LiteralBits::Int("100".to_string()),
            output: PortId(28),
            span: SourceSpan::new("dsl/std/types.dag", 10939, 10964),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(28),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(27), PortId(28)],
            output: PortId(29),
            span: SourceSpan::new("dsl/std/types.dag", 10939, 10964),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(29),
            data: LiteralBits::Int("599".to_string()),
            output: PortId(30),
            span: SourceSpan::new("dsl/std/types.dag", 10939, 10964),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(30),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(27), PortId(30)],
            output: PortId(31),
            span: SourceSpan::new("dsl/std/types.dag", 10939, 10964),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(31),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(29), PortId(31)],
            output: PortId(32),
            span: SourceSpan::new("dsl/std/types.dag", 10939, 10964),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(32),
            name: "<registry-refinement:HttpStatus>".to_string(),
            value: PortId(32),
            params: vec![PortId(27)],
            span: SourceSpan::new("dsl/std/types.dag", 10939, 10964),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(33),
            data: LiteralBits::Int("1".to_string()),
            output: PortId(34),
            span: SourceSpan::new("dsl/std/types.dag", 11095, 11120),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(34),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(33), PortId(34)],
            output: PortId(35),
            span: SourceSpan::new("dsl/std/types.dag", 11095, 11120),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(35),
            data: LiteralBits::Int("65535".to_string()),
            output: PortId(36),
            span: SourceSpan::new("dsl/std/types.dag", 11095, 11120),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(36),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(33), PortId(36)],
            output: PortId(37),
            span: SourceSpan::new("dsl/std/types.dag", 11095, 11120),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(37),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(35), PortId(37)],
            output: PortId(38),
            span: SourceSpan::new("dsl/std/types.dag", 11095, 11120),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(38),
            name: "<registry-refinement:Port>".to_string(),
            value: PortId(38),
            params: vec![PortId(33)],
            span: SourceSpan::new("dsl/std/types.dag", 11095, 11120),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(39),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(39), PortId(39)],
            output: PortId(40),
            span: SourceSpan::new("dsl/std/types.dag", 12318, 12338),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(40),
            data: LiteralBits::String("PathSegment".to_string()),
            output: PortId(41),
            span: SourceSpan::new("dsl/std/types.dag", 12318, 12338),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(41),
            data: LiteralBits::String("PathSegment".to_string()),
            output: PortId(42),
            span: SourceSpan::new("dsl/std/types.dag", 12318, 12338),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(42),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(41), PortId(42)],
            output: PortId(43),
            span: SourceSpan::new("dsl/std/types.dag", 12318, 12338),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(43),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(40), PortId(43)],
            output: PortId(44),
            span: SourceSpan::new("dsl/std/types.dag", 12318, 12338),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(44),
            name: "<registry-refinement:PathSegment>".to_string(),
            value: PortId(44),
            params: vec![PortId(39)],
            span: SourceSpan::new("dsl/std/types.dag", 12318, 12338),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(45),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(45), PortId(45)],
            output: PortId(46),
            span: SourceSpan::new("dsl/std/types.dag", 12378, 12398),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(46),
            data: LiteralBits::String("GlobSegment".to_string()),
            output: PortId(47),
            span: SourceSpan::new("dsl/std/types.dag", 12378, 12398),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(47),
            data: LiteralBits::String("GlobSegment".to_string()),
            output: PortId(48),
            span: SourceSpan::new("dsl/std/types.dag", 12378, 12398),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(48),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(47), PortId(48)],
            output: PortId(49),
            span: SourceSpan::new("dsl/std/types.dag", 12378, 12398),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(49),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(46), PortId(49)],
            output: PortId(50),
            span: SourceSpan::new("dsl/std/types.dag", 12378, 12398),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(50),
            name: "<registry-refinement:GlobSegment>".to_string(),
            value: PortId(50),
            params: vec![PortId(45)],
            span: SourceSpan::new("dsl/std/types.dag", 12378, 12398),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(51),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(52),
            span: SourceSpan::new("dsl/std/types.dag", 13382, 13395),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(52),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(51), PortId(52)],
            output: PortId(53),
            span: SourceSpan::new("dsl/std/types.dag", 13382, 13395),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(53),
            name: "<registry-refinement:EpochMs>".to_string(),
            value: PortId(53),
            params: vec![PortId(51)],
            span: SourceSpan::new("dsl/std/types.dag", 13382, 13395),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(54),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(55),
            span: SourceSpan::new("dsl/std/types.dag", 13427, 13440),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(55),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(54), PortId(55)],
            output: PortId(56),
            span: SourceSpan::new("dsl/std/types.dag", 13427, 13440),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(56),
            name: "<registry-refinement:Duration>".to_string(),
            value: PortId(56),
            params: vec![PortId(54)],
            span: SourceSpan::new("dsl/std/types.dag", 13427, 13440),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(57),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(58),
            span: SourceSpan::new("dsl/std/types.dag", 13472, 13485),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(58),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(57), PortId(58)],
            output: PortId(59),
            span: SourceSpan::new("dsl/std/types.dag", 13472, 13485),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(59),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(57), PortId(57)],
            output: PortId(60),
            span: SourceSpan::new("dsl/std/types.dag", 13487, 13508),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(60),
            data: LiteralBits::String("Milliseconds".to_string()),
            output: PortId(61),
            span: SourceSpan::new("dsl/std/types.dag", 13487, 13508),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(61),
            data: LiteralBits::String("Milliseconds".to_string()),
            output: PortId(62),
            span: SourceSpan::new("dsl/std/types.dag", 13487, 13508),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(62),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(61), PortId(62)],
            output: PortId(63),
            span: SourceSpan::new("dsl/std/types.dag", 13487, 13508),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(63),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(60), PortId(63)],
            output: PortId(64),
            span: SourceSpan::new("dsl/std/types.dag", 13487, 13508),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(64),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(59), PortId(64)],
            output: PortId(65),
            span: SourceSpan::new("dsl/std/types.dag", 13472, 13508),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(65),
            name: "<registry-refinement:Milliseconds>".to_string(),
            value: PortId(65),
            params: vec![PortId(57)],
            span: SourceSpan::new("dsl/std/types.dag", 13472, 13508),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(66),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(67),
            span: SourceSpan::new("dsl/std/types.dag", 13540, 13553),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(67),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(66), PortId(67)],
            output: PortId(68),
            span: SourceSpan::new("dsl/std/types.dag", 13540, 13553),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(68),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(66), PortId(66)],
            output: PortId(69),
            span: SourceSpan::new("dsl/std/types.dag", 13555, 13571),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(69),
            data: LiteralBits::String("Seconds".to_string()),
            output: PortId(70),
            span: SourceSpan::new("dsl/std/types.dag", 13555, 13571),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(70),
            data: LiteralBits::String("Seconds".to_string()),
            output: PortId(71),
            span: SourceSpan::new("dsl/std/types.dag", 13555, 13571),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(71),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(70), PortId(71)],
            output: PortId(72),
            span: SourceSpan::new("dsl/std/types.dag", 13555, 13571),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(72),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(69), PortId(72)],
            output: PortId(73),
            span: SourceSpan::new("dsl/std/types.dag", 13555, 13571),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(73),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(68), PortId(73)],
            output: PortId(74),
            span: SourceSpan::new("dsl/std/types.dag", 13540, 13571),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(74),
            name: "<registry-refinement:Seconds>".to_string(),
            value: PortId(74),
            params: vec![PortId(66)],
            span: SourceSpan::new("dsl/std/types.dag", 13540, 13571),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(75),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(75), PortId(75)],
            output: PortId(76),
            span: SourceSpan::new("dsl/std/types.dag", 13777, 13797),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(76),
            data: LiteralBits::String("LogicalTime".to_string()),
            output: PortId(77),
            span: SourceSpan::new("dsl/std/types.dag", 13777, 13797),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(77),
            data: LiteralBits::String("LogicalTime".to_string()),
            output: PortId(78),
            span: SourceSpan::new("dsl/std/types.dag", 13777, 13797),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(78),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(77), PortId(78)],
            output: PortId(79),
            span: SourceSpan::new("dsl/std/types.dag", 13777, 13797),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(79),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(76), PortId(79)],
            output: PortId(80),
            span: SourceSpan::new("dsl/std/types.dag", 13777, 13797),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(80),
            name: "<registry-refinement:LogicalTime>".to_string(),
            value: PortId(80),
            params: vec![PortId(75)],
            span: SourceSpan::new("dsl/std/types.dag", 13777, 13797),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(81),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(81), PortId(81)],
            output: PortId(82),
            span: SourceSpan::new("dsl/std/types.dag", 14359, 14376),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(82),
            data: LiteralBits::String("IntentId".to_string()),
            output: PortId(83),
            span: SourceSpan::new("dsl/std/types.dag", 14359, 14376),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(83),
            data: LiteralBits::String("IntentId".to_string()),
            output: PortId(84),
            span: SourceSpan::new("dsl/std/types.dag", 14359, 14376),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(84),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(83), PortId(84)],
            output: PortId(85),
            span: SourceSpan::new("dsl/std/types.dag", 14359, 14376),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(85),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(82), PortId(85)],
            output: PortId(86),
            span: SourceSpan::new("dsl/std/types.dag", 14359, 14376),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(86),
            name: "<registry-refinement:IntentId>".to_string(),
            value: PortId(86),
            params: vec![PortId(81)],
            span: SourceSpan::new("dsl/std/types.dag", 14359, 14376),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(87),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(87), PortId(87)],
            output: PortId(88),
            span: SourceSpan::new("dsl/std/types.dag", 14415, 14431),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(88),
            data: LiteralBits::String("IssueId".to_string()),
            output: PortId(89),
            span: SourceSpan::new("dsl/std/types.dag", 14415, 14431),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(89),
            data: LiteralBits::String("IssueId".to_string()),
            output: PortId(90),
            span: SourceSpan::new("dsl/std/types.dag", 14415, 14431),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(90),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(89), PortId(90)],
            output: PortId(91),
            span: SourceSpan::new("dsl/std/types.dag", 14415, 14431),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(91),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(88), PortId(91)],
            output: PortId(92),
            span: SourceSpan::new("dsl/std/types.dag", 14415, 14431),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(92),
            name: "<registry-refinement:IssueId>".to_string(),
            value: PortId(92),
            params: vec![PortId(87)],
            span: SourceSpan::new("dsl/std/types.dag", 14415, 14431),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(93),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(93), PortId(93)],
            output: PortId(94),
            span: SourceSpan::new("dsl/std/types.dag", 14470, 14485),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(94),
            data: LiteralBits::String("RunKey".to_string()),
            output: PortId(95),
            span: SourceSpan::new("dsl/std/types.dag", 14470, 14485),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(95),
            data: LiteralBits::String("RunKey".to_string()),
            output: PortId(96),
            span: SourceSpan::new("dsl/std/types.dag", 14470, 14485),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(96),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(95), PortId(96)],
            output: PortId(97),
            span: SourceSpan::new("dsl/std/types.dag", 14470, 14485),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(97),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(94), PortId(97)],
            output: PortId(98),
            span: SourceSpan::new("dsl/std/types.dag", 14470, 14485),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(98),
            name: "<registry-refinement:RunKey>".to_string(),
            value: PortId(98),
            params: vec![PortId(93)],
            span: SourceSpan::new("dsl/std/types.dag", 14470, 14485),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(99),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(99), PortId(99)],
            output: PortId(100),
            span: SourceSpan::new("dsl/std/types.dag", 14524, 14543),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(100),
            data: LiteralBits::String("ArtifactId".to_string()),
            output: PortId(101),
            span: SourceSpan::new("dsl/std/types.dag", 14524, 14543),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(101),
            data: LiteralBits::String("ArtifactId".to_string()),
            output: PortId(102),
            span: SourceSpan::new("dsl/std/types.dag", 14524, 14543),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(102),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(101), PortId(102)],
            output: PortId(103),
            span: SourceSpan::new("dsl/std/types.dag", 14524, 14543),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(103),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(100), PortId(103)],
            output: PortId(104),
            span: SourceSpan::new("dsl/std/types.dag", 14524, 14543),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(104),
            name: "<registry-refinement:ArtifactId>".to_string(),
            value: PortId(104),
            params: vec![PortId(99)],
            span: SourceSpan::new("dsl/std/types.dag", 14524, 14543),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(105),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(105), PortId(105)],
            output: PortId(106),
            span: SourceSpan::new("dsl/std/types.dag", 14582, 14601),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(106),
            data: LiteralBits::String("LeaseToken".to_string()),
            output: PortId(107),
            span: SourceSpan::new("dsl/std/types.dag", 14582, 14601),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(107),
            data: LiteralBits::String("LeaseToken".to_string()),
            output: PortId(108),
            span: SourceSpan::new("dsl/std/types.dag", 14582, 14601),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(108),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(107), PortId(108)],
            output: PortId(109),
            span: SourceSpan::new("dsl/std/types.dag", 14582, 14601),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(109),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(106), PortId(109)],
            output: PortId(110),
            span: SourceSpan::new("dsl/std/types.dag", 14582, 14601),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(110),
            name: "<registry-refinement:LeaseToken>".to_string(),
            value: PortId(110),
            params: vec![PortId(105)],
            span: SourceSpan::new("dsl/std/types.dag", 14582, 14601),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(111),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(111), PortId(111)],
            output: PortId(112),
            span: SourceSpan::new("dsl/std/types.dag", 14640, 14657),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(112),
            data: LiteralBits::String("WorkerId".to_string()),
            output: PortId(113),
            span: SourceSpan::new("dsl/std/types.dag", 14640, 14657),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(113),
            data: LiteralBits::String("WorkerId".to_string()),
            output: PortId(114),
            span: SourceSpan::new("dsl/std/types.dag", 14640, 14657),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(114),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(113), PortId(114)],
            output: PortId(115),
            span: SourceSpan::new("dsl/std/types.dag", 14640, 14657),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(115),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(112), PortId(115)],
            output: PortId(116),
            span: SourceSpan::new("dsl/std/types.dag", 14640, 14657),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(116),
            name: "<registry-refinement:WorkerId>".to_string(),
            value: PortId(116),
            params: vec![PortId(111)],
            span: SourceSpan::new("dsl/std/types.dag", 14640, 14657),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(117),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(117), PortId(117)],
            output: PortId(118),
            span: SourceSpan::new("dsl/std/types.dag", 14696, 14714),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(118),
            data: LiteralBits::String("CommentId".to_string()),
            output: PortId(119),
            span: SourceSpan::new("dsl/std/types.dag", 14696, 14714),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(119),
            data: LiteralBits::String("CommentId".to_string()),
            output: PortId(120),
            span: SourceSpan::new("dsl/std/types.dag", 14696, 14714),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(120),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(119), PortId(120)],
            output: PortId(121),
            span: SourceSpan::new("dsl/std/types.dag", 14696, 14714),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(121),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(118), PortId(121)],
            output: PortId(122),
            span: SourceSpan::new("dsl/std/types.dag", 14696, 14714),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(122),
            name: "<registry-refinement:CommentId>".to_string(),
            value: PortId(122),
            params: vec![PortId(117)],
            span: SourceSpan::new("dsl/std/types.dag", 14696, 14714),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(123),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(123), PortId(123)],
            output: PortId(124),
            span: SourceSpan::new("dsl/std/types.dag", 14753, 14771),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(124),
            data: LiteralBits::String("SignalKey".to_string()),
            output: PortId(125),
            span: SourceSpan::new("dsl/std/types.dag", 14753, 14771),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(125),
            data: LiteralBits::String("SignalKey".to_string()),
            output: PortId(126),
            span: SourceSpan::new("dsl/std/types.dag", 14753, 14771),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(126),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(125), PortId(126)],
            output: PortId(127),
            span: SourceSpan::new("dsl/std/types.dag", 14753, 14771),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(127),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(124), PortId(127)],
            output: PortId(128),
            span: SourceSpan::new("dsl/std/types.dag", 14753, 14771),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(128),
            name: "<registry-refinement:SignalKey>".to_string(),
            value: PortId(128),
            params: vec![PortId(123)],
            span: SourceSpan::new("dsl/std/types.dag", 14753, 14771),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(129),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(129), PortId(129)],
            output: PortId(130),
            span: SourceSpan::new("dsl/std/types.dag", 14810, 14830),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(130),
            data: LiteralBits::String("ContentHash".to_string()),
            output: PortId(131),
            span: SourceSpan::new("dsl/std/types.dag", 14810, 14830),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(131),
            data: LiteralBits::String("ContentHash".to_string()),
            output: PortId(132),
            span: SourceSpan::new("dsl/std/types.dag", 14810, 14830),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(132),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(131), PortId(132)],
            output: PortId(133),
            span: SourceSpan::new("dsl/std/types.dag", 14810, 14830),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(133),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(130), PortId(133)],
            output: PortId(134),
            span: SourceSpan::new("dsl/std/types.dag", 14810, 14830),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(134),
            name: "<registry-refinement:ContentHash>".to_string(),
            value: PortId(134),
            params: vec![PortId(129)],
            span: SourceSpan::new("dsl/std/types.dag", 14810, 14830),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(135),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(135), PortId(135)],
            output: PortId(136),
            span: SourceSpan::new("dsl/std/types.dag", 15105, 15132),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(136),
            data: LiteralBits::String("WorkflowProducerId".to_string()),
            output: PortId(137),
            span: SourceSpan::new("dsl/std/types.dag", 15105, 15132),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(137),
            data: LiteralBits::String("WorkflowProducerId".to_string()),
            output: PortId(138),
            span: SourceSpan::new("dsl/std/types.dag", 15105, 15132),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(138),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(137), PortId(138)],
            output: PortId(139),
            span: SourceSpan::new("dsl/std/types.dag", 15105, 15132),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(139),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(136), PortId(139)],
            output: PortId(140),
            span: SourceSpan::new("dsl/std/types.dag", 15105, 15132),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(140),
            name: "<registry-refinement:WorkflowProducerId>".to_string(),
            value: PortId(140),
            params: vec![PortId(135)],
            span: SourceSpan::new("dsl/std/types.dag", 15105, 15132),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(141),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(141), PortId(141)],
            output: PortId(142),
            span: SourceSpan::new("dsl/std/types.dag", 15177, 15204),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(142),
            data: LiteralBits::String("WorkflowObserverId".to_string()),
            output: PortId(143),
            span: SourceSpan::new("dsl/std/types.dag", 15177, 15204),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(143),
            data: LiteralBits::String("WorkflowObserverId".to_string()),
            output: PortId(144),
            span: SourceSpan::new("dsl/std/types.dag", 15177, 15204),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(144),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(143), PortId(144)],
            output: PortId(145),
            span: SourceSpan::new("dsl/std/types.dag", 15177, 15204),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(145),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(142), PortId(145)],
            output: PortId(146),
            span: SourceSpan::new("dsl/std/types.dag", 15177, 15204),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(146),
            name: "<registry-refinement:WorkflowObserverId>".to_string(),
            value: PortId(146),
            params: vec![PortId(141)],
            span: SourceSpan::new("dsl/std/types.dag", 15177, 15204),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(147),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(147), PortId(147)],
            output: PortId(148),
            span: SourceSpan::new("dsl/std/types.dag", 15247, 15272),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(148),
            data: LiteralBits::String("WorkflowProverId".to_string()),
            output: PortId(149),
            span: SourceSpan::new("dsl/std/types.dag", 15247, 15272),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(149),
            data: LiteralBits::String("WorkflowProverId".to_string()),
            output: PortId(150),
            span: SourceSpan::new("dsl/std/types.dag", 15247, 15272),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(150),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(149), PortId(150)],
            output: PortId(151),
            span: SourceSpan::new("dsl/std/types.dag", 15247, 15272),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(151),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(148), PortId(151)],
            output: PortId(152),
            span: SourceSpan::new("dsl/std/types.dag", 15247, 15272),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(152),
            name: "<registry-refinement:WorkflowProverId>".to_string(),
            value: PortId(152),
            params: vec![PortId(147)],
            span: SourceSpan::new("dsl/std/types.dag", 15247, 15272),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(153),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(153), PortId(153)],
            output: PortId(154),
            span: SourceSpan::new("dsl/std/types.dag", 15312, 15334),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(154),
            data: LiteralBits::String("WorkflowRunId".to_string()),
            output: PortId(155),
            span: SourceSpan::new("dsl/std/types.dag", 15312, 15334),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(155),
            data: LiteralBits::String("WorkflowRunId".to_string()),
            output: PortId(156),
            span: SourceSpan::new("dsl/std/types.dag", 15312, 15334),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(156),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(155), PortId(156)],
            output: PortId(157),
            span: SourceSpan::new("dsl/std/types.dag", 15312, 15334),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(157),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(154), PortId(157)],
            output: PortId(158),
            span: SourceSpan::new("dsl/std/types.dag", 15312, 15334),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(158),
            name: "<registry-refinement:WorkflowRunId>".to_string(),
            value: PortId(158),
            params: vec![PortId(153)],
            span: SourceSpan::new("dsl/std/types.dag", 15312, 15334),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(159),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(159), PortId(159)],
            output: PortId(160),
            span: SourceSpan::new("dsl/std/types.dag", 21921, 21946),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(160),
            data: LiteralBits::String("FilesystemHandle".to_string()),
            output: PortId(161),
            span: SourceSpan::new("dsl/std/types.dag", 21921, 21946),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(161),
            data: LiteralBits::String("FilesystemHandle".to_string()),
            output: PortId(162),
            span: SourceSpan::new("dsl/std/types.dag", 21921, 21946),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(162),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(161), PortId(162)],
            output: PortId(163),
            span: SourceSpan::new("dsl/std/types.dag", 21921, 21946),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(163),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(160), PortId(163)],
            output: PortId(164),
            span: SourceSpan::new("dsl/std/types.dag", 21921, 21946),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(164),
            name: "<registry-refinement:FilesystemHandle>".to_string(),
            value: PortId(164),
            params: vec![PortId(159)],
            span: SourceSpan::new("dsl/std/types.dag", 21921, 21946),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(165),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(165), PortId(165)],
            output: PortId(166),
            span: SourceSpan::new("dsl/std/types.dag", 21982, 22004),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(166),
            data: LiteralBits::String("NetworkHandle".to_string()),
            output: PortId(167),
            span: SourceSpan::new("dsl/std/types.dag", 21982, 22004),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(167),
            data: LiteralBits::String("NetworkHandle".to_string()),
            output: PortId(168),
            span: SourceSpan::new("dsl/std/types.dag", 21982, 22004),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(168),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(167), PortId(168)],
            output: PortId(169),
            span: SourceSpan::new("dsl/std/types.dag", 21982, 22004),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(169),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(166), PortId(169)],
            output: PortId(170),
            span: SourceSpan::new("dsl/std/types.dag", 21982, 22004),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(170),
            name: "<registry-refinement:NetworkHandle>".to_string(),
            value: PortId(170),
            params: vec![PortId(165)],
            span: SourceSpan::new("dsl/std/types.dag", 21982, 22004),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(171),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(171), PortId(171)],
            output: PortId(172),
            span: SourceSpan::new("dsl/std/types.dag", 22042, 22061),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(172),
            data: LiteralBits::String("ToolHandle".to_string()),
            output: PortId(173),
            span: SourceSpan::new("dsl/std/types.dag", 22042, 22061),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(173),
            data: LiteralBits::String("ToolHandle".to_string()),
            output: PortId(174),
            span: SourceSpan::new("dsl/std/types.dag", 22042, 22061),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(174),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(173), PortId(174)],
            output: PortId(175),
            span: SourceSpan::new("dsl/std/types.dag", 22042, 22061),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(175),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(172), PortId(175)],
            output: PortId(176),
            span: SourceSpan::new("dsl/std/types.dag", 22042, 22061),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(176),
            name: "<registry-refinement:ToolHandle>".to_string(),
            value: PortId(176),
            params: vec![PortId(171)],
            span: SourceSpan::new("dsl/std/types.dag", 22042, 22061),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(177),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(179),
            span: SourceSpan::new("dsl/std/unicode.dag", 4240, 4253),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(178),
            data: LiteralBits::Int("9".to_string()),
            output: PortId(180),
            span: SourceSpan::new("dsl/std/unicode.dag", 4257, 4258),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(179),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(179), PortId(180)],
            output: PortId(181),
            span: SourceSpan::new("dsl/std/unicode.dag", 4240, 4258),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(180),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(182),
            span: SourceSpan::new("dsl/std/unicode.dag", 4262, 4275),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(181),
            data: LiteralBits::Int("10".to_string()),
            output: PortId(183),
            span: SourceSpan::new("dsl/std/unicode.dag", 4279, 4281),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(182),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(182), PortId(183)],
            output: PortId(184),
            span: SourceSpan::new("dsl/std/unicode.dag", 4262, 4281),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(183),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(181), PortId(184)],
            output: PortId(185),
            span: SourceSpan::new("dsl/std/unicode.dag", 4240, 4281),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(184),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(186),
            span: SourceSpan::new("dsl/std/unicode.dag", 4285, 4298),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(185),
            data: LiteralBits::Int("12".to_string()),
            output: PortId(187),
            span: SourceSpan::new("dsl/std/unicode.dag", 4302, 4304),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(186),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(186), PortId(187)],
            output: PortId(188),
            span: SourceSpan::new("dsl/std/unicode.dag", 4285, 4304),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(187),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(185), PortId(188)],
            output: PortId(189),
            span: SourceSpan::new("dsl/std/unicode.dag", 4240, 4304),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(188),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(190),
            span: SourceSpan::new("dsl/std/unicode.dag", 4314, 4327),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(189),
            data: LiteralBits::Int("13".to_string()),
            output: PortId(191),
            span: SourceSpan::new("dsl/std/unicode.dag", 4331, 4333),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(190),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(190), PortId(191)],
            output: PortId(192),
            span: SourceSpan::new("dsl/std/unicode.dag", 4314, 4333),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(191),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(189), PortId(192)],
            output: PortId(193),
            span: SourceSpan::new("dsl/std/unicode.dag", 4240, 4333),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(192),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(194),
            span: SourceSpan::new("dsl/std/unicode.dag", 4337, 4350),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(193),
            data: LiteralBits::Int("32".to_string()),
            output: PortId(195),
            span: SourceSpan::new("dsl/std/unicode.dag", 4354, 4356),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(194),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(194), PortId(195)],
            output: PortId(196),
            span: SourceSpan::new("dsl/std/unicode.dag", 4337, 4356),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(195),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(193), PortId(196)],
            output: PortId(197),
            span: SourceSpan::new("dsl/std/unicode.dag", 4240, 4356),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(196),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(198),
            span: SourceSpan::new("dsl/std/unicode.dag", 4370, 4383),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(197),
            data: LiteralBits::Int("48".to_string()),
            output: PortId(199),
            span: SourceSpan::new("dsl/std/unicode.dag", 4387, 4389),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(198),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(198), PortId(199)],
            output: PortId(200),
            span: SourceSpan::new("dsl/std/unicode.dag", 4370, 4389),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(199),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(201),
            span: SourceSpan::new("dsl/std/unicode.dag", 4393, 4406),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(200),
            data: LiteralBits::Int("57".to_string()),
            output: PortId(202),
            span: SourceSpan::new("dsl/std/unicode.dag", 4410, 4412),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(201),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(201), PortId(202)],
            output: PortId(203),
            span: SourceSpan::new("dsl/std/unicode.dag", 4393, 4412),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(202),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(200), PortId(203)],
            output: PortId(204),
            span: SourceSpan::new("dsl/std/unicode.dag", 4370, 4412),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(203),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(205),
            span: SourceSpan::new("dsl/std/unicode.dag", 4437, 4450),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(204),
            data: LiteralBits::Int("65".to_string()),
            output: PortId(206),
            span: SourceSpan::new("dsl/std/unicode.dag", 4454, 4456),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(205),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(205), PortId(206)],
            output: PortId(207),
            span: SourceSpan::new("dsl/std/unicode.dag", 4437, 4456),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(206),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(208),
            span: SourceSpan::new("dsl/std/unicode.dag", 4460, 4473),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(207),
            data: LiteralBits::Int("90".to_string()),
            output: PortId(209),
            span: SourceSpan::new("dsl/std/unicode.dag", 4477, 4479),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(208),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(208), PortId(209)],
            output: PortId(210),
            span: SourceSpan::new("dsl/std/unicode.dag", 4460, 4479),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(209),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(207), PortId(210)],
            output: PortId(211),
            span: SourceSpan::new("dsl/std/unicode.dag", 4437, 4479),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(210),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(212),
            span: SourceSpan::new("dsl/std/unicode.dag", 4489, 4502),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(211),
            data: LiteralBits::Int("97".to_string()),
            output: PortId(213),
            span: SourceSpan::new("dsl/std/unicode.dag", 4506, 4508),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(212),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(212), PortId(213)],
            output: PortId(214),
            span: SourceSpan::new("dsl/std/unicode.dag", 4489, 4508),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(213),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(215),
            span: SourceSpan::new("dsl/std/unicode.dag", 4512, 4525),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(214),
            data: LiteralBits::Int("122".to_string()),
            output: PortId(216),
            span: SourceSpan::new("dsl/std/unicode.dag", 4529, 4532),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(215),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(215), PortId(216)],
            output: PortId(217),
            span: SourceSpan::new("dsl/std/unicode.dag", 4512, 4532),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(216),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(214), PortId(217)],
            output: PortId(218),
            span: SourceSpan::new("dsl/std/unicode.dag", 4489, 4532),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(217),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(211), PortId(218)],
            output: PortId(219),
            span: SourceSpan::new("dsl/std/unicode.dag", 4437, 4532),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(218),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(220),
            span: SourceSpan::new("dsl/std/unicode.dag", 4542, 4555),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(219),
            data: LiteralBits::Int("95".to_string()),
            output: PortId(221),
            span: SourceSpan::new("dsl/std/unicode.dag", 4559, 4561),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(220),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(220), PortId(221)],
            output: PortId(222),
            span: SourceSpan::new("dsl/std/unicode.dag", 4542, 4561),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(221),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(219), PortId(222)],
            output: PortId(223),
            span: SourceSpan::new("dsl/std/unicode.dag", 4437, 4561),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(222),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(224),
            span: SourceSpan::new("dsl/std/unicode.dag", 4746, 4759),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(223),
            data: LiteralBits::Int("48".to_string()),
            output: PortId(225),
            span: SourceSpan::new("dsl/std/unicode.dag", 4763, 4765),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(224),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(224), PortId(225)],
            output: PortId(226),
            span: SourceSpan::new("dsl/std/unicode.dag", 4746, 4765),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(225),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(227),
            span: SourceSpan::new("dsl/std/unicode.dag", 4769, 4782),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(226),
            data: LiteralBits::Int("57".to_string()),
            output: PortId(228),
            span: SourceSpan::new("dsl/std/unicode.dag", 4786, 4788),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(227),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(227), PortId(228)],
            output: PortId(229),
            span: SourceSpan::new("dsl/std/unicode.dag", 4769, 4788),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(228),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(226), PortId(229)],
            output: PortId(230),
            span: SourceSpan::new("dsl/std/unicode.dag", 4746, 4788),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(229),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(231),
            span: SourceSpan::new("dsl/std/unicode.dag", 4798, 4811),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(230),
            data: LiteralBits::Int("65".to_string()),
            output: PortId(232),
            span: SourceSpan::new("dsl/std/unicode.dag", 4815, 4817),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(231),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(231), PortId(232)],
            output: PortId(233),
            span: SourceSpan::new("dsl/std/unicode.dag", 4798, 4817),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(232),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(234),
            span: SourceSpan::new("dsl/std/unicode.dag", 4821, 4834),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(233),
            data: LiteralBits::Int("90".to_string()),
            output: PortId(235),
            span: SourceSpan::new("dsl/std/unicode.dag", 4838, 4840),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(234),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(234), PortId(235)],
            output: PortId(236),
            span: SourceSpan::new("dsl/std/unicode.dag", 4821, 4840),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(235),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(233), PortId(236)],
            output: PortId(237),
            span: SourceSpan::new("dsl/std/unicode.dag", 4798, 4840),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(236),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(230), PortId(237)],
            output: PortId(238),
            span: SourceSpan::new("dsl/std/unicode.dag", 4746, 4840),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(237),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(239),
            span: SourceSpan::new("dsl/std/unicode.dag", 4850, 4863),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(238),
            data: LiteralBits::Int("97".to_string()),
            output: PortId(240),
            span: SourceSpan::new("dsl/std/unicode.dag", 4867, 4869),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(239),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(239), PortId(240)],
            output: PortId(241),
            span: SourceSpan::new("dsl/std/unicode.dag", 4850, 4869),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(240),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(242),
            span: SourceSpan::new("dsl/std/unicode.dag", 4873, 4886),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(241),
            data: LiteralBits::Int("122".to_string()),
            output: PortId(243),
            span: SourceSpan::new("dsl/std/unicode.dag", 4890, 4893),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(242),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(242), PortId(243)],
            output: PortId(244),
            span: SourceSpan::new("dsl/std/unicode.dag", 4873, 4893),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(243),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(241), PortId(244)],
            output: PortId(245),
            span: SourceSpan::new("dsl/std/unicode.dag", 4850, 4893),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(244),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(238), PortId(245)],
            output: PortId(246),
            span: SourceSpan::new("dsl/std/unicode.dag", 4746, 4893),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(245),
            target: TransformTarget::Callable(DeclarationId(234)),
            inputs: vec![PortId(177)],
            output: PortId(247),
            span: SourceSpan::new("dsl/std/unicode.dag", 4903, 4916),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(246),
            data: LiteralBits::Int("95".to_string()),
            output: PortId(248),
            span: SourceSpan::new("dsl/std/unicode.dag", 4920, 4922),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(247),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Eq)),
            inputs: vec![PortId(247), PortId(248)],
            output: PortId(249),
            span: SourceSpan::new("dsl/std/unicode.dag", 4903, 4922),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(248),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::Or)),
            inputs: vec![PortId(246), PortId(249)],
            output: PortId(250),
            span: SourceSpan::new("dsl/std/unicode.dag", 4746, 4922),
        }));
        nodes.push(Behavior::Branch(BranchNode {
            id: NodeId(249),
            input: PortId(178),
            paths: vec![
                Path {
                    body: NodeId(195),
                    output: PortId(197),
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Whitespace".to_string(),
                        span: SourceSpan::new("dsl/std/unicode.dag", 4220, 4230),
                    },
                    binding: None,
                },
                Path {
                    body: NodeId(202),
                    output: PortId(204),
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Digit".to_string(),
                        span: SourceSpan::new("dsl/std/unicode.dag", 4361, 4366),
                    },
                    binding: None,
                },
                Path {
                    body: NodeId(221),
                    output: PortId(223),
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "IdentStart".to_string(),
                        span: SourceSpan::new("dsl/std/unicode.dag", 4417, 4427),
                    },
                    binding: None,
                },
                Path {
                    body: NodeId(248),
                    output: PortId(250),
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "IdentContinue".to_string(),
                        span: SourceSpan::new("dsl/std/unicode.dag", 4723, 4736),
                    },
                    binding: None,
                },
            ],
            output: PortId(251),
            span: SourceSpan::new("dsl/std/unicode.dag", 4115, 4926),
            emit_participation: Some(BranchEmitParticipation::UserMatch),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(250),
            name: "char_in_class".to_string(),
            value: PortId(251),
            params: vec![PortId(177), PortId(178)],
            span: SourceSpan::new("dsl/std/unicode.dag", 4115, 4926),
            lane2_workflow: None,
            emit_participation: Some(BindEmitParticipation::UserCallable),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(251),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(253),
            span: SourceSpan::new("dsl/std/unicode.dag", 8909, 8910),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(252),
            target: TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            inputs: vec![PortId(252), PortId(253)],
            output: PortId(254),
            span: SourceSpan::new("dsl/std/unicode.dag", 8905, 8910),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(253),
            name: "code_point".to_string(),
            value: PortId(254),
            params: vec![PortId(252)],
            span: SourceSpan::new("dsl/std/unicode.dag", 8905, 8910),
            lane2_workflow: None,
            emit_participation: Some(BindEmitParticipation::UserCallable),
        }));
        nodes
    }
}

#[allow(clippy::vec_init_then_push)]
fn bootstrapped_std_fixture_dag_declarations() -> Vec<Declaration> {
    {
        let mut declarations = Vec::with_capacity(693);
        declarations.push(Declaration {
            id: DeclarationId(0),
            name: Some("Classical".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "True".to_string(),
                        ty: DeclarationId(311),
                    },
                    Field {
                        label: "False".to_string(),
                        ty: DeclarationId(312),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/logic.dag", 287, 316),
        });
        declarations.push(Declaration {
            id: DeclarationId(1),
            name: Some("classical_not".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(0)],
                output: DeclarationId(0),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/logic.dag", 362, 429)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/logic.dag", 318, 429),
        });
        declarations.push(Declaration {
            id: DeclarationId(2),
            name: Some("classical_and".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(0), DeclarationId(0)],
                output: DeclarationId(0),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/logic.dag", 489, 553)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/logic.dag", 431, 553),
        });
        declarations.push(Declaration {
            id: DeclarationId(3),
            name: Some("classical_or".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(0), DeclarationId(0)],
                output: DeclarationId(0),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/logic.dag", 612, 675)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/logic.dag", 555, 675),
        });
        declarations.push(Declaration {
            id: DeclarationId(4),
            name: Some("Bit".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(0),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 897, 917),
        });
        declarations.push(Declaration {
            id: DeclarationId(5),
            name: Some("Nibble".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "bits".to_string(),
                    ty: DeclarationId(313),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1003, 1034),
        });
        declarations.push(Declaration {
            id: DeclarationId(6),
            name: Some("Byte".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "bits".to_string(),
                    ty: DeclarationId(314),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1118, 1147),
        });
        declarations.push(Declaration {
            id: DeclarationId(7),
            name: Some("Word16".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "bytes".to_string(),
                    ty: DeclarationId(315),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1251, 1284),
        });
        declarations.push(Declaration {
            id: DeclarationId(8),
            name: Some("Word32".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "bytes".to_string(),
                    ty: DeclarationId(316),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1285, 1318),
        });
        declarations.push(Declaration {
            id: DeclarationId(9),
            name: Some("Word64".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "bytes".to_string(),
                    ty: DeclarationId(317),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1319, 1352),
        });
        declarations.push(Declaration {
            id: DeclarationId(10),
            name: Some("Word128".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "bytes".to_string(),
                    ty: DeclarationId(318),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1353, 1387),
        });
        declarations.push(Declaration {
            id: DeclarationId(11),
            name: Some("Result".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Ok".to_string(),
                        ty: DeclarationId(319),
                    },
                    Field {
                        label: "Err".to_string(),
                        ty: DeclarationId(320),
                    },
                ],
            },
            type_params: vec![DeclarationId(12), DeclarationId(13)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/error_primitives.dag", 597, 657),
        });
        declarations.push(Declaration {
            id: DeclarationId(12),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("ok".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/error_primitives.dag", 597, 657),
        });
        declarations.push(Declaration {
            id: DeclarationId(13),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("err".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/error_primitives.dag", 597, 657),
        });
        declarations.push(Declaration {
            id: DeclarationId(14),
            name: Some("DivError".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "DivideByZero".to_string(),
                        ty: DeclarationId(321),
                    },
                    Field {
                        label: "Overflow".to_string(),
                        ty: DeclarationId(322),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/error_primitives.dag", 920, 959),
        });
        declarations.push(Declaration {
            id: DeclarationId(15),
            name: Some("Magma".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "op".to_string(),
                    ty: DeclarationId(332),
                }],
            },
            type_params: vec![DeclarationId(16)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 4702, 4739),
        });
        declarations.push(Declaration {
            id: DeclarationId(16),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 4702, 4739),
        });
        declarations.push(Declaration {
            id: DeclarationId(17),
            name: Some("Semigroup".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "op".to_string(),
                    ty: DeclarationId(333),
                }],
            },
            type_params: vec![DeclarationId(18)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 4817, 4939),
        });
        declarations.push(Declaration {
            id: DeclarationId(18),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 4817, 4939),
        });
        declarations.push(Declaration {
            id: DeclarationId(19),
            name: Some("Monoid".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "op".to_string(),
                        ty: DeclarationId(334),
                    },
                    Field {
                        label: "identity".to_string(),
                        ty: DeclarationId(20),
                    },
                ],
            },
            type_params: vec![DeclarationId(20)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5037, 5089),
        });
        declarations.push(Declaration {
            id: DeclarationId(20),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5037, 5089),
        });
        declarations.push(Declaration {
            id: DeclarationId(21),
            name: Some("CommutativeMonoid".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "op".to_string(),
                        ty: DeclarationId(335),
                    },
                    Field {
                        label: "identity".to_string(),
                        ty: DeclarationId(22),
                    },
                ],
            },
            type_params: vec![DeclarationId(22)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5148, 5239),
        });
        declarations.push(Declaration {
            id: DeclarationId(22),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5148, 5239),
        });
        declarations.push(Declaration {
            id: DeclarationId(23),
            name: Some("Group".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "op".to_string(),
                        ty: DeclarationId(336),
                    },
                    Field {
                        label: "identity".to_string(),
                        ty: DeclarationId(24),
                    },
                    Field {
                        label: "inverse".to_string(),
                        ty: DeclarationId(337),
                    },
                ],
            },
            type_params: vec![DeclarationId(24)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5311, 5384),
        });
        declarations.push(Declaration {
            id: DeclarationId(24),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5311, 5384),
        });
        declarations.push(Declaration {
            id: DeclarationId(25),
            name: Some("AbelianGroup".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "op".to_string(),
                        ty: DeclarationId(338),
                    },
                    Field {
                        label: "identity".to_string(),
                        ty: DeclarationId(26),
                    },
                    Field {
                        label: "inverse".to_string(),
                        ty: DeclarationId(339),
                    },
                ],
            },
            type_params: vec![DeclarationId(26)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5422, 5530),
        });
        declarations.push(Declaration {
            id: DeclarationId(26),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5422, 5530),
        });
        declarations.push(Declaration {
            id: DeclarationId(27),
            name: Some("GroupCompletion".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![DeclarationId(28)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 7957, 7977),
        });
        declarations.push(Declaration {
            id: DeclarationId(28),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("M".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 7957, 7977),
        });
        declarations.push(Declaration {
            id: DeclarationId(29),
            name: Some("FieldOfFractions".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![DeclarationId(30)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 10044, 10065),
        });
        declarations.push(Declaration {
            id: DeclarationId(30),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("R".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 10044, 10065),
        });
        declarations.push(Declaration {
            id: DeclarationId(31),
            name: Some("Semiring".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "add".to_string(),
                        ty: DeclarationId(340),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(32),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(341),
                    },
                    Field {
                        label: "one".to_string(),
                        ty: DeclarationId(32),
                    },
                ],
            },
            type_params: vec![DeclarationId(32)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 10502, 10583),
        });
        declarations.push(Declaration {
            id: DeclarationId(32),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 10502, 10583),
        });
        declarations.push(Declaration {
            id: DeclarationId(33),
            name: Some("CommutativeSemiring".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "add".to_string(),
                        ty: DeclarationId(342),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(34),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(343),
                    },
                    Field {
                        label: "one".to_string(),
                        ty: DeclarationId(34),
                    },
                ],
            },
            type_params: vec![DeclarationId(34)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 10977, 11069),
        });
        declarations.push(Declaration {
            id: DeclarationId(34),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 10977, 11069),
        });
        declarations.push(Declaration {
            id: DeclarationId(35),
            name: Some("Ring".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "add".to_string(),
                        ty: DeclarationId(344),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(36),
                    },
                    Field {
                        label: "negate".to_string(),
                        ty: DeclarationId(345),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(346),
                    },
                    Field {
                        label: "one".to_string(),
                        ty: DeclarationId(36),
                    },
                ],
            },
            type_params: vec![DeclarationId(36)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 11196, 11294),
        });
        declarations.push(Declaration {
            id: DeclarationId(36),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 11196, 11294),
        });
        declarations.push(Declaration {
            id: DeclarationId(37),
            name: Some("OrderedRing".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "add".to_string(),
                        ty: DeclarationId(347),
                    },
                    Field {
                        label: "sub".to_string(),
                        ty: DeclarationId(348),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(38),
                    },
                    Field {
                        label: "negate".to_string(),
                        ty: DeclarationId(349),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(350),
                    },
                    Field {
                        label: "div".to_string(),
                        ty: DeclarationId(352),
                    },
                    Field {
                        label: "one".to_string(),
                        ty: DeclarationId(38),
                    },
                    Field {
                        label: "compare".to_string(),
                        ty: DeclarationId(353),
                    },
                    Field {
                        label: "eq".to_string(),
                        ty: DeclarationId(354),
                    },
                    Field {
                        label: "ne".to_string(),
                        ty: DeclarationId(355),
                    },
                    Field {
                        label: "lt".to_string(),
                        ty: DeclarationId(356),
                    },
                    Field {
                        label: "le".to_string(),
                        ty: DeclarationId(357),
                    },
                    Field {
                        label: "gt".to_string(),
                        ty: DeclarationId(358),
                    },
                    Field {
                        label: "ge".to_string(),
                        ty: DeclarationId(359),
                    },
                ],
            },
            type_params: vec![DeclarationId(38)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12055, 12390),
        });
        declarations.push(Declaration {
            id: DeclarationId(38),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12055, 12390),
        });
        declarations.push(Declaration {
            id: DeclarationId(39),
            name: Some("Field".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "add".to_string(),
                        ty: DeclarationId(360),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(40),
                    },
                    Field {
                        label: "negate".to_string(),
                        ty: DeclarationId(361),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(362),
                    },
                    Field {
                        label: "one".to_string(),
                        ty: DeclarationId(40),
                    },
                    Field {
                        label: "reciprocal".to_string(),
                        ty: DeclarationId(363),
                    },
                    Field {
                        label: "compare".to_string(),
                        ty: DeclarationId(364),
                    },
                ],
            },
            type_params: vec![DeclarationId(40)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12528, 12684),
        });
        declarations.push(Declaration {
            id: DeclarationId(40),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12528, 12684),
        });
        declarations.push(Declaration {
            id: DeclarationId(41),
            name: Some("Lattice".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "meet".to_string(),
                        ty: DeclarationId(365),
                    },
                    Field {
                        label: "join".to_string(),
                        ty: DeclarationId(366),
                    },
                ],
            },
            type_params: vec![DeclarationId(42)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13004, 13067),
        });
        declarations.push(Declaration {
            id: DeclarationId(42),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13004, 13067),
        });
        declarations.push(Declaration {
            id: DeclarationId(43),
            name: Some("BoundedLattice".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "meet".to_string(),
                        ty: DeclarationId(367),
                    },
                    Field {
                        label: "join".to_string(),
                        ty: DeclarationId(368),
                    },
                    Field {
                        label: "top".to_string(),
                        ty: DeclarationId(44),
                    },
                    Field {
                        label: "bottom".to_string(),
                        ty: DeclarationId(44),
                    },
                ],
            },
            type_params: vec![DeclarationId(44)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13175, 13266),
        });
        declarations.push(Declaration {
            id: DeclarationId(44),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13175, 13266),
        });
        declarations.push(Declaration {
            id: DeclarationId(45),
            name: Some("BooleanAlgebra".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "meet".to_string(),
                        ty: DeclarationId(369),
                    },
                    Field {
                        label: "join".to_string(),
                        ty: DeclarationId(370),
                    },
                    Field {
                        label: "complement".to_string(),
                        ty: DeclarationId(371),
                    },
                    Field {
                        label: "top".to_string(),
                        ty: DeclarationId(46),
                    },
                    Field {
                        label: "bottom".to_string(),
                        ty: DeclarationId(46),
                    },
                ],
            },
            type_params: vec![DeclarationId(46)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13464, 13580),
        });
        declarations.push(Declaration {
            id: DeclarationId(46),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13464, 13580),
        });
        declarations.push(Declaration {
            id: DeclarationId(47),
            name: Some("FreeMonoid".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "concat".to_string(),
                        ty: DeclarationId(375),
                    },
                    Field {
                        label: "empty".to_string(),
                        ty: DeclarationId(376),
                    },
                    Field {
                        label: "append".to_string(),
                        ty: DeclarationId(378),
                    },
                    Field {
                        label: "slice".to_string(),
                        ty: DeclarationId(380),
                    },
                    Field {
                        label: "length".to_string(),
                        ty: DeclarationId(381),
                    },
                    Field {
                        label: "is_empty".to_string(),
                        ty: DeclarationId(382),
                    },
                    Field {
                        label: "count".to_string(),
                        ty: DeclarationId(383),
                    },
                    Field {
                        label: "first".to_string(),
                        ty: DeclarationId(385),
                    },
                    Field {
                        label: "last".to_string(),
                        ty: DeclarationId(387),
                    },
                    Field {
                        label: "map".to_string(),
                        ty: DeclarationId(390),
                    },
                    Field {
                        label: "filter".to_string(),
                        ty: DeclarationId(393),
                    },
                    Field {
                        label: "fold".to_string(),
                        ty: DeclarationId(395),
                    },
                    Field {
                        label: "flat_map".to_string(),
                        ty: DeclarationId(399),
                    },
                    Field {
                        label: "any".to_string(),
                        ty: DeclarationId(401),
                    },
                    Field {
                        label: "all".to_string(),
                        ty: DeclarationId(403),
                    },
                    Field {
                        label: "enumerate".to_string(),
                        ty: DeclarationId(407),
                    },
                    Field {
                        label: "reverse".to_string(),
                        ty: DeclarationId(409),
                    },
                    Field {
                        label: "skip".to_string(),
                        ty: DeclarationId(411),
                    },
                    Field {
                        label: "take".to_string(),
                        ty: DeclarationId(413),
                    },
                    Field {
                        label: "sort_by".to_string(),
                        ty: DeclarationId(416),
                    },
                    Field {
                        label: "contains".to_string(),
                        ty: DeclarationId(417),
                    },
                ],
            },
            type_params: vec![DeclarationId(48)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17472, 18434),
        });
        declarations.push(Declaration {
            id: DeclarationId(48),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17472, 18434),
        });
        declarations.push(Declaration {
            id: DeclarationId(49),
            name: Some("PartialFunction".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "lookup".to_string(),
                        ty: DeclarationId(419),
                    },
                    Field {
                        label: "empty".to_string(),
                        ty: DeclarationId(420),
                    },
                    Field {
                        label: "get".to_string(),
                        ty: DeclarationId(422),
                    },
                    Field {
                        label: "insert".to_string(),
                        ty: DeclarationId(424),
                    },
                    Field {
                        label: "merge".to_string(),
                        ty: DeclarationId(427),
                    },
                    Field {
                        label: "keys".to_string(),
                        ty: DeclarationId(429),
                    },
                    Field {
                        label: "values".to_string(),
                        ty: DeclarationId(431),
                    },
                    Field {
                        label: "has".to_string(),
                        ty: DeclarationId(432),
                    },
                    Field {
                        label: "contains_key".to_string(),
                        ty: DeclarationId(433),
                    },
                    Field {
                        label: "size".to_string(),
                        ty: DeclarationId(434),
                    },
                ],
            },
            type_params: vec![DeclarationId(50), DeclarationId(51)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18949, 19288),
        });
        declarations.push(Declaration {
            id: DeclarationId(50),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("K".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18949, 19288),
        });
        declarations.push(Declaration {
            id: DeclarationId(51),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("V".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18949, 19288),
        });
        declarations.push(Declaration {
            id: DeclarationId(52),
            name: Some("Ordering".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Less".to_string(),
                        ty: DeclarationId(435),
                    },
                    Field {
                        label: "Equal".to_string(),
                        ty: DeclarationId(436),
                    },
                    Field {
                        label: "Greater".to_string(),
                        ty: DeclarationId(437),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19489, 19527),
        });
        declarations.push(Declaration {
            id: DeclarationId(53),
            name: Some("AlgebraProfile".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "OrderedRingProfile".to_string(),
                        ty: DeclarationId(438),
                    },
                    Field {
                        label: "ApproximateFieldProfile".to_string(),
                        ty: DeclarationId(439),
                    },
                    Field {
                        label: "BooleanAlgebraProfile".to_string(),
                        ty: DeclarationId(440),
                    },
                    Field {
                        label: "BooleanAlgebraCollectionProfile".to_string(),
                        ty: DeclarationId(441),
                    },
                    Field {
                        label: "FreeMonoidScalarProfile".to_string(),
                        ty: DeclarationId(442),
                    },
                    Field {
                        label: "FreeMonoidCollectionProfile".to_string(),
                        ty: DeclarationId(443),
                    },
                    Field {
                        label: "PartialFunctionProfile".to_string(),
                        ty: DeclarationId(444),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21612, 21831),
        });
        declarations.push(Declaration {
            id: DeclarationId(54),
            name: Some("ContainerSource".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "SameAsReceiver".to_string(),
                        ty: DeclarationId(445),
                    },
                    Field {
                        label: "Named".to_string(),
                        ty: DeclarationId(446),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21833, 21899),
        });
        declarations.push(Declaration {
            id: DeclarationId(55),
            name: Some("AlgebraTypeTemplate".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "ReceiverSelf".to_string(),
                        ty: DeclarationId(447),
                    },
                    Field {
                        label: "ReceiverElement".to_string(),
                        ty: DeclarationId(448),
                    },
                    Field {
                        label: "ReceiverKey".to_string(),
                        ty: DeclarationId(449),
                    },
                    Field {
                        label: "ReceiverValue".to_string(),
                        ty: DeclarationId(450),
                    },
                    Field {
                        label: "NamedTemplate".to_string(),
                        ty: DeclarationId(451),
                    },
                    Field {
                        label: "ContainerOf".to_string(),
                        ty: DeclarationId(452),
                    },
                    Field {
                        label: "OptionalOf".to_string(),
                        ty: DeclarationId(453),
                    },
                    Field {
                        label: "TupleOf".to_string(),
                        ty: DeclarationId(454),
                    },
                    Field {
                        label: "CallableOf".to_string(),
                        ty: DeclarationId(456),
                    },
                    Field {
                        label: "AlgebraTypeVariable".to_string(),
                        ty: DeclarationId(457),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21901, 22349),
        });
        declarations.push(Declaration {
            id: DeclarationId(56),
            name: Some("CollectionSizeEffect".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "ShrinkEffect".to_string(),
                        ty: DeclarationId(458),
                    },
                    Field {
                        label: "ProjectionEffect".to_string(),
                        ty: DeclarationId(459),
                    },
                    Field {
                        label: "IdentityEffect".to_string(),
                        ty: DeclarationId(460),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22565, 22742),
        });
        declarations.push(Declaration {
            id: DeclarationId(57),
            name: Some("CostShape".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "ShapeConstant".to_string(),
                        ty: DeclarationId(461),
                    },
                    Field {
                        label: "ShapeLinearScan".to_string(),
                        ty: DeclarationId(462),
                    },
                    Field {
                        label: "ShapeIterateBody".to_string(),
                        ty: DeclarationId(463),
                    },
                    Field {
                        label: "ShapeSortBody".to_string(),
                        ty: DeclarationId(464),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23200, 23385),
        });
        declarations.push(Declaration {
            id: DeclarationId(58),
            name: Some("AlgebraFieldTemplate".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "param_types".to_string(),
                        ty: DeclarationId(465),
                    },
                    Field {
                        label: "return_type".to_string(),
                        ty: DeclarationId(55),
                    },
                    Field {
                        label: "size_effect".to_string(),
                        ty: DeclarationId(466),
                    },
                    Field {
                        label: "cost_shape".to_string(),
                        ty: DeclarationId(467),
                    },
                    Field {
                        label: "callback_element_position".to_string(),
                        ty: DeclarationId(468),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23410, 23841),
        });
        declarations.push(Declaration {
            id: DeclarationId(59),
            name: Some("kernel_algebra_profile".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(53),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(469)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Map(
                FieldMap::from_entries(vec![
                    (
                        "Int".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(438),
                            payload: vec![],
                        },
                    ),
                    (
                        "Float".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(439),
                            payload: vec![],
                        },
                    ),
                    (
                        "Bool".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(440),
                            payload: vec![],
                        },
                    ),
                    (
                        "String".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(442),
                            payload: vec![],
                        },
                    ),
                    (
                        "List".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(443),
                            payload: vec![],
                        },
                    ),
                    (
                        "Set".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(441),
                            payload: vec![],
                        },
                    ),
                    (
                        "Map".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(444),
                            payload: vec![],
                        },
                    ),
                ])
                .expect("ValueBody::Map"),
            )),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23843, 24153),
        });
        declarations.push(Declaration {
            id: DeclarationId(60),
            name: Some("ordered_ring_templates".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(323),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 24213, 25355)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 24155, 25355),
        });
        declarations.push(Declaration {
            id: DeclarationId(61),
            name: Some("approximate_field_templates".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(324),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 25420, 26514)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 25357, 26514),
        });
        declarations.push(Declaration {
            id: DeclarationId(62),
            name: Some("boolean_algebra_templates".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(325),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 26577, 27338)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 26516, 27338),
        });
        declarations.push(Declaration {
            id: DeclarationId(63),
            name: Some("boolean_algebra_collection_templates".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(326),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 27703, 31048)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 27631, 31048),
        });
        declarations.push(Declaration {
            id: DeclarationId(64),
            name: Some("free_monoid_scalar_templates".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(327),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 31114, 34351)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 31050, 34351),
        });
        declarations.push(Declaration {
            id: DeclarationId(65),
            name: Some("free_monoid_collection_templates".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(328),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 34421, 39686)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 34353, 39686),
        });
        declarations.push(Declaration {
            id: DeclarationId(66),
            name: Some("partial_function_templates".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(329),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 39750, 42574)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 39688, 42574),
        });
        declarations.push(Declaration {
            id: DeclarationId(67),
            name: Some("algebra_templates_for_profile".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(53)],
                output: DeclarationId(330),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 42664, 43127)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 42576, 43127),
        });
        declarations.push(Declaration {
            id: DeclarationId(68),
            name: Some("algebra_type_param_names".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(53)],
                output: DeclarationId(331),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/algebra.dag", 43469, 43750)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 43400, 43750),
        });
        declarations.push(Declaration {
            id: DeclarationId(69),
            name: Some("Magnitude".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/magnitude.dag", 1464, 1478),
        });
        declarations.push(Declaration {
            id: DeclarationId(70),
            name: Some("MachineWidth".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![DeclarationId(71)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/machine_constraints.dag", 3296, 3313),
        });
        declarations.push(Declaration {
            id: DeclarationId(71),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("bits".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/machine_constraints.dag", 3296, 3313),
        });
        declarations.push(Declaration {
            id: DeclarationId(72),
            name: Some("PointerWidth".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/machine_constraints.dag", 6208, 6225),
        });
        declarations.push(Declaration {
            id: DeclarationId(73),
            name: Some("Compose".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![Field {
                    label: "Phantom".to_string(),
                    ty: DeclarationId(470),
                }],
            },
            type_params: vec![DeclarationId(74), DeclarationId(75)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/machine_constraints.dag", 7292, 7342),
        });
        declarations.push(Declaration {
            id: DeclarationId(74),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("Algebra".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/machine_constraints.dag", 7292, 7342),
        });
        declarations.push(Declaration {
            id: DeclarationId(75),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam(
                "MachineConstraint".to_string(),
            )),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/machine_constraints.dag", 7292, 7342),
        });
        declarations.push(Declaration {
            id: DeclarationId(76),
            name: Some("Nat".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(33),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(34),
                    value: DeclarationId(69),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/nat.dag", 2186, 2227),
        });
        declarations.push(Declaration {
            id: DeclarationId(77),
            name: Some("Int8".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(87),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(472),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2549, 2590),
        });
        declarations.push(Declaration {
            id: DeclarationId(78),
            name: Some("Int16".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(87),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(474),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2591, 2634),
        });
        declarations.push(Declaration {
            id: DeclarationId(79),
            name: Some("Int32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(87),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(476),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2635, 2678),
        });
        declarations.push(Declaration {
            id: DeclarationId(80),
            name: Some("Int64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(87),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(478),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2679, 2722),
        });
        declarations.push(Declaration {
            id: DeclarationId(81),
            name: Some("Int128".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(87),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(480),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2723, 2768),
        });
        declarations.push(Declaration {
            id: DeclarationId(82),
            name: Some("UInt8".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(88),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(482),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2834, 2877),
        });
        declarations.push(Declaration {
            id: DeclarationId(83),
            name: Some("UInt16".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(88),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(484),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2878, 2923),
        });
        declarations.push(Declaration {
            id: DeclarationId(84),
            name: Some("UInt32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(88),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(486),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2924, 2969),
        });
        declarations.push(Declaration {
            id: DeclarationId(85),
            name: Some("UInt64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(88),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(488),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2970, 3015),
        });
        declarations.push(Declaration {
            id: DeclarationId(86),
            name: Some("UInt128".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(88),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(490),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 3016, 3063),
        });
        declarations.push(Declaration {
            id: DeclarationId(87),
            name: Some("Int".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(25),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(26),
                    value: DeclarationId(491),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 5921, 5966),
        });
        declarations.push(Declaration {
            id: DeclarationId(88),
            name: Some("UInt".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(76),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 5967, 5982),
        });
        declarations.push(Declaration {
            id: DeclarationId(89),
            name: Some("IntPlatform".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(87),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(492),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 7619, 7680),
        });
        declarations.push(Declaration {
            id: DeclarationId(90),
            name: Some("UIntPlatform".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(88),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(493),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 7681, 7742),
        });
        declarations.push(Declaration {
            id: DeclarationId(91),
            name: Some("NonNegativeInt".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(76),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 9184, 9209),
        });
        declarations.push(Declaration {
            id: DeclarationId(92),
            name: Some("PositiveInt".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(76))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(494)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 9210, 9249),
        });
        declarations.push(Declaration {
            id: DeclarationId(93),
            name: Some("Rational".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(39),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(40),
                    value: DeclarationId(495),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/rational.dag", 1245, 1289),
        });
        declarations.push(Declaration {
            id: DeclarationId(94),
            name: Some("RoundingMode".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "ToNearestEven".to_string(),
                        ty: DeclarationId(496),
                    },
                    Field {
                        label: "ToZero".to_string(),
                        ty: DeclarationId(497),
                    },
                    Field {
                        label: "ToPositiveInfinity".to_string(),
                        ty: DeclarationId(498),
                    },
                    Field {
                        label: "ToNegativeInfinity".to_string(),
                        ty: DeclarationId(499),
                    },
                    Field {
                        label: "ToAwayFromZero".to_string(),
                        ty: DeclarationId(500),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1030, 1141),
        });
        declarations.push(Declaration {
            id: DeclarationId(95),
            name: Some("Precision".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Unbounded".to_string(),
                        ty: DeclarationId(501),
                    },
                    Field {
                        label: "BinaryPrecision".to_string(),
                        ty: DeclarationId(502),
                    },
                    Field {
                        label: "DecimalPrecision".to_string(),
                        ty: DeclarationId(503),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1404, 1589),
        });
        declarations.push(Declaration {
            id: DeclarationId(96),
            name: Some("NanPolicy".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoNaN".to_string(),
                        ty: DeclarationId(504),
                    },
                    Field {
                        label: "QuietNaN".to_string(),
                        ty: DeclarationId(505),
                    },
                    Field {
                        label: "QuietAndSignalingNaN".to_string(),
                        ty: DeclarationId(506),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1812, 1868),
        });
        declarations.push(Declaration {
            id: DeclarationId(97),
            name: Some("InfinityPolicy".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoInfinity".to_string(),
                        ty: DeclarationId(507),
                    },
                    Field {
                        label: "SignedInfinity".to_string(),
                        ty: DeclarationId(508),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2096, 2145),
        });
        declarations.push(Declaration {
            id: DeclarationId(98),
            name: Some("SignedZeroPolicy".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoSignedZero".to_string(),
                        ty: DeclarationId(509),
                    },
                    Field {
                        label: "SignedZero".to_string(),
                        ty: DeclarationId(510),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2376, 2425),
        });
        declarations.push(Declaration {
            id: DeclarationId(99),
            name: Some("SubnormalPolicy".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoSubnormals".to_string(),
                        ty: DeclarationId(511),
                    },
                    Field {
                        label: "GradualUnderflow".to_string(),
                        ty: DeclarationId(512),
                    },
                    Field {
                        label: "FlushToZero".to_string(),
                        ty: DeclarationId(513),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2720, 2788),
        });
        declarations.push(Declaration {
            id: DeclarationId(100),
            name: Some("SpecialValues".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "nan".to_string(),
                        ty: DeclarationId(96),
                    },
                    Field {
                        label: "infinity".to_string(),
                        ty: DeclarationId(97),
                    },
                    Field {
                        label: "signed_zero".to_string(),
                        ty: DeclarationId(98),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 3064, 3162),
        });
        declarations.push(Declaration {
            id: DeclarationId(101),
            name: Some("ApproximateField".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "base".to_string(),
                        ty: DeclarationId(514),
                    },
                    Field {
                        label: "rounding".to_string(),
                        ty: DeclarationId(94),
                    },
                    Field {
                        label: "precision".to_string(),
                        ty: DeclarationId(95),
                    },
                    Field {
                        label: "special_values".to_string(),
                        ty: DeclarationId(100),
                    },
                    Field {
                        label: "subnormal_policy".to_string(),
                        ty: DeclarationId(99),
                    },
                ],
            },
            type_params: vec![DeclarationId(102)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 3398, 3559),
        });
        declarations.push(Declaration {
            id: DeclarationId(102),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("F".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 3398, 3559),
        });
        declarations.push(Declaration {
            id: DeclarationId(103),
            name: Some("Ieee754Float".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1255, 1272),
        });
        declarations.push(Declaration {
            id: DeclarationId(104),
            name: Some("Real".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(101),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(102),
                    value: DeclarationId(515),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1453, 1504),
        });
        declarations.push(Declaration {
            id: DeclarationId(105),
            name: Some("Real32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(104),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(517),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1726, 1771),
        });
        declarations.push(Declaration {
            id: DeclarationId(106),
            name: Some("Real64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(104),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(519),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1772, 1817),
        });
        declarations.push(Declaration {
            id: DeclarationId(107),
            name: Some("Float32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(105),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1819, 1840),
        });
        declarations.push(Declaration {
            id: DeclarationId(108),
            name: Some("Float64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(106),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1841, 1862),
        });
        declarations.push(Declaration {
            id: DeclarationId(109),
            name: Some("Float".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(108),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1881, 1901),
        });
        declarations.push(Declaration {
            id: DeclarationId(110),
            name: Some("kernel_type_set".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(122),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(634)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Map(
                FieldMap::from_entries(vec![
                    (
                        "String".to_string(),
                        FieldValue::Literal(LiteralBits::Bool(true)),
                    ),
                    (
                        "Int".to_string(),
                        FieldValue::Literal(LiteralBits::Bool(true)),
                    ),
                    (
                        "Bool".to_string(),
                        FieldValue::Literal(LiteralBits::Bool(true)),
                    ),
                    (
                        "Float".to_string(),
                        FieldValue::Literal(LiteralBits::Bool(true)),
                    ),
                    (
                        "Secret".to_string(),
                        FieldValue::Literal(LiteralBits::Bool(true)),
                    ),
                    (
                        "Json".to_string(),
                        FieldValue::Literal(LiteralBits::Bool(true)),
                    ),
                    (
                        "Unit".to_string(),
                        FieldValue::Literal(LiteralBits::Bool(true)),
                    ),
                    (
                        "Bytes".to_string(),
                        FieldValue::Literal(LiteralBits::Bool(true)),
                    ),
                ])
                .expect("ValueBody::Map"),
            )),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 3041, 3206),
        });
        declarations.push(Declaration {
            id: DeclarationId(111),
            name: Some("is_kernel_type".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225)],
                output: DeclarationId(122),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/types.dag", 3248, 3344)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 3208, 3344),
        });
        declarations.push(Declaration {
            id: DeclarationId(112),
            name: Some("container_type_arity".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(87),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(635)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Map(
                FieldMap::from_entries(vec![
                    (
                        "List".to_string(),
                        FieldValue::Literal(LiteralBits::Int("1".to_string())),
                    ),
                    (
                        "Set".to_string(),
                        FieldValue::Literal(LiteralBits::Int("1".to_string())),
                    ),
                    (
                        "Map".to_string(),
                        FieldValue::Literal(LiteralBits::Int("2".to_string())),
                    ),
                ])
                .expect("ValueBody::Map"),
            )),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 3506, 3587),
        });
        declarations.push(Declaration {
            id: DeclarationId(113),
            name: Some("is_container_type".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225)],
                output: DeclarationId(122),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/types.dag", 3632, 3734)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 3589, 3734),
        });
        declarations.push(Declaration {
            id: DeclarationId(114),
            name: Some("container_expected_arity".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225)],
                output: DeclarationId(520),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/types.dag", 3786, 3827)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 3736, 3827),
        });
        declarations.push(Declaration {
            id: DeclarationId(115),
            name: Some("container_param_names_for".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225)],
                output: DeclarationId(521),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/types.dag", 4067, 4204)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 4003, 4204),
        });
        declarations.push(Declaration {
            id: DeclarationId(116),
            name: Some("container_param_name".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225), DeclarationId(87)],
                output: DeclarationId(522),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/types.dag", 4272, 4512)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 4206, 4512),
        });
        declarations.push(Declaration {
            id: DeclarationId(117),
            name: Some("ordered_element_collections".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(122),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(636)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Map(
                FieldMap::from_entries(vec![(
                    "List".to_string(),
                    FieldValue::Literal(LiteralBits::Bool(true)),
                )])
                .expect("ValueBody::Map"),
            )),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 4706, 4778),
        });
        declarations.push(Declaration {
            id: DeclarationId(118),
            name: Some("is_ordered_element_collection".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225)],
                output: DeclarationId(122),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/types.dag", 4835, 4892)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 4780, 4892),
        });
        declarations.push(Declaration {
            id: DeclarationId(119),
            name: Some("container_template_algebra_rows".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(225),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(637)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Map(
                FieldMap::from_entries(vec![
                    (
                        "List".to_string(),
                        FieldValue::Literal(LiteralBits::String("FreeMonoid".to_string())),
                    ),
                    (
                        "list".to_string(),
                        FieldValue::Literal(LiteralBits::String("FreeMonoid".to_string())),
                    ),
                    (
                        "Set".to_string(),
                        FieldValue::Literal(LiteralBits::String("BooleanAlgebra".to_string())),
                    ),
                    (
                        "set".to_string(),
                        FieldValue::Literal(LiteralBits::String("BooleanAlgebra".to_string())),
                    ),
                    (
                        "Map".to_string(),
                        FieldValue::Literal(LiteralBits::String("PartialFunction".to_string())),
                    ),
                    (
                        "map".to_string(),
                        FieldValue::Literal(LiteralBits::String("PartialFunction".to_string())),
                    ),
                    (
                        "FreeMonoid".to_string(),
                        FieldValue::Literal(LiteralBits::String("FreeMonoid".to_string())),
                    ),
                    (
                        "free_monoid".to_string(),
                        FieldValue::Literal(LiteralBits::String("FreeMonoid".to_string())),
                    ),
                    (
                        "BooleanAlgebra".to_string(),
                        FieldValue::Literal(LiteralBits::String("BooleanAlgebra".to_string())),
                    ),
                    (
                        "boolean_algebra".to_string(),
                        FieldValue::Literal(LiteralBits::String("BooleanAlgebra".to_string())),
                    ),
                    (
                        "PartialFunction".to_string(),
                        FieldValue::Literal(LiteralBits::String("PartialFunction".to_string())),
                    ),
                    (
                        "partial_function".to_string(),
                        FieldValue::Literal(LiteralBits::String("PartialFunction".to_string())),
                    ),
                ])
                .expect("ValueBody::Map"),
            )),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 5684, 6123),
        });
        declarations.push(Declaration {
            id: DeclarationId(120),
            name: Some("container_template_algebra".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225)],
                output: DeclarationId(523),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/types.dag", 6180, 6232)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 6125, 6232),
        });
        declarations.push(Declaration {
            id: DeclarationId(121),
            name: Some("canonical_container_names".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(524),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/types.dag", 6521, 6600)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 6474, 6600),
        });
        declarations.push(Declaration {
            id: DeclarationId(122),
            name: Some("Bool".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "True".to_string(),
                        ty: DeclarationId(525),
                    },
                    Field {
                        label: "False".to_string(),
                        ty: DeclarationId(526),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: Some(DeclarationId(527)),
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 7574, 7598),
        });
        declarations.push(Declaration {
            id: DeclarationId(123),
            name: Some("Unit".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 7666, 7675),
        });
        declarations.push(Declaration {
            id: DeclarationId(124),
            name: Some("Json".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 7792, 7801),
        });
        declarations.push(Declaration {
            id: DeclarationId(125),
            name: Some("Bytes".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 7802, 7812),
        });
        declarations.push(Declaration {
            id: DeclarationId(126),
            name: Some("Char".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(87))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(638)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 8440, 8491),
        });
        declarations.push(Declaration {
            id: DeclarationId(127),
            name: Some("List".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(128),
                }],
            },
            type_params: vec![DeclarationId(128)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9255, 9295),
        });
        declarations.push(Declaration {
            id: DeclarationId(128),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("element".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9255, 9295),
        });
        declarations.push(Declaration {
            id: DeclarationId(129),
            name: Some("Set".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(45),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(46),
                    value: DeclarationId(130),
                }],
            },
            type_params: vec![DeclarationId(130)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9296, 9339),
        });
        declarations.push(Declaration {
            id: DeclarationId(130),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("element".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9296, 9339),
        });
        declarations.push(Declaration {
            id: DeclarationId(131),
            name: Some("Map".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(49),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(50),
                        value: DeclarationId(132),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(51),
                        value: DeclarationId(133),
                    },
                ],
            },
            type_params: vec![DeclarationId(132), DeclarationId(133)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9340, 9390),
        });
        declarations.push(Declaration {
            id: DeclarationId(132),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("key".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9340, 9390),
        });
        declarations.push(Declaration {
            id: DeclarationId(133),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("value".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9340, 9390),
        });
        declarations.push(Declaration {
            id: DeclarationId(134),
            name: Some("CommitSha".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10736, 10762),
        });
        declarations.push(Declaration {
            id: DeclarationId(135),
            name: Some("Sha256".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10830, 10856),
        });
        declarations.push(Declaration {
            id: DeclarationId(136),
            name: Some("RetryCount".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(87))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(639)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10857, 10908),
        });
        declarations.push(Declaration {
            id: DeclarationId(137),
            name: Some("HttpStatus".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(87))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(640)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10909, 10964),
        });
        declarations.push(Declaration {
            id: DeclarationId(138),
            name: Some("Email".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11038, 11064),
        });
        declarations.push(Declaration {
            id: DeclarationId(139),
            name: Some("Port".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(87))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(641)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11065, 11120),
        });
        declarations.push(Declaration {
            id: DeclarationId(140),
            name: Some("GistId".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11209, 11235),
        });
        declarations.push(Declaration {
            id: DeclarationId(141),
            name: Some("Secret".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: Some(NominalOpacity {
                permitted_accessors: vec![],
            }),
            span: SourceSpan::new("dsl/std/types.dag", 11236, 11277),
        });
        declarations.push(Declaration {
            id: DeclarationId(142),
            name: Some("SecretValue".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(141))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(642)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11278, 11320),
        });
        declarations.push(Declaration {
            id: DeclarationId(143),
            name: Some("Url".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11381, 11407),
        });
        declarations.push(Declaration {
            id: DeclarationId(144),
            name: Some("SemVer".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11480, 11506),
        });
        declarations.push(Declaration {
            id: DeclarationId(145),
            name: Some("NonEmptyStr".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(225))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(643)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11507, 11552),
        });
        declarations.push(Declaration {
            id: DeclarationId(146),
            name: Some("LanguageId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(225))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(644)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11553, 11598),
        });
        declarations.push(Declaration {
            id: DeclarationId(147),
            name: Some("SecretName".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(225))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(645)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11599, 11644),
        });
        declarations.push(Declaration {
            id: DeclarationId(148),
            name: Some("PathSegment".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(646)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12279, 12338),
        });
        declarations.push(Declaration {
            id: DeclarationId(149),
            name: Some("GlobSegment".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(647)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12339, 12398),
        });
        declarations.push(Declaration {
            id: DeclarationId(150),
            name: Some("FilePathParts".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "segments".to_string(),
                    ty: DeclarationId(528),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12399, 12451),
        });
        declarations.push(Declaration {
            id: DeclarationId(151),
            name: Some("GlobPattern".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "segments".to_string(),
                    ty: DeclarationId(529),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12452, 12502),
        });
        declarations.push(Declaration {
            id: DeclarationId(152),
            name: Some("FilePath".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(225))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(648)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12503, 12546),
        });
        declarations.push(Declaration {
            id: DeclarationId(153),
            name: Some("SourceSpan".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "file".to_string(),
                        ty: DeclarationId(152),
                    },
                    Field {
                        label: "start".to_string(),
                        ty: DeclarationId(87),
                    },
                    Field {
                        label: "end".to_string(),
                        ty: DeclarationId(87),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13164, 13224),
        });
        declarations.push(Declaration {
            id: DeclarationId(154),
            name: Some("Timestamp".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13324, 13350),
        });
        declarations.push(Declaration {
            id: DeclarationId(155),
            name: Some("EpochMs".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(87))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(649)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13351, 13395),
        });
        declarations.push(Declaration {
            id: DeclarationId(156),
            name: Some("Duration".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(87))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(650)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13396, 13440),
        });
        declarations.push(Declaration {
            id: DeclarationId(157),
            name: Some("Milliseconds".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(87))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(651)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13441, 13508),
        });
        declarations.push(Declaration {
            id: DeclarationId(158),
            name: Some("Seconds".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(87))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(652)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13509, 13571),
        });
        declarations.push(Declaration {
            id: DeclarationId(159),
            name: Some("LogicalTime".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(653)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13738, 13797),
        });
        declarations.push(Declaration {
            id: DeclarationId(160),
            name: Some("IntentId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(654)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14321, 14376),
        });
        declarations.push(Declaration {
            id: DeclarationId(161),
            name: Some("IssueId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(655)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14377, 14431),
        });
        declarations.push(Declaration {
            id: DeclarationId(162),
            name: Some("RunKey".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(656)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14432, 14485),
        });
        declarations.push(Declaration {
            id: DeclarationId(163),
            name: Some("ArtifactId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(657)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14486, 14543),
        });
        declarations.push(Declaration {
            id: DeclarationId(164),
            name: Some("LeaseToken".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(658)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14544, 14601),
        });
        declarations.push(Declaration {
            id: DeclarationId(165),
            name: Some("WorkerId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(659)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14602, 14657),
        });
        declarations.push(Declaration {
            id: DeclarationId(166),
            name: Some("CommentId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(660)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14658, 14714),
        });
        declarations.push(Declaration {
            id: DeclarationId(167),
            name: Some("SignalKey".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(661)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14715, 14771),
        });
        declarations.push(Declaration {
            id: DeclarationId(168),
            name: Some("ContentHash".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(662)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14772, 14830),
        });
        declarations.push(Declaration {
            id: DeclarationId(169),
            name: Some("WorkflowProducerId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(663)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15061, 15132),
        });
        declarations.push(Declaration {
            id: DeclarationId(170),
            name: Some("WorkflowObserverId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(664)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15133, 15204),
        });
        declarations.push(Declaration {
            id: DeclarationId(171),
            name: Some("WorkflowProverId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(665)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15205, 15272),
        });
        declarations.push(Declaration {
            id: DeclarationId(172),
            name: Some("WorkflowRunId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(145))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(666)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15273, 15334),
        });
        declarations.push(Declaration {
            id: DeclarationId(173),
            name: Some("GitRef".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(225))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(667)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15414, 15456),
        });
        declarations.push(Declaration {
            id: DeclarationId(174),
            name: Some("GcpProjectId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(225))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(668)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15841, 15883),
        });
        declarations.push(Declaration {
            id: DeclarationId(175),
            name: Some("ServiceAccountEmail".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16008, 16041),
        });
        declarations.push(Declaration {
            id: DeclarationId(176),
            name: Some("Platform".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Linux".to_string(),
                        ty: DeclarationId(530),
                    },
                    Field {
                        label: "Macos".to_string(),
                        ty: DeclarationId(531),
                    },
                    Field {
                        label: "Windows".to_string(),
                        ty: DeclarationId(532),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16427, 16470),
        });
        declarations.push(Declaration {
            id: DeclarationId(177),
            name: Some("TopologyNodeKind".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Pure".to_string(),
                        ty: DeclarationId(533),
                    },
                    Field {
                        label: "Transport".to_string(),
                        ty: DeclarationId(534),
                    },
                    Field {
                        label: "SubDag".to_string(),
                        ty: DeclarationId(535),
                    },
                    Field {
                        label: "Env".to_string(),
                        ty: DeclarationId(536),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16598, 16653),
        });
        declarations.push(Declaration {
            id: DeclarationId(178),
            name: Some("DocSourceKind".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Template".to_string(),
                        ty: DeclarationId(537),
                    },
                    Field {
                        label: "Generated".to_string(),
                        ty: DeclarationId(538),
                    },
                    Field {
                        label: "Static".to_string(),
                        ty: DeclarationId(539),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16654, 16707),
        });
        declarations.push(Declaration {
            id: DeclarationId(179),
            name: Some("FermiDepth".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Xs".to_string(),
                        ty: DeclarationId(540),
                    },
                    Field {
                        label: "S".to_string(),
                        ty: DeclarationId(541),
                    },
                    Field {
                        label: "M".to_string(),
                        ty: DeclarationId(542),
                    },
                    Field {
                        label: "L".to_string(),
                        ty: DeclarationId(543),
                    },
                    Field {
                        label: "Xl".to_string(),
                        ty: DeclarationId(544),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16709, 16746),
        });
        declarations.push(Declaration {
            id: DeclarationId(180),
            name: Some("CredentialFlow".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Stored".to_string(),
                        ty: DeclarationId(545),
                    },
                    Field {
                        label: "PlatformInjected".to_string(),
                        ty: DeclarationId(546),
                    },
                    Field {
                        label: "WorkloadIdentity".to_string(),
                        ty: DeclarationId(549),
                    },
                    Field {
                        label: "InteractiveAuth".to_string(),
                        ty: DeclarationId(551),
                    },
                    Field {
                        label: "Chained".to_string(),
                        ty: DeclarationId(553),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16994, 17316),
        });
        declarations.push(Declaration {
            id: DeclarationId(181),
            name: Some("Arch".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "X86_64".to_string(),
                        ty: DeclarationId(554),
                    },
                    Field {
                        label: "X86".to_string(),
                        ty: DeclarationId(555),
                    },
                    Field {
                        label: "Aarch64".to_string(),
                        ty: DeclarationId(556),
                    },
                    Field {
                        label: "Arm".to_string(),
                        ty: DeclarationId(557),
                    },
                    Field {
                        label: "Armv7".to_string(),
                        ty: DeclarationId(558),
                    },
                    Field {
                        label: "Mips".to_string(),
                        ty: DeclarationId(559),
                    },
                    Field {
                        label: "Mipsel".to_string(),
                        ty: DeclarationId(560),
                    },
                    Field {
                        label: "Mips64".to_string(),
                        ty: DeclarationId(561),
                    },
                    Field {
                        label: "Mips64el".to_string(),
                        ty: DeclarationId(562),
                    },
                    Field {
                        label: "Riscv64".to_string(),
                        ty: DeclarationId(563),
                    },
                    Field {
                        label: "Wasm32".to_string(),
                        ty: DeclarationId(564),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17391, 17494),
        });
        declarations.push(Declaration {
            id: DeclarationId(182),
            name: Some("Vendor".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "UnknownVendor".to_string(),
                        ty: DeclarationId(565),
                    },
                    Field {
                        label: "Pc".to_string(),
                        ty: DeclarationId(566),
                    },
                    Field {
                        label: "Apple".to_string(),
                        ty: DeclarationId(567),
                    },
                    Field {
                        label: "W64".to_string(),
                        ty: DeclarationId(568),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17495, 17541),
        });
        declarations.push(Declaration {
            id: DeclarationId(183),
            name: Some("Os".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Linux".to_string(),
                        ty: DeclarationId(569),
                    },
                    Field {
                        label: "Macos".to_string(),
                        ty: DeclarationId(570),
                    },
                    Field {
                        label: "Windows".to_string(),
                        ty: DeclarationId(571),
                    },
                    Field {
                        label: "Freebsd".to_string(),
                        ty: DeclarationId(572),
                    },
                    Field {
                        label: "Android".to_string(),
                        ty: DeclarationId(573),
                    },
                    Field {
                        label: "Ios".to_string(),
                        ty: DeclarationId(574),
                    },
                    Field {
                        label: "Wasi".to_string(),
                        ty: DeclarationId(575),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17542, 17608),
        });
        declarations.push(Declaration {
            id: DeclarationId(184),
            name: Some("AbiEnv".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoneAbi".to_string(),
                        ty: DeclarationId(576),
                    },
                    Field {
                        label: "Gnu".to_string(),
                        ty: DeclarationId(577),
                    },
                    Field {
                        label: "GnuEabi".to_string(),
                        ty: DeclarationId(578),
                    },
                    Field {
                        label: "GnuEabihf".to_string(),
                        ty: DeclarationId(579),
                    },
                    Field {
                        label: "Musl".to_string(),
                        ty: DeclarationId(580),
                    },
                    Field {
                        label: "Msvc".to_string(),
                        ty: DeclarationId(581),
                    },
                    Field {
                        label: "AndroidAbi".to_string(),
                        ty: DeclarationId(582),
                    },
                    Field {
                        label: "Eabi".to_string(),
                        ty: DeclarationId(583),
                    },
                    Field {
                        label: "Eabihf".to_string(),
                        ty: DeclarationId(584),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17609, 17701),
        });
        declarations.push(Declaration {
            id: DeclarationId(185),
            name: Some("ExecutionEnv".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Native".to_string(),
                        ty: DeclarationId(585),
                    },
                    Field {
                        label: "Wsl".to_string(),
                        ty: DeclarationId(586),
                    },
                    Field {
                        label: "Container".to_string(),
                        ty: DeclarationId(587),
                    },
                    Field {
                        label: "Ci".to_string(),
                        ty: DeclarationId(588),
                    },
                    Field {
                        label: "Emulator".to_string(),
                        ty: DeclarationId(589),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17702, 17762),
        });
        declarations.push(Declaration {
            id: DeclarationId(186),
            name: Some("TargetTriple".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "arch".to_string(),
                        ty: DeclarationId(181),
                    },
                    Field {
                        label: "vendor".to_string(),
                        ty: DeclarationId(182),
                    },
                    Field {
                        label: "os".to_string(),
                        ty: DeclarationId(183),
                    },
                    Field {
                        label: "env".to_string(),
                        ty: DeclarationId(590),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17764, 17839),
        });
        declarations.push(Declaration {
            id: DeclarationId(187),
            name: Some("RuntimePlatform".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "host".to_string(),
                        ty: DeclarationId(186),
                    },
                    Field {
                        label: "env".to_string(),
                        ty: DeclarationId(185),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17841, 17906),
        });
        declarations.push(Declaration {
            id: DeclarationId(188),
            name: Some("EntryKind".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "RegularFile".to_string(),
                        ty: DeclarationId(591),
                    },
                    Field {
                        label: "Directory".to_string(),
                        ty: DeclarationId(592),
                    },
                    Field {
                        label: "Symlink".to_string(),
                        ty: DeclarationId(593),
                    },
                    Field {
                        label: "Missing".to_string(),
                        ty: DeclarationId(594),
                    },
                    Field {
                        label: "Other".to_string(),
                        ty: DeclarationId(595),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18215, 18293),
        });
        declarations.push(Declaration {
            id: DeclarationId(189),
            name: Some("SymlinkTarget".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "TargetFile".to_string(),
                        ty: DeclarationId(596),
                    },
                    Field {
                        label: "TargetDir".to_string(),
                        ty: DeclarationId(597),
                    },
                    Field {
                        label: "Broken".to_string(),
                        ty: DeclarationId(598),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18423, 18481),
        });
        declarations.push(Declaration {
            id: DeclarationId(190),
            name: Some("TextFilePath".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(152),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 19642, 19672),
        });
        declarations.push(Declaration {
            id: DeclarationId(191),
            name: Some("BinaryFilePath".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(152),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 19753, 19783),
        });
        declarations.push(Declaration {
            id: DeclarationId(192),
            name: Some("MimeType".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20000, 20022),
        });
        declarations.push(Declaration {
            id: DeclarationId(193),
            name: Some("HttpMethod".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "GET".to_string(),
                        ty: DeclarationId(599),
                    },
                    Field {
                        label: "POST".to_string(),
                        ty: DeclarationId(600),
                    },
                    Field {
                        label: "PUT".to_string(),
                        ty: DeclarationId(601),
                    },
                    Field {
                        label: "PATCH".to_string(),
                        ty: DeclarationId(602),
                    },
                    Field {
                        label: "DELETE".to_string(),
                        ty: DeclarationId(603),
                    },
                    Field {
                        label: "HEAD".to_string(),
                        ty: DeclarationId(604),
                    },
                    Field {
                        label: "OPTIONS".to_string(),
                        ty: DeclarationId(605),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20766, 20834),
        });
        declarations.push(Declaration {
            id: DeclarationId(194),
            name: Some("AuthScheme".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Bearer".to_string(),
                        ty: DeclarationId(606),
                    },
                    Field {
                        label: "Header".to_string(),
                        ty: DeclarationId(607),
                    },
                    Field {
                        label: "Basic".to_string(),
                        ty: DeclarationId(608),
                    },
                    Field {
                        label: "ApiKey".to_string(),
                        ty: DeclarationId(609),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21302, 21398),
        });
        declarations.push(Declaration {
            id: DeclarationId(195),
            name: Some("AccessToken".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "token".to_string(),
                        ty: DeclarationId(141),
                    },
                    Field {
                        label: "scheme".to_string(),
                        ty: DeclarationId(194),
                    },
                    Field {
                        label: "expires_at".to_string(),
                        ty: DeclarationId(610),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21520, 21653),
        });
        declarations.push(Declaration {
            id: DeclarationId(196),
            name: Some("Credential".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "token".to_string(),
                        ty: DeclarationId(141),
                    },
                    Field {
                        label: "scheme".to_string(),
                        ty: DeclarationId(194),
                    },
                    Field {
                        label: "header_name".to_string(),
                        ty: DeclarationId(611),
                    },
                    Field {
                        label: "source_id".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "required_scopes".to_string(),
                        ty: DeclarationId(612),
                    },
                    Field {
                        label: "expires_in".to_string(),
                        ty: DeclarationId(613),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21655, 21805),
        });
        declarations.push(Declaration {
            id: DeclarationId(197),
            name: Some("FilesystemHandle".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(152))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(669)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21882, 21946),
        });
        declarations.push(Declaration {
            id: DeclarationId(198),
            name: Some("NetworkHandle".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(123))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(670)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21947, 22004),
        });
        declarations.push(Declaration {
            id: DeclarationId(199),
            name: Some("ToolHandle".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(225))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(671)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22005, 22061),
        });
        declarations.push(Declaration {
            id: DeclarationId(200),
            name: Some("TransportRequest".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "method".to_string(),
                        ty: DeclarationId(193),
                    },
                    Field {
                        label: "url".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "headers".to_string(),
                        ty: DeclarationId(124),
                    },
                    Field {
                        label: "body".to_string(),
                        ty: DeclarationId(225),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22138, 22229),
        });
        declarations.push(Declaration {
            id: DeclarationId(201),
            name: Some("TransportResponse".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "status".to_string(),
                        ty: DeclarationId(87),
                    },
                    Field {
                        label: "headers".to_string(),
                        ty: DeclarationId(124),
                    },
                    Field {
                        label: "body".to_string(),
                        ty: DeclarationId(225),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22231, 22302),
        });
        declarations.push(Declaration {
            id: DeclarationId(202),
            name: Some("FileResponse".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "success".to_string(),
                        ty: DeclarationId(122),
                    },
                    Field {
                        label: "content".to_string(),
                        ty: DeclarationId(225),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22304, 22374),
        });
        declarations.push(Declaration {
            id: DeclarationId(203),
            name: Some("ShellResponse".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "exit_code".to_string(),
                        ty: DeclarationId(87),
                    },
                    Field {
                        label: "stdout".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "stderr".to_string(),
                        ty: DeclarationId(225),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22376, 22449),
        });
        declarations.push(Declaration {
            id: DeclarationId(204),
            name: Some("RestResponse".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "status".to_string(),
                        ty: DeclarationId(87),
                    },
                    Field {
                        label: "headers".to_string(),
                        ty: DeclarationId(124),
                    },
                    Field {
                        label: "body".to_string(),
                        ty: DeclarationId(124),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22451, 22515),
        });
        declarations.push(Declaration {
            id: DeclarationId(205),
            name: Some("TestResult".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "ok".to_string(),
                        ty: DeclarationId(122),
                    },
                    Field {
                        label: "stdout".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "stderr".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "duration_ms".to_string(),
                        ty: DeclarationId(157),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22592, 22699),
        });
        declarations.push(Declaration {
            id: DeclarationId(206),
            name: Some("Summary".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "total".to_string(),
                        ty: DeclarationId(87),
                    },
                    Field {
                        label: "passed".to_string(),
                        ty: DeclarationId(87),
                    },
                    Field {
                        label: "failed".to_string(),
                        ty: DeclarationId(87),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22701, 22758),
        });
        declarations.push(Declaration {
            id: DeclarationId(207),
            name: Some("StageResult".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "success".to_string(),
                        ty: DeclarationId(122),
                    },
                    Field {
                        label: "stdout".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "stderr".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "skipped".to_string(),
                        ty: DeclarationId(122),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22760, 22861),
        });
        declarations.push(Declaration {
            id: DeclarationId(208),
            name: Some("DocumentLine".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "text".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "is_comment".to_string(),
                        ty: DeclarationId(122),
                    },
                    Field {
                        label: "is_blank".to_string(),
                        ty: DeclarationId(122),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23014, 23086),
        });
        declarations.push(Declaration {
            id: DeclarationId(209),
            name: Some("DocumentSection".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "title".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "has_title".to_string(),
                        ty: DeclarationId(122),
                    },
                    Field {
                        label: "lines".to_string(),
                        ty: DeclarationId(614),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23088, 23174),
        });
        declarations.push(Declaration {
            id: DeclarationId(210),
            name: Some("Document".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "header".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "has_header".to_string(),
                        ty: DeclarationId(122),
                    },
                    Field {
                        label: "comment_prefix".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "sections".to_string(),
                        ty: DeclarationId(615),
                    },
                    Field {
                        label: "trailing_newline".to_string(),
                        ty: DeclarationId(122),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23176, 23313),
        });
        declarations.push(Declaration {
            id: DeclarationId(211),
            name: Some("TextFile".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "document".to_string(),
                        ty: DeclarationId(210),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23315, 23368),
        });
        declarations.push(Declaration {
            id: DeclarationId(212),
            name: Some("RenderedTextFile".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "content".to_string(),
                        ty: DeclarationId(225),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23370, 23428),
        });
        declarations.push(Declaration {
            id: DeclarationId(213),
            name: Some("ToolEntry".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "command".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "description".to_string(),
                        ty: DeclarationId(616),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23505, 23579),
        });
        declarations.push(Declaration {
            id: DeclarationId(214),
            name: Some("ToolRegistry".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "tools".to_string(),
                    ty: DeclarationId(617),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23581, 23627),
        });
        declarations.push(Declaration {
            id: DeclarationId(215),
            name: Some("DagTopology".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "nodes".to_string(),
                        ty: DeclarationId(618),
                    },
                    Field {
                        label: "edges".to_string(),
                        ty: DeclarationId(619),
                    },
                    Field {
                        label: "subdag_boundaries".to_string(),
                        ty: DeclarationId(620),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23886, 23996),
        });
        declarations.push(Declaration {
            id: DeclarationId(216),
            name: Some("TopologyNode".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "id".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "label".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "kind".to_string(),
                        ty: DeclarationId(177),
                    },
                    Field {
                        label: "parent".to_string(),
                        ty: DeclarationId(621),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23998, 24118),
        });
        declarations.push(Declaration {
            id: DeclarationId(217),
            name: Some("TopologyEdge".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "from".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "to".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "port".to_string(),
                        ty: DeclarationId(622),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24120, 24185),
        });
        declarations.push(Declaration {
            id: DeclarationId(218),
            name: Some("DagDiff".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "added".to_string(),
                        ty: DeclarationId(623),
                    },
                    Field {
                        label: "removed".to_string(),
                        ty: DeclarationId(624),
                    },
                    Field {
                        label: "changed".to_string(),
                        ty: DeclarationId(625),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24187, 24273),
        });
        declarations.push(Declaration {
            id: DeclarationId(219),
            name: Some("CodegenTarget".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(152),
                    },
                    Field {
                        label: "backend".to_string(),
                        ty: DeclarationId(626),
                    },
                    Field {
                        label: "target".to_string(),
                        ty: DeclarationId(627),
                    },
                    Field {
                        label: "runtime_env".to_string(),
                        ty: DeclarationId(628),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24342, 24476),
        });
        declarations.push(Declaration {
            id: DeclarationId(220),
            name: Some("CodegenBackend".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Rust".to_string(),
                        ty: DeclarationId(629),
                    },
                    Field {
                        label: "Go".to_string(),
                        ty: DeclarationId(630),
                    },
                    Field {
                        label: "C".to_string(),
                        ty: DeclarationId(631),
                    },
                    Field {
                        label: "Mips".to_string(),
                        ty: DeclarationId(632),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24478, 24520),
        });
        declarations.push(Declaration {
            id: DeclarationId(221),
            name: Some("PragmaDirective".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "key".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "value".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "scope".to_string(),
                        ty: DeclarationId(633),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24522, 24593),
        });
        declarations.push(Declaration {
            id: DeclarationId(222),
            name: Some("DocSource".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(152),
                    },
                    Field {
                        label: "kind".to_string(),
                        ty: DeclarationId(178),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24734, 24791),
        });
        declarations.push(Declaration {
            id: DeclarationId(223),
            name: Some("ReferenceModel".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![DeclarationId(224)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 26725, 26744),
        });
        declarations.push(Declaration {
            id: DeclarationId(224),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 26725, 26744),
        });
        declarations.push(Declaration {
            id: DeclarationId(225),
            name: Some("String".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(126),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/string_type.dag", 606, 636),
        });
        declarations.push(Declaration {
            id: DeclarationId(226),
            name: Some("CharClass".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Whitespace".to_string(),
                        ty: DeclarationId(672),
                    },
                    Field {
                        label: "Digit".to_string(),
                        ty: DeclarationId(673),
                    },
                    Field {
                        label: "IdentStart".to_string(),
                        ty: DeclarationId(674),
                    },
                    Field {
                        label: "IdentContinue".to_string(),
                        ty: DeclarationId(675),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 3662, 3726),
        });
        declarations.push(Declaration {
            id: DeclarationId(227),
            name: Some("char_in_class".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(126), DeclarationId(226)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(250))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 4059, 4926),
        });
        declarations.push(Declaration {
            id: DeclarationId(228),
            name: Some("DisplayWidth".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "ZeroWidth".to_string(),
                        ty: DeclarationId(676),
                    },
                    Field {
                        label: "Narrow".to_string(),
                        ty: DeclarationId(677),
                    },
                    Field {
                        label: "Wide".to_string(),
                        ty: DeclarationId(678),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5290, 5335),
        });
        declarations.push(Declaration {
            id: DeclarationId(229),
            name: Some("display_width_columns".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(228)],
                output: DeclarationId(87),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 5386, 5462)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5337, 5462),
        });
        declarations.push(Declaration {
            id: DeclarationId(230),
            name: Some("UnicodeBlock".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "start".to_string(),
                        ty: DeclarationId(87),
                    },
                    Field {
                        label: "end_inclusive".to_string(),
                        ty: DeclarationId(87),
                    },
                    Field {
                        label: "default_width".to_string(),
                        ty: DeclarationId(228),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5697, 5797),
        });
        declarations.push(Declaration {
            id: DeclarationId(231),
            name: Some("zero_width_blocks".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(230),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(679)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::List(vec![
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Combining Diacritical Marks".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("768".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("879".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(676),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Combining Diacritical Marks Extended".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("6832".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("6911".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(676),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Combining Diacritical Marks Supplement".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("7616".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("7679".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(676),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Combining Marks for Symbols".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("8400".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("8447".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(676),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("Variation Selectors".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65024".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65039".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(676),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Combining Half Marks".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65056".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65071".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(676),
                            payload: vec![],
                        },
                    ),
                ]),
            ])),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5859, 6608),
        });
        declarations.push(Declaration {
            id: DeclarationId(232),
            name: Some("zero_width_codepoints".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(87),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(680)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::List(vec![
                FieldValue::Literal(LiteralBits::Int("8203".to_string())),
                FieldValue::Literal(LiteralBits::Int("8204".to_string())),
                FieldValue::Literal(LiteralBits::Int("8205".to_string())),
                FieldValue::Literal(LiteralBits::Int("65279".to_string())),
            ])),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 6675, 6887),
        });
        declarations.push(Declaration {
            id: DeclarationId(233),
            name: Some("wide_blocks".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(230),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(681)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::List(vec![
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("Hangul Jamo".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("4352".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("4447".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "CJK Radicals and Symbols".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("11904".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("12350".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Hiragana / Katakana / CJK Compat".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("12353".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("13247".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("CJK Extension A".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("13312".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("19903".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "CJK Unified Ideographs".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("19968".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("40959".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Yi Syllables and Radicals".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("40960".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("42191".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("Hangul Syllables".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("44032".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("55215".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "CJK Compatibility Ideographs".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("63744".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("64255".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "CJK Compatibility Forms".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65072".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65135".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("Fullwidth ASCII".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65281".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65376".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("Fullwidth Signs".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65504".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("65510".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("CJK Extension B+".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("131072".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("196607".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("CJK Extension G+".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("196608".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("262143".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Misc Symbols and Dingbats".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("9728".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("10175".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String(
                            "Symbols / Pictographs / Emoticons".to_string(),
                        )),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("127744".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("129535".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
                FieldValue::Record(vec![
                    (
                        "name".to_string(),
                        FieldValue::Literal(LiteralBits::String("Symbols Extended".to_string())),
                    ),
                    (
                        "start".to_string(),
                        FieldValue::Literal(LiteralBits::Int("129536".to_string())),
                    ),
                    (
                        "end_inclusive".to_string(),
                        FieldValue::Literal(LiteralBits::Int("131071".to_string())),
                    ),
                    (
                        "default_width".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(678),
                            payload: vec![],
                        },
                    ),
                ]),
            ])),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 6934, 8723),
        });
        declarations.push(Declaration {
            id: DeclarationId(234),
            name: Some("code_point".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(126)],
                output: DeclarationId(87),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(253))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 8873, 8910),
        });
        declarations.push(Declaration {
            id: DeclarationId(235),
            name: Some("in_block".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87), DeclarationId(230)],
                output: DeclarationId(122),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 8962, 9014)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 8912, 9014),
        });
        declarations.push(Declaration {
            id: DeclarationId(236),
            name: Some("char_display_width".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(126)],
                output: DeclarationId(228),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 9063, 9366)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 9016, 9366),
        });
        declarations.push(Declaration {
            id: DeclarationId(237),
            name: Some("char_width".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(126)],
                output: DeclarationId(87),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 9398, 9454)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 9368, 9454),
        });
        declarations.push(Declaration {
            id: DeclarationId(238),
            name: Some("string_display_width".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225)],
                output: DeclarationId(87),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 9498, 9636)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 9456, 9636),
        });
        declarations.push(Declaration {
            id: DeclarationId(239),
            name: Some("repeat_string_loop".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225), DeclarationId(225), DeclarationId(87)],
                output: DeclarationId(225),
                body: ArrowBody::Unparsed(SourceSpan::new(
                    "dsl/std/render_repeat_string_bootstrap.dag",
                    598,
                    717,
                )),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/render_repeat_string_bootstrap.dag", 526, 717),
        });
        declarations.push(Declaration {
            id: DeclarationId(240),
            name: Some("repeat_string".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225), DeclarationId(87)],
                output: DeclarationId(225),
                body: ArrowBody::Unparsed(SourceSpan::new(
                    "dsl/std/render_repeat_string_bootstrap.dag",
                    765,
                    818,
                )),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/render_repeat_string_bootstrap.dag", 719, 818),
        });
        declarations.push(Declaration {
            id: DeclarationId(241),
            name: Some("MethodDeclaration".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "name".to_string(),
                    ty: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 4779, 4820),
        });
        declarations.push(Declaration {
            id: DeclarationId(242),
            name: Some("add_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("add".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5269, 5321),
        });
        declarations.push(Declaration {
            id: DeclarationId(243),
            name: Some("all_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("all".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5322, 5374),
        });
        declarations.push(Declaration {
            id: DeclarationId(244),
            name: Some("any_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("any".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5375, 5427),
        });
        declarations.push(Declaration {
            id: DeclarationId(245),
            name: Some("append_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("append".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5428, 5486),
        });
        declarations.push(Declaration {
            id: DeclarationId(246),
            name: Some("bottom_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("bottom".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5487, 5545),
        });
        declarations.push(Declaration {
            id: DeclarationId(247),
            name: Some("chars_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("chars".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5546, 5602),
        });
        declarations.push(Declaration {
            id: DeclarationId(248),
            name: Some("clamp_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("clamp".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5603, 5659),
        });
        declarations.push(Declaration {
            id: DeclarationId(249),
            name: Some("compare_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("compare".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5660, 5720),
        });
        declarations.push(Declaration {
            id: DeclarationId(250),
            name: Some("complement_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("complement".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5721, 5787),
        });
        declarations.push(Declaration {
            id: DeclarationId(251),
            name: Some("concat_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("concat".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5788, 5846),
        });
        declarations.push(Declaration {
            id: DeclarationId(252),
            name: Some("contains_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("contains".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5847, 5909),
        });
        declarations.push(Declaration {
            id: DeclarationId(253),
            name: Some("count_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("count".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5910, 5966),
        });
        declarations.push(Declaration {
            id: DeclarationId(254),
            name: Some("diff_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("diff".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 5967, 6021),
        });
        declarations.push(Declaration {
            id: DeclarationId(255),
            name: Some("empty_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("empty".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6022, 6078),
        });
        declarations.push(Declaration {
            id: DeclarationId(256),
            name: Some("ends_with_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("ends_with".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6079, 6143),
        });
        declarations.push(Declaration {
            id: DeclarationId(257),
            name: Some("enumerate_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("enumerate".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6144, 6208),
        });
        declarations.push(Declaration {
            id: DeclarationId(258),
            name: Some("filter_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("filter".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6209, 6267),
        });
        declarations.push(Declaration {
            id: DeclarationId(259),
            name: Some("first_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("first".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6268, 6324),
        });
        declarations.push(Declaration {
            id: DeclarationId(260),
            name: Some("flat_map_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("flat_map".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6325, 6387),
        });
        declarations.push(Declaration {
            id: DeclarationId(261),
            name: Some("fold_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("fold".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6388, 6442),
        });
        declarations.push(Declaration {
            id: DeclarationId(262),
            name: Some("get_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("get".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6443, 6495),
        });
        declarations.push(Declaration {
            id: DeclarationId(263),
            name: Some("has_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("has".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6496, 6548),
        });
        declarations.push(Declaration {
            id: DeclarationId(264),
            name: Some("intersect_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("intersect".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6549, 6613),
        });
        declarations.push(Declaration {
            id: DeclarationId(265),
            name: Some("is_empty_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("is_empty".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6614, 6676),
        });
        declarations.push(Declaration {
            id: DeclarationId(266),
            name: Some("join_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("join".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6677, 6731),
        });
        declarations.push(Declaration {
            id: DeclarationId(267),
            name: Some("keys_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("keys".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6732, 6786),
        });
        declarations.push(Declaration {
            id: DeclarationId(268),
            name: Some("last_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("last".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6787, 6841),
        });
        declarations.push(Declaration {
            id: DeclarationId(269),
            name: Some("length_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("length".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6842, 6900),
        });
        declarations.push(Declaration {
            id: DeclarationId(270),
            name: Some("list_push_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("list_push".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6901, 6965),
        });
        declarations.push(Declaration {
            id: DeclarationId(271),
            name: Some("lookup_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("lookup".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 6966, 7024),
        });
        declarations.push(Declaration {
            id: DeclarationId(272),
            name: Some("map_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("map".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7025, 7077),
        });
        declarations.push(Declaration {
            id: DeclarationId(273),
            name: Some("map_contains_key_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("map_contains_key".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7078, 7156),
        });
        declarations.push(Declaration {
            id: DeclarationId(274),
            name: Some("map_get_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("map_get".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7157, 7217),
        });
        declarations.push(Declaration {
            id: DeclarationId(275),
            name: Some("map_has_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("map_has".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7218, 7278),
        });
        declarations.push(Declaration {
            id: DeclarationId(276),
            name: Some("map_insert_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("map_insert".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7279, 7345),
        });
        declarations.push(Declaration {
            id: DeclarationId(277),
            name: Some("map_keys_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("map_keys".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7346, 7408),
        });
        declarations.push(Declaration {
            id: DeclarationId(278),
            name: Some("map_merge_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("map_merge".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7409, 7473),
        });
        declarations.push(Declaration {
            id: DeclarationId(279),
            name: Some("map_values_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("map_values".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7474, 7540),
        });
        declarations.push(Declaration {
            id: DeclarationId(280),
            name: Some("meet_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("meet".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7541, 7595),
        });
        declarations.push(Declaration {
            id: DeclarationId(281),
            name: Some("member_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("member".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7596, 7654),
        });
        declarations.push(Declaration {
            id: DeclarationId(282),
            name: Some("mul_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("mul".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7655, 7707),
        });
        declarations.push(Declaration {
            id: DeclarationId(283),
            name: Some("negate_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("negate".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7708, 7766),
        });
        declarations.push(Declaration {
            id: DeclarationId(284),
            name: Some("one_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("one".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7767, 7819),
        });
        declarations.push(Declaration {
            id: DeclarationId(285),
            name: Some("reciprocal_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("reciprocal".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7820, 7886),
        });
        declarations.push(Declaration {
            id: DeclarationId(286),
            name: Some("replace_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("replace".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7887, 7947),
        });
        declarations.push(Declaration {
            id: DeclarationId(287),
            name: Some("reverse_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("reverse".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 7948, 8008),
        });
        declarations.push(Declaration {
            id: DeclarationId(288),
            name: Some("skip_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("skip".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8009, 8063),
        });
        declarations.push(Declaration {
            id: DeclarationId(289),
            name: Some("sort_by_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("sort_by".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8064, 8124),
        });
        declarations.push(Declaration {
            id: DeclarationId(290),
            name: Some("split_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("split".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8125, 8181),
        });
        declarations.push(Declaration {
            id: DeclarationId(291),
            name: Some("starts_with_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("starts_with".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8182, 8250),
        });
        declarations.push(Declaration {
            id: DeclarationId(292),
            name: Some("string_contains_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("string_contains".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8251, 8327),
        });
        declarations.push(Declaration {
            id: DeclarationId(293),
            name: Some("substring_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("substring".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8328, 8392),
        });
        declarations.push(Declaration {
            id: DeclarationId(294),
            name: Some("take_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("take".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8393, 8447),
        });
        declarations.push(Declaration {
            id: DeclarationId(295),
            name: Some("to_int_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("to_int".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8448, 8506),
        });
        declarations.push(Declaration {
            id: DeclarationId(296),
            name: Some("to_lower_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("to_lower".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8507, 8569),
        });
        declarations.push(Declaration {
            id: DeclarationId(297),
            name: Some("to_string_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("to_string".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8570, 8634),
        });
        declarations.push(Declaration {
            id: DeclarationId(298),
            name: Some("to_upper_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("to_upper".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8635, 8697),
        });
        declarations.push(Declaration {
            id: DeclarationId(299),
            name: Some("top_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("top".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8698, 8750),
        });
        declarations.push(Declaration {
            id: DeclarationId(300),
            name: Some("trim_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("trim".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8751, 8805),
        });
        declarations.push(Declaration {
            id: DeclarationId(301),
            name: Some("union_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("union".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8806, 8862),
        });
        declarations.push(Declaration {
            id: DeclarationId(302),
            name: Some("values_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("values".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8863, 8921),
        });
        declarations.push(Declaration {
            id: DeclarationId(303),
            name: Some("with_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("with".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8922, 8976),
        });
        declarations.push(Declaration {
            id: DeclarationId(304),
            name: Some("zero_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(241),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(241)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![(
                    "name".to_string(),
                    FieldValue::Literal(LiteralBits::String("zero".to_string())),
                )],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/methods.dag", 8977, 9031),
        });
        declarations.push(Declaration {
            id: DeclarationId(305),
            name: Some("DeclarationRef".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(225),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 692, 720),
        });
        declarations.push(Declaration {
            id: DeclarationId(306),
            name: Some("VariantNaming".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "AsAuthored".to_string(),
                        ty: DeclarationId(682),
                    },
                    Field {
                        label: "SnakeCase".to_string(),
                        ty: DeclarationId(683),
                    },
                    Field {
                        label: "StripPrefixAndSnakeCase".to_string(),
                        ty: DeclarationId(684),
                    },
                    Field {
                        label: "StripSuffixAndSnakeCase".to_string(),
                        ty: DeclarationId(685),
                    },
                    Field {
                        label: "StripPrefixSuffixAndSnakeCase".to_string(),
                        ty: DeclarationId(686),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 1850, 2060),
        });
        declarations.push(Declaration {
            id: DeclarationId(307),
            name: Some("VariantEncoding".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "StringVariant".to_string(),
                        ty: DeclarationId(687),
                    },
                    Field {
                        label: "InternallyTaggedObject".to_string(),
                        ty: DeclarationId(688),
                    },
                    Field {
                        label: "UntaggedVariant".to_string(),
                        ty: DeclarationId(689),
                    },
                    Field {
                        label: "TaggedVariant".to_string(),
                        ty: DeclarationId(690),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 2606, 2780),
        });
        declarations.push(Declaration {
            id: DeclarationId(308),
            name: Some("CoproductWireContract".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "coproduct".to_string(),
                        ty: DeclarationId(305),
                    },
                    Field {
                        label: "encoding".to_string(),
                        ty: DeclarationId(307),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 3002, 3088),
        });
        declarations.push(Declaration {
            id: DeclarationId(309),
            name: Some("WireContract".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "variant_encoding".to_string(),
                    ty: DeclarationId(307),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 3233, 3290),
        });
        declarations.push(Declaration {
            id: DeclarationId(310),
            name: Some("WireFormat".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Json".to_string(),
                        ty: DeclarationId(691),
                    },
                    Field {
                        label: "Text".to_string(),
                        ty: DeclarationId(692),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 3481, 3510),
        });
        declarations.push(Declaration {
            id: DeclarationId(311),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/logic.dag", 304, 308),
        });
        declarations.push(Declaration {
            id: DeclarationId(312),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/logic.dag", 311, 316),
        });
        declarations.push(Declaration {
            id: DeclarationId(313),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(4),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1023, 1032),
        });
        declarations.push(Declaration {
            id: DeclarationId(314),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(4),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1136, 1145),
        });
        declarations.push(Declaration {
            id: DeclarationId(315),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(6),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1272, 1282),
        });
        declarations.push(Declaration {
            id: DeclarationId(316),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(6),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1306, 1316),
        });
        declarations.push(Declaration {
            id: DeclarationId(317),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(6),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1340, 1350),
        });
        declarations.push(Declaration {
            id: DeclarationId(318),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(6),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/bit.dag", 1375, 1385),
        });
        declarations.push(Declaration {
            id: DeclarationId(319),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "value".to_string(),
                    ty: DeclarationId(12),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/error_primitives.dag", 620, 636),
        });
        declarations.push(Declaration {
            id: DeclarationId(320),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "value".to_string(),
                    ty: DeclarationId(13),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/error_primitives.dag", 639, 657),
        });
        declarations.push(Declaration {
            id: DeclarationId(321),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/error_primitives.dag", 936, 948),
        });
        declarations.push(Declaration {
            id: DeclarationId(322),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/error_primitives.dag", 951, 959),
        });
        declarations.push(Declaration {
            id: DeclarationId(323),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(58),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 24186, 24212),
        });
        declarations.push(Declaration {
            id: DeclarationId(324),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(58),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 25393, 25419),
        });
        declarations.push(Declaration {
            id: DeclarationId(325),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(58),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 26550, 26576),
        });
        declarations.push(Declaration {
            id: DeclarationId(326),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(58),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 27676, 27702),
        });
        declarations.push(Declaration {
            id: DeclarationId(327),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(58),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 31087, 31113),
        });
        declarations.push(Declaration {
            id: DeclarationId(328),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(58),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 34394, 34420),
        });
        declarations.push(Declaration {
            id: DeclarationId(329),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(58),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 39723, 39749),
        });
        declarations.push(Declaration {
            id: DeclarationId(330),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(58),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 42637, 42663),
        });
        declarations.push(Declaration {
            id: DeclarationId(331),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 43456, 43468),
        });
        declarations.push(Declaration {
            id: DeclarationId(332),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(16), DeclarationId(16)],
                output: DeclarationId(16),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 4724, 4737),
        });
        declarations.push(Declaration {
            id: DeclarationId(333),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(18), DeclarationId(18)],
                output: DeclarationId(18),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 4843, 4856),
        });
        declarations.push(Declaration {
            id: DeclarationId(334),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(20), DeclarationId(20)],
                output: DeclarationId(20),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5060, 5073),
        });
        declarations.push(Declaration {
            id: DeclarationId(335),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(22), DeclarationId(22)],
                output: DeclarationId(22),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5182, 5195),
        });
        declarations.push(Declaration {
            id: DeclarationId(336),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(24), DeclarationId(24)],
                output: DeclarationId(24),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5333, 5346),
        });
        declarations.push(Declaration {
            id: DeclarationId(337),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(24)],
                output: DeclarationId(24),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5372, 5382),
        });
        declarations.push(Declaration {
            id: DeclarationId(338),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(26), DeclarationId(26)],
                output: DeclarationId(26),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5451, 5464),
        });
        declarations.push(Declaration {
            id: DeclarationId(339),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(26)],
                output: DeclarationId(26),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 5490, 5500),
        });
        declarations.push(Declaration {
            id: DeclarationId(340),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(32), DeclarationId(32)],
                output: DeclarationId(32),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 10528, 10541),
        });
        declarations.push(Declaration {
            id: DeclarationId(341),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(32), DeclarationId(32)],
                output: DeclarationId(32),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 10559, 10572),
        });
        declarations.push(Declaration {
            id: DeclarationId(342),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(34), DeclarationId(34)],
                output: DeclarationId(34),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 11014, 11027),
        });
        declarations.push(Declaration {
            id: DeclarationId(343),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(34), DeclarationId(34)],
                output: DeclarationId(34),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 11045, 11058),
        });
        declarations.push(Declaration {
            id: DeclarationId(344),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(36), DeclarationId(36)],
                output: DeclarationId(36),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 11218, 11231),
        });
        declarations.push(Declaration {
            id: DeclarationId(345),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(36)],
                output: DeclarationId(36),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 11252, 11262),
        });
        declarations.push(Declaration {
            id: DeclarationId(346),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(36), DeclarationId(36)],
                output: DeclarationId(36),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 11270, 11283),
        });
        declarations.push(Declaration {
            id: DeclarationId(347),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(38),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12084, 12097),
        });
        declarations.push(Declaration {
            id: DeclarationId(348),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(38),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12105, 12118),
        });
        declarations.push(Declaration {
            id: DeclarationId(349),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38)],
                output: DeclarationId(38),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12139, 12149),
        });
        declarations.push(Declaration {
            id: DeclarationId(350),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(38),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12157, 12170),
        });
        declarations.push(Declaration {
            id: DeclarationId(351),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(11),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(12),
                        value: DeclarationId(38),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(13),
                        value: DeclarationId(14),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12190, 12209),
        });
        declarations.push(Declaration {
            id: DeclarationId(352),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(351),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12178, 12209),
        });
        declarations.push(Declaration {
            id: DeclarationId(353),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(52),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12230, 12250),
        });
        declarations.push(Declaration {
            id: DeclarationId(354),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12257, 12273),
        });
        declarations.push(Declaration {
            id: DeclarationId(355),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12280, 12296),
        });
        declarations.push(Declaration {
            id: DeclarationId(356),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12303, 12319),
        });
        declarations.push(Declaration {
            id: DeclarationId(357),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12326, 12342),
        });
        declarations.push(Declaration {
            id: DeclarationId(358),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12349, 12365),
        });
        declarations.push(Declaration {
            id: DeclarationId(359),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12372, 12388),
        });
        declarations.push(Declaration {
            id: DeclarationId(360),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(40), DeclarationId(40)],
                output: DeclarationId(40),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12551, 12564),
        });
        declarations.push(Declaration {
            id: DeclarationId(361),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(40)],
                output: DeclarationId(40),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12585, 12595),
        });
        declarations.push(Declaration {
            id: DeclarationId(362),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(40), DeclarationId(40)],
                output: DeclarationId(40),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12603, 12616),
        });
        declarations.push(Declaration {
            id: DeclarationId(363),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(40)],
                output: DeclarationId(40),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12640, 12650),
        });
        declarations.push(Declaration {
            id: DeclarationId(364),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(40), DeclarationId(40)],
                output: DeclarationId(52),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 12662, 12682),
        });
        declarations.push(Declaration {
            id: DeclarationId(365),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(42), DeclarationId(42)],
                output: DeclarationId(42),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13030, 13043),
        });
        declarations.push(Declaration {
            id: DeclarationId(366),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(42), DeclarationId(42)],
                output: DeclarationId(42),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13052, 13065),
        });
        declarations.push(Declaration {
            id: DeclarationId(367),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(44), DeclarationId(44)],
                output: DeclarationId(44),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13208, 13221),
        });
        declarations.push(Declaration {
            id: DeclarationId(368),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(44), DeclarationId(44)],
                output: DeclarationId(44),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13230, 13243),
        });
        declarations.push(Declaration {
            id: DeclarationId(369),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(46), DeclarationId(46)],
                output: DeclarationId(46),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13497, 13510),
        });
        declarations.push(Declaration {
            id: DeclarationId(370),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(46), DeclarationId(46)],
                output: DeclarationId(46),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13519, 13532),
        });
        declarations.push(Declaration {
            id: DeclarationId(371),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(46)],
                output: DeclarationId(46),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 13547, 13557),
        });
        declarations.push(Declaration {
            id: DeclarationId(372),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17529, 17542),
        });
        declarations.push(Declaration {
            id: DeclarationId(373),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17544, 17557),
        });
        declarations.push(Declaration {
            id: DeclarationId(374),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17562, 17575),
        });
        declarations.push(Declaration {
            id: DeclarationId(375),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(372), DeclarationId(373)],
                output: DeclarationId(374),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17526, 17575),
        });
        declarations.push(Declaration {
            id: DeclarationId(376),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17585, 17598),
        });
        declarations.push(Declaration {
            id: DeclarationId(377),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17618, 17631),
        });
        declarations.push(Declaration {
            id: DeclarationId(378),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(377),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17609, 17631),
        });
        declarations.push(Declaration {
            id: DeclarationId(379),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17733, 17746),
        });
        declarations.push(Declaration {
            id: DeclarationId(380),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87), DeclarationId(87)],
                output: DeclarationId(379),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17717, 17746),
        });
        declarations.push(Declaration {
            id: DeclarationId(381),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(87),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17757, 17768),
        });
        declarations.push(Declaration {
            id: DeclarationId(382),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17781, 17793),
        });
        declarations.push(Declaration {
            id: DeclarationId(383),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(87),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17803, 17814),
        });
        declarations.push(Declaration {
            id: DeclarationId(384),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(48),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17832, 17834),
        });
        declarations.push(Declaration {
            id: DeclarationId(385),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(384),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17824, 17834),
        });
        declarations.push(Declaration {
            id: DeclarationId(386),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(48),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17851, 17853),
        });
        declarations.push(Declaration {
            id: DeclarationId(387),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(386),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17843, 17853),
        });
        declarations.push(Declaration {
            id: DeclarationId(388),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(48),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17951, 17961),
        });
        declarations.push(Declaration {
            id: DeclarationId(389),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17966, 17979),
        });
        declarations.push(Declaration {
            id: DeclarationId(390),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(388)],
                output: DeclarationId(389),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17948, 17979),
        });
        declarations.push(Declaration {
            id: DeclarationId(391),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17993, 18006),
        });
        declarations.push(Declaration {
            id: DeclarationId(392),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18011, 18024),
        });
        declarations.push(Declaration {
            id: DeclarationId(393),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(391)],
                output: DeclarationId(392),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 17990, 18024),
        });
        declarations.push(Declaration {
            id: DeclarationId(394),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48), DeclarationId(48)],
                output: DeclarationId(48),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18039, 18052),
        });
        declarations.push(Declaration {
            id: DeclarationId(395),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48), DeclarationId(394)],
                output: DeclarationId(48),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18033, 18058),
        });
        declarations.push(Declaration {
            id: DeclarationId(396),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18083, 18096),
        });
        declarations.push(Declaration {
            id: DeclarationId(397),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(396),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18074, 18096),
        });
        declarations.push(Declaration {
            id: DeclarationId(398),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18101, 18114),
        });
        declarations.push(Declaration {
            id: DeclarationId(399),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(397)],
                output: DeclarationId(398),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18071, 18114),
        });
        declarations.push(Declaration {
            id: DeclarationId(400),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18125, 18138),
        });
        declarations.push(Declaration {
            id: DeclarationId(401),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(400)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18122, 18147),
        });
        declarations.push(Declaration {
            id: DeclarationId(402),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18158, 18171),
        });
        declarations.push(Declaration {
            id: DeclarationId(403),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(402)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18155, 18180),
        });
        declarations.push(Declaration {
            id: DeclarationId(404),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(
                "Tuple".to_string(),
            )),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18245, 18258),
        });
        declarations.push(Declaration {
            id: DeclarationId(405),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(404),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18245, 18258),
        });
        declarations.push(Declaration {
            id: DeclarationId(406),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(405),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18234, 18259),
        });
        declarations.push(Declaration {
            id: DeclarationId(407),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(406),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18226, 18259),
        });
        declarations.push(Declaration {
            id: DeclarationId(408),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18279, 18292),
        });
        declarations.push(Declaration {
            id: DeclarationId(409),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(408),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18271, 18292),
        });
        declarations.push(Declaration {
            id: DeclarationId(410),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18312, 18325),
        });
        declarations.push(Declaration {
            id: DeclarationId(411),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(410),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18301, 18325),
        });
        declarations.push(Declaration {
            id: DeclarationId(412),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18345, 18358),
        });
        declarations.push(Declaration {
            id: DeclarationId(413),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(412),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18334, 18358),
        });
        declarations.push(Declaration {
            id: DeclarationId(414),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48), DeclarationId(48)],
                output: DeclarationId(87),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18373, 18388),
        });
        declarations.push(Declaration {
            id: DeclarationId(415),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(48),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18393, 18406),
        });
        declarations.push(Declaration {
            id: DeclarationId(416),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(414)],
                output: DeclarationId(415),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18370, 18406),
        });
        declarations.push(Declaration {
            id: DeclarationId(417),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18419, 18432),
        });
        declarations.push(Declaration {
            id: DeclarationId(418),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(51),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18997, 18999),
        });
        declarations.push(Declaration {
            id: DeclarationId(419),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50)],
                output: DeclarationId(418),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 18988, 18999),
        });
        declarations.push(Declaration {
            id: DeclarationId(420),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(49),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(50),
                        value: DeclarationId(50),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(51),
                        value: DeclarationId(51),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19009, 19030),
        });
        declarations.push(Declaration {
            id: DeclarationId(421),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(51),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19047, 19049),
        });
        declarations.push(Declaration {
            id: DeclarationId(422),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50)],
                output: DeclarationId(421),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19038, 19049),
        });
        declarations.push(Declaration {
            id: DeclarationId(423),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(49),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(50),
                        value: DeclarationId(50),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(51),
                        value: DeclarationId(51),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19072, 19093),
        });
        declarations.push(Declaration {
            id: DeclarationId(424),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50), DeclarationId(51)],
                output: DeclarationId(423),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19060, 19093),
        });
        declarations.push(Declaration {
            id: DeclarationId(425),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(49),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(50),
                        value: DeclarationId(50),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(51),
                        value: DeclarationId(51),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19106, 19127),
        });
        declarations.push(Declaration {
            id: DeclarationId(426),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(49),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(50),
                        value: DeclarationId(50),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(51),
                        value: DeclarationId(51),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19132, 19153),
        });
        declarations.push(Declaration {
            id: DeclarationId(427),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(425)],
                output: DeclarationId(426),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19103, 19153),
        });
        declarations.push(Declaration {
            id: DeclarationId(428),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(50),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19170, 19183),
        });
        declarations.push(Declaration {
            id: DeclarationId(429),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(428),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19162, 19183),
        });
        declarations.push(Declaration {
            id: DeclarationId(430),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(51),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19202, 19215),
        });
        declarations.push(Declaration {
            id: DeclarationId(431),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(430),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19194, 19215),
        });
        declarations.push(Declaration {
            id: DeclarationId(432),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19223, 19236),
        });
        declarations.push(Declaration {
            id: DeclarationId(433),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50)],
                output: DeclarationId(122),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19253, 19266),
        });
        declarations.push(Declaration {
            id: DeclarationId(434),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(87),
                body: ArrowBody::NoBody,
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19275, 19286),
        });
        declarations.push(Declaration {
            id: DeclarationId(435),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19505, 19509),
        });
        declarations.push(Declaration {
            id: DeclarationId(436),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19512, 19517),
        });
        declarations.push(Declaration {
            id: DeclarationId(437),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 19520, 19527),
        });
        declarations.push(Declaration {
            id: DeclarationId(438),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21636, 21654),
        });
        declarations.push(Declaration {
            id: DeclarationId(439),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21659, 21682),
        });
        declarations.push(Declaration {
            id: DeclarationId(440),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21687, 21708),
        });
        declarations.push(Declaration {
            id: DeclarationId(441),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21713, 21744),
        });
        declarations.push(Declaration {
            id: DeclarationId(442),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21749, 21772),
        });
        declarations.push(Declaration {
            id: DeclarationId(443),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21777, 21804),
        });
        declarations.push(Declaration {
            id: DeclarationId(444),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21809, 21831),
        });
        declarations.push(Declaration {
            id: DeclarationId(445),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21858, 21872),
        });
        declarations.push(Declaration {
            id: DeclarationId(446),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "name".to_string(),
                    ty: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21877, 21899),
        });
        declarations.push(Declaration {
            id: DeclarationId(447),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21930, 21942),
        });
        declarations.push(Declaration {
            id: DeclarationId(448),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21947, 21962),
        });
        declarations.push(Declaration {
            id: DeclarationId(449),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21967, 21978),
        });
        declarations.push(Declaration {
            id: DeclarationId(450),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 21983, 21996),
        });
        declarations.push(Declaration {
            id: DeclarationId(451),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "name".to_string(),
                    ty: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22001, 22031),
        });
        declarations.push(Declaration {
            id: DeclarationId(452),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "source".to_string(),
                        ty: DeclarationId(54),
                    },
                    Field {
                        label: "element".to_string(),
                        ty: DeclarationId(55),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22036, 22105),
        });
        declarations.push(Declaration {
            id: DeclarationId(453),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "inner".to_string(),
                    ty: DeclarationId(55),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22110, 22151),
        });
        declarations.push(Declaration {
            id: DeclarationId(454),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "first".to_string(),
                        ty: DeclarationId(55),
                    },
                    Field {
                        label: "second".to_string(),
                        ty: DeclarationId(55),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22156, 22223),
        });
        declarations.push(Declaration {
            id: DeclarationId(455),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(55),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22249, 22274),
        });
        declarations.push(Declaration {
            id: DeclarationId(456),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "params".to_string(),
                        ty: DeclarationId(455),
                    },
                    Field {
                        label: "return_type".to_string(),
                        ty: DeclarationId(55),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22228, 22310),
        });
        declarations.push(Declaration {
            id: DeclarationId(457),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "id".to_string(),
                    ty: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22315, 22349),
        });
        declarations.push(Declaration {
            id: DeclarationId(458),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22595, 22607),
        });
        declarations.push(Declaration {
            id: DeclarationId(459),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22665, 22681),
        });
        declarations.push(Declaration {
            id: DeclarationId(460),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 22728, 22742),
        });
        declarations.push(Declaration {
            id: DeclarationId(461),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23219, 23232),
        });
        declarations.push(Declaration {
            id: DeclarationId(462),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23248, 23263),
        });
        declarations.push(Declaration {
            id: DeclarationId(463),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23309, 23325),
        });
        declarations.push(Declaration {
            id: DeclarationId(464),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23372, 23385),
        });
        declarations.push(Declaration {
            id: DeclarationId(465),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(55),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23468, 23493),
        });
        declarations.push(Declaration {
            id: DeclarationId(466),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(56),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23544, 23565),
        });
        declarations.push(Declaration {
            id: DeclarationId(467),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(57),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23580, 23590),
        });
        declarations.push(Declaration {
            id: DeclarationId(468),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(87),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23835, 23839),
        });
        declarations.push(Declaration {
            id: DeclarationId(469),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(53),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/algebra.dag", 23872, 23899),
        });
        declarations.push(Declaration {
            id: DeclarationId(470),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/machine_constraints.dag", 7335, 7342),
        });
        declarations.push(Declaration {
            id: DeclarationId(471),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "8".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2587, 2588),
        });
        declarations.push(Declaration {
            id: DeclarationId(472),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(471),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2574, 2589),
        });
        declarations.push(Declaration {
            id: DeclarationId(473),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "16".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2630, 2632),
        });
        declarations.push(Declaration {
            id: DeclarationId(474),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(473),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2617, 2633),
        });
        declarations.push(Declaration {
            id: DeclarationId(475),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "32".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2674, 2676),
        });
        declarations.push(Declaration {
            id: DeclarationId(476),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(475),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2661, 2677),
        });
        declarations.push(Declaration {
            id: DeclarationId(477),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "64".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2718, 2720),
        });
        declarations.push(Declaration {
            id: DeclarationId(478),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(477),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2705, 2721),
        });
        declarations.push(Declaration {
            id: DeclarationId(479),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "128".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2763, 2766),
        });
        declarations.push(Declaration {
            id: DeclarationId(480),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(479),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2750, 2767),
        });
        declarations.push(Declaration {
            id: DeclarationId(481),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "8".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2874, 2875),
        });
        declarations.push(Declaration {
            id: DeclarationId(482),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(481),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2861, 2876),
        });
        declarations.push(Declaration {
            id: DeclarationId(483),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "16".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2919, 2921),
        });
        declarations.push(Declaration {
            id: DeclarationId(484),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(483),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2906, 2922),
        });
        declarations.push(Declaration {
            id: DeclarationId(485),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "32".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2965, 2967),
        });
        declarations.push(Declaration {
            id: DeclarationId(486),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(485),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2952, 2968),
        });
        declarations.push(Declaration {
            id: DeclarationId(487),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "64".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 3011, 3013),
        });
        declarations.push(Declaration {
            id: DeclarationId(488),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(487),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 2998, 3014),
        });
        declarations.push(Declaration {
            id: DeclarationId(489),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "128".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 3058, 3061),
        });
        declarations.push(Declaration {
            id: DeclarationId(490),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(489),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 3045, 3062),
        });
        declarations.push(Declaration {
            id: DeclarationId(491),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(27),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(28),
                    value: DeclarationId(76),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 5945, 5965),
        });
        declarations.push(Declaration {
            id: DeclarationId(492),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(72),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 7653, 7679),
        });
        declarations.push(Declaration {
            id: DeclarationId(493),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(72),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 7715, 7741),
        });
        declarations.push(Declaration {
            id: DeclarationId(494),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(76)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(2))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 9242, 9249),
        });
        declarations.push(Declaration {
            id: DeclarationId(495),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(29),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(30),
                    value: DeclarationId(87),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/rational.dag", 1267, 1288),
        });
        declarations.push(Declaration {
            id: DeclarationId(496),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1052, 1065),
        });
        declarations.push(Declaration {
            id: DeclarationId(497),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1070, 1076),
        });
        declarations.push(Declaration {
            id: DeclarationId(498),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1081, 1099),
        });
        declarations.push(Declaration {
            id: DeclarationId(499),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1104, 1122),
        });
        declarations.push(Declaration {
            id: DeclarationId(500),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1127, 1141),
        });
        declarations.push(Declaration {
            id: DeclarationId(501),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1423, 1432),
        });
        declarations.push(Declaration {
            id: DeclarationId(502),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "significand_bits".to_string(),
                        ty: DeclarationId(92),
                    },
                    Field {
                        label: "exponent_bits".to_string(),
                        ty: DeclarationId(92),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1437, 1514),
        });
        declarations.push(Declaration {
            id: DeclarationId(503),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "digits".to_string(),
                        ty: DeclarationId(92),
                    },
                    Field {
                        label: "exponent_digits".to_string(),
                        ty: DeclarationId(92),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1519, 1589),
        });
        declarations.push(Declaration {
            id: DeclarationId(504),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1829, 1834),
        });
        declarations.push(Declaration {
            id: DeclarationId(505),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1837, 1845),
        });
        declarations.push(Declaration {
            id: DeclarationId(506),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1848, 1868),
        });
        declarations.push(Declaration {
            id: DeclarationId(507),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2118, 2128),
        });
        declarations.push(Declaration {
            id: DeclarationId(508),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2131, 2145),
        });
        declarations.push(Declaration {
            id: DeclarationId(509),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2400, 2412),
        });
        declarations.push(Declaration {
            id: DeclarationId(510),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2415, 2425),
        });
        declarations.push(Declaration {
            id: DeclarationId(511),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2743, 2755),
        });
        declarations.push(Declaration {
            id: DeclarationId(512),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2758, 2774),
        });
        declarations.push(Declaration {
            id: DeclarationId(513),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2777, 2788),
        });
        declarations.push(Declaration {
            id: DeclarationId(514),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(39),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(40),
                    value: DeclarationId(102),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 3433, 3441),
        });
        declarations.push(Declaration {
            id: DeclarationId(515),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(29),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(30),
                    value: DeclarationId(87),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1482, 1503),
        });
        declarations.push(Declaration {
            id: DeclarationId(516),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "32".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1767, 1769),
        });
        declarations.push(Declaration {
            id: DeclarationId(517),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(516),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1754, 1770),
        });
        declarations.push(Declaration {
            id: DeclarationId(518),
            name: None,
            connective: TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(
                "64".to_string(),
            ))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1813, 1815),
        });
        declarations.push(Declaration {
            id: DeclarationId(519),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(518),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/float.dag", 1800, 1816),
        });
        declarations.push(Declaration {
            id: DeclarationId(520),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(87),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 3781, 3785),
        });
        declarations.push(Declaration {
            id: DeclarationId(521),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 4054, 4066),
        });
        declarations.push(Declaration {
            id: DeclarationId(522),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(225),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 4264, 4271),
        });
        declarations.push(Declaration {
            id: DeclarationId(523),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(225),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 6172, 6179),
        });
        declarations.push(Declaration {
            id: DeclarationId(524),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 6508, 6520),
        });
        declarations.push(Declaration {
            id: DeclarationId(525),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 7586, 7590),
        });
        declarations.push(Declaration {
            id: DeclarationId(526),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 7593, 7598),
        });
        declarations.push(Declaration {
            id: DeclarationId(527),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(45),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(46),
                    value: DeclarationId(122),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 7574, 7598),
        });
        declarations.push(Declaration {
            id: DeclarationId(528),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(148),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12432, 12449),
        });
        declarations.push(Declaration {
            id: DeclarationId(529),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(149),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12483, 12500),
        });
        declarations.push(Declaration {
            id: DeclarationId(530),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16447, 16452),
        });
        declarations.push(Declaration {
            id: DeclarationId(531),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16455, 16460),
        });
        declarations.push(Declaration {
            id: DeclarationId(532),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16463, 16470),
        });
        declarations.push(Declaration {
            id: DeclarationId(533),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16622, 16626),
        });
        declarations.push(Declaration {
            id: DeclarationId(534),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16629, 16638),
        });
        declarations.push(Declaration {
            id: DeclarationId(535),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16641, 16647),
        });
        declarations.push(Declaration {
            id: DeclarationId(536),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16650, 16653),
        });
        declarations.push(Declaration {
            id: DeclarationId(537),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16678, 16686),
        });
        declarations.push(Declaration {
            id: DeclarationId(538),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16689, 16698),
        });
        declarations.push(Declaration {
            id: DeclarationId(539),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16701, 16707),
        });
        declarations.push(Declaration {
            id: DeclarationId(540),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16727, 16729),
        });
        declarations.push(Declaration {
            id: DeclarationId(541),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16732, 16733),
        });
        declarations.push(Declaration {
            id: DeclarationId(542),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16736, 16737),
        });
        declarations.push(Declaration {
            id: DeclarationId(543),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16740, 16741),
        });
        declarations.push(Declaration {
            id: DeclarationId(544),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16744, 16746),
        });
        declarations.push(Declaration {
            id: DeclarationId(545),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "secret_name".to_string(),
                    ty: DeclarationId(145),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17018, 17053),
        });
        declarations.push(Declaration {
            id: DeclarationId(546),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "env_var".to_string(),
                    ty: DeclarationId(145),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17058, 17099),
        });
        declarations.push(Declaration {
            id: DeclarationId(547),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(175),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17174, 17194),
        });
        declarations.push(Declaration {
            id: DeclarationId(548),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17209, 17221),
        });
        declarations.push(Declaration {
            id: DeclarationId(549),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "audience".to_string(),
                        ty: DeclarationId(145),
                    },
                    Field {
                        label: "service_account".to_string(),
                        ty: DeclarationId(547),
                    },
                    Field {
                        label: "scopes".to_string(),
                        ty: DeclarationId(548),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17104, 17227),
        });
        declarations.push(Declaration {
            id: DeclarationId(550),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17258, 17270),
        });
        declarations.push(Declaration {
            id: DeclarationId(551),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "scopes".to_string(),
                    ty: DeclarationId(550),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17232, 17272),
        });
        declarations.push(Declaration {
            id: DeclarationId(552),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(180),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17294, 17314),
        });
        declarations.push(Declaration {
            id: DeclarationId(553),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "steps".to_string(),
                    ty: DeclarationId(552),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17277, 17316),
        });
        declarations.push(Declaration {
            id: DeclarationId(554),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17403, 17409),
        });
        declarations.push(Declaration {
            id: DeclarationId(555),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17412, 17415),
        });
        declarations.push(Declaration {
            id: DeclarationId(556),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17418, 17425),
        });
        declarations.push(Declaration {
            id: DeclarationId(557),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17428, 17431),
        });
        declarations.push(Declaration {
            id: DeclarationId(558),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17434, 17439),
        });
        declarations.push(Declaration {
            id: DeclarationId(559),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17442, 17446),
        });
        declarations.push(Declaration {
            id: DeclarationId(560),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17449, 17455),
        });
        declarations.push(Declaration {
            id: DeclarationId(561),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17458, 17464),
        });
        declarations.push(Declaration {
            id: DeclarationId(562),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17467, 17475),
        });
        declarations.push(Declaration {
            id: DeclarationId(563),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17478, 17485),
        });
        declarations.push(Declaration {
            id: DeclarationId(564),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17488, 17494),
        });
        declarations.push(Declaration {
            id: DeclarationId(565),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17509, 17522),
        });
        declarations.push(Declaration {
            id: DeclarationId(566),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17525, 17527),
        });
        declarations.push(Declaration {
            id: DeclarationId(567),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17530, 17535),
        });
        declarations.push(Declaration {
            id: DeclarationId(568),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17538, 17541),
        });
        declarations.push(Declaration {
            id: DeclarationId(569),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17552, 17557),
        });
        declarations.push(Declaration {
            id: DeclarationId(570),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17560, 17565),
        });
        declarations.push(Declaration {
            id: DeclarationId(571),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17568, 17575),
        });
        declarations.push(Declaration {
            id: DeclarationId(572),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17578, 17585),
        });
        declarations.push(Declaration {
            id: DeclarationId(573),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17588, 17595),
        });
        declarations.push(Declaration {
            id: DeclarationId(574),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17598, 17601),
        });
        declarations.push(Declaration {
            id: DeclarationId(575),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17604, 17608),
        });
        declarations.push(Declaration {
            id: DeclarationId(576),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17623, 17630),
        });
        declarations.push(Declaration {
            id: DeclarationId(577),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17633, 17636),
        });
        declarations.push(Declaration {
            id: DeclarationId(578),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17639, 17646),
        });
        declarations.push(Declaration {
            id: DeclarationId(579),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17649, 17658),
        });
        declarations.push(Declaration {
            id: DeclarationId(580),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17661, 17665),
        });
        declarations.push(Declaration {
            id: DeclarationId(581),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17668, 17672),
        });
        declarations.push(Declaration {
            id: DeclarationId(582),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17675, 17685),
        });
        declarations.push(Declaration {
            id: DeclarationId(583),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17688, 17692),
        });
        declarations.push(Declaration {
            id: DeclarationId(584),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17695, 17701),
        });
        declarations.push(Declaration {
            id: DeclarationId(585),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17722, 17728),
        });
        declarations.push(Declaration {
            id: DeclarationId(586),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17731, 17734),
        });
        declarations.push(Declaration {
            id: DeclarationId(587),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17737, 17746),
        });
        declarations.push(Declaration {
            id: DeclarationId(588),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17749, 17751),
        });
        declarations.push(Declaration {
            id: DeclarationId(589),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17754, 17762),
        });
        declarations.push(Declaration {
            id: DeclarationId(590),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(184),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 17830, 17837),
        });
        declarations.push(Declaration {
            id: DeclarationId(591),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18234, 18245),
        });
        declarations.push(Declaration {
            id: DeclarationId(592),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18250, 18259),
        });
        declarations.push(Declaration {
            id: DeclarationId(593),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18264, 18271),
        });
        declarations.push(Declaration {
            id: DeclarationId(594),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18276, 18283),
        });
        declarations.push(Declaration {
            id: DeclarationId(595),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18288, 18293),
        });
        declarations.push(Declaration {
            id: DeclarationId(596),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18446, 18456),
        });
        declarations.push(Declaration {
            id: DeclarationId(597),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18461, 18470),
        });
        declarations.push(Declaration {
            id: DeclarationId(598),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 18475, 18481),
        });
        declarations.push(Declaration {
            id: DeclarationId(599),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20784, 20787),
        });
        declarations.push(Declaration {
            id: DeclarationId(600),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20790, 20794),
        });
        declarations.push(Declaration {
            id: DeclarationId(601),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20797, 20800),
        });
        declarations.push(Declaration {
            id: DeclarationId(602),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20803, 20808),
        });
        declarations.push(Declaration {
            id: DeclarationId(603),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20811, 20817),
        });
        declarations.push(Declaration {
            id: DeclarationId(604),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20820, 20824),
        });
        declarations.push(Declaration {
            id: DeclarationId(605),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 20827, 20834),
        });
        declarations.push(Declaration {
            id: DeclarationId(606),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21322, 21328),
        });
        declarations.push(Declaration {
            id: DeclarationId(607),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "name".to_string(),
                    ty: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21333, 21356),
        });
        declarations.push(Declaration {
            id: DeclarationId(608),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "username".to_string(),
                    ty: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21361, 21387),
        });
        declarations.push(Declaration {
            id: DeclarationId(609),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21392, 21398),
        });
        declarations.push(Declaration {
            id: DeclarationId(610),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(154),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21590, 21600),
        });
        declarations.push(Declaration {
            id: DeclarationId(611),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(225),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21725, 21732),
        });
        declarations.push(Declaration {
            id: DeclarationId(612),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21772, 21784),
        });
        declarations.push(Declaration {
            id: DeclarationId(613),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(87),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21799, 21803),
        });
        declarations.push(Declaration {
            id: DeclarationId(614),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(208),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23154, 23172),
        });
        declarations.push(Declaration {
            id: DeclarationId(615),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(209),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23265, 23286),
        });
        declarations.push(Declaration {
            id: DeclarationId(616),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(225),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23570, 23577),
        });
        declarations.push(Declaration {
            id: DeclarationId(617),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(213),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23610, 23625),
        });
        declarations.push(Declaration {
            id: DeclarationId(618),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(216),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23914, 23932),
        });
        declarations.push(Declaration {
            id: DeclarationId(619),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(217),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23942, 23960),
        });
        declarations.push(Declaration {
            id: DeclarationId(620),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 23982, 23994),
        });
        declarations.push(Declaration {
            id: DeclarationId(621),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(225),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24082, 24089),
        });
        declarations.push(Declaration {
            id: DeclarationId(622),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(225),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24176, 24183),
        });
        declarations.push(Declaration {
            id: DeclarationId(623),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24211, 24223),
        });
        declarations.push(Declaration {
            id: DeclarationId(624),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24235, 24247),
        });
        declarations.push(Declaration {
            id: DeclarationId(625),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24259, 24271),
        });
        declarations.push(Declaration {
            id: DeclarationId(626),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(220),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24406, 24421),
        });
        declarations.push(Declaration {
            id: DeclarationId(627),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(186),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24432, 24445),
        });
        declarations.push(Declaration {
            id: DeclarationId(628),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(185),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24461, 24474),
        });
        declarations.push(Declaration {
            id: DeclarationId(629),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24500, 24504),
        });
        declarations.push(Declaration {
            id: DeclarationId(630),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24507, 24509),
        });
        declarations.push(Declaration {
            id: DeclarationId(631),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24512, 24513),
        });
        declarations.push(Declaration {
            id: DeclarationId(632),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24516, 24520),
        });
        declarations.push(Declaration {
            id: DeclarationId(633),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(225),
                    CardinalityBound::AtMostOne,
                ),
            ),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 24584, 24591),
        });
        declarations.push(Declaration {
            id: DeclarationId(634),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(122),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 3063, 3080),
        });
        declarations.push(Declaration {
            id: DeclarationId(635),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(87),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 3533, 3549),
        });
        declarations.push(Declaration {
            id: DeclarationId(636),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(122),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 4740, 4757),
        });
        declarations.push(Declaration {
            id: DeclarationId(637),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(131),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(132),
                        value: DeclarationId(225),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(133),
                        value: DeclarationId(225),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 5722, 5741),
        });
        declarations.push(Declaration {
            id: DeclarationId(638),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(20))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 8462, 8491),
        });
        declarations.push(Declaration {
            id: DeclarationId(639),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(26))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10887, 10908),
        });
        declarations.push(Declaration {
            id: DeclarationId(640),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(32))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10939, 10964),
        });
        declarations.push(Declaration {
            id: DeclarationId(641),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(38))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11095, 11120),
        });
        declarations.push(Declaration {
            id: DeclarationId(642),
            name: Some("<registered predicate not lowered: SecretValue>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11278, 11320),
        });
        declarations.push(Declaration {
            id: DeclarationId(643),
            name: Some("<registered predicate not lowered: NonEmptyStr>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11507, 11552),
        });
        declarations.push(Declaration {
            id: DeclarationId(644),
            name: Some("<registered predicate not lowered: LanguageId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11553, 11598),
        });
        declarations.push(Declaration {
            id: DeclarationId(645),
            name: Some("<registered predicate not lowered: SecretName>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11599, 11644),
        });
        declarations.push(Declaration {
            id: DeclarationId(646),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(44))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12318, 12338),
        });
        declarations.push(Declaration {
            id: DeclarationId(647),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(50))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12378, 12398),
        });
        declarations.push(Declaration {
            id: DeclarationId(648),
            name: Some("<registered predicate not lowered: FilePath>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12503, 12546),
        });
        declarations.push(Declaration {
            id: DeclarationId(649),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(53))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13382, 13395),
        });
        declarations.push(Declaration {
            id: DeclarationId(650),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(56))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13427, 13440),
        });
        declarations.push(Declaration {
            id: DeclarationId(651),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(65))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13472, 13508),
        });
        declarations.push(Declaration {
            id: DeclarationId(652),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(87)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(74))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13540, 13571),
        });
        declarations.push(Declaration {
            id: DeclarationId(653),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(80))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13777, 13797),
        });
        declarations.push(Declaration {
            id: DeclarationId(654),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(86))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14359, 14376),
        });
        declarations.push(Declaration {
            id: DeclarationId(655),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(92))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14415, 14431),
        });
        declarations.push(Declaration {
            id: DeclarationId(656),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(98))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14470, 14485),
        });
        declarations.push(Declaration {
            id: DeclarationId(657),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(104))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14524, 14543),
        });
        declarations.push(Declaration {
            id: DeclarationId(658),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(110))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14582, 14601),
        });
        declarations.push(Declaration {
            id: DeclarationId(659),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(116))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14640, 14657),
        });
        declarations.push(Declaration {
            id: DeclarationId(660),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(122))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14696, 14714),
        });
        declarations.push(Declaration {
            id: DeclarationId(661),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(128))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14753, 14771),
        });
        declarations.push(Declaration {
            id: DeclarationId(662),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(134))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14810, 14830),
        });
        declarations.push(Declaration {
            id: DeclarationId(663),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(140))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15105, 15132),
        });
        declarations.push(Declaration {
            id: DeclarationId(664),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(146))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15177, 15204),
        });
        declarations.push(Declaration {
            id: DeclarationId(665),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(152))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15247, 15272),
        });
        declarations.push(Declaration {
            id: DeclarationId(666),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(145)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(158))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15312, 15334),
        });
        declarations.push(Declaration {
            id: DeclarationId(667),
            name: Some("<registered predicate not lowered: GitRef>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15414, 15456),
        });
        declarations.push(Declaration {
            id: DeclarationId(668),
            name: Some("<registered predicate not lowered: GcpProjectId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15841, 15883),
        });
        declarations.push(Declaration {
            id: DeclarationId(669),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(152)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(164))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21921, 21946),
        });
        declarations.push(Declaration {
            id: DeclarationId(670),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(123)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(170))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 21982, 22004),
        });
        declarations.push(Declaration {
            id: DeclarationId(671),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(225)],
                output: DeclarationId(122),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(176))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22042, 22061),
        });
        declarations.push(Declaration {
            id: DeclarationId(672),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 3679, 3689),
        });
        declarations.push(Declaration {
            id: DeclarationId(673),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 3692, 3697),
        });
        declarations.push(Declaration {
            id: DeclarationId(674),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 3700, 3710),
        });
        declarations.push(Declaration {
            id: DeclarationId(675),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 3713, 3726),
        });
        declarations.push(Declaration {
            id: DeclarationId(676),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5310, 5319),
        });
        declarations.push(Declaration {
            id: DeclarationId(677),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5322, 5328),
        });
        declarations.push(Declaration {
            id: DeclarationId(678),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5331, 5335),
        });
        declarations.push(Declaration {
            id: DeclarationId(679),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(230),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5883, 5901),
        });
        declarations.push(Declaration {
            id: DeclarationId(680),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(87),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 6703, 6712),
        });
        declarations.push(Declaration {
            id: DeclarationId(681),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(127),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(128),
                    value: DeclarationId(230),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 6952, 6970),
        });
        declarations.push(Declaration {
            id: DeclarationId(682),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 1873, 1883),
        });
        declarations.push(Declaration {
            id: DeclarationId(683),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 1888, 1897),
        });
        declarations.push(Declaration {
            id: DeclarationId(684),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "prefix".to_string(),
                    ty: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 1902, 1944),
        });
        declarations.push(Declaration {
            id: DeclarationId(685),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "suffix".to_string(),
                    ty: DeclarationId(225),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 1949, 1991),
        });
        declarations.push(Declaration {
            id: DeclarationId(686),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "prefix".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "suffix".to_string(),
                        ty: DeclarationId(225),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 1996, 2060),
        });
        declarations.push(Declaration {
            id: DeclarationId(687),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "naming".to_string(),
                    ty: DeclarationId(306),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 2631, 2670),
        });
        declarations.push(Declaration {
            id: DeclarationId(688),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "tag_field".to_string(),
                        ty: DeclarationId(225),
                    },
                    Field {
                        label: "naming".to_string(),
                        ty: DeclarationId(306),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 2675, 2742),
        });
        declarations.push(Declaration {
            id: DeclarationId(689),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 2747, 2762),
        });
        declarations.push(Declaration {
            id: DeclarationId(690),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 2767, 2780),
        });
        declarations.push(Declaration {
            id: DeclarationId(691),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 3499, 3503),
        });
        declarations.push(Declaration {
            id: DeclarationId(692),
            name: None,
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/serialization.dag", 3506, 3510),
        });
        declarations
    }
}

fn bootstrapped_std_fixture_dag_ports() -> HashMap<PortId, Port> {
    {
        let mut ports = HashMap::new();
        ports.insert(
            PortId(0),
            Port {
                id: PortId(0),
                state: PortState::Resolved(TypeShape::new(DeclarationId(76))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(1),
            Port {
                id: PortId(1),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(0)),
            },
        );
        ports.insert(
            PortId(2),
            Port {
                id: PortId(2),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(1)),
            },
        );
        ports.insert(
            PortId(3),
            Port {
                id: PortId(3),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(4),
            Port {
                id: PortId(4),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(3)),
            },
        );
        ports.insert(
            PortId(5),
            Port {
                id: PortId(5),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(4)),
            },
        );
        ports.insert(
            PortId(6),
            Port {
                id: PortId(6),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(5)),
            },
        );
        ports.insert(
            PortId(7),
            Port {
                id: PortId(7),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(6)),
            },
        );
        ports.insert(
            PortId(8),
            Port {
                id: PortId(8),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(7)),
            },
        );
        ports.insert(
            PortId(9),
            Port {
                id: PortId(9),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(8)),
            },
        );
        ports.insert(
            PortId(10),
            Port {
                id: PortId(10),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(9)),
            },
        );
        ports.insert(
            PortId(11),
            Port {
                id: PortId(11),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(10)),
            },
        );
        ports.insert(
            PortId(12),
            Port {
                id: PortId(12),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(11)),
            },
        );
        ports.insert(
            PortId(13),
            Port {
                id: PortId(13),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(12)),
            },
        );
        ports.insert(
            PortId(14),
            Port {
                id: PortId(14),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(13)),
            },
        );
        ports.insert(
            PortId(15),
            Port {
                id: PortId(15),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(14)),
            },
        );
        ports.insert(
            PortId(16),
            Port {
                id: PortId(16),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(15)),
            },
        );
        ports.insert(
            PortId(17),
            Port {
                id: PortId(17),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(16)),
            },
        );
        ports.insert(
            PortId(18),
            Port {
                id: PortId(18),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(17)),
            },
        );
        ports.insert(
            PortId(19),
            Port {
                id: PortId(19),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(18)),
            },
        );
        ports.insert(
            PortId(20),
            Port {
                id: PortId(20),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(19)),
            },
        );
        ports.insert(
            PortId(21),
            Port {
                id: PortId(21),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(22),
            Port {
                id: PortId(22),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(21)),
            },
        );
        ports.insert(
            PortId(23),
            Port {
                id: PortId(23),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(22)),
            },
        );
        ports.insert(
            PortId(24),
            Port {
                id: PortId(24),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(23)),
            },
        );
        ports.insert(
            PortId(25),
            Port {
                id: PortId(25),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(24)),
            },
        );
        ports.insert(
            PortId(26),
            Port {
                id: PortId(26),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(25)),
            },
        );
        ports.insert(
            PortId(27),
            Port {
                id: PortId(27),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(28),
            Port {
                id: PortId(28),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(27)),
            },
        );
        ports.insert(
            PortId(29),
            Port {
                id: PortId(29),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(28)),
            },
        );
        ports.insert(
            PortId(30),
            Port {
                id: PortId(30),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(29)),
            },
        );
        ports.insert(
            PortId(31),
            Port {
                id: PortId(31),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(30)),
            },
        );
        ports.insert(
            PortId(32),
            Port {
                id: PortId(32),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(31)),
            },
        );
        ports.insert(
            PortId(33),
            Port {
                id: PortId(33),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(34),
            Port {
                id: PortId(34),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(33)),
            },
        );
        ports.insert(
            PortId(35),
            Port {
                id: PortId(35),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(34)),
            },
        );
        ports.insert(
            PortId(36),
            Port {
                id: PortId(36),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(35)),
            },
        );
        ports.insert(
            PortId(37),
            Port {
                id: PortId(37),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(36)),
            },
        );
        ports.insert(
            PortId(38),
            Port {
                id: PortId(38),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(37)),
            },
        );
        ports.insert(
            PortId(39),
            Port {
                id: PortId(39),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(40),
            Port {
                id: PortId(40),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(39)),
            },
        );
        ports.insert(
            PortId(41),
            Port {
                id: PortId(41),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(40)),
            },
        );
        ports.insert(
            PortId(42),
            Port {
                id: PortId(42),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(41)),
            },
        );
        ports.insert(
            PortId(43),
            Port {
                id: PortId(43),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(42)),
            },
        );
        ports.insert(
            PortId(44),
            Port {
                id: PortId(44),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(43)),
            },
        );
        ports.insert(
            PortId(45),
            Port {
                id: PortId(45),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(46),
            Port {
                id: PortId(46),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(45)),
            },
        );
        ports.insert(
            PortId(47),
            Port {
                id: PortId(47),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(46)),
            },
        );
        ports.insert(
            PortId(48),
            Port {
                id: PortId(48),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(47)),
            },
        );
        ports.insert(
            PortId(49),
            Port {
                id: PortId(49),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(48)),
            },
        );
        ports.insert(
            PortId(50),
            Port {
                id: PortId(50),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(49)),
            },
        );
        ports.insert(
            PortId(51),
            Port {
                id: PortId(51),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(52),
            Port {
                id: PortId(52),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(51)),
            },
        );
        ports.insert(
            PortId(53),
            Port {
                id: PortId(53),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(52)),
            },
        );
        ports.insert(
            PortId(54),
            Port {
                id: PortId(54),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(55),
            Port {
                id: PortId(55),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(54)),
            },
        );
        ports.insert(
            PortId(56),
            Port {
                id: PortId(56),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(55)),
            },
        );
        ports.insert(
            PortId(57),
            Port {
                id: PortId(57),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(58),
            Port {
                id: PortId(58),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(57)),
            },
        );
        ports.insert(
            PortId(59),
            Port {
                id: PortId(59),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(58)),
            },
        );
        ports.insert(
            PortId(60),
            Port {
                id: PortId(60),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(59)),
            },
        );
        ports.insert(
            PortId(61),
            Port {
                id: PortId(61),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(60)),
            },
        );
        ports.insert(
            PortId(62),
            Port {
                id: PortId(62),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(61)),
            },
        );
        ports.insert(
            PortId(63),
            Port {
                id: PortId(63),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(62)),
            },
        );
        ports.insert(
            PortId(64),
            Port {
                id: PortId(64),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(63)),
            },
        );
        ports.insert(
            PortId(65),
            Port {
                id: PortId(65),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(64)),
            },
        );
        ports.insert(
            PortId(66),
            Port {
                id: PortId(66),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(67),
            Port {
                id: PortId(67),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(66)),
            },
        );
        ports.insert(
            PortId(68),
            Port {
                id: PortId(68),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(67)),
            },
        );
        ports.insert(
            PortId(69),
            Port {
                id: PortId(69),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(68)),
            },
        );
        ports.insert(
            PortId(70),
            Port {
                id: PortId(70),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(69)),
            },
        );
        ports.insert(
            PortId(71),
            Port {
                id: PortId(71),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(70)),
            },
        );
        ports.insert(
            PortId(72),
            Port {
                id: PortId(72),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(71)),
            },
        );
        ports.insert(
            PortId(73),
            Port {
                id: PortId(73),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(72)),
            },
        );
        ports.insert(
            PortId(74),
            Port {
                id: PortId(74),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(73)),
            },
        );
        ports.insert(
            PortId(75),
            Port {
                id: PortId(75),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(76),
            Port {
                id: PortId(76),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(75)),
            },
        );
        ports.insert(
            PortId(77),
            Port {
                id: PortId(77),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(76)),
            },
        );
        ports.insert(
            PortId(78),
            Port {
                id: PortId(78),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(77)),
            },
        );
        ports.insert(
            PortId(79),
            Port {
                id: PortId(79),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(78)),
            },
        );
        ports.insert(
            PortId(80),
            Port {
                id: PortId(80),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(79)),
            },
        );
        ports.insert(
            PortId(81),
            Port {
                id: PortId(81),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(82),
            Port {
                id: PortId(82),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(81)),
            },
        );
        ports.insert(
            PortId(83),
            Port {
                id: PortId(83),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(82)),
            },
        );
        ports.insert(
            PortId(84),
            Port {
                id: PortId(84),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(83)),
            },
        );
        ports.insert(
            PortId(85),
            Port {
                id: PortId(85),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(84)),
            },
        );
        ports.insert(
            PortId(86),
            Port {
                id: PortId(86),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(85)),
            },
        );
        ports.insert(
            PortId(87),
            Port {
                id: PortId(87),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(88),
            Port {
                id: PortId(88),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(87)),
            },
        );
        ports.insert(
            PortId(89),
            Port {
                id: PortId(89),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(88)),
            },
        );
        ports.insert(
            PortId(90),
            Port {
                id: PortId(90),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(89)),
            },
        );
        ports.insert(
            PortId(91),
            Port {
                id: PortId(91),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(90)),
            },
        );
        ports.insert(
            PortId(92),
            Port {
                id: PortId(92),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(91)),
            },
        );
        ports.insert(
            PortId(93),
            Port {
                id: PortId(93),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(94),
            Port {
                id: PortId(94),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(93)),
            },
        );
        ports.insert(
            PortId(95),
            Port {
                id: PortId(95),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(94)),
            },
        );
        ports.insert(
            PortId(96),
            Port {
                id: PortId(96),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(95)),
            },
        );
        ports.insert(
            PortId(97),
            Port {
                id: PortId(97),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(96)),
            },
        );
        ports.insert(
            PortId(98),
            Port {
                id: PortId(98),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(97)),
            },
        );
        ports.insert(
            PortId(99),
            Port {
                id: PortId(99),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(100),
            Port {
                id: PortId(100),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(99)),
            },
        );
        ports.insert(
            PortId(101),
            Port {
                id: PortId(101),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(100)),
            },
        );
        ports.insert(
            PortId(102),
            Port {
                id: PortId(102),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(101)),
            },
        );
        ports.insert(
            PortId(103),
            Port {
                id: PortId(103),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(102)),
            },
        );
        ports.insert(
            PortId(104),
            Port {
                id: PortId(104),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(103)),
            },
        );
        ports.insert(
            PortId(105),
            Port {
                id: PortId(105),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(106),
            Port {
                id: PortId(106),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(105)),
            },
        );
        ports.insert(
            PortId(107),
            Port {
                id: PortId(107),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(106)),
            },
        );
        ports.insert(
            PortId(108),
            Port {
                id: PortId(108),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(107)),
            },
        );
        ports.insert(
            PortId(109),
            Port {
                id: PortId(109),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(108)),
            },
        );
        ports.insert(
            PortId(110),
            Port {
                id: PortId(110),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(109)),
            },
        );
        ports.insert(
            PortId(111),
            Port {
                id: PortId(111),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(112),
            Port {
                id: PortId(112),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(111)),
            },
        );
        ports.insert(
            PortId(113),
            Port {
                id: PortId(113),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(112)),
            },
        );
        ports.insert(
            PortId(114),
            Port {
                id: PortId(114),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(113)),
            },
        );
        ports.insert(
            PortId(115),
            Port {
                id: PortId(115),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(114)),
            },
        );
        ports.insert(
            PortId(116),
            Port {
                id: PortId(116),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(115)),
            },
        );
        ports.insert(
            PortId(117),
            Port {
                id: PortId(117),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(118),
            Port {
                id: PortId(118),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(117)),
            },
        );
        ports.insert(
            PortId(119),
            Port {
                id: PortId(119),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(118)),
            },
        );
        ports.insert(
            PortId(120),
            Port {
                id: PortId(120),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(119)),
            },
        );
        ports.insert(
            PortId(121),
            Port {
                id: PortId(121),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(120)),
            },
        );
        ports.insert(
            PortId(122),
            Port {
                id: PortId(122),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(121)),
            },
        );
        ports.insert(
            PortId(123),
            Port {
                id: PortId(123),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(124),
            Port {
                id: PortId(124),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(123)),
            },
        );
        ports.insert(
            PortId(125),
            Port {
                id: PortId(125),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(124)),
            },
        );
        ports.insert(
            PortId(126),
            Port {
                id: PortId(126),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(125)),
            },
        );
        ports.insert(
            PortId(127),
            Port {
                id: PortId(127),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(126)),
            },
        );
        ports.insert(
            PortId(128),
            Port {
                id: PortId(128),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(127)),
            },
        );
        ports.insert(
            PortId(129),
            Port {
                id: PortId(129),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(130),
            Port {
                id: PortId(130),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(129)),
            },
        );
        ports.insert(
            PortId(131),
            Port {
                id: PortId(131),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(130)),
            },
        );
        ports.insert(
            PortId(132),
            Port {
                id: PortId(132),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(131)),
            },
        );
        ports.insert(
            PortId(133),
            Port {
                id: PortId(133),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(132)),
            },
        );
        ports.insert(
            PortId(134),
            Port {
                id: PortId(134),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(133)),
            },
        );
        ports.insert(
            PortId(135),
            Port {
                id: PortId(135),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(136),
            Port {
                id: PortId(136),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(135)),
            },
        );
        ports.insert(
            PortId(137),
            Port {
                id: PortId(137),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(136)),
            },
        );
        ports.insert(
            PortId(138),
            Port {
                id: PortId(138),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(137)),
            },
        );
        ports.insert(
            PortId(139),
            Port {
                id: PortId(139),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(138)),
            },
        );
        ports.insert(
            PortId(140),
            Port {
                id: PortId(140),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(139)),
            },
        );
        ports.insert(
            PortId(141),
            Port {
                id: PortId(141),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(142),
            Port {
                id: PortId(142),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(141)),
            },
        );
        ports.insert(
            PortId(143),
            Port {
                id: PortId(143),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(142)),
            },
        );
        ports.insert(
            PortId(144),
            Port {
                id: PortId(144),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(143)),
            },
        );
        ports.insert(
            PortId(145),
            Port {
                id: PortId(145),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(144)),
            },
        );
        ports.insert(
            PortId(146),
            Port {
                id: PortId(146),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(145)),
            },
        );
        ports.insert(
            PortId(147),
            Port {
                id: PortId(147),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(148),
            Port {
                id: PortId(148),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(147)),
            },
        );
        ports.insert(
            PortId(149),
            Port {
                id: PortId(149),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(148)),
            },
        );
        ports.insert(
            PortId(150),
            Port {
                id: PortId(150),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(149)),
            },
        );
        ports.insert(
            PortId(151),
            Port {
                id: PortId(151),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(150)),
            },
        );
        ports.insert(
            PortId(152),
            Port {
                id: PortId(152),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(151)),
            },
        );
        ports.insert(
            PortId(153),
            Port {
                id: PortId(153),
                state: PortState::Resolved(TypeShape::new(DeclarationId(145))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(154),
            Port {
                id: PortId(154),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(153)),
            },
        );
        ports.insert(
            PortId(155),
            Port {
                id: PortId(155),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(154)),
            },
        );
        ports.insert(
            PortId(156),
            Port {
                id: PortId(156),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(155)),
            },
        );
        ports.insert(
            PortId(157),
            Port {
                id: PortId(157),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(156)),
            },
        );
        ports.insert(
            PortId(158),
            Port {
                id: PortId(158),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(157)),
            },
        );
        ports.insert(
            PortId(159),
            Port {
                id: PortId(159),
                state: PortState::Resolved(TypeShape::new(DeclarationId(152))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(160),
            Port {
                id: PortId(160),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(159)),
            },
        );
        ports.insert(
            PortId(161),
            Port {
                id: PortId(161),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(160)),
            },
        );
        ports.insert(
            PortId(162),
            Port {
                id: PortId(162),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(161)),
            },
        );
        ports.insert(
            PortId(163),
            Port {
                id: PortId(163),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(162)),
            },
        );
        ports.insert(
            PortId(164),
            Port {
                id: PortId(164),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(163)),
            },
        );
        ports.insert(
            PortId(165),
            Port {
                id: PortId(165),
                state: PortState::Resolved(TypeShape::new(DeclarationId(123))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(166),
            Port {
                id: PortId(166),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(165)),
            },
        );
        ports.insert(
            PortId(167),
            Port {
                id: PortId(167),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(166)),
            },
        );
        ports.insert(
            PortId(168),
            Port {
                id: PortId(168),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(167)),
            },
        );
        ports.insert(
            PortId(169),
            Port {
                id: PortId(169),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(168)),
            },
        );
        ports.insert(
            PortId(170),
            Port {
                id: PortId(170),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(169)),
            },
        );
        ports.insert(
            PortId(171),
            Port {
                id: PortId(171),
                state: PortState::Resolved(TypeShape::new(DeclarationId(225))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(172),
            Port {
                id: PortId(172),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(171)),
            },
        );
        ports.insert(
            PortId(173),
            Port {
                id: PortId(173),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(172)),
            },
        );
        ports.insert(
            PortId(174),
            Port {
                id: PortId(174),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(173)),
            },
        );
        ports.insert(
            PortId(175),
            Port {
                id: PortId(175),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(174)),
            },
        );
        ports.insert(
            PortId(176),
            Port {
                id: PortId(176),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(175)),
            },
        );
        ports.insert(
            PortId(177),
            Port {
                id: PortId(177),
                state: PortState::Resolved(TypeShape::new(DeclarationId(126))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(178),
            Port {
                id: PortId(178),
                state: PortState::Resolved(TypeShape::new(DeclarationId(226))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(179),
            Port {
                id: PortId(179),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(177)),
            },
        );
        ports.insert(
            PortId(180),
            Port {
                id: PortId(180),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(178)),
            },
        );
        ports.insert(
            PortId(181),
            Port {
                id: PortId(181),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(179)),
            },
        );
        ports.insert(
            PortId(182),
            Port {
                id: PortId(182),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(180)),
            },
        );
        ports.insert(
            PortId(183),
            Port {
                id: PortId(183),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(181)),
            },
        );
        ports.insert(
            PortId(184),
            Port {
                id: PortId(184),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(182)),
            },
        );
        ports.insert(
            PortId(185),
            Port {
                id: PortId(185),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(183)),
            },
        );
        ports.insert(
            PortId(186),
            Port {
                id: PortId(186),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(184)),
            },
        );
        ports.insert(
            PortId(187),
            Port {
                id: PortId(187),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(185)),
            },
        );
        ports.insert(
            PortId(188),
            Port {
                id: PortId(188),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(186)),
            },
        );
        ports.insert(
            PortId(189),
            Port {
                id: PortId(189),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(187)),
            },
        );
        ports.insert(
            PortId(190),
            Port {
                id: PortId(190),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(188)),
            },
        );
        ports.insert(
            PortId(191),
            Port {
                id: PortId(191),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(189)),
            },
        );
        ports.insert(
            PortId(192),
            Port {
                id: PortId(192),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(190)),
            },
        );
        ports.insert(
            PortId(193),
            Port {
                id: PortId(193),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(191)),
            },
        );
        ports.insert(
            PortId(194),
            Port {
                id: PortId(194),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(192)),
            },
        );
        ports.insert(
            PortId(195),
            Port {
                id: PortId(195),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(193)),
            },
        );
        ports.insert(
            PortId(196),
            Port {
                id: PortId(196),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(194)),
            },
        );
        ports.insert(
            PortId(197),
            Port {
                id: PortId(197),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(195)),
            },
        );
        ports.insert(
            PortId(198),
            Port {
                id: PortId(198),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(196)),
            },
        );
        ports.insert(
            PortId(199),
            Port {
                id: PortId(199),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(197)),
            },
        );
        ports.insert(
            PortId(200),
            Port {
                id: PortId(200),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(198)),
            },
        );
        ports.insert(
            PortId(201),
            Port {
                id: PortId(201),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(199)),
            },
        );
        ports.insert(
            PortId(202),
            Port {
                id: PortId(202),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(200)),
            },
        );
        ports.insert(
            PortId(203),
            Port {
                id: PortId(203),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(201)),
            },
        );
        ports.insert(
            PortId(204),
            Port {
                id: PortId(204),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(202)),
            },
        );
        ports.insert(
            PortId(205),
            Port {
                id: PortId(205),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(203)),
            },
        );
        ports.insert(
            PortId(206),
            Port {
                id: PortId(206),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(204)),
            },
        );
        ports.insert(
            PortId(207),
            Port {
                id: PortId(207),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(205)),
            },
        );
        ports.insert(
            PortId(208),
            Port {
                id: PortId(208),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(206)),
            },
        );
        ports.insert(
            PortId(209),
            Port {
                id: PortId(209),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(207)),
            },
        );
        ports.insert(
            PortId(210),
            Port {
                id: PortId(210),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(208)),
            },
        );
        ports.insert(
            PortId(211),
            Port {
                id: PortId(211),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(209)),
            },
        );
        ports.insert(
            PortId(212),
            Port {
                id: PortId(212),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(210)),
            },
        );
        ports.insert(
            PortId(213),
            Port {
                id: PortId(213),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(211)),
            },
        );
        ports.insert(
            PortId(214),
            Port {
                id: PortId(214),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(212)),
            },
        );
        ports.insert(
            PortId(215),
            Port {
                id: PortId(215),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(213)),
            },
        );
        ports.insert(
            PortId(216),
            Port {
                id: PortId(216),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(214)),
            },
        );
        ports.insert(
            PortId(217),
            Port {
                id: PortId(217),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(215)),
            },
        );
        ports.insert(
            PortId(218),
            Port {
                id: PortId(218),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(216)),
            },
        );
        ports.insert(
            PortId(219),
            Port {
                id: PortId(219),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(217)),
            },
        );
        ports.insert(
            PortId(220),
            Port {
                id: PortId(220),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(218)),
            },
        );
        ports.insert(
            PortId(221),
            Port {
                id: PortId(221),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(219)),
            },
        );
        ports.insert(
            PortId(222),
            Port {
                id: PortId(222),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(220)),
            },
        );
        ports.insert(
            PortId(223),
            Port {
                id: PortId(223),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(221)),
            },
        );
        ports.insert(
            PortId(224),
            Port {
                id: PortId(224),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(222)),
            },
        );
        ports.insert(
            PortId(225),
            Port {
                id: PortId(225),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(223)),
            },
        );
        ports.insert(
            PortId(226),
            Port {
                id: PortId(226),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(224)),
            },
        );
        ports.insert(
            PortId(227),
            Port {
                id: PortId(227),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(225)),
            },
        );
        ports.insert(
            PortId(228),
            Port {
                id: PortId(228),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(226)),
            },
        );
        ports.insert(
            PortId(229),
            Port {
                id: PortId(229),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(227)),
            },
        );
        ports.insert(
            PortId(230),
            Port {
                id: PortId(230),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(228)),
            },
        );
        ports.insert(
            PortId(231),
            Port {
                id: PortId(231),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(229)),
            },
        );
        ports.insert(
            PortId(232),
            Port {
                id: PortId(232),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(230)),
            },
        );
        ports.insert(
            PortId(233),
            Port {
                id: PortId(233),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(231)),
            },
        );
        ports.insert(
            PortId(234),
            Port {
                id: PortId(234),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(232)),
            },
        );
        ports.insert(
            PortId(235),
            Port {
                id: PortId(235),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(233)),
            },
        );
        ports.insert(
            PortId(236),
            Port {
                id: PortId(236),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(234)),
            },
        );
        ports.insert(
            PortId(237),
            Port {
                id: PortId(237),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(235)),
            },
        );
        ports.insert(
            PortId(238),
            Port {
                id: PortId(238),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(236)),
            },
        );
        ports.insert(
            PortId(239),
            Port {
                id: PortId(239),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(237)),
            },
        );
        ports.insert(
            PortId(240),
            Port {
                id: PortId(240),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(238)),
            },
        );
        ports.insert(
            PortId(241),
            Port {
                id: PortId(241),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(239)),
            },
        );
        ports.insert(
            PortId(242),
            Port {
                id: PortId(242),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(240)),
            },
        );
        ports.insert(
            PortId(243),
            Port {
                id: PortId(243),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(241)),
            },
        );
        ports.insert(
            PortId(244),
            Port {
                id: PortId(244),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(242)),
            },
        );
        ports.insert(
            PortId(245),
            Port {
                id: PortId(245),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(243)),
            },
        );
        ports.insert(
            PortId(246),
            Port {
                id: PortId(246),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(244)),
            },
        );
        ports.insert(
            PortId(247),
            Port {
                id: PortId(247),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(245)),
            },
        );
        ports.insert(
            PortId(248),
            Port {
                id: PortId(248),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(246)),
            },
        );
        ports.insert(
            PortId(249),
            Port {
                id: PortId(249),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(247)),
            },
        );
        ports.insert(
            PortId(250),
            Port {
                id: PortId(250),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(248)),
            },
        );
        ports.insert(
            PortId(251),
            Port {
                id: PortId(251),
                state: PortState::Resolved(TypeShape::new(DeclarationId(122))),
                produced_by: Some(NodeId(249)),
            },
        );
        ports.insert(
            PortId(252),
            Port {
                id: PortId(252),
                state: PortState::Resolved(TypeShape::new(DeclarationId(126))),
                produced_by: None,
            },
        );
        ports.insert(
            PortId(253),
            Port {
                id: PortId(253),
                state: PortState::Uninferred,
                produced_by: Some(NodeId(251)),
            },
        );
        ports.insert(
            PortId(254),
            Port {
                id: PortId(254),
                state: PortState::Resolved(TypeShape::new(DeclarationId(87))),
                produced_by: Some(NodeId(252)),
            },
        );
        ports
    }
}

fn bootstrapped_std_fixture_dag_diagnostics() -> DiagnosticTable {
    DiagnosticTable::new()
}

fn bootstrapped_std_fixture_dag_clusters() -> Vec<Cluster> {
    vec![]
}

fn bootstrapped_std_fixture_dag_optional_match_disjs() -> HashMap<DeclarationId, DeclarationId> {
    HashMap::new()
}
