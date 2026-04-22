# SG-3g-b — `lower_helpers` wire-in (real receipt) `(M)`

> **Status (2026-04-22): SHIPPED.** Prerequisite [SG-3f-e](sg-3f-e-parse-parse-surface-convergence.md)
> landed: `parse_generated.rs` re-exports the `parse_surface` Surface carriers, so
> `parse::SurfaceExpr` and `parse_surface::SurfaceExpr` are one type — no
> cross-module clone bridge for lowering. The live lowerer consumes the
> generated helpers: `src/v3/compiler/src/lower.rs` imports
> `crate::lower_helpers::{expr_span, item_span, pattern_binding_names}` and
> routes span extraction and pattern list projection through them. Registry /
> regen: `lenses/lower_helpers.dag` → `lower_helpers_generated.rs` (unchanged
> for wire-in). See `src/v3/compiler/src/lib.rs` (`lower_helpers` module) and
> `regen.dag` `lens_lower_helpers_entry`.
>
> The **A/B/C** type-convergence options in the Work section below are
> **historical** (pre–SG-3f-e). SG-3f-e implemented the “single Rust type per
> carrier” outcome; this lane then completed the mechanical wire-in and receipt
> checks.

## Live state (current tree)

Verifiable facts — satisfies **Documentation Describes Live State**
(`INVARIANTS.md` *Related rules*); use these to audit any prose in this file:

- `src/v3/compiler/src/lower.rs` — `use crate::lower_helpers::{expr_span, item_span, pattern_binding_names}`; live lowering calls the generated surface helpers.
- `src/v3/compiler/src/parse_generated.rs` — `pub use crate::parse_surface::{ … }` for Surface carriers, so `parse::SurfaceExpr` and `parse_surface::SurfaceExpr` are the same type.
- `src/v3/lenses/lower_helpers.dag` / `src/v3/compiler/src/lower_helpers_generated.rs` / `regen.dag` `lens_lower_helpers_entry` — authority → generated Rust (ratchet: SG-6 / integration tests).
- `src/v3/compiler/src/parse_generated.rs` also still defines
  `pub(crate) fn expr_span(expr: &SurfaceExpr) -> &SourceSpan` for **parser-
  local** hot paths (see `lib.rs` `lower_helpers` module comment) — that is
  *not* the generated `lower_helpers` helper; it is intentional and separate
  from the lowerer wire-in.

## Context (historical — pre-wire-in / PR #612)

PR #612 (SG-3g) landed `lower_helpers.dag` + generated helper + ratchet as **staging only** — explicitly framed in the PR body: *"Lane is not converged until the real lowerer consumes the generated helper; wire-in is blocked on two separate prerequisite lanes, neither of which is SG-3g's work."*

The slice chosen: `expr_span(SurfaceExpr) -> SourceSpan` — a pure match over 11 named-field variants returning each variant's `span` field. At #612, this appeared at **16 sites** in `src/v3/compiler/src/lower.rs` (count may have drifted; grep is authoritative).

SG-3g-b *was* the wire-in: replace the hand-authored call sites in `lower.rs` with the generated helper — that turns #612 from "staging exists" into "real receipt." **This is now done;** see **Live state** above.

> **Record:** Sections **Read first (historical)**, **Work**, **STOP-AND-ESCALATE**,
> and the lower portion of **Acceptance** below preserve the *original* lane
> definition, STOP gates, and pre-convergence blockers. They are **archival**:
> if anything disagrees with **Live state** or a grep of the current tree, trust
> the code.

## Read first (historical — pre-convergence / execution planning)

- PR #612's body — explicit framing of what staged vs what was parked. **"What did not land"** names the historical wire-in blockers.
- `src/v3/lenses/lower_helpers.dag` — the authority source
- `src/v3/compiler/src/lower_helpers_generated.rs` — the generated helper
- `src/v3/compiler/src/lower.rs` — `expr_span` / `item_span` / `pattern_binding_names` use sites
- **Resolved:** `parse` / `parse_surface` convergence (SG-3f-e) — the generated
  helper and `lower.rs` now share a single `SurfaceExpr` type via
  `parse_generated` re-exports; the old deep-clone `From` bridge is **gone**.
- **ROADMAP debt (still applies to other lens slices, not to this wire-in):**
  `render_variant_constructor` external tuple-variants; `SurfaceLiteral →
  LiteralBits` emit rename — not SG-3g-b wire-in blockers for the `expr_span`
  slice

## Work

**Phase 0 — verify the type-convergence blocker is what the PR body named**

Before touching call sites, verify:
- `src/v3/compiler/src/parse.rs` (or `parse_parser_body.txt`) exposes `parse::SurfaceExpr`
- `src/v3/lenses/lower_helpers.dag` → `lower_helpers_generated.rs` emits against `parse_surface::SurfaceExpr`
- The existing `From` bridge (if any) between the two is deep-cloning (grep for `impl From<parse_surface::SurfaceExpr> for parse::SurfaceExpr` or equivalent)

