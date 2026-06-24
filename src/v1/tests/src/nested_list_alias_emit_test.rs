//! E0107: a type alias whose RHS is a NESTED container (`List<List<T>>`) dropped the inner
//! type argument when emitted. `type Schedule = List<List<Runnable>>` emitted
//! `pub type Schedule = Vec<Vec>;` — the inner `Vec` argless → `E0107: missing generics for Vec`.
//!
//! Root cause: after resolve, each container child of the alias RHS is a WRAPPER node with
//! empty `children` and the real (nested) type in `.inferred = Resolved{..}`. The general
//! type renderer (`render_node_type`) peels that wrapper via `child_type_node`, but
//! `render_rust_alias_rhs_type` recursed on the raw `arg` directly, so a child whose resolved
//! type is itself a container rendered as a bare base (`Vec`) with no `<arg>`.
//!
//! Faithful fix (emitter-faithfulness, no model change): peel each alias-RHS arg through its
//! `Resolved` inferred node when that node is itself a container — the inner args are already
//! present in `.inferred`, the renderer just wasn't reading them.

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module nestlist.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Item {\n  x: Nat\n}\n\n",
    "type Grid = List<List<Item>>\n\n",
    "type Row = List<Item>\n\n",
    "fn use_grid(g: Grid) -> Grid {\n  g\n}\n"
);

fn emit_host() -> String {
    compile_dag_named("src/v1/nested_list_alias_fixture.dag", FIXTURE, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The single emitted line `pub type NAME = ...;`.
fn alias_line(emitted: &str, name: &str) -> String {
    let needle = format!("type {name} =");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("alias `{name}` not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest.find('\n').unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn nested_list_alias_keeps_inner_type_arg() {
    let emitted = emit_host();
    let grid = alias_line(&emitted, "Grid");
    // The inner container must carry its element type: `Vec<Vec<Item>>`, never a bare `Vec<Vec>`.
    assert!(
        grid.contains("Vec<Vec<Item>>"),
        "nested-container alias must keep the inner type arg, got:\n{grid}"
    );
    assert!(
        !grid.contains("Vec<Vec>") && !grid.contains("Vec<Vec,") && !grid.contains("Vec<Vec ")
            && !grid.contains("Vec<Vec;"),
        "alias RHS dropped the inner `Vec` type arg (E0107), got:\n{grid}"
    );
}

#[test]
fn single_level_container_alias_unchanged_control() {
    // Control: a non-nested container alias already worked and must stay `Vec<Item>` — the fix
    // peels only when the arg's resolved type is itself a container, so leaves this untouched.
    let emitted = emit_host();
    let row = alias_line(&emitted, "Row");
    assert!(
        row.contains("Vec<Item>"),
        "single-level container alias must stay `Vec<Item>`, got:\n{row}"
    );
}
