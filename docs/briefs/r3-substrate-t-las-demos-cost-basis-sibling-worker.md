# Worker brief — Substrate T-LAS Demos: cost-basis sibling (#1953 + #1954)

**Sub-issues**: gunbc#1953 (CRDT cost basis demo) + gunbc#1954 (memory-peak cost basis demo). Both parented under #1939; same hard-prerequisite + TestClaim shape pattern as #1952.
**Authority**: `docs/design-lens-application-surface.md` §4.2 (CRDT cost basis, line 294) + §4.3 (memory-peak cost basis, line 315); V execution-split brief at `docs/briefs/r3-v-t-lens-application-surface-execution-split-worker.md` Slice B B1 rows #93 + #94.
**Closure predicates**: §1.8 gate #93 (`crdt_cost_basis_demonstrated`) + #94 (`memory_peak_cost_basis_demonstrated`).
**Status**: **execution brief — no canvas needed**; design-lock specifies fixture shape verbatim for both. Sibling pattern with #1952 (`r3-substrate-t-las-demo-complexity-contract-compile-error-worker.md`).

## Scope (sibling-of-#1952 framing)

This brief covers BOTH cost-basis demos. Worker's call at dispatch time on whether to ship as one multi-demo PR or two sequential PRs (per Director note at #1952 brief: "could dispatch as 3 sequential PRs OR a single multi-demo PR (Worker's call at dispatch time, with Mgr ratification if going multi-demo)"). If multi-demo, ping Mgr for ratification at PR-open.

### #1953 — CRDT cost basis demo (§4.2 verbatim)

```dag
apply_lens(cost, my_crdt_field, Enforce {
  budget: SymbolicCost { per_op: O_log_replicas }
  diagnostic_severity: Error
})
```

- `section: DeclarationScope { declaration: my_crdt_field }` (same shape as §4.1)
- Per-write cost basis is **cost-lens-owned authority** (separate fact from lens-application config) — see `docs/audit/t-user-authored-cost-basis-discipline-worked-examples.md`. Worker reads that audit before authoring fixture.
- Compiler-side processing: cost lens reads CRDT field's per-write cost; when composing with surrounding lens applications via T-CostLens-Composition, per-op budget participates. Loop writing N times → O(N·log replicas) by composition.

### #1954 — Memory-peak cost basis demo (§4.3 verbatim)

```dag
apply_lens(cost, my_memory_intensive_function, Enforce {
  budget: SymbolicCost { dimension: Memory, per_call: O_input_size }
  diagnostic_severity: Error
})
```

- Cost lens carries **multi-dimension** `SymbolicCost` (per existing `Dimension<SymbolicCost>` substrate from T-CostLens-Composition).
- Time-dimension and memory-dimension independent: both lens applications can apply to the same function with different budgets per dimension.
- Memory-peak composition semantics (max / live-overlap / similar) MUST be declared in the cost-lens authority — generic application config does NOT decide peak algebra.

## Hard prerequisites (same as #1952 + cost-specific)

- **T-LAS Slice A landed** (gates #88-#91): `EnforcedApplication<Output, Budget>` + `IntrospectApplication<Output>` + `SectionRef` + `LensEnforcement<SymbolicCost, Budget>` + Enforce-mode diagnostic routing per design §3 + INVARIANTS C-8.
- **T-LBP cost-lens BEHAVIORALLY COMPLETE** (gate #80 `cost_lens_behaviorally_complete`): per-target realization cost reading + Lookup<SymbolicCost> composition per T-CostLens-Composition (γ ratified per gunbc#828 #issuecomment-4395691775).
- **#1954-specific**: `Dimension<SymbolicCost>` substrate landed (per Director ratification at gunbc#828 #issuecomment-4395691775 ask #3 — deferred to separate Dimensions sub-lane). If Dimensions sub-lane has not landed at pickup time, **STOP-and-PING** the Mgr; #1953 may be dispatchable independently while #1954 holds.
- **#1953-specific**: cost-basis-declaration substrate per `t-user-authored-cost-basis-discipline-worked-examples.md` audit. Worker grep-verifies the audit's named substrate exists at HEAD before authoring.

## Acceptance gates (same-slice, all must pass per demo)

For EACH of #1953 + #1954:

1. **Fixture program** lands at `src/v3/compiler/tests/integration/`: function/declaration with structured cost basis + `apply_lens(cost, <target>, Enforce { budget: <expr>, diagnostic_severity: Error })` per §4.2/§4.3 verbatim.
2. **TestClaim** asserting either Diagnostic-emission (when actual exceeds budget) OR clean-compile (when actual ≤ budget). Per V execution-split: TestClaim shape Verification-owned; PING Verification Mgr (#2075).
3. **Diagnostic structural validation** per gate #91 violation routing — severity = Error; attribution = lens-application site; mentions actual vs budget composition.
4. **No regression on T-LBP cost-lens cementing** (gate #80) — coordinate timing with #1951 cementing test.
5. Bootstrap regen + workspace tests + clippy clean.
6. **§1.8 gate advances to PASSING**: #93 for #1953; #94 for #1954.

## STOP / PING criteria

- **STOP** if either hard prerequisite is not landed at pickup. Same discipline as #1952.
- **STOP** if `Dimension<SymbolicCost>` substrate is not landed (specific to #1954). #1953 may proceed independently.
- **STOP** if T-CostLens-Composition (γ-ratified worker brief at #2114) hasn't been worker-dispatched + landed before these demos pick up — cost lens BEHAVIORALLY COMPLETE is the hard precondition.
- **STOP** if memory-peak composition semantics (max / live-overlap) are not yet declared in cost-lens authority at HEAD (specific to #1954) — surface to Mgr; that's cost-lens-authority scope, not demo scope.
- **PING** Verification Mgr (#2075) at PR-open: TestClaim shape coordination per V execution-split brief.

## Cross-Mgr coordination

- **Verification Mgr (#2075)**: TestClaim authoring; same-slice consumption preferred per #1952 sibling pattern.
- **Substrate Mgr (this lane)**: cost-lens authority changes (composition semantics for memory-peak per §4.3) may surface as scope-creep; Mgr coordinates.

## Worker pin (Mgr disposition)

Same precedent as #1952 (demonstration-tier, distinct from substrate-fact-introduction). **valiant-ant-72** or other workers with prior demo/fixture experience. Multi-demo dispatch consolidation = same worker for both #1953 + #1954.

## Auto-spawn caveat

Per Director's standing note + cache-staleness cluster ctrl#217: HOLD dispatch until auto-spawn fix lands. Each demo is M-sized; multi-demo PR is L-sized. Surgical-recreate path candidate if Director ratifies for cascade unblock.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director endorsement of T-LAS demos pre-staging via execution-brief direct path. Sibling of `r3-substrate-t-las-demo-complexity-contract-compile-error-worker.md` (#1952); share hard-prerequisite + TestClaim + Diagnostic-validation shape pattern.
