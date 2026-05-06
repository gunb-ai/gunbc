<!--
For all PRs:
  - Replace this preamble with a short summary of WHAT changed and WHY.
  - Reference any relevant ROADMAP entries, INVARIANTS, briefs, or prior PRs.

If this PR adds, modifies, or expands a hand-Rust file under `src/v3/`
— including Rust tests under `src/v3/compiler/tests/` and any other
hand-authored `.rs` in the v3 tree, since they are part of the SG-0
census (T-PB-A non-test subset OR T-PB-B test subset) — fill in the
"Per-PR dissolution gate" section below per INVARIANTS.md#p5-progress-is-dissolution
"Dispatch-Discipline Mechanisms" (b).
PRs that touch only `.dag` source, generated Rust, docs, non-Rust
test fixtures, or hand-Rust outside `src/v3/` may delete the gate
section.
-->

## Evaluator freeze — `src/v3/compiler/src/test_runner.rs` (R3 Bundle 4b)

<!--
**Delete this entire section** if the PR does not touch `src/v3/compiler/src/test_runner.rs`.

If it **does** touch that file: debt ledger
`docs/debt/r3-debt-paydown-ledger-2026-05-02.md` row **`test_runner.rs` predicate-language growth**
requires a **named dissolution hook** the reviewer can open in **one hop**. Cite **exactly one** of:

1. **`docs/briefs/r2-pr-b-2-runner-extension-bundle.md` — §Runner authority discipline** (dissolution-target table):
   name the **workstream row** this PR advances or the PR that **amends** this table; or
2. **`docs/briefs/r3-pb-runtime-test-predicate-dissolution-hook.md` — §PB-runtime dissolution hook qualification**
   (`#pb-runtime-dissolution-hook-qualification`): PB-runtime **allowed hook destination** — Q1–Q4 qualification
   and disqualifiers in that brief section; name in the PR body how this PR satisfies them when using option **(2)**; or
3. **`ROADMAP.md`**: the concrete bullet **`test_runner.rs` becoming a parallel test-predicate authority**
   (tracked-debts / Pattern B row — search in-tree) as the freeze exception / deferral anchor.

**Frozen without hook:** new or expanded bespoke predicate arms, producer identities, oracle paths, or
observation carve-outs in `test_runner.rs`. PB-runtime hook wiring remains a **parallel** lane — cite **(2)** or
Evaluator brief **(1)** as appropriate; do not duplicate PB-owned implementation in this repo’s Evaluator PRs.

**STOP+PING** if the change needs a **new substrate or census ratchet carrier** instead of a docs-listed hook.
-->

## Summary

(What changed and why.)

## Per-PR dissolution gate (required for new/expanded hand-Rust under `v3/`)

<!--
Per INVARIANTS.md#p5-progress-is-dissolution "Dispatch-Discipline Mechanisms" (b): no new or
expanded hand-Rust under `src/v3/` without a single, checkable receipt.
Fill the single bullet under the gate using **exactly one** of the
three dispositions below — not a mix, not a vague umbrella phrase.

(1) Deletes: name the file or scaffold path that this PR removes or
    fully retires (repo-relative path is enough).

(2) SG-0 census shrink: name the ratchet slice in
    `src/v3/compiler/tests/integration/sg0_census_test.rs`
    (`EXPECTED_HAND_AUTHORED_NON_TEST`, `EXPECTED_HAND_AUTHORED_TEST`, or
    the fragment lists) and give **before → after** entry counts (e.g.
    "`EXPECTED_HAND_AUTHORED_TEST`: 71 → 70 paths").

(3) Explicit deferral: name the **lane or workstream ID** (e.g. T-PB-B,
    SG-2c) **and** cite a **concrete ROADMAP row** — path in-tree plus
    stable heading or table row, or a permalink (GitHub line link,
    `#fragment` that resolves in `ROADMAP.md`). A reviewer must open the
    cited row in one hop.

**Insufficient (do not use as the sole gate answer):** "see ROADMAP",
"TBD", "tracked elsewhere", "follow-up PR", lane name alone, or any
uncited narrative deferral without the row/link above.

If the PR introduces a string/path/name identity bridge (sentinel,
fixture-name routing, `span.file ==` check, `include_str!` side-channel),
it MUST be authored against the §0 identity-carrier pass program (see
PR #810 §0); not as a one-off. Cite the program brief.
-->

- **Exactly one disposition** (delete path **or** census shrink with N→M **or** lane + cited ROADMAP row/link): ___

## SG-0 net-shrink discipline (required when `sg0_census_test.rs` changes)

<!--
CI: `scripts/check-pr-sg0-net-shrink-discipline.sh` (`.github/workflows/ci.yml` `ci` job).

When this PR edits `src/v3/compiler/tests/integration/sg0_census_test.rs`, the
GitHub PR **description** must include a line starting exactly with
`SG-0 hand-path delta:` followed by a signed net path delta for this PR's
census edits (`0`, `+0`, `-3`, `+1`, …).

If the delta is a **strict net add** (`+1`, `+2`, … — not `+0`), the
description must also include a line containing `SG-0 pairing: (a)` **or**
`(b)` **or** `(c)` with the rationale on that line or immediately after.
Pairing classes: **(a)** same-PR retirements (removed paths named), **(b)**
Director-budget citation (URL), **(c)** structural deferral + named follow-up
dispatch.

**Delete this entire section** if `sg0_census_test.rs` is untouched.

Authority: ROADMAP.md bullet *SG-0 PR-window net-shrink discipline*.
-->

**CI reads raw PR description text.** The lines the gate matches must start at column 0 with `SG-0 hand-path delta:` and (when required) `SG-0 pairing:` — a leading markdown list marker (`- …`) or bold wrapper on the same line will **not** satisfy the checker. Paste the two lines below the checklist into the description body as plain text (you can keep the bullets as a personal reminder).

- **Paste into PR description — `SG-0 hand-path delta:`** ___

- **Paste into PR description — `SG-0 pairing:`** ___ (`n/a` unless delta is strict `+N`, `N>0`)

## Per-PR debt-paydown receipt (required for all PRs)

<!--
Per docs/briefs/r3-debt-paydown-program-coordination.md (#1518) and
INVARIANTS.md#p5-progress-is-dissolution (Dispatch-Discipline Mechanisms — velocity tripwire).

Fill this section with a single-checkable receipt for tracked ROADMAP debt
rows touched by this PR. This is separate from the hand-Rust dissolution
gate above: the gate covers SG-0 hand-Rust scaffold discipline; this section
covers ROADMAP debt-row retirement discipline for the R3 Debt-Paydown
standing program.

Use one bullet per touched debt row. Each bullet must use exactly one of
the three dispositions below:

(1) Debt paid: cite the ROADMAP row by path + heading/anchor or permalink,
    and name the retirement mechanism in this PR.

(2) Debt found, routed: cite the ROADMAP row by path + heading/anchor or
    permalink, and name the owning lane plus the filed retirement issue/PR.
    Routing is interim only; the row remains open until the retirement PR
    merges.

(3) No debt touched: use only when the PR neither introduces, modifies, nor
    retires a tracked-debt row.

Insufficient: "see ROADMAP", "TBD", "tracked elsewhere", "follow-up PR",
a lane name without a cited row, or any routed debt without a filed
retirement issue/PR.
-->

- **Exactly one disposition** (Debt paid **or** Debt found + routed **or** No debt touched): ___

## Test plan

- [ ] (Bulleted checklist of TODOs for verifying the PR.)
