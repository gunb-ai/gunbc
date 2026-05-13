//! SG-0 — v3 Rust authority census + ratchet.
//!
//! Enumerates every `.rs` file under `src/v3/compiler` and partitions
//! it into **generated** (listed in the producer-owned manifest at
//! [`v3_compiler::generated_files::GENERATED_FILES`]) versus
//! **hand-authored** (everything else). The hand-authored set is
//! compared against [`EXPECTED_HAND_AUTHORED_NON_TEST`] plus
//! [`EXPECTED_HAND_AUTHORED_TEST`] below — the ratchet. The split is
//! load-bearing: T-PB-A owns the non-test subset, while T-PB-B owns
//! the test subset.
//! Drift in either direction fails:
//!
//! - **new hand-authored file**: a contributor added a `.rs` without
//!   porting the logic to `.dag`. The PR should port the logic and
//!   remove the file, reduce it to a narrow host shim (see `compiler.dag`
//!   for the shim rule), or (last resort) extend the matching
//!   `EXPECTED_HAND_AUTHORED_*` sub-ratchet with director sign-off.
//! - **missing expected file**: an SG lane retired the file. Remove
//!   the entry from its expected sub-ratchet — this is the normal
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

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use v3_compiler::generated_files::{
    GENERATED_FILES, RUST_TEST_GENERATOR_MANIFEST, RUST_TEST_GENERATOR_MANIFEST_BYTES,
};

// Relative to workspace root; mirrors the single census root
// informally named in `dsl/gunbc/compiler.dag`.
const CENSUS_ROOT: &str = "src/v3/compiler";
const RETIRED_LENS_TESTGEN_RS: &str = "src/v3/compiler/src/lens_testgen.rs";
const RETIRED_LENS_APPLY_RS: &str = "src/v3/compiler/src/lens_apply.rs";

#[test]
fn r3_gate_5_lens_apply_rs_stays_retired() {
    let retired_path = workspace_root().join(RETIRED_LENS_APPLY_RS);

    assert!(
        !retired_path.exists(),
        "R3 gate #5 (`lens_apply_dot_rs_retired`) requires \
         `{RETIRED_LENS_APPLY_RS}` to stay retired. Bounded lens application \
         (`apply_lens_declaration`, reflection helpers) lives in \
         `lens_declaration_apply.rs` until PB-Runtime owns the surface end-to-end."
    );
}

#[test]
fn r3_gate_6_lens_testgen_rs_stays_retired() {
    let retired_path = workspace_root().join(RETIRED_LENS_TESTGEN_RS);

    assert!(
        !retired_path.exists(),
        "R3 gate #6 (`lens_testgen_dot_rs_retired`) requires \
         `{RETIRED_LENS_TESTGEN_RS}` to stay retired. Keep the stable \
         `v3_compiler::lens_testgen` API routed through `lens_declaration_apply.rs` \
         until PB-Runtime owns testgen end-to-end."
    );
}

#[test]
fn emit_production_code_has_no_declaration_by_name_calls() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let emit_root = manifest_dir.join("src").join("emit");
    let mut files = vec![manifest_dir.join("src").join("emit.rs")];
    for entry in fs::read_dir(&emit_root).expect("read src/emit") {
        let path = entry.expect("emit dir entry").path();
        if path.extension() == Some(OsStr::new("rs")) {
            files.push(path);
        }
    }
    files.sort();

    let mut offenders = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path).expect("read emit source");
        let production_source = source
            .split("\n#[cfg(test)]")
            .next()
            .expect("split always yields a prefix");
        if production_source.contains(".declaration_by_name(") {
            offenders.push(
                path.strip_prefix(manifest_dir)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
            );
        }
    }

    assert!(
        offenders.is_empty(),
        "emit production modules must use cached DeclarationId accessors, not Dag::declaration_by_name. Offenders: {offenders:#?}"
    );
}

