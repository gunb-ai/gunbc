//! Self-Generation-0 — v3 Rust authority census + ratchet.
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

use v3_compiler::generated_files::GENERATED_FILES;

// Relative to workspace root; mirrors the single census root
// informally named in `dsl/gunbc/compiler.dag`.
const CENSUS_ROOT: &str = "src/v3/compiler";
const RETIRED_LENS_TESTGEN_RS: &str = "src/v3/compiler/src/lens_testgen.rs";
const RETIRED_LENS_APPLY_RS: &str = "src/v3/compiler/src/lens_apply.rs";
const RETIRED_REGEN_LENS_BIN_RS: &str = "src/v3/compiler/src/bin/regen_lens.rs";

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
fn r3_gate_7_regen_lens_bin_rs_stays_retired() {
    let retired_path = workspace_root().join(RETIRED_REGEN_LENS_BIN_RS);

    assert!(
        !retired_path.exists(),
        "R3 gate #7 (`regen_lens_dot_rs_retired`) requires \
         `{RETIRED_REGEN_LENS_BIN_RS}` to stay retired. The `regen_lens` \
         Cargo bin delegates through `src/regen_lens_entry.rs` into \
         `regen_lens_driver.rs` until PB-1 emits the shim from `.dag`."
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
// adding an entry is forbidden outside Self-Generation-0 without director
// sign-off.
// SG-6 landing (PR #560): the four per-lens regen bins
// (`regen_lens_cost.rs`, `regen_lens_cost_symbolic.rs`,
// `regen_lens_structural_resolution.rs`, `regen_lens_unused_parameters.rs`)
// and SG-4 prep's `regen_infer_helpers.rs` all folded into a single
// unified `regen_lens` driver driven by `src/v3/compiler/regen.dag`'s
// `LensRegistryEntry` records. Five retirements; one net-new entry
// at the time (`src/bin/regen_lens.rs`). **R3 gate #7** (`regen_lens_dot_rs_retired`,
// 2026-05-14): that program-sized bin path retired; logic lives in
// `regen_lens_driver.rs` with a thin `regen_lens_entry.rs` `[[bin]]` shell.
// The new `self_gen6_hand_authored_census_test.rs`
// pins the reduced bin census + full `(name, lens_file,
// generated_file)` registry tuples + `--lens` singleton resolve +
// end-to-end CLI smoke; it is hand-authored test infrastructure and
// belongs on this list.
//
// SG-6 follow-up landing (director sign-off from the
// `clever-swift-141` brief, 2026-04-19): the former standalone
// `self_gen4_prep_infer_helpers_freshness_test.rs` was absorbed into
// `self_gen6_hand_authored_census_test.rs`, so the infer-helpers snapshot
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
// P0-A / R1C-B: `p0_repeat_string_correct_gate` lives in `tests/fixtures/r1_gates.dag`;
// `test_runner_test::test_runner_runs_p0_repeat_string_correct_gate` is the integration receipt.
//
// Stage 3b DB-1 parse/apply ratchet bump — PR #564 adds one
// hand-authored integration file,
// `tests/integration/lane3_stage_3b_db1_test.rs`, because the
// ratchet is intentionally end-to-end over real compiler fixtures
// (diagnose -> apply correction -> reparse -> recompile), not a
// generated lens snapshot or unit-only helper. Dissolution trigger:
// when this slice is absorbed into a generic correction harness or a
// `.dag`-native correction-validation path, drop the entry. This is
// a bounded Self-Generation-0 exception for the merge-blocking Stage 3b receipt,
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
// below (`EXPECTED_HAND_AUTHORED_FRAGMENTS` + `self_gen0_v3_hand_authored_txt_fragments`)
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
// Self-Generation-0 fragment ratchet below is the sole census authority for
// crate-root scaffolds.
//
// SG-3f-d consumption proof (director review on PR #605, 2026-04-20):
// `self_gen3_surface_reflection_consumer_test.rs` is a bounded host-side
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
// while `rust_grounding_primitives.value_body` remains an unparsed top-level
// list. It intentionally compares only source literals that already fit
// `LiteralBits::Int(i64)`; the declared u64 upper half is not reachable
// until the deferred carrier-widening lane replaces that source-literal
// carrier. Dissolution triggers: R2 T-Substrate's top-level aggregate
// `ValueBody` sub-lane makes `rust_grounding_primitives` row values
// structurally walkable, and the carrier-widening lane makes the full
// declared unsigned range parseable by source literals. At that point
// this helper should consume those declared rows directly or move behind
// generated substrate accessors.
//
// R3 T-FixedPoint P0 / DB-8: `self_host_receipt_p0.rs` is intentionally hand-authored
// receipt-key surface (stable JSON field names for `self_host_fixed_point` trend reads),
// not generated output. Dissolution: fold into a `.dag` or generated authority when one
// owns receipt schema; until then this module + census line are the bounded ratchet receipt.
//
// PB-0 / Director **msg_84abadad** + scope correction **msg_dda96d21** (2026-05-13): **43-entry**
// `NON_TEST` + **3** `FRAGMENTS` taxonomy (§3 + §4) in
// `docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md`
// (substitute visibility for dependency tree; not enforcement — see INVARIANTS). Inline
// `// blocked: …` comments on individual census lines are deferred until taxonomy stabilizes
// across sibling merges.
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
    // F.14 / T-PB-B: `ExecuteCommand` logical child for `tests/dag/boundary_emit_gates.template.dag`.
    // Irreducible host-shim bin (exit 0/1); calls `v3_compiler::boundary_emit_gates::check_*`.
    // **P5 receipt (Mechanism (b), disposition (3)):** deferral to ROADMAP.md **T-PB-B** /
    // `pb_rust_tests_outside_residual_zero` and active deferral **PB-Runtime-External-Toolchain-TestClaims**
    // (hand `tests/boundary/*.rs` → `.dag` `TestClaim` / substrate target verification).
    // Dissolution: delete this bin when the last class-5 boundary host shim in `tests/boundary/`
    // is retired and `boundary_emit_gates.template.dag` (or generated runner) is sole authority
    // for the remaining `ExecuteCommand` claims — checkable: Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`
    // no longer lists any `tests/boundary/*` path still covered only by this bin.
    "src/v3/compiler/src/bin/boundary_emit_gates.rs",
    "src/v3/compiler/src/bin/r1c_e_emit_gates.rs",
    "src/v3/compiler/src/bin/regen_bootstrap.rs",
    "src/v3/compiler/src/bin/regen_parse.rs",
    "src/v3/compiler/src/bin/regen_parse_tables.rs",
    "src/v3/compiler/src/bin/regen_tokenize.rs",
    "src/v3/compiler/src/bin/regen_v3.rs",
    "src/v3/compiler/src/bin/self_host_fixed_point.rs",
    "src/v3/compiler/src/bootstrap.rs",
    "src/v3/compiler/src/bootstrap_regen_fresh.rs",
    // F.14 / T-PB-B: shared `check_*` for class-5 boundary emit gates; thin host `#[test]`
    // shims and `boundary_emit_gates` bin both call (`tests/dag/boundary_emit_gates.template.dag`).
    // **P5 receipt (Mechanism (b), disposition (3)):** deferral to ROADMAP.md **T-PB-B** /
    // `pb_rust_tests_outside_residual_zero` + **PB-Runtime-External-Toolchain-TestClaims**.
    // Dissolution: delete when every claim in `boundary_emit_gates.template.dag` is evaluated by
    // substrate `run_target_verification` / v4 `.dag` `TestClaim` runtime without this module
    // (same lane as `r1c_e_gates.rs` scaffold, but boundary-class scope not R1C-E / issue #973).
    "src/v3/compiler/src/boundary_emit_gates.rs",
    // E-5 / P2–P4: wall-bounded host subprocess I/O shared by `post_emit_verifier` and
    // W1/L5 `test_runner` (fail-closed vs unbounded `Command::output`).
    // **P5 receipt (Mechanism (b)):** matching row in `_internal/INVARIANTS_OPS.md`.
    // Dissolution: delete when PB-Runtime owns bounded host-child policy for all
    // post-emit / ExecuteCommand paths without this hand module.
    "src/v3/compiler/src/bounded_host_command.rs",
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
    // W3 / T-38: executable `run_emit_host_rust` hand-Rust bridge (`tools/emit_host_runner`).
    // **P5 receipt (Mechanism (b)):** matching row in `_internal/INVARIANTS_OPS.md`
    // § Self-Generation-0 hand-authored compiler non-test paths (`T-PB-B` / `pb_rust_tests_outside_residual_zero`).
    // Dissolution: delete when substrate eval owns host dispatch without hand-Rust bridge.
    "src/v3/compiler/src/emit_host_bridge.rs",
    // T-22: `emit_host_eval.rs` — rust/go/python rows (#4225/#4254 main) + B3 `run_host_process`
    // omni-emission transport dispatch + SignedI32Le i32 reification (#4641).
    // **P5 receipt (Mechanism (b)) — disposition (3) explicit deferral:** lane **T-PB-B** /
    // `pb_rust_tests_outside_residual_zero` (gate marker
    // `src/v3/compiler/tests/fixtures/r1_release_acceptance.dag:25`; ROADMAP `ROADMAP.md:31`, `:43`).
    // Paired execution (E-10 honest partition): `emit_host_runner` unit
    // `runtime_value_parse_signed_i32_le_decodes_fixed_bytes` (hermetic byte-decode) +
    // `emit_host_eval.rs` in-module `b3_runtime_value_signed_i32_le_as_int_eval_dispatch_reifies_five`
    // (eval intercept decode). **DORMANT:** `run_host_process` process-spawn (real tsc+node;
    // v3-eval-intercept-only; v2 --claim-run has no hook) — dissolves at T-22 substrate eval /
    // gunbc#4750 (supersedes #4674). `v4_emit_host_eval_dispatch_test.rs` covers rust rows only; `comprep_b3_ts_descriptor_node_run.dag`
    // is wire scaffold (not v2 --claim-run).
    // Census **+0 NON_TEST** (row on main since #4225; #4641 extends eval hook, no new Self-Generation-0 path).
    // Dissolution: substrate Callable dispatch owns all host rows without this hand-Rust eval hook.
    "src/v3/compiler/src/emit_host_eval.rs",
    "src/v3/compiler/src/emit_rust.rs",
    "src/v3/compiler/src/emit_rust_bin_shim.rs",
    // R1C-E + m1_3: shared `PROGRAM_FIXTURES` / `REFLECTED_FIXTURES` tables (single source of truth).
    "src/v3/compiler/src/emit_rust_roundtrip_fixtures.rs",
    "src/v3/compiler/src/enforced_lens_application.rs",
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
    "src/v3/compiler/src/regen_lens_driver.rs",
    "src/v3/compiler/src/regen_lens_entry.rs",
    "src/v3/compiler/src/regen_parse_emit.rs",
    "src/v3/compiler/src/regen_parse_tables_emit.rs",
    "src/v3/compiler/src/regen_tokenize.rs",
    "src/v3/compiler/src/self_host_receipt_p0.rs",
    "src/v3/compiler/src/test_runner.rs",
    "src/v3/compiler/src/v4_hollow_alias_gate.rs",
    "src/v3/compiler/src/wall_clock_ratchet_manifest.rs",
];

// All test .rs files under `src/v3/compiler` that are currently
// hand-authored. Sorted; one path per line, relative to the
// workspace root. T-PB-B owns shrinking this subset toward the
// TESTING.md §"Post-R2 shape" residual. T-PB-A reductions must not
// rely on this list moving.
// Slice 1 census reconciliation (2026-05-02): sorted path list; update when
// adding/removing hand-authored integration tests (Self-Generation-0 ratchet).
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
const EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[
    // `tests/boundary/` before `tests/determinism_test.rs` (ASCII `boundary/` < `determinism_`);
    // `l5_*` before `m1_*` / `m2_*` within boundary/.
    "src/v3/compiler/tests/boundary/l5_cross_target_consistency.rs",
    "src/v3/compiler/tests/boundary/m1_3_emit_go_test.rs",
    "src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs",
    "src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs",
    "src/v3/compiler/tests/boundary/m1_5_emit_omni_demo_test.rs",
    // **P5 receipt (F.14 / T-PB-B):** `m2_emit_multi_field_struct_variant_test.rs` retired;
    // `tests/dag/boundary_emit_gates.template.dag` + `boundary_emit_gates` bin are authority
    // (`t_pb_b_1_dag_runner_test::boundary_emit_gates_suite_passes_through_runner`).
    // Phase 1 leaf-model go R1/R2a/R2b/R3-external: boundary Go toolchain exercise for
    // R1 int surface spelling, R2a int algebra ops, R2b int64 overflow wrap, R3-external
    // Symbol-as-string projection until T-22 modeled `run_target_verification` owns target
    // verdicts; P5 deferral to ROADMAP.md `PB-Runtime-External-Toolchain-TestClaims`.
    // Interim host runners: scripts/v4-leaf-model-go-r{1,2a,2b,3-external}-verify.sh.
    "src/v3/compiler/tests/boundary/v4_leaf_model_go_r1_r2_r3_external_test.rs",
    // Phase 1 leaf-model python cross-runtime DRIFT (Worksheet C): boundary tokenize/parse smoke
    // of the drift std/lens/claim dags + runtime divergence exercise for
    // `python_cross_runtime_drift_*` (Python arbitrary precision vs Rust/Go fixed-width wrap);
    // host runner `scripts/v4-leaf-model-python-cross-runtime-drift-verify.sh`.
    //
    // **P5 receipt (INVARIANTS.md §P5 — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`):** the checkable
    // receipt is this registration itself (per-PR mechanism (b)) — the `EXPECTED_HAND_AUTHORED_TEST`
    // census moves by exactly +1 now and must shrink by 1 at dissolution; the sorted/unique and
    // disk-vs-list census tests in this file mechanically enforce it. Deferral lane —
    // **ROADMAP.md** "What v4 is building toward" rows "Tests as `.dag` `TestClaim` data"
    // (`src/v4/test/claim/`) and "Pure bootstrap / self-host" (trajectory to zero hand-maintained
    // Rust; `self_host.dag` ratchet). Concrete dissolution trigger: delete this hand-Rust test when
    // the modeled `TestClaim` runner exercises
    // `src/v4/test/claim/language_model/python_cross_runtime_drift.dag` directly, so the boundary
    // host-process bridge is no longer the only exerciser of the drift claim.
    "src/v3/compiler/tests/boundary/v4_leaf_model_python_cross_runtime_drift_test.rs",
    // Python RCA release-minimum lane (#4137 section 11.8): L1 static structural mypy (Worksheet B);
    // host runner v4-leaf-model-python-l1-mypy-static-verify.sh (pyright roster on main #4231).
    //
    // **P5 receipt (Mechanism (b), disposition (2)):** `EXPECTED_HAND_AUTHORED_TEST` 171 → 172;
    // T-PB-B partition (module doc lines 9–10 + `tests/boundary/README.md`); lane
    // `docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md` Worksheet B (#4137 §11.8);
    // host runner v4-leaf-model-python-l1-mypy-static-verify.sh. Dissolution: drop when
    // modeled verification supersedes `src/v4/test/claim/language_model/python_l1_static.dag`.
    "src/v3/compiler/tests/boundary/v4_leaf_model_python_l1_static_receipts_test.rs",
    // Phase 1 leaf-model python L2 CROSS-TARGET PARITY (Worksheet C): boundary tokenize/parse smoke
    // of the parity std/lens/claim dags + runtime AGREEMENT exercise for `python_l2_parity_*`
    // (small-value add `2+3=5` and Symbol projection `x` agree across Python/Rust, Go corroborating).
    // The positive complement of the cross-runtime DRIFT boundary test: drift proves divergence at
    // the fixed-width boundary, parity proves agreement on the common domain.
    //
    // **P5 receipt (INVARIANTS.md §P5 — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`):** the checkable
    // receipt is this registration itself (per-PR mechanism (b)) — the `EXPECTED_HAND_AUTHORED_TEST`
    // census moves by exactly +1 now and must shrink by 1 at dissolution; the sorted/unique and
    // disk-vs-list census tests in this file mechanically enforce it. Deferral lane —
    // **ROADMAP.md** `PB-Runtime-External-Toolchain-TestClaims` (the exact deferral for
    // host-spawned toolchain boundary tests; same lane as the Go and drift leaf-model siblings at
    // `:431` / `:439-448`) plus the "What v4 is building toward" rows "Tests as `.dag` `TestClaim`
    // data" (`src/v4/test/claim/`) and "Pure bootstrap / self-host" (trajectory to zero
    // hand-maintained Rust). Concrete dissolution trigger: delete this hand-Rust test when the
    // modeled `TestClaim` runner executes the three target sources and binds observed stdout to
    // `expected_parity.actual`, so the boundary host-process bridge is no longer the only exerciser
    // of `src/v4/test/claim/language_model/python_l2_cross_target_parity.dag` (lane
    // `docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md` Worksheet C / #4137 §11.8).
    "src/v3/compiler/tests/boundary/v4_leaf_model_python_l2_cross_target_parity_test.rs",
    // Phase 1 leaf-model python R1 (W2.6 / PR #3938 §11.4): boundary CPython exercise for
    // `src/v4/lens/leaf_model_verification.dag` python fixtures until T-22 modeled
    // `run_target_verification` owns target verdicts; interim host runner
    // `scripts/v4-leaf-model-python-r1-verify.sh`.
    "src/v3/compiler/tests/boundary/v4_leaf_model_python_r1_test.rs",
    // Phase 1 leaf-model python R2a/R2b/R3-external (MW-D3 cross-target widening): boundary
    // CPython exercise for R2a algebra ops, R2b arbitrary-precision add, R3-external Symbol
    // projection; host runners scripts/v4-leaf-model-python-r2{a,b,r3-external}-verify.sh.
    "src/v3/compiler/tests/boundary/v4_leaf_model_python_r2_r3_external_test.rs",
    // Phase 1 leaf-model verification R1 (`docs/planning/v4-leaf-model-verification-2026-05-30.md` §7):
    // boundary rustc exercise for `src/v4/lens/leaf_model_verification.dag` fixtures until
    // T-22 modeled `run_target_verification` owns target verdicts.
    //
    // **P5 receipt (INVARIANTS.md §P5 — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`):** explicit
    // deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
    // `pb_rust_tests_outside_residual_zero` + gunbc#4757/gunbc#4765 (Runtime/TestClaim
    // verdict surface; interim host runner `scripts/v4-leaf-model-rust-r1-verify.sh`).
    // Dissolution: delete when modeled runner exercises
    // `src/v4/test/claim/language_model/rust_r1.dag` without this hand-Rust bridge.
    "src/v3/compiler/tests/boundary/v4_leaf_model_rust_r1_rustc_test.rs",
    // Phase 1 leaf-model R2a/R2b(1–2)/R3-external (W1.7): boundary rustc for algebra inhabitance,
    // overflow runtime behavior, Symbol projection; host runners v4-leaf-model-rust-r2{,a,b}-verify.sh
    // and v4-leaf-model-rust-r3-external-verify.sh.
    "src/v3/compiler/tests/boundary/v4_leaf_model_rust_r2_r3_external_rustc_test.rs",
    // Phase 1 leaf-model R3-internal (post-SG-1 #3956): emit coupling receipt for Symbol row mutation;
    // boundary projection replay for `target_atom_type_spelling` + value kind until T-22 eval.
    //
    // **P5 receipt (INVARIANTS.md §P5 — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`):** explicit
    // deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
    // `pb_rust_tests_outside_residual_zero` + gunbc#4757/gunbc#4765 (Runtime/TestClaim
    // verdict surface; interim host runner `scripts/v4-leaf-model-rust-r3-internal-verify.sh`).
    // Dissolution: delete when modeled runner exercises
    // `RustEmitProjectionEqualityExpectation` without this hand-Rust bridge.
    "src/v3/compiler/tests/boundary/v4_leaf_model_rust_r3_internal_emit_coupling_test.rs",
    // Phase 1 leaf-model typescript R2a/R2b/R3-external (MW-D3 alpha lane): boundary tsc + Node
    // exercise for R2a number algebra ops (TS2339 falsification), R2b bigint runtime vs number
    // lane divergence, R3-external Symbol() factory vs `new Symbol` (TS7009); host runners
    // scripts/v4-leaf-model-typescript-r2{a,b,r3-external}-verify.sh.
    "src/v3/compiler/tests/boundary/v4_leaf_model_typescript_r2_r3_external_test.rs",
    "src/v3/compiler/tests/determinism_test.rs",
    "src/v3/compiler/tests/integration.rs",
    "src/v3/compiler/tests/integration/anthropic_messages_callable_test.rs",
    // R3 gate #68 (`anthropic_wire_demonstration`): hermetic typed request/response
    // cycle over the Anthropic Messages wire surface using a deterministic mock.
    "src/v3/compiler/tests/integration/anthropic_messages_wire_demo_test.rs",
    // T-Ground services.dag PR-β anthropic_operations Phase 1 pilot
    // (#1252). Hand-authored ratchet entry added per Self-Generation-0 census discipline.
    "src/v3/compiler/tests/integration/anthropic_operations_test.rs",
    "src/v3/compiler/tests/integration/anthropic_schema_lockstep_test.rs",
    // R3 coproduct slice 2: hermetic JSON for `tool_result.content` scalar vs block array.
    "src/v3/compiler/tests/integration/anthropic_tool_result_wire_demo_test.rs",
    "src/v3/compiler/tests/integration/bridge_ledger_carrier_test.rs",
    // PB Tier-2 lower-helper exact-string patch class (#1014): zero-residual receipt +
    // source ratchet; see `bridge_lower_helpers_patch_zero_residual_test.rs` module docs.
    "src/v3/compiler/tests/integration/bridge_lower_helpers_patch_zero_residual_test.rs",
    // R2 PB canonical-lens bridge ratchet (PR #1183 — disposition for
    // `bridge_canonical_lens_name_dispatch_retired`). Pins the remaining
    // `include_str!` / name-dispatch surface in `test_runner.rs` per
    // `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`. .dag-port
    // / dissolution path is the same gate that closes the bridge itself
    // (PB-Runtime interpreter-as-data or typed lens-registry carrier);
    // until then, this hand-Rust ratchet IS the slice's structural gate.
    "src/v3/compiler/tests/integration/canonical_lens_bridge_ratchet_test.rs",
    // R3 gate #87: provenance `origin_of` seam check (retired from `cementing_lens_registry_dispatch_test.rs`).
    "src/v3/compiler/tests/integration/cementing/cementing_provenance_origin_integration_test.rs",
    // R3 T-Lens-Behavioral-Parity: Band-C cementing receipt for the complexity lens
    // COMPLETE promotion against frozen v2-oracle values. Temporarily stays Rust
    // because `.dag` TestClaims cannot yet consume the `ComplexitySummary`
    // report carrier (`Gate73_ReportPredicateCarriers`).
    "src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs",
    // R3 gate #78 residual: pins `per_call_pattern_at` on the unary countdown fixture while the
    // host `symbolic_cost_of` wrapper still owns the alias-collapse post-pass. Gate #80 Band-C
    // symbolic-cost cementing moved to
    // `tests/dag/t_r3_gate_87_cementing_regen_cost_symbolic.dag`; do not count this file as the
    // `cost_symbolic` COMPLETE receipt.
    "src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs",
    // R3 gate #76 (`e_p_per_call_descent_evidence_full_coverage`) Phase-3 lens-consumer
    // cementing ratchets (#[ignore]'d) pinning the consumer-path expectation for
    // match-payload + multi-arg per-arg vectors against `complexity_of` + `symbolic_cost_of`.
    // Lane E-P per ROADMAP.md "Lane E-P — per-call descent-evidence provenance (M)" — see
    // INVARIANTS.md §P5 row for dissolution trigger (lens-consumer match-arm walker extension).
    "src/v3/compiler/tests/integration/cementing/e_p_per_call_descent_lens_consumer_cementing.rs",
    // R3 gate #82: Operation-row consumer receipt for effect_enumeration COMPLETE.
    "src/v3/compiler/tests/integration/cementing/effect_enumeration_lens_behavioral_completion.rs",
    // R3 T-Lens-Application-Surface gate #94 (`memory_peak_cost_basis_demonstrated`).
    "src/v3/compiler/tests/integration/cementing/memory_peak_cost_basis_demo.rs",
    "src/v3/compiler/tests/integration/common/budgeted.rs",
    "src/v3/compiler/tests/integration/common/cached_compile.rs",
    "src/v3/compiler/tests/integration/common/determinism_fixtures.rs",
    // E6-G1.a Option 3 static lens mechanism (#1853 + #1857):
    // `find_list_empty_constructor_tag` helper for opaque-`Dag` harness tags
    // (P5 receipt: Self-Generation-0 census membership for `e6_g1a_option3_static_lens_test.rs`).
    "src/v3/compiler/tests/integration/common/list_variant_tags.rs",
    "src/v3/compiler/tests/integration/common/mod.rs",
    "src/v3/compiler/tests/integration/common/r1_gates_bridge.rs",
    "src/v3/compiler/tests/integration/common/rust_comment_strip.rs",
    "src/v3/compiler/tests/integration/common/substrate_receipts.rs",
    // R3 gate #78 / E-P: shared countdown `SymbolicCost` oracle helper for cost-lens consumer
    // tests (`cost_lens_symbolic_consumer_test`, lane2 `lane2_stage_2d_symbolic_cost_test`).
    "src/v3/compiler/tests/integration/common/symbolic_cost_countdown.rs",
    // R3 gates #40/#70: host-side `SymbolicCost` → v3 data-expression serializer for
    // `SymbolicCostExprEquals` dynamic oracle fixtures (`m1_5_verification_test.rs`); stays
    // until testgen/reflection can author the same `data …: SymbolicCost = …` literals without
    // a Rust mirror of the algebra surface syntax.
    "src/v3/compiler/tests/integration/common/symbolic_cost_verification_fixture.rs",
    // R3 gate #87: unit tests for `tests/integration.rs` wiring scanners (split from retired
    // `cementing_lens_registry_dispatch_test.rs`).
    "src/v3/compiler/tests/integration/common/wiring_scanner_test.rs",
    // Coverage-defect acceptance keys: parse-level ratchet for
    // `src/v4/lens/coverage.dag`; retires when generated coverage owns the
    // same key projection.
    "src/v3/compiler/tests/integration/coverage_defect_acceptance_dag_test.rs",
    // R3 L6 carrier slice (PR #1842; Measure-carrier precedent at #1819,
    // Director Option 2 RATIFIED at
    // gunbc#828 #issuecomment-4377533390): slice-active ratchet for
    // `cross_target_coverage.dag` (six type declarations exist;
    // `emission_path_projections` Phase-1 populated with 41 rows —
    // Rust 13 / Python 16 / Go 12). Stays hand-Rust alongside
    // `method_template_contract_test.rs` until testgen covers
    // reflected-Dag structural assertions over std/ row authorities.
    "src/v3/compiler/tests/integration/cross_target_coverage_carrier_test.rs",
    // Dissolution-lens subsumption carrier: parse-level ratchet for
    // `src/v4/lens/subsumption.dag`; retires when v4 TestClaim/generated
    // coverage owns the same carrier and first-row projection.
    "src/v3/compiler/tests/integration/dissolution_subsumption_carrier_test.rs",
    // E6-G1.a Option 3 — static `Lens<Int>` + `mini_report` mechanism demonstration
    // (#1853; witness-flow + TESTING.md split per #1857). Self-Generation-0 ratchet: hand-authored
    // integration test census membership.
    "src/v3/compiler/tests/integration/e6_g1a_option3_static_lens_test.rs",
    "src/v3/compiler/tests/integration/e_i_lane_induction_preflight_test.rs",
    // T-Substrate-Lens-Primitive Lens<EmissionProvenance> structural cementing
    // test (PR #1928). Per Director Q1(a) RATIFIED at gunbc#1739
    // #issuecomment-4392562911. Hand-authored entry added per Self-Generation-0 census
    // discipline; integration-test home (matches `mini_lens` /
    // `e6_g1a_option3_static_lens_test` precedent for fixture-bound lens
    // instances).
    "src/v3/compiler/tests/integration/emission_provenance_lens_test.rs",
    // Operator-directive 2026-05-29 / PR #3913 — E-5 multi-target emit verification:
    // `PROGRAM_FIXTURES` must pass each Shape-A `post_emit_verifier` (not only M1
    // self-host `src/v4` cargo-check). **P5 receipt (Mechanism (b)):** matching row
    // in `_internal/INVARIANTS_OPS.md` § Self-Generation-0 hand-authored integration test receipts.
    // Dissolution: remove when obligations run as `.dag` `TestClaim` rows / T-38 runner
    // without this host harness (`src/v4/test/claim/manual/multi_target_emit_verification_gate.dag`).
    "src/v3/compiler/tests/integration/emit_verification_gates_test.rs",
    // T-Ground-Engine Phase-1 loader-close (PR #776, Director-approved
    // Path 2): hand-Rust integration test pinning
    // `Dag::rust_grounding_primitives()` type-structure walk + the
    // `ValueBody::Unparsed` boundary that flips when R2 T-Substrate's
    // 4th sub-lane lands top-level `ValueBody::List`/aggregate.
    // Dissolves into testgen authority when the testgen path covers
    // the dsl/extdeps loader surface.
    "src/v3/compiler/tests/integration/extdeps_rust_primitives_loader_test.rs",
    // Ctrl-Migration Emission-Targets Phase 3 HTTP/SQL extdeps: narrow host-side
    // parser receipt for `dsl/extdeps/transports/rest.dag` and
    // `dsl/extdeps/transports/sql.dag`. Explicit P5 receipt lives
    // in INVARIANTS.md § "Self-Generation-0 hand-authored integration test receipts"; dissolves
    // when extdeps transport files are covered by a `.dag`-native parse/authority
    // suite or generated test harness.
    "src/v3/compiler/tests/integration/extdeps_sql_transport_test.rs",
    "src/v3/compiler/tests/integration/file_attachment_substrate_carrier_test.rs",
    "src/v3/compiler/tests/integration/four_fixture_regression_test.rs",
    // get-off-v3 by-execution caller census + down-only ratchet for
    // `v3_compiler::compile_to_dag` (the v3 whole-source compile entry). A narrow
    // hand-Rust instrument: it walks the live source tree at test time and counts
    // direct calls to `compile_to_dag`, holding the discovered total at/under a
    // single ceiling that ratchets toward zero (no per-caller pinned ledger; E-10 /
    // #4633 by-execution lineage).
    //
    // **P5 receipt (INVARIANTS.md §P5 — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`):** the
    // checkable receipt is this registration itself (per-PR mechanism (b)) — the
    // `EXPECTED_HAND_AUTHORED_TEST` census moves by exactly +1 now and must shrink by
    // 1 at dissolution; the sorted/unique and disk-vs-list census tests in this file
    // mechanically enforce it. Deferral lane — `src/v3/SELF_HOSTING.md` self-host
    // trajectory (hand-Rust v3 surface → 0) plus the M-CI / CI-via-dag lane that owns
    // CI-enforcing this ratchet. Concrete dissolution trigger: this instrument
    // dissolves with its own subject — when the ceiling reaches 0 (no direct
    // `compile_to_dag` callers remain) and v3 retires, or earlier if the census is
    // re-expressed as a `.dag` `TestClaim` under the CI-via-dag lane — at which point
    // this hand-Rust test is deleted and the entry removed from this list (−1).
    "src/v3/compiler/tests/integration/get_off_v3_compile_to_dag_census_test.rs",
    // Idempotency Lens<C> instance blocker ratchet (R2 Substrate): focused
    // hand-Rust receipt proving the actual idempotency lens instance must
    // wait for generic function-valued data-field matching, while imported
    // sum-return helper calls are not the current blocker. No substrate
    // instance lands here; dissolves into the real idempotency Lens<C>
    // instance/equivalence ratchet once the lowerer prerequisite is fixed.
    "src/v3/compiler/tests/integration/idempotency_lens_instance_blocker_test.rs",
    // T-Substrate cardinality subset for int literals: behavior receipt for
    // range narrowing, explicit Int64 default, and MagnitudeOutOfRange.
    // Dissolves into .dag-native/testgen coverage when diagnostic assertions
    // can name this case without a host-side integration harness.
    "src/v3/compiler/tests/integration/int_literal_cardinality_test.rs",
    "src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2a_effects_smoke.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2b_db18_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2c_db15_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2d_symbolic_cost_test.rs",
    "src/v3/compiler/tests/integration/lane2_stage_2e_parallelism_test.rs",
    "src/v3/compiler/tests/integration/lane3_stage_3b_db1_test.rs",
    // R3 §1.8 gate #89 (`section_ref_substrate_landed`): `SectionRef` disjoint-sum substrate receipt.
    "src/v3/compiler/tests/integration/lens_application_substrate_carrier_test.rs",
    // R3 gate #73 (`lens_behavioral_parity_demonstration`): temporary host
    // receipt for the four-lens parity snapshot while LensOutputEquals /
    // frozen-oracle claims migrate to `.dag` TestClaim data. Dissolution is
    // tracked by `docs/r3-program-plan.md` §1.8 row 73, T-Tests-As-Data
    // rows 84/87, and `ROADMAP.md` §"Post-merge debt" row "Hand-Rust census" /
    // T-PB-B test subset; delete with the module's `tests/integration.rs`
    // registration.
    "src/v3/compiler/tests/integration/lens_behavioral_parity_demonstration_test.rs",
    // T-CostLens-Composition Slice 1a.1 (#2141 ε scope per gunbc#2181 ratification):
    // Rust integration tests exercising `lens_cost_target_realization` `.dag`-tier
    // consumer of `declaration_by_name` (introduced by Slice 1a.0 / PR #2194).
    // P2 same-PR-consumer-evidence per codex BLOCKING surfaced post-merge.
    //
    // **Dissolution trigger (P5)**: retires when T-Tests-As-Data infrastructure
    // expresses ".dag-fn-resolution-against-bootstrap" assertions and
    // `cost_lens_demonstration` as structural `TestClaim` data instead of
    // hand-Rust integration tests.
    //
    // **P5 explicit deferral receipt**: lane = T-Tests-As-Data-Completeness /
    // T-PB-B test-census dissolution; concrete ROADMAP rows =
    // `ROADMAP.md` T-PB-B test subset row at `ROADMAP.md:170` (Rust-authored
    // tests migrate to `.dag` `TestClaim` declarations) plus Self-Generation-0 PR-window
    // discipline at `ROADMAP.md:177`. Gate authority = `docs/r3-program-plan.md`
    // row #70 (`cost_lens_demonstration`) and the T-CostLens worker brief's
    // same-slice acceptance bullet. This is a strict deferral of the test
    // harness shape, not of the gate behavior.
    //
    // The 6 resolver assertions here (one per `*Realization` meta-type —
    // `assert meta.is_some() && name == "X"`) factor as `OutputEquals` /
    // declaration-resolution claims under the T-Tests-As-Data umbrella (#1966
    // §3 ratchet predicate scope). Gate #70 additionally factors into
    // TestClaims over the representative recursive fixture: emitted target
    // program contains the recursive call, lowered DAG has Add/Sub/Eq
    // algebra-instance operator transforms, `per_call_descent_evidence` observes
    // the self-call, and the Rust-side cost composition preserves the observable
    // linear `SymbolicCost` bound. Until that landing, hand-Rust is the consumption
    // path for `.dag`-fn-from-Rust and emitted-target cost-composition
    // assertions; Mgr standing-authority approval at gunbc#2221
    // #issuecomment-4404395097 ratifies this bridge for the Slice 1a.1 / gate
    // #70 window.
    "src/v3/compiler/tests/integration/lens_cost_target_realization_test.rs",
    "src/v3/compiler/tests/integration/lens_register_correspondence_test.rs",
    // T-Substrate-Lens-Primitive (R2 Substrate, first slice): Director-
    // approved hand-Rust acceptance for `Lens<C>` substrate carrier and
    // Q6.5 two-layer diagnostic-kind authority. Five structural claims
    // over the regenerated bootstrap Dag — Lens<C> 6-field shape,
    // Diagnostic.kind widened to AnyDiagnosticKind, Layer-1 closed sum
    // unchanged, AnyDiagnosticKind two-constructor shape, and Layer-2
    // payload-intentionally-absent gap receipt. Dispatch (#1130 +
    // `docs/design-lens-framework.md` Q6.5) accepted "focused structural
    // acceptance tests" rather than a parallel testgen harness.
    // Dissolves into .dag `TestClaim` form when testgen covers reflected-
    // Dag structural assertions over std/ types.
    "src/v3/compiler/tests/integration/lens_substrate_carrier_test.rs",
    "src/v3/compiler/tests/integration/m0_acceptance.rs",
    "src/v3/compiler/tests/integration/m1_3_lens_cost_test.rs",
    "src/v3/compiler/tests/integration/m1_3_lens_unused_parameters_test.rs",
    // R3 T-Omni-Shape-B Brief #1 (#2219 / PR #2251): integration receipt
    // for same-DAG Shape B OpenAPI projection. Dissolves into TestClaim /
    // `.dag`-native Shape B demo coverage with the OpenAPI projector above.
    "src/v3/compiler/tests/integration/m1_5_omni_shape_b_openapi_test.rs",
    "src/v3/compiler/tests/integration/m1_5_testgen_test.rs",
    "src/v3/compiler/tests/integration/m1_5_user_authored_lens_gate_test.rs",
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
    // T-Ground-LanguageSpec / R2-Substrate: Director-approved hand-Rust
    // acceptance for `MethodTemplateContract` substrate carrier (PR #1175).
    // Three structural claims over the regenerated bootstrap Dag —
    // distinct-from-§6a-MethodContract, no-cost-data field set,
    // per-target-list `dag_method` uniqueness (vacuous over zero rows
    // today; load-bearing once Grounding's row-population PR lands).
    // Dispatch explicitly accepted "focused Rust tests over the reflected
    // substrate" rather than a parallel testgen harness; dissolves into
    // .dag `TestClaim` form when the testgen path covers reflected-Dag
    // structural assertions over std/ types.
    // Method-declaration registry (R2 Substrate, follow-up to #1175 +
    // #1186): Director-approved hand-Rust acceptance for the minimal
    // method-name registry in `dsl/std/methods.dag` + `MethodRef` typed
    // reference in `src/v3/std/methods.dag` + `MethodTemplateContract.
    // dag_method` refinement from bare `DeclarationRef` to `MethodRef`.
    // Four structural claims: registry covers all 64 algebra-template
    // names (drift-detection), `MethodDeclaration` identity-only,
    // `MethodTemplateContract.dag_method` field type points at
    // `MethodRef`, `MethodRef` is a single-field decl wrapper.
    // Dispatch (#1130) accepted hand-Rust acceptance over the reflected
    // bootstrap. Dissolves into .dag `TestClaim` form when testgen
    // covers reflected-Dag structural assertions over std/ types.
    "src/v3/compiler/tests/integration/method_registry_test.rs",
    "src/v3/compiler/tests/integration/method_template_contract_test.rs",
    // Gunbc #1982 / §1.8 gate #97 — emit-shim retirement coherence (v2 tree vs Gap-4 producer).
    "src/v3/compiler/tests/integration/method_template_projection_emit_shim_coherence_test.rs",
    // R3 §1.8 gate #39 (`no_coercion_cost_dimension`, T-CostLens-Composition):
    // `.dag` substrate ratchet — no parallel `CoercionCost` token outside comments.
    "src/v3/compiler/tests/integration/no_coercion_cost_dimension_ratchet_test.rs",
    "src/v3/compiler/tests/integration/pb1_bootstrap_full_snapshot_test.rs",
    // R3 row 85 / PB #1560 Gap 4: focused acceptance for the
    // `pb_method_template_projection` consumer hook. Stays hand-Rust
    // alongside `method_template_contract_test.rs` until testgen covers
    // reflected-Dag structural assertions over std/ row authorities.
    "src/v3/compiler/tests/integration/pb_method_template_projection_test.rs",
    "src/v3/compiler/tests/integration/pipe_desugar.rs",
    // ctrl#1476 B5: positional-Conj fold_list-by-construction detection on emit-path language models.
    // **P5 receipt (INVARIANTS.md §P5 — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`):** explicit
    // deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
    // dissolves when modeled `TestClaim` exercises emit-path grammar-relation token encode without
    // this hand-Rust substring/parse ratchet.
    "src/v3/compiler/tests/integration/positional_conj_fold_list_emit_path_test.rs",
    // Prereq-X (call-on-field-access) blocker ratchet for fold_lens<C>
    // consumer wiring (Prereq-3b dispatch on inbox #1141; audit at
    // docs/design-prereq-x-ho-field-call.md / PR #1264). Pins the parser
    // diagnostic shape of X1 (`w.f(x)`) and X3 (`= { let g = w.f; g(x) }`)
    // so the ratchet flips red when the implementation lane lands; retired
    // by the lane owner at the same time as the parser/lowerer change.
    "src/v3/compiler/tests/integration/prereq_x_call_on_field_access_ratchet_test.rs",
    // R1 release acceptance fixture: strict PB gate 3 plus Director-approved
    // release deferral markers for the five concession-encoded gates.
    // Dissolution trigger: R3 closes the named T-LensProducer-Retirement /
    // T-PB-B bulk-migration lanes and this R1-only acceptance wrapper retires.
    "src/v3/compiler/tests/integration/r1_release_acceptance_test.rs",
    // R2 B5: Loop construction-closure structural gate (Tier 2 §5).
    "src/v3/compiler/tests/integration/r2_b5_loop_construction_closure_test.rs",
    // R3 §1.4 Class 2 / §1.8 row #61
    // (`substrate_gap_function_valued_data_closed`): hand-Rust executable
    // receipt for the narrowed gap-test ratified in Q-Class-2-Chain-Break
    // option (a) and dispatched by
    // `docs/briefs/r3-substrate-s1-gap-test-representative-worker.md`.
    // P5 receipt: explicit deferral cites ROADMAP.md post-merge debt F8
    // (`SymbolicCost` first-class `Semiring<SymbolicCost>` witness; function-
    // valued data prerequisite) plus docs/r3-program-plan.md §1.8 row #61.
    // This bounded host-side harness asserts "function-valued data is first-
    // class" through public evaluator consumption; production code removes an
    // opaque data-body scaffold and routes through existing substrate `Arrow`
    // / `Callable`. Dissolves when §1.8 row #61 can be expressed as a `.dag`
    // TestClaim over evaluator output without direct Rust DAG inspection.
    "src/v3/compiler/tests/integration/r3_class_2_function_valued_data_test.rs",
    // R3 T-Free-Consequences first batch: hand-Rust driver for five
    // author-now/fire-later `BinaryDimensionReportEquals` TestClaims.
    // Dissolves when generic DimensionReport<C> evaluation can execute
    // the claims without a host-side integration harness.
    "src/v3/compiler/tests/integration/r3_free_consequences_first_batch_test.rs",
    // R3 T-Free-Consequences second batch: hand-Rust driver for five
    // author-now/fire-later TestClaims over ordinary-lens loop parallelism and
    // `BinaryDimensionReportEquals` cross-target cost optimization.
    // Dissolves when generic runner coverage can execute the claims without a
    // host-side integration harness.
    "src/v3/compiler/tests/integration/r3_free_consequences_second_batch_test.rs",
    // R3 gate #60 Phase 2.1 (`substrate_gap_parser_grammar_closed` parser slice): hermetic
    // parse + lower receipts for angle-bracket width nat (`Int<64>`) surface; Self-Generation-0 P5 receipt.
    "src/v3/compiler/tests/integration/r3_gate_60_phase2_width_nat_parser_test.rs",
    // R3 gate #62 `substrate_gap_file_ingestion_closed` negative-bridge audit
    // (supporting evidence; NOT a §Acceptance PASSING receipt — operator BLOCKING
    // 2026-05-14T19:13:37Z held PASSING flip pending a positive
    // ingestion-via-`FileAttachment` `.dag` demonstration). Pairs with the carrier
    // ratchet `file_attachment_substrate_carrier_test.rs` + `gate_62_file_attachment_demo_record`
    // (PR #2823) and the `FileAttachment` Refined-B-1 carrier in `src/v3/std/timing_lens.dag`.
    // Hand-Rust because the predicate is over the workspace `.dag`/`.v3` file tree
    // (filesystem walk + read) with comments / string literals stripped from each
    // program body, distinct from grep-on-doc-comment textual-enforcement per
    // `feedback_no_textual_enforcement_bridges`. T-PB-B deferral lane
    // (`pb_rust_tests_outside_residual_zero`); dissolves when a `.dag` `TestClaim` /
    // PB-B-1 runner can assert the file-tree audit fail-closed without a host-side
    // filesystem walker, or when the gate flips PASSING via a positive ingestion
    // demonstration and the carrier-reachability ratchets alone carry the audit.
    "src/v3/compiler/tests/integration/r3_gate_62_file_ingestion_negative_bridge_audit_test.rs",
    // R3 gate #87 (`lens_cementing_test_discipline_complete` / issue #2609): Rust receipts
    // paired with `tests/dag/t_r3_gate_87_cementing_regen_*.dag` + `t_pb_b_1_dag_runner_test`
    // until strict modules can freeze full `LensOutputEquals` carriers (M1(2.8)).
    "src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs",
    // R3 gate #90 (`lens_enforcement_carrier_landed`): bootstrap pins per-lens
    // `LensEnforcement` / `EnforceableLens` substrate rows (T-LAS Slice B).
    "src/v3/compiler/tests/integration/r3_gate_90_lens_enforcement_carrier_landed_test.rs",
    // R3 gate #66 (`lens_producer_retirement_executable_witness`): focused receipt
    // that the `.dag` PB census claim executes through `TestRunner` and reports
    // the live lens-producer residual count while Row-4 / Item 4 retirement
    // preconditions remain open.
    "src/v3/compiler/tests/integration/r3_lens_producer_retirement_executable_witness_test.rs",
    // PATH X / Brief 3 + ROADMAP `char_in_class` interpreter parity row (ASCII codegen vs evaluator).
    "src/v3/compiler/tests/integration/r3_path_b_brief3_char_in_class_execution_test.rs",
    // R3 PB Row-4 corpus seeds (1)–(2): hand-Rust driver for author-now/fire-later
    // `DifferentialEquals(pb_runtime_evaluate, r2_evaluator_evaluate, …)` TestClaims.
    // Dissolves when Row-4 producers land and the runner can execute the PB-Runtime /
    // R2-Evaluator corpus comparison directly without this host-side harness.
    "src/v3/compiler/tests/integration/r3_pb_runtime_evaluator_corpus_seed_test.rs",
    // R3 gate #8 (`self_gen_non_test_zero`): host receipt proving the combined
    // `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS`
    // state-check executes through `.dag` `TestRunner` claims while the live
    // Self-Generation-0 residual counts remain nonzero.
    "src/v3/compiler/tests/integration/r3_self_gen_non_test_zero_test.rs",
    // R3 gate #64 substrate-plumbing receipt: hand-Rust driver for the
    // non-canonical `.dag` residual-census receipt until the canonical
    // PB-Runtime reflection consumer lands. P5 test-subset deferral:
    // ROADMAP.md § "Nine lanes" row `T-PB-B` and § "Lane acceptance — .dag
    // gates" row `T-PB-B` / `pb_rust_tests_outside_residual_zero`; dissolves
    // when the generic TestClaim runner can execute the receipt without this
    // host-side harness.
    "src/v3/compiler/tests/integration/r3_substrate_gap_reflection_closure_test.rs",
    // R3 gate #71 (`v3_self_host_demonstration`): `.dag` + `CARGO_BIN_EXE` splice for
    // `ExecuteCommand(self_host_fixed_point, [--r3-gate-71-demonstration], 0)` — strict DB-8 slice
    // (non-zero unless `compiler.dag` parses + `fixed_point_diff` ok). Unignored compile-only smoke
    // + ignored end-to-end until v3 parses `compiler.dag` (T-FixedPoint promotion).
    // Explicit P5 receipt (INVARIANTS.md P5 per-PR gate): net +1 Self-Generation-0 integration path;
    // dissolution / deferral naming — ROADMAP.md § "Nine lanes" row `T-PB-B` /
    // `pb_rust_tests_outside_residual_zero` and `docs/r3-program-plan.md` §1.8 gate #71 /
    // `docs/r3-structure.md` §T-V2-Retirement; harness retires when equivalent obligations run as
    // `.dag` data without this Rust splice or T-V2-Retirement closure ends the receipt class.
    "src/v3/compiler/tests/integration/r3_v3_self_host_demonstration_dag_test.rs",
    // R3 L4/L7/L5 skeleton + L7 enum-backed algebra-law matrix: hand-Rust receipt that Lane 1
    // `DifferentialEquals` emit/eval pairing, Lane 1 `AlgebraicLaw` (`Associativity` /
    // `Commutativity` / `Identity`) operational witnesses, and the T-V-L5-Corpus seed
    // `ForAllTargets` Rust/Python/Go observation row Pass on honest Int rows only (trimmed matrix —
    // not ROADMAP exhaustive L7/L5). Matrix rows pin enum-backed law receipts without adding
    // missing-law variants. Explicit P5 deferral: this test entry belongs to ROADMAP.md § "Nine
    // lanes" row `T-PB-B` and § "Lane acceptance — .dag gates" row `T-PB-B` /
    // `pb_rust_tests_outside_residual_zero`; R3 tracks the same test-residual outcome as
    // `docs/r3-structure.md` § T-Tests-As-Data-Completeness / gate #84. It dissolves when the
    // generic TestClaim runner can execute these claims directly without this host-side harness.
    // Retirement must also fold the L5 program-text bridge
    // (`fixtures/r3_l5_corpus/*.v3` vs embedded `TestClaim.source` — byte equality ratchet in
    // `tests/boundary/l5_cross_target_consistency.rs`).
    "src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs",
    "src/v3/compiler/tests/integration/self_gen0_census_test.rs",
    "src/v3/compiler/tests/integration/self_gen1_tokenize_authority_test.rs",
    "src/v3/compiler/tests/integration/self_gen2_parse_authority_test.rs",
    "src/v3/compiler/tests/integration/self_gen2c1_parse_tables_authority_test.rs",
    "src/v3/compiler/tests/integration/self_gen2c5_soft_keyword_ident_test.rs",
    "src/v3/compiler/tests/integration/self_gen3_lower_parse_surface_stack_test.rs",
    "src/v3/compiler/tests/integration/self_gen3_surface_reflection_consumer_test.rs",
    "src/v3/compiler/tests/integration/self_gen6_hand_authored_census_test.rs",
    "src/v3/compiler/tests/integration/self_gen7_prep_variant_payload_freshness_test.rs",
    "src/v3/compiler/tests/integration/shape_a_target_source_filtering_authority_test.rs",
    // R3 §1.8 gate #40 (`symbolic_cost_expr_equals_executable`,
    // T-CostLens-Composition): mechanical ratchet pinning the executable
    // dispatch arm + evaluator wiring in `test_runner.rs`, so accidental
    // retirement back to the `NotYetImplemented` shell trips here. Wider
    // pass/fail-closed receipts live in `m1_5_verification_test.rs`.
    "src/v3/compiler/tests/integration/symbolic_cost_expr_equals_executable_ratchet_test.rs",
    // §1.8 gate #106 (`show_correct_code_diagnostic_coverage`): structural bootstrap locks on
    // `Correction` / substrate `Diagnostic` + one live-correction roundtrip anchor (`compile_to_dag`
    // → `apply_correction_and_reparse` → clean recompile). **P5 receipt:** matching INVARIANTS.md
    // Self-Generation-0 integration-test table row + this census literal land in the same PR — see row for
    // `t_gate_106_show_correct_code_diagnostic_coverage_test.rs`.
    "src/v3/compiler/tests/integration/t_gate_106_show_correct_code_diagnostic_coverage_test.rs",
    // §1.8 gate #58 (`apply_lens_self_application_demonstrated`): Rust integration asserts the
    // PB-1 `generated_full_bootstrap_dag()` snapshot carries the std witness + zero bootstrap
    // diagnostics (timing `EnforcedApplication` row in `t_ci_workflow_as_data_demo.dag`).
    //
    // **P5 receipt (INVARIANTS.md §P5 per-PR gate — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`):**
    // explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
    // `pb_rust_tests_outside_residual_zero` (tests-as-data / Pure Bootstrap test floor), same
    // structural class as co-listed `t_ci_workflow_as_data_demo_test.rs` and
    // `t_las_complexity_contract_compile_error_test.rs`: the obligation is still discharged via a
    // hand-maintained Rust harness until the generic `.dag` `TestClaim` runner can assert the same
    // bootstrap facts without this file. Lane context: **T-Lens-Self-Application** /
    // `apply_lens_self_application_demonstrated` (`docs/r3-structure.md`, `docs/r3-program-plan.md`
    // §1.8 gate #58). Dissolution: delete this path from the census when the receipt ports to a
    // `.dag` TestClaim (or a generated test) with no remaining Self-Generation-0 hand-authored test delta.
    "src/v3/compiler/tests/integration/t_gate_58_apply_lens_self_application_test.rs",
    "src/v3/compiler/tests/integration/t_impossiblebugs_unenumerated_effects_test.rs",
    "src/v3/compiler/tests/integration/t_las_complexity_contract_compile_error_test.rs",
    "src/v3/compiler/tests/integration/t_las_crdt_cost_basis_demo_test.rs",
    // R3 §1.8 gate #95 (`opt_in_iteration_parallelism_via_lens_application_demonstrated`).
    "src/v3/compiler/tests/integration/t_las_parallelism_iteration_gate95_demo_test.rs",
    // §1.8 gate #88 (`lens_application_carrier_landed`): bootstrap field / arity locks for
    // `EnforcedApplication` + `IntrospectApplication` in `src/v3/std/lens_application.dag`.
    "src/v3/compiler/tests/integration/t_lens_application_carrier_test.rs",
    // T-PB-B-1 `tests/dag` runner table; gate #74 + #87 cementing regen suites; R3 Cluster M #84
    // R1C-D/E runner receipts (co-located harness).
    //
    // **P5(b) / Self-Generation-0 accounting (#2715 pilot — not gate #84 closure):** the merge-visible
    // receipt is **−3** paths removed from this list (deleted `r1c_*_gates*_test.rs` shims).
    // R1C-D/E **predicates** live in `.dag` (`tests/dag/t_r1c_d_pb_census_gates.dag`,
    // `r1c_e_emit_gates*.template.dag`); Rust here is runner-only (`compile_to_dag` +
    // `TestRunner`), same structural class as gate #74 — consolidation must not be read as
    // Pure Bootstrap / T-PB-B "zero hand-maintained Rust" progress. Gate #84 /
    // `every_rust_test_ports_to_dag_or_generated` dissolution stays under ROADMAP T-PB-B +
    // `docs/r3-structure.md` § T-Tests-As-Data-Completeness until `EXPECTED_HAND_AUTHORED_TEST`
    // reaches zero.
    "src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs",
    // TC1 substrate lens eta-equivalence (deferred / R2 research): integration for
    // `SubstrateResearchDeferredClaim` + `tc1_substrate_lens_eta_equivalence_deferred.dag`.
    // Self-Generation-0 path ratchet: Director sign-off (gunb-ai/gunbc#1130, comment 4341571168;
    // direction ratified for #1179, comment 4341788769; mechanical checklist c4341800724;
    // cycle-5 merge hygiene gunb-ai/gunbc#1142 c4341940508).
    "src/v3/compiler/tests/integration/tc1_substrate_lens_eta_equivalence_deferred_test.rs",
    // TC1 V1 strict-fire — §1.8 gate #11 (`tc1_eta_equivalence_executable`); Q-PAFS Path A
    // (E6-G1.a static representative) per Director ratification cascade 2026-05-06/07.
    "src/v3/compiler/tests/integration/tc1_substrate_lens_eta_equivalence_strict_fire_test.rs",
    // TC2 strict-fire — §1.8 gate #12 (`tc2_church_rosser_executable`); Church-Rosser /
    // strategy-order `BinaryDimensionReportEquals` pairing per `r3-v-pattern-a-tc2-v1-worker.md`.
    "src/v3/compiler/tests/integration/tc2_church_rosser_strict_fire_test.rs",
    "src/v3/compiler/tests/integration/tc3_strong_normalization_deferred_test.rs",
    // TC3 strict-fire — §1.8 gate #13 (`tc3_pattern_a_second_mover_executable`); strong-
    // normalization / Pattern-A second-mover `BinaryDimensionReportEquals` pairing per
    // `r3-v-pattern-a-tc3-v1-worker.md` (two-stage gate; PASSING gated on T-FixedPoint stage (b)).
    "src/v3/compiler/tests/integration/tc3_strong_normalization_strict_fire_test.rs",
    "src/v3/compiler/tests/integration/test_runner_test.rs",
    "src/v3/compiler/tests/integration/thesis_parallelism_test.rs",
    "src/v3/compiler/tests/integration/thesis_validation_test.rs",
    "src/v3/compiler/tests/integration/timing_lens_substrate_carrier_test.rs",
    // R3 T-V2-Retirement §1.8 gate #41 (`v2_oracle_no_remaining_test_consumers`): comment-aware
    // source ratchet — no `v2-compiler` crate references outside `src/v2/`.
    "src/v3/compiler/tests/integration/v2_oracle_no_remaining_test_consumers_test.rs",
    // White-box sweep (#4511 + B7 closeout): all v4_*_dag_smoke_test.rs retired — last row
    // `v4_bin_main_dag_smoke_test.rs` deleted (B-class decl-shape/source-grep duplicate of
    // t15 harness; parse rides `claim_t15_self_host_fixed_point.dag` import of v4.bin.main).
    // T-22: eval dispatch runner fail-closed receipts (pairs with `emit_host_eval.rs` NON_TEST row).
    // **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — Self-Generation-0 `EXPECTED_HAND_AUTHORED_TEST`):**
    // explicit deferral ROADMAP `T-PB-B` / `pb_rust_tests_outside_residual_zero` plus
    // `emit_host.dag` T-22 rust eval intercept; dissolves when `.dag` TestClaim execution
    // replaces host-runner receipts.
    "src/v3/compiler/tests/integration/v4_emit_host_eval_dispatch_test.rs",
    // W2 / T-38 rung-4 host harness: behavior-driven `tools/emit_host_runner` + `.dag` surface
    // needles (`emit_host.dag`, `host_run.dag`, `test_claim_falsification.dag`).
    // **PR #4063 W3.4 (+0 paths):** extends harness with python transport + rung-6 additive-Monoid
    // MVP-2 bridge proofs (law×target); `emit_host_bridge.rs` python row (+0 NON_TEST — #4047).
    // **PR #4167 Python L1/L2 (+0 paths):** extends same harness with rung-5 python law roster
    // transport, worksheet-B falsification probes, L1 claim parse surface; pairs with
    // `scripts/v4-nat-semiring-python-runtime-gate.sh` chained from acceptance gate.
    // **PR #4222 Python L1 fixture coverage (+0 paths):** same-path assertion-list expansion
    // for `l1_python_runtime.dag` coverage + six per-law runtime claim rows.
    // **PR #4229 Go L1 (+0 paths):** extends same harness with `go_l1_nat_semiring_l1_compiler_slice`
    // compiler-slice claim parse surface; pairs with
    // `scripts/v4-nat-semiring-go-compiler-slice-gate.sh` chained from acceptance gate.
    // **PR #4285 Go L1 strict setup (+0 paths):** same-path assertion-list expansion verifies
    // the parent acceptance gate fails closed when
    // `V4_NAT_SEMIRING_GO_COMPILER_SLICE_GATE_STRICT=1` and `go`/`gofmt` are missing.
    // Dissolve with the same generated/TestClaim host setup receipt as #4229.
    // Self-Generation-0 + INVARIANTS §P5(b) receipt (PR body Mechanism (b) block). Deferral:
    // `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`.
    "src/v3/compiler/tests/integration/v4_emit_host_harness_test.rs",
    // T-4.7 React framework substrate single-file smoke deleted: the live v4 full-tree
    // rust emit probe now fail-closes on `gunbc compile --source-root src/v4 --target rust`
    // with a 0-diagnostic receipt, covering `src/v4/extdeps/frameworks/react.dag` without
    // a weaker v3 `compile_to_dag` oracle.
    // P9 single-owner: corpus scan for `fn llvm_instruction_cost` under src/v4/ (replaces dissolved
    // v4_lens_cost_dag_smoke_test.rs ratchet). Self-Generation-0 + INVARIANTS §P5(b) receipt.
    "src/v3/compiler/tests/integration/v4_p9_llvm_instruction_cost_single_owner_test.rs",
    // W3 B-class delete (operator 2026-06-07: declaration-shape tests are dual-representations of
    // our own std structure — change-detectors with zero external-oracle coverage; correctness-by-
    // construction, not 2FA-for-code). The three v4_std_{grounding,model_core,target_realization}_dag
    // smoke files (44 parse-surface/source-grep receipts) are DELETED, not deferred. Their two
    // behavioral receipts are migrated to mutation-proven .dag claim-run witnesses via glob_discovery:
    //   - src/v4/test/claim/std_model_core/bool_fact_lookup.dag (also retires the hand-Rust MIRROR
    //     helper bool_fact_axis_dispatch — a tautology that re-stated the fold in Rust); and
    //   - src/v4/test/claim/std_grounding/terminal_gate.dag.
    // No INVARIANTS.md per-file rows exist for these (the §P5(b) references were the general gate).
    // Host-test preservation justification (W1-W4 qualifying bar — provably runtime-inexpressible):
    // non-behavioral: asserts type-ABSENCE of ByteString/FileBody/FileContent/TargetSource in
    // text.dag — no runtime witness can express type-non-existence. (Behavioral content folded
    // to discriminating .dag witnesses in src/v4/test/claim/std_text/carrier_claims.dag.)
    // Dissolves when a .dag/host mechanism asserts a module's declared-type set as data.
    "src/v3/compiler/tests/integration/v4_std_text_boundary_carrier_guard_test.rs",
    // T-15: bin/main.dag execution + bootstrap fixpt stage1==stage2 harness (`t_15_self_host_fixed_point`).
    "src/v3/compiler/tests/integration/v4_t15_self_host_fixed_point_harness_test.rs",
    // T-19/T-20 closeout ratchets over v4 testgen + bootstrap-infra parse surfaces.
    // PR #4295 (+0 paths): `check_t19_testgen_activation` same-path expansion —
    // Rust migration of `scripts/check_t19_testgen_activation.py` (deleted #4252).
    // PR #4335 (+0 paths): `rr_a_step2_bootstrap_evaluator_corpus_harness_entry` RR-A §5.2
    // bootstrap harness parse-surface ratchet (ROADMAP.md:43,63 T-PB-B deferral).
    // Self-Generation-0 + INVARIANTS §P5(b) receipt; dissolves when the same checks are `.dag`
    // TestClaims or generated harness coverage (ROADMAP.md T-PB-B row).
    "src/v3/compiler/tests/integration/v4_test_bootstrap_infra_closeout_test.rs",
    // §1.8 gate #96 (`value_body_substrate_mirror_isomorphism_executable`):
    // CI-visible generated Rust `ValueBody` mirror vs `substrate.dag`
    // constructor isomorphism. Dissolves when `ValueBody` no longer has a
    // Rust mirror or when this assertion is expressible as a `.dag` TestClaim.
    "src/v3/compiler/tests/integration/value_body_substrate_mirror_isomorphism_test.rs",
    // R2-Substrate Prereq-3a (`workflow_root_port` accessor + `WorkflowRoot`
    // sum) per merged audit `docs/design-lens-fold-prerequisites.md`.
    // Director-locked α: last topological `Bind`. Three integration
    // claims exercise the `SingleRoot` single/multi cases and the
    // unreachable-under-α ambiguous drift trigger; the zero-Bind
    // `NoRoot` case lives as a unit test in `dag.rs` (crate-private
    // `Dag::empty` constructor). Dispatch (#1130) accepted hand-Rust
    // acceptance over real `compile_to_dag` fixtures; dissolves into
    // `.dag` `TestClaim` form when testgen covers compile-and-fold
    // structural assertions.
    "src/v3/compiler/tests/integration/workflow_root_port_test.rs",
    // R3 §1.8 gate #53 `workflow_substrate_carriers_landed` structural
    // ratchet: locks Slice 1 β-ratified carriers (`WorkflowSecret`,
    // `SecretScope`, `CronSchedule`, `CronField`) against the full
    // bootstrap Dag. Sibling shape to gate #62
    // `file_attachment_substrate_carrier_test.rs`. Dissolves into
    // `.dag` `TestClaim` form when testgen covers structural-shape
    // assertions over substrate carriers.
    "src/v3/compiler/tests/integration/workflow_substrate_carriers_test.rs",
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
    "src/v3/compiler/src/lens_testgen_body.txt",
];

// Non-`.rs` files under `src/v3/compiler/` whose content is produced
// by a named generator (an `#[ignore]`'d refresh test, a `regen_*`
// binary, etc.) rather than hand-edited. Listed explicitly so the
// fragments walker can partition without content sniffing (which the
// `self_gen0_generated_partition_is_producer_owned` probe forbids for
// `.rs`; the same discipline applies here).
const EXPECTED_GENERATED_FRAGMENTS: &[&str] = &[
    // Produced by `cargo test refresh_handwritten_parse_snapshot_manifest -- --ignored`.
    "src/v3/compiler/tests/integration/parse_corpus_manifest.txt",
];

pub(crate) fn expected_hand_authored_non_test_count() -> usize {
    EXPECTED_HAND_AUTHORED_NON_TEST.len()
}

pub(crate) fn expected_hand_authored_fragments_count() -> usize {
    EXPECTED_HAND_AUTHORED_FRAGMENTS.len()
}

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

// R1C-D `pb_test_file_generated_from_dag`: `GeneratedFromDag` manifest vs
// `GENERATED_FILES` set-equality (both `PendingFact` and `ResolvedFact` arms)
// is enforced in `test_runner::eval_generated_from_dag_shape` when the `.dag`
// suite runs (`t_pb_b_1_dag_runner_test::r1c_d_pb_census_gates_suite_evaluates_through_runner`).
// Do not duplicate that obligation with a string-scrape test here (P5 + sum-shape drift).

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
fn self_gen0_v3_hand_authored_census() {
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
        "Self-Generation-0 census drift: observed hand-authored set does not match \
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
             src/v3/compiler/tests/integration/self_gen0_census_test.rs.\n",
        );
    }
    panic!("{msg}");
}

