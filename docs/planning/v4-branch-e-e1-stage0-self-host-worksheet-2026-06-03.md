# v4 Branch E E.1 Stage0 Self-Host Worksheet — Bit-Identical Re-Emit Acceptance Shape

> **Status:** W4-PREP DESIGN ONLY — no self-host runner realization, no stage0 replacement, no Rust
> dissolution in this note. E.1 implementation is **W4.7**, gated below.
> **Work item:** `node://adhoc-56d1be0a897d42f9-reopen` — Branch E Mgr (`royal-badger-408`).
> **Authorization:** PM (`nimble-dove-733`) 2026-06-03 — "HOLD impl; author the E.1 stage0 self-host
> worksheet / RR-E follow-on (bit-identical re-emit acceptance shape) so you're ready the moment W4
> dispatches + Rung 1 lands."
> **Relationship to RR-E:** this is the E.1 follow-on to the merged RR-E worksheet (#4288) and the E.3
> DAG-regen design closure (#4302). RR-E/E.3 settled the *source-authority round-trip* shape; this note
> settles the *bit-identical re-emit fixed-point* shape that sits on top of it.

## Why this is a worksheet (not a receipt)

The v4 self-host substrate model **already exists and is single-authority** — this note does not
redeclare it. It records the **independent missing design fact**: the precise acceptance shape that
flips the already-modeled runner from fail-closed to live, plus the gate chain that licenses the flip.
The modeled artifacts this worksheet consumes (do not duplicate):

- `src/v4/compiler/self_host.dag` — `StructurallyEqual`, `BootstrapRoundtripEqual`, `FixedPointEqual`,
  `StageEmissionEquality`, `PromotionWitness`, and the scaffolded `self_host_fixed_point_validate`
  runner (returns `Rejected { self_host_runner_not_realized }` today — fail-closed).
- `src/v4/workflow/bootstrap.dag` — the four-stage plan `seed → stage0 → stage1 → stage2` with
  `fixpt stage1 == stage2`; `compiled_by` chain (`seed ← v2` used once, `self0 ← stage0`,
  `self1 ← stage1`; v2 never re-enters the loop); content-hash pins currently 🟡 placeholder
  Symbols/`Hash` data until T-15 B1 `content_hash` supplies merkle digests.
- `src/v4/test/claim/self_host/claim_t15_self_host_fixed_point.dag` — `EqualsClaim` over
  stage1-vs-stage2 emitted-rust digest carriers (B1 placeholder operands today; execution deferred to
  the T-22 eval / host harness `t_15_self_host_fixed_point`).
- `src/v4/test/claim/self_host/claim_rejects_until_substrate.dag` — `DiagnosticClaim` asserting the
  runner fails closed until execution substrate lands.
- `src/v4/test/claim/self_host/claim_runner_compiles.dag` — `CompilesClaim` that the runner shape is
  parse/compile-visible.

## §10.0-adapted worksheet

```text
Migration class:        E1-STAGE0-SELF-HOST (bit-identical re-emit fixed point; stage0 becomes a seed,
                        the .dag pipeline becomes the compiler-of-record)
Representative failure:  Declaring "self-host achieved" on a fixed-point hash that compared placeholder
                        Hash data, or on a runner that never executed the candidate stage against the
                        model corpus at runtime — i.e. stamping SELF_HOSTING.md §7 Step 3 green without
                        Step 1/Step 2 having actually run.
Immediate local patch:   Wire bootstrap_plan to alias stage1/stage2 hash pin Symbols so the structural
                        gate passes, then call that "fixed point"; or hand-diff two emitted-Rust dumps
                        in a script and stamp BootstrapRoundtripEqual without the runner executing.
Why forbidden:           INVARIANTS — a fixed-point claim over placeholder digests proves nothing about
                        convergence (bootstrap.dag header is explicit: structural wiring only, NOT proof
                        of fixed-point convergence). The acceptance is a runtime fact about emitted
                        bytes, not a structural alias. Cementing Rust into stage0 templates to keep a
                        census ratchet green is the explicitly-forbidden gaming shape (project spirit):
                        the ratchet is downstream of substrate migration, not a path to it.
DFS path:
  E.1 stage0 self-host rows — the bit-identical re-emit chain (SELF_HOSTING.md §7 adapted to v4):
    E1.1  seed (v2) compiles the v4 .dag pipeline source     -> stage0 binary        [seed used once]
    E1.2  stage0 compiles the SAME .dag pipeline source       -> stage1 binary        [self0 ← stage0]
    E1.3  stage1 compiles the SAME .dag pipeline source       -> stage2 binary        [self1 ← stage1]
    E1.4  ACCEPTANCE: digest(stage1) == digest(stage2)        -> FixedPointEqual       [the fixed point]
    E1.5  ACCEPTANCE: stage1 emits the model corpus and the
          re-emitted source round-trips per E.3/H.7.2          -> BootstrapRoundtripEqual
    E1.6  PromotionWitness = both sub-checks Hold              -> stage0 demoted to seed
  Acceptance reads existing types verbatim — FixedPointEqual.{emitted_stage,reference_stage} and
  BootstrapRoundtripEqual.{produced_stage,expected_stage} from self_host.dag. No new acceptance types.
Deepest unsound boundary:
  Two emitted-Rust artifacts can be byte-identical while NEITHER executes — a fixed point over a
  pipeline that does not actually run is a frozen tautology, not self-hosting. Equally: a runner can
  report Holds on placeholder Hash pins that were unified by bootstrap_plan_well_formed rather than
  computed by content_hash. Both hide the missing execution + digest substrate behind a green witness.
Systemic fix:
  E.1 acceptance requires THREE substrate facts to be live, in order:
    (1) Execution-Runnable keystone (Rung 1) — gunbc test executes TestClaims at runtime, so the
        self-host runner can actually run stage1 against the corpus and observe emitted bytes
        (PM-relayed; under royal-gull, PR #4353 — OPEN at time of writing).
    (2) T-15 B1 content_hash — merkle digests replace the placeholder Hash data, so digest(stage1)
        and digest(stage2) are computed, and bootstrap_plan_well_formed requires digest equality
        (stage1 != stage2 until convergence is proven), not symbolic alias.
    (3) candidate-generation substrate — the stage1 candidate Node fed to
        self_host_fixed_point_validate is produced by the pipeline, not hand-authored.
  Only when (1)+(2)+(3) are green does self_host_fixed_point_validate stop returning
  self_host_runner_not_realized and begin returning a real PromotionWitness. E.1 implementation is
  the body fill over the ALREADY-DECLARED runner — model-before-implement, the scaffold is the slot.
Non-goals:
  - No self-host runner realization, candidate generation, or digest computation in this PR (W4.7).
  - No edit to stage0 Rust to satisfy a census/footprint ratchet (gaming shape).
  - No claiming fixed-point convergence on placeholder Hash pins (bootstrap.dag is explicit it is not).
  - No Rust->DAG decompilation (R5 anti-scope); seed v2 compiles .dag source forward only.
  - No new acceptance types — E.1 fills the bodies of self_host.dag's declared scaffolds.
  - No reordering of seed-used-once: v2 compiles the seed stage0 exactly once and never re-enters.
Falsification probe:
  After (1)+(2)+(3) land, E.1 acceptance MUST fail closed if:
    a. digest(stage1) != digest(stage2) — non-convergence (real, not aliased, digests).
    b. stage1 did not execute at runtime (Rung 1 absent) — runner returns the not-realized diagnostic.
    c. the BootstrapRoundtripEqual sub-check is satisfied by a JSON-IR/emitted-Rust diff rather than
       the E.3/H.7.2 canonical .dag source round-trip (SourceAstEqual primary).
  A green PromotionWitness on placeholder digests, or with stage0 still authoritative, is the failure
  this probe rejects.
Metric allowed only as secondary:
  Hand-maintained stage0 Rust footprint / census delta. Secondary to the FixedPointEqual +
  BootstrapRoundtripEqual runtime receipt; census pressure is downstream of migration, never the path.
```

## Bit-Identical Re-Emit Acceptance Shape (the deliverable)

```text
seed (v2)
  --compiles--> stage0 binary            [E1.1; seed used once, never re-enters]
stage0
  --compiles v4 .dag pipeline source-->  stage1 binary   [E1.2; compiled_by self0 ← stage0]
stage1
  --compiles SAME .dag pipeline source--> stage2 binary  [E1.3; compiled_by self1 ← stage1]

ACCEPTANCE (both must Hold, at runtime, on computed digests):
  FixedPointEqual { emitted_stage: digest(stage2), reference_stage: digest(stage1) }   == Holds
  BootstrapRoundtripEqual { produced_stage: stage1_corpus_emit,
                            expected_stage: E.3/H.7.2 canonical .dag source round-trip } == Holds
  => PromotionWitness Holds => stage0 Rust demoted to bootstrap seed (E.4 dissolution may then begin).
```

This is SELF_HOSTING.md §7's three-step meta-circular test (compiled-pipeline-v1 == compiled-pipeline-v2,
byte-identical) projected onto the v4 four-stage `bootstrap.dag` plan and the `self_host.dag` witness
types. E.1 adds nothing to the model; it makes the already-declared `Holds` reachable.

## Gate Chain (W4.7 dispatch readiness)

| Gate | Provides | Status (2026-06-03) | Source |
|------|----------|---------------------|--------|
| W3.2 coercion realization | cross-target coercion the pipeline emit relies on | ✅ MERGED | #4359 |
| W3.4 G3.3–G3.4 spine | inference/grounding spine for full-tree emit | 🟡 building | bright-dove (PM-relayed) |
| Rung 1 Execution-Runnable | `gunbc test` executes TestClaims at runtime | 🔴 OPEN | royal-gull, PR #4353 |
| T-15 B1 `content_hash` | merkle digests replace placeholder `Hash` pins | 🔴 pending | bootstrap.dag T-20 dissolve-on T-15 |
| candidate generation | pipeline-produced stage1 candidate Node | 🔴 pending | self_host.dag runner input |

**E.1 cannot be dispatched until Rung 1 (#4353) lands** — "self-host can't run until TestClaims actually
execute at runtime" (PM 2026-06-03). T-15 B1 content_hash and candidate generation are co-requisite for
a *meaningful* (non-placeholder) acceptance; a structural-only green is explicitly forbidden above.

## Boundaries

- **Branch H / source authority:** owns the canonical `.dag` serializer
  (`target_serialize_source_from_model` in `src/v4/compiler/source_authority.dag`, H.7.2 / #4298). E.1's
  `BootstrapRoundtripEqual` sub-check consumes that round-trip; `SourceAstEqual` is the primary receipt,
  `SemanticIrEqual` secondary, JSON-IR/emitted-Rust non-substitute. E.1 does not author a parallel
  serializer.
- **Execution-Runnable keystone (Rung 1, royal-gull #4353):** owns runtime TestClaim execution. E.1
  consumes it; E.1 does not build the runtime transport.
- **T-15 / T-22 harness:** owns digest computation and the host eval harness
  (`t_15_self_host_fixed_point`). E.1's FixedPointEqual consumes computed digests; E.1 does not invent a
  digest scheme.
- **Branch C C.1–C.5:** `06_translate` / Shape-A scaling stays C-owned; E.1 consumes C.5 green for
  scaling beyond the first module and must not expand `05_emit` or co-own translate.
- **E.4 hand-Rust dissolution:** strictly downstream of a Holding `PromotionWitness`. stage0 stays
  authoritative until the fixed point Holds; census deltas are not the migration path.

## Next Dispatch Shape (when W4.7 opens)

```text
Rung 1 (#4353) lands + T-15 B1 content_hash live
  -> E1 body fill: self_host_fixed_point_validate executes stage1 against the model corpus
  -> E1.4 FixedPointEqual on computed digests (replaces fixed_point_scaffold Violates)
  -> E1.5 BootstrapRoundtripEqual via E.3/H.7.2 canonical source round-trip
  -> E1.6 PromotionWitness Holds; claim_t15_self_host_fixed_point flips from B1 placeholder to real
  -> E.4 hand-Rust stage0 demoted to seed; dissolution lane opens
```

The implementation PR is not allowed to report `PromotionWitness Holds` unless (a) the runner executed
at runtime, (b) the digests were computed (not aliased), and (c) the source round-trip used the H.7.2
canonical `.dag` source TargetModel.

## W4-Prep Acceptance Checklist

- [x] Bit-identical re-emit acceptance shape named over the existing four-stage `bootstrap.dag` plan.
- [x] Acceptance bound to the already-declared `self_host.dag` witness types — no new acceptance types.
- [x] Gate chain enumerated with current status; Rung 1 (#4353) named as the hard blocker.
- [x] Forbidden shapes named: placeholder-digest fixed point, stage0 Rust cementing, JSON-IR round-trip.
- [x] Falsification probe fails closed on non-convergence, no-runtime-execution, and IR-substitute.
- [x] No runner realization, candidate generation, digest computation, or Rust dissolution in this PR.
