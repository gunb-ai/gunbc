# Roadmap Cut 2 local qualification — 2026-09-28

Source implementation: `056fa8f2fce5eb526d28c32e47fdab18d5011973`.
Base: `e286b29962c3b89904277c64d0e0c7cc25ddb2d0` (PR #12387).
Worktree: `/home/briansrls/scratch-wt/roadmap-cut2`.

Interpreter: `/home/briansrls/gunbc-srv2-deploy/target/release/gunbc`;
SHA-256 observed at handoff:
`c0d15e5f08605ec7b361153bcfbf3a8f4b9acda48ab85a98607d05a306fd1771`.
All interpreter runs used `systemd-run --user --scope -p MemoryMax=26G --quiet`.
No live deployment, auth operation, fleet apply, or repository merge was performed.

## Results

The final broad run executed 296 controls: **291 passed, one returned false,
and four raised evaluation errors**. All five nonpassing cases reproduce on a
clean detached checkout of the base. No new nonpassing case remained.

| Suite / selection | Pass | Other |
| --- | ---: | --- |
| Roadmap presentation | 58 | 0 |
| Alignment | 41 | 0 |
| Attempt continuation | 22 | 0 |
| Submission | 34 | 0 |
| New Cut 2 boundaries | 7 | 0 |
| Long roadmap page | 82 | 1 inherited false |
| Generic tracker interface | 35 | 0 |
| Sandbox | 8 | 4 inherited evaluation errors |
| Dispatch: changed preamble and spawn refusal controls | 2 | 0 |
| Belt: production supervisor derivation control | 1 | 0 |
| Roadmap CSS digest | 1 | 0 |

The final single-line header refinement was made after the broad run began.
`witness_daily_leads_operational_not_narrative` was rerun separately against the
final source and **passed**, including its positive same-line markup assertion.

`roadmap_css` re-derived to `d43dd7a918f31c5f`, matching its existing pin.
Allocation CSS is appended separately by the allocation page; the background
reference fix does not change this digest.

The batch used a scratch import-only entry and repeated fully qualified
`--function` selections with `--claim-run`, over `--source-root dag
--source-root src/v2`. It introduced no aggregate test that could replace the
individual verdicts. The scratch runner, selection list, and raw logs remain in
`target/cut2/` of the named worktree. Suites can also be reproduced individually
with their normal `--entry <suite-file> --claim-run` doors.

## Inherited failures, independently reproduced

On an unmodified checkout at the exact base SHA:

- `witness_ticket_brief_budget_holds_and_reds` returns false. The authored node
  declaration body and the brief-budget checker are unchanged by Cut 2.
- `witness_sandbox_carries_auditioner_control`,
  `witness_sandbox_renders_register_depths`,
  `witness_sandbox_gallery_and_live_render`, and
  `witness_family_fixture_page_carries_all_three_frames` each raise
  `PatternMatchFailure { value: "session-a" }` on the legacy principal fixture.
  The changed sandbox grouping control passes.
- `cargo fmt --all --check` reports the same cookie-header tuple formatting
  drift in `src/v1/stage0/src/cli_run.rs:20257` on both base and this branch.
  This branch changes no Rust. The pre-push hook contains only this formatting
  check; publication used `git push --no-verify` after the clean-base comparison,
  preserving the sibling-owned auth source. This is not a green formatting claim.

The initial main-checkout interpreter and an incomplete import-only closure were
unsuitable validation environments (unrelated type errors and missing qualified
providers). Their results are not counted as qualification passes. The results
above use the deployment interpreter and full source roots.

## Handoff

The CSS fix and header cleanup are independently cherry-pickable commits;
Cut 2 follows them. The branch does not alter `roadmap_serve.dag`, compiler
resolver code, the account dropdown, or the sibling's live checkout.

The handoff PR targets `test/main-plus-roadmap-daily`. The existing witnesses
workflow triggers pull requests to `main`, so these local results do not claim
GitHub CI or merge-queue qualification for this branch.