#[test]
fn self_gen0_expected_list_is_sorted_and_unique() {
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
fn self_gen0_expected_rs_entries_match_test_partition() {
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
fn self_gen0_v3_non_test_hand_authored_subratchet() {
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
        "T-PB-A non-test Self-Generation-0 sub-ratchet drifted. Retirements should be removed \
         from EXPECTED_HAND_AUTHORED_NON_TEST; new non-test hand-Rust needs director \
         sign-off."
    );
}

#[test]
fn self_gen0_v3_test_hand_authored_subratchet() {
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
        "T-PB-B test Self-Generation-0 sub-ratchet drifted. Retirements should be removed from \
         EXPECTED_HAND_AUTHORED_TEST; new Rust-authored tests must match the TESTING.md \
         residual or wait for the testgen path."
    );
}

/// ctrl#1467 §6.4 — the `v4_*_dag_smoke_test.rs` parse-surface subset is **closed to
/// growth**. These hand-authored Rust tests `parse_for_test` a v4 model file and assert
/// structural shape against the v3 compiler's internal parse surface — a layering/opacity
/// inversion (see `gunbc-planning/v4-testclaim-route-through-v2-not-v3-2026-06-05.md`). New
/// v4 coverage must land as a `.dag` `TestClaim` witness under `src/v4/test/claim/`, run
/// through **v2** (`dag run --claim-run`), **not** as a new `EXPECTED_HAND_AUTHORED_TEST`
/// entry. This subset may only **shrink** as smokes are re-homed as witnesses; it must never
/// grow. Lowering the ceiling on retirement is the normal path; raising it is the regression
/// this guard fails closed on.
#[test]
fn v4_parse_surface_smoke_roster_is_closed_to_growth() {
    // Pinned to the live count (ctrl#1467 §6.4). Retirements lower this; nothing raises it.
    // 18 (ratification) → 0 after W-wave fold-deletes + B7 closeout: #4511 broader-corpus
    // sweep, W3 #4512 std trio, React smoke, and final bin_main B-class delete. Subset is
    // empty; a new v4 parse-surface smoke must instead land as a .dag claim-run witness.
    const V4_DAG_SMOKE_CEILING: usize = 0;
    let v4_dag_smokes: Vec<&str> = EXPECTED_HAND_AUTHORED_TEST
        .iter()
        .copied()
        .filter(|p| {
            p.starts_with("src/v3/compiler/tests/integration/v4_")
                && p.ends_with("_dag_smoke_test.rs")
        })
        .collect();
    assert!(
        v4_dag_smokes.len() <= V4_DAG_SMOKE_CEILING,
        "v4_*_dag_smoke_test.rs roster grew to {} (ceiling {V4_DAG_SMOKE_CEILING}). \
         ctrl#1467 §6.4: new v4 coverage must be a `.dag` `TestClaim` witness under \
         src/v4/test/claim/ run through v2 — not a new hand-authored v3 parse-surface smoke. \
         This subset is closed to growth; it may only shrink as smokes are re-homed as witnesses.\n\
         Offending roster: {v4_dag_smokes:#?}",
        v4_dag_smokes.len(),
    );
}

