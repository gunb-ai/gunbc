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

/// SIBLING PARSE-TO-`None` PATH — SILENCE REPAIRED, SIGNATURE DELIBERATELY NOT CHANGED HERE.
///
/// This still collapses a parse failure into `None`, i.e. into absence, and the
/// SIGNATURE stays that way on purpose: its consumer population has MIXED policies
/// rather than one intentional policy — `decls_parse_only_from_disk` converts `None`
/// into a fail-closed error, while at least one census walk (`coproduct_reflection`'s
/// `decl_facts_corpus_walk`) uses `let Some(parsed) = ... else { continue; }` and
/// silently excludes the file. Converting the signature would change those censuses'
/// denominators, which is a separate subject with its own witnesses.
///
/// What was repaired here (RFM: parse_errors_are_swallowed_before_the_batch_names_the_file)
/// is the SILENCE, not the signature: before this, the ErrorNode — message, file, offset —
/// was in scope at the collapse and dropped, so every layer above could only name the file
/// it was handed (`claim_batch: cannot parse --entry <path> while validating functions`).
/// A parse failure now renders its message with its position (the same located-render the
/// parse sweep carries) BEFORE the None is returned. Absence may still be the correct
/// READING of a parse failure for a given caller — but "could not look" is no longer
/// allowed to be unspeakable (DESIGN section 5: an answer must not hide that nothing was
/// looked at).
///
/// So this comment still does NOT claim absence is the correct reading for the class.
/// The sibling population is unreconciled and this change neither reconciles nor closes
/// it; closing it means routing those callers through the typed helper or recording, per
/// caller, why absence is the correct reading of a parse failure there.
pub fn parse_dag_content(content: &str, filename: &str) -> Option<ParsedDagFile> {
    match try_parse_dag_content(content, filename) {
        Ok(parsed) => Some(parsed),
        Err(DagParseFailure::Diagnostic(error)) => {
            // THE SWALLOW IS REPAIRED AT THIS BOUNDARY (RFM: parse_errors_are_swallowed_before_the_batch_names_the_file).
            // The Option signature stays: its dispositioning paragraph above explains why the None
            // CONSUMER population is unreconciled. What must not stay is the silence, so the
            // diagnostic is rendered with its position before the collapse. A resolve-layer error
            // does NOT pass through here: it is emitted located by the resolve layer.
            let message = crate::v1_std_core::diagnostic_to_message(error.clone());
            let span = crate::v1_std_core::diagnostic_to_span(error);
            let located = if span.file == filename {
                let index = build_newline_index(filename.to_string(), content.to_string());
                let lc = crate::v1_std_core::byte_to_line_col(index, span.start);
                format!("{filename}:{}:{}", lc.line, lc.col)
            } else {
                filename.to_string()
            };
            eprintln!("{located}: {message}");
            None
        }
        Err(DagParseFailure::NoModuleProduced) => None,
    }
}

/// Why a `.dag` text produced no parsed file: the parser's own diagnostic, or a parse that
/// reported no error and still produced no module. Typed so a caller can REFUSE with the file
/// located (`ModuleBindingRefusal::rendered`, through `cli_run::refuse_unparseable_corpus_sources`) instead of receiving the `None` that `parse_dag_content`
/// collapses both into.
pub enum DagParseFailure {
    Diagnostic(Rc<crate::v1_std_core::CompilerDiagnostic>),
    NoModuleProduced,
}

pub fn try_parse_dag_content(
    content: &str,
    filename: &str,
) -> Result<ParsedDagFile, DagParseFailure> {
    let tokens = tokenize(
        content.to_string(),
        filename.to_string(),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    let source_index = build_newline_index(filename.to_string(), content.to_string());
    let mut indices = HashMap::new();
    indices.insert(filename.to_string(), source_index);
    let source_indices: SourceIndices = Rc::new(indices);
    let result = parse(tokens, source_indices.clone());
    if let Some(error) = result.error.as_ref() {
        return Err(DagParseFailure::Diagnostic(error.diagnostic.clone()));
    }
    let module = result
        .module
        .as_ref()
        .ok_or(DagParseFailure::NoModuleProduced)?;
    Ok(ParsedDagFile {
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
