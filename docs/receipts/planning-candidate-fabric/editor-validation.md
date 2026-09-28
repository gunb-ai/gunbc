# Planning editor qualification

2026-09-28. Dependency: `ada7daefc28696ce4ba242437cd26daf61bd0bab` (#12465).
The original focused-run source identity is recorded in `editor-source-hashes.json`; the later
full-serve result below has its own clean-commit anchor. Neither is production deployment evidence.
Original focused-run binary SHA-256:
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

## Full serve gate: corrected and passed

The earlier broad run and serve compile exceeded 6 GiB. Those attempts were inconclusive, and
focused tests did not establish integrated readiness. A later 26 GiB strict compile found real
errors: omitted `attempt_stale` and response `headers`, missing `PlanningApplied` rendering,
stale sandbox/issue call signatures, invalid generic request construction, and the legacy broker's
different request evidence type. These were corrected without weakening the compiler gate.

At clean code commit `827cb0e6afc59c07b99cb04c24f710655d666d7b`, the branch-built binary compiled
all 1,049 modules and bound a listener for `roadmap_serve_handle_srv2_preview`.
`serve-readiness.json`, `serve-readiness.log`, and `serve-build.log` retain the result. The gate
stopped the process immediately afterward. Build took 4m55s; serve reached readiness in 348.98s.
Binary SHA-256: `77f6fb4bf54d28a76a509308ebc881f2a652d6869ec15a1ff7a4c893f5cb5605`.

The 40 planning controls and two new transport-adapter controls passed using that branch-built
binary. Their outputs are `branch-built-planning-controls.log` and
`approval-transport-controls.log`. The adapter passes only the transport observation to the
legacy broker's existing identity-header trust/capability/writer checks. It does not turn Google
or service authentication into tailnet identity, and does not claim the A7 migration is complete.

This clears the full-serve compilation/readiness blocker. No HTTP requests were sent, so no
protected credentials or live store were accessed. Production session handling, post-acceptance
dashboard refresh and protected storage commissioning remain separate runtime gates. PR #12465's
HOLD remains in force; no preview unit, live dashboard, deployment or merge was changed.

The subsequent receipt/documentation commit changes no executable source; the tested code commit
above is the evidence anchor.

## Reproduction

Full serve gate (builds this checkout, then requires readiness):

```sh
python3 tools/tests/roadmap_serve_readiness_control.py
```

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