// All non-test .rs files under `src/v3/compiler` that are currently
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
// P0-A / R1C-B: `tests/integration/p0_std_render_repeat_string_test.rs` hosts the
// v3 `TestRunner` gate suite `p0_repeat_string_correct_gate` (live v2 oracle retired).
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
// T-PB-A-B: `lower.rs` stays on this list — canonical lowering remains
// hand-maintained Rust until `lower.dag` + reflected `Surface*`. The former
// `regen_lower` pass-through scaffold and non-authoritative
// `lower_generated.rs` snapshot were retired so the census no longer preserves
// a generated-looking lower projection that `lib.rs` did not consume.
//
// Lane E-I Step 0 (PR #726): `e_i_lane_induction_preflight_test.rs` is a
// bounded bootstrap receipt that `SumBound`'s `terms` field instantiates
// `List<CostBound>` (not label-only) and that `sum_bound` exists after
// `regen_bootstrap`. Not a generated
// snapshot. Dissolution trigger: the same structural fact is covered by a
// `.dag`-native or testgen-only harness without needing this host-side probe.
//
// T-PB-A Slice 1: `lens_unused_parameters.rs` folded into inline
// `pub mod lens_unused_parameters` in `lib.rs` (peer to `lens_cost`); one
// non-test census line retired.
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
// Note on census authority scope: the file is NOT added to
// `compiler.dag::stage0.hand_maintained_src` — that list models
// basenames inside `source_dir` (`src/v3/compiler/src/`) whose
// consumers are the freshness-diff and stage0-copy commands. The
// parser body fragment lives at the crate root, so the existing
// consumers never see it; adding it would be a dead entry. The
// SG-0 fragment ratchet below is the sole census authority for
// crate-root scaffolds.
//
// SG-3f-d consumption proof (director review on PR #605, 2026-04-20):
// `sg3_surface_reflection_consumer_test.rs` is a bounded host-side
// rustc harness proving reflected `Surface*` carriers are consumable
// from `.dag`, emitted against `parse_surface`, and executable against
// real parser output. It is intentionally not modeled as a generated
// snapshot because the receipt is behavioral end-to-end linkage.
// Dissolution trigger: when the same proof lands through a
// producer-owned/generated path, retire this hand-authored harness and
// drop its census entry.
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
//
// T-Substrate cardinality subset for int literals (2026-04-25): the
// range-comparison shim in `int_literal_ranges.rs` is host-side
// reconciliation glue over already-declared String-decimal range facts
// while `rust_pilot_primitives.value_body` remains an unparsed top-level
// list. It intentionally compares only source literals that already fit
// `LiteralBits::Int(i64)`; the declared u64 upper half is not reachable
// until the deferred carrier-widening lane replaces that source-literal
// carrier. Dissolution triggers: R2 T-Substrate's top-level aggregate
// `ValueBody` sub-lane makes `rust_pilot_primitives` row values
// structurally walkable, and the carrier-widening lane makes the full
// declared unsigned range parseable by source literals. At that point
// this helper should consume those declared rows directly or move behind
// generated substrate accessors.
//
// R3 T-FixedPoint P0 / DB-8: `self_host_receipt_p0.rs` is intentionally hand-authored
// receipt-key surface (stable JSON field names for `self_host_fixed_point` trend reads),
// not generated output. Dissolution: fold into a `.dag` or generated authority when one
// owns receipt schema; until then this module + census line are the bounded ratchet receipt.
const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[
    // R3 C1 perf-budget bench skeleton: Phase-1 Criterion harness for
    // `tier3_mirror_dissolution_perf_within_budget` per
    // `docs/briefs/r3-pb-tier3-perf-budget-worker.md` deliverable 0b
    // (parent brief #1331; readiness matrix #1358; this PR #1362).
    // Intentionally hand-authored: it measures live public substrate
    // entrypoints (`merge_evidence`, `positive_descent_count`,
    // `lower_call_pattern`, `type_iteration_dimension`,
    // `lane2_workflow_idempotency_report`). R3 gate #4 **parallel module**
    // `workflow_idempotency.rs` retired; native projection co-located in
    // `dag/effects.rs` (full evaluator/emitted-authority slice still open while
    // the std arrow is `Unparsed` in bootstrap). These benches still
    // target hot Rust call paths (Criterion). Broader Tier3 bench retirement
    // deletes this harness per parent brief §"Phase 1 deliverables" once the
    // remaining mirrors dissolve; only the frozen `tier3_baseline.json` data
    // survives.
    "src/v3/compiler/benches/tier3_mirror_perf.rs",
    "src/v3/compiler/build.rs",
    "src/v3/compiler/src/bin/r1c_e_emit_gates.rs",
    "src/v3/compiler/src/bin/regen_bootstrap.rs",
    "src/v3/compiler/src/bin/regen_lens.rs",
    "src/v3/compiler/src/bin/regen_parse.rs",
    "src/v3/compiler/src/bin/regen_parse_tables.rs",
    "src/v3/compiler/src/bin/regen_tokenize.rs",
    "src/v3/compiler/src/bin/regen_v3.rs",
    "src/v3/compiler/src/bin/self_host_fixed_point.rs",
    "src/v3/compiler/src/bootstrap.rs",
    "src/v3/compiler/src/bootstrap_regen_fresh.rs",
    // R3 gate #87 / T-Tests-As-Data-Completeness: `CementingDispatchMatchesProjection` host
    // evaluator for `tests/dag/cementing_dispatch.dag` (P5 consumer receipt; dissolves when
    // predicate substrate owns the walk without host FS coupling).
    "src/v3/compiler/src/cementing_dispatch.rs",
    "src/v3/compiler/src/complexity_lattice.rs",
    "src/v3/compiler/src/cost_basis_declaration.rs",
    "src/v3/compiler/src/dag.rs",
    "src/v3/compiler/src/dag/builder.rs",
    // Closed Cardinality payload + idempotent target shim (API closure).
    "src/v3/compiler/src/dag/cardinality_payload.rs",
    "src/v3/compiler/src/dag/effects.rs",
    "src/v3/compiler/src/dag/ports.rs",
    "src/v3/compiler/src/diagnostics.rs",
    "src/v3/compiler/src/dimension.rs",
    "src/v3/compiler/src/emit.rs",
    // CollectionOps `*_contract` → `MethodTemplateContract` identity gate (PR #1577 / #1602).
    "src/v3/compiler/src/emit/collection_ops_method_contract.rs",
    "src/v3/compiler/src/emit/python_target.rs",
    "src/v3/compiler/src/emit/rust_target.rs",
    "src/v3/compiler/src/emit_rust.rs",
    "src/v3/compiler/src/emit_rust_bin_shim.rs",
    // R1C-E + m1_3: shared `PROGRAM_FIXTURES` / `REFLECTED_FIXTURES` tables (single source of truth).
    "src/v3/compiler/src/emit_rust_roundtrip_fixtures.rs",
    "src/v3/compiler/src/enforced_lens_application.rs",
    // T-WAD Slice 7 / gate #103: pure `CIWorkflowDag` gate-id selection (P5 receipt
    // row in INVARIANTS.md §SG-0 hand-authored compiler non-test paths).
    "src/v3/compiler/src/gunbc_ci.rs",
    "src/v3/compiler/src/infer.rs",
    "src/v3/compiler/src/int_literal_ranges.rs",
    // R3 gate #87: `tests/integration.rs` wiring scanner shared by Band-C cementing dispatch
    // (`cementing_dispatch.rs`) and integration tests (P5 receipt for host promotion from
    // `tests/integration/common/mod.rs`).
    "src/v3/compiler/src/integration_rs_wiring_scan.rs",
    "src/v3/compiler/src/lens_declaration_apply.rs",
    "src/v3/compiler/src/lens_t_las_carrier.rs",
    "src/v3/compiler/src/lib.rs",
    "src/v3/compiler/src/lower.rs",
    // R3 gate #94: cost-lens memory-peak compose + enforcement authority (ties `dominant`/max_path).
    "src/v3/compiler/src/memory_peak_cost.rs",
    // R3 T-Omni-Shape-B Brief #1 (#2219 / PR #2251): transitional
    // Rust-side OpenAPI projection receipt after the Shape A/Shape B boundary
    // fix moved it out of `emit.rs`. Dissolves when the equivalent Shape B
    // `.dag` program owns the OpenAPI artifact projection end-to-end.
    "src/v3/compiler/src/omni_shape_b_openapi.rs",
    // R3 row 85 / PB #1560 Gap 4: target-keyed projection of the
    // `MethodTemplateContract` rows from the full bootstrap `Dag` for
    // PB-zero / v2-retirement consumers (decision in
    // `docs/decisions/r3-row85-method-template-read-surface.md`).
    "src/v3/compiler/src/pb_method_template_projection.rs",
    "src/v3/compiler/src/pipeline_authority.rs",
    "src/v3/compiler/src/post_emit_verifier.rs",
    // PB-1 Item 5: host mirror of `dsl/std/process.dag` `ProcessExit` for emitted bin shims.
    "src/v3/compiler/src/process_exit.rs",
    // R1C-E (T-Emit `.dag` `TestClaim` wrappers): shared `check_*` API the host
    // `#[test]` harness and `r1c_e_emit_gates` `bin` both call. Single source of
    // truth for the emit-gate assertions; scaffold until R1 close dissolves it.
    "src/v3/compiler/src/r1c_e_gates.rs",
    // R3 T-Free-Consequences: authored comment → `lane2_workflow` staging until lowering owns it.
    "src/v3/compiler/src/r3_fc_lane2_loop_witness.rs",
    // R3 gate #87: PB-B-1 runner table + `cementing_dispatch` shared inventory for
    // `tests/dag/t_r3_gate_87_cementing_regen_*.dag` (INVARIANTS P2 single authority).
    "src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs",
    "src/v3/compiler/src/regen_bootstrap_emit.rs",
    "src/v3/compiler/src/regen_parse_emit.rs",
    "src/v3/compiler/src/regen_parse_tables_emit.rs",
    "src/v3/compiler/src/regen_tokenize.rs",
    "src/v3/compiler/src/self_host_receipt_p0.rs",
    "src/v3/compiler/src/test_runner.rs",
];

