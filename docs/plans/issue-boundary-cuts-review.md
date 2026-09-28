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

Use the existing pinned binary (`sha256 c0d15e5f08605ec7b361153bcfbf3a8f4b9acda48ab85a98607d05a306fd1771`).
`tools/tests/dag_validation_closure.py` creates byte-identical import closures and a source-hash
manifest under `target/`. It does not edit the imported modules.

```
python3 tools/tests/issue_command_wet.py
python3 tools/tests/dag_validation_closure.py test.manual.issue_assignment_client --output target/issue-sort-validation
systemd-run --user --scope -p MemoryMax=6G -p MemorySwapMax=0 --quiet ./target/release/gunbc run --source-root target/issue-sort-validation --entry target/issue-sort-validation/test.manual.issue_assignment_client.dag --function write_issue_sort_script --arg path=/tmp/issue-sort-script.js
CHROMIUM_EXECUTABLE=/home/briansrls/.cache/ms-playwright/chromium-1217/chrome-linux/chrome /tmp/pwenv/bin/python tools/tests/issue_sort_browser_control.py
```

The sort browser control can also load `/tmp/issue-sort-style.css`; validation used the stylesheet
read from the existing served page. This branch changes no stylesheet bytes, so no CSS parity pin
was changed. It uses fixture progress observations; it does not claim those example rows are
currently completed in production.

The assignment browser control uses the same emitter entry's `write_assignment_client_script`
function and `/tmp/issue-assignment-script.js`, then
`tools/tests/issue_assignment_browser_control.py`. The native form control is
`tools/tests/issue_form_browser_control.py`, after creating the
`test.claim.http.form_urlencoded_witness_test` closure at `target/form-codec-validation`.
