# SG-3g-b — `lower_helpers` wire-in (real receipt) `(M)`

## Context

PR #612 (SG-3g) landed `lower_helpers.dag` + generated helper + ratchet as **staging only** — explicitly framed in the PR body: *"Lane is not converged until the real lowerer consumes the generated helper; wire-in is blocked on two separate prerequisite lanes, neither of which is SG-3g's work."*

The slice chosen: `expr_span(SurfaceExpr) -> SourceSpan` — a pure match over 11 named-field variants returning each variant's `span` field. Used at **16 sites** across `src/v3/compiler/src/lower.rs`.

SG-3g-b is the wire-in: replace the 16 hand-authored `expr_span` call sites in `lower.rs` with calls to the generated helper. That's what turns #612 from "staging exists" into "real receipt."

## Read first

- PR #612's body — explicit framing of what staged vs what's parked
- `src/v3/lenses/lower_helpers.dag` — the authority source
- `src/v3/compiler/src/lower_helpers_generated.rs` — the generated helper
- `src/v3/compiler/src/lower.rs` — the 16 call sites (grep for the current `expr_span` pattern or the 11 `SurfaceExpr` variant matches that SG-3g replaced)
- The two prerequisite blockers named in #612's body (they're NOT blocking wire-in; just naming them so this lane doesn't accidentally widen into them):
  - `SurfaceLiteral → LiteralBits` rename gap
  - `render_variant_constructor` external tuple variant handling

## Work

1. **Identify all 16 `expr_span` call sites** in `lower.rs` (grep for the pattern, verify count matches #612's body claim).
2. **Replace each call site** with a call to the generated `lower_helpers::expr_span(...)` fn. Import the generated module as needed in `lib.rs` / `lower.rs`.
3. **Delete the inline `SurfaceExpr` span-extraction match** from `lower.rs` if it exists as a local helper (it should; that's what the 16 sites were presumably duplicating).
4. **Verify** lowering behavior is bit-identical: existing lowering tests pass; `m1_substrate_test`, `m2_*`, and any lane-related tests stay green.
5. **Check DB-8 fixed-point** — `self_host_fixed_point` on the fixture must still converge bit-identically.

## Acceptance

- Zero hand-authored `match` expressions in `lower.rs` that extract `.span` from `SurfaceExpr` — all go through `lower_helpers::expr_span`
- The 16 call sites named in #612 all updated
- Lowering tests green
- DB-8 fixed-point converges bit-identically to pre-wire-in
- `lower_helpers.dag` → `lower_helpers_generated.rs` freshness ratchet still green
- PR body explicitly frames this as **SG-3g-b wire-in receipt** (not SG-3b proper, not full `lower.rs` retirement)

## STOP-AND-ESCALATE

- If the 16 call sites aren't all using the identical match pattern — the slice chosen may be narrower than expected. Surface the actual count and variant coverage.
- If wire-in reveals that `expr_span` needs access to `Dag` state that the generated helper can't see — the slice wasn't as pure as #612 thought. Surface the state dependency; don't work around.
- If DB-8 fixed-point drifts — bytes must match. Don't adjust the golden; root-cause the drift.
- If wire-in forces touching `parse.rs` or `infer.rs` — wrong scope, STOP.

## Non-goals

- **Not retiring `lower.rs`** — this is one slice wire-in, not full retirement. `lower.rs` keeps ~99% of its logic hand-authored until more slices land.
- **Not extending the slice** — pure `expr_span` only; don't pull adjacent functions into this PR.
- **Not addressing the two named blockers** (`SurfaceLiteral → LiteralBits`, `render_variant_constructor`) — those get separate ROADMAP debt rows. Not this lane.
- **Not touching `lower_helpers.dag`** — the authority shape landed in #612 and should not need revision for wire-in.

## Size

M. Mechanical replacement of 16 call sites with bit-identical behavior preservation. Small scope; gated by thorough verification that lowering output is unchanged.

Expected LOC delta: ~-20 to -40 (the inline match pattern deduplicates into 16 call sites to the generated helper) + ~5 for imports. Net small negative.

## Dispatch note

Director reviews. Key acceptance signal: DB-8 fixed-point bit-identical + lowering test suite green. If either fails, STOP and diagnose before re-trying.

After this ships: ROADMAP SG-3g lane transitions from "partial-staging" to "prototype receipt landed." The next SG-3 work would be: additional lower.rs slices (SG-3g-c, d, ...) following the same pattern, eventually heading toward SG-3b proper.
