# SG-2c growth discipline checkpoint — SG-2c-proper capability gap

**Date:** 2026-04-22
**Size:** XS (decision artifact, no code)
**Scope:** decide before SG-2c-6 dispatches whether to continue row extraction, pivot
to the capability blocker, or run both in parallel.

## The question

Each `SG-2c-N` lane lifts another row out of `parse_parser_body.txt` into
`parse_tables.dag`. SG-2c-1…5 have landed; SG-2c-6 is WIP. The worry
(flagged in the 2026-04-22 reflective analysis): continued row extraction
without SG-2c-proper progress builds a **parallel grammar schema** that
hardens and becomes its own dissolution problem, even while the residual
recursive-descent algorithm in `parse_parser_body.txt` stays intact.

At what point does another `SG-2c-N` row stop buying progress toward
SG-2c-proper and start entrenching the parallel-authority shape?

## What has landed through SG-2c-5

Bounded declarative tables co-resident in `parse_tables.dag`, each with a
same-PR E-6 consumer in `parse_parser_body.txt`:

| Lane | Row family | Replaces in `parse_parser_body.txt` |
|---|---|---|
| SG-2c-1 | `BinaryOpRow` × `BinaryOpLevel` | five precedence matches (`parse_logical_or`, `parse_logical_and`, `parse_comparison`, `parse_additive`, `parse_term`) |
| SG-2c-2 | `TopLevelItemKwRow` (→ `top_level_item_dispatch`) | `parse_item` keyword dispatch |
| SG-2c-3 | (derived `is_type_rhs_boundary_keyword` from SG-2c-2 rows) | `skip_where_clause` / `rhs_is_sum` type-RHS lookahead |
| SG-2c-4 | `BracketRow` (→ `bracket_role`) | two inline bracket-membership match blocks in the same helpers |
| SG-2c-5 | (landed via PR #636) | further table extraction along the same axis |

Every row is **pure data** — no control flow, no cursor state, no error
recovery. The set of rows that fit this shape is finite: operator
precedence, keyword→dispatch-class, membership predicates, role
membership. After SG-2c-5, the remaining "cheap" cells on that axis are
shrinking visibly; SG-2c-6 is already the sixth pass and the blast
radius per lane is getting smaller.

## The named capability blocker for SG-2c-proper

Both `src/v3/compiler/parse_tables.dag:1-11` and the header of
`parse_parser_body.txt` already name the blocker. Making it concrete:

> **Capability:** recursive `.dag` function bodies over `List<Token>` with
> cursor threading — i.e., `fn parse_X(tokens: List<Token>, pos: Int) ->
> Result<(Surface*, Int), Diagnostic>` style functions that (a) recurse
> into other `parse_Y` functions, (b) match on `Token` sum variants
> inline, (c) short-circuit on `Diagnostic`, and (d) thread the advanced
> cursor back to the caller.

Cited source of the gap: `src/v3/std/list.dag:13-15` —

> "The bodies remain preserved as block text because the current compiler
> still lacks full structural recursion + list-body emission support."

Concretely, SG-2c-proper needs **all** of:

1. **Structural recursion over `List<T>` bodies** emitting to Rust/Go/Python (std/list.dag
   today keeps `fold`/`map` bodies as block text — signatures are authoritative,
   bodies are not yet emitted). This is the hard part: emission of recursive
   list traversal without going through a hidden host-Rust shim (forbidden by
   SG-2c's STOP-AND-ESCALATE rule).
2. **Match on a user-defined sum** (`Token`) inside those bodies, with
   per-variant binding and exhaustiveness. The existing SG-2c-N tables sidestep
   this by projecting `(Token, …) → Row` via keyword-name equality, not
   variant-match.
3. **Tuple-return short-circuit** — the `(Surface*, Int)` + `Result` shape
   that lets a parser function advance the cursor and propagate diagnostics
   in one step.
4. **Span fusion** — `span.fuse` over `Token` spans is idiomatic throughout
   `parse_parser_body.txt`; needs to express in `.dag` over the same
   carriers (lightweight vs. the other three).

Of these, (1) is the load-bearing blocker — (2)/(3)/(4) become straightforward
once the recursive-list-body emission pipe exists, because they are just
patterns that appear *inside* such bodies. SELF_HOSTING §6 Phase 4a
("~1500-2000 lines of `.dag`") assumes (1)–(4) are all working; the
document calls Phase 4a "still open" precisely because (1) isn't.

## Scope estimate — capability pivot

Rough, honest ranges; measured from comparable prior lanes, not from a work plan:

- **(1) List-body emission over `List<Token>`.** Connective-work territory:
  touches the emitter path for every target (Rust/Go/Python/dag). Prior
  comparable work (SG-3f runtime_mirrors / SG-3g-b lower-helpers wire-in)
  landed over multiple lane-weeks each; (1) is at least that size, plausibly
  larger because it's the first time `fold`/`map` bodies cross the structural
  emission bar. **L** in lane-size terms.
- **(2) User-sum match inside emitted bodies.** Likely already reachable
  once (1) lands, since match over user-defined sums is exercised in other
  std bodies — but the combination *recursive list traversal + nested
  variant match* has not been end-to-end proved. **S-M** conditional on (1).
- **(3) Tuple-return short-circuit.** The carrier surface exists
  (`Result<T, Diagnostic>`, tuple types in std); the question is whether the
  current emission pipe threads them through recursive bodies. **S**.
- **(4) Span fusion.** `.dag` method call + substrate support. **XS-S**.

**SG-2c-proper itself** (the `.dag` port of the 1350-line
`parse_parser_body.txt`) is **L** (SELF_HOSTING's 1500-2000 LOC estimate),
sits downstream of (1)–(4), and does not begin until the capability stack
is live.

**Aggregate pivot scope:** L (capability) + L (port) = two consecutive
lane-wave commitments, with the capability lane non-optional.

## Scope estimate — cadence continuation

Each `SG-2c-N` row at current cadence is **XS-S**: a data row, a projection,
a same-PR consumer splice, a ratchet update. Marginal value per lane is
visibly declining after SG-2c-4 (SG-2c-3 was already *derived* from SG-2c-2
rows rather than adding a fresh row family). The useful cells on the pure-data
axis are finite and largely exhausted.

**What cadence continuation *cannot* do:**

- retire `parse_parser_body.txt` as semantic authority (the residual
  algorithm stays);
- close the Phase 4a acceptance criteria in SELF_HOSTING §6;
- move the `parse_tables.dag` four-type exemption in the std/-consolidation
  ratchet (the "Compiler–`std/` consolidation program — specific migrations"
  row in ROADMAP and its `parse_tables.dag` precedent-rule bullet — that
  exemption unblocks only via SG-2c-proper per-row classification).

**What cadence continuation *does* do:**

- reduces the hand-coded residual in `parse_parser_body.txt` ahead of the
  port, so the port itself has less algorithm to translate;
- further validates the "table + projection + consumer" shape before it's
  subsumed by the full parse.dag.

## Parallel-authority risk assessment

Is the `parse_tables.dag` row pile becoming a hardening parallel grammar
schema?

- **Against hardening:** every row declared today is cited in
  `parse_tables.dag`'s header as *bounded* to what fits current capabilities
  and *trigger-gated* to dissolve into a Phase 4a/4b `parse.dag`. The file
  exempts itself from the std/-consolidation ratchet under a precedent rule
  (the `parse_tables.dag` bullet under the "Compiler–`std/` consolidation
  program — specific migrations" ROADMAP row). Rows are *data*, not
  *algorithm* — when SG-2c-proper lands,
  a `.dag` parser consuming these rows as plain tables is straightforward;
  the rows do not need inversion or dual-authority reconciliation.
- **Toward hardening:** the projection helpers (`binary_op_at_level`,
  `bracket_role`, `is_type_rhs_boundary_keyword`, `top_level_item_dispatch`)
  are now call-sites inside `parse_parser_body.txt`. Porting the residual
  algorithm has to preserve those call shapes or re-derive them from the
  row data — a small but real coupling cost.
- **Net:** risk is **modest, not accelerating**. The rows are
  well-bounded, self-documenting about their dissolution trigger, and do
  not expand the compiler's type surface beyond the exempted four types.
  The coupling cost of each additional row grows linearly, not super-
  linearly, in the SG-2c-proper port.

## Recommendation

**Natural ceiling — land SG-2c-6 if it has a concrete same-PR consumer, then stop.**

Specifically:

1. **SG-2c-6 — dispatch only if it passes a sharper bar.** Require it to
   (a) name the specific inline match or membership block in
   `parse_parser_body.txt` it replaces same-PR, and (b) cite a row family
   *not already derivable* from SG-2c-1…5 rows (the SG-2c-3 precedent —
   derive, not re-declare — applies first). If neither condition holds,
   skip SG-2c-6 and do not queue SG-2c-7+.
2. **Do not pre-queue SG-2c-7+.** The useful pure-data cells are nearly
   exhausted; additional lanes on this axis shrink marginal-value while
   the coupling cost to SG-2c-proper grows per row.
3. **Begin the capability-pivot design work in parallel, not serially.**
   The list-body emission gap (capability (1) above) is C1-territory
   (substrate-connective work) per the brief's own STOP-AND-ESCALATE
   clause; surfacing its design doc and lane shape is itself a separate
   lane and should be dispatched as such, not deferred behind SG-2c-6.
4. **Keep `parse_tables.dag`'s exemption in the std/-consolidation ratchet
   un-touched** until SG-2c-proper lands. The `parse_tables.dag`
   precedent-rule bullet under the "Compiler–`std/` consolidation program
   — specific migrations" ROADMAP row already handles any opportunistic
   individual-row migration.

**Why not "continue":** cadence continuation without a sharper dispatch
bar risks fabricating rows that duplicate SG-2c-3's derive-don't-re-declare
lesson; the declining marginal value is already visible.

**Why not "pure pivot":** SG-2c-6, *if it has a genuine same-PR consumer*,
is still an XS-S win and shrinks the port target. Throwing it out isn't
cheaper than landing it under a sharper dispatch bar.

**Why parallel (not serial) capability work:** the capability lane's
design phase (scope analysis, per-target emission plan, parity
criteria) does not contend with `parse_parser_body.txt` or
`parse_tables.dag` and therefore does not block on SG-2c-6.

## STOP-AND-ESCALATE triggers surfaced by this review

- The capability (1) — recursive list-body emission over `List<Token>` —
  is **substrate-connective territory** per the brief's STOP clause. It
  should be scoped as a separate lane (likely a C1-size design lane
  followed by an execution lane), not folded into an SG-2c-N.
- The cadence has a natural ceiling visible now: SG-2c-3 already had to
  *derive* rather than add a row family. Surfacing this rather than
  pushing through it organically ends the row-extraction sub-program at a
  clean boundary.

## Non-goals (per brief)

No row extraction, no capability fix, no parser retirement, no edits to
`parse_tables.dag` or `parse_parser_body.txt`. This doc is the decision
artifact; ROADMAP gets a one-line status update pointing here.
