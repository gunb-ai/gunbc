# R2 Structure

**Status:** `PROPOSAL` — pending user sign-off + R1 closure + promotion to `ROADMAP.md` as `## Release R2 Program` section.

**Authority:** single-source while open. Amendments before promotion land in this doc. After promotion, amendments follow the same discipline as R1's `## Release R1 Program` section (director-authored PRs with manager acknowledgement).

**Scope naming note:** `docs/db-history/db-18.md` uses "R2 carrier" as internal DB-stage nomenclature that predates release-level R# naming. Our release-level R2 (this doc) is unrelated to DB-18's stage label; no collision of meaning, just of string.

## Summary

R2 is the **close-everything** release. Every remaining Tier-1 thesis claim not closed by R1 lands here. The two co-anchor claims — **Grounding Completeness** and **Lens Purity by Construction** — are joined by modeling-faithfulness dissolution, substrate prereqs, remaining impossible-bug classes, self-hosting shim-floor close, and tests-as-data closure.

The program's velocity signal (R1 closing in ~16 hours) drives two framing decisions: (1) scope is aggressive rather than deferred; (2) coordination is light-touch, throughput-oriented.

## Program count — 2 active releases total

- **R1** closing.
- **R2** = close-everything.
- **R3** reserved as *escape hatch only*, for items that genuinely cannot close in R2 despite honest effort. At R1's velocity, invocation should be rare and itself signal a problem worth examining.

Post-R2 is external work (adoption, documentation, community) — not on the thesis-claim release ledger.

## Goals

1. **Grounding Completeness** — target-side primitive types for Rust/Python/Go structurally declared; coercion via inhabitance search; Track-13 dissolution. Inherits from `ROADMAP.md:149` "Post-R1 Program — Grounding Completeness" → promotes to R2 lane `T-Ground`.

