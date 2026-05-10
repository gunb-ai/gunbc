// AUTO-GENERATED from `dsl/std/*.dag` via `regen_bootstrap`.
// Regenerate instead of hand-editing.

pub(crate) fn bootstrapped_std_fixture_dag() -> Dag {
    Dag {
        nodes: bootstrapped_std_fixture_dag_nodes(),
        declarations: bootstrapped_std_fixture_dag_declarations(),
        ports: bootstrapped_std_fixture_dag_ports(),
        diagnostics: bootstrapped_std_fixture_dag_diagnostics(),
        next_node_id: 33,
        next_declaration_id: 683,
        next_port_id: 33,
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
        declaration_append_begin_after_bootstrap: 683,
    }
}

#[allow(clippy::vec_init_then_push)]
fn bootstrapped_std_fixture_dag_nodes() -> Vec<Behavior> {
    {
        let mut nodes = Vec::with_capacity(33);
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(0),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(1),
            span: SourceSpan::new("dsl/std/integer.dag", 9551, 9558),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(1),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Gt)),
            inputs: vec![PortId(0), PortId(1)],
            output: PortId(2),
            span: SourceSpan::new("dsl/std/integer.dag", 9551, 9558),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(2),
            name: "<refinement:PositiveInt>".to_string(),
            value: PortId(2),
            params: vec![PortId(0)],
            span: SourceSpan::new("dsl/std/integer.dag", 9551, 9558),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(3),
            data: LiteralBits::Int("1".to_string()),
            output: PortId(4),
            span: SourceSpan::new("dsl/std/types.dag", 10914, 10935),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(4),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(3), PortId(4)],
            output: PortId(5),
            span: SourceSpan::new("dsl/std/types.dag", 10914, 10935),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(5),
            data: LiteralBits::Int("5".to_string()),
            output: PortId(6),
            span: SourceSpan::new("dsl/std/types.dag", 10914, 10935),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(6),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(3), PortId(6)],
            output: PortId(7),
            span: SourceSpan::new("dsl/std/types.dag", 10914, 10935),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(7),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(5), PortId(7)],
            output: PortId(8),
            span: SourceSpan::new("dsl/std/types.dag", 10914, 10935),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(8),
            name: "<refinement:RetryCount>".to_string(),
            value: PortId(8),
            params: vec![PortId(3)],
            span: SourceSpan::new("dsl/std/types.dag", 10914, 10935),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(9),
            data: LiteralBits::Int("100".to_string()),
            output: PortId(10),
            span: SourceSpan::new("dsl/std/types.dag", 10966, 10991),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(10),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(9), PortId(10)],
            output: PortId(11),
            span: SourceSpan::new("dsl/std/types.dag", 10966, 10991),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(11),
            data: LiteralBits::Int("599".to_string()),
            output: PortId(12),
            span: SourceSpan::new("dsl/std/types.dag", 10966, 10991),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(12),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(9), PortId(12)],
            output: PortId(13),
            span: SourceSpan::new("dsl/std/types.dag", 10966, 10991),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(13),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(11), PortId(13)],
            output: PortId(14),
            span: SourceSpan::new("dsl/std/types.dag", 10966, 10991),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(14),
            name: "<refinement:HttpStatus>".to_string(),
            value: PortId(14),
            params: vec![PortId(9)],
            span: SourceSpan::new("dsl/std/types.dag", 10966, 10991),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(15),
            data: LiteralBits::Int("1".to_string()),
            output: PortId(16),
            span: SourceSpan::new("dsl/std/types.dag", 11122, 11147),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(16),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(15), PortId(16)],
            output: PortId(17),
            span: SourceSpan::new("dsl/std/types.dag", 11122, 11147),
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(17),
            data: LiteralBits::Int("65535".to_string()),
            output: PortId(18),
            span: SourceSpan::new("dsl/std/types.dag", 11122, 11147),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(18),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Le)),
            inputs: vec![PortId(15), PortId(18)],
            output: PortId(19),
            span: SourceSpan::new("dsl/std/types.dag", 11122, 11147),
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(19),
            target: TransformTarget::Operator(OperatorKind::Logical(LogicalOp::And)),
            inputs: vec![PortId(17), PortId(19)],
            output: PortId(20),
            span: SourceSpan::new("dsl/std/types.dag", 11122, 11147),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(20),
            name: "<refinement:Port>".to_string(),
            value: PortId(20),
            params: vec![PortId(15)],
            span: SourceSpan::new("dsl/std/types.dag", 11122, 11147),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(21),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(22),
            span: SourceSpan::new("dsl/std/types.dag", 13409, 13422),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(22),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(21), PortId(22)],
            output: PortId(23),
            span: SourceSpan::new("dsl/std/types.dag", 13409, 13422),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(23),
            name: "<refinement:EpochMs>".to_string(),
            value: PortId(23),
            params: vec![PortId(21)],
            span: SourceSpan::new("dsl/std/types.dag", 13409, 13422),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(24),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(25),
            span: SourceSpan::new("dsl/std/types.dag", 13454, 13467),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(25),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(24), PortId(25)],
            output: PortId(26),
            span: SourceSpan::new("dsl/std/types.dag", 13454, 13467),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(26),
            name: "<refinement:Duration>".to_string(),
            value: PortId(26),
            params: vec![PortId(24)],
            span: SourceSpan::new("dsl/std/types.dag", 13454, 13467),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(27),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(28),
            span: SourceSpan::new("dsl/std/types.dag", 13980, 13993),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(28),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(27), PortId(28)],
            output: PortId(29),
            span: SourceSpan::new("dsl/std/types.dag", 13980, 13993),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(29),
            name: "<refinement:Milliseconds>".to_string(),
            value: PortId(29),
            params: vec![PortId(27)],
            span: SourceSpan::new("dsl/std/types.dag", 13980, 13993),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes.push(Behavior::Value(ValueNode {
            id: NodeId(30),
            data: LiteralBits::Int("0".to_string()),
            output: PortId(31),
            span: SourceSpan::new("dsl/std/types.dag", 14025, 14038),
            lane2_workflow: None,
        }));
        nodes.push(Behavior::Transform(TransformNode {
            id: NodeId(31),
            target: TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Ge)),
            inputs: vec![PortId(30), PortId(31)],
            output: PortId(32),
            span: SourceSpan::new("dsl/std/types.dag", 14025, 14038),
        }));
        nodes.push(Behavior::Bind(BindNode {
            id: NodeId(32),
            name: "<refinement:Seconds>".to_string(),
            value: PortId(32),
            params: vec![PortId(30)],
            span: SourceSpan::new("dsl/std/types.dag", 14025, 14038),
            lane2_workflow: None,
            emit_participation: None,
        }));
        nodes
    }
}

