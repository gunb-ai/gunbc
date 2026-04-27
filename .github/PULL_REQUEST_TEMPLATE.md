<!--
For all PRs:
  - Replace this preamble with a short summary of WHAT changed and WHY.
  - Reference any relevant ROADMAP entries, INVARIANTS, briefs, or prior PRs.

If this PR adds, modifies, or expands a hand-Rust file under `src/v3/`
— including Rust tests under `src/v3/compiler/tests/` and any other
hand-authored `.rs` in the v3 tree, since they are part of the SG-0
census (T-PB-A non-test subset OR T-PB-B test subset) — fill in the
"Per-PR dissolution gate" section below per INVARIANTS.md §P5
"Dispatch-Discipline Mechanisms" (b).
PRs that touch only `.dag` source, generated Rust, docs, non-Rust
test fixtures, or hand-Rust outside `src/v3/` may delete the gate
section.
-->

## Summary

(What changed and why.)

## Per-PR dissolution gate (required for new/expanded hand-Rust under `v3/`)

<!--
Per INVARIANTS.md §P5 "Dispatch-Discipline Mechanisms" (b): no new or
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

## Test plan

- [ ] (Bulleted checklist of TODOs for verifying the PR.)
