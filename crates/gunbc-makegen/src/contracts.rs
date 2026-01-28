use gunbc_contracts::*;
use gunbc_ir::{PortName, TypeId};

pub fn context_block() -> BlockContract {
    BlockContract {
        id: "context".into(),
        inputs: vec![],
        outputs: vec![
            PortContract { name: PortName("workspace_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("per_crate_targets".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("lint_targets".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("output_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("force".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
        behavior: BehaviorContract::Pure,
    }
}

pub fn check_block() -> BlockContract {
    BlockContract {
        id: "check".into(),
        inputs: vec![
            PortContract { name: PortName("workspace_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("output_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("force".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("per_crate_targets".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("lint_targets".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("input_hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("makefile_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("needs_generate".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("file_exists".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("workspace_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
            PortContract { name: PortName("per_crate_targets".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("lint_targets".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
        behavior: BehaviorContract::Observe,
    }
}

pub fn parse_workspace_block() -> BlockContract {
    BlockContract {
        id: "parse_workspace".into(),
        inputs: vec![
            PortContract { name: PortName("needs_generate".into()), type_id: TypeId("Bool".into()), optional: false, guard: Some("needs_generate == true".into()) },
            PortContract { name: PortName("workspace_path".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("crate_names".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            PortContract { name: PortName("crate_paths".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            PortContract { name: PortName("crate_is_bin".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            PortContract { name: PortName("crate_is_lib".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
        ],
        behavior: BehaviorContract::Pure,
    }
}

pub fn generate_targets_block() -> BlockContract {
    BlockContract {
        id: "generate_targets".into(),
        inputs: vec![
            PortContract { name: PortName("crate_names".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            PortContract { name: PortName("per_crate_targets".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            PortContract { name: PortName("lint_targets".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("targets".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
        ],
        behavior: BehaviorContract::Pure,
    }
}

pub fn generate_rules_block() -> BlockContract {
    BlockContract {
        id: "generate_rules".into(),
        inputs: vec![
            PortContract { name: PortName("targets".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            PortContract { name: PortName("crate_names".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("rules".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
        ],
        behavior: BehaviorContract::Pure,
    }
}

pub fn compose_makefile_block() -> BlockContract {
    BlockContract {
        id: "compose_makefile".into(),
        inputs: vec![
            PortContract { name: PortName("rules".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            PortContract { name: PortName("input_hash".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        outputs: vec![
            PortContract { name: PortName("content".into()), type_id: TypeId("String".into()), optional: false, guard: None },
        ],
        behavior: BehaviorContract::Pure,
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
        behavior: BehaviorContract::Pure,
    }
}

pub fn sink_block(dry_run: bool) -> BlockContract {
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
        behavior: if dry_run { BehaviorContract::Observe } else { BehaviorContract::WritesWorldIdempotent },
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
            parse_workspace_block(),
            generate_targets_block(),
            generate_rules_block(),
            compose_makefile_block(),
            resolve_block(),
            sink_block(true),
        ];
        for block in &blocks {
            assert!(!block.id.is_empty(), "block must have a non-empty id");
            assert!(!block.outputs.is_empty(), "block '{}' must have outputs", block.id);
        }
    }

    #[test]
    fn sink_behavior_varies_with_dry_run() {
        assert_eq!(sink_block(true).behavior, BehaviorContract::Observe);
        assert_eq!(sink_block(false).behavior, BehaviorContract::WritesWorldIdempotent);
    }
}
