use im_rc::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{build_newline_index, diagnostic_to_message, node_name_span, SourceSpan};

/// One parse-derived module⇄path row for manifest emission (host binding authority).
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedModuleBinding {
    pub module_path: String,
    pub ident_span: Rc<SourceSpan>,
}

/// Returns true when the first non-blank, non-comment line starts with `module `.
/// Used only to distinguish module-less fragments (skip on parse failure) from
/// broken module declarations (refuse per §5) — not as a binding authority.
fn module_declaration_line_present(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return true;
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    false
}

/// Parse a `.dag` file's module declaration through the v1 bootstrap parser.
///
/// `Ok(None)` — module-less fragment (no `module` declaration).
/// `Ok(Some(_))` — parsed module name + ident span.
/// `Err(_)` — bootstrap parse failed; callers on the module-index path must refuse (§5).
pub fn parse_module_binding(
    path: &Path,
    content: &str,
) -> Result<Option<ParsedModuleBinding>, String> {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    let tokens = tokenize(content.to_string(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content.to_string());
    let mut indices = HashMap::new();
    indices.insert(filename.to_string(), source_index);
    let source_indices = Rc::new(indices);
    let result = parse(tokens, source_indices);
    if let Some(err) = result.error.as_ref() {
        if module_declaration_line_present(content) {
            return Err(format!(
                "parse error in {}: {}",
                path.display(),
                diagnostic_to_message(err.diagnostic.clone())
            ));
        }
        return Ok(None);
    }
    let Some(module) = result.module.as_ref() else {
        return Ok(None);
    };
    if module.name.is_empty() {
        return Ok(None);
    }
    Ok(Some(ParsedModuleBinding {
        module_path: module.name.clone(),
        ident_span: node_name_span(module.clone()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_module_binding_none_for_moduleless_fixture() {
        let path = Path::new("fixture.dag");
        let fixture = "type Foo { x: Int }\n";
        assert!(matches!(parse_module_binding(path, fixture), Ok(None)));
    }

    #[test]
    fn parse_module_binding_some_for_module_decl() {
        let path = Path::new("fixture.dag");
        let binding = parse_module_binding(path, "module v1.test.fixture\n")
            .expect("module decl must parse")
            .expect("module decl must bind");
        assert_eq!(binding.module_path, "v1.test.fixture");
        assert_eq!(binding.ident_span.start, 7);
    }

    #[test]
    fn parse_orchestration_dag_binding() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("../../src/v2/std/orchestration.dag");
        let content = std::fs::read_to_string(&path).expect("read orchestration.dag");
        let result = parse_module_binding(&path, &content);
        match &result {
            Ok(Some(b)) => assert_eq!(b.module_path, "v2.std.orchestration"),
            other => panic!("unexpected: {:?}", other),
        }
    }
}
