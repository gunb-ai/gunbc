# gunbc — Roadmap

High-level task tracker for **gunbc** (the compiler/language we are building). This is the *what's in
flight*, not the *why* — `DESIGN.md` remains the single source of truth for rationale, and every item
here must stay consistent with it. Product / ctrl features are tracked elsewhere, not here.

This file is a convenience index, **not a second authority**: a task's real state is its branch/PR and
the marks on the substrate (§3 single-authority). Keep entries terse; link the PR, not the prose.

---

## Now (in flight)

- **Self-host fixed point** — v2 compiler compiling its own `src/v2` to a bit-identical fixed point.
  - Lane 2a match / record-construct / coproduct-destructure end-to-end — landed (#5057).
  - Lane 2b generic instantiation substrate — landed (#5064).
  - Lane 3a `SourceRootIngest` (real multi-file host read → content-addressed manifest) — in flight (#5126).
  - Lane 3a.2 resolver / `QualifiedName` materialization — in flight (merry-crab).
- **Route C — cross-tree resolver unification** — module-qualify type/constructor names so closures span
  the `dsl/` and `src/v2/` trees; then de-fork the duplicated stds (FreeMonoid, `content_hash`, …).
  Census-sized, sequenced *after* 3a.2. (bold-bee; depends on the fork census below.)
- **Fact-cardinality lens** — one parameterized lens subsuming the four problem-classes
  (parallel-ledger / anemic-leaf / fused-realization / fabricated-green) into the single invariant
  "exactly one grounded authority per fact"; ratchet gate; emits the cross-tree fork census. (#5124)
- **CI wall-time reduction** (~27 min on main → target minutes) — three independent prongs:
  - P1 infra: repair the broken cargo `target/` cache + collapse the double cargo build into one
    invocation (~49% of the time). *Needs an owner.*
  - P2: make `dsl_compile_clean` soundly memoizable — declare `content_hash(dsl-tree)` as the shell op's
    input, then replay via RecordedFixture (#5090). Naive memo today would be fail-open. (~23%)
  - P3: resolver resolve-time blowup, see Known issues. (~15%)

## Next

- **Caching via Realization** — inhabit the `realize` kernel so memoization is *derived* from purity +
  content-key, not declared. `cache_interface.dag` is modeled but un-inhabited; ParseTable (#5093) is
  inhabitant #1. CI verdict memo (P2 above) is a consumer.
- **§3 extdeps/std de-conflation** — split interface-shape / transport / policy across the verified
  backlog (react_markup XSS fork, transport-fused service ops, nickname enums); one follow-up program.
- **CI floor as one `.dag` binary** — generate `ci.yml` from a `.dag` model instead of hand-editing it
  (#5104 was a partial step; `dsl/extdeps/github/ci.dag` still records `ci.yml` as hand-edited), and finish
  the glob-discovery corpus cutover so a `dsl/` file outside the hardcoded closures cannot stay green-while-broken.
- **Axioms in `.dag` + a syllogism lens** — model DESIGN §1 (A1–A3) and have a lens enforce every claim
  as a consequence-chain back to an axiom (no orphan, no cycle). (DESIGN open thread.)

## Known issues / debt

- **Resolver resolve-time blowup** — `budget_roster_completeness_test.dag` resolves in ~290s
  (machine-independent) vs its near-twin at 0.5s for the same item count (~500×). Being characterized for
  complexity class; if algorithmic it is a **Lane-4 self-host blocker**, not just CI cleanup. (merry-crab)
- **Resolve blowup is an UN-ENROLLED complexity violation — not a new issue, not a new lens.**
  `complexity_lens(n: Node)` (`lens/complexity.dag:314`) already folds an arbitrary body and rates this
  exact class (nested fold → polynomial degree → dominance RED). The budget roster is already *designed*
  to enroll the compiler's own stage bodies — its header reserves `02_parse` / compiler-stage subjects
  for "COMPREP waves 2+ / self-host breadth" — but today enrolls only wave-1 (source-bridged add). So
  the ~500× resolve blowup is a complexity violation the existing gate would catch once resolve is a
  subject; the gap is **enrollment / self-host breadth, not a missing lens**. Fix = enroll the resolve
  stage as a complexity-budget subject-producer (the planned wave-2+ work); **no scaffold, no new lens.**
  Caveat: this catches it iff the super-linearity is in the *modeled* resolve (`03_resolve.dag`); if it
  lives only in the Rust seed's execution, that is a model↔realization fidelity gap (§7), still not a
  new lens. Folds into Lane-4 self-host; merry-crab's characterization decides which.

## Recently landed (context)

- #5111 cost-model loop zero-absorption fix + bind/branch/loop budgets wired.
- #5098 grammar-inverse emit generalized to a structural reverse-parse fold.
- #5089 dsl whole-tree compile-clean CI gate.
- #5090 RecordedFixture hermetic record/replay seam.
- #5101 CI floor consolidated into one composed `.dag` run (`ci_floor_gates.dag`).
