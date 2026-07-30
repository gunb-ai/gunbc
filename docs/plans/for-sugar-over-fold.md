# Brief — `for` as syntax sugar over `fold`

**Status:** brief only. No code lands from this note. Written 2026-07-29 out of a gap found while
eliminating the heal skew guard's hand-authored shell (PR #7420); that PR deliberately dissolved its
need for `for` rather than wait on this, so nothing is blocked on it.

**Why it is not urgent.** The corpus has **zero** `for`-statements today and does not want one:
iteration is `fold` (1,346 sites), `map` (383), `filter` (304). `for` is already a *reserved* word in
the dag keyword table (`src/v1/stage0/src/extdeps_languages_dag_syntax.rs:172`) with no production
behind it. So this is an ergonomics addition, not a capability one — which is exactly the shape
DESIGN §4 licenses ("surface syntax is sugar that adds no power") and exactly the shape §6 warns can
become the purity trap if priced in elegance instead of displaced cost.

---

## 1. The two `for`s, which must not be conflated

This is the load-bearing distinction, and getting it wrong would fork an authority.

| | **(A) `for` in `.dag` source** | **(B) `PipelineStep.For`** |
|---|---|---|
| what it is | a surface production the frontend folds into a core `Node` | a modeled *bash* `for`-statement in `v2.std.orchestration` |
| desugars to | `fold` (this brief) | nothing — it is *emitted*, not executed |
| today | does not exist (reserved word only) | declared, **refused** by `orch_emit_step` (`^orch_emit_step_for_unsupported`) |
| authority | the sugar rule table | `05_emit_orchestration` + the bash grammar rows |

They are different concepts that happen to share a keyword. (A) is a language feature. (B) is a
target-language construct we render into a foreign executor. **Neither implies the other**, and a
single "add for" work item that tried to do both would be modeling two things as one.

This brief is **(A)**. (B) is a separate, smaller item — see §6.

## 2. The desugaring

`for` is a *statement* form, and `.dag` is expression-oriented and bounded-forward (§4: execution is
bounded and forward; recursion is sugar over `Loop`). So the honest desugaring is not "a loop" but an
accumulator fold, and the surface must therefore make the accumulator explicit rather than imply
mutation:

```
for x in xs { ... }                  // NO — implies a statement-sequenced mutable body
for acc = init; x in xs { expr }     // shape to design: acc is named, expr is its next value
```

which folds to exactly:

```
fold(xs, init: init, f: fn(acc, x) { expr })
```

Design questions to settle **before** any grammar row is written — each is a real fork risk, not a
style choice:

1. **Is the accumulator explicit?** If it is implicit, `for` is not sugar over `fold` — it is a new
   behavior with mutation semantics, which §4's closed vocabulary does not have. Recommend explicit.
2. **Does `for` cover `map`/`filter` too, or only `fold`?** `map` and `filter` are already
   `fold`-derivable. Sugar that desugars to *whichever of the three the body shape implies* is a
   heuristic — §4 rules a heuristic never necessary in a closed system. Recommend: `for` means
   `fold`, full stop; `map`/`filter` stay named.
3. **Termination.** `fold` over a finite list terminates by construction, so `for`-over-a-list
   inherits `DescentEvidence.Strict` for free. `for` must **not** admit a condition form
   (`for ...; cond; ...`), which would reintroduce the unbounded case `Loop` + `^loop_bound_edge`
   already model with real descent evidence (`dag/std/termination.dag`).

## 3. Where it lands — the existing sugar rule table, not a new mechanism

The body-lowering lane already built the carrier this belongs in: **one sugar rule table** keyed by
`SugarKey` = surface-atom | production identity, landed in #6443 with `type_alias_rhs` migrated onto
it as the proof. So `for` is **a row in that table**, not a new frontend path — which is the whole
point of §2 (one concept, derive every use) and the reason this is cheap.

Consequences that fall out of using the existing table rather than a bespoke fold:
- the row is read *forward* by normalize and *backward* by the emitter — §4's one-grammar-both-
  directions — so a `for` in `.dag` and a rendered `for` in a target language are the same row read
  two ways, and **(B) in §1 becomes nearly free once (A) exists**;
- exactly-one rule selection must be fail-closed (the table's existing contract): an ambiguous
  desugaring refuses, never picks;
- no new node kinds, so no `04_infer` arm, no new `DescentEvidence` case, no emitter change.

## 4. Acceptance bar

- A `.dag` `for` and its hand-written `fold` produce a **byte-identical core `Node` tree** — proven by
  execution on a discriminating corpus, not by typecheck. That equality *is* the "adds no power"
  claim, made falsifiable.
- RED controls, each of which must actually go red:
  - a `for` whose desugaring would need mutation (no explicit accumulator) → refuses;
  - a condition-form `for` → refuses (does not silently become a `While`);
  - an ambiguous rule selection → refuses rather than choosing;
  - a `for` over a non-list → typed, located refusal.
- Zero corpus churn on landing: no existing `fold` is rewritten. Migration, if ever, is a separate
  priced decision — rewriting 1,346 `fold` sites to gain a keyword is the purity trap by definition.

## 5. What this is worth (§6 — denominate the benefit)

Honest answer: **low, today.** The displaced cost is authoring ergonomics on nested folds, and the
corpus has not complained — 1,346 folds exist and none of them is waiting on this. The reason to do
it anyway is the §1 reserved-word debt: `for` is *already* in the keyword table, so a reader
reasonably expects it to work, and a reserved word with no production is a small standing lie. That
is a real but small cost, which is why this is a brief and not a work item.

The thing that would *raise* its priority is §6's second consumer: if a second caller wants
`PipelineStep.For` emitted (see below), then (A) and (B) share the row and the combined displaced
cost may clear the bar.

## 6. The sibling item — `PipelineStep.For` emission

Separate, smaller, and independently justifiable:

- `orch_emit_step` refuses `For`; `ValueSource` (`CmdSubstLines | Glob | ModeledList`) has no emitter
  either. Both would need bash grammar rows (`bash_stmt_for_*`) beside the existing
  `bash_stmt_exit_*` / `while` rows.
- **Do not add it speculatively.** It currently has *zero* would-be consumers: PR #7420 removed the
  only candidate by modeling the decision instead of the loop (a `:(exclude)` pathspec complement for
  the classification, one NUL-delimited `xargs` application for the resolution) — both of which are
  better models than a shell loop regardless of emitter support, because a per-path loop was
  re-deriving a set git can hand over whole.
- So the trigger is: **a second, genuine ordered-iteration intent that cannot be expressed as a set
  operation.** When one appears, land the rows then. Until then `For` stays declared-and-refused,
  which is honest (typed, located, counted) rather than inert.
- If it stays consumer-less indefinitely, the correct move is **deletion**, not implementation —
  `CaptureSpec.CmdSubst` beside it is already filed as exactly this class ("declared with zero
  constructors tree-wide — genuinely dead").

## 7. Sequencing

1. Settle §2's three questions (operator call — 1 and 2 are semantic, not preference).
2. `for` as a `SugarKey` row + the node-identity witness and its four REDs. No emitter work.
3. Stop. Revisit §6 only when a second consumer exists.
