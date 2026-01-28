use gunbc_contracts::*;
use gunbc_ir::{NodeId, PortName, TypeId};

pub fn context_block() -> BlockContract {
    BlockContract {
        id: "context".into(),
        inputs: vec![],
        outputs: vec![
            PortContract { name: PortName("workspace_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("output_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("force".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
    }
}

pub fn check_block() -> BlockContract {
    BlockContract {
        id: "check".into(),
        inputs: vec![
            PortContract { name: PortName("workspace_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("output_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("force".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("input_hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("makefile_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("needs_generate".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("file_exists".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
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

pub fn resolve_block() -> BlockContract {
    BlockContract {
        id: "resolve".into(),
        inputs: vec![
            PortContract { name: PortName("content".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("input_hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("makefile_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("needs_generate".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("file_exists".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("content".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("needs_write".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("makefile_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("file_existed".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
    }
}

pub fn sink_block(_dry_run: bool) -> BlockContract {
    BlockContract {
        id: "sink".into(),
        inputs: vec![
            PortContract { name: PortName("content".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("needs_write".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("makefile_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("file_existed".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("status".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

pub fn makegen_pattern() -> PatternContract {
    PatternContract {
        name: "makegen".into(),
        slots: vec![
            SlotContract { node_id: NodeId("context".into()), block_id: "context".into() },
            SlotContract { node_id: NodeId("check".into()), block_id: "check".into() },
            SlotContract { node_id: NodeId("compose".into()), block_id: "compose".into() },
            SlotContract { node_id: NodeId("resolve".into()), block_id: "resolve".into() },
            SlotContract { node_id: NodeId("sink".into()), block_id: "sink".into() },
        ],
        edges: vec![
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("workspace_path".into()), to_node: NodeId("check".into()), to_port: PortName("workspace_path".into()) },
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("output_path".into()), to_node: NodeId("check".into()), to_port: PortName("output_path".into()) },
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("force".into()), to_node: NodeId("check".into()), to_port: PortName("force".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("input_hash".into()), to_node: NodeId("compose".into()), to_port: PortName("input_hash".into()) },
            EdgeContract { from_node: NodeId("compose".into()), from_port: PortName("content".into()), to_node: NodeId("resolve".into()), to_port: PortName("content".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("input_hash".into()), to_node: NodeId("resolve".into()), to_port: PortName("input_hash".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("makefile_path".into()), to_node: NodeId("resolve".into()), to_port: PortName("makefile_path".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("needs_generate".into()), to_node: NodeId("resolve".into()), to_port: PortName("needs_generate".into()) },
            EdgeContract { from_node: NodeId("check".into()), from_port: PortName("file_exists".into()), to_node: NodeId("resolve".into()), to_port: PortName("file_exists".into()) },
            EdgeContract { from_node: NodeId("resolve".into()), from_port: PortName("content".into()), to_node: NodeId("sink".into()), to_port: PortName("content".into()) },
            EdgeContract { from_node: NodeId("resolve".into()), from_port: PortName("needs_write".into()), to_node: NodeId("sink".into()), to_port: PortName("needs_write".into()) },
            EdgeContract { from_node: NodeId("resolve".into()), from_port: PortName("makefile_path".into()), to_node: NodeId("sink".into()), to_port: PortName("makefile_path".into()) },
            EdgeContract { from_node: NodeId("resolve".into()), from_port: PortName("file_existed".into()), to_node: NodeId("sink".into()), to_port: PortName("file_existed".into()) },
        ],
        export_slot: NodeId("sink".into()),
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
            resolve_block(),
            sink_block(true),
        ];
        for block in &blocks {
            assert!(!block.id.is_empty(), "block must have a non-empty id");
            assert!(!block.outputs.is_empty(), "block '{}' must have outputs", block.id);
        }
    }
}
