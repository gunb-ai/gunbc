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

/// Wrapper block for the auth SubDAG output.
pub fn auth_block() -> BlockContract {
    BlockContract {
        id: "auth".into(),
        inputs: vec![],
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
            PortContract { name: PortName("repo".into()), type_id: TypeId("String".into()), optional: false, guard: None },
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

pub fn wrap_single_gist_file_block() -> BlockContract {
    BlockContract {
        id: "wrap_single_gist_file".into(),
        inputs: vec![
            PortContract { name: PortName("snapshot".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("files".into()), type_id: TypeId("MapStrStr".into()), optional: false, guard: None },
        ],
    }
}

pub fn compose_gist_files_block() -> BlockContract {
    BlockContract {
        id: "compose_gist_files".into(),
        inputs: vec![
            PortContract { name: PortName("contents".into()), type_id: TypeId("MapStrStr".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("files".into()), type_id: TypeId("MapStrStr".into()), optional: false, guard: None },
        ],
    }
}

pub fn build_gist_request_block() -> BlockContract {
    BlockContract {
        id: "build_gist_request".into(),
        inputs: vec![
            PortContract { name: PortName("files".into()), type_id: TypeId("MapStrStr".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("request".into()), type_id: TypeId("GitHub::Gist::CreateRequest".into()), optional: false, guard: None },
        ],
    }
}

pub fn gist_block() -> BlockContract {
    BlockContract {
        id: "gist".into(),
        inputs: vec![
            PortContract { name: PortName("request".into()), type_id: TypeId("GitHub::Gist::CreateRequest".into()), optional: false, guard: None },
            PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("response".into()), type_id: TypeId("GitHub::Gist::CreateResponse".into()), optional: false, guard: None },
            PortContract { name: PortName("gist_url".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
    }
}

pub fn gistgen_pattern_single_file() -> PatternContract {
    PatternContract {
        name: "gistgen_single_file".into(),
        slots: vec![
            SlotContract { node_id: NodeId("context".into()), block_id: "context".into() },
            SlotContract { node_id: NodeId("auth".into()), block_id: "auth".into() },
            SlotContract { node_id: NodeId("enumerate_files".into()), block_id: "enumerate_files".into() },
            SlotContract { node_id: NodeId("filter_files".into()), block_id: "filter_files".into() },
            SlotContract { node_id: NodeId("read_files".into()), block_id: "read_files".into() },
            SlotContract { node_id: NodeId("compose_snapshot".into()), block_id: "compose_snapshot".into() },
            SlotContract { node_id: NodeId("wrap_single_gist_file".into()), block_id: "wrap_single_gist_file".into() },
            SlotContract { node_id: NodeId("build_gist_request".into()), block_id: "build_gist_request".into() },
            SlotContract { node_id: NodeId("gist".into()), block_id: "gist".into() },
        ],
        edges: vec![
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("repo".into()), to_node: NodeId("enumerate_files".into()), to_port: PortName("repo".into()) },
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("selection_spec".into()), to_node: NodeId("filter_files".into()), to_port: PortName("selection_spec".into()) },
            EdgeContract { from_node: NodeId("enumerate_files".into()), from_port: PortName("files".into()), to_node: NodeId("filter_files".into()), to_port: PortName("files".into()) },
            EdgeContract { from_node: NodeId("filter_files".into()), from_port: PortName("files".into()), to_node: NodeId("read_files".into()), to_port: PortName("files".into()) },
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("repo".into()), to_node: NodeId("read_files".into()), to_port: PortName("repo".into()) },
            EdgeContract { from_node: NodeId("read_files".into()), from_port: PortName("contents".into()), to_node: NodeId("compose_snapshot".into()), to_port: PortName("contents".into()) },
            EdgeContract { from_node: NodeId("compose_snapshot".into()), from_port: PortName("snapshot".into()), to_node: NodeId("wrap_single_gist_file".into()), to_port: PortName("snapshot".into()) },
            EdgeContract { from_node: NodeId("wrap_single_gist_file".into()), from_port: PortName("files".into()), to_node: NodeId("build_gist_request".into()), to_port: PortName("files".into()) },
            EdgeContract { from_node: NodeId("build_gist_request".into()), from_port: PortName("request".into()), to_node: NodeId("gist".into()), to_port: PortName("request".into()) },
            EdgeContract { from_node: NodeId("auth".into()), from_port: PortName("token".into()), to_node: NodeId("gist".into()), to_port: PortName("token".into()) },
        ],
        export_slot: NodeId("gist".into()),
    }
}

pub fn gistgen_pattern_file_map() -> PatternContract {
    PatternContract {
        name: "gistgen_file_map".into(),
        slots: vec![
            SlotContract { node_id: NodeId("context".into()), block_id: "context".into() },
            SlotContract { node_id: NodeId("auth".into()), block_id: "auth".into() },
            SlotContract { node_id: NodeId("enumerate_files".into()), block_id: "enumerate_files".into() },
            SlotContract { node_id: NodeId("filter_files".into()), block_id: "filter_files".into() },
            SlotContract { node_id: NodeId("read_files".into()), block_id: "read_files".into() },
            SlotContract { node_id: NodeId("compose_gist_files".into()), block_id: "compose_gist_files".into() },
            SlotContract { node_id: NodeId("build_gist_request".into()), block_id: "build_gist_request".into() },
            SlotContract { node_id: NodeId("gist".into()), block_id: "gist".into() },
        ],
        edges: vec![
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("repo".into()), to_node: NodeId("enumerate_files".into()), to_port: PortName("repo".into()) },
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("selection_spec".into()), to_node: NodeId("filter_files".into()), to_port: PortName("selection_spec".into()) },
            EdgeContract { from_node: NodeId("enumerate_files".into()), from_port: PortName("files".into()), to_node: NodeId("filter_files".into()), to_port: PortName("files".into()) },
            EdgeContract { from_node: NodeId("filter_files".into()), from_port: PortName("files".into()), to_node: NodeId("read_files".into()), to_port: PortName("files".into()) },
            EdgeContract { from_node: NodeId("context".into()), from_port: PortName("repo".into()), to_node: NodeId("read_files".into()), to_port: PortName("repo".into()) },
            EdgeContract { from_node: NodeId("read_files".into()), from_port: PortName("contents".into()), to_node: NodeId("compose_gist_files".into()), to_port: PortName("contents".into()) },
            EdgeContract { from_node: NodeId("compose_gist_files".into()), from_port: PortName("files".into()), to_node: NodeId("build_gist_request".into()), to_port: PortName("files".into()) },
            EdgeContract { from_node: NodeId("build_gist_request".into()), from_port: PortName("request".into()), to_node: NodeId("gist".into()), to_port: PortName("request".into()) },
            EdgeContract { from_node: NodeId("auth".into()), from_port: PortName("token".into()), to_node: NodeId("gist".into()), to_port: PortName("token".into()) },
        ],
        export_slot: NodeId("gist".into()),
    }
}
