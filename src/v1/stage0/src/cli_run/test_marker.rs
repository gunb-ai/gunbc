//! The seed's ONE reading of the authored `test` marker.
//!
//! A test is a declaration carrying the authored `test` marker and nothing else (operator
//! override: the `*_test.dag` file-name convention and every text-line scanner for `test fn` are
//! retired). The marker is captured by the grammar, not by a line scanner: `v1.compiler.parse`
//! `test_marked_item_result` sets `declaration_marker` on the parsed item for both the `fn` and
//! the `data` form, and the v2 side reads the same channel through `v2.std.declaration_marker`
//! `TestMarkerChannel`. This module is the seed's single consumer of that mark, so a caller that
//! wants "which declarations of this source are tests" asks here and never re-derives it from
//! text.
//!
//! BASIS. The reading is the pooled heads reading (`pool_acquire::heads_reading_for`): the same
//! grammar as the full parse with function and data-value bodies skipped at token grain, parsed
//! once per (spelling, bytes) and shared with every other consumer of that file. An item's head
//! carries the marker, so the heads reading is sufficient and a second parse is never paid.
//!
//! A PARSE FAILURE REFUSES. There is no fallback to a text scan and no partial answer: a source
//! that does not parse has no determinable set of test declarations, and a caller that proceeded
//! on an empty or approximate set would be deciding about tests it could not read.

use crate::cli_run::pool_acquire;
use crate::v1_std_core::{
    authored_name_at, byte_to_line_col, diagnostic_to_message, diagnostic_to_span, module_items,
    CompilerDiagnostic, DeclarationMarker,
};
use im::HashMap;
use std::rc::Rc;

/// One declaration carrying the authored `test` marker: its authored name and its byte extent in
/// the source it was read from. The extent runs from the declaration's own head to the head of the
/// next module item (or the end of the source), so it is derived from item heads alone, which is
/// exactly what the heads reading guarantees; a body span is never consulted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TestMarkedDecl {
    pub name: String,
    pub span_start: i64,
    pub span_end: i64,
}

fn located_parse_refusal(file: &str, source: &str, diagnostic: Rc<CompilerDiagnostic>) -> String {
    let span = diagnostic_to_span(diagnostic.clone());
    let message = diagnostic_to_message(diagnostic);
    // THE LOCATION IS THE AUTHORITY'S. This formatted `{file}:{line}:{col}` in host Rust and
    // invented its own unlocated shape (the bare path) for the one case it thought about -- a span
    // naming a different file. The authority owns both shapes now, so the arms that site could not
    // name -- no index for the file, a `<synthetic>` span, an offset outside the indexed text --
    // come back typed and naming the file, instead of silently looking like a location. The located
    // text is unchanged (`m.dag:3:7: <message>; the source does not parse, …`).
    let tail = "; the source does not parse, so its test-marked declarations cannot be read";
    let mut source_indices: im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>> =
        im::HashMap::new();
    if span.file == file {
        source_indices.insert(
            file.to_string(),
            pool_acquire::newline_index_for(file, source),
        );
    }
    format!(
        "{}: {message}{tail}",
        crate::v1_std_core::render_span_location(span, std::rc::Rc::new(source_indices))
    )
}

/// Every declaration of `source` (read as `file`) carrying the authored `test` marker, `fn` and
/// `data` forms alike, in source order.
pub(crate) fn test_marked_decls(source: &str, file: &str) -> Result<Vec<TestMarkedDecl>, String> {
    let reading = pool_acquire::heads_reading_for(file, source);
    if let Some(err) = reading.error.as_ref() {
        return Err(located_parse_refusal(file, source, err.diagnostic.clone()));
    }
    let Some(module) = reading.module.as_ref() else {
        return Err(format!(
            "{file}: the parse produced no module, so its test-marked declarations cannot be read"
        ));
    };
    let source_indices = Rc::new({
        let mut m = HashMap::new();
        m.insert(
            file.to_string(),
            pool_acquire::newline_index_for(file, source),
        );
        m
    });
    let items = module_items(module.clone());
    let mut heads: Vec<i64> = items.iter().map(|item| item.span.start).collect();
    heads.sort_unstable();
    let source_end = source.len() as i64;
    Ok(items
        .iter()
        .filter(|item| item.declaration_marker == DeclarationMarker::TestMarked)
        .map(|item| TestMarkedDecl {
            name: authored_name_at(source_indices.clone(), item.clone()),
            span_start: item.span.start,
            span_end: heads
                .get(heads.partition_point(|head| *head <= item.span.start))
                .copied()
                .unwrap_or(source_end),
        })
        .collect())
}

