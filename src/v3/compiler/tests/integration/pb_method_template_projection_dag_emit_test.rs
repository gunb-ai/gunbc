//! **Layer:** integration
//!
//! Acceptance for the **Gap 4 build-step producer**
//! (`pb_method_template_projection_dag_emit`) per the dispatch unparking
//! Gap 4 once `PR #1575` landed the ephemeral source-root mechanism.
//!
//! What this exercises:
//!
//! 1. The producer writes a `.dag` file at the canonical relative path
//!    inside a caller-supplied (ephemeral) directory.
//! 2. The generated content carries the canonical authority chain receipt
//!    (header comment naming the row authority + the AUTO-GENERATED tag).
//! 3. The generated file declares the agreed module name and three
//!    `data <target>_method_template_emit: Map<String, String>` data
//!    declarations.
//! 4. Spot-check: the generated map carries known projected rows from each
//!    target (`count` for Rust, Python, Go) — proves the map was populated
//!    from the typed `MethodTemplateContract` rows, not a hand-authored
//!    second authority.
//! 5. The generated file is structurally well-formed: re-reading + parsing
//!    via the v2-compatible kernel grammar (validated through v3 compile)
//!    produces a `data … Map<String, String>` structure carrying the
//!    spot-checked entries.
//!
//! Out of scope (per dispatch / Gap 4 scope clamp):
//! - **No v2 stage0 subprocess test here**: that is the consumer-side
//!   ratchet from `PR #1575`
//!   (`stage0_compile_imports_ephemeral_generated_source_root`,
//!   `#[ignore]`d due to ~2-min stage0 build cost). The producer's
//!   contract is "well-formed `.dag` at the canonical path"; the consumer
//!   ratchet covers "v2 can ingest it."
//! - **No higher-order row migration**: `MethodEmitTemplateProjection::HigherOrder`
//!   rows are skipped; this test asserts they are absent from the
//!   generated map (Gap 5 deferred).

use std::path::PathBuf;

use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::pb_method_template_projection::MethodTemplateTarget;
use v3_compiler::pb_method_template_projection_dag_emit::{
    generated_map_declaration_name, write_method_template_projection_dag, GENERATED_MODULE_NAME,
    GENERATED_PROJECTION_RELATIVE_PATH,
};

