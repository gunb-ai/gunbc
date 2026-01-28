use gunbc_contracts::BlockContract;

/// Emit typed input/output structs for a block contract.
///
/// For a block with id "auth_check", generates:
/// - `AuthCheckInputs` struct with a field per input port
/// - `AuthCheckOutputs` struct with a field per output port
/// - `AuthCheckInputs::from_values(HashMap) -> Self`
/// - `AuthCheckOutputs::into_values(self) -> HashMap`
pub fn emit_io_structs(block: &BlockContract) -> String {
    let pascal = to_pascal_case(&block.id);
    let mut out = String::new();

    // Inputs struct
    out.push_str(&format!("/// Typed inputs for `{}`.\n", block.id));
    out.push_str("#[derive(Debug, Clone)]\n");
    out.push_str(&format!("pub struct {}Inputs {{\n", pascal));
    for p in &block.inputs {
        out.push_str(&format!("    pub {}: Value,\n", sanitize_ident(&p.name.0)));
    }
    out.push_str("}\n\n");

    out.push_str(&format!("impl {}Inputs {{\n", pascal));
    out.push_str("    pub fn from_values(mut m: HashMap<String, Value>) -> Self {\n");
    out.push_str("        Self {\n");
    for p in &block.inputs {
        let ident = sanitize_ident(&p.name.0);
        out.push_str(&format!(
            "            {}: m.remove(\"{}\").unwrap_or(Value::Unit),\n",
            ident, p.name.0
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    // Outputs struct
    out.push_str(&format!("/// Typed outputs for `{}`.\n", block.id));
    out.push_str("#[derive(Debug, Clone)]\n");
    out.push_str(&format!("pub struct {}Outputs {{\n", pascal));
    for p in &block.outputs {
        out.push_str(&format!("    pub {}: Value,\n", sanitize_ident(&p.name.0)));
    }
    out.push_str("}\n\n");

    out.push_str(&format!("impl {}Outputs {{\n", pascal));
    out.push_str("    pub fn into_values(self) -> HashMap<String, Value> {\n");
    out.push_str("        let mut m = HashMap::new();\n");
    for p in &block.outputs {
        let ident = sanitize_ident(&p.name.0);
        out.push_str(&format!(
            "        m.insert(\"{}\".into(), self.{});\n",
            p.name.0, ident
        ));
    }
    out.push_str("        m\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

fn sanitize_ident(s: &str) -> String {
    s.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_contracts::PortContract;
    use gunbc_ir::{PortName, TypeId};

    #[test]
    fn emit_io_generates_structs() {
        let block = BlockContract {
            id: "auth_check".into(),
            inputs: vec![],
            outputs: vec![
                PortContract { name: PortName("token".into()), type_id: TypeId("Secret".into()), optional: false, guard: None },
                PortContract { name: PortName("needs_create".into()), type_id: TypeId("Bool".into()), optional: false, guard: None },
            ],
        };
        let code = emit_io_structs(&block);
        assert!(code.contains("pub struct AuthCheckInputs"));
        assert!(code.contains("pub struct AuthCheckOutputs"));
        assert!(code.contains("pub token: Value"));
        assert!(code.contains("pub needs_create: Value"));
    }
}
