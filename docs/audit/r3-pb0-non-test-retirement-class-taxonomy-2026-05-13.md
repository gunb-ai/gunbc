# PB-0 — `EXPECTED_HAND_AUTHORED_NON_TEST` retirement-class taxonomy

**Authority:** R3 Debt-Paydown Mgr (`zesty-boar-261`) — Director dispatch **msg_84abadad-35fd-4c92-8db3-0e7737adfa4f** (operator-ratified); **scope correction** **msg_dda96d21-b937-448c-a72d-f7c8a44b691c** (PM awk-range verification **msg_bda6867a**).
**Procedure:** `docs/audit/r3-sg0-trajectory-tracker.md` §2 (`awk` window on `sg0_census_test.rs`).
**Snapshot:** `gunbc` @ `2b7241362` (`2b7241362a7f`).

## §0. Count discipline vs close-plan narrative

- **Live `EXPECTED_HAND_AUTHORED_NON_TEST` paths:** **55** (this table has one row per path) — **Gap 1 / Debt-Paydown lane** scope per Director **msg_dda96d21** (awk-range extraction; PM **msg_bda6867a**).
- **Live `EXPECTED_HAND_AUTHORED_TEST` paths:** **122** — **Gap 5 / Verification lane** (`bright-bee-903` / `still-moth-538`); **not** classified here.
- **Combined `NON_TEST` + `TEST`:** **177** (`55 + 122`) — matches close-plan Σ narrative **excluding** fragments.
- **Live Σ (`NON_TEST` + `TEST` + `FRAGMENTS`):** **179** (`55 + 122 + 2`).
- **Earlier relay inflation:** loose `grep -c` over the whole `sg0_census_test.rs` file inflated a **non-test-only** headline count; **authoritative** counts are always the **per-const `awk` windows** (tracker §2).

## §1. Classification legend

- **(a) RETIREABLE NOW** — no named structural blocker in §3; suitable for near-term cycle worker PRs under `docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md` (still subject to P5 / same-PR dissolution receipts).
- **(b) BLOCKED ON NAMED PREREQ** — cite §1.8 row, Gap id, or audit doc anchor.
- **(c) BLOCKED ON NEW CANVAS** — needs new or expanded Mgr canvas before census line can drop (or first-pass taxonomy bucket pending refinement).

## §2. Track A — Cycle-2 named scope (immediate dispatch)

Per Director Track A, the first cycle-2 worker batch **names** these **6** paths (also marked **(a)** in §3):