2. **Lens Purity by Construction** — every lens body `.dag`-authored; kernel closure replaces reviewer-convention. `lens_producer_files_remaining` gate (introduced via PR #752) lands at zero.

3. **Self-hosting shim-floor close** — T-PB-A non-test census reaches ≤5 irreducible shims per `docs/design-pure-bootstrap.md`; T-PB-B outside-residual-zero (per `TESTING.md §Post-R2 shape`); compiler-std consolidation ratchet → 0.

4. **Modeling-faithfulness dissolution** — three Tier-1 type-refinement gaps close:
   - Surface int-literal magnitude at concept layer (P4 row on `ROADMAP.md`; originating analysis on PR #745)
   - `Secret<T>` nominal-opaque graduation (`ROADMAP.md` post-merge-debt section, 2026-04-23 thesis-doc surface)
   - `Dimension<Carrier>` typed value wrapper with phantom-parameter unit-mismatch enforcement (ibid.)

5. **Substrate prereqs** — named as explicit R2 lanes unblocking Goal 4:
   - DB-18 parametric algebra attachment (unblocks `Secret<T>` nominal construction restriction + `Dimension` phantom-parameter arithmetic; see `docs/db-history/db-18.md` for Part-2 scope; Part-3+ is the R2 work)
   - Cardinality-substrate (unblocks `Cardinality(element, Exact(n))` carrier for fixed-width types, per P4 row + the Grounding blocker column at `ROADMAP.md:162-164`)

6. **Remaining R2+ impossible-bug classes** — three classes currently tagged `[R2+]` at `ROADMAP.md:72` (THESIS §"Enumerable impossible-bug classes" is the authority on scheduling tags):
   - Nested-optional flatten
   - Unhandled diagnostic paths
   - Unenumerated effects

7. **E-family carrier port closure** — E-I finish → E-P → E-M → §6a per-method-metadata call, per `docs/design-substrate-carrier-port-program.md`. Per-method-metadata option-pick deferred in R1; decides here.

8. **R2 closure demo** — simple "it runs" artifact per lane close. Director-coordinated. No dedicated demo lane (see Demo discipline below).

## Manager structure

**2 standing managers + Director.** Count = concurrent critical paths (one per manager).

### Grounding Manager

Continues `docs/briefs/grounding-manager.md` (refreshed for R2 scope on promotion). Owns T-Ground sub-program.

- **Critical path:** T-Ground-Pilot → T-Ground-Engine → T-Ground-Tests → T-Ground-Dissolve.
- **Fill queue:** T-Ground-Rust, T-Ground-Python, T-Ground-Go (3-way parallel after Pilot validates).

### Structural Close Manager

New brief at `docs/briefs/r2-structural-close-manager.md` (to author on promotion). Consolidates former R1 Self-hosting + Substrate + Testgen-tail authorities. Naming rationale: scope covers E-family carrier port + substrate prereqs + modeling-faithfulness + shim floor + lens migration + testgen predicate wiring + impossible-bug class closure — "self-hosting" is accurate for only a subset, so "Structural Close" names the actual scope honestly.

- **Critical path:** T-EFamilyClose (E-I finish → E-P → E-M) + §6a metadata pick.
- **Fill queues** (all file-level or item-level parallel; any available worker picks top-priority unblocked):
  - T-LensMigration (per-file parallel)
  - T-ShimFloor (per-file parallel)
  - T-Modeling (int-lit / Secret<T> / Dimensions — 3-way parallel; Secret<T> + Dimensions block on T-Substrate DB-18 sub-lane; int-lit blocks on T-Substrate cardinality-substrate sub-lane)
  - T-Substrate (DB-18 parametric algebra; cardinality-substrate — 2-way parallel; unblockers for T-Modeling)
  - T-ImpossibleBugs (3 remaining classes — sparse; fills when other fill queues are saturated)

### Director (ad-hoc)

- R1 residual closure surveillance (none expected per all-gates-green closure criterion).
- R2 demo coordination: surfaces "it runs" artifacts at each lane close to user.
- Cross-manager dependency surfacing when critical paths block.
- Weekly dependency health check: which lanes are within 1 step of unblocking? Which workers are on fill vs. ready? Are bottlenecks compounding?

## Lane structure

| Lane | Size | Manager | Covers |
|---|---|---|---|
| T-Ground | XL | Grounding | Full T-Ground-* sub-program (Goal 1) |
| T-LensMigration | L | Structural Close | Every lens producer `.rs` → `.dag` (Goal 2) |
| T-EFamilyClose | M | Structural Close | E-I finish + E-P + E-M + §6a (Goal 7) |
| T-ShimFloor | M | Structural Close | T-PB-A non-lens reductions; T-PB-B outside-residual-zero (Goal 3) |
| T-Modeling | M | Structural Close | int-lit / Secret<T> / Dimensions (Goal 4) |
| T-Substrate | M | Structural Close | DB-18 parametric algebra; cardinality-substrate (Goal 5) |
| T-ImpossibleBugs | S | Structural Close | nested-optional flatten / unhandled-diagnostic-paths / unenumerated-effects (Goal 6) |
| T-Demo | S | Director (ad-hoc) | R2 closure demo artifacts per lane close (Goal 8) |

## Dependency DAG

```
T-Ground:         Pilot → {Rust, Python, Go} → Engine → Tests → Dissolve
T-EFamilyClose:   E-I (in flight) → E-P → E-M → §6a pick
T-Substrate:      DB-18 parametric algebra  ─┐
                                              ├─→ unblock T-Modeling Secret<T>, Dimensions
                  cardinality-substrate  ─────┘   (cardinality for int-lit narrowing)
T-Modeling:       int-lit ← cardinality-substrate
                  Secret<T> ← DB-18
                  Dimensions ← DB-18
T-LensMigration:  per-file independent (any worker)
T-ShimFloor:      per-file independent (any worker)
T-ImpossibleBugs: 3 independent classes (any worker)
T-Demo:           per-lane artifact (trails each lane close)
```

Parallel-capable work at any time ≥ N workers × fill-queue depth (5 fill queues on Structural Close Manager + 3 on Grounding).

## R1 closure criteria

**All R1 gates green.** R1 closes when all 9 lane gates named at `ROADMAP.md:61-73` evaluate green, including omni-emit (`emit_omni_demo_fixtures_green`). No director-defined subset-close. Rationale: consistent with anti-deferral stance — tail-shaped work closes before R1 declares done; R2 doesn't inherit R1 residuals.

## Transition mechanics

1. **R1 gates green** → Director declares R1 closed.
2. **R1 residual sweep** — every open R1 ledger row gets an R1-or-R2 assignment. No orphaning. Done in the R1 closure PR. Expected to be short under all-gates-green criterion.
3. **Manager dissolution** — R1 Surface Manager archives (closure banner); R1 Testgen Manager archives (scope folds into Structural Close); R1 Substrate Manager archives (E-family folds into Structural Close); R1 Self-hosting Manager expands and renames to Structural Close Manager.
4. **R2 open** — this doc promotes to `ROADMAP.md` as `## Release R2 Program` section. `docs/briefs/r2-structural-close-manager.md` authored. `docs/briefs/grounding-manager.md` refreshed for R2 scope.

## Demo discipline — visibility as structural requirement

Simple "look, it runs" or "before/after analysis" artifact ships with each lane closure PR. Director coordinates surfacing to user. No time-based cadence; gate-close natural rhythm at R1 velocity is enough.

Forms that qualify:
- Running artifact + 1-paragraph "what this demonstrates"
- Before/after: "this program didn't compile; now it does"
- Census snapshot: "retired N hand-Rust files this milestone"
- Diagnostic demonstration: "here's a bad program, here's the error, here's the fix suggestion"

Purpose: proof-of-work visibility at director cadence. Without it, program slips invisibly over long horizons.

## Decisions locked

- **Goal 4 in R2** (not R3+). User anti-deferral stance; Director's "defer to R3+" counter reviewed and overridden.
- **R1 closure criteria = all-gates-green**. User anti-deferral stance.
- **Demo cadence = gate-close natural rhythm**. Simple artifact per close; no time-based schedule.
- **Manager count = 2 + Director**. Adjustable to 3 mid-R2 if Structural Close fill-queue depth becomes unmanageable (naming for a hypothetical third: "Modeling Manager" — would take T-Modeling + T-Substrate + T-ImpossibleBugs).
- **R2 includes substrate prereqs explicitly** (DB-18, cardinality-substrate) per user's (i)-over-(ii) preference: honest scope over tight scope.

## Open calls — none

All prior open calls resolved.

## Cross-refs

- Parent: `ROADMAP.md` (R1 program at `:15`; Grounding Completeness post-R1 at `:149`; Tracked-debts ledger at `:291`).
- Substrate design: `docs/design-substrate-carrier-port-program.md` (E-family lanes + §6a per-method-metadata).
- Self-hosting anchor: `docs/design-pure-bootstrap.md` (≤5 shim floor + SG census).
- Thesis: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags authority); `THESIS.md §"Thesis claims — complete list"` (Tier-1 claim lineage).
- Lens capability: `docs/v3-lens-capability-register.md` (per-lens capability tracking).
- DB history: `docs/db-history/db-18.md` (DB-18 Part-2 shipped; Part-3+ is R2 work).
- Related PRs: #745 (P4 int-literal row — substrate motivation for T-Modeling), #752 (T-PB-A lens-producer priority slice — substrate motivation for T-LensMigration gate).
