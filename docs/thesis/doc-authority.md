# Thesis Doc Authority — Mode and Claim Tagging

> **Mode:** `LIVE`

## Why this exists

A review loop on the previous "impossible bug classes" + "worked
composition example" drafts (PR #671, closed) surfaced a recurring
failure mode across reviewers: documents under `docs/thesis/` were
mixing claims about current tree state, proposed changes, and target
state all inside the same prose, in present-tense grammar. Each
round, reviewers caught one or two over-claims; each fix was local;
the next round found new instances elsewhere.

This is a structural-not-content problem; local prose tightening
cannot prevent the pattern `[live]` (diagnosis grounded in the
review history of closed PR #671). This document is the
rule-of-the-road that prevents it at the contract level `[live]`
(rules codified in the sections below).

## Four modes

Every thesis-subtree document declares one mode at the top, between
the title and the body `[live]` (rule codified in this section):

- **`LIVE`** — audits current tree state. Claims are about code as it
  exists today. Every claim cites file:line, test file, or commit
  SHA `[live]`.
- **`PROPOSAL`** — proposes commitments to `THESIS.md` or
  `ROADMAP.md`. Claims are about what should be. Promotion to
  authority requires a follow-up PR amending the authoring authority
  `[live]`.
- **`TARGET`** — worked example showing the destination. Every claim
  is paired with a live-state gap pointer `[live]`.
- **`MIXED`** — narrative / pedagogical documents that intentionally
  span live + proposed + target material in one coherent arc (e.g.,
  "how gunbc models X" walk-throughs). Permitted only if
  (a) per-claim tagging per Rule 2 is applied to every structural
  claim in the body, and (b) a claim-status summary table appears
  near the top of the document showing the live-vs-target split
  `[live]`. The pattern the contract is preventing — silent
  target-as-live narration — is blocked by the tagging discipline,
  not by a doc-level prohibition `[live]`. `MIXED` mode exists for
  narrative docs that would otherwise be artificially split to the
  detriment of the reader.

## Claim tagging

Every structural guarantee in a thesis doc ("CE", "IBC", "compiler
proves X", "complete", "no tests required", "structurally enforced",
"impossible", "prevented by construction") carries one of three tags
in the same paragraph `[live]`:

- **`[live]`** — current tree state. Must cite file:line.
- **`[proposed]`** — requires amendment to `THESIS.md` or
  `ROADMAP.md`. Must cite which.
- **`[target]`** — target state for the mature system. Must cite the
  live-state gap (audit row, lane name, or debt-ledger row).

Bare structural claims without a tag fail review by rule, not by
reviewer judgment `[live]`. The tag makes mode-mixing visible at the
paragraph level, not only at document level `[live]`.

**Self-application.** This contract is itself a thesis-subtree doc
in `LIVE` mode, so its own structural claims carry `[live]` tags.
Where a claim is about a rule codified in this file, the citation
points to the relevant section of this document; where a claim is
about another document (`THESIS.md`, `ROADMAP.md`, `INVARIANTS.md`),
the citation points to that file.

## R1 scope authority

R1 scope — the lane list, acceptance gates, and schedule — lives only
in `THESIS.md` and `ROADMAP.md` `[live]` (see
`THESIS.md:155` ("Thesis claims — complete list" section) and
`ROADMAP.md:15` ("Release R1 Program" section); lane table at
`ROADMAP.md:41-55`; acceptance gates at `:57-76`). No other
document maintains parallel R1 scope authority `[live]` — this
file establishes that rule; its companion authorities above are
the only places where R1 scope content is authored. Other
documents may:

- **Propose** scope changes in `PROPOSAL` mode, with a follow-up PR
  amending `THESIS.md` / `ROADMAP.md`.
- **Reference** current scope by citation.
- **Audit against** current scope in `LIVE` mode.

Tables titled "R1 lanes" or "R1 gates" or "R1 scope" outside the two
authoring authorities are violations `[live]`.

## Worked-example discipline

`TARGET` and `MIXED` mode documents carry a claim-status table
**near the top of the file, before any code** `[live]`. The table
shape:

| Claim | Status | Evidence or gap |
|---|---|---|
| X | `[live]` | `file:line` or test name |
| Y | `[target]` | `T-XYZ` lane, bug-class N, or debt-ledger row |

A caveat dump at the end of a worked example is a violation `[live]`.
Readers must see the full live-vs-target picture before reading any
code. Per-claim tags in the body remain required in addition to the
table `[live]`.

## GAP / PARTIAL single-ledger rule

A document that flags a gap points to one follow-up artifact — the
`ROADMAP.md` tracked-debt ledger row `[live]` (see
`ROADMAP.md:248` "## Tracked debts — 2026-04 analyses" section
header + per-category subsections at `:274` (P0), `:280` (P1),
`:286` (P2), `:295` (P3), `:303` (P4), `:308-359` (post-merge
cohorts), `:360-365` (PR #672 thesis-doc surface)). Documents link
to the ledger; documents do not recreate planning state, proposal
queues, or sub-ledgers `[live]`.

Numbered items (bug classes, lane identifiers, debt rows) live in one
authoritative doc `[live]`. Other docs cite by number; they do not
renumber. Cross-reference drift is a violation of this rule `[live]`.

## Relationship to INVARIANTS

This contract is a thesis-doc-specific enforcement shape of two
existing invariants `[live]` (see [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) P1 Modeling
Faithfulness + `:96` P2 Boundary Discipline):

- **P1 Modeling Faithfulness** — "Documentation Describes Live
  State." Mode declaration + per-claim tagging make the live-state
  claim explicit at paragraph granularity.
- **P2 Boundary Discipline** — "Single Authority." The R1 scope
  rule + single-ledger rule make authority boundaries concrete for
  the most-commonly-forked categories.

Not a new principle; a specific discipline that satisfies existing
ones for documents that repeatedly drift in the same ways.

## Applying to existing and new docs

**Existing `docs/thesis/` docs.** At next non-trivial edit `[live]`:

- Add a mode tag at the top.
- Sweep for structural claims; add per-claim tags.
- If the doc intentionally spans live + proposed + target material
  in one coherent narrative, convert it to `MIXED` mode: per-claim
  tags plus a claim-status summary table near the top, per the
  Worked-example discipline above. Otherwise, split into single-mode
  docs.

Not a bulk-amend-the-tree obligation; apply when touched `[live]`.

**New `docs/thesis/` docs land compliant from the start** —
contract-compliance is a PR-review line item for any new doc under
the subtree `[live]`.

## Scope of this contract

This contract governs documents under `docs/thesis/` specifically.
Documents under `docs/briefs/` (coordination briefs, lane briefs,
receipts) follow a lighter discipline: cite `file:line` for live
claims, cite a `ROADMAP.md` tracked-debt ledger row for `[target]`
dependencies, and respect the single-ledger rule — but are not
required to carry mode declarations or per-claim paragraph tags.
The rationale: briefs are operational documents whose lifecycle is
tied to in-flight work (dispatch, receipts, stale sweeps), not to
the long-lived thesis-claim authority that this contract protects.
If a brief grows into claim-authority territory, it graduates into
`docs/thesis/` and inherits the full contract `[live]`.

## Precedent

This is the thesis-doc surface applying the same `[invariant-reveal]`
discipline that `ROADMAP.md` introduced in PR #669 for tracked
debts `[live]` (see `ROADMAP.md:252-273` "Debt classification —
framing" subsection where the `[invariant-reveal]` tag was
introduced; PR #669 is the originating merge). Pattern: when the
same finding class fires across different sites, graduate the
finding to a rule rather than patching each site.

## Maintenance

This doc is `LIVE` mode about itself: the rules above describe the
current contract `[live]`. If the contract needs to change (new
mode, different tagging syntax, scope extension), a `PROPOSAL` doc
is filed, reviewed, and merged here in a follow-up PR.
