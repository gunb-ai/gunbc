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
| **T-LensProducer-Retirement** | XL | Three program-sized hand-Rust files retired via PB-Runtime + PB-1 patterns. **Internal sub-gates** (Director directive 2026-04-28 — XL framing kept; sub-gate visibility for closure-ledger reporting): (i) `lens_apply.rs` retired (gated on PB-Runtime interpreter-as-data); (ii) `lens_testgen.rs` retired (same gate as `lens_apply.rs`); (iii) `regen_lens.rs` retired (gated on PB-1 bin-shim emit pattern — distinct gate). **PB-Runtime foundation has begun pre-R3:** `T-PB-Runtime ExecuteCommand typed-outcome hardening` LANDED via #1049 (replaces `Other(ClaimResult)` partial carrier with 6-variant typed model; namespace-setup detection dissolved via `gunbc_execute_command_bootstrap` helper binary from #1063); T-PB-B ExecuteCommand boundary coverage extended via #1082. PB-Runtime interpreter-as-data still pending — that's the remaining gate for `lens_apply.rs` retirement. Closure ledger reports sub-gate progress; lane is one program. **Plus advanced lifetime analyzer cases d/e/f** (closures, async lifetimes, self-referential/Pin) folded in per `design-emission-model.md` Open call 2 — the lifetime analyzer is structurally what replaces `lens_apply.rs`'s reflection work, so advanced cases land alongside retirement. |
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
- **Q6 (lens framework)**: `Witness<C>` substrate stays as-is; structural-validation failures encode into `Diagnostic.kind` extensions. Relevant for PB-Runtime interpreter-as-data work.

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
- `bridge_canonical_lens_name_dispatch_retired` — distributed bridge #3
- `bridge_include_str_side_channels_retired` — distributed bridge #4 (e.g., `pipeline_authority.rs`)
- `bridge_patch_lower_helpers_residual_retired` — distributed bridge #5

## Sub-briefs (authored / pending)

Authored:
- Pre-R1 PB program briefs (in `pure-bootstrap-zero-manager.md`, archives on R2 promotion); content migration here covers post-R1 deliverables only.
- [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) — termination, computation, induction, and effect-carrier mirror dissolution worker pack.

Pending — pre-spawn Director-authored per inbox #828 coordination split; post-spawn manager-authored autonomously:
- `kernel_algebra_profile` worker brief (gated on Substrate Manager `ValueBody::Map` consumer plumbing)
- **R3 T-LensProducer-Retirement** worker briefs (gated on R2-Evaluator close + PB-Runtime interpreter-as-data design + PB-1 bin-shim emit pattern; 3 internal sub-gates)
- **R3 T-FixedPoint** worker brief (gated on SG-0 zero from T-LensProducer-Retirement)
- **R3 T-Tier3-Dissolution** worker brief (may share with Tier 3 Manager continuing post-R2)
- **R3 distributed bridge retirements** — 3 worker briefs (canonical lens-name dispatch / include_str! / patch_lower_helpers_* residual)

Closed:
- Tier 2 `patch_lower_helpers_*` retirement first slice (PR #1014)

## Working state (fill on spawn)

Spawn refresh, 2026-04-28 (post-#1078, status-refresh against landed PRs):

- R2 lanes unchanged in scope; status table tracks Tier 3 mirror dissolutions + kernel_algebra_profile.
- **kernel_algebra_profile substrate met:** ValueBody::Map landed (#1017) + tightened (#1068); consumer plumbing is the remaining work — dispatchable Day-1.
- **T-PB-Runtime foundation landed:** ExecuteCommand typed-outcome hardening (#1049) replaces partial `Other(ClaimResult)` carrier; T-PB-B boundary coverage (#1082). PB-Runtime interpreter-as-data still pending — that's the load-bearing gate for `lens_apply.rs` retirement.
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
