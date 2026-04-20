//! **Layer:** integration
//!
//! P0-A receipt: `dsl/std/render.dag` must repeat strings using a bounded counter,
//! not a singleton `fold` (regression for indent_text and other callers).

use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root (dsl/ should exist)")
        .to_path_buf()
}

#[test]
fn std_render_repeat_string_is_bounded_loop_not_singleton_fold() {
    let root = repo_root();
    assert!(
        root.join("dsl/std/render.dag").is_file(),
        "expected dsl/std/render.dag at {}",
        root.display()
    );
    let render =
        std::fs::read_to_string(root.join("dsl/std/render.dag")).expect("read dsl/std/render.dag");
    assert!(
        render.contains("repeat_string_loop"),
        "render.dag should define repeat_string_loop (bounded repetition)"
    );
    assert!(
        render.contains("remaining - 1"),
        "expected arithmetic descent on `remaining` in repeat_string_loop"
    );
    assert!(
        !render.contains("[0] |> fold(init:"),
        "singleton-fold bug pattern must not reappear in repeat_string (was: one copy regardless of n)"
    );
}
