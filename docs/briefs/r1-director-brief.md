# R1 Director Brief

## Purpose of this brief

One-level-up orientation for the five R1 managers. Answers: what R1
is, why the five-manager shape, how the managers compose, where to
escalate, and where to read the product thinking that motivates all
of it.

This is a coordination brief, not a scope document. R1 scope — the
authoritative lane list, acceptance gates, and schedule — lives only
in [`THESIS.md`](../../THESIS.md) and [`ROADMAP.md`](../../ROADMAP.md).
Nothing in this brief overrides those. If a manager brief conflicts
with them, the manager brief is wrong; escalate.

## Product direction

Before reading this brief, read
[PR #672](https://github.com/gunb-ai/gunbc/pull/672):

- [`docs/thesis/compositional-modeling.md`](../thesis/compositional-modeling.md)
  — the end-to-end story of what gunbc is trying to do and *why
  anyone should pay to model in it*. The short version: gunbc
  encourages modeling well beyond what typical languages allow
  (cardinality, refinements, phantom-carried conventions, cross-team
  reconciliation), and **rewards** that modeling with two payoffs
  that traditional languages don't offer: code generation (the
  mechanism processes the modeling so the developer writes no
  boilerplate) and compile-time guarantees (the compiler enforces
  the conventions wherever they flow). Modeling becomes cheap,
  reward gets larger, so "model more" becomes the rational default.
- [`docs/thesis/doc-authority.md`](../thesis/doc-authority.md) — the
  governance contract all thesis-subtree docs follow: mode
  declaration, per-claim tagging, R1-scope-single-authority,
  single-ledger rule for gaps. Every manager brief that references
  live state cites `file:line`; every `[target]` item cites a
  ROADMAP tracked-debt row.

R1 is the release that makes the story doc's `[target]` subset
actually work. Each manager owns a slice of what's required to move
that needle.

## R1 in one paragraph

R1 is the release that closes **self-hosting** (gunbc's own
compiler written in `.dag`, with hand-authored Rust reduced to a
named floor), **testing-as-data** (release gates evaluate as `.dag`
programs through the T-TestGen runner, not as hand-authored Rust
test suites), and a **demonstration** that the two together work on
a realistic end-to-end target. Substrate carrier work (`T-LaneE`)
and language surface work (`T-Sub`) are prerequisite programs that
unblock the other three. Debt paydown (`T-Receipts`) runs
continuously alongside, not as a gate.

## The five managers

Each manager owns one coherent slice of R1 work. The lane names
inside each manager's slice are authored in `ROADMAP.md`; the
mapping below names the manager-level framing question each slice
answers — that's the manager's *why*, used to keep scope honest.

| Manager | R1 lanes owned (`ROADMAP.md`) | Framing question the manager answers |
|---|---|---|
| **Substrate** | `T-LaneE` (five ordered sub-lanes T → C → I → P → M) | Does v3's substrate carry the same descent/call/value/method information v2 carried, well enough that complexity / cost / idempotency / parallelism lenses can run on v3 authority without Rust oracles? |
| **Self-hosting** | `T-PB-A` (pure bootstrap, non-test surface) + `T-PB-B` (tests-as-data, pipeline + contract tests port to `.dag`) | Does gunbc compile itself from a named minimal Rust shim, with the residual Rust floor structurally bounded and mechanically checkable? |
| **Surface** | `T-Sub` (`match` over user sums; `CharClass` in `std.unicode`; type-alias `where`) | Do the three missing surface capabilities that block self-hosting lanes and gate predicates actually close, with no "it works around the gap" scaffolding surviving? |
| **Testgen** | `T-TestGen` (runner, service simulation, first-class `TestClaim`) | Can the release gates evaluate as `.dag` programs end-to-end — schema compiles (done), runner executes each predicate structurally, and `MockBackedInvariant` wires service simulation? |
| **Release** | `T-Demo` + `T-Receipts` | Does a realistic end-to-end workflow (the "scale" example from Part 7 of the story doc) compose the above managers' deliveries convincingly, and are tracked debts continuously paid down without blocking any gate? |