#[allow(clippy::vec_init_then_push)]
fn bootstrapped_std_fixture_dag_declarations() -> Vec<Declaration> {
    {
        let mut declarations = Vec::with_capacity(683);
        declarations.push(Declaration {
            id: DeclarationId(0),
            name: Some("Classical".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "True".to_string(),
                        ty: DeclarationId(316),
                    },
                    Field {
                        label: "False".to_string(),
                        ty: DeclarationId(317),
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
            span: SourceSpan::new("dsl/std/bit.dag", 1003, 1034),
        });
        declarations.push(Declaration {
            id: DeclarationId(6),
            name: Some("Byte".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "bits".to_string(),
                    ty: DeclarationId(319),
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
                    ty: DeclarationId(320),
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
                    ty: DeclarationId(321),
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
                    ty: DeclarationId(322),
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
                    ty: DeclarationId(323),
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
                        ty: DeclarationId(324),
                    },
                    Field {
                        label: "Err".to_string(),
                        ty: DeclarationId(325),
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
                        ty: DeclarationId(326),
                    },
                    Field {
                        label: "Overflow".to_string(),
                        ty: DeclarationId(327),
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
                    ty: DeclarationId(337),
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
                    ty: DeclarationId(338),
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
                        ty: DeclarationId(339),
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
                        ty: DeclarationId(340),
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
                        ty: DeclarationId(341),
                    },
                    Field {
                        label: "identity".to_string(),
                        ty: DeclarationId(24),
                    },
                    Field {
                        label: "inverse".to_string(),
                        ty: DeclarationId(342),
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
                        ty: DeclarationId(343),
                    },
                    Field {
                        label: "identity".to_string(),
                        ty: DeclarationId(26),
                    },
                    Field {
                        label: "inverse".to_string(),
                        ty: DeclarationId(344),
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
                        ty: DeclarationId(345),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(32),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(346),
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
                        ty: DeclarationId(347),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(34),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(348),
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
                        ty: DeclarationId(349),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(36),
                    },
                    Field {
                        label: "negate".to_string(),
                        ty: DeclarationId(350),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(351),
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
                        ty: DeclarationId(352),
                    },
                    Field {
                        label: "sub".to_string(),
                        ty: DeclarationId(353),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(38),
                    },
                    Field {
                        label: "negate".to_string(),
                        ty: DeclarationId(354),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(355),
                    },
                    Field {
                        label: "div".to_string(),
                        ty: DeclarationId(357),
                    },
                    Field {
                        label: "one".to_string(),
                        ty: DeclarationId(38),
                    },
                    Field {
                        label: "compare".to_string(),
                        ty: DeclarationId(358),
                    },
                    Field {
                        label: "eq".to_string(),
                        ty: DeclarationId(359),
                    },
                    Field {
                        label: "ne".to_string(),
                        ty: DeclarationId(360),
                    },
                    Field {
                        label: "lt".to_string(),
                        ty: DeclarationId(361),
                    },
                    Field {
                        label: "le".to_string(),
                        ty: DeclarationId(362),
                    },
                    Field {
                        label: "gt".to_string(),
                        ty: DeclarationId(363),
                    },
                    Field {
                        label: "ge".to_string(),
                        ty: DeclarationId(364),
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
                        ty: DeclarationId(365),
                    },
                    Field {
                        label: "zero".to_string(),
                        ty: DeclarationId(40),
                    },
                    Field {
                        label: "negate".to_string(),
                        ty: DeclarationId(366),
                    },
                    Field {
                        label: "mul".to_string(),
                        ty: DeclarationId(367),
                    },
                    Field {
                        label: "one".to_string(),
                        ty: DeclarationId(40),
                    },
                    Field {
                        label: "reciprocal".to_string(),
                        ty: DeclarationId(368),
                    },
                    Field {
                        label: "compare".to_string(),
                        ty: DeclarationId(369),
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
                        ty: DeclarationId(370),
                    },
                    Field {
                        label: "join".to_string(),
                        ty: DeclarationId(371),
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
                        ty: DeclarationId(372),
                    },
                    Field {
                        label: "join".to_string(),
                        ty: DeclarationId(373),
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
                        ty: DeclarationId(374),
                    },
                    Field {
                        label: "join".to_string(),
                        ty: DeclarationId(375),
                    },
                    Field {
                        label: "complement".to_string(),
                        ty: DeclarationId(376),
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
                        ty: DeclarationId(380),
                    },
                    Field {
                        label: "empty".to_string(),
                        ty: DeclarationId(381),
                    },
                    Field {
                        label: "append".to_string(),
                        ty: DeclarationId(383),
                    },
                    Field {
                        label: "slice".to_string(),
                        ty: DeclarationId(385),
                    },
                    Field {
                        label: "length".to_string(),
                        ty: DeclarationId(386),
                    },
                    Field {
                        label: "is_empty".to_string(),
                        ty: DeclarationId(387),
                    },
                    Field {
                        label: "count".to_string(),
                        ty: DeclarationId(388),
                    },
                    Field {
                        label: "first".to_string(),
                        ty: DeclarationId(390),
                    },
                    Field {
                        label: "last".to_string(),
                        ty: DeclarationId(392),
                    },
                    Field {
                        label: "map".to_string(),
                        ty: DeclarationId(395),
                    },
                    Field {
                        label: "filter".to_string(),
                        ty: DeclarationId(398),
                    },
                    Field {
                        label: "fold".to_string(),
                        ty: DeclarationId(400),
                    },
                    Field {
                        label: "flat_map".to_string(),
                        ty: DeclarationId(404),
                    },
                    Field {
                        label: "any".to_string(),
                        ty: DeclarationId(406),
                    },
                    Field {
                        label: "all".to_string(),
                        ty: DeclarationId(408),
                    },
                    Field {
                        label: "enumerate".to_string(),
                        ty: DeclarationId(412),
                    },
                    Field {
                        label: "reverse".to_string(),
                        ty: DeclarationId(414),
                    },
                    Field {
                        label: "skip".to_string(),
                        ty: DeclarationId(416),
                    },
                    Field {
                        label: "take".to_string(),
                        ty: DeclarationId(418),
                    },
                    Field {
                        label: "sort_by".to_string(),
                        ty: DeclarationId(421),
                    },
                    Field {
                        label: "contains".to_string(),
                        ty: DeclarationId(422),
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
                        ty: DeclarationId(424),
                    },
                    Field {
                        label: "empty".to_string(),
                        ty: DeclarationId(425),
                    },
                    Field {
                        label: "get".to_string(),
                        ty: DeclarationId(427),
                    },
                    Field {
                        label: "insert".to_string(),
                        ty: DeclarationId(429),
                    },
                    Field {
                        label: "merge".to_string(),
                        ty: DeclarationId(432),
                    },
                    Field {
                        label: "keys".to_string(),
                        ty: DeclarationId(434),
                    },
                    Field {
                        label: "values".to_string(),
                        ty: DeclarationId(436),
                    },
                    Field {
                        label: "has".to_string(),
                        ty: DeclarationId(437),
                    },
                    Field {
                        label: "contains_key".to_string(),
                        ty: DeclarationId(438),
                    },
                    Field {
                        label: "size".to_string(),
                        ty: DeclarationId(439),
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
                        ty: DeclarationId(440),
                    },
                    Field {
                        label: "Equal".to_string(),
                        ty: DeclarationId(441),
                    },
                    Field {
                        label: "Greater".to_string(),
                        ty: DeclarationId(442),
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
                        ty: DeclarationId(443),
                    },
                    Field {
                        label: "ApproximateFieldProfile".to_string(),
                        ty: DeclarationId(444),
                    },
                    Field {
                        label: "BooleanAlgebraProfile".to_string(),
                        ty: DeclarationId(445),
                    },
                    Field {
                        label: "BooleanAlgebraCollectionProfile".to_string(),
                        ty: DeclarationId(446),
                    },
                    Field {
                        label: "FreeMonoidScalarProfile".to_string(),
                        ty: DeclarationId(447),
                    },
                    Field {
                        label: "FreeMonoidCollectionProfile".to_string(),
                        ty: DeclarationId(448),
                    },
                    Field {
                        label: "PartialFunctionProfile".to_string(),
                        ty: DeclarationId(449),
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
                        ty: DeclarationId(450),
                    },
                    Field {
                        label: "Named".to_string(),
                        ty: DeclarationId(451),
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
                        ty: DeclarationId(452),
                    },
                    Field {
                        label: "ReceiverElement".to_string(),
                        ty: DeclarationId(453),
                    },
                    Field {
                        label: "ReceiverKey".to_string(),
                        ty: DeclarationId(454),
                    },
                    Field {
                        label: "ReceiverValue".to_string(),
                        ty: DeclarationId(455),
                    },
                    Field {
                        label: "NamedTemplate".to_string(),
                        ty: DeclarationId(456),
                    },
                    Field {
                        label: "ContainerOf".to_string(),
                        ty: DeclarationId(457),
                    },
                    Field {
                        label: "OptionalOf".to_string(),
                        ty: DeclarationId(458),
                    },
                    Field {
                        label: "TupleOf".to_string(),
                        ty: DeclarationId(459),
                    },
                    Field {
                        label: "CallableOf".to_string(),
                        ty: DeclarationId(461),
                    },
                    Field {
                        label: "AlgebraTypeVariable".to_string(),
                        ty: DeclarationId(462),
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
                        ty: DeclarationId(463),
                    },
                    Field {
                        label: "ProjectionEffect".to_string(),
                        ty: DeclarationId(464),
                    },
                    Field {
                        label: "IdentityEffect".to_string(),
                        ty: DeclarationId(465),
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
                        ty: DeclarationId(466),
                    },
                    Field {
                        label: "ShapeLinearScan".to_string(),
                        ty: DeclarationId(467),
                    },
                    Field {
                        label: "ShapeIterateBody".to_string(),
                        ty: DeclarationId(468),
                    },
                    Field {
                        label: "ShapeSortBody".to_string(),
                        ty: DeclarationId(469),
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
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "param_types".to_string(),
                        ty: DeclarationId(470),
                    },
                    Field {
                        label: "return_type".to_string(),
                        ty: DeclarationId(55),
                    },
                    Field {
                        label: "size_effect".to_string(),
                        ty: DeclarationId(471),
                    },
                    Field {
                        label: "cost_shape".to_string(),
                        ty: DeclarationId(472),
                    },
                    Field {
                        label: "callback_element_position".to_string(),
                        ty: DeclarationId(473),
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
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(53),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(474)),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Map(
                FieldMap::from_entries(vec![
                    (
                        "Int".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(443),
                            payload: vec![],
                        },
                    ),
                    (
                        "Float".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(444),
                            payload: vec![],
                        },
                    ),
                    (
                        "Bool".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(445),
                            payload: vec![],
                        },
                    ),
                    (
                        "String".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(447),
                            payload: vec![],
                        },
                    ),
                    (
                        "List".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(448),
                            payload: vec![],
                        },
                    ),
                    (
                        "Set".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(446),
                            payload: vec![],
                        },
                    ),
                    (
                        "Map".to_string(),
                        FieldValue::Variant {
                            constructor: DeclarationId(449),
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
                output: DeclarationId(328),
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
                output: DeclarationId(329),
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
                output: DeclarationId(330),
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
                output: DeclarationId(331),
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
                output: DeclarationId(332),
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
                output: DeclarationId(333),
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
                output: DeclarationId(334),
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
                output: DeclarationId(335),
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
                output: DeclarationId(336),
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
                    ty: DeclarationId(475),
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
                        value: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2252, 2296),
        });
        declarations.push(Declaration {
            id: DeclarationId(78),
            name: Some("Int16".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(93),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(477),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2297, 2344),
        });
        declarations.push(Declaration {
            id: DeclarationId(79),
            name: Some("Int32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2345, 2392),
        });
        declarations.push(Declaration {
            id: DeclarationId(80),
            name: Some("Int64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(93),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(479),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2393, 2440),
        });
        declarations.push(Declaration {
            id: DeclarationId(81),
            name: Some("Int128".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2441, 2490),
        });
        declarations.push(Declaration {
            id: DeclarationId(82),
            name: Some("UInt8".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(94),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(481),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2556, 2602),
        });
        declarations.push(Declaration {
            id: DeclarationId(83),
            name: Some("UInt16".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(94),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2603, 2652),
        });
        declarations.push(Declaration {
            id: DeclarationId(84),
            name: Some("UInt32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(94),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(483),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2653, 2702),
        });
        declarations.push(Declaration {
            id: DeclarationId(85),
            name: Some("UInt64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(94),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2703, 2752),
        });
        declarations.push(Declaration {
            id: DeclarationId(86),
            name: Some("UInt128".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(94),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(485),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2753, 2804),
        });
        declarations.push(Declaration {
            id: DeclarationId(87),
            name: Some("IntW32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(79),
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
            span: SourceSpan::new("dsl/std/integer.dag", 4048, 4068),
        });
        declarations.push(Declaration {
            id: DeclarationId(88),
            name: Some("IntW64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(80),
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
            span: SourceSpan::new("dsl/std/integer.dag", 4069, 4089),
        });
        declarations.push(Declaration {
            id: DeclarationId(89),
            name: Some("IntW128".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(81),
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
            span: SourceSpan::new("dsl/std/integer.dag", 4090, 4111),
        });
        declarations.push(Declaration {
            id: DeclarationId(90),
            name: Some("UIntW32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(84),
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
            span: SourceSpan::new("dsl/std/integer.dag", 4112, 4134),
        });
        declarations.push(Declaration {
            id: DeclarationId(91),
            name: Some("UIntW64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(85),
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
            span: SourceSpan::new("dsl/std/integer.dag", 4135, 4157),
        });
        declarations.push(Declaration {
            id: DeclarationId(92),
            name: Some("UIntW128".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(86),
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
            span: SourceSpan::new("dsl/std/integer.dag", 4158, 4181),
        });
        declarations.push(Declaration {
            id: DeclarationId(93),
            name: Some("Int".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(25),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(26),
                    value: DeclarationId(486),
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
            span: SourceSpan::new("dsl/std/integer.dag", 6230, 6275),
        });
        declarations.push(Declaration {
            id: DeclarationId(94),
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
            span: SourceSpan::new("dsl/std/integer.dag", 6276, 6291),
        });
        declarations.push(Declaration {
            id: DeclarationId(95),
            name: Some("IntPlatform".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(93),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(487),
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
            span: SourceSpan::new("dsl/std/integer.dag", 7928, 7989),
        });
        declarations.push(Declaration {
            id: DeclarationId(96),
            name: Some("UIntPlatform".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(94),
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
            span: SourceSpan::new("dsl/std/integer.dag", 7990, 8051),
        });
        declarations.push(Declaration {
            id: DeclarationId(97),
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
            span: SourceSpan::new("dsl/std/integer.dag", 9493, 9518),
        });
        declarations.push(Declaration {
            id: DeclarationId(98),
            name: Some("PositiveInt".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(76))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(489)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/integer.dag", 9519, 9558),
        });
        declarations.push(Declaration {
            id: DeclarationId(99),
            name: Some("Rational".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(39),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(40),
                    value: DeclarationId(490),
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
            id: DeclarationId(100),
            name: Some("RoundingMode".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "ToNearestEven".to_string(),
                        ty: DeclarationId(491),
                    },
                    Field {
                        label: "ToZero".to_string(),
                        ty: DeclarationId(492),
                    },
                    Field {
                        label: "ToPositiveInfinity".to_string(),
                        ty: DeclarationId(493),
                    },
                    Field {
                        label: "ToNegativeInfinity".to_string(),
                        ty: DeclarationId(494),
                    },
                    Field {
                        label: "ToAwayFromZero".to_string(),
                        ty: DeclarationId(495),
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
            id: DeclarationId(101),
            name: Some("Precision".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Unbounded".to_string(),
                        ty: DeclarationId(496),
                    },
                    Field {
                        label: "BinaryPrecision".to_string(),
                        ty: DeclarationId(497),
                    },
                    Field {
                        label: "DecimalPrecision".to_string(),
                        ty: DeclarationId(498),
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
            id: DeclarationId(102),
            name: Some("NanPolicy".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoNaN".to_string(),
                        ty: DeclarationId(499),
                    },
                    Field {
                        label: "QuietNaN".to_string(),
                        ty: DeclarationId(500),
                    },
                    Field {
                        label: "QuietAndSignalingNaN".to_string(),
                        ty: DeclarationId(501),
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
            id: DeclarationId(103),
            name: Some("InfinityPolicy".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoInfinity".to_string(),
                        ty: DeclarationId(502),
                    },
                    Field {
                        label: "SignedInfinity".to_string(),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2096, 2145),
        });
        declarations.push(Declaration {
            id: DeclarationId(104),
            name: Some("SignedZeroPolicy".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoSignedZero".to_string(),
                        ty: DeclarationId(504),
                    },
                    Field {
                        label: "SignedZero".to_string(),
                        ty: DeclarationId(505),
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
            id: DeclarationId(105),
            name: Some("SubnormalPolicy".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoSubnormals".to_string(),
                        ty: DeclarationId(506),
                    },
                    Field {
                        label: "GradualUnderflow".to_string(),
                        ty: DeclarationId(507),
                    },
                    Field {
                        label: "FlushToZero".to_string(),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2720, 2788),
        });
        declarations.push(Declaration {
            id: DeclarationId(106),
            name: Some("SpecialValues".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "nan".to_string(),
                        ty: DeclarationId(102),
                    },
                    Field {
                        label: "infinity".to_string(),
                        ty: DeclarationId(103),
                    },
                    Field {
                        label: "signed_zero".to_string(),
                        ty: DeclarationId(104),
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
            id: DeclarationId(107),
            name: Some("ApproximateField".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "base".to_string(),
                        ty: DeclarationId(509),
                    },
                    Field {
                        label: "rounding".to_string(),
                        ty: DeclarationId(100),
                    },
                    Field {
                        label: "precision".to_string(),
                        ty: DeclarationId(101),
                    },
                    Field {
                        label: "special_values".to_string(),
                        ty: DeclarationId(106),
                    },
                    Field {
                        label: "subnormal_policy".to_string(),
                        ty: DeclarationId(105),
                    },
                ],
            },
            type_params: vec![DeclarationId(108)],
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
            id: DeclarationId(108),
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
            id: DeclarationId(109),
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
            span: SourceSpan::new("dsl/std/float.dag", 1248, 1265),
        });
        declarations.push(Declaration {
            id: DeclarationId(110),
            name: Some("Real".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(107),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(108),
                    value: DeclarationId(510),
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
            span: SourceSpan::new("dsl/std/float.dag", 1446, 1497),
        });
        declarations.push(Declaration {
            id: DeclarationId(111),
            name: Some("Real32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(110),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(511),
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
            span: SourceSpan::new("dsl/std/float.dag", 1719, 1768),
        });
        declarations.push(Declaration {
            id: DeclarationId(112),
            name: Some("Real64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(73),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(74),
                        value: DeclarationId(110),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(75),
                        value: DeclarationId(512),
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
            span: SourceSpan::new("dsl/std/float.dag", 1769, 1818),
        });
        declarations.push(Declaration {
            id: DeclarationId(113),
            name: Some("Float32".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(111),
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
            span: SourceSpan::new("dsl/std/float.dag", 1820, 1841),
        });
        declarations.push(Declaration {
            id: DeclarationId(114),
            name: Some("Float64".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(112),
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
            span: SourceSpan::new("dsl/std/float.dag", 1842, 1863),
        });
        declarations.push(Declaration {
            id: DeclarationId(115),
            name: Some("Float".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(114),
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
            span: SourceSpan::new("dsl/std/float.dag", 1882, 1902),
        });
        declarations.push(Declaration {
            id: DeclarationId(116),
            name: Some("kernel_type_set".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(128),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(626)),
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
            id: DeclarationId(117),
            name: Some("is_kernel_type".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230)],
                output: DeclarationId(128),
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
            id: DeclarationId(118),
            name: Some("container_type_arity".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(93),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(627)),
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
            id: DeclarationId(119),
            name: Some("is_container_type".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230)],
                output: DeclarationId(128),
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
            id: DeclarationId(120),
            name: Some("container_expected_arity".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230)],
                output: DeclarationId(513),
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
            id: DeclarationId(121),
            name: Some("container_param_names_for".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230)],
                output: DeclarationId(514),
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
            id: DeclarationId(122),
            name: Some("container_param_name".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230), DeclarationId(93)],
                output: DeclarationId(515),
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
            id: DeclarationId(123),
            name: Some("ordered_element_collections".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(128),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(628)),
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
            id: DeclarationId(124),
            name: Some("is_ordered_element_collection".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230)],
                output: DeclarationId(128),
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
            id: DeclarationId(125),
            name: Some("container_template_algebra_rows".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(230),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(629)),
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
            id: DeclarationId(126),
            name: Some("container_template_algebra".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230)],
                output: DeclarationId(516),
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
            id: DeclarationId(127),
            name: Some("canonical_container_names".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(517),
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
            id: DeclarationId(128),
            name: Some("Bool".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "True".to_string(),
                        ty: DeclarationId(518),
                    },
                    Field {
                        label: "False".to_string(),
                        ty: DeclarationId(519),
                    },
                ],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: Some(DeclarationId(682)),
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 7574, 7598),
        });
        declarations.push(Declaration {
            id: DeclarationId(129),
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
            id: DeclarationId(130),
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
            id: DeclarationId(131),
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
            id: DeclarationId(132),
            name: Some("Char".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/types.dag", 8503, 8518),
        });
        declarations.push(Declaration {
            id: DeclarationId(133),
            name: Some("List".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(134),
                }],
            },
            type_params: vec![DeclarationId(134)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9282, 9322),
        });
        declarations.push(Declaration {
            id: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 9282, 9322),
        });
        declarations.push(Declaration {
            id: DeclarationId(135),
            name: Some("Set".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(45),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(46),
                    value: DeclarationId(136),
                }],
            },
            type_params: vec![DeclarationId(136)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9323, 9366),
        });
        declarations.push(Declaration {
            id: DeclarationId(136),
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
            span: SourceSpan::new("dsl/std/types.dag", 9323, 9366),
        });
        declarations.push(Declaration {
            id: DeclarationId(137),
            name: Some("Map".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(49),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(50),
                        value: DeclarationId(138),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(51),
                        value: DeclarationId(139),
                    },
                ],
            },
            type_params: vec![DeclarationId(138), DeclarationId(139)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 9367, 9417),
        });
        declarations.push(Declaration {
            id: DeclarationId(138),
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
            span: SourceSpan::new("dsl/std/types.dag", 9367, 9417),
        });
        declarations.push(Declaration {
            id: DeclarationId(139),
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
            span: SourceSpan::new("dsl/std/types.dag", 9367, 9417),
        });
        declarations.push(Declaration {
            id: DeclarationId(140),
            name: Some("CommitSha".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 10763, 10789),
        });
        declarations.push(Declaration {
            id: DeclarationId(141),
            name: Some("Sha256".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 10857, 10883),
        });
        declarations.push(Declaration {
            id: DeclarationId(142),
            name: Some("RetryCount".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(93))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(630)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10884, 10935),
        });
        declarations.push(Declaration {
            id: DeclarationId(143),
            name: Some("HttpStatus".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(93))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(631)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10936, 10991),
        });
        declarations.push(Declaration {
            id: DeclarationId(144),
            name: Some("Email".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 11065, 11091),
        });
        declarations.push(Declaration {
            id: DeclarationId(145),
            name: Some("Port".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(93))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(632)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11092, 11147),
        });
        declarations.push(Declaration {
            id: DeclarationId(146),
            name: Some("GistId".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 11236, 11262),
        });
        declarations.push(Declaration {
            id: DeclarationId(147),
            name: Some("Secret".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 11263, 11304),
        });
        declarations.push(Declaration {
            id: DeclarationId(148),
            name: Some("SecretValue".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(147))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(633)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11305, 11347),
        });
        declarations.push(Declaration {
            id: DeclarationId(149),
            name: Some("Url".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 11408, 11434),
        });
        declarations.push(Declaration {
            id: DeclarationId(150),
            name: Some("SemVer".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 11507, 11533),
        });
        declarations.push(Declaration {
            id: DeclarationId(151),
            name: Some("NonEmptyStr".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(230))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(634)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11534, 11579),
        });
        declarations.push(Declaration {
            id: DeclarationId(152),
            name: Some("LanguageId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(230))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(635)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11580, 11625),
        });
        declarations.push(Declaration {
            id: DeclarationId(153),
            name: Some("SecretName".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(230))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(636)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 11626, 11671),
        });
        declarations.push(Declaration {
            id: DeclarationId(154),
            name: Some("PathSegment".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(637)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12306, 12365),
        });
        declarations.push(Declaration {
            id: DeclarationId(155),
            name: Some("GlobSegment".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(638)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12366, 12425),
        });
        declarations.push(Declaration {
            id: DeclarationId(156),
            name: Some("FilePathParts".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "segments".to_string(),
                    ty: DeclarationId(520),
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
            span: SourceSpan::new("dsl/std/types.dag", 12426, 12478),
        });
        declarations.push(Declaration {
            id: DeclarationId(157),
            name: Some("GlobPattern".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "segments".to_string(),
                    ty: DeclarationId(521),
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
            span: SourceSpan::new("dsl/std/types.dag", 12479, 12529),
        });
        declarations.push(Declaration {
            id: DeclarationId(158),
            name: Some("FilePath".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(230))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(639)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12530, 12573),
        });
        declarations.push(Declaration {
            id: DeclarationId(159),
            name: Some("SourceSpan".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "file".to_string(),
                        ty: DeclarationId(158),
                    },
                    Field {
                        label: "start".to_string(),
                        ty: DeclarationId(93),
                    },
                    Field {
                        label: "end".to_string(),
                        ty: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/types.dag", 13191, 13251),
        });
        declarations.push(Declaration {
            id: DeclarationId(160),
            name: Some("Timestamp".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 13351, 13377),
        });
        declarations.push(Declaration {
            id: DeclarationId(161),
            name: Some("EpochMs".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(93))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(640)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13378, 13422),
        });
        declarations.push(Declaration {
            id: DeclarationId(162),
            name: Some("Duration".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(93))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(641)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13423, 13467),
        });
        declarations.push(Declaration {
            id: DeclarationId(163),
            name: Some("Milliseconds".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(93))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(642)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13949, 13993),
        });
        declarations.push(Declaration {
            id: DeclarationId(164),
            name: Some("Seconds".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(93))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(643)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13994, 14038),
        });
        declarations.push(Declaration {
            id: DeclarationId(165),
            name: Some("IntentId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(644)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14562, 14617),
        });
        declarations.push(Declaration {
            id: DeclarationId(166),
            name: Some("IssueId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(645)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14618, 14672),
        });
        declarations.push(Declaration {
            id: DeclarationId(167),
            name: Some("RunKey".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(646)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14673, 14726),
        });
        declarations.push(Declaration {
            id: DeclarationId(168),
            name: Some("ArtifactId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(647)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14727, 14784),
        });
        declarations.push(Declaration {
            id: DeclarationId(169),
            name: Some("LeaseToken".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(648)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14785, 14842),
        });
        declarations.push(Declaration {
            id: DeclarationId(170),
            name: Some("WorkerId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(649)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14843, 14898),
        });
        declarations.push(Declaration {
            id: DeclarationId(171),
            name: Some("CommentId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(650)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14899, 14955),
        });
        declarations.push(Declaration {
            id: DeclarationId(172),
            name: Some("SignalKey".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(651)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14956, 15012),
        });
        declarations.push(Declaration {
            id: DeclarationId(173),
            name: Some("ContentHash".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(652)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15013, 15071),
        });
        declarations.push(Declaration {
            id: DeclarationId(174),
            name: Some("WorkflowProducerId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(653)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15302, 15373),
        });
        declarations.push(Declaration {
            id: DeclarationId(175),
            name: Some("WorkflowObserverId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(654)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15374, 15445),
        });
        declarations.push(Declaration {
            id: DeclarationId(176),
            name: Some("WorkflowProverId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(655)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15446, 15513),
        });
        declarations.push(Declaration {
            id: DeclarationId(177),
            name: Some("WorkflowRunId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(151))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(656)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15514, 15575),
        });
        declarations.push(Declaration {
            id: DeclarationId(178),
            name: Some("GitRef".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(230))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(657)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15655, 15697),
        });
        declarations.push(Declaration {
            id: DeclarationId(179),
            name: Some("GcpProjectId".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(230))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(658)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 16082, 16124),
        });
        declarations.push(Declaration {
            id: DeclarationId(180),
            name: Some("ServiceAccountEmail".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 16249, 16282),
        });
        declarations.push(Declaration {
            id: DeclarationId(181),
            name: Some("Platform".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Linux".to_string(),
                        ty: DeclarationId(522),
                    },
                    Field {
                        label: "Macos".to_string(),
                        ty: DeclarationId(523),
                    },
                    Field {
                        label: "Windows".to_string(),
                        ty: DeclarationId(524),
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
            span: SourceSpan::new("dsl/std/types.dag", 16668, 16711),
        });
        declarations.push(Declaration {
            id: DeclarationId(182),
            name: Some("TopologyNodeKind".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Pure".to_string(),
                        ty: DeclarationId(525),
                    },
                    Field {
                        label: "Transport".to_string(),
                        ty: DeclarationId(526),
                    },
                    Field {
                        label: "SubDag".to_string(),
                        ty: DeclarationId(527),
                    },
                    Field {
                        label: "Env".to_string(),
                        ty: DeclarationId(528),
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
            span: SourceSpan::new("dsl/std/types.dag", 16839, 16894),
        });
        declarations.push(Declaration {
            id: DeclarationId(183),
            name: Some("DocSourceKind".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Template".to_string(),
                        ty: DeclarationId(529),
                    },
                    Field {
                        label: "Generated".to_string(),
                        ty: DeclarationId(530),
                    },
                    Field {
                        label: "Static".to_string(),
                        ty: DeclarationId(531),
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
            span: SourceSpan::new("dsl/std/types.dag", 16895, 16948),
        });
        declarations.push(Declaration {
            id: DeclarationId(184),
            name: Some("FermiDepth".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Xs".to_string(),
                        ty: DeclarationId(532),
                    },
                    Field {
                        label: "S".to_string(),
                        ty: DeclarationId(533),
                    },
                    Field {
                        label: "M".to_string(),
                        ty: DeclarationId(534),
                    },
                    Field {
                        label: "L".to_string(),
                        ty: DeclarationId(535),
                    },
                    Field {
                        label: "Xl".to_string(),
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
            span: SourceSpan::new("dsl/std/types.dag", 16950, 16987),
        });
        declarations.push(Declaration {
            id: DeclarationId(185),
            name: Some("CredentialFlow".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Stored".to_string(),
                        ty: DeclarationId(537),
                    },
                    Field {
                        label: "PlatformInjected".to_string(),
                        ty: DeclarationId(538),
                    },
                    Field {
                        label: "WorkloadIdentity".to_string(),
                        ty: DeclarationId(541),
                    },
                    Field {
                        label: "InteractiveAuth".to_string(),
                        ty: DeclarationId(543),
                    },
                    Field {
                        label: "Chained".to_string(),
                        ty: DeclarationId(545),
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
            span: SourceSpan::new("dsl/std/types.dag", 17235, 17557),
        });
        declarations.push(Declaration {
            id: DeclarationId(186),
            name: Some("Arch".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "X86_64".to_string(),
                        ty: DeclarationId(546),
                    },
                    Field {
                        label: "X86".to_string(),
                        ty: DeclarationId(547),
                    },
                    Field {
                        label: "Aarch64".to_string(),
                        ty: DeclarationId(548),
                    },
                    Field {
                        label: "Arm".to_string(),
                        ty: DeclarationId(549),
                    },
                    Field {
                        label: "Armv7".to_string(),
                        ty: DeclarationId(550),
                    },
                    Field {
                        label: "Mips".to_string(),
                        ty: DeclarationId(551),
                    },
                    Field {
                        label: "Mipsel".to_string(),
                        ty: DeclarationId(552),
                    },
                    Field {
                        label: "Mips64".to_string(),
                        ty: DeclarationId(553),
                    },
                    Field {
                        label: "Mips64el".to_string(),
                        ty: DeclarationId(554),
                    },
                    Field {
                        label: "Riscv64".to_string(),
                        ty: DeclarationId(555),
                    },
                    Field {
                        label: "Wasm32".to_string(),
                        ty: DeclarationId(556),
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
            span: SourceSpan::new("dsl/std/types.dag", 17632, 17735),
        });
        declarations.push(Declaration {
            id: DeclarationId(187),
            name: Some("Vendor".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "UnknownVendor".to_string(),
                        ty: DeclarationId(557),
                    },
                    Field {
                        label: "Pc".to_string(),
                        ty: DeclarationId(558),
                    },
                    Field {
                        label: "Apple".to_string(),
                        ty: DeclarationId(559),
                    },
                    Field {
                        label: "W64".to_string(),
                        ty: DeclarationId(560),
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
            span: SourceSpan::new("dsl/std/types.dag", 17736, 17782),
        });
        declarations.push(Declaration {
            id: DeclarationId(188),
            name: Some("Os".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Linux".to_string(),
                        ty: DeclarationId(561),
                    },
                    Field {
                        label: "Macos".to_string(),
                        ty: DeclarationId(562),
                    },
                    Field {
                        label: "Windows".to_string(),
                        ty: DeclarationId(563),
                    },
                    Field {
                        label: "Freebsd".to_string(),
                        ty: DeclarationId(564),
                    },
                    Field {
                        label: "Android".to_string(),
                        ty: DeclarationId(565),
                    },
                    Field {
                        label: "Ios".to_string(),
                        ty: DeclarationId(566),
                    },
                    Field {
                        label: "Wasi".to_string(),
                        ty: DeclarationId(567),
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
            span: SourceSpan::new("dsl/std/types.dag", 17783, 17849),
        });
        declarations.push(Declaration {
            id: DeclarationId(189),
            name: Some("AbiEnv".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "NoneAbi".to_string(),
                        ty: DeclarationId(568),
                    },
                    Field {
                        label: "Gnu".to_string(),
                        ty: DeclarationId(569),
                    },
                    Field {
                        label: "GnuEabi".to_string(),
                        ty: DeclarationId(570),
                    },
                    Field {
                        label: "GnuEabihf".to_string(),
                        ty: DeclarationId(571),
                    },
                    Field {
                        label: "Musl".to_string(),
                        ty: DeclarationId(572),
                    },
                    Field {
                        label: "Msvc".to_string(),
                        ty: DeclarationId(573),
                    },
                    Field {
                        label: "AndroidAbi".to_string(),
                        ty: DeclarationId(574),
                    },
                    Field {
                        label: "Eabi".to_string(),
                        ty: DeclarationId(575),
                    },
                    Field {
                        label: "Eabihf".to_string(),
                        ty: DeclarationId(576),
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
            span: SourceSpan::new("dsl/std/types.dag", 17850, 17942),
        });
        declarations.push(Declaration {
            id: DeclarationId(190),
            name: Some("ExecutionEnv".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Native".to_string(),
                        ty: DeclarationId(577),
                    },
                    Field {
                        label: "Wsl".to_string(),
                        ty: DeclarationId(578),
                    },
                    Field {
                        label: "Container".to_string(),
                        ty: DeclarationId(579),
                    },
                    Field {
                        label: "Ci".to_string(),
                        ty: DeclarationId(580),
                    },
                    Field {
                        label: "Emulator".to_string(),
                        ty: DeclarationId(581),
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
            span: SourceSpan::new("dsl/std/types.dag", 17943, 18003),
        });
        declarations.push(Declaration {
            id: DeclarationId(191),
            name: Some("TargetTriple".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "arch".to_string(),
                        ty: DeclarationId(186),
                    },
                    Field {
                        label: "vendor".to_string(),
                        ty: DeclarationId(187),
                    },
                    Field {
                        label: "os".to_string(),
                        ty: DeclarationId(188),
                    },
                    Field {
                        label: "env".to_string(),
                        ty: DeclarationId(582),
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
            span: SourceSpan::new("dsl/std/types.dag", 18005, 18080),
        });
        declarations.push(Declaration {
            id: DeclarationId(192),
            name: Some("RuntimePlatform".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "host".to_string(),
                        ty: DeclarationId(191),
                    },
                    Field {
                        label: "env".to_string(),
                        ty: DeclarationId(190),
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
            span: SourceSpan::new("dsl/std/types.dag", 18082, 18147),
        });
        declarations.push(Declaration {
            id: DeclarationId(193),
            name: Some("EntryKind".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "RegularFile".to_string(),
                        ty: DeclarationId(583),
                    },
                    Field {
                        label: "Directory".to_string(),
                        ty: DeclarationId(584),
                    },
                    Field {
                        label: "Symlink".to_string(),
                        ty: DeclarationId(585),
                    },
                    Field {
                        label: "Missing".to_string(),
                        ty: DeclarationId(586),
                    },
                    Field {
                        label: "Other".to_string(),
                        ty: DeclarationId(587),
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
            span: SourceSpan::new("dsl/std/types.dag", 18456, 18534),
        });
        declarations.push(Declaration {
            id: DeclarationId(194),
            name: Some("SymlinkTarget".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "TargetFile".to_string(),
                        ty: DeclarationId(588),
                    },
                    Field {
                        label: "TargetDir".to_string(),
                        ty: DeclarationId(589),
                    },
                    Field {
                        label: "Broken".to_string(),
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
            span: SourceSpan::new("dsl/std/types.dag", 18664, 18722),
        });
        declarations.push(Declaration {
            id: DeclarationId(195),
            name: Some("TextFilePath".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(158),
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
            span: SourceSpan::new("dsl/std/types.dag", 19883, 19913),
        });
        declarations.push(Declaration {
            id: DeclarationId(196),
            name: Some("BinaryFilePath".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(158),
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
            span: SourceSpan::new("dsl/std/types.dag", 19994, 20024),
        });
        declarations.push(Declaration {
            id: DeclarationId(197),
            name: Some("MimeType".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 20241, 20263),
        });
        declarations.push(Declaration {
            id: DeclarationId(198),
            name: Some("HttpMethod".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "GET".to_string(),
                        ty: DeclarationId(591),
                    },
                    Field {
                        label: "POST".to_string(),
                        ty: DeclarationId(592),
                    },
                    Field {
                        label: "PUT".to_string(),
                        ty: DeclarationId(593),
                    },
                    Field {
                        label: "PATCH".to_string(),
                        ty: DeclarationId(594),
                    },
                    Field {
                        label: "DELETE".to_string(),
                        ty: DeclarationId(595),
                    },
                    Field {
                        label: "HEAD".to_string(),
                        ty: DeclarationId(596),
                    },
                    Field {
                        label: "OPTIONS".to_string(),
                        ty: DeclarationId(597),
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
            span: SourceSpan::new("dsl/std/types.dag", 21007, 21075),
        });
        declarations.push(Declaration {
            id: DeclarationId(199),
            name: Some("AuthScheme".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Bearer".to_string(),
                        ty: DeclarationId(598),
                    },
                    Field {
                        label: "Header".to_string(),
                        ty: DeclarationId(599),
                    },
                    Field {
                        label: "Basic".to_string(),
                        ty: DeclarationId(600),
                    },
                    Field {
                        label: "ApiKey".to_string(),
                        ty: DeclarationId(601),
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
            span: SourceSpan::new("dsl/std/types.dag", 21543, 21639),
        });
        declarations.push(Declaration {
            id: DeclarationId(200),
            name: Some("AccessToken".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "token".to_string(),
                        ty: DeclarationId(147),
                    },
                    Field {
                        label: "scheme".to_string(),
                        ty: DeclarationId(199),
                    },
                    Field {
                        label: "expires_at".to_string(),
                        ty: DeclarationId(602),
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
            span: SourceSpan::new("dsl/std/types.dag", 21761, 21894),
        });
        declarations.push(Declaration {
            id: DeclarationId(201),
            name: Some("Credential".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "token".to_string(),
                        ty: DeclarationId(147),
                    },
                    Field {
                        label: "scheme".to_string(),
                        ty: DeclarationId(199),
                    },
                    Field {
                        label: "header_name".to_string(),
                        ty: DeclarationId(603),
                    },
                    Field {
                        label: "source_id".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "required_scopes".to_string(),
                        ty: DeclarationId(604),
                    },
                    Field {
                        label: "expires_in".to_string(),
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
            span: SourceSpan::new("dsl/std/types.dag", 21896, 22046),
        });
        declarations.push(Declaration {
            id: DeclarationId(202),
            name: Some("FilesystemHandle".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(158))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(659)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22123, 22187),
        });
        declarations.push(Declaration {
            id: DeclarationId(203),
            name: Some("NetworkHandle".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(129))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(660)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22188, 22245),
        });
        declarations.push(Declaration {
            id: DeclarationId(204),
            name: Some("ToolHandle".to_string()),
            connective: TypeConnective::Atom(AtomPayload::ResolvedByStructure(DeclarationId(230))),
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: Some(DeclarationId(661)),
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22246, 22302),
        });
        declarations.push(Declaration {
            id: DeclarationId(205),
            name: Some("TransportRequest".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "method".to_string(),
                        ty: DeclarationId(198),
                    },
                    Field {
                        label: "url".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "headers".to_string(),
                        ty: DeclarationId(130),
                    },
                    Field {
                        label: "body".to_string(),
                        ty: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 22379, 22470),
        });
        declarations.push(Declaration {
            id: DeclarationId(206),
            name: Some("TransportResponse".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "status".to_string(),
                        ty: DeclarationId(93),
                    },
                    Field {
                        label: "headers".to_string(),
                        ty: DeclarationId(130),
                    },
                    Field {
                        label: "body".to_string(),
                        ty: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 22472, 22543),
        });
        declarations.push(Declaration {
            id: DeclarationId(207),
            name: Some("FileResponse".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "success".to_string(),
                        ty: DeclarationId(128),
                    },
                    Field {
                        label: "content".to_string(),
                        ty: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 22545, 22615),
        });
        declarations.push(Declaration {
            id: DeclarationId(208),
            name: Some("ShellResponse".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "exit_code".to_string(),
                        ty: DeclarationId(93),
                    },
                    Field {
                        label: "stdout".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "stderr".to_string(),
                        ty: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 22617, 22690),
        });
        declarations.push(Declaration {
            id: DeclarationId(209),
            name: Some("RestResponse".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "status".to_string(),
                        ty: DeclarationId(93),
                    },
                    Field {
                        label: "headers".to_string(),
                        ty: DeclarationId(130),
                    },
                    Field {
                        label: "body".to_string(),
                        ty: DeclarationId(130),
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
            span: SourceSpan::new("dsl/std/types.dag", 22692, 22756),
        });
        declarations.push(Declaration {
            id: DeclarationId(210),
            name: Some("TestResult".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "ok".to_string(),
                        ty: DeclarationId(128),
                    },
                    Field {
                        label: "stdout".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "stderr".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "duration_ms".to_string(),
                        ty: DeclarationId(163),
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
            span: SourceSpan::new("dsl/std/types.dag", 22833, 22940),
        });
        declarations.push(Declaration {
            id: DeclarationId(211),
            name: Some("Summary".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "total".to_string(),
                        ty: DeclarationId(93),
                    },
                    Field {
                        label: "passed".to_string(),
                        ty: DeclarationId(93),
                    },
                    Field {
                        label: "failed".to_string(),
                        ty: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/types.dag", 22942, 22999),
        });
        declarations.push(Declaration {
            id: DeclarationId(212),
            name: Some("StageResult".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "success".to_string(),
                        ty: DeclarationId(128),
                    },
                    Field {
                        label: "stdout".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "stderr".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "skipped".to_string(),
                        ty: DeclarationId(128),
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
            span: SourceSpan::new("dsl/std/types.dag", 23001, 23102),
        });
        declarations.push(Declaration {
            id: DeclarationId(213),
            name: Some("DocumentLine".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "text".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "is_comment".to_string(),
                        ty: DeclarationId(128),
                    },
                    Field {
                        label: "is_blank".to_string(),
                        ty: DeclarationId(128),
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
            span: SourceSpan::new("dsl/std/types.dag", 23255, 23327),
        });
        declarations.push(Declaration {
            id: DeclarationId(214),
            name: Some("DocumentSection".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "title".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "has_title".to_string(),
                        ty: DeclarationId(128),
                    },
                    Field {
                        label: "lines".to_string(),
                        ty: DeclarationId(606),
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
            span: SourceSpan::new("dsl/std/types.dag", 23329, 23415),
        });
        declarations.push(Declaration {
            id: DeclarationId(215),
            name: Some("Document".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "header".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "has_header".to_string(),
                        ty: DeclarationId(128),
                    },
                    Field {
                        label: "comment_prefix".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "sections".to_string(),
                        ty: DeclarationId(607),
                    },
                    Field {
                        label: "trailing_newline".to_string(),
                        ty: DeclarationId(128),
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
            span: SourceSpan::new("dsl/std/types.dag", 23417, 23554),
        });
        declarations.push(Declaration {
            id: DeclarationId(216),
            name: Some("TextFile".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "document".to_string(),
                        ty: DeclarationId(215),
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
            span: SourceSpan::new("dsl/std/types.dag", 23556, 23609),
        });
        declarations.push(Declaration {
            id: DeclarationId(217),
            name: Some("RenderedTextFile".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "content".to_string(),
                        ty: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 23611, 23669),
        });
        declarations.push(Declaration {
            id: DeclarationId(218),
            name: Some("ToolEntry".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "command".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "description".to_string(),
                        ty: DeclarationId(608),
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
            span: SourceSpan::new("dsl/std/types.dag", 23746, 23820),
        });
        declarations.push(Declaration {
            id: DeclarationId(219),
            name: Some("ToolRegistry".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "tools".to_string(),
                    ty: DeclarationId(609),
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
            span: SourceSpan::new("dsl/std/types.dag", 23822, 23868),
        });
        declarations.push(Declaration {
            id: DeclarationId(220),
            name: Some("DagTopology".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "nodes".to_string(),
                        ty: DeclarationId(610),
                    },
                    Field {
                        label: "edges".to_string(),
                        ty: DeclarationId(611),
                    },
                    Field {
                        label: "subdag_boundaries".to_string(),
                        ty: DeclarationId(612),
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
            span: SourceSpan::new("dsl/std/types.dag", 24127, 24237),
        });
        declarations.push(Declaration {
            id: DeclarationId(221),
            name: Some("TopologyNode".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "id".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "label".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "kind".to_string(),
                        ty: DeclarationId(182),
                    },
                    Field {
                        label: "parent".to_string(),
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
            span: SourceSpan::new("dsl/std/types.dag", 24239, 24359),
        });
        declarations.push(Declaration {
            id: DeclarationId(222),
            name: Some("TopologyEdge".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "from".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "to".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "port".to_string(),
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
            span: SourceSpan::new("dsl/std/types.dag", 24361, 24426),
        });
        declarations.push(Declaration {
            id: DeclarationId(223),
            name: Some("DagDiff".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "added".to_string(),
                        ty: DeclarationId(615),
                    },
                    Field {
                        label: "removed".to_string(),
                        ty: DeclarationId(616),
                    },
                    Field {
                        label: "changed".to_string(),
                        ty: DeclarationId(617),
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
            span: SourceSpan::new("dsl/std/types.dag", 24428, 24514),
        });
        declarations.push(Declaration {
            id: DeclarationId(224),
            name: Some("CodegenTarget".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(158),
                    },
                    Field {
                        label: "backend".to_string(),
                        ty: DeclarationId(618),
                    },
                    Field {
                        label: "target".to_string(),
                        ty: DeclarationId(619),
                    },
                    Field {
                        label: "runtime_env".to_string(),
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
            span: SourceSpan::new("dsl/std/types.dag", 24583, 24717),
        });
        declarations.push(Declaration {
            id: DeclarationId(225),
            name: Some("CodegenBackend".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Rust".to_string(),
                        ty: DeclarationId(621),
                    },
                    Field {
                        label: "Go".to_string(),
                        ty: DeclarationId(622),
                    },
                    Field {
                        label: "C".to_string(),
                        ty: DeclarationId(623),
                    },
                    Field {
                        label: "Mips".to_string(),
                        ty: DeclarationId(624),
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
            span: SourceSpan::new("dsl/std/types.dag", 24719, 24761),
        });
        declarations.push(Declaration {
            id: DeclarationId(226),
            name: Some("PragmaDirective".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "key".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "value".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "scope".to_string(),
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
            span: SourceSpan::new("dsl/std/types.dag", 24763, 24834),
        });
        declarations.push(Declaration {
            id: DeclarationId(227),
            name: Some("DocSource".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "path".to_string(),
                        ty: DeclarationId(158),
                    },
                    Field {
                        label: "kind".to_string(),
                        ty: DeclarationId(183),
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
            span: SourceSpan::new("dsl/std/types.dag", 24975, 25032),
        });
        declarations.push(Declaration {
            id: DeclarationId(228),
            name: Some("ReferenceModel".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![DeclarationId(229)],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 26966, 26985),
        });
        declarations.push(Declaration {
            id: DeclarationId(229),
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
            span: SourceSpan::new("dsl/std/types.dag", 26966, 26985),
        });
        declarations.push(Declaration {
            id: DeclarationId(230),
            name: Some("String".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(132),
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
            span: SourceSpan::new("dsl/std/string_type.dag", 520, 550),
        });
        declarations.push(Declaration {
            id: DeclarationId(231),
            name: Some("CharClass".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Whitespace".to_string(),
                        ty: DeclarationId(662),
                    },
                    Field {
                        label: "Digit".to_string(),
                        ty: DeclarationId(663),
                    },
                    Field {
                        label: "IdentStart".to_string(),
                        ty: DeclarationId(664),
                    },
                    Field {
                        label: "IdentContinue".to_string(),
                        ty: DeclarationId(665),
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
            id: DeclarationId(232),
            name: Some("char_in_class".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(132), DeclarationId(231)],
                output: DeclarationId(128),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 3780, 4378)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 3728, 4378),
        });
        declarations.push(Declaration {
            id: DeclarationId(233),
            name: Some("DisplayWidth".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "ZeroWidth".to_string(),
                        ty: DeclarationId(666),
                    },
                    Field {
                        label: "Narrow".to_string(),
                        ty: DeclarationId(667),
                    },
                    Field {
                        label: "Wide".to_string(),
                        ty: DeclarationId(668),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 4742, 4787),
        });
        declarations.push(Declaration {
            id: DeclarationId(234),
            name: Some("display_width_columns".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(233)],
                output: DeclarationId(93),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 4838, 4914)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 4789, 4914),
        });
        declarations.push(Declaration {
            id: DeclarationId(235),
            name: Some("UnicodeBlock".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "name".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "start".to_string(),
                        ty: DeclarationId(93),
                    },
                    Field {
                        label: "end_inclusive".to_string(),
                        ty: DeclarationId(93),
                    },
                    Field {
                        label: "default_width".to_string(),
                        ty: DeclarationId(233),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 5149, 5249),
        });
        declarations.push(Declaration {
            id: DeclarationId(236),
            name: Some("zero_width_blocks".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(235),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(669)),
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
                            constructor: DeclarationId(666),
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
                            constructor: DeclarationId(666),
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
                            constructor: DeclarationId(666),
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
                            constructor: DeclarationId(666),
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
                            constructor: DeclarationId(666),
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
                            constructor: DeclarationId(666),
                            payload: vec![],
                        },
                    ),
                ]),
            ])),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 5311, 6060),
        });
        declarations.push(Declaration {
            id: DeclarationId(237),
            name: Some("zero_width_codepoints".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(93),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(670)),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 6127, 6339),
        });
        declarations.push(Declaration {
            id: DeclarationId(238),
            name: Some("wide_blocks".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(235),
                }],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(671)),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
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
                            constructor: DeclarationId(668),
                            payload: vec![],
                        },
                    ),
                ]),
            ])),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 6386, 8175),
        });
        declarations.push(Declaration {
            id: DeclarationId(239),
            name: Some("code_point".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(132)],
                output: DeclarationId(93),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 8355, 8366)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 8325, 8366),
        });
        declarations.push(Declaration {
            id: DeclarationId(240),
            name: Some("in_block".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93), DeclarationId(235)],
                output: DeclarationId(128),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 8418, 8470)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 8368, 8470),
        });
        declarations.push(Declaration {
            id: DeclarationId(241),
            name: Some("char_display_width".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(132)],
                output: DeclarationId(233),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 8519, 8822)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 8472, 8822),
        });
        declarations.push(Declaration {
            id: DeclarationId(242),
            name: Some("char_width".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(132)],
                output: DeclarationId(93),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 8854, 8910)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 8824, 8910),
        });
        declarations.push(Declaration {
            id: DeclarationId(243),
            name: Some("string_display_width".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230)],
                output: DeclarationId(93),
                body: ArrowBody::Unparsed(SourceSpan::new("dsl/std/unicode.dag", 8954, 9092)),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/unicode.dag", 8912, 9092),
        });
        declarations.push(Declaration {
            id: DeclarationId(244),
            name: Some("repeat_string_loop".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230), DeclarationId(230), DeclarationId(93)],
                output: DeclarationId(230),
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
            id: DeclarationId(245),
            name: Some("repeat_string".to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(230), DeclarationId(93)],
                output: DeclarationId(230),
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
            id: DeclarationId(246),
            name: Some("MethodDeclaration".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "name".to_string(),
                    ty: DeclarationId(230),
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
            id: DeclarationId(247),
            name: Some("add_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(248),
            name: Some("all_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(249),
            name: Some("any_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(250),
            name: Some("append_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(251),
            name: Some("bottom_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(252),
            name: Some("chars_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(253),
            name: Some("clamp_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(254),
            name: Some("compare_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(255),
            name: Some("complement_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(256),
            name: Some("concat_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(257),
            name: Some("contains_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(258),
            name: Some("count_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(259),
            name: Some("diff_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(260),
            name: Some("empty_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(261),
            name: Some("ends_with_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(262),
            name: Some("enumerate_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(263),
            name: Some("filter_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(264),
            name: Some("first_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(265),
            name: Some("flat_map_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(266),
            name: Some("fold_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(267),
            name: Some("get_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(268),
            name: Some("has_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(269),
            name: Some("intersect_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(270),
            name: Some("is_empty_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(271),
            name: Some("join_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(272),
            name: Some("keys_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(273),
            name: Some("last_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(274),
            name: Some("length_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(275),
            name: Some("list_push_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(276),
            name: Some("lookup_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(277),
            name: Some("map_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(278),
            name: Some("map_contains_key_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(279),
            name: Some("map_get_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(280),
            name: Some("map_has_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(281),
            name: Some("map_insert_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(282),
            name: Some("map_keys_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(283),
            name: Some("map_merge_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(284),
            name: Some("map_values_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(285),
            name: Some("meet_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(286),
            name: Some("member_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(287),
            name: Some("mul_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(288),
            name: Some("negate_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(289),
            name: Some("one_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(290),
            name: Some("reciprocal_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(291),
            name: Some("replace_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(292),
            name: Some("reverse_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(293),
            name: Some("skip_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(294),
            name: Some("sort_by_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(295),
            name: Some("split_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(296),
            name: Some("starts_with_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(297),
            name: Some("string_contains_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(298),
            name: Some("substring_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(299),
            name: Some("take_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(300),
            name: Some("to_int_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(301),
            name: Some("to_lower_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(302),
            name: Some("to_string_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(303),
            name: Some("to_upper_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(304),
            name: Some("top_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(305),
            name: Some("trim_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(306),
            name: Some("union_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(307),
            name: Some("values_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(308),
            name: Some("with_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(309),
            name: Some("zero_method".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(246),
                arguments: vec![],
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: Some(DeclarationId(246)),
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
            id: DeclarationId(310),
            name: Some("DeclarationRef".to_string()),
            connective: TypeConnective::Instantiation {
                template: DeclarationId(230),
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
            id: DeclarationId(311),
            name: Some("VariantNaming".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "AsAuthored".to_string(),
                        ty: DeclarationId(672),
                    },
                    Field {
                        label: "SnakeCase".to_string(),
                        ty: DeclarationId(673),
                    },
                    Field {
                        label: "StripPrefixAndSnakeCase".to_string(),
                        ty: DeclarationId(674),
                    },
                    Field {
                        label: "StripSuffixAndSnakeCase".to_string(),
                        ty: DeclarationId(675),
                    },
                    Field {
                        label: "StripPrefixSuffixAndSnakeCase".to_string(),
                        ty: DeclarationId(676),
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
            id: DeclarationId(312),
            name: Some("VariantEncoding".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "StringVariant".to_string(),
                        ty: DeclarationId(677),
                    },
                    Field {
                        label: "InternallyTaggedObject".to_string(),
                        ty: DeclarationId(678),
                    },
                    Field {
                        label: "TaggedVariant".to_string(),
                        ty: DeclarationId(679),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 2391, 2545),
        });
        declarations.push(Declaration {
            id: DeclarationId(313),
            name: Some("CoproductWireContract".to_string()),
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "coproduct".to_string(),
                        ty: DeclarationId(310),
                    },
                    Field {
                        label: "encoding".to_string(),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 2767, 2853),
        });
        declarations.push(Declaration {
            id: DeclarationId(314),
            name: Some("WireContract".to_string()),
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "variant_encoding".to_string(),
                    ty: DeclarationId(312),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 2998, 3055),
        });
        declarations.push(Declaration {
            id: DeclarationId(315),
            name: Some("WireFormat".to_string()),
            connective: TypeConnective::Disj {
                variants: vec![
                    Field {
                        label: "Json".to_string(),
                        ty: DeclarationId(680),
                    },
                    Field {
                        label: "Text".to_string(),
                        ty: DeclarationId(681),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 3246, 3275),
        });
        declarations.push(Declaration {
            id: DeclarationId(316),
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
            id: DeclarationId(317),
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
            id: DeclarationId(318),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(319),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(320),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(321),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(322),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(323),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(324),
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
            id: DeclarationId(325),
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
            id: DeclarationId(326),
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
            id: DeclarationId(327),
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
            id: DeclarationId(328),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(329),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(330),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(331),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(332),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(333),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(334),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(335),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(336),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 43456, 43468),
        });
        declarations.push(Declaration {
            id: DeclarationId(337),
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
            id: DeclarationId(338),
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
            id: DeclarationId(339),
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
            id: DeclarationId(340),
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
            id: DeclarationId(341),
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
            id: DeclarationId(342),
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
            id: DeclarationId(343),
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
            id: DeclarationId(344),
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
            id: DeclarationId(345),
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
            id: DeclarationId(346),
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
            id: DeclarationId(347),
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
            id: DeclarationId(348),
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
            id: DeclarationId(349),
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
            id: DeclarationId(350),
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
            id: DeclarationId(351),
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
            id: DeclarationId(352),
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
            id: DeclarationId(353),
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
            id: DeclarationId(354),
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
            id: DeclarationId(355),
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
            id: DeclarationId(356),
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
            id: DeclarationId(357),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(356),
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
            id: DeclarationId(358),
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
            id: DeclarationId(359),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(128),
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
            id: DeclarationId(360),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(128),
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
            id: DeclarationId(361),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(128),
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
            id: DeclarationId(362),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(128),
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
            id: DeclarationId(363),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(128),
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
            id: DeclarationId(364),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(38), DeclarationId(38)],
                output: DeclarationId(128),
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
            id: DeclarationId(365),
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
            id: DeclarationId(366),
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
            id: DeclarationId(367),
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
            id: DeclarationId(368),
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
            id: DeclarationId(369),
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
            id: DeclarationId(370),
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
            id: DeclarationId(371),
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
            id: DeclarationId(372),
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
            id: DeclarationId(373),
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
            id: DeclarationId(374),
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
            id: DeclarationId(375),
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
            id: DeclarationId(376),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 17529, 17542),
        });
        declarations.push(Declaration {
            id: DeclarationId(378),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 17562, 17575),
        });
        declarations.push(Declaration {
            id: DeclarationId(380),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(377), DeclarationId(378)],
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
            span: SourceSpan::new("dsl/std/algebra.dag", 17526, 17575),
        });
        declarations.push(Declaration {
            id: DeclarationId(381),
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
            id: DeclarationId(382),
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
            id: DeclarationId(383),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(382),
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
            id: DeclarationId(384),
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
            id: DeclarationId(385),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93), DeclarationId(93)],
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
            span: SourceSpan::new("dsl/std/algebra.dag", 17717, 17746),
        });
        declarations.push(Declaration {
            id: DeclarationId(386),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(93),
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
            id: DeclarationId(387),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(128),
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
            id: DeclarationId(388),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(93),
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
            id: DeclarationId(389),
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
            id: DeclarationId(390),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
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
            span: SourceSpan::new("dsl/std/algebra.dag", 17824, 17834),
        });
        declarations.push(Declaration {
            id: DeclarationId(391),
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
            id: DeclarationId(392),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(391),
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
            id: DeclarationId(393),
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
            id: DeclarationId(394),
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
            id: DeclarationId(395),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(393)],
                output: DeclarationId(394),
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
            id: DeclarationId(396),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(128),
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
            id: DeclarationId(397),
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
            id: DeclarationId(398),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(396)],
                output: DeclarationId(397),
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
            id: DeclarationId(399),
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
            id: DeclarationId(400),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48), DeclarationId(399)],
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
            id: DeclarationId(401),
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
            id: DeclarationId(402),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(401),
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
            id: DeclarationId(403),
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
            id: DeclarationId(404),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(402)],
                output: DeclarationId(403),
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
            id: DeclarationId(405),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(128),
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
            id: DeclarationId(406),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(405)],
                output: DeclarationId(128),
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
            id: DeclarationId(407),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(128),
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
            id: DeclarationId(408),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(407)],
                output: DeclarationId(128),
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
            id: DeclarationId(409),
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
            id: DeclarationId(410),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(409),
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
            id: DeclarationId(411),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(47),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(48),
                    value: DeclarationId(410),
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
            id: DeclarationId(412),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(411),
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
            id: DeclarationId(413),
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
            id: DeclarationId(414),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(413),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 18312, 18325),
        });
        declarations.push(Declaration {
            id: DeclarationId(416),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
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
            span: SourceSpan::new("dsl/std/algebra.dag", 18301, 18325),
        });
        declarations.push(Declaration {
            id: DeclarationId(417),
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
            id: DeclarationId(418),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
                output: DeclarationId(417),
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
            id: DeclarationId(419),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48), DeclarationId(48)],
                output: DeclarationId(93),
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
            id: DeclarationId(420),
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
            id: DeclarationId(421),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(419)],
                output: DeclarationId(420),
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
            id: DeclarationId(422),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(48)],
                output: DeclarationId(128),
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
            id: DeclarationId(423),
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
            id: DeclarationId(424),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50)],
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
            span: SourceSpan::new("dsl/std/algebra.dag", 18988, 18999),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 19009, 19030),
        });
        declarations.push(Declaration {
            id: DeclarationId(426),
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
            id: DeclarationId(427),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50)],
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
            span: SourceSpan::new("dsl/std/algebra.dag", 19038, 19049),
        });
        declarations.push(Declaration {
            id: DeclarationId(428),
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
            id: DeclarationId(429),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50), DeclarationId(51)],
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
            span: SourceSpan::new("dsl/std/algebra.dag", 19060, 19093),
        });
        declarations.push(Declaration {
            id: DeclarationId(430),
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
            id: DeclarationId(431),
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
            id: DeclarationId(432),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(430)],
                output: DeclarationId(431),
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
            id: DeclarationId(433),
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
            id: DeclarationId(434),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(433),
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
            id: DeclarationId(435),
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
            id: DeclarationId(436),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(435),
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
            id: DeclarationId(437),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50)],
                output: DeclarationId(128),
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
            id: DeclarationId(438),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(50)],
                output: DeclarationId(128),
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
            id: DeclarationId(439),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![],
                output: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 19505, 19509),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 19512, 19517),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 19520, 19527),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 21636, 21654),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 21659, 21682),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 21687, 21708),
        });
        declarations.push(Declaration {
            id: DeclarationId(446),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 21749, 21772),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 21777, 21804),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 21809, 21831),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 21858, 21872),
        });
        declarations.push(Declaration {
            id: DeclarationId(451),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "name".to_string(),
                    ty: DeclarationId(230),
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
            id: DeclarationId(452),
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
            id: DeclarationId(453),
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
            id: DeclarationId(454),
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
            id: DeclarationId(455),
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
            id: DeclarationId(456),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "name".to_string(),
                    ty: DeclarationId(230),
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
            id: DeclarationId(457),
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
            id: DeclarationId(458),
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
            id: DeclarationId(459),
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
            id: DeclarationId(460),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(461),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "params".to_string(),
                        ty: DeclarationId(460),
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
            id: DeclarationId(462),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "id".to_string(),
                    ty: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 22595, 22607),
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
            span: SourceSpan::new("dsl/std/algebra.dag", 22665, 22681),
        });
        declarations.push(Declaration {
            id: DeclarationId(465),
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
            id: DeclarationId(466),
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
            id: DeclarationId(467),
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
            id: DeclarationId(468),
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
            id: DeclarationId(469),
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
            id: DeclarationId(470),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            id: DeclarationId(471),
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
            id: DeclarationId(472),
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
            id: DeclarationId(473),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(93),
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
            id: DeclarationId(474),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
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
            id: DeclarationId(475),
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
            id: DeclarationId(476),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2277, 2295),
        });
        declarations.push(Declaration {
            id: DeclarationId(477),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(7),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2323, 2343),
        });
        declarations.push(Declaration {
            id: DeclarationId(478),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(8),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2371, 2391),
        });
        declarations.push(Declaration {
            id: DeclarationId(479),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(9),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2419, 2439),
        });
        declarations.push(Declaration {
            id: DeclarationId(480),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(10),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2468, 2489),
        });
        declarations.push(Declaration {
            id: DeclarationId(481),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2583, 2601),
        });
        declarations.push(Declaration {
            id: DeclarationId(482),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(7),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2631, 2651),
        });
        declarations.push(Declaration {
            id: DeclarationId(483),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(8),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2681, 2701),
        });
        declarations.push(Declaration {
            id: DeclarationId(484),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(9),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2731, 2751),
        });
        declarations.push(Declaration {
            id: DeclarationId(485),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(10),
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
            span: SourceSpan::new("dsl/std/integer.dag", 2782, 2803),
        });
        declarations.push(Declaration {
            id: DeclarationId(486),
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
            span: SourceSpan::new("dsl/std/integer.dag", 6254, 6274),
        });
        declarations.push(Declaration {
            id: DeclarationId(487),
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
            span: SourceSpan::new("dsl/std/integer.dag", 7962, 7988),
        });
        declarations.push(Declaration {
            id: DeclarationId(488),
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
            span: SourceSpan::new("dsl/std/integer.dag", 8024, 8050),
        });
        declarations.push(Declaration {
            id: DeclarationId(489),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(76)],
                output: DeclarationId(128),
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
            span: SourceSpan::new("dsl/std/integer.dag", 9551, 9558),
        });
        declarations.push(Declaration {
            id: DeclarationId(490),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(29),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(30),
                    value: DeclarationId(93),
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
            id: DeclarationId(491),
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
            id: DeclarationId(492),
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
            id: DeclarationId(493),
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
            id: DeclarationId(494),
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
            id: DeclarationId(495),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1423, 1432),
        });
        declarations.push(Declaration {
            id: DeclarationId(497),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "significand_bits".to_string(),
                        ty: DeclarationId(98),
                    },
                    Field {
                        label: "exponent_bits".to_string(),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1437, 1514),
        });
        declarations.push(Declaration {
            id: DeclarationId(498),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "digits".to_string(),
                        ty: DeclarationId(98),
                    },
                    Field {
                        label: "exponent_digits".to_string(),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1519, 1589),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1829, 1834),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1837, 1845),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 1848, 1868),
        });
        declarations.push(Declaration {
            id: DeclarationId(502),
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
            id: DeclarationId(503),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2400, 2412),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2415, 2425),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2743, 2755),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2758, 2774),
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
            span: SourceSpan::new("src/v3/std/approximate_field.dag", 2777, 2788),
        });
        declarations.push(Declaration {
            id: DeclarationId(509),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(39),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(40),
                    value: DeclarationId(108),
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
            id: DeclarationId(510),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(29),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(30),
                    value: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/float.dag", 1475, 1496),
        });
        declarations.push(Declaration {
            id: DeclarationId(511),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(8),
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
            span: SourceSpan::new("dsl/std/float.dag", 1747, 1767),
        });
        declarations.push(Declaration {
            id: DeclarationId(512),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(70),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(71),
                    value: DeclarationId(9),
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
            span: SourceSpan::new("dsl/std/float.dag", 1797, 1817),
        });
        declarations.push(Declaration {
            id: DeclarationId(513),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(93),
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
            id: DeclarationId(514),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 4054, 4066),
        });
        declarations.push(Declaration {
            id: DeclarationId(515),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(230),
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
            id: DeclarationId(516),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(230),
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
            id: DeclarationId(517),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 6508, 6520),
        });
        declarations.push(Declaration {
            id: DeclarationId(518),
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
            id: DeclarationId(519),
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
            id: DeclarationId(520),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(154),
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
            span: SourceSpan::new("dsl/std/types.dag", 12459, 12476),
        });
        declarations.push(Declaration {
            id: DeclarationId(521),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(155),
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
            span: SourceSpan::new("dsl/std/types.dag", 12510, 12527),
        });
        declarations.push(Declaration {
            id: DeclarationId(522),
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
            span: SourceSpan::new("dsl/std/types.dag", 16688, 16693),
        });
        declarations.push(Declaration {
            id: DeclarationId(523),
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
            span: SourceSpan::new("dsl/std/types.dag", 16696, 16701),
        });
        declarations.push(Declaration {
            id: DeclarationId(524),
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
            span: SourceSpan::new("dsl/std/types.dag", 16704, 16711),
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
            span: SourceSpan::new("dsl/std/types.dag", 16863, 16867),
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
            span: SourceSpan::new("dsl/std/types.dag", 16870, 16879),
        });
        declarations.push(Declaration {
            id: DeclarationId(527),
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
            span: SourceSpan::new("dsl/std/types.dag", 16882, 16888),
        });
        declarations.push(Declaration {
            id: DeclarationId(528),
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
            span: SourceSpan::new("dsl/std/types.dag", 16891, 16894),
        });
        declarations.push(Declaration {
            id: DeclarationId(529),
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
            span: SourceSpan::new("dsl/std/types.dag", 16919, 16927),
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
            span: SourceSpan::new("dsl/std/types.dag", 16930, 16939),
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
            span: SourceSpan::new("dsl/std/types.dag", 16942, 16948),
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
            span: SourceSpan::new("dsl/std/types.dag", 16968, 16970),
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
            span: SourceSpan::new("dsl/std/types.dag", 16973, 16974),
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
            span: SourceSpan::new("dsl/std/types.dag", 16977, 16978),
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
            span: SourceSpan::new("dsl/std/types.dag", 16981, 16982),
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
            span: SourceSpan::new("dsl/std/types.dag", 16985, 16987),
        });
        declarations.push(Declaration {
            id: DeclarationId(537),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "secret_name".to_string(),
                    ty: DeclarationId(151),
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
            span: SourceSpan::new("dsl/std/types.dag", 17259, 17294),
        });
        declarations.push(Declaration {
            id: DeclarationId(538),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "env_var".to_string(),
                    ty: DeclarationId(151),
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
            span: SourceSpan::new("dsl/std/types.dag", 17299, 17340),
        });
        declarations.push(Declaration {
            id: DeclarationId(539),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(180),
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
            span: SourceSpan::new("dsl/std/types.dag", 17415, 17435),
        });
        declarations.push(Declaration {
            id: DeclarationId(540),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 17450, 17462),
        });
        declarations.push(Declaration {
            id: DeclarationId(541),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "audience".to_string(),
                        ty: DeclarationId(151),
                    },
                    Field {
                        label: "service_account".to_string(),
                        ty: DeclarationId(539),
                    },
                    Field {
                        label: "scopes".to_string(),
                        ty: DeclarationId(540),
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
            span: SourceSpan::new("dsl/std/types.dag", 17345, 17468),
        });
        declarations.push(Declaration {
            id: DeclarationId(542),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 17499, 17511),
        });
        declarations.push(Declaration {
            id: DeclarationId(543),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "scopes".to_string(),
                    ty: DeclarationId(542),
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
            span: SourceSpan::new("dsl/std/types.dag", 17473, 17513),
        });
        declarations.push(Declaration {
            id: DeclarationId(544),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(185),
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
            span: SourceSpan::new("dsl/std/types.dag", 17535, 17555),
        });
        declarations.push(Declaration {
            id: DeclarationId(545),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "steps".to_string(),
                    ty: DeclarationId(544),
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
            span: SourceSpan::new("dsl/std/types.dag", 17518, 17557),
        });
        declarations.push(Declaration {
            id: DeclarationId(546),
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
            span: SourceSpan::new("dsl/std/types.dag", 17644, 17650),
        });
        declarations.push(Declaration {
            id: DeclarationId(547),
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
            span: SourceSpan::new("dsl/std/types.dag", 17653, 17656),
        });
        declarations.push(Declaration {
            id: DeclarationId(548),
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
            span: SourceSpan::new("dsl/std/types.dag", 17659, 17666),
        });
        declarations.push(Declaration {
            id: DeclarationId(549),
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
            span: SourceSpan::new("dsl/std/types.dag", 17669, 17672),
        });
        declarations.push(Declaration {
            id: DeclarationId(550),
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
            span: SourceSpan::new("dsl/std/types.dag", 17675, 17680),
        });
        declarations.push(Declaration {
            id: DeclarationId(551),
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
            span: SourceSpan::new("dsl/std/types.dag", 17683, 17687),
        });
        declarations.push(Declaration {
            id: DeclarationId(552),
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
            span: SourceSpan::new("dsl/std/types.dag", 17690, 17696),
        });
        declarations.push(Declaration {
            id: DeclarationId(553),
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
            span: SourceSpan::new("dsl/std/types.dag", 17699, 17705),
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
            span: SourceSpan::new("dsl/std/types.dag", 17708, 17716),
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
            span: SourceSpan::new("dsl/std/types.dag", 17719, 17726),
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
            span: SourceSpan::new("dsl/std/types.dag", 17729, 17735),
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
            span: SourceSpan::new("dsl/std/types.dag", 17750, 17763),
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
            span: SourceSpan::new("dsl/std/types.dag", 17766, 17768),
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
            span: SourceSpan::new("dsl/std/types.dag", 17771, 17776),
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
            span: SourceSpan::new("dsl/std/types.dag", 17779, 17782),
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
            span: SourceSpan::new("dsl/std/types.dag", 17793, 17798),
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
            span: SourceSpan::new("dsl/std/types.dag", 17801, 17806),
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
            span: SourceSpan::new("dsl/std/types.dag", 17809, 17816),
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
            span: SourceSpan::new("dsl/std/types.dag", 17819, 17826),
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
            span: SourceSpan::new("dsl/std/types.dag", 17829, 17836),
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
            span: SourceSpan::new("dsl/std/types.dag", 17839, 17842),
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
            span: SourceSpan::new("dsl/std/types.dag", 17845, 17849),
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
            span: SourceSpan::new("dsl/std/types.dag", 17864, 17871),
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
            span: SourceSpan::new("dsl/std/types.dag", 17874, 17877),
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
            span: SourceSpan::new("dsl/std/types.dag", 17880, 17887),
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
            span: SourceSpan::new("dsl/std/types.dag", 17890, 17899),
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
            span: SourceSpan::new("dsl/std/types.dag", 17902, 17906),
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
            span: SourceSpan::new("dsl/std/types.dag", 17909, 17913),
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
            span: SourceSpan::new("dsl/std/types.dag", 17916, 17926),
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
            span: SourceSpan::new("dsl/std/types.dag", 17929, 17933),
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
            span: SourceSpan::new("dsl/std/types.dag", 17936, 17942),
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
            span: SourceSpan::new("dsl/std/types.dag", 17963, 17969),
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
            span: SourceSpan::new("dsl/std/types.dag", 17972, 17975),
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
            span: SourceSpan::new("dsl/std/types.dag", 17978, 17987),
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
            span: SourceSpan::new("dsl/std/types.dag", 17990, 17992),
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
            span: SourceSpan::new("dsl/std/types.dag", 17995, 18003),
        });
        declarations.push(Declaration {
            id: DeclarationId(582),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(189),
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
            span: SourceSpan::new("dsl/std/types.dag", 18071, 18078),
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
            span: SourceSpan::new("dsl/std/types.dag", 18475, 18486),
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
            span: SourceSpan::new("dsl/std/types.dag", 18491, 18500),
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
            span: SourceSpan::new("dsl/std/types.dag", 18505, 18512),
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
            span: SourceSpan::new("dsl/std/types.dag", 18517, 18524),
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
            span: SourceSpan::new("dsl/std/types.dag", 18529, 18534),
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
            span: SourceSpan::new("dsl/std/types.dag", 18687, 18697),
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
            span: SourceSpan::new("dsl/std/types.dag", 18702, 18711),
        });
        declarations.push(Declaration {
            id: DeclarationId(590),
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
            span: SourceSpan::new("dsl/std/types.dag", 18716, 18722),
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
            span: SourceSpan::new("dsl/std/types.dag", 21025, 21028),
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
            span: SourceSpan::new("dsl/std/types.dag", 21031, 21035),
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
            span: SourceSpan::new("dsl/std/types.dag", 21038, 21041),
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
            span: SourceSpan::new("dsl/std/types.dag", 21044, 21049),
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
            span: SourceSpan::new("dsl/std/types.dag", 21052, 21058),
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
            span: SourceSpan::new("dsl/std/types.dag", 21061, 21065),
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
            span: SourceSpan::new("dsl/std/types.dag", 21068, 21075),
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
            span: SourceSpan::new("dsl/std/types.dag", 21563, 21569),
        });
        declarations.push(Declaration {
            id: DeclarationId(599),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "name".to_string(),
                    ty: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 21574, 21597),
        });
        declarations.push(Declaration {
            id: DeclarationId(600),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "username".to_string(),
                    ty: DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 21602, 21628),
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
            span: SourceSpan::new("dsl/std/types.dag", 21633, 21639),
        });
        declarations.push(Declaration {
            id: DeclarationId(602),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(160),
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
            span: SourceSpan::new("dsl/std/types.dag", 21831, 21841),
        });
        declarations.push(Declaration {
            id: DeclarationId(603),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 21966, 21973),
        });
        declarations.push(Declaration {
            id: DeclarationId(604),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 22013, 22025),
        });
        declarations.push(Declaration {
            id: DeclarationId(605),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/types.dag", 22040, 22044),
        });
        declarations.push(Declaration {
            id: DeclarationId(606),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 23395, 23413),
        });
        declarations.push(Declaration {
            id: DeclarationId(607),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(214),
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
            span: SourceSpan::new("dsl/std/types.dag", 23506, 23527),
        });
        declarations.push(Declaration {
            id: DeclarationId(608),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 23811, 23818),
        });
        declarations.push(Declaration {
            id: DeclarationId(609),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(218),
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
            span: SourceSpan::new("dsl/std/types.dag", 23851, 23866),
        });
        declarations.push(Declaration {
            id: DeclarationId(610),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(221),
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
            span: SourceSpan::new("dsl/std/types.dag", 24155, 24173),
        });
        declarations.push(Declaration {
            id: DeclarationId(611),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(222),
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
            span: SourceSpan::new("dsl/std/types.dag", 24183, 24201),
        });
        declarations.push(Declaration {
            id: DeclarationId(612),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 24223, 24235),
        });
        declarations.push(Declaration {
            id: DeclarationId(613),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 24323, 24330),
        });
        declarations.push(Declaration {
            id: DeclarationId(614),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 24417, 24424),
        });
        declarations.push(Declaration {
            id: DeclarationId(615),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 24452, 24464),
        });
        declarations.push(Declaration {
            id: DeclarationId(616),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 24476, 24488),
        });
        declarations.push(Declaration {
            id: DeclarationId(617),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
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
            span: SourceSpan::new("dsl/std/types.dag", 24500, 24512),
        });
        declarations.push(Declaration {
            id: DeclarationId(618),
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
            span: SourceSpan::new("dsl/std/types.dag", 24647, 24662),
        });
        declarations.push(Declaration {
            id: DeclarationId(619),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(191),
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
            span: SourceSpan::new("dsl/std/types.dag", 24673, 24686),
        });
        declarations.push(Declaration {
            id: DeclarationId(620),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(190),
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
            span: SourceSpan::new("dsl/std/types.dag", 24702, 24715),
        });
        declarations.push(Declaration {
            id: DeclarationId(621),
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
            span: SourceSpan::new("dsl/std/types.dag", 24741, 24745),
        });
        declarations.push(Declaration {
            id: DeclarationId(622),
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
            span: SourceSpan::new("dsl/std/types.dag", 24748, 24750),
        });
        declarations.push(Declaration {
            id: DeclarationId(623),
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
            span: SourceSpan::new("dsl/std/types.dag", 24753, 24754),
        });
        declarations.push(Declaration {
            id: DeclarationId(624),
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
            span: SourceSpan::new("dsl/std/types.dag", 24757, 24761),
        });
        declarations.push(Declaration {
            id: DeclarationId(625),
            name: None,
            connective: TypeConnective::Cardinality(
                CardinalityPayload::new_unchecked_bypassing_idempotence(
                    DeclarationId(230),
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
            span: SourceSpan::new("dsl/std/types.dag", 24825, 24832),
        });
        declarations.push(Declaration {
            id: DeclarationId(626),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(128),
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
            id: DeclarationId(627),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(93),
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
            id: DeclarationId(628),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(128),
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
            id: DeclarationId(629),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(137),
                arguments: vec![
                    TemplateArgument {
                        parameter: DeclarationId(138),
                        value: DeclarationId(230),
                    },
                    TemplateArgument {
                        parameter: DeclarationId(139),
                        value: DeclarationId(230),
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
            id: DeclarationId(630),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
                output: DeclarationId(128),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(8))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10914, 10935),
        });
        declarations.push(Declaration {
            id: DeclarationId(631),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
                output: DeclarationId(128),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(14))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 10966, 10991),
        });
        declarations.push(Declaration {
            id: DeclarationId(632),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
                output: DeclarationId(128),
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
            span: SourceSpan::new("dsl/std/types.dag", 11122, 11147),
        });
        declarations.push(Declaration {
            id: DeclarationId(633),
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
            span: SourceSpan::new("dsl/std/types.dag", 11305, 11347),
        });
        declarations.push(Declaration {
            id: DeclarationId(634),
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
            span: SourceSpan::new("dsl/std/types.dag", 11534, 11579),
        });
        declarations.push(Declaration {
            id: DeclarationId(635),
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
            span: SourceSpan::new("dsl/std/types.dag", 11580, 11625),
        });
        declarations.push(Declaration {
            id: DeclarationId(636),
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
            span: SourceSpan::new("dsl/std/types.dag", 11626, 11671),
        });
        declarations.push(Declaration {
            id: DeclarationId(637),
            name: Some("<registered predicate not lowered: PathSegment>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12306, 12365),
        });
        declarations.push(Declaration {
            id: DeclarationId(638),
            name: Some("<registered predicate not lowered: GlobSegment>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 12366, 12425),
        });
        declarations.push(Declaration {
            id: DeclarationId(639),
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
            span: SourceSpan::new("dsl/std/types.dag", 12530, 12573),
        });
        declarations.push(Declaration {
            id: DeclarationId(640),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
                output: DeclarationId(128),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(23))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13409, 13422),
        });
        declarations.push(Declaration {
            id: DeclarationId(641),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
                output: DeclarationId(128),
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
            span: SourceSpan::new("dsl/std/types.dag", 13454, 13467),
        });
        declarations.push(Declaration {
            id: DeclarationId(642),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
                output: DeclarationId(128),
                body: ArrowBody::UserDefined(BindNodeId::new_unchecked(NodeId(29))),
            },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 13980, 13993),
        });
        declarations.push(Declaration {
            id: DeclarationId(643),
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![DeclarationId(93)],
                output: DeclarationId(128),
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
            span: SourceSpan::new("dsl/std/types.dag", 14025, 14038),
        });
        declarations.push(Declaration {
            id: DeclarationId(644),
            name: Some("<registered predicate not lowered: IntentId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14562, 14617),
        });
        declarations.push(Declaration {
            id: DeclarationId(645),
            name: Some("<registered predicate not lowered: IssueId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14618, 14672),
        });
        declarations.push(Declaration {
            id: DeclarationId(646),
            name: Some("<registered predicate not lowered: RunKey>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14673, 14726),
        });
        declarations.push(Declaration {
            id: DeclarationId(647),
            name: Some("<registered predicate not lowered: ArtifactId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14727, 14784),
        });
        declarations.push(Declaration {
            id: DeclarationId(648),
            name: Some("<registered predicate not lowered: LeaseToken>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14785, 14842),
        });
        declarations.push(Declaration {
            id: DeclarationId(649),
            name: Some("<registered predicate not lowered: WorkerId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14843, 14898),
        });
        declarations.push(Declaration {
            id: DeclarationId(650),
            name: Some("<registered predicate not lowered: CommentId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14899, 14955),
        });
        declarations.push(Declaration {
            id: DeclarationId(651),
            name: Some("<registered predicate not lowered: SignalKey>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 14956, 15012),
        });
        declarations.push(Declaration {
            id: DeclarationId(652),
            name: Some("<registered predicate not lowered: ContentHash>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15013, 15071),
        });
        declarations.push(Declaration {
            id: DeclarationId(653),
            name: Some("<registered predicate not lowered: WorkflowProducerId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15302, 15373),
        });
        declarations.push(Declaration {
            id: DeclarationId(654),
            name: Some("<registered predicate not lowered: WorkflowObserverId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15374, 15445),
        });
        declarations.push(Declaration {
            id: DeclarationId(655),
            name: Some("<registered predicate not lowered: WorkflowProverId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15446, 15513),
        });
        declarations.push(Declaration {
            id: DeclarationId(656),
            name: Some("<registered predicate not lowered: WorkflowRunId>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 15514, 15575),
        });
        declarations.push(Declaration {
            id: DeclarationId(657),
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
            span: SourceSpan::new("dsl/std/types.dag", 15655, 15697),
        });
        declarations.push(Declaration {
            id: DeclarationId(658),
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
            span: SourceSpan::new("dsl/std/types.dag", 16082, 16124),
        });
        declarations.push(Declaration {
            id: DeclarationId(659),
            name: Some("<registered predicate not lowered: FilesystemHandle>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22123, 22187),
        });
        declarations.push(Declaration {
            id: DeclarationId(660),
            name: Some("<registered predicate not lowered: NetworkHandle>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22188, 22245),
        });
        declarations.push(Declaration {
            id: DeclarationId(661),
            name: Some("<registered predicate not lowered: ToolHandle>".to_string()),
            connective: TypeConnective::Conj { children: vec![] },
            type_params: vec![],
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("dsl/std/types.dag", 22246, 22302),
        });
        declarations.push(Declaration {
            id: DeclarationId(662),
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
            id: DeclarationId(663),
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
            id: DeclarationId(664),
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
            id: DeclarationId(665),
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
            id: DeclarationId(666),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 4762, 4771),
        });
        declarations.push(Declaration {
            id: DeclarationId(667),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 4774, 4780),
        });
        declarations.push(Declaration {
            id: DeclarationId(668),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 4783, 4787),
        });
        declarations.push(Declaration {
            id: DeclarationId(669),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(235),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 5335, 5353),
        });
        declarations.push(Declaration {
            id: DeclarationId(670),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(93),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 6155, 6164),
        });
        declarations.push(Declaration {
            id: DeclarationId(671),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(133),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(134),
                    value: DeclarationId(235),
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
            span: SourceSpan::new("dsl/std/unicode.dag", 6404, 6422),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 1873, 1883),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 1888, 1897),
        });
        declarations.push(Declaration {
            id: DeclarationId(674),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "prefix".to_string(),
                    ty: DeclarationId(230),
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
            id: DeclarationId(675),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "suffix".to_string(),
                    ty: DeclarationId(230),
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
            id: DeclarationId(676),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "prefix".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "suffix".to_string(),
                        ty: DeclarationId(230),
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
            id: DeclarationId(677),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![Field {
                    label: "naming".to_string(),
                    ty: DeclarationId(311),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 2416, 2455),
        });
        declarations.push(Declaration {
            id: DeclarationId(678),
            name: None,
            connective: TypeConnective::Conj {
                children: vec![
                    Field {
                        label: "tag_field".to_string(),
                        ty: DeclarationId(230),
                    },
                    Field {
                        label: "naming".to_string(),
                        ty: DeclarationId(311),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 2460, 2527),
        });
        declarations.push(Declaration {
            id: DeclarationId(679),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 2532, 2545),
        });
        declarations.push(Declaration {
            id: DeclarationId(680),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 3264, 3268),
        });
        declarations.push(Declaration {
            id: DeclarationId(681),
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
            span: SourceSpan::new("dsl/std/serialization.dag", 3271, 3275),
        });
        declarations.push(Declaration {
            id: DeclarationId(682),
            name: None,
            connective: TypeConnective::Instantiation {
                template: DeclarationId(45),
                arguments: vec![TemplateArgument {
                    parameter: DeclarationId(46),
                    value: DeclarationId(128),
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
                state: PortState::Resolved(TypeShape::new(DeclarationId(93))),
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
                state: PortState::Resolved(TypeShape::new(DeclarationId(93))),
                produced_by: None,
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
                state: PortState::Resolved(TypeShape::new(DeclarationId(93))),
                produced_by: None,
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
                state: PortState::Resolved(TypeShape::new(DeclarationId(93))),
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
                state: PortState::Resolved(TypeShape::new(DeclarationId(93))),
                produced_by: None,
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
                state: PortState::Resolved(TypeShape::new(DeclarationId(93))),
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
                state: PortState::Resolved(TypeShape::new(DeclarationId(93))),
                produced_by: None,
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
