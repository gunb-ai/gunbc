# Worker brief — Substrate T-LAS Demo: complexity-contract compile-error

**Sub-issue**: gunbc#1952 (parented under #1939 Substrate Mgr lane).
**Authority**: `docs/design-lens-application-surface.md` §"Closure gate" line 292 (`complexity_violation_compile_error_demonstrated`); V execution-split brief at `docs/briefs/r3-v-t-lens-application-surface-execution-split-worker.md` Slice B B1 row #92.
**Closure predicate**: §1.8 gate #92 `complexity_violation_compile_error_demonstrated`.
**Status**: **execution brief — no canvas needed**; design-lock specifies fixture shape verbatim. Direct dispatch on T-LAS Slice A landing + T-LBP complexity-lens BEHAVIORALLY COMPLETE.

## Scope

Land the **first T-LAS demonstration** per `design-lens-application-surface.md:292`:

> "complexity_violation_compile_error_demonstrated — a TestClaim that constructs a function with O(n²) body + a lens application requiring O(log n) + asserts a Diagnostic is produced."

Concrete fixture shape per design-lock §"Apply complexity lens with Enforce" (lines 272-292):
- Function with O(n²) body (e.g., nested loop on input list)
- `apply_lens(complexity, fn, Enforce { budget: LogN, diagnostic_severity: Error })`
- Compiler emits a `Diagnostic` with severity `Error` because actual complexity (O(n²)) exceeds named budget (O(log n))

## Hard prerequisites

- **T-LAS Slice A landed** (gates #88-#91): `EnforcedApplication<Output, Budget>` + `IntrospectApplication<Output>` + `SectionRef` + `LensEnforcement<ComplexitySummary, Budget>` + Enforce-mode diagnostic routing through `DiagnosticSeverity` per design §3 + INVARIANTS C-8.
- **T-LBP complexity-lens BEHAVIORALLY COMPLETE**: `complexity_lens: Lens<ComplexitySummary>` returns work/span/asymptotic_class/work_certainty/span_certainty per design line 168. `complexity_lens_behaviorally_complete` (gate #79) PASSING.

If either prerequisite is not landed at worker-pickup time: **STOP-and-PING** the Mgr; do NOT author a placeholder fixture against a NotYetImplemented carrier.

## Acceptance gates (same-slice, all must pass)

1. **Fixture program** lands at `src/v3/compiler/tests/integration/` (or appropriate fixture location): function with provably-O(n²) body + `apply_lens(complexity, <fn>, Enforce { budget: <O(log n)-class>, diagnostic_severity: Error })`. Use a simple-enough O(n²) construction that the complexity lens can prove the bound (nested loops over input list — NOT a recursive call where T-E-P producer broadening might undercoverage).
2. **TestClaim** asserting the compile produces a `Diagnostic` with severity `Error` referencing the lens-application's `SectionRef` declaration. Per V execution-split brief: TestClaim shape is Verification-owned (`TESTING.md`); coordinate with Verification Mgr (#2075) at PR-open time for shape alignment.
3. **Diagnostic content structurally validates** the violation routing per gate #91 — severity = Error (not Warning); attribution = lens-application site; mentions actual complexity vs budget.
4. **No regression on existing complexity-lens-behaviorally-complete tests**: gate #79 cementing test (per #1950 brief landing) stays green.
5. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean.
6. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.
7. **§1.8 gate #92 advances** to PASSING / executable status; ROADMAP row text refresh on T-Lens-Application-Surface lane row noting demo landing.

## STOP / PING criteria

- **STOP** if either hard prerequisite (T-LAS Slice A or T-LBP complexity BEHAVIORALLY COMPLETE) is not landed at pickup time. Surface to Mgr; do NOT proceed against a partial substrate.
- **STOP** if the O(n²) fixture body forces T-E-P producer broadening to handle additional `SubValueRelation` variants. Per `feedback_state_space_vs_behavioral_invariants`, demo fixtures should exercise existing producer coverage, not force new producer work; pick a simpler fixture (nested loops over a list literal) or surface scope-creep.
- **STOP** if Diagnostic-emission semantics for Enforce-mode require additional substrate beyond `LensEnforcement<Output, Budget>` (e.g., a Diagnostic-routing helper) — surface to Mgr; that's Slice A scope-extension.
- **PING** Verification Mgr (#2075) at PR-open per V execution-split brief: TestClaim shape + corpus fixture authority is Verification-owned. Coordinate same-slice or via cross-Mgr sequence (Verification authors TestClaim shape; this PR consumes).

## Cross-Mgr coordination

- **Verification Mgr (#2075 / wise-bear-525)**: TestClaim authoring + diagnostic-shape assertion per V execution-split Slice B B1. PING at PR-open; same-slice consumption preferred for crisp landing.
- **PB Mgr (#2074)**: no expected handoff (this is a leaf demo PR; doesn't touch lens-producer-retirement scope).

## Related siblings

- #1953 T-LAS CRDT cost basis demo (gate #93) — sibling brief; same Slice B B1 row.
- #1954 T-LAS memory-peak cost basis demo (gate #94) — sibling brief; same Slice B B1 row.
- All three demos share the same hard-prerequisite pattern (Slice A + relevant lens BEHAVIORALLY COMPLETE) and same TestClaim/Diagnostic shape; could dispatch as 3 sequential PRs OR a single multi-demo PR (Worker's call at dispatch time, with Mgr ratification if going multi-demo).

## Worker pin (Mgr disposition)

Demonstration-tier work — different precedent owners than substrate-fact-introduction. Suggest **valiant-ant-72** (per quick-crab freed-pool list) or workers with prior demo/fixture experience. Final pin at dispatch.

## Auto-spawn caveat

Per Director's standing note + cache-staleness cluster ctrl#217: HOLD dispatch until auto-spawn fix lands. This is M-sized (single fixture + TestClaim wiring) per design-lock simplicity — could be candidate for surgical-recreate path if Director ratifies for cascade unblock.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director endorsement of T-LAS demos pre-staging via execution-brief direct path (canvas-first not needed; design-lock specifies shape).
