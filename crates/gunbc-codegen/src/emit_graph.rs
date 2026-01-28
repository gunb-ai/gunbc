use gunbc_contracts::{
    BehaviorContract, BlockContract, PatternContract, PatternDecisionContract,
};

/// Emit a function that constructs a SubDAG from a pattern contract and its block contracts.
///
/// The generated function is generic over `T: Executable`, taking one `T` per slot
/// and returning `Node<T>` — the wrapper node containing the inner DAG.
pub fn emit_subdag_builder(
    pattern: &PatternContract,
    blocks: &[BlockContract],
    wrapper_inputs: &[gunbc_contracts::PortContract],
    wrapper_outputs: &[gunbc_contracts::PortContract],
    wrapper_behavior: &BehaviorContract,
    _pattern_decisions: &[PatternDecisionContract],
) -> String {
    let fn_name = format!("build_{}_subdag", pattern.name);
    let mut out = String::new();

    // Function signature
    out.push_str(&format!(
        "/// Constructs the `{}` SubDAG with correct wiring by construction.\n",
        pattern.name
    ));
    out.push_str(&format!("pub fn {}<T: Executable>(\n", fn_name));
    for slot in &pattern.slots {
        out.push_str(&format!(
            "    {}: T,\n",
            sanitize_ident(&slot.node_id.0)
        ));
    }
    out.push_str(") -> Node<T> {\n");

    // Build inner nodes
    out.push_str("    let inner_nodes: Vec<Node<T>> = vec![\n");
    for slot in &pattern.slots {
        let block = blocks.iter().find(|b| b.id == slot.block_id).unwrap();
        out.push_str("        Node {\n");
        out.push_str(&format!(
            "            id: NodeId(\"{}\".into()),\n",
            slot.node_id.0
        ));

        // Inputs
        out.push_str("            inputs: vec![\n");
        for p in &block.inputs {
            if let Some(ref guard) = p.guard {
                out.push_str(&format!(
                    "                Port {{ name: PortName(\"{}\".into()), type_id: TypeId(\"{}\".into()), guard: Some(\"{}\".into()) }},\n",
                    p.name.0, p.type_id.0, guard
                ));
            } else {
                out.push_str(&format!(
                    "                Port {{ name: PortName(\"{}\".into()), type_id: TypeId(\"{}\".into()), guard: None }},\n",
                    p.name.0, p.type_id.0
                ));
            }
        }
        out.push_str("            ],\n");

        // Outputs
        out.push_str("            outputs: vec![\n");
        for p in &block.outputs {
            out.push_str(&format!(
                "                Port {{ name: PortName(\"{}\".into()), type_id: TypeId(\"{}\".into()), guard: None }},\n",
                p.name.0, p.type_id.0
            ));
        }
        out.push_str("            ],\n");

        // Metadata
        let behavior_str = behavior_to_code(&block.behavior);
        out.push_str(&format!(
            "            metadata: NodeMetadata {{ tool: ToolId(\"{}\".into()), behavior: {} }},\n",
            pattern.tool.0, behavior_str
        ));

        out.push_str(&format!(
            "            body: NodeBody::Opaque({}),\n",
            sanitize_ident(&slot.node_id.0)
        ));
        out.push_str("        },\n");
    }
    out.push_str("    ];\n\n");

    // Build edges
    out.push_str("    let inner_edges: Vec<Edge> = vec![\n");
    for e in &pattern.edges {
        out.push_str(&format!(
            "        Edge {{ from_node: NodeId(\"{}\".into()), from_port: PortName(\"{}\".into()), to_node: NodeId(\"{}\".into()), to_port: PortName(\"{}\".into()) }},\n",
            e.from_node.0, e.from_port.0, e.to_node.0, e.to_port.0
        ));
    }
    out.push_str("    ];\n\n");

    // Build inner DAG metadata with pattern decisions and export_node
    out.push_str("    let inner_metadata = DagMetadata {\n");
    out.push_str("        pattern_decisions: vec![\n");
    out.push_str(&format!(
        "            PatternDecisionEntry {{ tool: ToolId(\"{}\".into()), pattern: \"{}\".into(), decision: PatternDecision::Instantiated }},\n",
        pattern.tool.0, pattern.name
    ));
    out.push_str("        ],\n");

    out.push_str(&format!(
        "        export_node: Some(NodeId(\"{}\".into())),\n",
        pattern.export_slot.0
    ));
    out.push_str("    };\n\n");

    // Inner DAG
    out.push_str("    let inner_dag = Dag {\n");
    out.push_str("        nodes: inner_nodes,\n");
    out.push_str("        edges: inner_edges,\n");
    out.push_str("        metadata: inner_metadata,\n");
    out.push_str("    };\n\n");

    // Wrapper node
    out.push_str("    Node {\n");
    out.push_str(&format!(
        "        id: NodeId(\"{}\".into()),\n",
        pattern.name
    ));

    // Wrapper inputs
    out.push_str("        inputs: vec![\n");
    for p in wrapper_inputs {
        if let Some(ref guard) = p.guard {
            out.push_str(&format!(
                "            Port {{ name: PortName(\"{}\".into()), type_id: TypeId(\"{}\".into()), guard: Some(\"{}\".into()) }},\n",
                p.name.0, p.type_id.0, guard
            ));
        } else {
            out.push_str(&format!(
                "            Port {{ name: PortName(\"{}\".into()), type_id: TypeId(\"{}\".into()), guard: None }},\n",
                p.name.0, p.type_id.0
            ));
        }
    }
    out.push_str("        ],\n");

    // Wrapper outputs
    out.push_str("        outputs: vec![\n");
    for p in wrapper_outputs {
        out.push_str(&format!(
            "            Port {{ name: PortName(\"{}\".into()), type_id: TypeId(\"{}\".into()), guard: None }},\n",
            p.name.0, p.type_id.0
        ));
    }
    out.push_str("        ],\n");

    let wrapper_beh = behavior_to_code(wrapper_behavior);
    out.push_str(&format!(
        "        metadata: NodeMetadata {{ tool: ToolId(\"{}\".into()), behavior: {} }},\n",
        pattern.tool.0, wrapper_beh
    ));
    out.push_str("        body: NodeBody::SubDag(inner_dag),\n");
    out.push_str("    }\n");

    out.push_str("}\n");

    out
}

