# ROADMAP completeness diff — current main (245 ln) vs emit(authority) (201 ln, ✦-lane held)

Status: **re-transcribed faithfully from current main.** Zero content loss confirmed by token audit:
every plan-doc path and every PR ref on main is present in the emit; the only *added* refs are the
C1–C5 corrections (#5470/#5478, #5525/#5527/#5513/#5467, #5491/#5508/#5520, #5481/#5514). The ✦ lane
is the ONLY missing block — HELD pending #5545 merge (will be transcribed natively from post-#5545 main).

All 5 witnesses green by execution; the effectful gate runs drift-clean on the regenerated file with a
live red-receipt (perturb the merged set → drift RED).

## Classification of every delta

### [held — pending #5545] — the ONLY missing content
- **✦ Ergonomics LANE** (main L60–88). Model machinery is in place (`lead_lanes: List<RoadmapSection>`,
  sigil-from-position). Authoring deferred per warm-lark's ordering ruling: transcribe the FINAL
  post-#5545 bytes natively, not hand-folded from bright-stag's PR description.

### [intended-correction] — the five corrections (and nothing else adds content)
- **C1** §1: `**Section 1 spawn-width foundation — std.measure expressibility** (#5470/#5478)` — DerivableLine, box derives `[x]`, refs from binding; placed in the "What's on `.dag` today" group after the width item. *(placement is bright-stag's call — flag.)*
- **C2** §6: `**round-trip law (ingest∘emit = id, DecodeFidelity-bounded)** (#5525/#5527)` — own `[x]` row distinct from the medium-axis check; authority #5513 cited; no overclaim.
- **C3** §6: `**PR→checkbox status + section-emit + projection layer** (#5491/#5508/#5520)` — replaces the old "model + witness landed, host-fed gate next (§6.4)" sub-row.
- **C4** §5 milestone: adds `· class-3 corpus-coherence + cargo-green seed ✓ (#5481)` + the `*(§7 regen-fixpoint deferred, #5514; src/v1 NOT yet deletable)*` no-overclaim tail.
- **C5** §0 rust-gate: line restored in full, stays `[ ]` (no status change), edge-(b) brief pointer included.
- §1 **title reverted** to "CI as the substrate integration dogfood (the correctness floor)" (GAP 2 — your revert directive, not a new delta).

### [benign-rewrap] — form only, content byte-preserved
- **Line wrapping**: main hard-wraps prose at ~100 cols; the projection emits one line per paragraph. Every multi-line `<` / single-line `>` hunk is this. Content identical.
- **Pointer frame** (your shape-2a ruling): `([label](path))` → ` [label](path)` — parens/commas are projection frame; **label + path are data, preserved exactly** (cause table / edge-(b) brief / detail / decision record / force-check plan / Disposition plan / scope / merge-freshness decision record all kept).
- **Refs-from-binding** (§3): a DerivableLine's `(#N)` is generated from `prs` and sits right after the title; main hand-placed it mid-sentence. Same ref, repositioned.
- **Structural-title bold is now authored data** (not emit-applied): the emit no longer force-bolds derivable titles; bold lives in the title string (the §5 fence, consistent with AuthoredLine). This made the emit match main's *per-item* bolding exactly (main bolds some derivable titles, not others; I matched each). Two authored audit items (lens/gate wiring, fail-open code) un-bolded to match main.
- **Uniform blank line after a sub-group label** (`**Audits (done):**` ¶ then list). main is inconsistent (§0 has the blank, §1 host-band doesn't); the projection is uniform.

### [option (B) — IMPLEMENTED] — 2 compact multi-task bullets, now plain
main has two plain `- ` bullets carrying *inline* `[x]`/`[ ]` glyphs:
- `- hermetic fixtures feed P2: [x] M4.1 … ; [ ] M5 …` (mixed states)
- `- blockers: [ ] B1 … · [ ] B2 …`

Resolved per both owners' ruling (B): a `PlainLine` RoadmapItem variant + roadmap-layer
run-partitioning (consecutive task/plain nodes → adjacent `TaskListBlock`/`UnorderedListBlock`).
**No std.markdown touch** — the heterogeneous-list unification is a separate parked defect (warm-lark).
- **hermetic** (nested under P2): **byte-exact** — `md_item` joins nested blocks with `"\n"`, so it sits
  contiguous between P2 and P3 with no leading checkbox and no blank line.
- **blockers** (top-level): renders as a plain bullet with **one** structural blank line before it
  (`block_separator` = `"\n\n"` between the task-list block and the unordered-list block). Classified
  **[benign-rewrap]** — the std-heterogeneous-list fix that would remove the blank is deferred to its own
  scoped PR (one unified `ListBlock`, item = `Task{state}` | `Plain`).

All content (inline boxes, both PR refs, both plan links) preserved verbatim inside the fenced strings.

**Net: zero [GAP] except the held ✦-lane. Both owners signed (content + gate-integrity); option (B)
implemented (plain bullets, no phantom checkbox); the only added top-level delta is the blockers blank-line
([benign-rewrap]). #5535 stays DRAFT until #5545 merges → ✦-lane transcribed natively → bright-stag ✦
sign-off → this artifact deleted → flip ready.**
