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

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use v3_compiler::generated_files::GENERATED_FILES;

// Relative to workspace root; mirrors the single census root
// informally named in `dsl/gunbc/compiler.dag`.
const CENSUS_ROOT: &str = "src/v3/compiler";

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
    // Intentionally hand-authored: it measures live public mirror
    // entrypoints (`merge_evidence`, `positive_descent_count`,
    // `lower_call_pattern`, `type_iteration_dimension`,
    // `lane2_workflow_idempotency_report`) before T-Tier3-Dissolution
    // retires them — generated output cannot exist yet because the
    // measurement target is the not-yet-dissolved Rust code.
    // Dissolution trigger: deletes alongside the mirror-dissolution PRs
    // per parent brief §"Phase 1 deliverables" — the bench harness has
    // no role post-Phase-1; only the frozen `tier3_baseline.json` data
    // survives.
    "src/v3/compiler/benches/tier3_mirror_perf.rs",
    "src/v3/compiler/build.rs",
    // R3 row 85 / PB #1560 Gap 4 build-step shim: invokes
    // `pb_method_template_projection_dag_emit` to materialize the
    // ephemeral v2 source-root module consumed during stage0
    // regeneration. Dissolution trigger: delete with the v2-retirement
    // build-step consumer path once legacy v2 method-template reads are
    // fully retired.
    "src/v3/compiler/src/bin/emit_method_template_projection.rs",
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
    "src/v3/compiler/src/infer.rs",
    "src/v3/compiler/src/int_literal_ranges.rs",
    "src/v3/compiler/src/lens_apply.rs",
    // T-PB-A: `lens_depth.rs` retired — unused observational lens (no in-tree consumer).
    "src/v3/compiler/src/lens_testgen.rs",
    "src/v3/compiler/src/lib.rs",
    "src/v3/compiler/src/lower.rs",
    // R3 row 85 / PB #1560 Gap 4: target-keyed projection of the
    // `MethodTemplateContract` rows from the full bootstrap `Dag` for
    // PB-zero / v2-retirement consumers (decision in
    // `docs/decisions/r3-row85-method-template-read-surface.md`).
    "src/v3/compiler/src/pb_method_template_projection.rs",
    // R3 row 85 / PB #1560 Gap 4 build-step: producer that writes the
    // canonical `MethodTemplateContract` projection to a build-time-
    // ephemeral `.dag` dependency root for v2 consumption via the
    // ephemeral source-root mechanism from PR #1575.
    "src/v3/compiler/src/pb_method_template_projection_dag_emit.rs",
    "src/v3/compiler/src/pipeline_authority.rs",
    "src/v3/compiler/src/post_emit_verifier.rs",
    // PB-1 Item 5: host mirror of `dsl/std/process.dag` `ProcessExit` for emitted bin shims.
    "src/v3/compiler/src/process_exit.rs",
    // R1C-E (T-Emit `.dag` `TestClaim` wrappers): shared `check_*` API the host
    // `#[test]` harness and `r1c_e_emit_gates` `bin` both call. Single source of
    // truth for the emit-gate assertions; scaffold until R1 close dissolves it.
    "src/v3/compiler/src/r1c_e_gates.rs",
    "src/v3/compiler/src/regen_bootstrap_emit.rs",
    "src/v3/compiler/src/regen_parse_emit.rs",
    "src/v3/compiler/src/regen_parse_tables_emit.rs",
    "src/v3/compiler/src/self_host_receipt_p0.rs",
    "src/v3/compiler/src/test_runner.rs",
    "src/v3/compiler/src/workflow_idempotency.rs",
    "src/v3/compiler/src/workflow_parallelism.rs",
];

