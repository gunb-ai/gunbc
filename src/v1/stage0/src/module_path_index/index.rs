use im_rc::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{build_newline_index, node_name_span, Node, SourceSpan};

/// One parse-derived module⇄path row for manifest emission (host binding authority).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModuleBinding {
    pub module_path: String,
    pub ident_span: Rc<SourceSpan>,
}

/// Parse a `.dag` file's module declaration through the v1 bootstrap parser.
/// Returns `None` when the file has no module, fails to parse, or declares an empty name.
pub fn parse_module_binding(path: &Path, content: &str) -> Option<ParsedModuleBinding> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("?");
    let tokens = tokenize(content.to_string(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content.to_string());
    let mut indices = HashMap::new();
    indices.insert(filename.to_string(), source_index);
    let source_indices = Rc::new(indices);
    let result = parse(tokens, source_indices);
    if result.error.is_some() {
        return None;
    }
    let module = result.module.as_ref()?;
    if module.name.is_empty() {
        return None;
    }
    Some(ParsedModuleBinding {
        module_path: module.name.clone(),
        ident_span: node_name_span(module.clone()),
    })
}

/// Module path from parsed content — projection used by `build_module_path_index`.
pub fn module_path_from_parsed_content(path: &Path, content: &str) -> Option<String> {
    parse_module_binding(path, content).map(|b| b.module_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_module_binding_none_for_moduleless_fixture() {
        let path = Path::new("fixture.dag");
        let fixture = "type Foo { x: Int }\n";
        assert!(parse_module_binding(path, fixture).is_none());
    }

    #[test]
    fn parse_module_binding_some_for_module_decl() {
        let path = Path::new("fixture.dag");
        let binding = parse_module_binding(path, "module v1.test.fixture\n")
            .expect("module decl must parse");
        assert_eq!(binding.module_path, "v1.test.fixture");
        assert_eq!(binding.ident_span.start, 7);
    }
}
