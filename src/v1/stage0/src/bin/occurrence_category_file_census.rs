#![allow(clippy::disallowed_macros)]

//! Named instrument: per-file occurrence-category census.
//!
//! Re-derives, for each named source path: declaration count, reference count,
//! clause-E-selected reference count, NamespaceSegmentOccurrence reference count,
//! and dotted TypeOccurrence reference count. DESIGN §6 — name the instrument;
//! never transcribe these figures into authority prose.
//!
//!   ctrl-build --remote -- bash -lc \
//!     'cargo build --release -p v1-compiler --bin occurrence_category_file_census \
//!      && ./target/release/occurrence_category_file_census <path.dag>...'

use std::collections::HashMap;
use std::process::ExitCode;

use v1_compiler::std_occurrence_binding_candidates::reference_derived_dependency_binding_references;
use v1_compiler::std_occurrence_identity::OccurrenceCategory;
use v1_compiler::v1_compiler_parse::parse_with_table;
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{build_newline_index, empty_intern_table};

fn refuse(msg: &str) -> ExitCode {
    eprintln!("occurrence_category_file_census: REFUSED: {msg}");
    ExitCode::from(1)
}

fn census_one(path: &str) -> Result<(), String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("could not read {path}: {e}"))?;
    let index = build_newline_index(path.to_string(), source.clone());
    let source_indices = v1_compiler::v1_rt::rc_map_insert(
        v1_compiler::v1_rt::rc_empty_map(),
        path.to_string(),
        index,
    );
    let parsed = parse_with_table(
        tokenize(source, path.to_string()),
        source_indices,
        empty_intern_table(),
    );
    if let Some(err) = parsed.result.error.clone() {
        return Err(format!("parse refused for {path}: {err:?}"));
    }
    let transport = parsed.occurrence_transport.clone();
    let mut by_id: HashMap<i64, String> = HashMap::new();
    for e in transport.index.entries.iter() {
        by_id.insert(
            e.projection.occurrence.value,
            e.projection.authored_name.clone(),
        );
    }
    let selected = reference_derived_dependency_binding_references(transport.references.clone());
    let mut ns_segment_refs: u64 = 0;
    let mut dotted_type_refs: u64 = 0;
    for r in transport.references.iter() {
        match r.category {
            OccurrenceCategory::NamespaceSegmentOccurrence => ns_segment_refs += 1,
            OccurrenceCategory::TypeOccurrence => {
                if by_id
                    .get(&r.occurrence.value)
                    .map(|n| n.contains('.'))
                    .unwrap_or(false)
                {
                    dotted_type_refs += 1;
                }
            }
            _ => {}
        }
    }
    // One machine line per file — stable keys for re-derivation consumers.
    println!(
        "path={path} decls={} refs={} clause_e_selected={} ns_segment_refs={ns_segment_refs} dotted_type_refs={dotted_type_refs}",
        transport.declarations.len(),
        transport.references.len(),
        selected.len(),
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return refuse("usage: occurrence_category_file_census <path.dag>...");
    }
    for path in &args {
        if let Err(e) = census_one(path) {
            return refuse(&e);
        }
    }
    ExitCode::SUCCESS
}
