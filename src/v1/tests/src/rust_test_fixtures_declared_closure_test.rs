//! Closure-only control for `v2.extdeps.languages.rust_test`'s import of
//! `v2.extdeps.languages.rust` (PR #7811, the Class B un-strip).
//!
//! Before #7811, `rust_test_fixtures.dag` carried zero import statements and its bare
//! references to `rust.dag` symbols resolved only by pool-membership coincidence: some
//! unrelated file elsewhere in a whole-tree compile happened to import `rust.dag` first,
//! dragging it into the assembled pool. #7811 adds the declared import block. What that
//! PR does not prove on its own is that resolution now depends on the declared edge
//! rather than on whatever else is compiled alongside it — a whole-tree CI run already
//! greened before the fix, so a green whole-tree run after the fix proves nothing new
//! (DESIGN §5, "specification-without-execution").
//!
//! This test compiles `rust_test_fixtures.dag` alone, closed over strictly its own
//! declared-import transitive closure (`resolve_imports_transitively_with_source_roots`
//! walks only the entry's own `import` statements — no other file is ever present in the
//! compiled set, so nothing can supply `rust.dag` by coincidence). That makes it cheap: a
//! two-or-three-module closure, not a whole-tree execution.
//!
//! Two arms:
//!   - green: the real, current file (with the import) compiles clean in isolation.
//!   - RED control: the same file with the import block mechanically stripped, compiled
//!     in the same isolated closure (no other file present), fails to resolve the
//!     `rust.dag` symbols it still references bare. This is the discriminating arm: it
//!     is what proves the green arm is about the declared edge, not about incidental
//!     compile-set composition.

use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::compile_sources;

use crate::helpers::{
    diagnostic_messages, read_v2_file, resolve_imports_transitively_with_source_roots,
    v2_layer_roots,
};

const ENTRY_PATH: &str = "src/v2/extdeps/languages/rust_test_fixtures.dag";
const IMPORT_HEADER: &str = "import v2.extdeps.languages.rust {";

/// A symbol that #7811's import block names and that the fixture file references bare;
/// used as the located witness that resolution failed for the right reason.
const WITNESS_SYMBOL: &str = "rust_grammar_terminal";

fn compile_entry_alone(content: &str) -> Rc<v1_compiler::v1_compiler_compile::PipelineResult> {
    let sources =
        resolve_imports_transitively_with_source_roots(ENTRY_PATH, content, &v2_layer_roots());
    compile_sources(Rc::new(sources.into()), RenderTarget::Rust)
}

/// Mechanically removes the `import v2.extdeps.languages.rust { ... }` block by brace
/// counting, leaving every other line (including the fixture's bare references to the
/// symbols that block named) untouched.
fn strip_rust_import_block(content: &str) -> String {
    let start = content
        .find(IMPORT_HEADER)
        .expect("fixture must currently declare the v2.extdeps.languages.rust import");
    let after_header = start + IMPORT_HEADER.len();
    let mut depth = 1usize;
    let mut end = after_header;
    for (offset, ch) in content[after_header..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = after_header + offset + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(end > after_header, "unterminated import block in fixture");
    let mut stripped = String::new();
    stripped.push_str(&content[..start]);
    stripped.push_str(&content[end..]);
    stripped
}

#[test]
fn declared_import_block_is_present_and_strippable() {
    let content = read_v2_file(ENTRY_PATH);
    assert!(
        content.contains(IMPORT_HEADER),
        "src/v2/extdeps/languages/rust_test_fixtures.dag has no declared import of \
         v2.extdeps.languages.rust — expected PR #7811 (Class B un-strip) to have \
         landed; this control depends on that import existing"
    );
    let stripped = strip_rust_import_block(&content);
    assert!(
        !stripped.contains(IMPORT_HEADER),
        "strip_rust_import_block left the import header in place"
    );
    assert!(
        stripped.contains(WITNESS_SYMBOL),
        "strip_rust_import_block must not touch the fixture's own bare reference to {}",
        WITNESS_SYMBOL
    );
}

#[test]
fn declared_closure_alone_resolves_rust_symbols() {
    let content = read_v2_file(ENTRY_PATH);
    let result = compile_entry_alone(&content);
    let messages = diagnostic_messages(&result);
    let hard: Vec<&String> = messages
        .iter()
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        hard.is_empty(),
        "rust_test_fixtures.dag failed to resolve from its own declared import closure \
         alone (entry + transitively-imported modules only, no unrelated file present): \
         {hard:?}"
    );
}

#[test]
fn without_the_declared_import_the_isolated_closure_refuses() {
    let content = read_v2_file(ENTRY_PATH);
    let stripped = strip_rust_import_block(&content);

    let result = compile_entry_alone(&stripped);
    let messages = diagnostic_messages(&result);

    assert!(
        messages.iter().any(|m| m.contains(WITNESS_SYMBOL)),
        "RED control: with the import block removed and no other file present to \
         coincidentally supply v2.extdeps.languages.rust, resolving {} bare must fail — \
         the declared closure has nothing to bind it to. Got: {messages:?}",
        WITNESS_SYMBOL
    );
}
