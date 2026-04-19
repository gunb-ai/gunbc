//! SG-0 — v3 Rust authority census + ratchet.
//!
//! Enumerates every `.rs` file under `src/v3/compiler` and partitions
//! it into **generated** (listed in the producer-owned manifest at
//! [`v3_compiler::generated_files::GENERATED_FILES`]) versus
//! **hand-authored** (everything else). The hand-authored set is
//! compared against [`EXPECTED_HAND_AUTHORED`] below — the ratchet.
//! Drift in either direction fails:
//!
//! - **new hand-authored file**: a contributor added a `.rs` without
//!   porting the logic to `.dag`. The PR should port the logic and
<<<<<<< HEAD
//!   remove the file, reduce it to a narrow host shim (see `compiler.dag`
//!   for the shim rule), or (last resort) extend `EXPECTED_HAND_AUTHORED`
//!   with director sign-off.
=======
//!   remove the file, reduce it to a narrow host shim, or (last
//!   resort) extend `EXPECTED_HAND_AUTHORED` with director sign-off.
>>>>>>> 2bc514d78 (SG-0: track generated files via manifest)
//! - **missing expected file**: an SG lane retired the file. Remove
//!   the entry from `EXPECTED_HAND_AUTHORED` — this is the normal
//!   shrinkage path and the primary success condition for SG-1..SG-7.
//!
//! **Producer-owned partition.** A `.rs` file counts as generated iff
//! its workspace-relative path is a member of `GENERATED_FILES`, which
//! is emitted by `src/v3/compiler/build.rs` on every build from the
<<<<<<< HEAD
//! reviewed `REGEN_OUTPUTS` literal. Every codegen driver (the
//! `regen_*` binaries plus the ignored `emit_lens_provenance_snapshot`
//! test) imports the same manifest and asserts its output path is in
//! the list before writing — so a new generated file can only land if
//! `build.rs` names it. File contents do not participate: a hand-authored
//! `.rs` that begins with `// AUTO-GENERATED` does not slip through.
=======
//! reviewed `REGEN_OUTPUTS` literal. File contents do not participate:
//! a hand-authored `.rs` that begins with `// AUTO-GENERATED` does not
//! slip through.
>>>>>>> 2bc514d78 (SG-0: track generated files via manifest)

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use v3_compiler::generated_files::GENERATED_FILES;

// Relative to workspace root; mirrors the single census root
// informally named in `dsl/gunbc/compiler.dag`.
const CENSUS_ROOT: &str = "src/v3/compiler";

