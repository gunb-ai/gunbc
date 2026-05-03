//! V2 consumability ratchet for the **R3 row 85 / PB #1560 Gap 4 build-step
//! producer** (`v3_compiler::pb_method_template_projection_dag_emit`).
//!
//! The v2-tests crate cannot depend on v3-compiler (the boundary forbids
//! it), so this test fabricates a `.dag` matching the producer's output
//! **structural shape** — module declaration, per-target `Map<String,
//! String>` declaration names, and one spot-check entry per target — and
//! proves v2's import / compile pipeline consumes it via the ephemeral
//! source-root mechanism from PR #1575. The fixture is **not byte-
//! identical** to `render_dag`'s output: the header comment wording is
//! intentionally distinct so reviewers do not mistake this fixture for
//! the producer's authoritative emission. Byte-equivalent end-to-end
//! coverage lives in PR #1575's `stage0_compile_imports_ephemeral_generated_source_root`
//! (`#[ignore]`d, slow path).
//!
//! Together with the v3-side producer integration tests
//! (`pb_method_template_projection_dag_emit_test`, which assert the
//! producer writes this exact shape) and PR #1575's existing
//! `resolver_imports_ephemeral_generated_source_root` ratchet (which
//! covers ephemeral source-root resolution for arbitrary `.dag`), this
//! closes the loop: v3 producer ↔ v2 consumer agree on the
//! `generated.method_template_projection` shape without committing the
//! generated file or importing `v3.std.*` into v2.
//!
//! What this exercises:
//!
//! 1. A `.dag` carrying `data rust_method_template_emit: Map<String, String>`
//!    at the canonical relative path (`generated/method_template_projection.dag`)
//!    inside an ephemeral source-root resolves cleanly through v2's
//!    `compile_dag_named_with_source_roots`.
//! 2. The generated module is loaded **only** from the ephemeral root —
//!    not from any `src/` or `dsl/` path. Asserts the no-committed-source
//!    invariant via the resolver's loaded-paths receipt.

use crate::helpers::{assert_no_diagnostics, compile_dag_named_with_source_roots};
use v2_compiler::v2_compiler_artifact::RenderTarget;

fn temp_dir(label: &str) -> std::path::PathBuf {
    let unique = format!(
        "v2-pb-method-template-consumability-{label}-{ns}",
        ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    );
    std::env::temp_dir().join(unique)
}

#[test]
fn v2_consumes_generated_method_template_projection_shape() {
    let generated_root = temp_dir("generated-root");
    let generated_dir = generated_root.join("generated");
    std::fs::create_dir_all(&generated_dir).expect("create generated dir");

    // Structural shape parity (not byte identity) with
    // `write_method_template_projection_dag`'s output: module
    // declaration, three `data <target>_method_template_emit:
    // Map<String, String>` declarations, and one `count` spot-check
    // entry per target. The header text intentionally differs so this
    // fixture cannot be confused with producer-authoritative bytes; the
    // v3-side producer tests own header-and-content invariants over the
    // real emission, this v2-side ratchet owns "v2 imports this shape."
    let generated_dag = "\
// AUTO-GENERATED — do not commit. Produced by v3-compiler's
// `pb_method_template_projection_dag_emit`. Single row authority:
// `src/v3/std/{rust,python,go}_method_template_contracts.dag`.

module generated.method_template_projection

data rust_method_template_emit: Map<String, String> = {
  \"count\": \"(\\{recv\\}.len() as i64)\",
}

data python_method_template_emit: Map<String, String> = {
  \"count\": \"len(\\{recv\\})\",
}

data go_method_template_emit: Map<String, String> = {
  \"count\": \"len(\\{recv\\})\",
}
";
    std::fs::write(
        generated_dir.join("method_template_projection.dag"),
        generated_dag,
    )
    .expect("write generated module");

    // Entry imports one of the per-target maps. v2's import resolution
    // walks the ephemeral source-root and finds the generated module.
    let entry_source = "\
module ephemeral.entry

import generated.method_template_projection { rust_method_template_emit }

fn rust_count_template() -> Map<String, String> { rust_method_template_emit }
";
    let result = compile_dag_named_with_source_roots(
        "ephemeral/entry.dag",
        entry_source,
        RenderTarget::Dag,
        std::slice::from_ref(&generated_root),
    );

    assert_no_diagnostics(&result);

    let loaded_paths: Vec<_> = result
        .newline_indices
        .iter()
        .map(|index| index.file.as_str())
        .collect();
    assert!(
        loaded_paths
            .iter()
            .any(|path| path.contains("generated/method_template_projection.dag")),
        "expected the ephemeral generated module to be loaded; got: {loaded_paths:?}"
    );
    // The "do not commit" invariant: the generated file must be loaded
    // from the temp source-root, not from anywhere under `src/` or `dsl/`.
    assert!(
        !loaded_paths
            .iter()
            .any(|path| path.starts_with("src/") || path.starts_with("dsl/")),
        "ephemeral generated dependency must not be tracked under src/ or dsl/: {loaded_paths:?}"
    );

    let _ = std::fs::remove_dir_all(&generated_root);
}
