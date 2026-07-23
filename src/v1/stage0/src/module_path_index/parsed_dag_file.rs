use im::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{build_newline_index, NewlineIndex, Node};

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

/// Parse-only module items for one `.dag` file (no resolve). Shared substrate for
/// `decl_facts(roots)` (#5966) and emit-only corpus audits.
pub struct ParsedDagFile {
    pub items: Rc<im::Vector<Rc<Node>>>,
    pub source_indices: SourceIndices,
}

pub fn parse_dag_file(path: &Path) -> Option<ParsedDagFile> {
    let content = std::fs::read_to_string(path).ok()?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    let tokens = tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut indices = HashMap::new();
    indices.insert(filename.to_string(), source_index);
    let source_indices: SourceIndices = Rc::new(indices);
    let result = parse(tokens, source_indices.clone());
    if result.error.is_some() {
        return None;
    }
    let module = result.module.as_ref()?;
    Some(ParsedDagFile {
        items: module.children.clone(),
        source_indices,
    })
}

pub fn parse_file(path: &Path) -> Option<(Rc<im::Vector<Rc<Node>>>, SourceIndices)> {
    parse_dag_file(path).map(|parsed| (parsed.items, parsed.source_indices))
}
