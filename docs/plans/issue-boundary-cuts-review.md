# Issue boundary cuts and project focus — review index

Base: `e286b29962c3b89904277c64d0e0c7cc25ddb2d0` (PR #12387).
Branch: `codex/issue-cut1-toposort`. Implementation is isolated from the serving checkout.
Nothing in this branch is merged or deployed. The auth flow, compiler, fleet configuration and
held allocation PR #12465 are untouched.

## Review order

1. `d446000e2d6` — Cut 1: command/state boundary. Private carrier snapshots, exact remote-ref
   CAS, atomic event + operation receipt, original-result retry, native URL-encoded forms and
   truthful detail-history refusals. [Detailed receipt](receipts/issue-cut1-command-boundary-2026-09-28.md).
2. `b09db59f677` — Cut 4.1: five Workspace administration obligations over the existing goal
   assessment vocabulary. Row 4 has a supported Policy API read, but manual mutation. The
   others use bounded console readback. No Google setting or credential was changed.
   [API qualification](receipts/google-workspace-admin-api-2026-09-28.md).
3. `abd2d6857f2` — Cut 2: independent nesting, shared by worker alignment, supervision,
   integration and presentation. A single event transition produces standing and assignment;
   progress actors and attempt existence do not assign ownership. Typed ticket fields survive
   presentation; description uses the brief; fleet placement leaves Google's extdeps model.
4. `8e8796ea0d5` — Project focus: clicking a title focuses its recursive project, displays the
   recent three completed items and upcoming work in dependency order, and offers “…” for
   earlier work. Open selected issue retains detail navigation; modifier-click keeps native
   link behavior. Undo restores order/focus/expansion, column sorting exits topological mode,
   and cycles refuse before changing the view.

The first authored nesting migration is shell → DAG batch 1: the ten `shell-dag-*` work items
belong to `shell-typed-invocation`, as the existing batch contract explicitly states and the
operator selected for this acceptance case. Other reverse dependency links were **not** copied
into ownership. Their scheduling meaning remains intact. Additional projects require authored
membership; issues with no nesting stay independent. Both ends of nesting are the same work-item
kind. There is no separate Epic/Task hierarchy; the redundant hardcoded Task chip is removed.

The old tree's shared-prerequisite links remain dependency links, but do not inflate owned-child
progress counts. This prevents the old “0 of 1” project claim for work owned somewhere else.

## Validation boundaries

The changed workflow has focused pure, actual carrier and browser controls. The full page
suite still reports the pre-existing ticket-brief budget failure. A run against an untouched
worktree at the base reproduced these six violations: `2-scm-native-authority-program` (134),
`floor-ceiling-roster-cut-completeness` (123), `frontend-eval-training-program` (177),
`fleet-mtcollins1-first-host` (153), `fleet-slot-retirement-wet-proof` (116), and
`cardinality-vertical-slice` (119). The limit is 100 words. This branch does not weaken it or edit
unrelated ticket contracts.

The full serve witness closure previously exceeded the unchanged 6 GiB limit (exit 137).
Focused passes are not a green integrated floor. This branch remains a draft for review, not a
merge/deployment recommendation. Each receipt names the executed boundary; a typecheck is not
reported as an executed HTTP acceptance test.

Known boundaries retained from the handoff: first-time Fabric assignment still invokes the
existing inline launch path; this cut does not install a durable launch-obligation consumer.
Per-issue attempt observation remains unavailable with an explicit rendered reason. Workspace
readback types are not a deployed Google observer; receipt producers must authenticate and
retain their evidence. Broad protected-storage cutover remains in the held allocation lane.

## Reproducing the narrow controls

Each control is a `.dag` entry, run with `gunbc run --source-root dag --source-root src/v2
--entry <entry> --claim-run`. The Git CAS/race/replay control is
`test.manual.issue_command_wet`. The emitted clients come from `test.manual.issue_assignment_client`
(`write_issue_sort_script`, `write_assignment_client_script`). The form codec is
`test.claim.http.form_urlencoded_witness_test`. The Python browser drivers and closure copier that
were used locally had no `.dag` authority and no consumer, so they were removed rather than landed
as unmodeled harnesses (DESIGN §6). The browser-side controls have no committed instrument until
they are modeled.

## Final local results

- Page: 83 PASS; 1 FAIL, the six brief overruns reproduced on the untouched base.
- Presentation: 57 PASS. Alignment: 42 PASS. Continuation: 22 PASS. Submission: 34 PASS.
- Event-state: 19 PASS. Tracker: 34 PASS. Fleet component projection: 2 PASS.
- Recursive nesting: 3 PASS. Workspace administration: 5 PASS. Form codec: 2 PASS.
- Actual Git CAS/race/replay commands: PASS. Native Chromium form → DAG decoder: PASS.
- Emitted assignment retry client: PASS. Emitted project-focus/sort client with served CSS: PASS.

These results are re-derived by the entries named above; the transcribed logs are not committed
(DESIGN §6: name the instrument, never transcribe its output). The late detail-parameter forwarding and whitespace cleanup are
small follow-ups to the executed page snapshot; they do not change the default served inputs.

The ordinary push was rejected by `.githooks/pre-push`: inherited Rust formatting drift at
`src/v1/stage0/src/cli_run.rs:20257`. `cargo fmt --all --check` reproduces it on the untouched
base. The hook explicitly documents `git push --no-verify` as its override. That override is
used only to publish this draft for review; Rust source and the hook are unchanged. No landing
check is represented as passing because the push was allowed.