If verification confirms the above: proceed to the core wire-in work below. If the bridge is already non-cloning (either because SG-3f follow-up landed, or the types already share a structural representation), the blocker is already resolved — proceed directly to step 2.

**Core wire-in (resolve the type-convergence problem)**

Options for the worker to evaluate and propose in the PR body:

A. **Unify `parse::SurfaceExpr` and `parse_surface::SurfaceExpr`** at the type level. If they're already structurally identical and the distinction is historical, collapse to one. This makes `lower_helpers::expr_span` directly callable on whatever `lower.rs` already has.

B. **Change `lower.rs` to consume `parse_surface::SurfaceExpr`** throughout. If `lower.rs` is the newer/fresher consumer, adopt the newer type as its input.

C. **Generate `lower_helpers` against `parse::SurfaceExpr` instead.** Reverse direction — change the helper's input type so it fits the existing lowerer.

Worker picks based on which has the smallest blast radius; PR body declares the choice + rationale. STOP-AND-ESCALATE if the type unification pulls in parse/lower semantic changes beyond type-level convergence.

**Wire-in (post type convergence)**

1. **Identify all 16 `expr_span` call sites** in `lower.rs` (grep for the pattern; verify count).
2. **Replace each call site** with a call to the generated `lower_helpers::expr_span(...)` fn. Import via `lib.rs` / `lower.rs` as needed.
3. **Delete the inline `SurfaceExpr` span-extraction match** from `lower.rs` if it exists as a local helper (the 16 sites were duplicating it).
4. **Verify** lowering behavior is bit-identical: existing lowering tests pass; `m1_substrate_test`, `m2_*`, and any lane-related tests stay green.
5. **Check DB-8 fixed-point** — `self_host_fixed_point` on the fixture must still converge bit-identically.

## Acceptance

- No hand-authored `lower.rs` span/projection path that duplicates the current `lower_helpers` surface: `expr_span` / `item_span` / `pattern_binding_names` are imported from `crate::lower_helpers` and used for those concerns (per `lower.rs` `use` line)
- The `expr_span` call sites named in #612 are all routed through the generated helper (count may grow with upstream edits; grep is authoritative)
- Lowering tests green
- DB-8 fixed-point converges bit-identically to pre-wire-in
- `lower_helpers.dag` → `lower_helpers_generated.rs` freshness ratchet still green
- PR body explicitly frames this as **SG-3g-b wire-in receipt** (not SG-3b proper, not full `lower.rs` retirement)

## STOP-AND-ESCALATE

- **If wire-in requires per-call deep cloning across the `parse::SurfaceExpr` ↔ `parse_surface::SurfaceExpr` boundary** — STOP. That regresses the lowerer, not improves it. Escalate with the specific convergence path attempted and why it failed. This is the critical no-compromise gate; deep-cloning wire-in is worse than the current staging state.
- If the 16 call sites aren't all using the identical match pattern — the slice chosen may be narrower than expected. Surface the actual count and variant coverage.
- If wire-in reveals that `expr_span` needs access to `Dag` state that the generated helper can't see — the slice wasn't as pure as #612 thought. Surface the state dependency; don't work around.
- If DB-8 fixed-point drifts — bytes must match. Don't adjust the golden; root-cause the drift.
- If the chosen type-convergence option (A/B/C from the Work section) pulls in parse/lower *semantic* changes beyond type-level convergence — STOP. The blocker is type-level; semantics-level changes belong in separate lanes (SG-2c proper, SG-3b proper).

## Non-goals

- **Not retiring `lower.rs`** — this is one slice wire-in, not full retirement. `lower.rs` keeps ~99% of its logic hand-authored until more slices land.
- **Not adding new helper families in this receipt** — wire-in consumes whatever the `.dag` already listed at ship time (`expr_span`, `item_span`, `pattern_binding_names` in the current tree). New projections stay in follow-on SG-3g-c / cluster briefs.
- **Not addressing the two named blockers** (`SurfaceLiteral → LiteralBits`, `render_variant_constructor`) — those get separate ROADMAP debt rows. Not this lane.
- **Not changing `lower_helpers.dag` for the wire-in receipt** — authority landed in #612; lens edits are a different lane unless a bug is found.

## Size

M. Mechanical replacement of 16 call sites with bit-identical behavior preservation. Small scope; gated by thorough verification that lowering output is unchanged.

Expected LOC delta: ~-20 to -40 (the inline match pattern deduplicates into 16 call sites to the generated helper) + ~5 for imports. Net small negative.

## Dispatch note

Director reviews. Key acceptance signal: DB-8 fixed-point bit-identical + lowering test suite green. If either fails, STOP and diagnose before re-trying.

**Shipped outcome:** ROADMAP SG-3g is no longer "staging-only" for these helpers: the live lowerer imports `lower_helpers` and the regen/lens ratchet stays enforced. **Next work** (separate briefs / wave slots): additional `lower.rs` consumption or new `lower_helpers.dag` rows are gated on ROADMAP `emit_rust_module` debt where applicable; the director "Generated Helper Consumption" cluster frames paired lower+infer tranches when those briefs are written.