fn behavior_to_code(b: &BehaviorContract) -> String {
    match b {
        BehaviorContract::Pure => "BehaviorKind::Pure".into(),
        BehaviorContract::Observe => "BehaviorKind::Observe".into(),
        BehaviorContract::WritesWorldIdempotent => {
            "BehaviorKind::WritesWorld(Idempotency::Idempotent)".into()
        }
        BehaviorContract::WritesWorldNotIdempotent => {
            "BehaviorKind::WritesWorld(Idempotency::NotIdempotent)".into()
        }
    }
}

fn sanitize_ident(s: &str) -> String {
    s.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_contracts::*;
    use gunbc_ir::{NodeId, PortName, ToolId, TypeId};

    fn auth_blocks() -> Vec<BlockContract> {
        vec![
            BlockContract {
                id: "auth_check".into(),
                inputs: vec![],
                outputs: vec![
                    PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
                    PortContract { name: PortName("needs_create".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
                ],
                behavior: BehaviorContract::Observe,
            },
            BlockContract {
                id: "auth_create".into(),
                inputs: vec![
                    PortContract { name: PortName("needs_create".into()), type_id: TypeId("Bool".into()), optional: false, guard: Some("needs_create == true".into()) },
                ],
                outputs: vec![
                    PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
                ],
                behavior: BehaviorContract::WritesWorldIdempotent,
            },
            BlockContract {
                id: "auth_resolve".into(),
                inputs: vec![
                    PortContract { name: PortName("check_token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
                    PortContract { name: PortName("create_token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
                ],
                outputs: vec![
                    PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
                ],
                behavior: BehaviorContract::Pure,
            },
        ]
    }

    fn auth_pattern() -> PatternContract {
        PatternContract {
            name: "auth".into(),
            tool: ToolId("auth".into()),
            slots: vec![
                SlotContract { node_id: NodeId("auth_check".into()), block_id: "auth_check".into() },
                SlotContract { node_id: NodeId("auth_create".into()), block_id: "auth_create".into() },
                SlotContract { node_id: NodeId("auth_resolve".into()), block_id: "auth_resolve".into() },
            ],
            edges: vec![
                EdgeContract { from_node: NodeId("auth_check".into()), from_port: PortName("token".into()), to_node: NodeId("auth_resolve".into()), to_port: PortName("check_token".into()) },
                EdgeContract { from_node: NodeId("auth_check".into()), from_port: PortName("needs_create".into()), to_node: NodeId("auth_create".into()), to_port: PortName("needs_create".into()) },
                EdgeContract { from_node: NodeId("auth_create".into()), from_port: PortName("token".into()), to_node: NodeId("auth_resolve".into()), to_port: PortName("create_token".into()) },
            ],
            export_slot: NodeId("auth_resolve".into()),
        }
    }

    #[test]
    fn emit_subdag_builder_generates_valid_code() {
        let blocks = auth_blocks();
        let pattern = auth_pattern();
        let wrapper_outputs = vec![
            PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ];
        let decisions = vec![PatternDecisionContract {
            tool: ToolId("auth".into()),
            pattern: "upsert".into(),
            decision: DecisionContract::Instantiated,
        }];

        let code = emit_subdag_builder(
            &pattern,
            &blocks,
            &[],
            &wrapper_outputs,
            &BehaviorContract::WritesWorldIdempotent,
            &decisions,
        );

        assert!(code.contains("pub fn build_auth_subdag<T: Executable>("));
        assert!(code.contains("auth_check: T,"));
        assert!(code.contains("-> Node<T>"));
        assert!(code.contains("export_node: Some(NodeId(\"auth_resolve\""));
        assert!(code.contains("NodeBody::Opaque(auth_check)"));
    }
}
