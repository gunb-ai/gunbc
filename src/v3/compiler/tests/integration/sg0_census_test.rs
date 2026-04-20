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
//!   remove the file, reduce it to a narrow host shim (see `compiler.dag`
//!   for the shim rule), or (last resort) extend `EXPECTED_HAND_AUTHORED`
//!   with director sign-off.
//! - **missing expected file**: an SG lane retired the file. Remove
//!   the entry from `EXPECTED_HAND_AUTHORED` — this is the normal
//!   shrinkage path and the primary success condition for SG-1..SG-7.
//!
//! **Producer-owned partition.** A `.rs` file counts as generated iff
//! its workspace-relative path is a member of `GENERATED_FILES`, which
//! is emitted by `src/v3/compiler/build.rs` on every build from the
//! reviewed `REGEN_OUTPUTS` literal. Every codegen driver (the
//! `regen_*` binaries plus the ignored `emit_lens_provenance_snapshot`
//! test) imports the same manifest and asserts its output path is in
//! the list before writing — so a new generated file can only land if
//! `build.rs` names it. File contents do not participate: a hand-authored
//! `.rs` that begins with `// AUTO-GENERATED` does not slip through.

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
// SG-6 landing (PR #560): the four per-lens regen bins
// (`regen_lens_cost.rs`, `regen_lens_cost_symbolic.rs`,
// `regen_lens_structural_resolution.rs`, `regen_lens_unused_parameters.rs`)
// and SG-4 prep's `regen_infer_helpers.rs` all folded into a single
// `regen_lens.rs` shim driven by `src/v3/compiler/regen.dag`'s
// `LensRegistryEntry` records. Five retirements; one net-new entry
// (`regen_lens.rs`). The new `sg6_hand_authored_census_test.rs`
// pins the reduced bin census + full `(name, lens_file,
// generated_file)` registry tuples + `--lens` singleton resolve +
// end-to-end CLI smoke; it is hand-authored test infrastructure and
// belongs on this list.
//
// SG-6 follow-up landing (director sign-off from the
// `clever-swift-141` brief, 2026-04-19): the former standalone
// `sg4_prep_infer_helpers_freshness_test.rs` was absorbed into
// `sg6_hand_authored_census_test.rs`, so the infer-helpers snapshot
// gate now resolves through the same `LensRegistryEntry` authority as
// the unified `regen_lens` driver. One more hand-authored test file
// retired; no standalone per-helper freshness harness remains.
//
// Phase 0 test-taxonomy reorg — four target-emission tests moved from
// `tests/integration/` to `tests/boundary/` (TESTING.md § test layers,
// class-5 rustc/go/python roundtrips). Path rename only: net count
// unchanged, no new hand-authored files. The consolidated
// `tests/integration.rs` binary still includes them via `#[path =
// "boundary/..."]` so the one-bootstrap compile amortization holds.
//
// P0-A (PR #595): bounded `repeat_string_loop` receipt — one integration
// file `tests/integration/p0_std_render_repeat_string_test.rs` asserts
// `dsl/std/render.dag` structure; not generated. Dissolution: fold into a
// broader std-render harness or `.dag`-native structural test when one exists.
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
//
// SG-3f-prep (director Option B): `lower.rs` stays on this list — canonical
// lowering remains hand-maintained Rust until `lower.dag` + reflected `Surface*`.
// `regen_lower` + `lower_generated.rs` are prep-only (not `lib.rs` authority).
//
// Phase 1 Dag builder surface — PR #570 adds one narrow host-side
// helper file, `src/dag/builder.rs`, to keep the test-facing graph
// constructors scoped away from the main `dag.rs` body while the
// direct-Dag migration replaces `compile_to_dag(source)` fixtures.
// Dissolution trigger: once the migration wave settles, fold the
// builder back into `dag.rs` or move the surface behind a
// producer-owned path. This is a bounded migration exception, not a
// precedent for free-standing handwritten helpers.
//
// Post-2026-04-20 merge wave (`cleanup-post-merge-slop` brief): PR #589
// retired `parse.rs` from this `.rs` census, but the 1350-line
// recursive-descent parse algorithm migrated to
// `src/v3/compiler/parse_parser_body.txt` — a scaffold fragment
// `include_str!`'d into the `regen_parse` output. Ratchet extension
// below (`EXPECTED_HAND_AUTHORED_FRAGMENTS` + `sg0_v3_hand_authored_txt_fragments`)
// counts non-`.rs` scaffolds so the net measurement matches reality.
// Dissolution trigger: same as the header on `parse_parser_body.txt`
// (SG-2b proper / SG-3f surface reflection follow-on).
//
// L4b split — `dag.rs` was a 2800-line god-file mixing ports, nodes,
// declarations, clusters, and the std.effects mirror. The split carves
// two leaf clusters into sibling submodules (`dag/ports.rs`,
// `dag/effects.rs`) that the module root re-exports
// verbatim. No behavior change; file count goes up but per-file
// coupling goes down. These are pure re-organization of already
// hand-authored substrate, not new handwritten logic. Dissolution path:
// the same `include!` / producer-owned route that eventually replaces
// `dag.rs` itself replaces these submodules simultaneously.
const EXPECTED_HAND_AUTHORED: &[&str] = &[
    "src/v3/compiler/build.rs",
    "src/v3/compiler/src/bin/regen_lens.rs",
    "src/v3/compiler/src/bin/regen_lower.rs",
    "src/v3/compiler/src/bin/regen_parse.rs",
    "src/v3/compiler/src/bin/regen_tokenize.rs",
    "src/v3/compiler/src/bin/regen_v3.rs",
    "src/v3/compiler/src/bin/self_host_fixed_point.rs",
    "src/v3/compiler/src/bootstrap.rs",
    "src/v3/compiler/src/dag.rs",
    "src/v3/compiler/src/dag/builder.rs",
    "src/v3/compiler/src/dag/effects.rs",
    "src/v3/compiler/src/dag/ports.rs",
    "src/v3/compiler/src/diagnostics.rs",
    "src/v3/compiler/src/dimension.rs",
    "src/v3/compiler/src/emit.rs",
    "src/v3/compiler/src/emit/python_target.rs",
    "src/v3/compiler/src/emit/rust_target.rs",
    "src/v3/compiler/src/emit_rust.rs",
    "src/v3/compiler/src/infer.rs",
    "src/v3/compiler/src/lens_depth.rs",
    "src/v3/compiler/src/lens_idempotency.rs",
    "src/v3/compiler/src/lens_parallelism.rs",
    "src/v3/compiler/src/lens_testgen.rs",
    "src/v3/compiler/src/lens_unused_parameters.rs",
    "src/v3/compiler/src/lib.rs",
    "src/v3/compiler/src/lower.rs",
    "src/v3/compiler/src/pipeline_authority.rs",
    "src/v3/compiler/src/post_emit_verifier.rs",
    "src/v3/compiler/src/regen_parse_emit.rs",
    "src/v3/compiler/src/tokenize.rs",
    "src/v3/compiler/src/workflow_idempotency.rs",
    "src/v3/compiler/src/workflow_parallelism.rs",
    "src/v3/compiler/tests/boundary/m1_3_emit_go_test.rs",
    "src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs",
    "src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs",
    "src/v3/compiler/tests/boundary/m2_emit_multi_field_struct_variant_test.rs",
    "src/v3/compiler/tests/determinism_test.rs",
    "src/v3/compiler/tests/integration.rs",
    "src/v3/compiler/tests/integration/common/budgeted.rs",
    "src/v3/compiler/tests/integration/common/cached_compile.rs",
    "src/v3/compiler/tests/integration/common/determinism_fixtures.rs",
    "src/v3/compiler/tests/integration/common/mod.rs",
    "src/v3/compiler/tests/integration/common/substrate_receipts.rs",
    "src/v3/compiler/tests/integration/four_fixture_regression_test.rs",
    "src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2a_effects_smoke.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2b_db18_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2c_db15_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2d_symbolic_cost_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2e_parallelism_test.rs",
    "src/v3/compiler/tests/integration/lane3_stage_3b_db1_test.rs",
    "src/v3/compiler/tests/integration/m0_acceptance.rs",
    "src/v3/compiler/tests/integration/m1_3_lens_cost_test.rs",
    "src/v3/compiler/tests/integration/m1_3_lens_unused_parameters_test.rs",
    "src/v3/compiler/tests/integration/m1_5_testgen_test.rs",
    "src/v3/compiler/tests/integration/m1_5_verification_test.rs",
    "src/v3/compiler/tests/integration/m1_fn_external_body_reconciliation_test.rs",
    "src/v3/compiler/tests/integration/m1_lens_structural_resolution_test.rs",
    "src/v3/compiler/tests/integration/m1_substrate_test.rs",
    "src/v3/compiler/tests/integration/m2_feature_parity_test.rs",
    "src/v3/compiler/tests/integration/m2_field_access_binding_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_cost_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_idempotency_emit_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_idempotency_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_provenance_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_structural_resolution_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_unused_parameters_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_lens_variant_payload_migration_test.rs",
    "src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs",
    "src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs",
    "src/v3/compiler/tests/integration/pipe_desugar.rs",
    "src/v3/compiler/tests/integration/sg0_census_test.rs",
    "src/v3/compiler/tests/integration/sg1_tokenize_authority_test.rs",
    "src/v3/compiler/tests/integration/sg2_parse_authority_test.rs",
    "src/v3/compiler/tests/integration/sg3_lower_authority_test.rs",
    "src/v3/compiler/tests/integration/sg3_lower_parse_surface_stack_test.rs",
    "src/v3/compiler/tests/integration/sg6_hand_authored_census_test.rs",
    "src/v3/compiler/tests/integration/sg7_prep_variant_payload_freshness_test.rs",
    "src/v3/compiler/tests/integration/thesis_parallelism_test.rs",
    "src/v3/compiler/tests/integration/thesis_validation_test.rs",
];

