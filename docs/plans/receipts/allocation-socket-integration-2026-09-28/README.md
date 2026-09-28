# Restricted-socket integration qualification

The socket changes from open PR #12482 at
`6c6a238c9d7607b4f596ab814d84866c024ed791` are integrated at `82c8bf43f`.
Unrelated compiler/bash changes were excluded. Protected service admission and
exact refusal controls were corrected at `42d676c53`.

The branch binary built successfully under 6 GiB/no swap. Its SHA-256 is
`bf9fae33a19042737978cd913c77f7aab3548a83ab4d58c59f34b6c6404ead9f`.
Full preview startup at `82c8bf43f` resolved 850 modules and reached listening
readiness in 297.21 seconds, peak 12,018,429,952 bytes, no swap/OOM events.
The helper stopped its own server; this is neither deployment nor route acceptance.

The actual protected Unix-socket fixture at `42d676c53` passes all receipt steps,
including authenticated append/read, restart, exact HTTP 400 above-cap refusal,
exact HTTP 403 peer/signature refusals, cookie/response-header transport, and a
stopped-server negative control. The injected root-proxy/no-login case is a
modeled control, not a real Tailscale/root-peer observation. The normal socket
case uses the real kernel peer account. Fixture keys were removed and are not
included. Temporary store contents are not production state.

The fixture server first OOM-killed under the obsolete 6 GiB server allowance
(scope run-rd682235616c042598bcc4e9495b8308d). It passed with the existing serving
budget: 53 GiB max, 49 GiB high, no swap. Clients remain limited to 6 GiB.
No deployment budget was changed.

18 file/handler, 12 wire, seven roster and two identity controls passed on the
socket integration using the pre-transport-build evaluator binary. The real
Unix fixture and full preview used the newly built binary above.

The live, read-only identity receipt establishes that srv2's proxy request has
no login principal. Signed service authentication, not a fabricated public
login-roster entry, therefore admits protected proxy traffic. Invalid or absent
service signatures still refuse; public write admission is unchanged.

No production storage write, installation, commissioning, VM launch or merge
occurred. Queued-operation fencing/drainage and live lifecycle acceptance remain
outstanding. The allocation HOLD remains.
