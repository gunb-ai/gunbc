# R2 Pure Bootstrap Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827; refreshed 2026-04-28 post-#1078 merge to absorb 3 distributed bridge retirements + R3 continuation lanes per Director cascade Items 4 + 8 ratified 2026-04-28). **R1-close-gated** per `r2-structure.md` Transition mechanics (PB scope = "what survives R1 close" — pre-R1-close spawn is NOT applicable to this manager). Migrates content from [`pure-bootstrap-zero-manager.md`](pure-bootstrap-zero-manager.md) (which archives on R2 promotion); **scope narrowed** per the gate-deferral resolution in PR #827.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of **7** standing R2 managers (count rose from 6 to 7 with Evaluator added 2026-04-28 per #1078).
- **R1 vs R2 boundary:** R1 owns Pure Bootstrap census-reduction work via T-PB-A / T-PB-B lanes per ROADMAP single authority on gate semantics (target = 0 per `docs/design-pure-bootstrap-zero.md` LIVE 2026-04-25). **R2 Pure Bootstrap Manager owns post-R1 PB program work that survives R1 close — not a duplicate of R1's census-reduction lanes.**
- **R3 continuation:** PB Manager continues into R3 with **T-LensProducer-Retirement (XL)** + **T-FixedPoint (M)** + **T-Tier3-Dissolution (M)** + **3 distributed bridge retirements** per Director cascade Item 4. Largest R2-manager continuation footprint into R3.
- **Cross-program coordination:** B4's §0.7 file-preference rank carrier touches PB territory; coordinate with Substrate Manager. Bridge-retirement coordination → Verification Manager (R3) for the unified `bridge_retirement_ledger_zero` audit gate.
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1): PB rarely introduces substrate (consumes substrate to dissolve mirrors). When it does (e.g., emergent dissolution surfaces a substrate gap), self-serve through the 3-step decision procedure or signal Substrate Manager.

## Program scope (T-PB; post-R1 only)