## Sequence + gate chain

R1's critical path (from `ROADMAP.md:119`):

```
max(T-LaneE,
    T-PB-A,
    T-Sub → T-TestGen → T-PB-B) → T-Demo
```

Manager-level reading:

- **Substrate** (T-LaneE) and **Self-hosting**'s non-test half
  (T-PB-A) run in parallel with no cross-blockers.
- **Surface** (T-Sub) → **Testgen** (T-TestGen) → **Self-hosting**'s
  test half (T-PB-B) is the serial spine. T-TestGen is the
  gate-enabling lane: most release predicates are `[ext]` and
  compile only after T-TestGen's schema extensions land
  (`ROADMAP.md:59`).
- **Release**'s demo half (T-Demo) is downstream of all four.
- **Release**'s debt track (T-Receipts) is continuous, not gated —
  bundle 2–4 items per PR per the standing preference
  (`ROADMAP.md:28`).

Practical consequence for manager coordination: Substrate and
Self-hosting managers dispatch Day-1. Surface dispatches Day-1 and
Testgen watches Surface's closure. Testgen's runner closure unblocks
Self-hosting's T-PB-B half and the majority of release gates at
once. Release demo waits on the convergence.

## Escalation

Anything that touches R1 scope — adding a lane, changing a gate,
moving the schedule — escalates to the director (this brief's
owner) who coordinates amendments to `THESIS.md` / `ROADMAP.md`.
Managers do not author R1 scope; they own lane-level dispatch
inside the scope their slice names.

Cross-manager dependencies are coordinated at the director level
when they cross the critical-path spine. Within-slice sequencing is
the manager's call.

Debt-row disputes (is this a tracked debt, is this scheduled, which
row owns which gap) route through the tracked-debt ledger section
of `ROADMAP.md` per the single-ledger rule
([`doc-authority.md`](../thesis/doc-authority.md)). Managers surface
candidate rows; director promotes to the ledger.

## Manager brief shape (for the five follow-ups)

Each manager brief should:

1. Open with a pointer to PR #672 and this director brief — same
   product direction, same coordination context.
2. Declare its slice of R1 by naming the lanes it owns (cite
   `ROADMAP.md:<line>` for each).
3. Answer the framing question in the table above — how the slice's
   deliverables answer the question, not just what they are.
4. Name its lane-owners and sequence them (Day-1 dispatch vs. gated
   on another lane).
5. List explicit hand-off points to other managers (e.g., "Testgen
   unblocks Self-hosting's T-PB-B half when runner closes").
6. Flag any `[target]` it depends on that is **unscheduled** (no
   tracked-debt ledger row yet) so the director can add the row
   before manager work blocks on the gap.

Manager briefs do not re-author lane content; `ROADMAP.md` remains
authoritative. Manager briefs provide the *why*, the sequencing,
and the cross-manager coordination surface above what `ROADMAP.md`
already names.

## Notes and open items

- **Unscheduled gaps surfaced in PR #672's story doc** are two:
  Duration/Money unit-mismatch enforcement consumer, and `Secret<T>`
  nominal-wrapper graduation. Neither has a tracked-debt row in
  `ROADMAP.md` today. Release Manager should verify whether R1
  depends on either (likely not — they're story-level `[target]`
  items beyond R1's demo scope) and, if not, flag them as
  post-R1 tracked-debt follow-ups for the director to add.
- **`T-Receipts` is not a gate but is also not unbounded.** The
  manager owns the standing preference of 2–4 items per receipt PR
  and the invariant-reveal discipline per PR #669. Receipt volume
  is an indicator: if it spikes, something upstream drifted.
- **Director ↔ manager loop cadence** is expected to be
  asynchronous per-manager, with a director-level review when a
  manager's slice opens a new cross-cutting question or blocks on
  another manager's deliverable.
