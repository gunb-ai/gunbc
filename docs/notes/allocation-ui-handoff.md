# Allocation frontend handoff

Branch: `codex/allocation-ui`, based on live checkout revision `f2a1b91ae9`.

`GET /allocations` adds a microVM allocation draft page to the existing site and shared navigation. It reuses the shared header, theme, and session UI. The server renders draft editing enabled only for `GoogleOidcSession`; anonymous, invalid-session, and service-evidence requests get a sign-in prompt and disabled fields. Personalized HTML has `Cache-Control: no-store`.

The page accepts draft vCPU, guest memory in GiB, disk in GiB, and lifetime in hours. Native browser validation requires positive integers. Review shows the draft locally; edits invalidate the displayed summary. Placement is any eligible srv host. Defaults are illustrative draft inputs, not fleet capacity, policy limits, or resource grants.

Request allocation remains disabled with an explanation. There is no allocation POST handler, backend call, local storage, fake capacity, fabricated progress, IP, or hostname. The connection section states that no request has been made from this page. No site-wide authentication requirement has been enabled; the user deferred that until login works reliably.

## Integration points

- Page and fields: `dag/gunbc/roadmap/roadmap_allocation_page.dag`.
- Typed client AST and draft behavior (composes the shared account/theme/sound controls without workspace polling): `roadmap_allocation_client.dag`.
- Additional typed CSS: `roadmap_allocation_style.dag`. Shared roadmap CSS also fixes the sign-in link remaining visible under `[hidden]`; its digest pin is re-derived and witnessed.
- Shared navigation path: `roadmap_allocation_location.dag`.
- Shared account script repair: `roadmap_component.dag` now emits block-bodied promise callbacks and references the actual account variables. The inherited script emitted invalid JavaScript (`=> return` / `=> if`) and referenced undefined `b`; browser validation found this. The shared CSS hidden selector now includes the sign-in link; its inline-flex class previously overrode the browser hidden style. Non-successful session responses and fetch failures leave Sign in visible.
- Minimal existing-file edits: shared header link in `roadmap_page.dag`; page/asset routes and no-store response in `roadmap_serve.dag`.

Connect request admission only after the allocation contract exists. The eventual POST must derive owner from the session and use the existing cookie/CSRF/write-admission machinery. A session-rendered form alone does not authorize allocation. Recheck admission at submission. Preserve positive integer units explicitly when converting to modeled resource requirements. Replace draft defaults/options with authored allocation policy when available.

Render accepted request status, placement, readiness, and connection details from the same allocation identity and observed receipts. Do not treat submitted, reserved, booted, and reachable as equivalent states. Define disk persistence, image, access method, expiry, cancellation, and idempotency with the backend work. Login currently uses the site's existing callback behavior; this page adds no new redirect contract.

## Validation

Focused page and route witnesses are in `dag/test/claim/long/allocation_page_witness_test.dag` and `dag/test/claim/roadmap/allocation_serve_witness_test.dag`. Run with the established `systemd-run --user --scope -p MemoryMax=6G` wrapper and `gunbc run --source-root dag --source-root src/v2 --entry <file> --claim-run`.

Local emitted-page browser checks: `/tmp/allocation-browser-test.py`; artifacts: `/tmp/allocation-preview/`. The harness mocks session reads and never reaches the live site or mutates fleet state.

Deployment remains with the operator. This branch has not been landed or deployed.

Final observed validation: 50/50 witnesses PASS (44 existing serve witnesses, 5 allocation witnesses, and the shared CSS digest witness), plus a successful preview export. Log: `/tmp/allocation-ui-validation.log`. The interpreter remained active after all 51 requested results were printed and was stopped; this records the observed verdicts, not a clean process-exit claim. CSS digest: `31fe6a606fac3fa7`.

Both emitted JavaScript assets pass `node --check`. Playwright passes signed-in, anonymous, and failed-session-response checks; native size validation; review and stale-summary behavior; disabled allocation submission; zero writes; no unrelated workspace fetches; no browser exceptions; and no horizontal overflow at 390px. Desktop, mobile, and mobile night screenshots were inspected. Google OAuth and a real allocation were not exercised.
