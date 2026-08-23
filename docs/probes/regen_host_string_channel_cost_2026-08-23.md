# The regen host's String channel: a measured price for a trigger that had none

DESIGN declares `v1_compiler.required_regen_host` `run_required_regen` **mitigatable**,
"because it mirrors the carrier ordering BY HAND rather than deriving it", and names its
next-rung trigger: **the host being derived from the carrier rather than written beside
it**.

That trigger has had no price attached to it. This document attaches one. It is a
measurement, not a proposal, and it deliberately does not propose the narrow fix — see
*Why no repair is proposed here*.

## The specimen

PR #8938, run `32605171964`, 2026-08-23T00:26Z:

```
required-ci: phases_run=3 failed=1
required-ci: FAILED PHASE regen refused: normalize emitted src/std_checked_arithmetic.rs:
             spawn rustfmt: Text file busy (os error 26)
```

`ETXTBSY` — the kernel refusing to exec a binary held open for writing, i.e. a concurrent
toolchain install on a shared self-hosted runner. Transient, retryable, and saying nothing
about any tree.

The witness floor on that same run: `planned=10585 executed=10585 terminal=10585
passed=10276 known_red_held=208 failed=0`. The run was red on a `rustfmt` spawn in a
different phase.

## Three owners in one channel

`normalize_generated_source` returns `Result<String, String>`. Three failures reach it,
with three different owners and three opposite remedies:

| failure | owner | remedy |
|---|---|---|
| `spawn rustfmt: <io error>` | the runner | retry; nothing about the tree |
| rustfmt exits non-zero, stderr returned raw | the **emitter** | a real defect in emitted bytes |
| `rustfmt did not reach a fixed point in 8 passes` | emitter ∧ formatter | investigate the disagreement |

`write_emitted_tree` then folds all three into `format!("normalize emitted {path}: {e}")`.

**Stated precisely, because the imprecise version is falsifiable:** these three are
*distinguishable in the rendered text* — the inner prefixes differ. They are **not
distinguishable to a program** without parsing prose. So the cost is not that a human
cannot read the log; it is that no consumer can branch on the distinction — no retry
policy, no classification, no counter — and any consumer that tried would have to
re-derive the taxonomy from message text, which is a second representation of a
distinction the model already owns.

## The measurement that relocates the finding

The obvious reading is that the `map_err` flattens three causes into one arm of the
carrier's typed vocabulary. **That is not what happens.**

- The carrier has the vocabulary: `RequiredRegenRefusal` in
  `src/v2/workflow/required_regen.dag` carries ten arms, including
  `CandidateTreeAbsent { reason: String }`.
- **The host constructs none of it.** `CandidateTreeAbsent` appears **zero** times in any
  `.rs` under `src/v1` — no construction, no match. Its only occurrence corpus-wide is the
  carrier declaration.
- `run_required_regen` returns `Result<_, String>` and propagates the bare string to
  `required-ci: FAILED PHASE {failure}`.

There is no carrier arm on this route to flatten *into*. The host has a disjoint String
channel end to end.

## The strongest single observation

The host **cites the carrier arm in prose and returns a `format!` string beneath it**. In
`run_required_regen`:

> `CandidateTreeUnproduced` in `v2.workflow.required_regen` is the modeled arm and it
> carries no tree

…and the code under that comment is `return Err(format!("{reason} — no candidate tree
produced, nothing to compare"))`.

That is **authority substitution** in the corpus failure vocabulary: a modeled arm quoted
as the authority for a thing that does not use it. It matters more than the errno taxonomy
above, because it shows the gap is **not oversight** — someone knew the arm existed, named
it correctly, and could not reach it from where they stood.

## Why no repair is proposed here

The narrow fix — a typed error enum inside the host, hand-matched to the carrier — would be
a **second hand-mirror of the same distinction**, and DESIGN prices the hand-mirroring
itself as the defect. It would make the declared class worse while making one symptom
quieter: the conflation stops costing lanes visible time, so the deficit stops ranking, and
the derive-from-carrier work loses the only pressure that was going to fund it. That is the
absorbing-fallback shape at authoring time.

Nor is it proposed with its cost stated on its face: a cost written on the face of a thing
that should not exist is a dissolution trigger doing duty as a permit.

Adding `io::ErrorKind` to the existing string was considered and rejected as adding
nothing: `Text file busy (os error 26)` already names the kind.

**The repair is the declared trigger, unchanged.** This document exists so that trigger has
a price when it is ranked against anything else.
