use gunbc_contracts::{BlockContract, PatternContract, PortContract};
use std::collections::{HashMap, HashSet, VecDeque};

/// Verify that a pattern contract defines an acyclic graph.
pub fn verify_acyclic(pattern: &PatternContract) -> Result<(), String> {
    let node_ids: HashSet<&str> = pattern.slots.iter().map(|s| s.node_id.0.as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = node_ids.iter().map(|&id| (id, 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = node_ids.iter().map(|&id| (id, Vec::new())).collect();

    for edge in &pattern.edges {
        *in_degree.get_mut(edge.to_node.0.as_str()).unwrap() += 1;
        adj.get_mut(edge.from_node.0.as_str()).unwrap().push(&edge.to_node.0);
    }

    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut visited = 0;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        if let Some(neighbors) = adj.get(id) {
            for &neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    if visited == node_ids.len() {
        Ok(())
    } else {
        Err("cycle detected in pattern contract".into())
    }
}

/// Verify that all edges in a pattern reference valid ports with matching types.
pub fn verify_type_agreement(
    pattern: &PatternContract,
    blocks: &[BlockContract],
) -> Result<(), String> {
    let block_map: HashMap<&str, &BlockContract> = blocks.iter().map(|b| (b.id.as_str(), b)).collect();

    for edge in &pattern.edges {
        let from_slot = pattern.slots.iter().find(|s| s.node_id == edge.from_node)
            .ok_or_else(|| format!("edge references unknown slot '{}'", edge.from_node.0))?;
        let to_slot = pattern.slots.iter().find(|s| s.node_id == edge.to_node)
            .ok_or_else(|| format!("edge references unknown slot '{}'", edge.to_node.0))?;

        let from_block = block_map.get(from_slot.block_id.as_str())
            .ok_or_else(|| format!("slot '{}' references unknown block '{}'", from_slot.node_id.0, from_slot.block_id))?;
        let to_block = block_map.get(to_slot.block_id.as_str())
            .ok_or_else(|| format!("slot '{}' references unknown block '{}'", to_slot.node_id.0, to_slot.block_id))?;

        let from_port = from_block.outputs.iter().find(|p| p.name == edge.from_port)
            .ok_or_else(|| format!("block '{}' has no output port '{}'", from_block.id, edge.from_port.0))?;
        let to_port = to_block.inputs.iter().find(|p| p.name == edge.to_port)
            .ok_or_else(|| format!("block '{}' has no input port '{}'", to_block.id, edge.to_port.0))?;

        if from_port.type_id != to_port.type_id {
            return Err(format!(
                "type mismatch on edge {}.{} -> {}.{}: {} != {}",
                edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0,
                from_port.type_id.0, to_port.type_id.0
            ));
        }
    }

    Ok(())
}

/// Verify that the export_slot exists and that its output ports cover
/// the expected wrapper outputs.
pub fn verify_export_alignment(
    pattern: &PatternContract,
    blocks: &[BlockContract],
    wrapper_outputs: &[PortContract],
) -> Result<(), String> {
    let export_slot = pattern.slots.iter().find(|s| s.node_id == pattern.export_slot)
        .ok_or_else(|| format!("export_slot '{}' not found in pattern slots", pattern.export_slot.0))?;

    let block_map: HashMap<&str, &BlockContract> = blocks.iter().map(|b| (b.id.as_str(), b)).collect();
    let export_block = block_map.get(export_slot.block_id.as_str())
        .ok_or_else(|| format!("export slot references unknown block '{}'", export_slot.block_id))?;

    for wp in wrapper_outputs {
        let has_port = export_block.outputs.iter().any(|p| p.name == wp.name && p.type_id == wp.type_id);
        if !has_port {
            return Err(format!(
                "export block '{}' missing output port '{}' (type {}), needed by wrapper",
                export_block.id, wp.name.0, wp.type_id.0
            ));
        }
    }

    Ok(())
}

/// Verify that every input port on every block in the pattern has exactly one incoming edge.
pub fn verify_port_saturation(
    pattern: &PatternContract,
    blocks: &[BlockContract],
) -> Result<(), String> {
    let block_map: HashMap<&str, &BlockContract> = blocks.iter().map(|b| (b.id.as_str(), b)).collect();
    let mut satisfied: HashSet<(&str, &str)> = HashSet::new();

    for edge in &pattern.edges {
        satisfied.insert((edge.to_node.0.as_str(), edge.to_port.0.as_str()));
    }

    for slot in &pattern.slots {
        let block = block_map.get(slot.block_id.as_str())
            .ok_or_else(|| format!("slot '{}' references unknown block '{}'", slot.node_id.0, slot.block_id))?;
        for input in &block.inputs {
            if !satisfied.contains(&(slot.node_id.0.as_str(), input.name.0.as_str())) {
                return Err(format!(
                    "unsatisfied input port '{}' on slot '{}'",
                    input.name.0, slot.node_id.0
                ));
            }
        }
    }

    Ok(())
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
    fn acyclic_pattern_passes() {
        assert!(verify_acyclic(&auth_pattern()).is_ok());
    }

    #[test]
    fn cyclic_pattern_fails() {
        let mut pattern = auth_pattern();
        pattern.edges.push(EdgeContract {
            from_node: NodeId("auth_resolve".into()),
            from_port: PortName("token".into()),
            to_node: NodeId("auth_check".into()),
            to_port: PortName("token".into()),
        });
        assert!(verify_acyclic(&pattern).is_err());
    }

    #[test]
    fn type_agreement_passes() {
        assert!(verify_type_agreement(&auth_pattern(), &auth_blocks()).is_ok());
    }

    #[test]
    fn type_mismatch_fails() {
        let mut blocks = auth_blocks();
        blocks[1].outputs[0].type_id = TypeId("String".into());
        assert!(verify_type_agreement(&auth_pattern(), &blocks).is_err());
    }

    #[test]
    fn export_alignment_passes() {
        let wrapper_outputs = vec![
            PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ];
        assert!(verify_export_alignment(&auth_pattern(), &auth_blocks(), &wrapper_outputs).is_ok());
    }

    #[test]
    fn export_alignment_fails_missing_port() {
        let wrapper_outputs = vec![
            PortContract { name: PortName("nonexistent".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ];
        assert!(verify_export_alignment(&auth_pattern(), &auth_blocks(), &wrapper_outputs).is_err());
    }

    #[test]
    fn port_saturation_passes() {
        assert!(verify_port_saturation(&auth_pattern(), &auth_blocks()).is_ok());
    }

    #[test]
    fn port_saturation_fails_missing_edge() {
        let mut pattern = auth_pattern();
        pattern.edges.retain(|e| !(e.to_node.0 == "auth_create" && e.to_port.0 == "needs_create"));
        assert!(verify_port_saturation(&pattern, &auth_blocks()).is_err());
    }
}
