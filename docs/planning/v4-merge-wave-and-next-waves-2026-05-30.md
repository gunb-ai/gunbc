# v4 Merge Wave (clearing now) + Next Waves Planning

> **Status:** PLANNING DRAFT — operator engagement requested on §7 decisions.
> **Date:** 2026-05-30
> **Author:** PM May 29 (session `nimble-dove-733`)
> **Trigger:** Operator authorized self-merge reversal 2026-05-30 ~15:10Z; ~15-PR backlog clearing now. Operator: *"can you take a doc covering the merges we're making now... and then, we can discuss the upcoming waves and any changes/decisions we have to make (in the doc)."*

This doc is the planning surface for the next conversation. §2 inventories the merges landing now. §3 captures what the merge wave delivers (the new substrate baseline). §4-§6 sketch three upcoming waves. §7 surfaces the decisions worth making before each wave dispatches.

---

## §1. Where we are

PR #3938 (correctness ladder) + #3959 (CI overhaul + leaf-model verification) merged earlier today established the architectural commitments. The §11.4 dispatch sequence from PR #3959 is partly in flight; the SG-class worksheets from PR #3938 §10 are partly complete. A ~15-PR backlog accumulated under the temporary manual-merge hold (now reversed). Once the backlog clears, the substrate baseline shifts substantially — multiple lanes' first artifacts land simultaneously.

This doc inventories what's landing and what comes next, so the next wave-dispatch can be planned coherently rather than reactively.

---

## §2. Merges in the current wave

Status as of 2026-05-30 ~15:15Z (just after operator self-merge re-authorization).

### Already merged (earlier today)

| # | PR | Lane | Substance |
|---|----|----|-----------|
| 1 | #3938 | PM (planning) | Correctness ladder + 17 standards + manager-lane architecture + §10 SG worksheets |
| 2 | #3949 | Close/Receipt | Manager pass + 346-probe close ledger with two-axis dispositions |
| 3 | #3953 | Ladder/Fixture (jolly-bee) | nat_semiring substrate wiring |
| 4 | #3959 | PM (planning) | CI overhaul scoping + leaf-model verification framework |
| 5 | #3969 | Modeling DFS (smart-seal-842) | CI schema doc (CiUpsertStep / UpsertInputRef / carveouts) |
| 6 | #3948 | Self-host/Release | v4-done six-predicate tracker |

### Landing in current wave (just merged or merging now)

| # | PR | Lane | Substance |
|---|----|----|-----------|
| 7 | #3973 | Close/Receipt | P1-P6 ↔ TASKS.md:806-817 line-anchor mapping |
| 8 | #3975 | Self-host/Release | P1-P6 :819 snapshot + P5/TestClaim vs resolve-posture-bridge disambiguation |
| 9 | #3967 | Self-host/Release | Closed as superseded by #3975 |
| 10 | #3977 | Modeling DFS | SG-7 worksheet (ByteOffsetCacheDigestAuthority + byte_offset_cache_key dissolve ci.dag:703-777) |
| 11 | #3947 | Compiler Spine | Joint rung 3-4 min runner interface + CiSelectionReceipt Phase 2.0 + cache_digest amend |
| 12 | #3946 | Ladder/Fixture | v4-ladder-rung-specs (nat_semiring rungs 0-2 acceptance predicates) |
| 13 | #3955 | Ladder/Fixture (jolly-bee) | ci.dag + GHA wiring for rung 0-2 gate |
| 14 | #3958 | Runtime/TestClaim (keen-raven) | W2 host harness for rung 4 on nat_semiring |
| 15 | #3960 | Compiler Spine (nimble-ram) | W1 RoundTripClaim eval path (ingest⁻¹ on dag target) |
| 16 | #3961 | Runtime/TestClaim | Verdict-surface contract (TestClaimRun<S,A> + Verdict<A> + FalsificationReceipt) |
| 17 | #3962 | Target Realization (fierce-deer) | SG-2 TargetTypeExpressionProjection |
| 18 | #3970 | Target Realization (silent-cat) | LeafModelClaim shape for rust.dag (carriers + claim corpus seed) |
| 19 | #3971 | Target Realization | TR spec: LeafModelClaim instantiation design |
| 20 | #3972 | Runtime/TestClaim (quick-lark) | Step 4 R1 fixture generator + runner against rustc (in progress) |
| 21 | #3974 | Compiler Spine (quiet-ant) | CI relief: concurrency cancel + docs-only fmt skip (Option C dropped — substrate-authority violation) |

### Held draft (TR sequencing)

| # | PR | Lane | Substance |
|---|----|----|-----------|
| 22 | #3956 | Target Realization (zesty-carp) | SG-1 TargetAtomRealization — held until R3 receipts via Step 4 |
| 23 | #3957 | Target Realization (vivid-heron) | SG-5 TargetCollectionRealization — held until SG-2 lands |
| 24 | (SG-6) | Target Realization | Queued behind SG-5 |

