---
status: draft (wait-window; awaits R3 host restoration before dispatch)
authority parent: R3 Substrate Manager (#1739)
roadmap row: ROADMAP.md "Stale §-style and line-number cross-references to pre-rewrite `INVARIANTS.md` — PARTIAL (T-Receipts W1)"
---

# R3 :425 — broader stale `INVARIANTS.md` reference sweep

## Context

T-Receipts W1 closed the named set of files (design-db16-refined,
clone-elimination, lane2-compile-time-proofs, M1_DESIGN, SELF_HOSTING,
sg-4b-1-fix-declaration-lookup-cleanup, p0-bug-no-profile-sentinel),
flipping their pre-rewrite line-number / `§`-style references to
stable `INVARIANTS.md#<id>` anchors. ROADMAP marks the row PARTIAL:
"Broader stale INVARIANTS prose outside that named set remains a
follow-up sweep."

The wait-window scan (2026-05-05) found ~15 additional files using
`INVARIANTS.md §"…"` (header-quote refs) or `INVARIANTS.md:<line>`
(line-number refs). Representative set:

- `docs/emit-target-spec-gaps.md`
- `docs/design-substrate-external-primitives.md`
- `docs/design-lens-framework.md`
- `docs/substrate-reflection-design.md`
- `docs/thesis-validation-plan.md`
- `docs/thesis/doc-authority.md`
- `docs/r2-structure.md`
- `docs/briefs/complexity-v2-v3-comparison-receipt.md`
- `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`
- `docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`
- `docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md`
- `docs/briefs/r3-pb-t-fixedpoint-worker.md`
- `docs/briefs/r2-pb-runtime-evaluator-convergence-matrix.md`
- `docs/briefs/r3-pb-binshim-retirement-worker.md`
- `docs/briefs/t-ground-languagespec.md`
- `docs/briefs/file-preference-bridge-dissolution.md`
- `docs/design-tests-as-data-completeness.md`
- `docs/briefs/r2-release-manager.md`

The fix is mechanical: each `§"<header>"` or `:<line>` reference
becomes the stable anchor `INVARIANTS.md#<id>` per the ID set the
W1 sweep established.

## Slice

1. Run a structured grep at HEAD to enumerate every remaining
   `INVARIANTS.md §` / `INVARIANTS.md:<line>` reference outside the
   already-fixed W1 set. Snapshot the count in the PR body.
2. For each match, identify the target invariant ID by reading the
   current `INVARIANTS.md` and locating the principle / procedure
   the prose is citing. The mapping is local to the cited principle
   name (`§P1` → `#p1-modeling-faithfulness` etc., per the W1
   convention).
3. Replace each reference with `INVARIANTS.md#<id>` and a short
   parenthetical principle name when the surrounding sentence reads
   ambiguously without it. Do not paraphrase the cited principle.
4. Where a reference cannot be mapped (the cited header no longer
   exists at HEAD; the prose was a stale claim from the pre-rewrite
   surface), STOP that file and add it to the per-file table in the
   PR body — do not fabricate a target ID.
5. Re-run the structured grep; assert zero matches outside any
   intentional historical-receipt blocks (which keep their original
   prose for provenance and are out of scope for this sweep).

## Acceptance

- Per-file count snapshotted in PR body (before / after / unmappable).
- Zero `INVARIANTS.md §"…"` and zero `INVARIANTS.md:<line>` matches
  outside the W1-fixed set and outside intentional historical-receipt
  banners.
- ROADMAP row flips PARTIAL → Retired (or stays PARTIAL with a
  narrowed residual if the unmappable set is non-empty; in that case
  the row's residual sentence names exactly which files remain and
  why).
- `cargo test --workspace --exclude v2-compiler-tests` green
  (defensive — pure prose changes, but the doc-tree may be linked from
  test fixtures).

## STOP-AND-ESCALATE

- If a citation cannot be mapped to a current `INVARIANTS.md` ID
  because the cited principle has been retired or renamed: STOP for
  that file. Surface the unmappable set to R3 Substrate Manager;
  resolution may require a Director call on whether the prose itself
  is stale.
- If the count exceeds ~30 unique sites or the mapping requires
  judgement on more than ~5 sites, STOP — the row's "small mechanical
  scope" framing has been violated and this needs to split.

## Authority audit receipt

1. **Substrate exists?** N/A — pure prose sweep; no carrier change.
2. **Existing brief?** None; T-Receipts W1 covered the named subset.
   This is the residual.
3. **Design-doc match?** N/A.
4. **Citations live?** Worker re-runs the grep at HEAD before
   authoring; the wait-window 2026-05-05 enumeration is illustrative,
   not authoritative.
5. **Carrier dissolves the bridge?** N/A; mechanical reference
   normalization, not a substrate dissolution.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per parent
session #828 instruction. Ratification pending host restoration and
parent dispatch slot allocation.
