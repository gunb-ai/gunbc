# Runbooks — operational index

Operational runbooks for the project's infrastructure (the CI compute fabric, hosts, access). These
are *operational* docs, not roadmap items — so they root here, at their own index, rather than from
`ROADMAP.md`/`DESIGN.md` (the doc-graph reachability wall, `gunbc.plans.inert_layer_lens` §8,
roots each doc *kind* at its own root so a runbook does not false-positive as a plan orphan).

A new runbook must be linked from this index in the same PR that adds it — the doc-graph analog of
"an inert lens is a lie".

## Index

- [BMC Redfish operator access (srv1 / srv2)](bmc-redfish-operator-access.md) — enable and verify
  out-of-band Redfish telemetry on the self-hosted CI fleet hosts.
- [BMC assimilator: keyless GCP token via Workload Identity Federation](bmc-assimilator-wif-setup.md) —
  apply-ready SA + WIF setup so `AuthPrintAccessToken()` resolves on the runner with no pasted token.
- [srv3 nbd-proxy ws-upgrade dry-run (§6 gate)](srv3-nbd-proxy-ws-upgrade-dry-run.md) — operator
  procedure to confirm nbd-proxy is compiled into srv3 bmcweb before L2 seed client or capability flip.
- [mtcollins1 stable SSH host key: mint, deliver, anchor](mtcollins1-ssh-host-key-mint.md) — operator/merry-fox-82 procedure for the option-A per-boot host key: mint + Secret Manager upload (stdin only, Q1), SecretRef pin, public-half transcription into the anchored row, gated live-boot readback and the matching that discharges it.
- [srv3 OS install actuation (prefix:os-install-actuated)](srv3-os-install-actuate.md) — runnable
  gunbc prep + operator-gated NBD-proxy virtual-media serve, boot-once CD, and post-install subsumption checks.
