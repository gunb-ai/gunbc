# Planning editor qualification

2026-09-28. Dependency: `ada7daefc28696ce4ba242437cd26daf61bd0bab` (#12465).
Implementation source identity is recorded in `editor-source-hashes.json`; this receipt is not
production deployment evidence. Binary SHA-256:
`e6d8571156538304137139853153e1d30f77f99ba71d8c76ccd3a943b8875e5b`.

## Passing boundaries

- 40 directly invoked DAG controls: candidate review, immutable publication/readback/retry,
  actual atomic application with children, stale/wrong-principal/invalid-graph refusals,
  HTTP authorization and CSRF, accepted dashboard/worker inputs, source status preservation,
  historical event identity and Fabric carrier behavior. `qualification-passes.log` is the
  PASS-line extract, not the complete filesystem trace.
- Browser control: actual HTML/JavaScript and loopback DAG handler with fixture authentication,
  real temporary Fabric file storage, typed edit, draft child, review, acceptance, readback,
  literal rendering of an HTML injection payload, CSRF refusal and original-result retry.
- Independent CLI processes racing two candidates based on the same stored revision: exactly
  one commits; the other reports conflict. Readback has the complete five-event transaction,
  and retry returns the winner's original result. This exercises real file-store head CAS.
- `git diff --check` and repository `cargo fmt --all --check`. A separate formatting-only commit
  normalizes inherited Rust whitespace so the existing pre-push guard can run without bypass.

The browser fixture has no real Google login or protected service authentication. The race uses
local Fabric files, not the commissioned remote service. Test authentication lives only in the
manual fixture module, never in the production route.

## Unresolved integrated gates

The broad imported integration run and full production `serve` compile both exceeded the
existing 6 GiB memory cap (exit 137). Neither counts as a pass. No requests were sent to the
production handler; no protected credentials or live store were accessed. The production
post-acceptance dashboard refresh is wired but has not been exercised against a live instance.

PR #12465 remains draft and held at the dependency SHA above. Its commissioning/migration and
integrated qualification must finish before this feature deploys. No merge or deployment occurred.

## Reproduction

From the checkout, with a compatible binary selected via `GUNBC_TEST_BINARY`:

```sh
python3 tools/tests/planning_editor_qualification.py
python3 tools/tests/dag_validation_closure.py test.manual.planning_editor_fixture --output target/planning-validation/browser-source
python3 tools/tests/planning_candidate_race_control.py
```

The closure helper copies whole modules byte-for-byte and emits a SHA-256 manifest. It does not
strip declarations or substitute production behavior. Qualification invokes each test directly.

For the browser, seed a fresh `/tmp/gunbc-planning-browser-*` root using `planning_fixture_run`
(`mode=seed`), then serve `planning_fixture_handle` from that closure on loopback with the same
`GUNBC_PLANNING_TEST_ROOT`. Run `tools/tests/planning_editor_browser_control.py` using a Python
environment with Playwright; `CHROMIUM_EXECUTABLE` and `PLANNING_TEST_URL` can select the installed
browser and loopback port. Stop the fixture server afterward. Never deploy the fixture module.