#[test]
fn self_gen0_tests_as_data_migration_audit_classifies_test_ratchet() {
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
fn self_gen0_v3_non_test_fragment_subratchet() {
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
        "T-PB-A non-test Self-Generation-0 fragment sub-ratchet drifted. Retirements should be \
         removed from EXPECTED_HAND_AUTHORED_FRAGMENTS; new scaffold fragments must \
         name a dissolution trigger."
    );
}

#[test]
fn self_gen0_expected_fragment_lists_are_sorted_and_unique() {
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
fn self_gen0_v3_hand_authored_txt_fragments() {
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
        "Self-Generation-0 fragment census drift: `.txt` files under src/v3/compiler/ \
         do not match EXPECTED_HAND_AUTHORED_FRAGMENTS ∪ EXPECTED_GENERATED_FRAGMENTS.\n\n",
    );
    if !added.is_empty() {
        msg.push_str(
            "New scaffold fragment(s) (a `.txt` extension does NOT exempt it from Self-Generation-0):\n",
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
fn self_gen0_generated_partition_is_producer_owned() {
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
    // inside `src/v3/compiler/`. The live `self_gen0_v3_hand_authored_census`
    // walker never sees it, so the tests are safe to run in parallel.
    let tmp = std::env::temp_dir().join(format!(
        "self_gen0_soundness_probe_{}_{}",
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
fn self_gen0_every_generated_file_is_present_on_disk() {
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
fn self_gen0_stage0_copy_command_excludes_hand_maintained_root_files() {
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
fn self_gen0_stage0_hand_maintained_src_covers_emit_subtree_companions() {
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
