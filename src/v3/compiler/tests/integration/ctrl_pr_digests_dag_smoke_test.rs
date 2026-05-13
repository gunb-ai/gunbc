//! **Layer:** integration
//!
//! Parser + typecheck smoke for Wave-1 catalog #8 `dsl/ctrl/pr_digests.dag`
//! (imports `extdeps.github.pulls` + `std.*` only; no new extdeps source facts).

use std::path::PathBuf;

use v3_compiler::compile_to_dag;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("expected src/v3/compiler -> workspace root")
        .to_path_buf()
}

#[test]
fn ctrl_pr_digests_dag_compiles_cleanly() {
    let path = workspace_root().join("dsl/ctrl/pr_digests.dag");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    compile_to_dag(&source, "dsl/ctrl/pr_digests.dag").unwrap_or_else(|err| {
        panic!("dsl/ctrl/pr_digests.dag should parse+lower+infer cleanly: {err:?}");
    });
}