- `src/v3/compiler/src/cementing_dispatch.rs`
- `src/v3/compiler/src/gunbc_ci.rs`
- `src/v3/compiler/src/integration_rs_wiring_scan.rs`
- `src/v3/compiler/src/lens_declaration_apply.rs`
- `src/v3/compiler/src/lens_t_las_carrier.rs`
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`

## §3. Per-path taxonomy (all `NON_TEST` rows)

| Path | Class | Blocker / rationale |
|------|-------|---------------------|
| `src/v3/compiler/benches/tier3_mirror_perf.rs` | **(b)** | §1.8 #2 `tier3_computation_mirror_dissolved` PASSING + Evaluator/`std.computation`; see `docs/audit/r3-tier3-computation-mirror-consumer-prestage-2026-05-13.md`. |
| `src/v3/compiler/build.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/gunbc_ci.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/r1c_e_emit_gates.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/regen_bootstrap.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/regen_lens.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/regen_parse.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/regen_parse_tables.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/regen_tokenize.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/regen_v3.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bin/self_host_fixed_point.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bootstrap.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/bootstrap_regen_fresh.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/cementing_dispatch.rs` | **(a)** | Cycle-2 worker dispatch (Director msg_84abadad Track A); retire per `docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md` + gate #87 / T-WAD adjacent receipts. |
| `src/v3/compiler/src/complexity_lattice.rs` | **(b)** | Cost/complexity lattice host; couples to cementing + cost lenses — retire with owning `.dag` lens programs. |
| `src/v3/compiler/src/cost_basis_declaration.rs` | **(b)** | Cost basis declaration host; couples to cost-lens / cementing migration. |
| `src/v3/compiler/src/dag.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/dag/builder.rs` | **(b)** | DAG substrate modules; tied to Tier3 + effects + cardinality program rows — retire with owning substrate move, not isolated. |
| `src/v3/compiler/src/dag/cardinality_payload.rs` | **(b)** | DAG substrate modules; tied to Tier3 + effects + cardinality program rows — retire with owning substrate move, not isolated. |
| `src/v3/compiler/src/dag/effects.rs` | **(b)** | DAG substrate modules; tied to Tier3 + effects + cardinality program rows — retire with owning substrate move, not isolated. |
| `src/v3/compiler/src/dag/ports.rs` | **(b)** | DAG substrate modules; tied to Tier3 + effects + cardinality program rows — retire with owning substrate move, not isolated. |
| `src/v3/compiler/src/diagnostics.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/dimension.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/emit.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/emit/collection_ops_method_contract.rs` | **(b)** | Emit surface; Gap 13 / R2-Grounding + collection-ops identity gates per adjacent census comments. |
| `src/v3/compiler/src/emit/python_target.rs` | **(b)** | Emit surface; Gap 13 / R2-Grounding + collection-ops identity gates per adjacent census comments. |
| `src/v3/compiler/src/emit/rust_target.rs` | **(b)** | Emit surface; Gap 13 / R2-Grounding + collection-ops identity gates per adjacent census comments. |
| `src/v3/compiler/src/emit_rust.rs` | **(b)** | R1C-E / emit-rust host harness; shared with `r1c_e_emit_gates` bin — dissolve with R1 close / `.dag` TestClaim migration per census notes. |
| `src/v3/compiler/src/emit_rust_bin_shim.rs` | **(b)** | R1C-E / emit-rust host harness; shared with `r1c_e_emit_gates` bin — dissolve with R1 close / `.dag` TestClaim migration per census notes. |
| `src/v3/compiler/src/emit_rust_roundtrip_fixtures.rs` | **(b)** | R1C-E / emit-rust host harness; shared with `r1c_e_emit_gates` bin — dissolve with R1 close / `.dag` TestClaim migration per census notes. |
| `src/v3/compiler/src/enforced_lens_application.rs` | **(b)** | T-Lens-Self-Application enforcement host; ties gate #58 / timing lens program — not isolated drop. |
| `src/v3/compiler/src/gunbc_ci.rs` | **(a)** | Cycle-2 worker dispatch (Director msg_84abadad Track A); retire per `docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md` + gate #87 / T-WAD adjacent receipts. |
| `src/v3/compiler/src/infer.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/int_literal_ranges.rs` | **(b)** | Numeric / literal-range host; T-Numeric-Construction adjacent — retire with numeric program. |
| `src/v3/compiler/src/integration_rs_wiring_scan.rs` | **(a)** | Cycle-2 worker dispatch (Director msg_84abadad Track A); retire per `docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md` + gate #87 / T-WAD adjacent receipts. |
| `src/v3/compiler/src/lens_declaration_apply.rs` | **(a)** | Cycle-2 worker dispatch (Director msg_84abadad Track A); retire per `docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md` + gate #87 / T-WAD adjacent receipts. |
| `src/v3/compiler/src/lens_t_las_carrier.rs` | **(a)** | Cycle-2 worker dispatch (Director msg_84abadad Track A); retire per `docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md` + gate #87 / T-WAD adjacent receipts. |
| `src/v3/compiler/src/lib.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/lower.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/memory_peak_cost.rs` | **(b)** | Gate #94 cost-lens memory-peak authority; dissolve with cost-lens substrate program. |
| `src/v3/compiler/src/omni_shape_b_openapi.rs` | **(b)** | Shape B OpenAPI transitional host; dissolve when `.dag` owns projection end-to-end per census comment. |
| `src/v3/compiler/src/pb_method_template_projection.rs` | **(b)** | Row-85 method-template read surface; `docs/decisions/r3-row85-method-template-read-surface.md` — PB-zero consumer bundle. |
| `src/v3/compiler/src/pipeline_authority.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/post_emit_verifier.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/process_exit.rs` | **(b)** | PB-1 Item 5 host mirror of `dsl/std/process.dag` — dissolve with PB-1 bin-shim program. |
| `src/v3/compiler/src/r1c_e_gates.rs` | **(b)** | R1C-E shared check API; scaffold until R1 close per census comment. |
| `src/v3/compiler/src/r3_fc_lane2_loop_witness.rs` | **(a)** | Narrow T-Free-Consequences staging witness; candidate for cycle-3+ scoped dissolution once lowering owns lane2 text (Mgr provisional (a)). |
| `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` | **(a)** | Cycle-2 worker dispatch (Director msg_84abadad Track A); retire per `docs/briefs/r3-pb0-non-test-retirement-worker-cycle2.md` + gate #87 / T-WAD adjacent receipts. |
| `src/v3/compiler/src/regen_bootstrap_emit.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/regen_parse_emit.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/regen_parse_tables_emit.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/regen_tokenize.rs` | **(b)** | Bootstrap / regen toolchain + P5 atomic migration; substrate owned by Cluster M + `regen_*` briefs — not a single-PR retirement without canvas. |
| `src/v3/compiler/src/self_host_receipt_p0.rs` | **(b)** | DB-8 receipt schema host mirror; dissolve when `.dag`/generated owns schema per census header comment. |
| `src/v3/compiler/src/test_runner.rs` | **(b)** | Core compiler host; dissolution bundles multiple §1.8 / substrate gates — needs sequenced program, not opportunistic census drop. |
| `src/v3/compiler/src/wall_clock_ratchet_manifest.rs` | **(a)** | Ratchet manifest host; candidate for batch with timing/WAD receipts (Mgr provisional (a)). |

## §4. Changelog

| Date | Change |
|------|--------|
| 2026-05-13 | Initial taxonomy + Track A six-path dispatch list (Director msg_84abadad). |
| 2026-05-13 | §0 count narrative aligned to Director msg_dda96d21 (55 NON_TEST + 122 TEST = 177; Σ 179). |
