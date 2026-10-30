# srv3 WebSocket endpoint probe

**Goal:** distinguish an NBD-proxy WebSocket route from ordinary HTTP routing without mounting media. `extdeps.bmc.webui.nbd_proxy` owns the protocol; board/firmware observations belong to `extdeps.bmc.capability` and the ASRock model.

**Standing:** the recorded 2.07.00 observation names `/vm/0/0` as present and `/nbd/0` as absent. This historical procedure is not a fresh receipt or evidence for another firmware. Read the current modeled firmware and mechanism solution before planning an install.

**Method:** obtain explicit approval for the upgrade, which may spawn a BMC process. Resolve the host and credential through the existing BMC model, authenticate before probing, then use generic WebSocket tooling for a bounded upgrade-only connection. Record endpoint, status, connection lifetime and first received bytes. Disconnect without sending an NBD greeting, export size, transmission flags or ISO data; independently check that no virtual medium appeared. Do not reset the host to resolve ambiguity.

**Validation:** an authenticated upgrade and observed protocol behavior support endpoint presence, not media attachment or successful installation. Ordinary GET 404 is inconclusive. Authentication, TLS and reachability failures leave capability unread; they do not establish absence. Preserve ambiguous results for reproduction. A capability change requires its independent observation and model update; this procedure alone licenses none.

The [original procedure](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/runbooks/srv3-nbd-proxy-ws-upgrade-dry-run.md) is historical provenance, including its retired endpoint assumptions. Use [BMC access](bmc-redfish-operator-access.md) for credential handling.