// All .rs files under `src/v3/compiler` that are currently
// hand-authored. Sorted; one path per line, relative to the
// workspace root. **Every SG-1..SG-7 PR shortens this list.**
// Removing an entry means the owning lane has retired the file;
// adding an entry is forbidden outside SG-0 without director
// sign-off.
//
// SG-4 prep carries two entries that SG-6 will retire together:
//   - `bin/regen_infer_helpers.rs` (the per-helper regen binary)
//   - `tests/integration/sg4_prep_infer_helpers_freshness_test.rs`
//     (the per-helper regenerate→diff-empty ratchet)
// When SG-6 folds all regen drivers and snapshot gates into a
// single generic target, both entries collapse alongside the four
// `regen_lens_*` lines above. Director sign-off
// (clever-swift-141 brief, 2026-04-19) covers the temporary +2.
//
// Stage 3b DB-1 parse/apply ratchet bump — PR #564 adds one
// hand-authored integration file,
// `tests/integration/lane3_stage_3b_db1_test.rs`, because the
// ratchet is intentionally end-to-end over real compiler fixtures
// (diagnose -> apply correction -> reparse -> recompile), not a
// generated lens snapshot or unit-only helper. Dissolution trigger:
// when this slice is absorbed into a generic correction harness or a
// `.dag`-native correction-validation path, drop the entry. This is
// a bounded SG-0 exception for the merge-blocking Stage 3b receipt,
// not a precedent for adding ad hoc integration files.
const EXPECTED_HAND_AUTHORED: &[&str] = &[
    "src/v3/compiler/build.rs",
    "src/v3/compiler/src/bin/regen_infer_helpers.rs",
    "src/v3/compiler/src/bin/regen_lens_cost.rs",
    "src/v3/compiler/src/bin/regen_lens_cost_symbolic.rs",
    "src/v3/compiler/src/bin/regen_lens_structural_resolution.rs",
    "src/v3/compiler/src/bin/regen_lens_unused_parameters.rs",
    "src/v3/compiler/src/bin/regen_v3.rs",
    "src/v3/compiler/src/bin/self_host_fixed_point.rs",
    "src/v3/compiler/src/bootstrap.rs",
    "src/v3/compiler/src/dag.rs",
    "src/v3/compiler/src/diagnostics.rs",
    "src/v3/compiler/src/dimension.rs",
    "src/v3/compiler/src/emit.rs",
    "src/v3/compiler/src/emit/python_target.rs",
    "src/v3/compiler/src/emit/rust_target.rs",
    "src/v3/compiler/src/emit_go.rs",
    "src/v3/compiler/src/emit_python.rs",
    "src/v3/compiler/src/emit_rust.rs",
    "src/v3/compiler/src/infer.rs",
    "src/v3/compiler/src/lens_depth.rs",
    "src/v3/compiler/src/lens_idempotency.rs",
    "src/v3/compiler/src/lens_parallelism.rs",
    "src/v3/compiler/src/lens_testgen.rs",
    "src/v3/compiler/src/lens_unused_parameters.rs",
    "src/v3/compiler/src/lib.rs",
    "src/v3/compiler/src/lower.rs",
    "src/v3/compiler/src/parse.rs",
    "src/v3/compiler/src/pipeline_authority.rs",
    "src/v3/compiler/src/post_emit_verifier.rs",
    "src/v3/compiler/src/serialize.rs",
    "src/v3/compiler/src/tokenize.rs",
    "src/v3/compiler/src/types.rs",
    "src/v3/compiler/src/variant_payload.rs",
    "src/v3/compiler/src/workflow_idempotency.rs",
    "src/v3/compiler/src/workflow_parallelism.rs",
    "src/v3/compiler/tests/determinism_test.rs",
    "src/v3/compiler/tests/integration.rs",
    "src/v3/compiler/tests/integration/common/budgeted.rs",
    "src/v3/compiler/tests/integration/common/cached_compile.rs",
    "src/v3/compiler/tests/integration/common/determinism_fixtures.rs",
    "src/v3/compiler/tests/integration/common/mod.rs",
    "src/v3/compiler/tests/integration/four_fixture_regression_test.rs",
    "src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2a_effects_smoke.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2b_db18_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2c_db15_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2d_symbolic_cost_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2e_parallelism_test.rs",
    "src/v3/compiler/tests/integration/lane3_stage_3b_db1_test.rs",
    "src/v3/compiler/tests/integration/m0_acceptance.rs",
    "src/v3/compiler/tests/integration/m1_3_emit_go_test.rs",
    "src/v3/compiler/tests/integration/m1_3_emit_rust_test.rs",
    "src/v3/compiler/tests/integration/m1_3_lens_cost_test.rs",
    "src/v3/compiler/tests/integration/m1_3_lens_unused_parameters_test.rs",
    "src/v3/compiler/tests/integration/m1_4_emit_python_test.rs",
    "src/v3/compiler/tests/integration/m1_5_testgen_test.rs",
    "src/v3/compiler/tests/integration/m1_5_verification_test.rs",
    "src/v3/compiler/tests/integration/m1_fn_external_body_reconciliation_test.rs",
    "src/v3/compiler/tests/integration/m1_lens_structural_resolution_test.rs",
    "src/v3/compiler/tests/integration/m1_substrate_test.rs",
    "src/v3/compiler/tests/integration/m2_emit_multi_field_struct_variant_test.rs",
    "src/v3/compiler/tests/integration/m2_feature_parity_test.rs",
    "src/v3/compiler/tests/integration/m2_field_access_binding_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_cost_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_idempotency_emit_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_idempotency_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_provenance_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_structural_resolution_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_unused_parameters_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs",
    "src/v3/compiler/tests/integration/pipe_desugar.rs",
    "src/v3/compiler/tests/integration/real_stdlib_parse_smoke.rs",
    "src/v3/compiler/tests/integration/sg0_census_test.rs",
    "src/v3/compiler/tests/integration/sg4_prep_infer_helpers_freshness_test.rs",
    "src/v3/compiler/tests/integration/thesis_parallelism_test.rs",
    "src/v3/compiler/tests/integration/thesis_validation_test.rs",
    "src/v3/compiler/tests/lane2_stage_2f_dimension_test.rs",
];