// All test .rs files under `src/v3/compiler` that are currently
// hand-authored. Sorted; one path per line, relative to the
// workspace root. T-PB-B owns shrinking this subset toward the
// TESTING.md §"Post-R2 shape" residual. T-PB-A reductions must not
// rely on this list moving.
// Slice 1 census reconciliation (2026-05-02): this list matches the current
// tree exactly; no additions or removals were needed.
const EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[
    "src/v3/compiler/tests/boundary/m1_3_emit_go_test.rs",
    "src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs",
    "src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs",
    "src/v3/compiler/tests/boundary/m1_5_emit_omni_demo_test.rs",
    "src/v3/compiler/tests/boundary/m2_emit_multi_field_struct_variant_test.rs",
    "src/v3/compiler/tests/determinism_test.rs",
    "src/v3/compiler/tests/integration.rs",
    "src/v3/compiler/tests/integration/anthropic_messages_callable_test.rs",
    // T-Ground services.dag PR-β anthropic_operations Phase 1 pilot
    // (#1252). Hand-authored ratchet entry added per SG-0 census discipline.
    "src/v3/compiler/tests/integration/anthropic_operations_test.rs",
    "src/v3/compiler/tests/integration/anthropic_schema_lockstep_test.rs",
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
    "src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs",
    "src/v3/compiler/tests/integration/common/budgeted.rs",
    "src/v3/compiler/tests/integration/common/cached_compile.rs",
    "src/v3/compiler/tests/integration/common/determinism_fixtures.rs",
    // E6-G1.a Option 3 static lens mechanism (#1853 worker brief + #1857):
    // `find_list_empty_constructor_tag` helper for opaque-`Dag` harness tags
    // (P1/P5 compile-time brief receipts live in `e6_g1a_option3_static_lens_test.rs`).
    "src/v3/compiler/tests/integration/common/list_variant_tags.rs",
    "src/v3/compiler/tests/integration/common/mod.rs",
    "src/v3/compiler/tests/integration/common/r1_gates_bridge.rs",
    "src/v3/compiler/tests/integration/common/substrate_receipts.rs",
    // R3 L6 carrier slice (PR #1842; Measure-carrier precedent at #1819,
    // Director Option 2 RATIFIED at
    // gunbc#828 #issuecomment-4377533390): slice-active ratchet for
    // `cross_target_coverage.dag` (six type declarations exist;
    // `emission_path_projections == []`). Stays hand-Rust alongside
    // `method_template_contract_test.rs` until testgen covers
    // reflected-Dag structural assertions over std/ row authorities.
    "src/v3/compiler/tests/integration/cross_target_coverage_carrier_test.rs",
    "src/v3/compiler/tests/integration/e_i_lane_induction_preflight_test.rs",
    // E6-G1.a Option 3 — static `Lens<Int>` + `mini_report` mechanism demonstration
    // (Director #1853 brief; witness-flow + TESTING.md split + `include_str!` brief
    // receipts per #1857). SG-0 ratchet: new hand-authored integration test.
    "src/v3/compiler/tests/integration/e6_g1a_option3_static_lens_test.rs",
    // T-Ground-Engine Phase-1 loader-close (PR #776, Director-approved
    // Path 2): hand-Rust integration test pinning
    // `Dag::rust_pilot_primitives()` type-structure walk + the
    // `ValueBody::Unparsed` boundary that flips when R2 T-Substrate's
    // 4th sub-lane lands top-level `ValueBody::List`/aggregate.
    // Dissolves into testgen authority when the testgen path covers
    // the dsl/extdeps loader surface.
    "src/v3/compiler/tests/integration/extdeps_rust_primitives_loader_test.rs",
    "src/v3/compiler/tests/integration/four_fixture_regression_test.rs",
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
    "src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs",
    "src/v3/compiler/tests/integration/pb1_bootstrap_full_snapshot_test.rs",
    // R3 row 85 / PB #1560 Gap 4: focused acceptance for the
    // `pb_method_template_projection` consumer hook. Stays hand-Rust
    // alongside `method_template_contract_test.rs` until testgen covers
    // reflected-Dag structural assertions over std/ row authorities.
    // R3 row 85 / PB #1560 Gap 4 build-step: focused acceptance for the
    // `pb_method_template_projection_dag_emit` producer (writes the
    // ephemeral `.dag`). Stays hand-Rust alongside the projection-side
    // tests until testgen covers reflected-Dag structural assertions
    // over std/ row authorities.
    "src/v3/compiler/tests/integration/pb_method_template_projection_dag_emit_test.rs",
    "src/v3/compiler/tests/integration/pb_method_template_projection_test.rs",
    "src/v3/compiler/tests/integration/pipe_desugar.rs",
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
    // R1C-D (PB census `.dag` `TestClaim` wrappers): runner-side receipt
    // for the six PB census gates in `tests/fixtures/r1_pb_census_gates.dag`.
    // Asserts `TestRunner` dispatches each PB census predicate to a wired
    // `eval_*_shape` slice (no `NotYetImplemented`) and that results are
    // structural `Pass`/`Fail` against the live SG-0 census authority.
    // Same residual class as the R1C-E driver below — paired hand-Rust
    // shim until R1 close dissolves the wrappers (D.5 / cascade-promotion
    // 0-floor work in the Pure Bootstrap to Zero program).
    "src/v3/compiler/tests/integration/r1c_d_pb_census_gates_test.rs",
    // R1C-E (T-Emit `.dag` `TestClaim` wrappers): integration-test driver
    // that splices `env!("CARGO_BIN_EXE_r1c_e_emit_gates")` into the
    // `tests/dag/r1c_e_emit_gates.template.dag` source and runs the suite
    // through `TestRunner`. Scaffold until R1 close dissolves the wrappers.
    "src/v3/compiler/tests/integration/r1c_e_emit_gates_dag_test.rs",
    "src/v3/compiler/tests/integration/r1c_e_emit_gates_omni_dag_test.rs",
    // R2 B5: Loop construction-closure structural gate (Tier 2 §5).
    "src/v3/compiler/tests/integration/r2_b5_loop_construction_closure_test.rs",
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
    // R3 L4/L7/L5 skeleton + L7 enum-backed algebra-law matrix: hand-Rust receipt that Lane 1
    // `DifferentialEquals` emit/eval pairing, Lane 2 `AlgebraicLaw::Identity`, and L5
    // `ForAllTargets` compile but defer as `NotYetImplemented`; matrix rows pin current
    // `Associativity` / `Commutativity` wired receipts plus `Identity` NYI receipts without adding
    // missing-law enum variants. Dissolves when `TestRunner` can evaluate these claims directly
    // without this host-side harness (same dissolution class as the R3 Free-Consequences batches).
    // Retirement must also fold the L5 program-text bridge (`fixtures/r3_l5_corpus/add_then_branch_seed.v3`
    // vs embedded `TestClaim.source` — byte equality ratchet lives only in this harness today).
    "src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs",
    "src/v3/compiler/tests/integration/services_carrier_shape_test.rs",
    "src/v3/compiler/tests/integration/sg0_census_test.rs",
    "src/v3/compiler/tests/integration/sg1_tokenize_authority_test.rs",
    "src/v3/compiler/tests/integration/sg2_parse_authority_test.rs",
    "src/v3/compiler/tests/integration/sg2c1_parse_tables_authority_test.rs",
    "src/v3/compiler/tests/integration/sg2c5_soft_keyword_ident_test.rs",
    "src/v3/compiler/tests/integration/sg3_lower_parse_surface_stack_test.rs",
    "src/v3/compiler/tests/integration/sg3_surface_reflection_consumer_test.rs",
    "src/v3/compiler/tests/integration/sg6_hand_authored_census_test.rs",
    "src/v3/compiler/tests/integration/sg7_prep_variant_payload_freshness_test.rs",
    "src/v3/compiler/tests/integration/shape_a_target_source_filtering_authority_test.rs",
    "src/v3/compiler/tests/integration/t_impossiblebugs_unenumerated_effects_test.rs",
    "src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs",
    "src/v3/compiler/tests/integration/t_pb_b_brief_d_fixture_smoke_test.rs",
    // TC1 substrate lens eta-equivalence (deferred / R2 research): integration for
    // `SubstrateResearchDeferredClaim` + `tc1_substrate_lens_eta_equivalence_deferred.dag`.
    // SG-0 path ratchet: Director sign-off (gunb-ai/gunbc#1130, comment 4341571168;
    // direction ratified for #1179, comment 4341788769; mechanical checklist c4341800724;
    // cycle-5 merge hygiene gunb-ai/gunbc#1142 c4341940508).
    "src/v3/compiler/tests/integration/tc1_substrate_lens_eta_equivalence_deferred_test.rs",
    "src/v3/compiler/tests/integration/tc3_strong_normalization_deferred_test.rs",
    "src/v3/compiler/tests/integration/test_runner_test.rs",
    "src/v3/compiler/tests/integration/thesis_parallelism_test.rs",
    "src/v3/compiler/tests/integration/thesis_validation_test.rs",
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
];

// Non-`.rs` scaffold fragments under `src/v3/compiler/` that are
// hand-authored and text-inlined into generated Rust (or otherwise
// dissolve when the corresponding `.dag` authority lands). The
// `.rs`-only census above cannot see these — a scaffold that renames
// itself `foo.txt` would silently escape the ratchet otherwise.
// Every entry here names a dissolution trigger in its own file header.
// Sorted; one path per line, relative to the workspace root.
const EXPECTED_HAND_AUTHORED_FRAGMENTS: &[&str] = &["src/v3/compiler/parse_parser_body.txt"];

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