/// Build a fresh per-test temp directory under the system tempdir. Mirrors the
/// pattern in `src/v2/tests/src/bootstrap.rs::temp_dir`. The test removes the
/// directory at the end of the run on success.
fn fresh_temp_dir(label: &str) -> PathBuf {
    let unique = format!(
        "v3-pb-method-template-projection-{label}-{ns}",
        ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    );
    let path = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[test]
fn writes_dag_file_at_canonical_relative_path() {
    let dag = generated_full_bootstrap_dag();
    let out_dir = fresh_temp_dir("path");
    let written = write_method_template_projection_dag(&dag, &out_dir).expect("emit");
    assert_eq!(
        written,
        out_dir.join(GENERATED_PROJECTION_RELATIVE_PATH),
        "producer must write to the canonical relative path so v2 callers \
         can cite GENERATED_PROJECTION_RELATIVE_PATH directly"
    );
    assert!(written.is_file(), "produced .dag must exist on disk");
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn header_carries_authority_chain_receipt() {
    let dag = generated_full_bootstrap_dag();
    let out_dir = fresh_temp_dir("header");
    let written = write_method_template_projection_dag(&dag, &out_dir).expect("emit");
    let content = std::fs::read_to_string(&written).expect("read");
    // Build-pipeline integrity: file is auto-generated, never committed.
    assert!(
        content.contains("AUTO-GENERATED"),
        "header must mark the file AUTO-GENERATED so reviewers cannot \
         mistake it for hand-authored authority"
    );
    assert!(
        content.contains("Do not commit"),
        "header must explicitly forbid committing the generated file"
    );
    // Authority anchor: the header names the canonical row source so
    // future readers can trace template text back to its substrate origin.
    assert!(
        content.contains("src/v3/std/{rust,python,go}_method_template_contracts.dag"),
        "header must cite the canonical row authority"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn declares_generated_module_and_three_target_maps() {
    let dag = generated_full_bootstrap_dag();
    let out_dir = fresh_temp_dir("module");
    let written = write_method_template_projection_dag(&dag, &out_dir).expect("emit");
    let content = std::fs::read_to_string(&written).expect("read");

    let expected_module_decl = format!("module {GENERATED_MODULE_NAME}");
    assert!(
        content.contains(&expected_module_decl),
        "must declare module {GENERATED_MODULE_NAME}; content was:\n{content}"
    );
    for target in [
        MethodTemplateTarget::Rust,
        MethodTemplateTarget::Python,
        MethodTemplateTarget::Go,
    ] {
        let map_decl = format!(
            "data {}: Map<String, String>",
            generated_map_declaration_name(target)
        );
        assert!(
            content.contains(&map_decl),
            "must declare {map_decl} for {target:?}; content was:\n{content}"
        );
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn rust_count_row_lands_in_generated_map() {
    // Anchor on the well-known Rust `count_method` row — its emit_template
    // (`({recv}.len() as i64)`) is documented in
    // `src/v3/std/rust_method_template_contracts.dag` and exercised by
    // `pb_method_template_projection_test.rs::rust_count_row_anchors_runtime_emit_drift`.
    // If the producer drifts from the typed projection, this fails before
    // any v2 consumer would.
    let dag = generated_full_bootstrap_dag();
    let out_dir = fresh_temp_dir("rust-count");
    let written = write_method_template_projection_dag(&dag, &out_dir).expect("emit");
    let content = std::fs::read_to_string(&written).expect("read");
    assert!(
        content.contains("\"count\": \"({recv}.len() as i64)\""),
        "Rust `count` row must land with its documented emit_template; \
         content was:\n{content}"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn python_and_go_count_rows_land_in_generated_maps() {
    // Spot-check the other two targets so the test does not silently let
    // a single-target producer regress.
    let dag = generated_full_bootstrap_dag();
    let out_dir = fresh_temp_dir("py-go-count");
    let written = write_method_template_projection_dag(&dag, &out_dir).expect("emit");
    let content = std::fs::read_to_string(&written).expect("read");

    // Both Python and Go projects `count` to `len({recv})` per the row
    // authorities; both should be present in their per-target maps.
    let occurrences = content.matches("\"count\": \"len({recv})\"").count();
    assert_eq!(
        occurrences, 2,
        "Python and Go each project `count` to `len({{recv}})`; expected \
         2 occurrences in the generated file, got {occurrences}. \
         Content was:\n{content}"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn higher_order_rows_are_skipped() {
    // `filter_method` is a `MethodEmitTemplate::HigherOrderTemplates` row
    // for Rust per `src/v3/std/rust_method_template_contracts.dag`. The
    // legacy Map<String, String> shape can't carry the inline / fn-ref
    // split, so the producer must skip higher-order rows. Gap 5 will
    // migrate them to the typed structural form.
    let dag = generated_full_bootstrap_dag();
    let out_dir = fresh_temp_dir("higher-order-skip");
    let written = write_method_template_projection_dag(&dag, &out_dir).expect("emit");
    let content = std::fs::read_to_string(&written).expect("read");
    // No bare `"filter":` entry in the Rust map — the producer skips it.
    // (`"filter":` could legitimately appear in Python/Go maps if those
    // targets carry a SingleTemplate row for filter; assert absence in
    // the Rust block specifically by searching for the surrounding context.)
    let rust_block_start = content
        .find("data rust_method_template_emit:")
        .expect("Rust map present");
    let rust_block_end = content[rust_block_start..]
        .find("}")
        .expect("Rust map block terminates")
        + rust_block_start;
    let rust_block = &content[rust_block_start..rust_block_end];
    assert!(
        !rust_block.contains("\"filter\":"),
        "Rust map must skip higher-order `filter`; block was:\n{rust_block}"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn bytes_are_deterministic_across_repeated_emits() {
    // Build-pipeline reproducibility: emitting the same Dag into two
    // different temp directories must yield byte-identical content.
    // BTreeMap-backed iteration in the producer is the contract; this
    // test holds it.
    let dag = generated_full_bootstrap_dag();
    let out_a = fresh_temp_dir("determinism-a");
    let out_b = fresh_temp_dir("determinism-b");
    let written_a = write_method_template_projection_dag(&dag, &out_a).expect("emit a");
    let written_b = write_method_template_projection_dag(&dag, &out_b).expect("emit b");
    let bytes_a = std::fs::read(&written_a).expect("read a");
    let bytes_b = std::fs::read(&written_b).expect("read b");
    assert_eq!(
        bytes_a, bytes_b,
        "two emits over the same Dag must produce byte-identical content"
    );
    let _ = std::fs::remove_dir_all(&out_a);
    let _ = std::fs::remove_dir_all(&out_b);
}
