# Invariant Retrospectives

This file is owned by the `gunbc_recent_invariants` worker. It scans the
last few days of `gunbc` `origin/main` commits and identifies recurring
invariants that the codebase appears to be rediscovering.

## Managed Summary

<!-- openclaw:recent-summary:start -->
- Last reviewed at: 2026-03-15T15:12:17-04:00
- Last reviewed head: `c77734be`
- Rolling window: 3 days
- Commits reviewed last run: 40
- Candidate invariants surfaced last run: 7
<!-- openclaw:recent-summary:end -->

## Managed Latest Retrospective

<!-- openclaw:recent-latest:start -->
### 2026-03-15T15:12:17-04:00

- Head ref: `origin/main` @ `c77734be`
- Rolling window: 3 days
- Commits reviewed: 40
- Candidate invariants surfaced: 7

#### Candidate Invariants

- Every pipeline stage must be fail-closed: error diagnostics stop downstream stages, and all upstream diagnostics are preserved in stage order.
- Boundary result types must encode success versus failure structurally; states like “artifact plus errors” or parallel optional fields for success and failure are invalid by construction.
- Missing or unresolved facts must never be fabricated as Unit, Unknown, empty strings, "*", empty collections, or dummy nodes; represent absence explicitly and emit diagnostics.
- Open language and compiler sets must be modeled structurally, not via string tags, duplicated keyword tables, or hardcoded method/type-name dispatch.
- Downstream stages must consume the authoritative typed or resolved output of the previous stage and must not re-declare or re-derive equivalent IR, type environments, or registries locally.
- Lowering and emission may translate representation but must preserve source semantics and encoded guarantees, including control-flow behavior, alias identity, provenance, and cardinality or non-emptiness constraints.
- Any choice derived from sets, maps, or graph order must use canonical ordering or raise an ambiguity diagnostic; hash iteration and incidental declaration order are not valid semantics.

#### Retrospective

**Assessment**
In the 3-day window ending March 14, 2026, `origin/main` mostly revisits the same compiler truths rather than isolated bugs. The repeated rediscoveries are: fail-closed stage contracts, no fabricated placeholders, structural representations over strings, single-authority IR/metadata, semantic preservation in emit/lower, and deterministic output.

**Recurring Themes**
- Diagnostics and gating were repeatedly repaired because stages kept allowing downstream work after upstream errors or silently dropping prior-stage diagnostics.
- Parser, resolver, typechecker, and emitter all reintroduced fabricated values: `Unit`, `Unknown`, empty strings, `"*"`, empty lists, and dummy AST nodes standing in for real failures.
- Multiple commits fixed places where code matched on strings or duplicated lookup tables instead of using the AST/type structure already available.
- The emitter repeatedly had to be pulled back toward consuming typed/resolved outputs instead of re-declaring or re-deriving the same facts from raw AST.
- Several fixes were about semantic drift in lowering/codegen: preserved shape was not enough if alias identity, non-emptiness, control flow, or field provenance changed.
- Determinism was a real correctness issue, not cosmetic cleanup; hash iteration and incidental ordering kept affecting emitted semantics or test stability.

**Candidate Invariants**
CANDIDATE-INVARIANT: Every pipeline stage must be fail-closed: error diagnostics stop downstream stages, and all upstream diagnostics are preserved in stage order.
CANDIDATE-INVARIANT: Boundary result types must encode success versus failure structurally; states like “artifact plus errors” or parallel optional fields for success and failure are invalid by construction.
CANDIDATE-INVARIANT: Missing or unresolved facts must never be fabricated as `Unit`, `Unknown`, empty strings, `"*"`, empty collections, or dummy nodes; represent absence explicitly and emit diagnostics.
CANDIDATE-INVARIANT: Open language and compiler sets must be modeled structurally, not via string tags, duplicated keyword tables, or hardcoded method/type-name dispatch.
CANDIDATE-INVARIANT: Downstream stages must consume the authoritative typed or resolved output of the previous stage and must not re-declare or re-derive equivalent IR, type environments, or registries locally.
CANDIDATE-INVARIANT: Lowering and emission may translate representation but must preserve source semantics and encoded guarantees, including control-flow behavior, alias identity, provenance, and cardinality or non-emptiness constraints.
CANDIDATE-INVARIANT: Any choice derived from sets, maps, or graph order must use canonical ordering or raise an ambiguity diagnostic; hash iteration and incidental declaration order are not valid semantics.

**High-Leverage Rubric/Process Adjustments**
- Add a boundary-contract checklist for every stage change: diagnostic channel present, fail-closed gating present, sum-type success/failure shape, and no duplicate local IR definitions.
- Add a fabrication audit to worker prompts: grep for `unit_type()`, `Unknown`, `""`, `"*"`, dummy spans/nodes, wildcard defaults, and empty-success fallbacks.
- Add an emit/lower review item that explicitly asks which source invariants are preserved: alias identity, non-empty containers, field provenance, async/error propagation, and resource/effect metadata.
- Add a determinism pass to the rubric: sort unordered collections, normalize structural signatures, canonicalize paths, and treat ambiguity as an error.
- Add a single-authority check for keyword tables, module maps, helper registries, and typed/resolved metadata so workers derive from existing structure instead of mirroring it.
<!-- openclaw:recent-latest:end -->

## Managed History

<!-- openclaw:recent-history:start -->
- 2026-03-15T15:12:17-04:00 reviewed `c77734be`; commits=40; candidates=7; **Assessment**
- 2026-03-15T15:08:11-04:00 reviewed `c77734be`; commits=40; candidates=7; **Assessment**
<!-- openclaw:recent-history:end -->
