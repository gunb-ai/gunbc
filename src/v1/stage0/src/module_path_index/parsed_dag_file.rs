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

/// WHY A PARSE COULD NOT ANSWER, kept as a type at the only boundary that knows it.
///
/// `parse_dag_content`'s `Option` is the disposition of a CONSUMER population, not a fact about
/// parsing -- its own doc comment says so, and says that closing the mixed policies means routing
/// callers through the typed helper. This is that helper's answer. A caller for which absence is
/// NOT the correct reading (a keyed read, which would otherwise answer "no such declaration" about
/// a file it never looked at) takes this instead of guessing a cause from the file name it was
/// handed. Two ways a file fails to answer, and they are different findings: a reported parse
/// diagnostic, and a file that parses but carries no `module` head, so nothing in it can be named
/// by a qualified key at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DagParseRefusal {
    /// tokenize->parse reported a diagnostic: the message, and the `file:line:col` it sits at when
    /// its span is this file (a synthetic span renders as the file alone, never a fabricated 1:1).
    ParseDiagnostic {
        file: String,
        message: String,
        located: String,
    },
    /// The parse produced no `module` head, so the file names none of its declarations.
    ModuleHeadAbsent { file: String },
    /// There were no bytes to parse. Not a parse failure and not an empty file: the caller could
    /// not look, and a reader that reports "no such declaration" about it is reporting from a file
    /// it never opened.
    ContentAbsent { file: String },
}

/// The refusal as ONE sentence, so no layer above re-derives prose from a variant it cannot see.
/// The located form is what the `Option` population prints today; holding the rendering here is
/// what lets both populations say the same thing about the same failure.
pub fn dag_parse_refusal_sentence(refusal: &DagParseRefusal) -> String {
    match refusal {
        DagParseRefusal::ParseDiagnostic {
            located, message, ..
        } => format!("{located}: {message}"),
        DagParseRefusal::ModuleHeadAbsent { file } => {
            format!("{file}: no `module` head; the file names none of its declarations")
        }
        DagParseRefusal::ContentAbsent { file } => {
            format!("{file}: no bytes could be read; nothing in it was looked at")
        }
    }
}

/// The typed sibling of `parse_dag_content`: the same tokenize->parse, the same located
/// diagnostic, and the REASON rather than a `None`. `parse_dag_content` is this function plus that
/// population's disposition (render, then collapse), so the two cannot drift apart.
pub fn parse_dag_content_checked(
    content: &str,
    filename: &str,
) -> Result<ParsedDagFile, DagParseRefusal> {
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
        // THE CAUSE IS KEPT, NOT DROPPED (RFM: parse_errors_are_swallowed_before_the_batch_names_the_file).
        // The ErrorNode holds the diagnostic and its span right here. `parse_dag_content` renders
        // it and collapses; a keyed caller refuses with it. Either way what is reported is a PARSE
        // diagnostic only -- tokenize->parse runs here, no resolve -- and it is the one claim_batch
        // reported as a bare filename while three dispatches guessed syntax (the RFM row names the
        // case). A resolve-layer error, such as the constructor-admission collision that also cost
        // that cycle, does NOT pass through here: it is emitted located by the resolve layer all
        // along (the seed's v1_compiler_infer constructor_admission_diags_from_lookup renders it;
        // claim_batch's resolve_timed prints it under its heading). Do not
        // "fix" a missing resolve cause at this line -- check the caller's display filter first.
        let message = crate::v1_std_core::diagnostic_to_message(error.diagnostic.clone());
        let span = crate::v1_std_core::diagnostic_to_span(error.diagnostic.clone());
        let located = if span.file == filename {
            let index = build_newline_index(filename.to_string(), content.to_string());
            let lc = crate::v1_std_core::byte_to_line_col(index, span.start);
            format!("{filename}:{}:{}", lc.line, lc.col)
        } else {
            filename.to_string()
        };
        return Err(DagParseRefusal::ParseDiagnostic {
            file: filename.to_string(),
            message,
            located,
        });
    }
    let module = result.module.as_ref().ok_or_else(|| {
        DagParseRefusal::ModuleHeadAbsent {
            file: filename.to_string(),
        }
    })?;
    Ok(ParsedDagFile {
        module: module.clone(),
        items: module.children.clone(),
        source_indices,
    })
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
/// allowed to be unspeakable (gunbc.design_argument::fail_closed: an answer must not hide that nothing was
/// looked at).
///
/// So this comment still does NOT claim absence is the correct reading for the class.
/// The sibling population is unreconciled and this change neither reconciles nor closes
/// it; closing it means routing those callers through the typed helper or recording, per
/// caller, why absence is the correct reading of a parse failure there. ONE HAS BEEN ROUTED:
/// the keyed declaration read (`coproduct_reflection`'s `keyed_decl_facts`, through
/// `decl_facts_of_source_checked`) takes `parse_dag_content_checked`, because for a read that
/// answers a named key an unread file is not an absence. What still lands here is the census
/// walk and `decls_parse_only_from_disk`, whose readings of `None` are their own to justify.
pub fn parse_dag_content(content: &str, filename: &str) -> Option<ParsedDagFile> {
    match parse_dag_content_checked(content, filename) {
        Ok(parsed) => Some(parsed),
        // THE SWALLOW IS REPAIRED AT THIS BOUNDARY (RFM: parse_errors_are_swallowed_before_the_batch_names_the_file).
        // The Option signature above stays -- its dispositioning paragraph explains why the None
        // CONSUMER population is unreconciled and off-limits here. What must not stay is the
        // silence: the diagnostic renders, with its position, before the None is returned. The
        // headless arm was never printed and still is not: it is not a parse diagnostic, and the
        // censuses that skip on None have always read it as absence.
        Err(refusal @ DagParseRefusal::ParseDiagnostic { .. }) => {
            eprintln!("{}", dag_parse_refusal_sentence(&refusal));
            None
        }
        Err(DagParseRefusal::ModuleHeadAbsent { .. }) => None,
        // Unreachable through this signature -- a `&str` is never absent -- and spelled out rather
        // than wildcarded, so a variant added to `DagParseRefusal` has to be dispositioned here
        // rather than silently absorbed (DESIGN: no `_` that silences exhaustiveness).
        Err(DagParseRefusal::ContentAbsent { .. }) => None,
    }
}

pub fn parse_dag_file(path: &Path) -> Option<ParsedDagFile> {
    let content = std::fs::read_to_string(path).ok()?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    parse_dag_content(&content, filename)
}

pub fn parse_file(path: &Path) -> Option<(Rc<im::Vector<Rc<Node>>>, SourceIndices)> {
    parse_dag_file(path).map(|parsed| (parsed.items, parsed.source_indices))
}
