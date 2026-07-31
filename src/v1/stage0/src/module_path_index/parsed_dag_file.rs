use im::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{build_newline_index, diagnostic_to_message, NewlineIndex, Node};

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

/// Parse-only module items for one `.dag` file (no resolve). Shared substrate for
/// `decl_facts(roots)` (#5966) and emit-only corpus audits.
pub struct ParsedDagFile {
    pub items: Rc<im::Vector<Rc<Node>>>,
    pub source_indices: SourceIndices,
}

/// Parse a `.dag` file, surfacing the concrete failure cause (read / parse-error /
/// no-module). Callers that need a located refusal must use this — collapsing the
/// three states into `Option::None` is a DESIGN §5 state-space conflation.
pub fn parse_dag_file_or_err(path: &Path) -> Result<ParsedDagFile, String> {
    let display = path.display();
    let content = std::fs::read_to_string(path).map_err(|e| format!("read {display}: {e}"))?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    let tokens = tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut indices = HashMap::new();
    indices.insert(filename.to_string(), source_index);
    let source_indices: SourceIndices = Rc::new(indices);
    let result = parse(tokens, source_indices.clone());
    if let Some(err) = &result.error {
        return Err(format!(
            "parse error in {display}: {}",
            diagnostic_to_message(err.diagnostic.clone())
        ));
    }
    let module = result
        .module
        .as_ref()
        .ok_or_else(|| format!("parse produced no module for {display}"))?;
    Ok(ParsedDagFile {
        items: module.children.clone(),
        source_indices,
    })
}

pub fn parse_dag_file(path: &Path) -> Option<ParsedDagFile> {
    parse_dag_file_or_err(path).ok()
}

pub fn parse_file(path: &Path) -> Option<(Rc<im::Vector<Rc<Node>>>, SourceIndices)> {
    parse_dag_file(path).map(|parsed| (parsed.items, parsed.source_indices))
}