**Does NOT own:**
- T-PB-A non-test hand-Rust census reduction (R1 lane work per ROADMAP).
- T-PB-B Rust-authored test census reduction (R1 lane work per ROADMAP).
- `lens_producer_files_remaining` priority slice (R1 T-PB-A gate per PR #752).

**Owns (post-R1 R2 program work):**

| Lane | Size | Status (at brief authoring) | Description |
|---|---|---|---|
| Tier 3 mirror dissolutions: termination | M | AUTHORED in [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) | `DescentEvidence`, `PositiveDescentAmount`, `ProportionalDivisor`, `ShrinkFactor`, `evidence_rank`, `merge_evidence` Rust mirror at `dag.rs:628-790` dissolves as v3 lowers + evaluates `.dag` runtime values. Tier 3 #10 from #810. |
| Tier 3 mirror dissolutions: computation | M | AUTHORED in [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) | `SizeBound`, `RecursionShape`-related Rust mirror at `dag.rs:839-915` dissolves with same substrate dependency. |
| Tier 3 mirror dissolutions: induction | M | AUTHORED in [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) | `RecursionShape`, `InductiveField`, `SubValueRelation` Rust mirror at `dag.rs:916-980` dissolves with same substrate dependency. |
| Tier 3 mirror dissolutions: effect-carrier | S | AUTHORED in [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) | `src/v3/compiler/src/dag/effects.rs` (216 LOC) + `compose_operation_effects` in `workflow_idempotency.rs` (105 LOC). Mechanical PB dissolution once self-hosting reaches it. Tier 3 #12 from #810. |
| `kernel_algebra_profile` mirror dissolution | M | **SUBSTRATE LANDED via #1017** + tightened via #1068 (`ValueBody::Map` + `FieldValue::Map` + `FieldMap` newtype with duplicate-key validation); **consumer plumbing pending** (read-path/API + arrow-body evaluation). Day-1 manager work: dispatch the consumer-migration worker brief that drives `kernel_algebra_profile` Rust-mirror dissolution against the new substrate. | Map-shaped (not list/sum); substrate dependency now met. |
| Tier 2 `patch_lower_helpers_*` retirement | S | CLOSED by PR #1014 (first slice); **R3 continuation lane** for residual | `patch_lower_helpers_generated_type_alias_refinement`, the lower-helpers `regen_lens` patch path, and the SG-6 special case were retired once generated `lower_helpers` emitted the `refinement` field natively. Residual under T-Bridge-Retirement distribution (R3). |
| Post-R1 emergent dissolutions | varies | NOT YET MATERIALIZED | Catch-all for new PB work that surfaces post-R1 (new mirror dissolutions discovered during R2; new Rust scaffolds inadvertently introduced and needing dissolution). |

**Owns (R3 continuation — Director cascade Item 4 + Item 8 ratified 2026-04-28):**

| R3 Lane | Size | Description |
|---|---|---|
| **T-LensProducer-Retirement** | XL | Three program-sized hand-Rust files retired via PB-Runtime + PB-1 patterns per [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) (Items 4+5 LANDED via #1176). **Internal sub-gates** per [§5.1 — R3-T-LensProducer-Retirement sub-gates](../design-pb-runtime-interpreter.md): (i) `lens_apply.rs` retired (gated on PB-Runtime interpreter-as-data per §3); (ii) `lens_testgen.rs` retired (same gate as `lens_apply.rs`); (iii) `regen_lens.rs` retired (gated on PB-1 bin-shim emit pattern per §4.2 — distinct gate). **PB-Runtime foundation already landed:** ExecuteCommand typed-outcome hardening (#1049) + T-PB-B boundary coverage (#1082). The 5-primitive constraint (per §3.1) names `Node | Conj | Disj | Cardinality | Bit` as the DAG-processor execution vocabulary (the dispatch primitives PB-Runtime operates over; not the 5 L1 `Behavior` variants `Value | Transform | Branch | Loop | Bind` which `Node` carries — different scopes per §3.1 mapping note). PB-Runtime ≡ R2-Evaluator's runtime model expressed as `.dag` (load-bearing distinction per §2 — dissolution-shaped, not parallel). Closure ledger reports sub-gate progress; lane is one program. **Plus advanced lifetime analyzer cases d/e/f** (closures, async lifetimes, self-referential/Pin) folded in per `design-emission-model.md` Open call 2 — the lifetime analyzer is structurally what replaces `lens_apply.rs`'s reflection work, so advanced cases land alongside retirement. |
| **BinShim instances + emit pattern + retirement dispatch** *(NEW; Item 5 ownership per #1176 §5.4)* | S | PLANNING BRIEF AUTHORED at [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) (PROPOSAL; dispatch-gated). PB Manager owns per-shim `BinShim` instance declarations under `dsl/std/runtime/bin_shims/` + the bin-shim emit pattern + retirement dispatch (replaces hand-Rust `bin/regen_*` shims). Substrate Manager owns the `BinShim` carrier-type shape itself (additional fields, signature refinement) — generalized carrier-shape evolution escalates via INVARIANTS §P1 substrate-fact-introduction procedure. `regen_lens.rs` retirement (T-LensProducer-Retirement sub-gate iii) gates on this lane closing. Per §5.4 boundary: PB owns instance-row authoring + retirement; Substrate owns carrier-type evolution. |
| **T-FixedPoint** | M | `compiler.dag` compiles to bit-identical stage0 Rust + bit-identical emitted artifacts; R1's `pb_self_compile_fixed_point` gate closes under stronger interpretation. Depends on R2-Evaluator + SG-0 zero from T-LensProducer-Retirement. |
| **T-Tier3-Dissolution** *(may share with Tier 3 Manager continuing post-R2)* | M | Four hand-Rust mirrors of `.dag` types retired (mirror bodies replaced by Evaluator-backed authority inside `dag.rs` / `dag/effects.rs` / `workflow_idempotency.rs`); **consumer count / mirror-symbol count reaches zero**. SG-0 delta is reported and **usually 0** because the hand-authored file remains on the census after mirror-block retirement — SG-0 reaches 0 through broader PB-Substrate / generated-file retirement + T-LensProducer-Retirement. |
| **3 distributed bridge retirements** *(part of T-Bridge-Retirement distribution map; see Cross-program below)* | varies | (3) canonical lens-name dispatch — lens-producer-retirement adjacent; (4) `include_str!` side channels (e.g., `pipeline_authority.rs`) — compiler-internal bootstrap; (5) `patch_lower_helpers_*` residual — Tier 2 retirement lineage; #1014 was first slice. Verification Manager owns the unified `bridge_retirement_ledger_zero` ledger gate; PB owns retirement work for these 3 bridges. |

## Cross-program dependencies

**Produces:** none (PB consumes substrate, doesn't produce carriers other managers consume). R3 produces SG-0 zero signal that gates T-FixedPoint.

**Consumes:**
- **Substrate Manager — `ValueBody::Map` carrier read-path/API + arrow-body evaluation**: unblocks `kernel_algebra_profile` mirror dissolution. (Substrate landed post-#1017; consumer plumbing pending.)
- **Substrate Manager — B4 §0.7 file-preference rank carrier**: touches PB territory; coordinate.
- **R1 close**: T-PB-A / T-PB-B census-reduction work completes per R1 gate authority. PB Manager spawns post-close to own everything else.
- **R2-Evaluator (R3 continuation)** — R3 lanes T-LensProducer-Retirement / T-FixedPoint / T-Tier3-Dissolution all gate on R2-Evaluator landing. PB Manager R3 work waits on R2-Evaluator close.
- **R2-T-Ground-Lifetime-Analyzer (R3 continuation)** — provides basic cases a/b/c. Advanced cases d/e/f land inside T-LensProducer-Retirement.
- **Verification Manager (R3) — `bridge_retirement_ledger_zero`**: PB signals per-bridge retirement to Verification's unified ledger.

## Locked design decisions consumed (per #1078 dialogue + cascade)

Worker briefs MUST consume these without re-litigation:

- **T-LensProducer-Retirement XL framing kept** (Director cascade Item 8 ratified 2026-04-28): lane stays as one program; 3 internal sub-gates report sub-progress to closure ledger but do NOT split into 3 independent lanes. Reduces lane fragmentation; preserves the "one program" coherence of lens-producer retirement.
- **T-Bridge-Retirement distribution map** (Director cascade Item 4 ratified 2026-04-28): 5 named bridges; 3 retire under PB ownership (canonical lens-name dispatch / `include_str!` side channels / `patch_lower_helpers_*` residual); 2 retire under Substrate (`SourceSpan.file` + `mark_bootstrap_secret_nominal_opacity()`); Verification owns ledger gate only. **Distribute work, centralize ledger** discipline.
- **Q6 + Q6.5 (LANDED via #1129)**: `Witness<C>` substrate stays as-is; two-layer diagnostic-kind authority per [`docs/design-lens-framework.md` §"Q6.5 — Two-layer authority for diagnostic kinds"](../design-lens-framework.md) — Layer 1 closed sum (Substrate); Layer 2 lens-instance kinds in lens's own `.dag` via structural inhabitance. Relevant for PB-Runtime interpreter-as-data work that runs lens-instance `validate` functions.
- **Reflection completeness (LANDED via #1129)**: [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) names the cascade gates for R3-T-LensProducer-Retirement (§7.3) — load-bearing for `lens_apply.rs` retirement.
- **PB-Runtime interpreter-as-data + PB-1 bin-shim emit pattern (LANDED via #1176)**: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) — Item 4 (PB-Runtime as `.dag` interpreter) + Item 5 (bin-shim generation pattern). Worker briefs MUST consume:
  - **§3.1** — 5-primitive constraint (PB-Runtime restricted to the DAG-processor execution vocabulary `Node | Conj | Disj | Cardinality | Bit` per `feedback_compiler_is_dag_processor`; the 5 L1 `Behavior` variants `Value | Transform | Branch | Loop | Bind` are dispatched ON inside `Node`, not parallel primitives — different scopes per §3.1 mapping note).
  - **§3.2** — `Value` coproduct shape (the runtime-value surface PB-Runtime exposes; mirrors R2-Evaluator's runtime model expressed as `.dag`).
  - **§4.2** — Bin-shim emit pattern (the structural template for retiring `bin/regen_*` hand-Rust shims).
  - **§5.1** — R3-T-LensProducer-Retirement sub-gate decomposition (sub-gates 1+2 → Item 4 PB-Runtime; sub-gate 3 → Item 5 PB-1 bin-shim).
  - **§5.4** — Cross-program coordination: PB Manager + Evaluator Manager co-author the convergence path. **PB owns** per-shim `BinShim` instance declarations (under `dsl/std/runtime/bin_shims/`) + bin-shim emit pattern + retirement dispatch. **Substrate owns** the `BinShim` carrier-type shape (generalized shape evolution escalates via §P1 substrate-fact-introduction). **Evaluator owns** runtime-value model that PB-Runtime mirrors.
  - **§6** — 6 anti-bridge invariants (no parallel runtime; no Y-combinator escape; no untyped Value; no closure-fabrication; no hand-Rust regen-shim authoring; no bypass-the-5-primitive rule).
  - **§7** — 3 TestClaim shapes (locked names per design doc): `pb_runtime_equivalent_to_evaluator_on_corpus` (§7.1 PB-Runtime equivalence fixture); `regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust` (§7.2 BinShim equivalence fixture); `no_new_bin_shim_hand_rust` (§7.3 No-new-bin-shim-hand-Rust fixture).

Full disposition: [`docs/r2-structure.md`](../r2-structure.md) §4 + [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure".

## Pre-spawn vs post-spawn authority

- **Pre-spawn (post-#1078-merge, before R1 close):** Director + PM coordinate on brief authoring per inbox #828 split. PM authors the manager skeleton (this file); Director authors any worker-level briefs not yet existing per the manager's "Pending" sub-briefs list. Both stop authoring once R2 spawns. **PB Manager itself is R1-close-gated** — does not spawn pre-R1-close (unlike the other 6 R2 managers).
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.

## Autonomous dispatch authority

- Authors all post-R1 PB sub-briefs without Director (R2 + R3 continuation).
- Dispatches workers against post-R1 PB sub-briefs.
- Resolves PB-internal scope refinements; escalates blockers and scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every PB worker brief names dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; per-PR gate applies to all hand-Rust dispatches.
- **Cross-program signal authority:** lane-close → R2 Release Manager (closure ledger); per-bridge retirement → Verification Manager (unified ledger gate); R3 lane closure → Director (R3 spin-up + R3 Release Manager, when authored).

## Reporting cadence

- Lane-close → R2 Release Manager (closure ledger; per **structural-acceptance-per-lane-close discipline** — the demo IS the structural gate).
- **T-LensProducer-Retirement sub-gate progress** → R2 Release Manager (per Director directive: closure ledger reports sub-gate progress within the one-program lane).
- Cross-program signals (consume Substrate carrier-readiness) → cross-manager queue.
- Per-bridge retirement signal → Verification Manager (R3) for `bridge_retirement_ledger_zero` audit.
- Blockers + scope changes → Director.
- **Weekly health surfacing to Director:** which lanes within 1 step of unblocking, R3 continuation readiness, sub-gate progress on T-LensProducer-Retirement.

## Acceptance — `.dag` gates

Each lane closes under a structural acceptance gate authored as a `.dag` `TestClaim`:

**R2 lanes:**
- `tier_3_termination_mirror_dissolved` — mirror at `dag.rs:628-790` deleted; v3 carries the load
- `tier_3_computation_mirror_dissolved` — mirror at `dag.rs:839-915` deleted
- `tier_3_induction_mirror_dissolved` — mirror at `dag.rs:916-980` deleted
- `tier_3_effect_carrier_mirror_dissolved` — `effects.rs` + `compose_operation_effects` deleted
- `kernel_algebra_profile_mirror_dissolved` — gated on ValueBody::Map consumer plumbing

**R3 continuation lanes:**
- `lens_apply_dot_rs_retired` — `src/v3/compiler/src/lens_apply.rs` deleted
- `lens_testgen_dot_rs_retired` — `src/v3/compiler/src/lens_testgen.rs` deleted
- `regen_lens_dot_rs_retired` — `src/v3/compiler/src/bin/regen_lens.rs` deleted
- `pb_self_compile_fixed_point_strong` — bit-identical stage0 + emitted artifacts
- `bridge_canonical_lens_name_dispatch_retired` — distributed bridge #3 (**partial retirement; full close blocked.** §0.1 `PROGRAM_INPUT_SENTINEL` + §0.2 `cost_bind_for_claim_file` already retired structurally on main. Remaining surface: 2 `include_str!` of canonical lens bytes + 2 `lens_decl.name == Some("…")` arms + 2 generic name-keyed lookups in `test_runner.rs`. Pinned by `tests/integration/canonical_lens_bridge_ratchet_test.rs`. Full close gated on PB-Runtime interpreter-as-data **or** typed lens-registry carrier substrate-introduction; see [`docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md))
- `bridge_include_str_side_channels_retired` — distributed bridge #4 (e.g., `pipeline_authority.rs`)
- `bridge_patch_lower_helpers_residual_retired` — distributed bridge #5 (**PB Tier-2 lower-helper exact-string patch class:** #1014 deleted the helper + `regen_lens` / SG-6 special cases; **zero code residual** in `src/v3/compiler/**/*.rs` + `build.rs` per 2026-04-29 grep audit. **Ratchet:** `tests/integration/bridge_lower_helpers_patch_zero_residual_test.rs` fails CI if contiguous `patch_lower`+`_helpers` reappears — scoped to that retired bridge, not other string transforms.)

## Sub-briefs (authored / pending)

Authored:
- Pre-R1 PB program briefs (in `pure-bootstrap-zero-manager.md`, archives on R2 promotion); content migration here covers post-R1 deliverables only.
- [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) — termination, computation, induction, and effect-carrier mirror dissolution worker pack.
- [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) — T-FixedPoint planning artifact (PROPOSAL; dispatch-gated on R2-Evaluator + SG-0=0 from T-LensProducer-Retirement).
- [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) — BinShim instances + emit pattern + retirement dispatch planning artifact (PROPOSAL; dispatch-gated on R2-Evaluator + Item 4 PB-Runtime + Substrate-owned `BinShim` carrier + §7.3 `CensusListConstant`/filter disposition).
- [`r2-pb-runtime-evaluator-convergence-matrix.md`](r2-pb-runtime-evaluator-convergence-matrix.md) — PB-Runtime ↔ R2-Evaluator convergence verification matrix (audit; docs-only) mapping `design-pb-runtime-interpreter.md` §§2, 3.2, 5.4, 6, 7.1 obligations onto R2-Evaluator surfaces + prerequisite state for #1231 / PR-A.1.
- [`r3-pb-runtime-equivalence-corpus-seed-audit.md`](r3-pb-runtime-equivalence-corpus-seed-audit.md) — corpus seed audit (docs-only, post-#1235) expanding the convergence matrix Row 4 corpus phrase into a per-seed table for arithmetic-on-`Int` / `List` map+fold / one `Lens<C>` instance.
- [`r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`](r3-pb-t-lensproducer-sub1-lens-apply-retirement.md) — T-LensProducer-Retirement sub-gate 1 (`lens_apply_dot_rs_retired`) skeleton (PROPOSAL; dispatch-gated on R2-Evaluator + Item 4 PB-Runtime + convergence-matrix Row 4 green + canonical-lens-bridge dependency surface migrated).
- [`r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md`](r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md) — T-LensProducer-Retirement sub-gate 2 (`lens_testgen_dot_rs_retired`) skeleton (PROPOSAL; dispatch-gated on same Item 4 chain as sub-gate 1; testgen consumer surface mapped).
- [`r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md) — T-LensProducer-Retirement sub-gate 3 (`regen_lens_dot_rs_retired`) skeleton (PROPOSAL; dispatch-gated on Item 5 bin-shim emit pattern + `BinShim` carrier + `regen_lens_shim` instance authored via the BinShim retirement program).
- [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) — PB-owned BinShim instance declaration framework (canonical home + naming convention + STOP+PING for the missing `BinShim` carrier substrate authority). Per-shim `.dag` files land here once Substrate Manager lands the carrier.
- [`r3-pb-regen-lens-first-binshim-target-retirement-readiness.md`](r3-pb-regen-lens-first-binshim-target-retirement-readiness.md) — first BinShim target (`regen_lens.rs`) retirement-readiness checklist (PROPOSAL; planning artifact only; owners + STOP + SG-0 / `REGEN_OUTPUTS`).
- [`r3-pb-regen-lens-consumer-audit.md`](r3-pb-regen-lens-consumer-audit.md) — carrier-independent consumer / build / SG-0 / `REGEN_OUTPUTS` / call-surface audit for `regen_lens.rs` + per-handoff rows for the future `BinShim` carrier + instance + emitter + §7.2 equivalence fixture. Docs-only; no carrier or schema invention.
- [`r3-pb-binshim-7-2-claim-shape.md`](r3-pb-binshim-7-2-claim-shape.md) — locked §7.2 BinShim equivalence `TestClaim` shape (`regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust`). Predicate locked as `ExecuteCommand` Mechanism (1) canonicalize-then-diff; live substrate verified; STOP+PING for the comparison script + emitted-Rust + hand-Rust snapshot that gate landing as a real `.dag` fixture.
- [`r3-pb-tv2-g1-readiness-receipt.md`](r3-pb-tv2-g1-readiness-receipt.md) — T-V2-Retirement G-1 first-consumer readiness check (docs-only). Verifies S-1 NOT MET on origin/main; STOP+PING per audit/migration-matrix STOP rules; pins next-unblock order (S-1 lands → §3.1 first slice → §3.2 cross-program → §3.3 Cargo edges → G-1 green).
- [`r3-pb-tv2-s1-input-packet.md`](r3-pb-tv2-s1-input-packet.md) — T-V2-Retirement S-1 input packet (input to PM/Director; not S-1 itself). Decision checklist for the PM-authored worker brief: 6 rows enumerating §3.1 / §3.2 / §3.3 / legacy emit chain / `verification.dag` convergence routing / S-1 scope; PB recommended defaults + owner per decision (PM / PB / Substrate / Director). Docs-only.
- [`r3-pb-tv2-population-coverage-audit.md`](r3-pb-tv2-population-coverage-audit.md) — T-V2-Retirement Population A (4 named tests in `src/v2/tests/`) + Population B (2 G-1 consumers outside `src/v2/`) coverage spot-check audit. Live v3 substrate analogs verified for all 4 Pop A targets (LIVE; tests MISSING — mechanical port). Pop B disposition recommendations match S-1 input packet Decisions 1+2; per-file v3-coverage state spot-checked. Net dispatch order from audit/matrix DAG. Docs-only.
- [`r3-pb-tv2-g1-execution-slices.md`](r3-pb-tv2-g1-execution-slices.md) — T-V2-Retirement G-1 execution brief for issue #1975. Makes §3.1 replace-vs-delete branch explicit, updates §3.2 after substrate `kernel_algebra_profile` authority migration landed, and keeps §3.3 Cargo-edge deletion atomic with the last v2 consumer removal.
- [`r3-pb-bridge-include-str-side-channels-closure.md`](r3-pb-bridge-include-str-side-channels-closure.md) — T-Bridge-Retirement bridge #4 (`bridge_include_str_side_channels_retired`) **standalone closure brief** for `pipeline_authority` / #1171 lineage (#1976; STOP-BLOCKED; **dispatch-gated** on structural compile-body witness or approved derivation — **not** runtime file-IO swap).
- [`r3-pb-bridge-canonical-lens-name-dispatch-closure.md`](r3-pb-bridge-canonical-lens-name-dispatch-closure.md) — T-Bridge-Retirement bridge #3 (`bridge_canonical_lens_name_dispatch_retired`) **closure worker brief** (PROPOSAL; **dispatch-gated** on PB-Runtime interpreter-as-data **or** typed lens-registry / cross-`Dag` resolution per §P1 — consumes [`r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md)).

Pending — pre-spawn Director-authored per inbox #828 coordination split; post-spawn manager-authored autonomously:
- `kernel_algebra_profile` worker brief (gated on Substrate Manager `ValueBody::Map` consumer plumbing)
- **R3 T-Tier3-Dissolution** worker brief (may share with Tier 3 Manager continuing post-R2)
- **R3 distributed bridge retirements** — ~~canonical lens-name dispatch~~ → [`r3-pb-bridge-canonical-lens-name-dispatch-closure.md`](r3-pb-bridge-canonical-lens-name-dispatch-closure.md); ~~`include_str!`~~ → [`r3-pb-bridge-include-str-side-channels-closure.md`](r3-pb-bridge-include-str-side-channels-closure.md); ~~`patch_lower_helpers_*` Tier-2 slice~~ closed PR #1014

Closed:
- Tier 2 `patch_lower_helpers_*` retirement first slice (PR #1014)

## Working state (fill on spawn)

Spawn refresh, 2026-04-28 (post-#1078, status-refresh against landed PRs):

- R2 lanes unchanged in scope; status table tracks Tier 3 mirror dissolutions + kernel_algebra_profile.
- **kernel_algebra_profile substrate met:** ValueBody::Map landed (#1017) + tightened (#1068); consumer plumbing is the remaining work — dispatchable Day-1.
- **T-PB-Runtime foundation landed:** ExecuteCommand typed-outcome hardening (#1049) replaces partial `Other(ClaimResult)` carrier; T-PB-B boundary coverage (#1082). **PB-Runtime interpreter-as-data design lock LANDED via #1176** ([`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md)) — sub-gates 1+2 (lens_apply.rs/lens_testgen.rs retirement) consume Item 4; sub-gate 3 (regen_lens.rs retirement) consumes Item 5 (PB-owned per-shim `BinShim` instances + emit pattern + retirement; Substrate-owned `BinShim` carrier-type evolution; boundary per §5.4).
- R3 continuation added: T-LensProducer-Retirement (XL with 3 internal sub-gates) + T-FixedPoint + T-Tier3-Dissolution + 3 distributed bridge retirements.
- 3 distributed bridges absorbed into existing PB scope per Director cascade Item 4 (no new manager spawning).

## Cross-refs

- Parent: `docs/r2-structure.md` §"Pure Bootstrap Manager"
- R3 continuation: `docs/r3-structure.md` §"Lane structure" (T-LensProducer-Retirement / T-FixedPoint / T-Tier3-Dissolution / T-Bridge-Retirement distribution map)
- Migrating from: `docs/briefs/pure-bootstrap-zero-manager.md` (archives on R2 promotion)
- Self-hosting design: `docs/design-pure-bootstrap-zero.md` LIVE 2026-04-25
- Tier 3 worker pack: [`docs/briefs/r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md)
- Tier 3 source: `docs/briefs/debt-paydown-synthesis-2026-04-25.md` items #10 + #12
- Lens framework spec: `docs/design-lens-framework.md` (Q6+Q7+Q8 locks; consumed by PB-Runtime interpreter-as-data)
- ROADMAP single authority on gate semantics: `ROADMAP.md §"Lane acceptance — .dag gates"`
- Thesis-claim disposition: `docs/thesis/r2-r3-thesis-mapping.md`
- PB-Zero / v2 emit boundary — **planning audit only** (row-authority consumer gap vs. live `src/v3/std/*_method_template_contracts.dag`; no implementation claim): [`docs/audit/pb-zero-v2-method-template-row-authority-consumer-gap.md`](../audit/pb-zero-v2-method-template-row-authority-consumer-gap.md)
- PB-Zero / v2 emit — **canonical read surface options** (STOP matrix only; no surface chosen): [`docs/audit/pb-zero-v2-canonical-read-surface-options-stop-matrix.md`](../audit/pb-zero-v2-canonical-read-surface-options-stop-matrix.md)
