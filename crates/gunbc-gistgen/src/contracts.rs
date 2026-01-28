use gunbc_contracts::*;
use gunbc_ir::{NodeId, PortName, TypeId};

pub fn auth_check() -> BlockContract {
    BlockContract {
        id: "auth_check".into(),
        inputs: vec![],
        outputs: vec![
            PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
            PortContract { name: PortName("needs_create".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
    }
}

pub fn auth_create() -> BlockContract {
    BlockContract {
        id: "auth_create".into(),
        inputs: vec![
            PortContract { name: PortName("needs_create".into()), type_id: TypeId("Bool".into()), optional: false, guard: Some("needs_create == true".into()) },
        ],
        outputs: vec![
            PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ],
    }
}

pub fn auth_resolve() -> BlockContract {
    BlockContract {
        id: "auth_resolve".into(),
        inputs: vec![
            PortContract { name: PortName("check_token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
            PortContract { name: PortName("create_token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ],
    }
}

pub fn upsert_pattern() -> PatternContract {
    PatternContract {
        name: "auth".into(),
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

pub fn context_block() -> BlockContract {
    BlockContract {
        id: "context".into(),
        inputs: vec![],
        outputs: vec![
            PortContract { name: PortName("repo".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("selection_spec".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

pub fn enumerate_files_block() -> BlockContract {
    BlockContract {
        id: "enumerate_files".into(),
        inputs: vec![
            PortContract { name: PortName("repo".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("files".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
        ],
    }
}

pub fn filter_files_block() -> BlockContract {
    BlockContract {
        id: "filter_files".into(),
        inputs: vec![
            PortContract { name: PortName("files".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            PortContract { name: PortName("selection_spec".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("files".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
        ],
    }
}

pub fn read_files_block() -> BlockContract {
    BlockContract {
        id: "read_files".into(),
        inputs: vec![
            PortContract { name: PortName("files".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("contents".into()), type_id: TypeId("MapStrStr".into()), optional: false, guard: None },
        ],
    }
}

pub fn compose_snapshot_block() -> BlockContract {
    BlockContract {
        id: "compose_snapshot".into(),
        inputs: vec![
            PortContract { name: PortName("contents".into()), type_id: TypeId("MapStrStr".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("snapshot".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

pub fn upload_gist_block(_dry_run: bool) -> BlockContract {
    BlockContract {
        id: "upload_gist".into(),
        inputs: vec![
            PortContract { name: PortName("snapshot".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("gist_url".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_codegen::{verify_acyclic, verify_type_agreement, verify_export_alignment, verify_port_saturation};

    fn auth_blocks() -> Vec<BlockContract> {
        vec![auth_check(), auth_create(), auth_resolve()]
    }

    #[test]
    fn auth_pattern_is_acyclic() {
        verify_acyclic(&upsert_pattern()).unwrap();
    }

    #[test]
    fn auth_pattern_types_agree() {
        verify_type_agreement(&upsert_pattern(), &auth_blocks()).unwrap();
    }

    #[test]
    fn auth_pattern_export_aligned() {
        let wrapper_outputs = vec![
            PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ];
        verify_export_alignment(&upsert_pattern(), &auth_blocks(), &wrapper_outputs).unwrap();
    }

    #[test]
    fn auth_pattern_ports_saturated() {
        verify_port_saturation(&upsert_pattern(), &auth_blocks()).unwrap();
    }

    #[test]
    fn all_blocks_have_ids() {
        let blocks: Vec<BlockContract> = vec![
            context_block(),
            enumerate_files_block(),
            filter_files_block(),
            read_files_block(),
            compose_snapshot_block(),
            upload_gist_block(true),
        ];
        for block in &blocks {
            assert!(!block.id.is_empty(), "block must have a non-empty id");
        }
    }
}
