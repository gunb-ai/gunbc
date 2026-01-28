use gunbc_contracts::BlockContract;

/// Emit per-block modules with `&'static str` constants for port names.
///
/// For a block with id "auth_check" and output ports "token" and "needs_create",
/// generates:
/// ```ignore
/// pub mod auth_check {
///     pub const TOKEN: &str = "token";
///     pub const NEEDS_CREATE: &str = "needs_create";
/// }
/// ```
pub fn emit_port_constants(block: &BlockContract) -> String {
    let mod_name = block.id.replace('-', "_");
    let mut out = String::new();

    out.push_str(&format!("pub mod {} {{\n", mod_name));

    let mut seen = std::collections::HashSet::new();
    for p in block.inputs.iter().chain(block.outputs.iter()) {
        let name = &p.name.0;
        if seen.insert(name.clone()) {
            let const_name = name.to_uppercase();
            out.push_str(&format!("    pub const {}: &str = \"{}\";\n", const_name, name));
        }
    }

    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_contracts::{BehaviorContract, PortContract};
    use gunbc_ir::{PortName, TypeId};

    #[test]
    fn generates_constants_for_block() {
        let block = BlockContract {
            id: "auth_check".into(),
            inputs: vec![],
            outputs: vec![
                PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
                PortContract { name: PortName("needs_create".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            ],
            behavior: BehaviorContract::Observe,
        };
        let code = emit_port_constants(&block);
        assert!(code.contains("pub mod auth_check {"));
        assert!(code.contains("pub const TOKEN: &str = \"token\";"));
        assert!(code.contains("pub const NEEDS_CREATE: &str = \"needs_create\";"));
    }

    #[test]
    fn deduplicates_shared_port_names() {
        let block = BlockContract {
            id: "filter".into(),
            inputs: vec![
                PortContract { name: PortName("files".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            ],
            outputs: vec![
                PortContract { name: PortName("files".into()), type_id: TypeId("StrList".into()), optional: false, guard: None },
            ],
            behavior: BehaviorContract::Pure,
        };
        let code = emit_port_constants(&block);
        // "files" should appear only once as a constant
        assert_eq!(code.matches("pub const FILES").count(), 1);
    }
}