// All test .rs files under `src/v3/compiler` that are currently
// hand-authored. Sorted; one path per line, relative to the
// workspace root. T-PB-B owns shrinking this subset toward the
// TESTING.md §"Post-R2 shape" residual. T-PB-A reductions must not
// rely on this list moving.
// Slice 1 census reconciliation (2026-05-02): sorted path list; update when
// adding/removing hand-authored integration tests (SG-0 ratchet).
//
// **Cementing-test discipline ratchet (gate #87 `lens_cementing_test_discipline_complete`).**
// New cementing receipts must follow `TESTING.md` §4 "One claim per test": one structural
// claim per `#[test]` / per `data foo: TestClaim`, and runner-drive tests assert
// `ClaimResult` by shape (`== ClaimResult::Pass` / `matches!(_, ClaimResult::Pass)`), never
// by stringified message contents. When porting a Rust receipt below to a `.dag` `TestClaim`,
// **the same PR removes the entry from this list** — `EXPECTED_HAND_AUTHORED_TEST` is the
// single cementing inventory. Don't introduce a parallel hand list (e.g. a separate
// "ported-but-still-listed" or "pending-port" set); the ratchet's whole point is that one
// monotonically-shrinking authority tracks the Rust→`.dag` migration.
const EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[];
// Non-`.rs` scaffold fragments under `src/v3/compiler/` that are
// hand-authored and text-inlined into generated Rust (or otherwise
// dissolve when the corresponding `.dag` authority lands). The
// `.rs`-only census above cannot see these — a scaffold that renames
// itself `foo.txt` would silently escape the ratchet otherwise.
// Every entry here names a dissolution trigger in its own file header.
// Sorted; one path per line, relative to the workspace root.
const EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &[
    "src/v3/compiler/parse_parser_body.txt",
    "src/v3/compiler/src/lens_testgen_body.txt",
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum TestsAsDataMigrationClass {
    CompileOrReject,
    LensOutputEquality,
    BehavioralObservation,
    BoundaryHostProcess,
    CementingV2Oracle,
    CensusOrRatchet,
    PropertyBased,
}

// Transitional gate #84 audit only. As each class migrates to `.dag`
// `TestClaim` data, remove that class's path matcher branch with the
// retired Rust paths; when `EXPECTED_HAND_AUTHORED_TEST` reaches zero,
// this classifier should disappear with it.
// Match order is load-bearing while this exists: check classification
// before deleting or reordering a branch, especially broad substring
// branches such as `contains("sg")`.
fn tests_as_data_migration_class(path: &str) -> Option<TestsAsDataMigrationClass> {
    use TestsAsDataMigrationClass::*;

    if path.starts_with("src/v3/compiler/tests/boundary/") {
        return Some(BoundaryHostProcess);
    }

    if path.contains("/cementing/")
        || path.ends_with("lens_behavioral_parity_demonstration_test.rs")
        || path.ends_with("r3_gate_87_lens_cementing_regen_receipts_test.rs")
    {
        return Some(CementingV2Oracle);
    }

    if path.contains("census")
        || path.contains("ratchet")
        || path.contains("bridge")
        || path.contains("sg")
        || path.contains("r1c_")
        || path.contains("v2_oracle")
        || path.contains("value_body_substrate_mirror")
        || path.contains("lens_producer_retirement")
    {
        return Some(CensusOrRatchet);
    }

    if path.contains("free_consequences")
        || path.contains("tc1_")
        || path.contains("tc2_")
        || path.contains("tc3_")
    {
        return Some(PropertyBased);
    }

    if path.contains("lens")
        || path.contains("cost")
        || path.contains("parallelism")
        || path.contains("timing")
        || path.contains("workflow")
        || path.contains("e6_g1a")
        || path.contains("lane2_stage_2d")
    {
        return Some(LensOutputEquality);
    }

    if path.contains("anthropic")
        || path.contains("operation")
        || path.contains("services")
        || path.contains("omni")
        || path.contains("openapi")
        || path.contains("runtime_evaluator_corpus")
        || path.contains("self_host_demonstration")
        || path.contains("t_ci_workflow_as_data_demo")
        || path.contains("pb1_bootstrap_full_snapshot")
    {
        return Some(BehavioralObservation);
    }

    if path.starts_with("src/v3/compiler/tests/integration/")
        || path.starts_with("src/v3/compiler/tests/determinism_test.rs")
        || path.starts_with("src/v3/compiler/tests/integration.rs")
    {
        return Some(CompileOrReject);
    }

    None
}

fn is_test_path(path: &str) -> bool {
    path.starts_with("src/v3/compiler/tests/")
}

fn expected_hand_authored_rs() -> BTreeSet<String> {
    EXPECTED_HAND_AUTHORED_NON_TEST
        .iter()
        .chain(EXPECTED_HAND_AUTHORED_TEST.iter())
        .map(|p| (*p).to_string())
        .collect()
}

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

    let expected = expected_hand_authored_rs();

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
        "SG-0 census drift: observed hand-authored set does not match \
         EXPECTED_HAND_AUTHORED_NON_TEST ∪ EXPECTED_HAND_AUTHORED_TEST.\n\n",
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
             Last resort: add the path to the matching EXPECTED_HAND_AUTHORED_* \
             sub-ratchet with a\n\
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
            "\nFix: remove these entries from the matching EXPECTED_HAND_AUTHORED_* \
             sub-ratchet in\n\
             src/v3/compiler/tests/integration/sg0_census_test.rs.\n",
        );
    }
    panic!("{msg}");
}