fn workspace_root() -> PathBuf {
<<<<<<< HEAD
    // CARGO_MANIFEST_DIR points at src/v3/compiler/. ancestors():
    //   [0] src/v3/compiler
    //   [1] src/v3
    //   [2] src
    //   [3] workspace root
=======
>>>>>>> 2bc514d78 (SG-0: track generated files via manifest)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .expect("workspace root is three ancestors above src/v3/compiler/")
        .to_path_buf()
}

fn walk_rs(root: &Path, ws: &Path, out: &mut BTreeSet<String>) {
    let entries = fs::read_dir(root).unwrap_or_else(|e| panic!("read_dir {}: {e}", root.display()));
    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.is_dir() {
<<<<<<< HEAD
            // `target/` holds Cargo build output — never part of the
            // census. Skip regardless of whether it lives at the
            // workspace root (default) or inside the crate (custom
            // CARGO_TARGET_DIR or local configurations).
=======
>>>>>>> 2bc514d78 (SG-0: track generated files via manifest)
            if path.file_name() == Some(OsStr::new("target")) {
                continue;
            }
            walk_rs(&path, ws, out);
        } else if path.extension() == Some(OsStr::new("rs")) {
            let rel = path
                .strip_prefix(ws)
                .expect("census walk stays inside workspace")
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(rel);
        }
    }
}

#[test]
fn sg0_v3_hand_authored_census() {
    let ws = workspace_root();
    let census_root = ws.join(CENSUS_ROOT);
    assert!(
        census_root.is_dir(),
        "v3 census root must exist at {}",
        census_root.display()
    );

    let mut all_rs: BTreeSet<String> = BTreeSet::new();
    walk_rs(&census_root, &ws, &mut all_rs);

    let generated: BTreeSet<String> = GENERATED_FILES.iter().map(|p| (*p).to_string()).collect();
    let hand_authored: BTreeSet<String> = all_rs.difference(&generated).cloned().collect();

    let expected: BTreeSet<String> = EXPECTED_HAND_AUTHORED
        .iter()
        .map(|p| (*p).to_string())
        .collect();

    if hand_authored == expected {
        return;
    }

    let added: Vec<&str> = hand_authored
        .difference(&expected)
        .map(String::as_str)
        .collect();
    let removed: Vec<&str> = expected
        .difference(&hand_authored)
        .map(String::as_str)
        .collect();

    let mut msg = String::from(
        "SG-0 census drift: observed hand-authored set does not match EXPECTED_HAND_AUTHORED.\n\n",
    );
    if !added.is_empty() {
        msg.push_str("New hand-authored .rs files (the SG program forbids adding these):\n");
        for p in &added {
            msg.push_str("  + ");
            msg.push_str(p);
            msg.push('\n');
        }
        msg.push_str(
            "\nFix: port the logic to .dag and remove the .rs, or reduce the file\n\
             to a narrow host shim and add its path to REGEN_OUTPUTS in\n\
             src/v3/compiler/build.rs (the shim must be written by a producer —\n\
             a hand-authored `// AUTO-GENERATED` header does not count).\n\
             Last resort: add the path to EXPECTED_HAND_AUTHORED with a\n\
             director-approved receipt.\n\n",
        );
    }
    if !removed.is_empty() {
        msg.push_str("Retired .rs files (great — an SG reduction!):\n");
        for p in &removed {
            msg.push_str("  - ");
            msg.push_str(p);
            msg.push('\n');
        }
        msg.push_str(
            "\nFix: remove these entries from EXPECTED_HAND_AUTHORED in\n\
             src/v3/compiler/tests/integration/sg0_census_test.rs.\n",
        );
    }
    panic!("{msg}");
}

#[test]
fn sg0_expected_list_is_sorted_and_unique() {
    let mut prev: Option<&str> = None;
    for p in EXPECTED_HAND_AUTHORED {
        if let Some(pv) = prev {
            assert!(
                pv < *p,
                "EXPECTED_HAND_AUTHORED must be sorted ASCII-ascending and unique; \
                 `{pv}` is not strictly less than `{p}`"
            );
        }
        prev = Some(p);
    }
}

