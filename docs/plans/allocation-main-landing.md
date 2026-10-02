# Allocation stack landing path

Audit: 2026-10-02. Main: `1a013596270`. Existing integration checkpoint: `e7c4426229c`. This document is a landing plan, not merge approval or commissioning evidence.

## Current branches

| PR | Head | Target | Standing |
|---|---|---|---|
| #12465 | codex/allocation-demo | main | Historical broad auth/storage/allocation draft; ancestor of the vertical |
| #12505 | codex/allocation-vertical | main | Stale parent; conflicts with main |
| #12691 | codex/workspace-commissioning-production | codex/allocation-vertical | Child includes both the parent feature work and repeated main merges |

The child contains 279 commits beyond its parent. Of those, 163 are already ancestors of current main. Against main the child has 116 unique commits (including merges), and main has two commits absent from that checkpoint. Its actual net main diff is still 792 files, 50,350 additions and 3,008 deletions; 415 changed files under docs/plans account for 16,835 additions. Reducing the commit count alone would not make this a small source review.

`codex/allocation-main-landing` merges the two newer main commits into the integration in an isolated worktree. The merge was clean. The original three PRs and their reviews have not been rewritten, closed, merged, or retargeted.

## Concrete first extraction

`codex/compiler-qualification-main` starts directly at current main and contains only the compiler qualification prerequisite: stable-source lexer cursor, per-file classification sharing, native/transition fixtures, and service-prefix admission repair. The HTTP hunks from cli_run.rs are deliberately excluded. The extraction is one source commit, 20 files, 1,056 additions and 191 deletions, with a hash manifest. Original qualification evidence remains scoped to its original revision; this branch requires its own checks.

## Remaining review units

These are dependency boundaries to extract and qualify, not claims that the remaining splits already exist:

1. **Authentication and protected state.** Browser-bound login, cookie/header transport, owner admission, protected state access and CAS contracts needed by allocation. Preserve their negative controls. Use #12465's reviewed design, but build the remaining delta against current main rather than merging its stale historical tree.
2. **Workspace request and lifecycle.** Sized durable requests, reservation binding, workspace payload, access observations, retirement and recovery. Reconcile #12505 over the landed foundation so its base no longer hides unrelated main changes.
3. **Initial commissioning.** Retain #12691's protected journal, host-owned executor, exclusion, exact invocation/readback and supply commitment. Once its prerequisite source is on main, target its narrowed remainder at main. Retargeting the current broad child now would carry all unlanded foundation work into that review.
4. **Separate follow-ups.** Broad tracker/task-history migration and unrelated presentation changes need their own cutovers. Keep historical receipts accessible on the preserved integration branch; carry compact evidence with each landing cut rather than copying every historical snapshot into each PR.

For every extraction, compare the resulting composed tree with the preserved integration checkpoint and account explicitly for omissions. Seal/access boundaries and generated projections must be checked together; a file-name partition is not evidence of an independent compile closure.

## Qualification and operational gate

The latest integration run `37027879359` completed: generated and emit-build passed, but floor/witnesses failed with `NonFoldResidueRosterDiverged unrostered=290`. Preserve and diagnose those specific sites; do not bulk-enrol exceptions merely to obtain green CI. No new full integration CI run is justified solely by this landing-plan document.

No VM commissioning, deployment, readiness publication, reservation or host mutation is performed by this restructuring. After the narrowed source stack is qualified and reviewed, retain the existing live sequence: exact artifact installation/readback, reviewed commissioning plan, cancellation/reattachment, committed readiness, no-op replan, then real allocation, SSH, release and same-slot reuse.
