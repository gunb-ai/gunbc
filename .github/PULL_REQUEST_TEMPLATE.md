<!--
For all PRs:
  - Replace this preamble with a short summary of WHAT changed and WHY.
  - Reference any relevant ROADMAP entries, INVARIANTS, briefs, or prior PRs.

If this PR adds, modifies, or expands a hand-Rust file under `src/v3/`,
fill in the "Per-PR dissolution gate" section below per
INVARIANTS.md §P5 "Dispatch-Discipline Mechanisms" (b).
PRs that touch only `.dag` source, generated Rust, docs, tests, or
non-`v3/` Rust may delete the gate section.
-->

## Summary

(What changed and why.)

## Per-PR dissolution gate (required for new/expanded hand-Rust under `v3/`)

<!--
Per INVARIANTS.md §P5 "Dispatch-Discipline Mechanisms" (b): no new
hand-Rust file in `src/v3/` lands without naming the file or scaffold
it deletes (or explicitly defers, with a named row). One of:

- "Deletes <path>"
- "Shrinks census line <name> in sg0_census_test.rs from N to M"
- "Defers to lane <ID> with named ROADMAP row <link>"

If the PR introduces a string/path/name identity bridge (sentinel,
fixture-name routing, `span.file ==` check, `include_str!` side-channel),
it MUST be authored against the §0 identity-carrier pass program (see
PR #810 §0); not as a one-off. Cite the program brief.
-->

- Deletes / shrinks / defers: ___

## Test plan

- [ ] (Bulleted checklist of TODOs for verifying the PR.)