static PROBE_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn sg0_generated_partition_is_producer_owned() {
<<<<<<< HEAD
    // Soundness: the manifest is the sole authority for the
    // generated/hand-authored partition. This test plants a
    // handwritten file whose first non-blank line is the most
    // convincing possible spoof — the exact string
    // `// AUTO-GENERATED from ...` that every real regen driver
    // emits — into an isolated temp tree, then runs the real
    // `walk_rs` + manifest-membership classification over it.
    // Because the probe's path is not in `GENERATED_FILES`, it must
    // land in the hand-authored set. If a future change reintroduces
    // content-based filtering (either inside `walk_rs` or in the
    // partition step), the probe would wrongly classify as generated
    // and this assertion flips — catching the regression loudly.
    //
    // Isolation: the probe lives under `std::env::temp_dir()`, not
    // inside `src/v3/compiler/`. The live `sg0_v3_hand_authored_census`
    // walker never sees it, so the tests are safe to run in parallel.

=======
>>>>>>> 2bc514d78 (SG-0: track generated files via manifest)
    let tmp = std::env::temp_dir().join(format!(
        "sg0_soundness_probe_{}_{}",
        std::process::id(),
        PROBE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _guard = TempDirGuard(tmp.clone());
    fs::create_dir_all(&tmp).unwrap_or_else(|e| panic!("create probe dir: {e}"));

    let probe_path = tmp.join("spoofed_header.rs");
    fs::write(
        &probe_path,
        "// AUTO-GENERATED from `src/v3/lenses/fake.dag` via `emit_rust_module`.\n\
         // Hand-authored file masquerading as generated — must still\n\
         // classify as hand-authored because the manifest is authority.\n\
         pub fn spoof() {}\n",
    )
    .unwrap_or_else(|e| panic!("write probe: {e}"));

    let mut walk_results: BTreeSet<String> = BTreeSet::new();
    walk_rs(&tmp, &tmp, &mut walk_results);

    let generated: BTreeSet<String> = GENERATED_FILES.iter().map(|p| (*p).to_string()).collect();
    let hand_authored: BTreeSet<String> = walk_results.difference(&generated).cloned().collect();

    let probe_rel = probe_path
        .strip_prefix(&tmp)
        .expect("probe is inside tmp")
        .to_string_lossy()
        .replace('\\', "/");

    assert!(
        hand_authored.contains(&probe_rel),
        "probe file with `// AUTO-GENERATED` first-line header must be \
<<<<<<< HEAD
         classified as hand-authored (the path is not in GENERATED_FILES). \
         The manifest is the sole authority; content headers must not \
         participate. If this fails, content-based classification has \
         regressed into walk_rs or the partition logic."
=======
         classified as hand-authored (the path is not in GENERATED_FILES)."
>>>>>>> 2bc514d78 (SG-0: track generated files via manifest)
    );
}

#[test]
fn sg0_every_generated_file_is_present_on_disk() {
<<<<<<< HEAD
    // The manifest is the authority for "which files are generated";
    // it must stay in lockstep with what producers actually write.
    // If a manifest entry points at a missing file, either the
    // producer hasn't run yet (fresh checkout + `cargo test` without
    // regen) or the file was deleted without updating build.rs.
    // Either way the census would silently shrink and this ratchet
    // would be unsound — fail loud here.
=======
>>>>>>> 2bc514d78 (SG-0: track generated files via manifest)
    let ws = workspace_root();
    let mut missing: Vec<&str> = Vec::new();
    for rel in GENERATED_FILES {
        if !ws.join(rel).is_file() {
            missing.push(rel);
        }
    }
    assert!(
        missing.is_empty(),
        "GENERATED_FILES references paths not present on disk: {missing:?} — \
<<<<<<< HEAD
         the producer (a regen_* binary or build.rs entry) did not write them. \
         Update REGEN_OUTPUTS in src/v3/compiler/build.rs, or run the relevant \
=======
         update REGEN_OUTPUTS in src/v3/compiler/build.rs, or run the relevant \
>>>>>>> 2bc514d78 (SG-0: track generated files via manifest)
         regen driver to populate the committed output."
    );
}