// Non-`.rs` scaffold fragments under `src/v3/compiler/` that are
// hand-authored and text-inlined into generated Rust (or otherwise
// dissolve when the corresponding `.dag` authority lands). The
// `.rs`-only census above cannot see these — a scaffold that renames
// itself `foo.txt` would silently escape the ratchet otherwise.
// Every entry here names a dissolution trigger in its own file header.
// Sorted; one path per line, relative to the workspace root.
const EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &[
    "src/v3/compiler/parse_parser_body.txt",
];

// Non-`.rs` files under `src/v3/compiler/` whose content is produced
// by a named generator (an `#[ignore]`'d refresh test, a `regen_*`
// binary, etc.) rather than hand-edited. Listed explicitly so the
// fragments walker can partition without content sniffing (which the
// `sg0_generated_partition_is_producer_owned` probe forbids for
// `.rs`; the same discipline applies here).
const EXPECTED_GENERATED_FRAGMENTS: &[&str] = &[
    // Produced by `cargo test refresh_handwritten_parse_snapshot_manifest -- --ignored`.
    "src/v3/compiler/tests/integration/parse_corpus_manifest.txt",
];

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at src/v3/compiler/. ancestors():
    //   [0] src/v3/compiler
    //   [1] src/v3
    //   [2] src
    //   [3] workspace root
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
            // `target/` holds Cargo build output — never part of the
            // census. Skip regardless of whether it lives at the
            // workspace root (default) or inside the crate (custom
            // CARGO_TARGET_DIR or local configurations).
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

