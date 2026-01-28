use gunbc_contracts::*;
use gunbc_ir::{NodeId, PortName, TypeId};

pub fn context_block() -> BlockContract {
    BlockContract {
        id: "context".into(),
        inputs: vec![],
        outputs: vec![
            PortContract { name: PortName("file_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("force".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("input_hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

pub fn check_block() -> BlockContract {
    BlockContract {
        id: "check".into(),
        inputs: vec![
            PortContract { name: PortName("file_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("force".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("input_hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("input_hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("file_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("needs_write".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("file_existed".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
    }
}

pub fn compose_block() -> BlockContract {
    BlockContract {
        id: "compose".into(),
        inputs: vec![
            PortContract { name: PortName("input_hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("content".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

pub fn sink_block(_dry_run: bool) -> BlockContract {
    BlockContract {
        id: "sink".into(),
        inputs: vec![
            PortContract { name: PortName("content".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("needs_write".into()), type_id: TypeId("Bool".into()), optional: false, guard: Some("needs_write == true".into()) },
            PortContract { name: PortName("file_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("file_existed".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("write_status".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

pub fn resolve_block() -> BlockContract {
    BlockContract {
        id: "resolve".into(),
        inputs: vec![
            PortContract { name: PortName("needs_write".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("write_status".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("status".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

pub fn gitignoregen_pattern() -> PatternContract {
    PatternContract {
        name: "gitignoregen".into(),
        slots: vec![
            SlotContract { node_id: NodeId("context".into()), block_id: "context".into() },
            SlotContract { node_id: NodeId("check".into()), block_id: "check".into() },
            SlotContract { node_id: NodeId("compose".into()), block_id: "compose".into() },
            SlotContract { node_id: NodeId("sink".into()), block_id: "sink".into() },
            SlotContract { node_id: NodeId("resolve".into()), block_id: "resolve".into() },
        ],
        edges: vec![
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("file_path".into()), to_node: NodeId("check".into()), to_port: PortName("file_path".into()) },
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("force".into()), to_node: NodeId("check".into()), to_port: PortName("force".into()) },
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("input_hash".into()), to_node: NodeId("check".into()), to_port: PortName("input_hash".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("input_hash".into()), to_node: NodeId("compose".into()), to_port: PortName("input_hash".into()) },
            EdgeContract { from_node: NodeId("compose".into()), from_port: PortName("content".into()), to_node: NodeId("sink".into()), to_port: PortName("content".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("needs_write".into()), to_node: NodeId("sink".into()), to_port: PortName("needs_write".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("file_path".into()), to_node: NodeId("sink".into()), to_port: PortName("file_path".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("file_existed".into()), to_node: NodeId("sink".into()), to_port: PortName("file_existed".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("needs_write".into()), to_node: NodeId("resolve".into()), to_port: PortName("needs_write".into()) },
            EdgeContract { from_node: NodeId("sink".into()), from_port: PortName("write_status".into()), to_node: NodeId("resolve".into()), to_port: PortName("write_status".into()) },
        ],
        export_slot: NodeId("resolve".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_blocks_have_ids() {
        let blocks: Vec<BlockContract> = vec![
            context_block(),
            check_block(),
            compose_block(),
            sink_block(true),
            resolve_block(),
        ];
        for block in &blocks {
            assert!(!block.id.is_empty(), "block must have a non-empty id");
            assert!(!block.outputs.is_empty(), "block '{}' must have outputs", block.id);
        }
    }
}