/// The authored names of the test-marked declarations of `source`: the `names` projection of
/// `test_marked_decls`, which is its one authority.
pub(crate) fn test_marked_decl_names(source: &str, file: &str) -> Result<Vec<String>, String> {
    Ok(test_marked_decls(source, file)?
        .into_iter()
        .map(|d| d.name)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_MARKED_TWO_PLAIN: &str = "module marker_fixture\n\
\n\
fn plain_fn() -> Bool { true }\n\
data plain_data: Bool = false\n\
test fn marked_fn_holds() -> Bool { true }\n\
test data marked_data_fixture: Bool = true\n";

    /// The seed's grammar recurses deeply on real corpus modules; every reading runs on the
    /// stack the production callers run on, so a pass here is not borrowed from the 2 MB test
    /// thread default.
    fn on_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("spawn the 8 MB reader thread")
            .join()
            .expect("reader thread must not panic")
    }

    #[test]
    fn marker_found_for_test_fn_and_test_data_and_nothing_else() {
        let names = on_big_stack(|| {
            test_marked_decl_names(TWO_MARKED_TWO_PLAIN, "fixtures/marker_fixture.dag")
        })
        .expect("the fixture parses");
        assert_eq!(
            names,
            vec![
                "marked_fn_holds".to_string(),
                "marked_data_fixture".to_string()
            ]
        );
    }

    #[test]
    fn marker_is_read_off_the_declaration_never_the_file_name() {
        // The retired convention keyed on the `_test.dag` suffix. The same source under a
        // `_test.dag` spelling and under a plain spelling reads identically, and a plain
        // source spelled `_test.dag` declares no test at all.
        let (suffixed, plain, no_marker) = on_big_stack(|| {
            (
                test_marked_decl_names(TWO_MARKED_TWO_PLAIN, "fixtures/a_test.dag"),
                test_marked_decl_names(TWO_MARKED_TWO_PLAIN, "fixtures/a.dag"),
                test_marked_decl_names(
                    "module no_marker\n\nfn only_plain() -> Bool { true }\n",
                    "fixtures/no_marker_test.dag",
                ),
            )
        });
        assert_eq!(suffixed.expect("parses"), plain.expect("parses"));
        assert_eq!(no_marker.expect("parses"), Vec::<String>::new());
    }

    #[test]
    fn decl_extent_covers_the_whole_declaration() {
        let decls =
            on_big_stack(|| test_marked_decls(TWO_MARKED_TWO_PLAIN, "fixtures/extent_fixture.dag"))
                .expect("the fixture parses");
        let fn_decl = decls
            .iter()
            .find(|d| d.name == "marked_fn_holds")
            .expect("marked fn present");
        let text = &TWO_MARKED_TWO_PLAIN[fn_decl.span_start as usize..fn_decl.span_end as usize];
        assert!(
            text.contains("{ true }"),
            "the extent must reach the end of the body: {text:?}"
        );
        assert!(
            !text.contains("marked_data_fixture"),
            "the extent must not run into the next declaration: {text:?}"
        );
    }

    #[test]
    fn parse_failure_refuses_and_never_falls_back_to_a_text_scan() {
        // An unterminated body: a line scanner would still find `test fn marked_fn_holds`
        // here. The marker reading refuses, naming the file.
        let source = "module broken\n\ntest fn marked_fn_holds() -> Bool {\n  true\n";
        let err = on_big_stack(move || test_marked_decl_names(source, "fixtures/broken.dag"))
            .expect_err("an unparseable source has no determinable test set");
        assert!(
            err.contains("fixtures/broken.dag"),
            "must name the file: {err}"
        );
        assert!(
            err.contains("cannot be read"),
            "must say what could not be read: {err}"
        );
    }
}
