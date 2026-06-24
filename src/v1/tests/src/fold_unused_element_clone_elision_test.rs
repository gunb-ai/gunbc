//! E0277: a generic `fold` whose lambda ignores its element parameter (`(acc, _) => ...`)
//! emitted `.iter().cloned().fold(..)`. The spurious `.cloned()` forces a `T: Clone` bound on
//! the generic element that the emitter never synthesizes → `the trait bound `T: Clone` is not
//! satisfied`. Canonical corpus site: `fn list_length<T>(items: List<T>) -> Int`.
//!
//! Root cause: the Rust `iter_owned` sharing template is unconditionally `{0}.iter().cloned()`,
//! cloning every element into the fold. When the closure binds the element as `_` it provably
//! cannot depend on owned-vs-borrowed, so `.cloned()` is dead weight that only adds the bound.
//! Fix: elide `.cloned()` from the template when the fold lambda's element parameter is `_`.
//!
//! Conservative gate (construction-safe): a NAMED-but-unused element does NOT fire the elision
//! (keeps a harmless clone), so this never produces a wrong borrow. The discriminating control
//! below is a fold that USES its element by name — it must KEEP `.cloned()`.

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module fold.fixture\n\n",
    "fn count_all<T>(items: List<T>) -> Int {\n",
    "  fold(items, init: 0, f: (acc, _) => acc + 1)\n",
    "}\n\n",
    "fn sum_all(items: List<Int>) -> Int {\n",
    "  fold(items, init: 0, f: (acc, x) => acc + x)\n",
    "}\n"
);

fn emit_host() -> String {
    compile_dag_named(
        "src/v1/fold_unused_element_fixture.dag",
        FIXTURE,
        RenderTarget::Rust,
    )
    .files
    .iter()
    .map(|f| f.content.clone())
    .collect::<Vec<_>>()
    .join("\n")
}

/// The emitted body line containing `.fold(` for the named function `fn NAME`.
fn fold_line(emitted: &str, fn_name: &str) -> String {
    let needle = format!("fn {fn_name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("{fn_name} not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let fold_off = rest
        .find(".fold(")
        .unwrap_or_else(|| panic!("{fn_name} has no .fold( call:\n{rest}"));
    // Walk back to the start of the receiver expression on this line.
    let line_start = rest[..fold_off].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = rest[fold_off..]
        .find('\n')
        .map(|i| fold_off + i)
        .unwrap_or(rest.len());
    rest[line_start..line_end].to_string()
}

#[test]
fn fold_with_unused_element_elides_cloned() {
    let emitted = emit_host();
    // `(acc, _) =>` ignores the element → no `.cloned()`, so no synthesized `T: Clone` bound.
    let unused = fold_line(&emitted, "count_all");
    assert!(
        !unused.contains(".cloned()"),
        "fold over an unused (`_`) element must NOT emit `.cloned()` (forces spurious T: Clone):\n{unused}"
    );
    assert!(
        unused.contains(".fold("),
        "expected a fold call for count_all:\n{unused}"
    );
}

#[test]
fn fold_with_named_element_keeps_cloned() {
    let emitted = emit_host();
    // Control: `(acc, x) =>` consumes the element by name → `.cloned()` MUST remain (the gate
    // is the underscore subset, never a wrong elision on a used element).
    let used = fold_line(&emitted, "sum_all");
    assert!(
        used.contains(".cloned()"),
        "fold over a named element must KEEP `.cloned()` (gate is the `_` subset only):\n{used}"
    );
}
