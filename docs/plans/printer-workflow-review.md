# Printer workflow review status

The printer start paths share one approval and claim implementation. Dispatch input validation
is not authorization: the common path verifies an unexpired ntfy grant, observes an explicit
IDLE or FINISH report with zero or the vendor-confirmed spurious 0500C011 code, and acquires a durable claim for the approved escalation ID before any
upload/start. Active/paused/unknown states, partial reports and other error codes refuse. Approval text requires the operator to
check the actual bed when approving. A second readiness observation and approval-expiry check occur
immediately before publishing the start.

Claims live in `/var/lib/gunbc/printer-starts` on the separately modeled printer LAN host. The
host check precedes the gate. The service account needs an owner-controlled, persistent directory
there before this path can operate; missing/unwritable storage refuses. Do not delete a claim to
retry an uncertain start. This review has not provisioned that directory or performed a live print.

One exact-version credential session covers preflight, upload, post-upload observation and start.
It is evidence of an existing-secret read, never PrinterCredentialProvisioned. Receipt source is
projected from the selected token source. The observed setup report is informational only.

The census distinguishes separately initiated operator requests from unattended provisioning.
Physical print starts select operator approval. Current code-enforced approval under a federated
read and the operator-token compatibility entry have explicit, bounded divergences; neither is
presented as a broker-minted printer credential. Token-file input does not establish credential
custody and is not authorization to relay tokens through chat.

Local approval, census and workflow-binding witnesses pass. CI is required and the PR remains
draft. CAD and slicer preparation are split out; approval-client cutover is prerequisite #13218.

Vendor-readiness correction (2026-10-04): Bambu Studio treats FINISH as a completed
job eligible for another print, not a cooling state. The bed may still hold a part in
either IDLE or FINISH, so fresh job-bound bed-clear approval remains mandatory.
The exact 0500C011 code is vendor-confirmed non-existent (BambuStudio issue 4495,
comment 2275068828); no broader nonzero-error exemption is introduced. Tests retain
refusal of the actual prior SD fault 83902511, adjacent codes, active/paused states,
missing/duplicate fields and string-typed errors. The durable claim and post-upload
readiness/approval-expiry checks remain unchanged.
