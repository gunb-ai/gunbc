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

/// SIBLING PARSE-TO-`None` PATH — DISPOSITIONED, DELIBERATELY NOT CHANGED HERE.
///
/// This collapses a parse failure into `None`, i.e. into absence, with no
/// diagnostic and no location — the same class `index::parse_module_binding` was
/// repaired for. It is NOT repaired in the same change because its consumers
/// differ: they are corpus censuses and walks (e.g. `coproduct_reflection`'s
/// `decl_facts_corpus_walk`) that treat an unparseable file as out-of-population
/// by design, and one of them, `decls_parse_only_from_disk`, is ALREADY the
/// fail-closed counterpart. Converting this signature would change those
/// censuses' denominators, which is a separate subject with its own witnesses.
///
/// So the class is NOT declared closed by the module-index repair. Closing it
/// means either routing these callers through the typed helper or recording, per
/// caller, why absence is the correct reading of a parse failure there.
pub fn parse_dag_content(content: &str, filename: &str) -> Option<ParsedDagFile> {
    let tokens = tokenize(content.to_string(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content.to_string());
    let mut indices = HashMap::new();
    indices.insert(filename.to_string(), source_index);
    let source_indices: SourceIndices = Rc::new(indices);
    let result = parse(tokens, source_indices.clone());
    if result.error.is_some() {
        return None;
    }
    let module = result.module.as_ref()?;
    Some(ParsedDagFile {
        module: module.clone(),
        items: module.children.clone(),
        source_indices,
    })
}

pub fn parse_dag_file(path: &Path) -> Option<ParsedDagFile> {
    let content = std::fs::read_to_string(path).ok()?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    parse_dag_content(&content, filename)
}

pub fn parse_file(path: &Path) -> Option<(Rc<im::Vector<Rc<Node>>>, SourceIndices)> {
    parse_dag_file(path).map(|parsed| (parsed.items, parsed.source_indices))
}