#[test]
fn sg0_expected_list_is_sorted_and_unique() {
    for (label, list) in [
        (
            "EXPECTED_HAND_AUTHORED_NON_TEST",
            EXPECTED_HAND_AUTHORED_NON_TEST,
        ),
        ("EXPECTED_HAND_AUTHORED_TEST", EXPECTED_HAND_AUTHORED_TEST),
    ] {
        let mut prev: Option<&str> = None;
        for p in list {
            if let Some(pv) = prev {
                assert!(
                    pv < *p,
                    "{label} must be sorted ASCII-ascending and unique; \
                     `{pv}` is not strictly less than `{p}`"
                );
            }
            prev = Some(p);
        }
    }
    let non_test: BTreeSet<&str> = EXPECTED_HAND_AUTHORED_NON_TEST.iter().copied().collect();
    let test: BTreeSet<&str> = EXPECTED_HAND_AUTHORED_TEST.iter().copied().collect();
    let overlap: Vec<&&str> = non_test.intersection(&test).collect();
    assert!(
        overlap.is_empty(),
        "EXPECTED_HAND_AUTHORED_NON_TEST and EXPECTED_HAND_AUTHORED_TEST must be disjoint; \
         overlap: {overlap:?}"
    );
}

#[test]
fn sg0_expected_rs_entries_match_test_partition() {
    let misplaced_non_test: Vec<&str> = EXPECTED_HAND_AUTHORED_NON_TEST
        .iter()
        .copied()
        .filter(|p| is_test_path(p))
        .collect();
    assert!(
        misplaced_non_test.is_empty(),
        "T-PB-A non-test ratchet must not include test paths; move these to \
         EXPECTED_HAND_AUTHORED_TEST: {misplaced_non_test:?}"
    );

    let misplaced_test: Vec<&str> = EXPECTED_HAND_AUTHORED_TEST
        .iter()
        .copied()
        .filter(|p| !is_test_path(p))
        .collect();
    assert!(
        misplaced_test.is_empty(),
        "T-PB-B test ratchet must only include paths under src/v3/compiler/tests/; \
         move these to EXPECTED_HAND_AUTHORED_NON_TEST: {misplaced_test:?}"
    );
}