---

## §3. What the merge wave delivers (substrate baseline post-clear)

After the wave clears:

**Authoritative scoping artifacts on main:**
- The 9-rung correctness ladder + 17-standard inventory + manager-lane architecture (PR #3938)
- CI overhaul phases 1.4/1.5/2/2.5 + Upsert<T>-shaped step substrate (PR #3959 §5-§7)
- Leaf-model verification framework (PR #3959 sibling doc — `LeafModelClaim<M,Subject,Expectation>`, R1/R2a/R2b/R3-external/R3-internal scope)
- §10 SG-1/SG-2/SG-5/SG-6/SG-7 root-cause worksheets with single-authority facts named
- Per-probe close ledger (346 probes) + v4-done six-predicate tracker

**Implementation/substrate slices on main:**
- `TargetTypeExpressionProjection` SG-2 substrate (fierce-deer #3962)
- `LeafModelClaim` carriers + rust.dag claim corpus seed (silent-cat #3970)
- TR spec for LeafModelClaim instantiation (#3971)
- Joint rung 3-4 runner interface (smart-stag #3947) + W1 RoundTripClaim eval (nimble-ram #3960) + W2 host harness (keen-raven #3958)
- Verdict-surface contract (`TestClaimRun<S,A>` / `Verdict<A>` / `FalsificationReceipt`) (#3961)
- Step 4 R1 fixture+runner against rustc (quick-lark #3972) — once stable, *first leaf-model verdict ever produced on this project*
- nat_semiring rungs 0-2 acceptance predicates + ci.dag+GHA wiring (#3946 + #3955)
- SG-7 worksheet sets up Compiler Spine impl
- CI relief A+B (concurrency cancel + docs-only fmt skip) (#3974)

**Held drafts (next-wave):**
- SG-1 (zesty-carp #3956) — gated on R3 receipts
- SG-5 (vivid-heron #3957) — gated on SG-2 land

---

## §4. Wave 1 (next-up, dispatchable immediately post-clear)

These are bounded, gated on the current wave landing, and have named owner managers ready:

| # | Item | Owner manager | Gated on |
|---|------|---------------|----------|
| W1.1 | **SG-7 implementation in ci.dag** (dissolve recursion via ByteOffsetCacheDigestAuthority) | Compiler Spine | #3977 merge (just landed) |
| W1.2 | **Phase 1.4: land `Upsert<T>` as usable substrate primitive** in dsl/std/patterns.dag | Modeling DFS | proud-pike-680 / cool-ibex-692 successor |
| W1.3 | **Step 4 R1 fixture+runner completion** (first leaf-model verdict landing end-to-end) | Runtime/TestClaim (quick-lark) | #3972 finalization |
| W1.4 | **Phase 1a: T-22 interpreter on ci_pipeline** (ci.dag becomes sole policy authority for integrity-class CI) | Compiler Spine | #3947 merge (in progress) |
| W1.5 | **Phase 2.1: ci.dag types + ci_selection_receipt_shadow** | Compiler Spine | #3947 merge |
| W1.6 | **SG-2 worker re-dispatch** (post SG-7 unblocks T-22 on ci.dag-affected PRs) | Target Realization | SG-7 impl + W1.1 |
| W1.7 | **R2a/R2b/R3-external/R3-internal** leaf-model claims authoring | Modeling DFS + Target Realization | Step 4 R1 stable (W1.3) |

**Wave 1 critical path:** W1.3 (first verdict) + W1.1 (SG-7 unblocks ci.dag) — both are gating dependencies for downstream.

---

## §5. Wave 2 (gated on Wave 1 maturity)

| # | Item | Owner manager | Gated on |
|---|------|---------------|----------|
| W2.1 | **SG-1 TargetAtomRealization** dispatch (using R3 receipts as verification) | Target Realization | W1.7 R3 + Step 4 |
| W2.2 | **SG-5 / SG-6** dispatch | Target Realization | W2.1 + same-file authoring resolved |
| W2.3 | **Phase 1.5: every CI step becomes `CiUpsertStep<T>`** | Modeling DFS + Compiler Spine | W1.2 Upsert<T> + W1.5 receipt shape |
| W2.4 | **Phase 1b: A3-A14 atom-by-atom migration + `scripts/check-*` deletion (A6-A8)** | Compiler Spine + clever-cat-115 coordination | W2.3 |
| W2.5 | **Phase 4 widening: 2-3 more fixtures (Branch-using, Loop-using)** | Ladder/Fixture (re-spawn if needed) | W1.3 + W1.7 |
| W2.6 | **Cross-target leaf-model verification: python.dag** | Modeling DFS + Target Realization + Runtime/TestClaim | W1.3 + LeafModelClaim shape generalizes |

---

## §6. Wave 3 (decision-loaded after Wave 2 maturity)

These are where the project's bigger choices live. None are dispatch-ready until Wave 1-2 mature.

| # | Item | Why decision-loaded |
|---|------|---------------------|
| W3.1 | **Phase 2 (T-24): Shape-B ci.yml emitted from CiPipeline; all hand-authored YAML deleted** | Heavy: requires all A0-A14 atoms ported (W2.4); coordinates with the CI relief PR being dissolved into substrate |
| W3.2 | **Phase 2.5: affected-set intersection gate firing** | Heavy: requires W1.2 Upsert<T> + W2.3 CiUpsertStep + W3.1 Phase 2 |
| W3.3 | **Cross-target equivalence on substantial fixture set (rung 5 closure)** | Scope: which targets (Python, Go, C++, TypeScript, etc.)? How many fixtures? |
| W3.4 | **L7 algebraic preservation post-emit (rung 6 closure)** | Per-fixture per-target per-algebra — combinatorial |
| W3.5 | **Self-emit fixpoint (rung 7) — T-15 close** | Load-bearing for predicate 4 (bit-identical self-output) of TASKS.md:805-817's six v4-done predicates — NOT v4-done alone. Per PR #3938 §8 D4: v4-done = all six predicates collectively (every other scheduled task + corpus compiles + emit compiles + bit-identical self-output + TestClaim suite passes + reproduction proves hand-Rust not editable authority). Requires every Wave 1-2 item + W3.1 + a binary that compiles compiler.dag to itself bit-identically. |
| W3.6 | **TestClaim corpus actually executes (rung 8)** | Requires runner + cache + all SG fixes; per Phase 0 audit it's the largest unaddressed standard block |
| W3.7 | **Lenses gate PRs (rung 9): complexity / ownership / idempotency / grounding / synthesis** | Each lens needs activation; substrate-rich/activation-poor pattern at its widest scope |

---

## §7. Decisions (operator-ratified 2026-05-30)

All decisions in this section are namespaced **MW-D*** (Merge-Wave Decision) to avoid collision with the D1-D7 sets in PR #3938 (correctness ladder §8) and PR #3959 (CI overhaul §8 + leaf-model verification §10).

### MW-D1. Wave 1 parallelism

W1.1 (SG-7 impl) + W1.2 (Phase 1.4 Upsert<T>) + W1.3 (Step 4 R1) + W1.4 (Phase 1a) + W1.5 (Phase 2.1) all touch different lanes.

**Operator decision 2026-05-30:** Dispatch W1.1–W1.5 in parallel after baseline verification (see §7.1). PM coordinates the launch and cross-lane interface checks; managers self-coordinate execution.

### MW-D2. Wave 1 ordering pressure

Three items have downstream cascades; if forced to expedite only two:

**Operator decision 2026-05-30:** Expedite **W1.3** (Step 4 R1 first leaf-model verdict — highest signal; first "model fact verified against external target" receipt) and **W1.1** (SG-7 impl — highest unblocker; clears ci.dag-affected PRs). Keep **W1.2** (Phase 1.4 Upsert<T>) running in parallel since it gates the deeper CI chain, but do NOT let it block the first external verdict.

### MW-D3. Cross-target widening (v4 release minimum)

**Operator decision 2026-05-30:** **Rust + Python + Go** is the v4 release-minimum cross-target set for fixture-level L5/L6 proof. C++ / TypeScript / LLVM / others are modeled and planned but **not required** for the first L5/L6 release proof unless TASKS explicitly makes them part of v4-done.

### MW-D4. Phase 2 (T-24 close) trigger

**Operator decision 2026-05-30:** Keep Phase 2 in T-24 done. Phase 1.4/1.5/2.5 can be major progress, but closing T-24 without generated YAML + deletion of hand-authored workflow YAML would require an explicit TASKS amendment. Per PR #3959 D-CI-4 + this MW-D4: full Phase 2 stays mandatory for T-24 [DONE].

### MW-D5. Manager succession policy (NEW substantive policy)

**Operator decision 2026-05-30:** **Single active accountable manager per role-node.** A successor may replace the prior manager only with an explicit succession receipt:
```text
- previous manager / session
- new manager / session
- carried open decisions
- carried blocked PRs
- carried worker handles
```
Parallel helper sessions are allowed only as **delegated workers/deputies**, not as co-equal managers for the same role-node. The cool-ibex-692 / proud-pike-680 overlap (Modeling DFS) is the trigger for this policy — going forward, the PM (or operator) ratifies succession explicitly; the dashboard should not silently spawn parallel co-equal managers.

### MW-D6. Dashboard-tooling patterns (5 surfaced)

**Operator decision 2026-05-30:** Dispatch ONE bounded "dashboard-control-plane incidents" audit under Close/Receipt or PM. It produces a small incident ledger + recommended dashboard fixes — **not compiler work**. Operator/tooling responsibility unless it directly affects merge authority. NOT on the compiler critical path.

### MW-D7. Operator engagement model through Wave 1

**Operator decision 2026-05-30:** **Hybrid.** PM coordinates Wave 1 launch, resolves interface/baseline questions; managers self-coordinate within their lanes after kickoff. PM re-enters only for cross-lane conflict or scope change.

### MW-D8. Wave 1 exit receipt (NEW)

**Operator decision 2026-05-30:** Wave 1 is complete only when ALL of:

```text
1. Step 4 R1 produces an actual leaf-model verdict (rust.dag R1 → rustc → Verdict<R1>).
2. SG-7 ci.dag recursion is dissolved OR replaced by a modeled authority
   (ByteOffsetCacheDigestAuthority + byte_offset_cache_key consumed).
3. Upsert<T> is either landed as usable substrate primitive OR explicitly
   blocked with a Modeling DFS worksheet naming the parser/substrate gap.
4. ci_selection_receipt_shadow exists and can be generated for at least
   one PR/change fixture (shadow mode, not active gating yet).
5. R2a/R2b/R3-external/R3-internal claim authoring has ready-to-run OR
   explicitly-blocked status (each claim authored or its blocker named).
```

Without all 5, Wave 1 is not complete regardless of how many PRs landed. Prevents "seven PRs merged, unclear whether the wave achieved anything."

---

## §7.1. Post-clear baseline verification (gates Wave 1 dispatch)

Before W1.1–W1.5 dispatch, PM (or designated manager) verifies the merge wave actually landed the expected baselines:

```text
- SG-7 worksheet (#3977) on main + Compiler Spine has main SHA pinned
- rung 3-4 runner interface (#3947) on main + branch-base for Phase 2.1 ready
- Verdict<A> surface (#3961) on main + downstream consumers can import
- R1 fixture+runner (#3972) on main OR explicitly in-progress with named PR
- SG-2 substrate (#3962) on main + held SG-1 (#3956) still draft
- Held drafts: #3956, #3957, #3960 remain held per TR sequencing
- Wave 1 launch waits on this baseline being green
```

Failure to verify before dispatch = Wave 1 starts on inconsistent baseline; managers may produce divergent work. This is the launch barrier, not the execution barrier.

---

## §7.2. Wave 1 is NOT v4-close

Important framing clarification per operator review:

**Wave 1 produces first verdicts and unblocks CI/target-realization chains. It does NOT close v4.** v4-done remains governed by all six predicates of `TASKS.md:805-817`:
1. Every other scheduled task in this plan complete (whole plan minus T-15)
2. v4 compiles `src/v4/compiler/*.dag` end-to-end
3. v4 emits Rust source that compiles to a binary
4. Binary, run on `src/v4/compiler/*.dag`, produces bit-identical output (rung 7)
5. TestClaim suite passes (rung 8)
6. Hand-authored Rust is not editable authority (reproduction-proven)

Wave 1 + Wave 2 + most of Wave 3 are the work-on-the-path toward these six. Wave 1 alone closes none of them — it produces the first leaf-model verdict (predicate-4-adjacent), unblocks ci.dag (predicate-1-adjacent), and lands Upsert<T> substrate (predicate-2/3-adjacent). The narrowed "rung 7 = v4-done" framing was retracted in PR #3938 §8 D4 and remains retracted here.

---

## §8. What this doc is NOT

- Not a re-design of any architectural commitment in PR #3938 or #3959. Those are landed authority.
- Not a complete plan for Wave 2/3. Operator engagement on §7 decisions shapes those waves before dispatch.
- Not a commitment to specific PR/commit IDs. The W1.x items reference owner managers; dispatch details emerge per manager.

---

## §9. Related artifacts

- `docs/planning/v4-correctness-ladder-2026-05-30.md` — 9-rung ladder + §10 SG worksheets + §11 manager lanes
- `docs/planning/v4-ci-overhaul-2026-05-30.md` — CI overhaul Phase 1.4/1.5/2/2.5
- `docs/planning/v4-leaf-model-verification-2026-05-30.md` — leaf-model verification framework
- `src/v4/TASKS.md §T-24` — CI overhaul ratified phase plan + v4-done definition
- `dsl/std/patterns.dag` UPSERT<T> — operator canon for Upsert<T> pattern
- PR #3941 / `docs/audit/v4-close-interrogation-validation-2026-05-30.md` — 346-probe two-axis disposition
- `docs/audit/v4-rustc-error-catalog-2026-05-29.md` — 7951-error catalog reframed as substrate-gap diagnosis
