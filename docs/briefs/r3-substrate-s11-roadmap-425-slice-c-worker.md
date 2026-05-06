---
status: draft (worker brief; cascade-clearance trigger fired post-#1801 merge 2026-05-06; dispatchable now)
authority parent: R3 Substrate Manager (#1739)
ratification: B1 RATIFIED prose+regen bundling (Substrate canvas C3 + Substrate Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385074769)
roadmap row: ROADMAP.md :425 (PARTIAL → Retired upon Slice C land); references docs/briefs/r3-425-stale-invariants-prose-sweep-worker.md
authority docs:
  - docs/briefs/r3-425-stale-invariants-prose-sweep-worker.md (parent ROADMAP :425 retirement brief)
  - PR #1795 (Slice A; merged) — proud-lynx-311 path-(a) precedent
  - PR #1801 (Slice B; merged) — smart-ram-167 prose+regen bundling precedent
  - docs/r3-design-schedule-2026-05-06.md §1 S11
gates: ROADMAP :425 row Retired; Q-Slice-C-Retirement-Receipt closes
worker pin: smart-ram-167 (#1759) — Slice B precedent owner; pattern-familiar (per Substrate canvas C3 + Substrate Mgr partition response)
---

# R3 Substrate S11 — ROADMAP :425 Slice C residual worker brief

## Context

ROADMAP.md row :425 ("retire stale INVARIANTS.md §P* prose references")
is a 3-slice retirement program. Two slices have landed:

- **Slice A** (#1795, proud-lynx-311) — initial path-(a) per-file
  enumeration retirement
- **Slice B** (#1801, smart-ram-167) — paired prose+regen bundling
  pattern; ROADMAP.md row reframed to post-A+B framing with Slice C
  residual list preserved

**Slice C residual** is the third and final slice — ~10 files
remaining per smart-ram-167's enumeration documented at #1801
post-rebase. Once Slice C lands, ROADMAP :425 row moves PARTIAL →
Retired.

This brief authorizes Slice C dispatch with **B1 RATIFIED
prose+regen bundling** (paired prose edits + bootstrap snapshot
regeneration in same PR per Slice A path-(a) precedent + Slice B
ratified bundling).

## Slice

### Phase 1 — Slice C residual files retirement

1. **Inventory residual files** at HEAD on main. Worker greps
   `ROADMAP.md` row :425 for the post-Slice-B residual list (Slice
   B's merged ROADMAP edit names the remaining files in the row's
   "post-A+B framing" — that list IS the Phase-1 inventory).

2. **Per-file edits** apply the standard INVARIANTS.md prose
   sweep pattern from Slice A + Slice B precedents:
   - Stale `§P1` / `§P2` / `§P3` / `§P4` / `§P5` references
     either updated to current heading anchors (per
     `INVARIANTS.md#p1-modeling-faithfulness` etc. — section
     anchors, NOT bare `:NNN` line numbers per
     `docs/briefs/brief-authoring-checklist.md` Citation
     discipline) or replaced with rule-text quotes per the
     reference's load-bearing semantic
   - Stale "P1 Boundary Discipline" / cross-numbering errors
     resolved to current §P1 (Modeling Faithfulness) / §P2
     (Boundary Discipline) numbering
   - Where prose paraphrases a current INVARIANTS.md rule,
     prefer rule-text quote (self-contained; survives doc
     edits) over indirect reference

3. **Bootstrap regeneration** in the SAME PR per B1 RATIFIED
   bundling. Files touched in `dsl/std/` + downstream
   `bootstrap_generated*.rs` snapshots regenerate per repo
   convention. Prose+regen bundling is the load-bearing
   discipline — do NOT split the regen into a follow-up PR.

### Phase 2 — ROADMAP row retirement

1. **ROADMAP.md row :425**: edit from "PARTIAL (post-A+B; Slice C
   residual: <list>)" → "Retired (Slice A #1795 + Slice B #1801 +
   Slice C #<this-PR>)". Worker greps current row at HEAD;
   exact prior wording dictates the edit.

2. **Q-Slice-C-Retirement-Receipt resolution** in PR body —
   audit-receipt naming the 3-slice program complete; ROADMAP
   row Retired; no residual; per Substrate canvas C3.

## Acceptance

- All Slice C residual files (per ROADMAP :425 post-A+B residual
  list at HEAD) edited per the Slice A/B precedent pattern.
- Bootstrap snapshots regenerated in the same PR (B1 RATIFIED
  bundling).
- ROADMAP.md row :425 status moves PARTIAL → Retired with
  3-slice program receipt.
- Q-Slice-C-Retirement-Receipt resolution in PR body.
- `cargo test --workspace --exclude v2-compiler-tests` green
  (3 pre-existing test failures noted in earlier sessions —
  worker re-verifies they reproduce on clean main and are
  unrelated to this change).
- `cargo test -p v2-compiler-tests` green; strict-compile
  diagnostic ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- Citation discipline per `brief-authoring-checklist.md`:
  cross-doc cites use anchors or rule-text quotes; bare `:NNN`
  PROHIBITED.
- 5-question authority audit in PR body.

## STOP-AND-ESCALATE

- **Slice C residual file inventory at HEAD differs from Slice
  B post-rebase enumeration**: re-verify; the canonical list
  is the ROADMAP :425 row's "post-A+B residual: ..." text at
  HEAD post-#1801 merge. Worker re-greps before authoring;
  if the residual list has drifted (e.g., another PR retired
  some entries opportunistically), Slice C scope narrows
  correspondingly.
- **A residual file's prose sweep requires a substantive
  semantic change** (not just `§P1` → `#p1-modeling-faithfulness`
  anchor conversion): STOP — the change is no longer paydown,
  it's a substantive rewrite. Surface to Substrate Mgr (#1739)
  for a separate brief. Slice C scope is mechanical anchor /
  quote conversion only.
- **Bootstrap regen produces unexpected delta beyond span
  updates** (e.g., downstream code paths shift in a non-
  mechanical way): STOP — the regen has caught a substantive
  change masked as paydown. Surface to Substrate Mgr.
- **`cargo fmt --all --check` introduces auto-format edits that
  the pre-push hook commits separately** (per repo convention,
  the hook lands a `chore: apply cargo fmt` commit and aborts
  the push for re-push): expected; re-run `git push` after the
  hook commit. Not a blocker.

## Authority audit receipt

1. **Substrate exists?** N/A — this is documentation paydown,
   not substrate landing. Worker greps ROADMAP :425 row + cited
   Slice A/B precedent PRs at HEAD before authoring.
2. **Existing brief?** Parent
   `r3-425-stale-invariants-prose-sweep-worker.md` exists; this
   brief is the Slice C dispatch, NOT a competing authority.
   Slice A worker brief and Slice B worker brief similarly
   subordinate to the parent.
3. **Design-doc match?** B1 RATIFIED prose+regen bundling is
   the locked decision (Substrate canvas C3; Substrate Mgr
   partition response 2026-05-06). Slice A path-(a) precedent
   + Slice B bundling precedent are the operational anchors.
4. **Citations live?** ROADMAP :425 row at HEAD post-#1801
   merge. Worker re-verifies at dispatch time.
5. **Carrier dissolves the bridge?** N/A — no carrier; the
   "bridge" is stale prose references that have lost
   correspondence to the current INVARIANTS.md heading
   structure. Anchor / quote conversion restores
   correspondence.

## Provenance

Drafted 2026-05-06 post-#1801 merge per cascade-clearance
trigger ratified at B1. Worker pin smart-ram-167 (#1759) —
Slice B precedent owner; pattern-familiar. Dispatch fires
immediately on this brief landing.