#[test]
fn sg0_v3_non_test_hand_authored_subratchet() {
    let ws = workspace_root();
    let census_root = ws.join(CENSUS_ROOT);

    let mut all_rs: BTreeSet<String> = BTreeSet::new();
    walk_rs(&census_root, &ws, &mut all_rs);

    let generated: BTreeSet<String> = GENERATED_FILES.iter().map(|p| (*p).to_string()).collect();
    let hand_authored: BTreeSet<String> = all_rs.difference(&generated).cloned().collect();
    let observed: BTreeSet<String> = hand_authored
        .iter()
        .filter(|p| !is_test_path(p))
        .cloned()
        .collect();
    let expected: BTreeSet<String> = EXPECTED_HAND_AUTHORED_NON_TEST
        .iter()
        .map(|p| (*p).to_string())
        .collect();

    assert_eq!(
        observed, expected,
        "T-PB-A non-test SG-0 sub-ratchet drifted. Retirements should be removed \
         from EXPECTED_HAND_AUTHORED_NON_TEST; new non-test hand-Rust needs director \
         sign-off."
    );
}

#[test]
fn sg0_v3_test_hand_authored_subratchet() {
    let ws = workspace_root();
    let census_root = ws.join(CENSUS_ROOT);

    let mut all_rs: BTreeSet<String> = BTreeSet::new();
    walk_rs(&census_root, &ws, &mut all_rs);

    let generated: BTreeSet<String> = GENERATED_FILES.iter().map(|p| (*p).to_string()).collect();
    let hand_authored: BTreeSet<String> = all_rs.difference(&generated).cloned().collect();
    let observed: BTreeSet<String> = hand_authored
        .iter()
        .filter(|p| is_test_path(p))
        .cloned()
        .collect();
    let expected: BTreeSet<String> = EXPECTED_HAND_AUTHORED_TEST
        .iter()
        .map(|p| (*p).to_string())
        .collect();

    assert_eq!(
        observed, expected,
        "T-PB-B test SG-0 sub-ratchet drifted. Retirements should be removed from \
         EXPECTED_HAND_AUTHORED_TEST; new Rust-authored tests must match the TESTING.md \
         residual or wait for the testgen path."
    );
}

