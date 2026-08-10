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
    pub module: Rc<Node>,
    pub items: Rc<im::Vector<Rc<Node>>>,
    pub source_indices: SourceIndices,
}

/// Why a source did not yield parsed items. Carried rather than collapsed into `None`, because
/// a caller assembling a declaration population needs to tell "this module declares nothing"
/// from "this module could not be read" — silently dropping the second makes every declaration
/// in it report as unbound for a reason that is about the drop rather than about the
/// declaration, which is the empty-observation narrow DESIGN's failure-mode list names.
#[derive(Debug, Clone)]
pub enum DagSourceParseRefusal {
    Unreadable { detail: String },
    ParseError { detail: String },
    NoModuleDeclared,
}

impl std::fmt::Display for DagSourceParseRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreadable { detail } => write!(f, "Unreadable: {detail}"),
            Self::ParseError { detail } => write!(f, "ParseError: {detail}"),
            Self::NoModuleDeclared => write!(f, "NoModuleDeclared"),
        }
    }
}

/// The parse itself, over content a caller already holds.
///
/// `parse_dag_file` is this function's filesystem instantiation — the read is the only thing
/// that bound the walk to disk, and a caller working from an in-memory source pool (a
/// `MultiEntryIndex`) must reach the same parse without going back to the filesystem, or the
/// declaration population it assembles describes a different tree than the one it will decide
/// over. One parse authority, two source media.
///
/// `name` keys the newline index and every span's `file`, exactly as the filesystem path did.
pub fn parse_dag_source(name: &str, content: &str) -> Result<ParsedDagFile, DagSourceParseRefusal> {
    let tokens = tokenize(content.to_string(), name.to_string());
    let source_index = build_newline_index(name.to_string(), content.to_string());
    let mut indices = HashMap::new();
    indices.insert(name.to_string(), source_index);
    let source_indices: SourceIndices = Rc::new(indices);
    let result = parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        return Err(DagSourceParseRefusal::ParseError {
            detail: format!("{err:?}"),
        });
    }
    let Some(module) = result.module.as_ref() else {
        return Err(DagSourceParseRefusal::NoModuleDeclared);
    };
    Ok(ParsedDagFile {
        module: module.clone(),
        items: module.children.clone(),
        source_indices,
    })
}

pub fn parse_dag_file(path: &Path) -> Option<ParsedDagFile> {
    let content = std::fs::read_to_string(path).ok()?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    parse_dag_source(filename, &content).ok()
}

pub fn parse_file(path: &Path) -> Option<(Rc<im::Vector<Rc<Node>>>, SourceIndices)> {
    parse_dag_file(path).map(|parsed| (parsed.items, parsed.source_indices))
}
