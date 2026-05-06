---
status: draft (wait-window; awaits R3 host restoration before dispatch)
authority parent: R3 Substrate Manager (#1739)
ratification: Director (#828) ratified option (a) at #issuecomment-4377264483 (2026-05-05)
roadmap row: ROADMAP.md "`Strict Forward Progress` subdoc ↔ heading drift"
supersedes: docs/briefs/r3-426-sfp-subdoc-design-call-surfacing.md (surfacing packet now obsolete)
---

# R3 :426 — SFP subdoc rename + dissolution-progress subdoc author

## Context

`docs/invariants/strict-forward-progress.md` carries
**bounded-execution** content; reviewer usage of the name "Strict
Forward Progress" carries the **dissolution-progress** meaning
(see the **Strict Forward Progress** bullet under
`INVARIANTS.md#p5-progress-is-dissolution` Related rules).
`INVARIANTS.md` flags the drift in three places — the
`> Note on naming.` blockquote and the **Bounded forward execution**
bullet under `INVARIANTS.md#p4-decidability` Related rules, plus
the `*Note:*` sub-bullet beneath the SFP bullet under
`INVARIANTS.md#p5-progress-is-dissolution` — as "future cleanup."

Director ratified option (a): **rename + author new**, preserving
the SFP name attached to the reviewer-usage meaning.

## Slice

1. `git mv docs/invariants/strict-forward-progress.md
   docs/invariants/bounded-forward-execution.md`. Content unchanged.
2. Author a fresh `docs/invariants/strict-forward-progress.md`
   carrying the dissolution-progress concept. Source the canonical
   statement from the **Strict Forward Progress** bullet under
   `INVARIANTS.md#p5-progress-is-dissolution` Related rules
   ("a change counts as progress only if it reduces ad-hoc state,
   duplicate authority, or implicit behavior; transitional
   scaffolds need explicit dissolution paths and cannot become the
   new steady state"). Cross-link the bounded-execution subdoc as
   a sibling concern, not a parent.
3. Update `INVARIANTS.md`:
   - **`> Note on naming.` blockquote under `#p4-decidability`
     Related rules** — rewrite: drop the drift caveat; redirect
     the bounded-execution pointer to `bounded-forward-execution.md`;
     affirm SFP name now points at the dissolution-progress subdoc.
   - **`**Bounded forward execution**` bullet under `#p4-decidability`
     Related rules** — rewrite the subdoc reference to point at
     `bounded-forward-execution.md`.
   - **`*Note:*` sub-bullet beneath the SFP bullet under
     `#p5-progress-is-dissolution`** — delete entirely (drift
     caveat); SFP bullet's subdoc reference now points at the
     freshly authored `strict-forward-progress.md`.
4. `rg "strict-forward-progress\.md"` across the tree; update any
   reference whose intent was bounded-execution (subdoc content)
   to `bounded-forward-execution.md`. References whose intent was
   the SFP rule name (reviewer usage) stay pointed at
   `strict-forward-progress.md` — they now resolve correctly.
5. ROADMAP row "`Strict Forward Progress` subdoc ↔ heading drift"
   flips Open → Retired with PR sha and the dissolution sentence
   ("rename + author new per Director sign-off
   #issuecomment-4377264483").

## Acceptance

- Two subdocs exist: `bounded-forward-execution.md` (renamed,
  content unchanged) + freshly authored `strict-forward-progress.md`
  (dissolution-progress).
- The three drift sites in `INVARIANTS.md` (the `> Note on naming.`
  blockquote and **Bounded forward execution** bullet under
  `#p4-decidability`, plus the `*Note:*` sub-bullet under
  `#p5-progress-is-dissolution`) updated; drift caveats removed.
- `rg "strict-forward-progress\.md"` shows no references with
  bounded-execution intent.
- `cargo fmt --all --check` clean (no Rust changes; defensive).
- ROADMAP row Retired.

## STOP-AND-ESCALATE

- If the freshly authored `strict-forward-progress.md` reveals that
  the dissolution-progress concept has substantive content not
  captured anywhere in `INVARIANTS.md` P5, STOP — the rule is
  under-documented and the worker brief assumed only a
  reorganization, not new authority. Surface to R3 Substrate
  Manager.
- If the cross-tree reference sweep at step 4 turns up references
  whose intent is genuinely ambiguous (was the cite about subdoc
  content or about the rule name?), STOP that reference and
  surface; do not silently route either way.

## Authority audit receipt

1. **Substrate exists?** N/A — invariants documentation, not `.dag`.
2. **Existing brief?** None; supersedes the surfacing packet
   `r3-426-sfp-subdoc-design-call-surfacing.md`.
3. **Design-doc match?** Director's ratification message at
   `#issuecomment-4377264483` names option (a) with the exact
   file moves used here.
4. **Citations live?** Four prose sites in `INVARIANTS.md` verified
   at HEAD (2026-05-05): `> Note on naming.` blockquote and
   **Bounded forward execution** bullet under `#p4-decidability`
   Related rules; **Strict Forward Progress** bullet and its
   `*Note:*` sub-bullet under `#p5-progress-is-dissolution`
   Related rules. Worker re-locates by prose excerpt at dispatch
   (anchors stable; line numbers drift).
5. **Carrier dissolves the bridge?** N/A; pure documentation
   reorganization, not substrate work.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per parent
session #828 instruction; replaces the earlier surfacing packet
once Director ratified option (a). Ratification pending host
restoration and parent dispatch slot allocation.