#[test]
fn sg0_generated_rust_test_manifest_covers_every_test_rs() {
    let ws = workspace_root();
    let tests_root = ws.join("src/v3/compiler/tests");

    let mut all_rs: BTreeSet<String> = BTreeSet::new();
    walk_rs(&tests_root, &ws, &mut all_rs);

    let manifest: BTreeSet<String> = GENERATED_RUST_TEST_FILES
        .iter()
        .map(|p| (*p).to_string())
        .collect();

    assert_eq!(
        all_rs, manifest,
        "gate #84 positive authority drift: every Rust test file must be present in \
         GENERATED_RUST_TEST_FILES. Missing entries are orphaned tests; extra entries are stale \
         generator-manifest rows."
    );

    let generated: BTreeSet<&str> = GENERATED_FILES.iter().copied().collect();
    let missing_from_partition: Vec<&str> = GENERATED_RUST_TEST_FILES
        .iter()
        .copied()
        .filter(|path| !generated.contains(path))
        .collect();
    assert!(
        missing_from_partition.is_empty(),
        "every generated Rust test manifest row must also be in GENERATED_FILES; missing: \
         {missing_from_partition:?}"
    );
}

#[test]
fn sg0_generated_rust_test_manifest_bytes_match_checked_in_files() {
    let ws = workspace_root();
    let manifest_paths: BTreeSet<&str> = GENERATED_RUST_TEST_FILES.iter().copied().collect();
    let byte_paths: BTreeSet<&str> = GENERATED_RUST_TEST_FILE_BYTES
        .iter()
        .map(|(path, _)| *path)
        .collect();
    assert_eq!(
        manifest_paths, byte_paths,
        "generated Rust test byte-comparison rows must match GENERATED_RUST_TEST_FILES exactly"
    );

    let mut drifted = Vec::new();
    for (path, generated_bytes) in GENERATED_RUST_TEST_FILE_BYTES {
        let checked_in =
            fs::read_to_string(ws.join(path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        if checked_in != *generated_bytes {
            drifted.push(*path);
        }
    }
    assert!(
        drifted.is_empty(),
        "generated Rust tests drifted from the generator-manifest byte snapshot: {drifted:?}"
    );
}

#[test]
fn sg0_tests_as_data_migration_audit_classifies_test_ratchet() {
    if EXPECTED_HAND_AUTHORED_TEST.is_empty() {
        return;
    }

    let mut by_class: BTreeMap<TestsAsDataMigrationClass, Vec<&str>> = BTreeMap::new();
    let mut unclassified = Vec::new();

    for path in EXPECTED_HAND_AUTHORED_TEST {
        match tests_as_data_migration_class(path) {
            Some(class) => by_class.entry(class).or_default().push(*path),
            None => unclassified.push(*path),
        }
    }

    assert!(
        unclassified.is_empty(),
        "gate #84 migration audit must classify every hand-authored test path; \
         unclassified paths: {unclassified:?}"
    );

    for class in [
        TestsAsDataMigrationClass::CompileOrReject,
        TestsAsDataMigrationClass::LensOutputEquality,
        TestsAsDataMigrationClass::BehavioralObservation,
        TestsAsDataMigrationClass::BoundaryHostProcess,
        TestsAsDataMigrationClass::CementingV2Oracle,
        TestsAsDataMigrationClass::CensusOrRatchet,
        TestsAsDataMigrationClass::PropertyBased,
    ] {
        assert!(
            by_class.get(&class).is_some_and(|paths| !paths.is_empty()),
            "gate #84 migration audit lost class coverage for {class:?}"
        );
    }
}

#[test]
fn sg0_v3_non_test_fragment_subratchet() {
    let misplaced_fragments: Vec<&str> = EXPECTED_HAND_AUTHORED_FRAGMENTS
        .iter()
        .copied()
        .filter(|p| is_test_path(p))
        .collect();
    assert!(
        misplaced_fragments.is_empty(),
        "EXPECTED_HAND_AUTHORED_FRAGMENTS is part of T-PB-A's non-test ratchet; \
         test fragments need a separate T-PB-B fragment authority before being added: \
         {misplaced_fragments:?}"
    );

    let ws = workspace_root();
    let census_root = ws.join(CENSUS_ROOT);

    let mut all_txt: BTreeSet<String> = BTreeSet::new();
    walk_txt(&census_root, &ws, &mut all_txt);

    let observed: BTreeSet<String> = all_txt
        .iter()
        .filter(|p| !is_test_path(p) && !EXPECTED_GENERATED_FRAGMENTS.contains(&p.as_str()))
        .cloned()
        .collect();
    let expected: BTreeSet<String> = EXPECTED_HAND_AUTHORED_FRAGMENTS
        .iter()
        .map(|p| (*p).to_string())
        .collect();

    assert_eq!(
        observed, expected,
        "T-PB-A non-test SG-0 fragment sub-ratchet drifted. Retirements should be \
         removed from EXPECTED_HAND_AUTHORED_FRAGMENTS; new scaffold fragments must \
         name a dissolution trigger."
    );
}

#[test]
fn sg0_expected_fragment_lists_are_sorted_and_unique() {
    for (label, list) in [
        (
            "EXPECTED_HAND_AUTHORED_FRAGMENTS",
            EXPECTED_HAND_AUTHORED_FRAGMENTS,
        ),
        ("EXPECTED_GENERATED_FRAGMENTS", EXPECTED_GENERATED_FRAGMENTS),
    ] {
        let mut prev: Option<&str> = None;
        for p in list {
            if let Some(pv) = prev {
                assert!(
                    pv < *p,
                    "{label} must be sorted ASCII-ascending and unique; \
                     `{pv}` is not strictly less than `{p}`"
                );
            }
            prev = Some(p);
        }
    }
    let hand: BTreeSet<&str> = EXPECTED_HAND_AUTHORED_FRAGMENTS.iter().copied().collect();
    let gen: BTreeSet<&str> = EXPECTED_GENERATED_FRAGMENTS.iter().copied().collect();
    let overlap: Vec<&&str> = hand.intersection(&gen).collect();
    assert!(
        overlap.is_empty(),
        "EXPECTED_HAND_AUTHORED_FRAGMENTS and EXPECTED_GENERATED_FRAGMENTS must be disjoint; \
         overlap: {overlap:?}"
    );
}

#[test]
fn sg0_v3_hand_authored_txt_fragments() {
    // Ratchet for non-`.rs` scaffold fragments under `src/v3/compiler/`.
    // Any `.txt` file found under the census root must be named in
    // either `EXPECTED_HAND_AUTHORED_FRAGMENTS` (scaffold, on the SG
    // paydown backlog) or `EXPECTED_GENERATED_FRAGMENTS` (produced by
    // a named generator). A `.txt` that looks like hand-authored Rust
    // moved out of a `.rs` file would otherwise escape the `.rs`-only
    // ratchet above.
    let ws = workspace_root();
    let census_root = ws.join(CENSUS_ROOT);

    let mut all_txt: BTreeSet<String> = BTreeSet::new();
    walk_txt(&census_root, &ws, &mut all_txt);

    let expected_all: BTreeSet<String> = EXPECTED_HAND_AUTHORED_FRAGMENTS
        .iter()
        .chain(EXPECTED_GENERATED_FRAGMENTS.iter())
        .map(|p| (*p).to_string())
        .collect();

    if all_txt == expected_all {
        return;
    }

    let added: Vec<&str> = all_txt
        .difference(&expected_all)
        .map(String::as_str)
        .collect();
    let removed: Vec<&str> = expected_all
        .difference(&all_txt)
        .map(String::as_str)
        .collect();

    let mut msg = String::from(
        "SG-0 fragment census drift: `.txt` files under src/v3/compiler/ \
         do not match EXPECTED_HAND_AUTHORED_FRAGMENTS ∪ EXPECTED_GENERATED_FRAGMENTS.\n\n",
    );
    if !added.is_empty() {
        msg.push_str(
            "New scaffold fragment(s) (a `.txt` extension does NOT exempt it from SG-0):\n",
        );
        for p in &added {
            msg.push_str("  + ");
            msg.push_str(p);
            msg.push('\n');
        }
        msg.push_str(
            "\nFix: port the logic to `.dag` and remove the file, or add the path to \
             EXPECTED_HAND_AUTHORED_FRAGMENTS with a dissolution-trigger comment in the \
             file's own header. If the file is produced by a named generator, add it to \
             EXPECTED_GENERATED_FRAGMENTS instead.\n\n",
        );
    }
    if !removed.is_empty() {
        msg.push_str("Retired scaffold fragment(s) (remove from the expected list):\n");
        for p in &removed {
            msg.push_str("  - ");
            msg.push_str(p);
            msg.push('\n');
        }
    }
    panic!("{msg}");
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
    assert!(
        list.contains("\"emit_rust_bin_shim.rs\""),
        "hand_maintained_src should exclude emit_rust_bin_shim.rs (PB-1 shell helper) from recursive freshness drift"
    );
}
