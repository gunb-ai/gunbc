---
status: draft (worker brief; cascade-clearance trigger fired post-#1820 merge 2026-05-06; dispatchable now)
authority parent: R3 Substrate Manager (#1739)
ratification: B6 RATIFIED single bundled Mgr-tier doc-sharpening PR per Substrate canvas + R3 design schedule §1 S12
roadmap row: ROADMAP F2 (`.v3` filename-suffix grammar dispatch) + F8 (bootstrap load-order/exclusion authority) — both rows retire on PR land
authority docs:
  - docs/r3-design-schedule-2026-05-06.md §1 S12
  - docs/r3-design-schedule-2026-05-06.md §3 P5 (PB Mgr coordination — comment-only OR named-co-author; no duplicate PR)
  - Substrate canvas B6 (single bundled doc-sharpening PR ratification)
gates: ROADMAP F2 + F8 rows retire
worker pin: smart-ram-167 (#1759) — Slice C precedent owner; pattern-familiar with prose+regen bundling discipline; available post-#1820 land
---

# R3 Substrate S12 — F2 + F8 doc-sharpening worker brief

## Context

ROADMAP F2 + F8 rows are documentation-discipline gaps surfaced by
Substrate canvas. Per B6 RATIFIED single bundled Mgr-tier
doc-sharpening PR — both rows retire in a single PR rather than
splitting (avoid coordination overhead; doc-sharpening is naturally
co-located).

**S12 cascade-clearance trigger fired** post-#1820 (Slice C) merge
— ROADMAP :425 row Retired closes the prose-reference paydown
program; F2/F8 doc-sharpening is the next sequenced doc-discipline
work in the same authoring lane.

## Scope

### F2 — `.v3` filename-suffix grammar dispatch

ROADMAP F2 row covers documentation-discipline around the `.v3`
filename suffix as a grammar-dispatch signal. Worker greps ROADMAP
F2 row at HEAD for canonical scope statement (the row's prose
text IS the scope). Likely covers:

- Where `.v3` filename-suffix routing is authoritative documented
  (i.e., which file/spec is the source of truth for "files with
  `.v3` suffix dispatch through grammar X")
- Stale pointers to legacy grammar-dispatch paths (parser/lexer
  forking on suffix) replaced with current authoritative reference
- Cross-doc citations that mention `.v3` dispatch updated to the
  current authority shape

### F8 — bootstrap load-order/exclusion authority

ROADMAP F8 row covers documentation-discipline around bootstrap
load-order + exclusion list authority (which `.dag` files load in
what order; which fixtures are excluded from the runtime bootstrap
set). Worker greps ROADMAP F8 row at HEAD for scope. Likely covers:

- Where bootstrap load-set authority is canonical (e.g.,
  `bootstrap.rs::std_fixtures()` vs design-doc references)
- Stale prose claiming load-set membership that drifts against
  the live `bootstrap.rs` content
- Cross-doc citations updated to point at the current authority

## Slice

### Phase 1 — F2 + F8 inventory at HEAD

1. Worker greps ROADMAP.md for F2 + F8 row text — that text IS the
   scope statement. If row text has drifted post-canvas (which is
   likely given multi-week timeline since canvas), reframe scope
   to match HEAD.

2. For each row, identify:
   - **Authority docs**: where the canonical statement lives
     (design doc / spec / source comment)
   - **Stale references**: prose that claims facts no longer true
     against current code paths
   - **Citation updates needed**: bare `:NNN` references converted
     to anchors / rule-text quotes per current citation discipline

### Phase 2 — Single bundled PR

1. Both F2 and F8 paydowns land in one PR per B6 RATIFIED bundling.
   Do NOT split — the discipline is co-located authoring.

2. Citation discipline per `docs/briefs/brief-authoring-checklist.md`:
   anchors / rule-text quotes only; no bare `:NNN` introductions.
   Existing `:NNN` references in touched files convert as part of
   the sweep (per per-Mgr paydown ratified at gunbc#828
   #issuecomment-4385583247).

3. ROADMAP F2 + F8 row updates: PARTIAL → Retired with PR-link
   receipt.

### Phase 3 — PB Mgr coordination (cross-program)

Per design schedule §3 P5: PB Mgr coordinates consumer-side. **No
duplicate PR**. PB-named co-author OR comment-only on Substrate's
PR. Worker:

1. Tags PB Mgr (#1742) inbox with link to S12 PR when opened.
2. PB Mgr decides comment-only or co-author shape (PM ratification
   if explicit co-author shape needed — per design schedule).
3. Worker does NOT block on PB Mgr response; their input is
   consumer-side review, not authoring blocker.

## Acceptance

- ROADMAP F2 + F8 row text edited PARTIAL → Retired with PR-link
  receipts.
- Stale prose / citations across affected docs updated to current
  authority shape.
- Citation discipline: anchors / rule-text quotes form throughout
  touched files.
- Bootstrap regen N/A (doc-only PR; no `dsl/std/` or substrate
  edits expected). If any `dsl/std/` edits prove necessary,
  STOP — scope has crept beyond doc-sharpening.
- 5-question authority audit in PR body per `brief-authoring-checklist.md`.
- PB Mgr (#1742) tagged on PR open for consumer-side coordination.
- Standard ratchets:
  - `cargo test --workspace --exclude v2-compiler-tests` green
    (3 pre-existing v2-compiler --lib failures verified unrelated)
  - `cargo test -p v2-compiler-tests` green; strict-compile
    diagnostic ratchet at 0
  - `cargo clippy --all-targets -- -D warnings` clean
  - `cargo fmt --all --check` clean

## STOP-AND-ESCALATE

- **F2 or F8 row text at HEAD differs materially from canvas B6
  framing**: re-frame scope to match HEAD; canvas was authored
  ~weeks ago and prose may have shifted. If canonical authority
  has fundamentally changed (e.g., F2 was retired by another PR),
  surface to Substrate Mgr (#1739) — S12 may need re-canvas or
  retirement.
- **F2 or F8 paydown requires substrate / code edits**, not
  doc-only changes (e.g., a file's `.v3` dispatch path turns out
  to need a code-level routing fix to honor the documentation):
  STOP — scope has crept beyond doc-sharpening. Surface to
  Substrate Mgr; this becomes a separate code-side brief, not S12.
- **PB Mgr (#1742) requests material change to PR shape**:
  comment-only feedback can absorb directly; explicit co-author
  shape requires PM ratification per design schedule §3 P5.
  Surface to Substrate Mgr if shape ambiguous.
- **Bundled PR exceeds reasonable review scope** (e.g., F2 + F8
  combined PR touches > 30 files): consider splitting per
  pragmatic review-load discipline; surface to Substrate Mgr
  for re-bundling decision before authoring.

## Authority audit receipt

1. **Substrate exists?** N/A — doc-only paydown. Worker greps
   ROADMAP F2 + F8 rows at HEAD for canonical scope statement.
2. **Existing brief?** None for S12 specifically. Parent
   ROADMAP rows F2 + F8 are the authority. This brief is the
   cascade-clearance dispatch packet.
3. **Design-doc match?** R3 design schedule §1 S12 + §3 P5 +
   Substrate canvas B6 are the design surface. Worker re-reads
   schedule + B6 canvas at dispatch time.
4. **Citations live?** ROADMAP F2 + F8 rows at HEAD post-#1820
   merge. Worker re-verifies at dispatch.
5. **Carrier dissolves the bridge?** N/A — no carrier; the
   "bridges" are stale prose / drifted authority pointers in
   F2 + F8 row referent docs. Anchor / quote / rule-text
   conversion restores correspondence.

## Provenance

Drafted 2026-05-06 post-#1820 merge per cascade-clearance
trigger ratified at B6 (single bundled doc-sharpening PR).
Worker pin smart-ram-167 (#1759) — Slice C precedent owner;
pattern-familiar with prose+regen bundling. Dispatch fires
immediately on this brief landing.
