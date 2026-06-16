# Documentation Authority — the doc map and the single-authority rule

> **Mode:** `LIVE` — describes the current contract. **Enforcement is manual /
> review-driven (operator call, 2026-06-16):** the prior automated gate
> (`scripts/check_doc_refs.py` + the `doc_refs` CI job) was removed in the CI
> rework (#5013); reference-resolution and single-authority are now upheld at
> review and on touch, not by CI.

## Why this exists

The codebase enforces single authority on its *code* — every fact in exactly one
place (`INVARIANTS.md` P2 / `MODELING.md` M2). Its *prose* drifted the other way:
the same claim restated across several docs, and the doc map duplicated in three
places — this file, `THESIS.md` "How the docs connect", and `ROADMAP.md` "How to
read the tree". The prior version of *this very file* cited a `ROADMAP.md` structure
(a "Release R1 Program" section, a lane table, a tracked-debt ledger at fixed line
numbers) that no longer exists `[live]` — spec-without-execution in prose, the same
disease the code-side CI catches.

This file is now the **single home for the doc map and the rule that keeps prose
single-authority**, scoped to the whole tree (not just `docs/thesis/`). The two
duplicate maps are dissolved to one-line links here.

**Consumer:** design-vetting (the `DIRECTION-CHECKLIST.md` scan, reviews) is only
reliable when the docs underneath have unambiguous authority. **Bound:** the goal is
*no fact has two homes* — not doc perfection. Scope there and stop.

## The rule

**Every fact has exactly one canonical home. Every other mention is a one-line
summary that links to that home and must not restate the fact.** `[live]`

This is P2/M2 applied to prose; it makes cost-of-change → 1 for docs (when a fact
changes, one file changes, not six in lockstep).

## Adding a gunbc doc is a heavy action

A doc in gunbc is **durable, load-bearing reference** — it earns its place like code
and is review-gated the same way. Scratch, planning, throwaway, and work-in-flight
docs do **not** belong here; they live in **ctrl** (`gunb-ai/ctrl`, `gunbc-planning/`).
**Default to ctrl;** promote into gunbc only when a doc is the durable home of a fact
with a real consumer. A gunbc doc with no durable fact and no inbound reference is a
scratch doc in the wrong repo — it goes to ctrl, or it goes away.

## The authority DAG — where facts live

**Tier 0 — this file:** the doc map + the rule. Supersedes the inline maps formerly
in `THESIS.md` and `ROADMAP.md`, which now carry a one-line "see doc-authority" `[live]`.

**Tier 1 — root docs summarize + link, never restate:**

| Doc | Owns |
|-----|------|
| [`THESIS.md`](../../THESIS.md) | why gunbc exists + the canonical *claims index* (each claim's argument lives in one `docs/thesis/` essay) |
| [`INVARIANTS.md`](../../INVARIANTS.md) | the must-not-violate rules (P1–P5 + the C/E/L/DB ID index) |
| [`MODELING.md`](../../MODELING.md) | how to extend the language (M1–M11) |
| [`ROADMAP.md`](../../ROADMAP.md) | current **state / plan** (what works, what's landing, lanes, deferrals) — *not* claims |
| [`CODING.md`](../../CODING.md) / [`TESTING.md`](../../TESTING.md) | implementation / test discipline |
| [`DIRECTION-CHECKLIST.md`](../../DIRECTION-CHECKLIST.md) `[live]` | the derived design-scan; **owns zero facts** — every item names its `→` authority home |

**Tier 2 — `docs/` is the fact home:**

- `docs/thesis/*` — one essay per thesis claim = the canonical *argument* for it.
- `docs/design-*` — design decisions (the home for things like the coercion-mismatch
  taxonomy, which currently over-states in `ROADMAP.md` `[live]` — re-home pending,
  touch-driven).
- `docs/audit/*` — dated `LIVE`-mode current-state audits.
- `docs/architecture.md`, `docs/algebraic-type-spec.md` — substrate / type-system specs.

**Not in gunbc:** planning docs live in **ctrl** (`gunb-ai/ctrl`, `gunbc-planning/`).
gunbc keeps **zero** planning snapshots; `docs/planning/` was **dissolved** (2026-06-16) —
its sole load-bearing design root was promoted to
[`docs/design-recompute-memoization.md`](../design-recompute-memoization.md). Source
`.dag`/`.rs` headers must not cite planning docs (they are ephemeral); any such citation
is a stale comment cleaned on the next touch of that file `[live]`.

**Frozen:** `src/v2/*.md`, `src/v3/*.md` carry a historical banner and read as frozen,
not live (v3 is frozen and out of the v4 loop; v2 is the reference compiler / v4 seed).
Marked in place — not moved, because moving manufactures the dangling-reference rot this
contract exists to prevent.

## The doc chain

The authority DAG above is the intent **and** the contract. A machine-generated
rendering of the *actual* Markdown link graph used to live here, refreshed by
`scripts/check_doc_refs.py --write-graph` and checked in CI. That generator and its
CI job were removed in the CI rework (#5013); rather than leave a frozen diagram to
rot unmaintained (the exact disease this file exists to prevent), the generated chain
was dropped. The Tier-0/1/2 table above is the single hand-maintained map; if an
automated link-graph is wanted again, it lands as a new gate (see Enforcement) and
this section is restored with it.

## Enforcement (review-driven; the automated gate was removed)

- Applies to every **new or touched** doc immediately — touch-driven; no big-bang
  rewrite (the same converge-on-touch move used elsewhere in the codebase).
- **No CI gate today.** The prior standing detector (`scripts/check_doc_refs.py`, run
  as the `doc_refs` CI job) was deleted in the CI rework (#5013). Until a replacement
  lands, reference-resolution and the single-authority rule are upheld **at review and
  on touch**: when you touch a doc, fix its references and convert any restated fact to
  a link. A dangling reference or a restated fact is a **review reject**, not a CI
  failure `[live]`. (If the gate is rebuilt, the operator's standing preference is a
  `.dag` claim discovered by `ci_claim_gate`, matching the one-binary CI floor — not a
  reintroduced shell/Python gate.)
- A new or touched doc that **restates** a fact owned elsewhere, instead of linking,
  fails review.

## Mode tagging

Every doc under `docs/thesis/` declares one mode between title and body; structural
claims carry a per-paragraph tag. (This part of the prior contract was sound; only its
stale `ROADMAP.md` citations were rot.)

- **`LIVE`** — audits current tree state. Claims cite `file:line`, test, or commit SHA.
- **`PROPOSAL`** — proposes a commitment to `THESIS.md` / `ROADMAP.md`; promotion
  requires a follow-up PR amending the authoring authority.
- **`TARGET`** — worked example of the destination; every claim pairs with a live-state
  gap pointer.
- **`MIXED`** — narrative spanning live + proposed + target; permitted only with
  per-claim tags **and** a claim-status summary table near the top.

Structural-guarantee words ("complete", "structurally enforced", "impossible",
"prevented by construction", "proves X") carry `[live]` / `[proposed]` / `[target]`
in the same paragraph. A bare structural claim without a tag fails review by rule.

**Self-application:** this file is `LIVE` about itself; its structural claims carry
`[live]` (citing the relevant section here) or, where about another doc, cite that doc.

## Scope / status authority

Scope and status live in **two homes only**: `THESIS.md` (the canonical claims index)
and `ROADMAP.md` (current state, lanes, deferrals). No other doc maintains parallel
scope/status authority `[live]`. Other docs may **propose** changes (`PROPOSAL` mode +
follow-up PR), **reference** current scope by link, or **audit against** it (`LIVE`
mode). A table titled "lanes" / "gates" / "scope" outside the two authorities is a
violation.

## Single-ledger rule

A doc that flags a gap links to **one** follow-up artifact (a `ROADMAP.md` tracked
row, or a ctrl `gunbc-planning/` item) — docs do not recreate planning state, proposal
queues, or sub-ledgers `[live]`. Numbered items (bug classes, lane IDs, debt rows) live
in one authoritative doc; others cite by number, never renumber. Cross-reference drift
is a violation.

## Relationship to INVARIANTS

This contract is the doc-tree's application of two existing invariants `[live]`
(see [`INVARIANTS.md`](../../INVARIANTS.md)):

- **P1 Modeling Faithfulness** — "documentation describes live state." Mode declaration
  + per-claim tags make the live-state claim explicit at paragraph granularity. With the
  reference-resolver gate removed (#5013), "describes live state" is a review obligation
  rather than a mechanically-checked one — the stance is unchanged, the enforcement is
  manual.
- **P2 Boundary Discipline** — "single authority." The one-fact-one-home rule + the
  scope/status and single-ledger rules make authority boundaries concrete for the
  categories that drift most.

Not a new principle; a specific discipline that satisfies existing ones for documents
that repeatedly drift the same way.

## Applying it

**New docs land compliant** — contract-compliance is a PR-review line item.

**Existing docs** — at next non-trivial edit: fix the doc's references, convert any
restated fact to a link, add the mode tag. Not a bulk-amend-the-tree obligation;
touch-driven `[live]`.

## Maintenance

`LIVE` about itself: the rules above describe the current contract. To change the
contract (new mode, different tagging, scope change), file a `PROPOSAL` doc and merge
it here in a follow-up PR.