fn walk_txt(root: &Path, ws: &Path, out: &mut BTreeSet<String>) {
    let entries = fs::read_dir(root).unwrap_or_else(|e| panic!("read_dir {}: {e}", root.display()));
    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.is_dir() {
            if path.file_name() == Some(OsStr::new("target")) {
                continue;
            }
            walk_txt(&path, ws, out);
        } else if path.extension() == Some(OsStr::new("txt")) {
            let rel = path
                .strip_prefix(ws)
                .expect("census walk stays inside workspace")
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(rel);
        }
    }
}

fn compiler_dag_source() -> String {
    let path = workspace_root().join("dsl/gunbc/compiler.dag");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
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
         classified as hand-authored (the path is not in GENERATED_FILES). \
         The manifest is the sole authority; content headers must not \
         participate. If this fails, content-based classification has \
         regressed into walk_rs or the partition logic."
    );
}

#[test]
fn sg0_every_generated_file_is_present_on_disk() {
    // The manifest is the authority for "which files are generated";
    // it must stay in lockstep with what producers actually write.
    // If a manifest entry points at a missing file, either the
    // producer hasn't run yet (fresh checkout + `cargo test` without
    // regen) or the file was deleted without updating build.rs.
    // Either way the census would silently shrink and this ratchet
    // would be unsound — fail loud here.
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
         the producer (a regen_* binary or build.rs entry) did not write them. \
         Update REGEN_OUTPUTS in src/v3/compiler/build.rs, or run the relevant \
         regen driver to populate the committed output."
    );
}

#[test]
fn sg0_stage0_copy_command_excludes_hand_maintained_root_files() {
    let source = compiler_dag_source();
    let start = source
        .find("fn copy_generated_command(")
        .expect("compiler.dag should define copy_generated_command");
    let tail = &source[start..];
    let end = tail
        .find("\n}\n\n// Diff exclusion args")
        .expect("copy_generated_command should end before diff_exclude_args");
    let copy_fn = &tail[..end];

    assert!(
        copy_fn.contains("cycle.generated.hand_maintained_src |> fold"),
        "copy_generated_command should derive an exclude filter from hand_maintained_src"
    );
    assert!(
        copy_fn.contains(" -maxdepth 1 -type f -name '*.rs'"),
        "copy_generated_command should only walk top-level emitted Rust files"
    );
    assert!(
        copy_fn.contains(" -exec "),
        "copy_generated_command should copy via find -exec so excluded names are not overwritten"
    );
    assert!(
        copy_fn.contains("cycle.generated.source_dir"),
        "copy_generated_command should target the declared generated source_dir"
    );
}

#[test]
fn sg0_stage0_hand_maintained_src_covers_emit_subtree_companions() {
    let source = compiler_dag_source();
    let start = source
        .find("hand_maintained_src: [")
        .expect("compiler.dag should declare hand_maintained_src");
    let tail = &source[start..];
    let end = tail
        .find("\n  ]")
        .expect("hand_maintained_src list should terminate");
    let list = &tail[..end];

    assert!(
        list.contains("\"python_target.rs\""),
        "hand_maintained_src should exclude emit/python_target.rs from recursive freshness drift"
    );
    assert!(
        list.contains("\"rust_target.rs\""),
        "hand_maintained_src should exclude emit/rust_target.rs from recursive freshness drift"
    );
}
