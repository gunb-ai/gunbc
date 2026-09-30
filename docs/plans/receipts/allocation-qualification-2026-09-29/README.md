# Allocation full-server qualification

Source: `0990a2ab7120a1a663bdb2fbf76b6fd53b9828eb` on `codex/allocation-vertical`.
The source diff was empty. The branch-local release build completed in 8m52s
under 6 GiB with swap disabled. Formatting and whitespace checks passed.

The full `roadmap_serve_handle_srv2_preview` entry reached its listening signal
in 287.119 seconds after resolving 851 sources. It used the exact branch binary
identified in receipt.json, full dag/ and src/v2 roots, and a local ephemeral
loopback port. The existing serving-budget allowance was used: a 12 GiB cap,
zero swap, peak 12,008,845,312 bytes (11.18 GiB), zero OOM events. The qualification
helper stopped its own server after readiness. No HTTP requests or host mutations
were performed, and the production site was not deployed.

This establishes integrated server startup, not allocation acceptance. CI run
36512520028 is pinned to the same source: compiler passed; nominal witnesses and
other required lanes remain outstanding at this checkpoint. Review HOLD is
already cleared and is not being reintroduced as a gate.

## Remaining production connections

The existing fleet plan and lightweight consumer both discover durable intent,
but their unplanned-live branch still records a preparation wait. No production
caller currently joins real commissioning, image, host/tool, gateway and provider
policy evidence into workspace_prepare_convergence_from_observations.
workspace_commissioning_admit and workspace_refresh_gateway_wet likewise have
no production callers establishing initial commissioning or recurring access
observations. These are implementation gaps, not merely missing test receipts.

Protected credential custody (#12580) must first pass its generated-artifact and
integrated checks, then install through the existing convergence workflow. Storage
and socket commissioning, exclusive slot transfer/readback, image/controller
installation, recurring request consumption and observation publication follow.
The acceptance campaign then starts at real Google login and website submission,
not a manual preparation helper, and includes external SSH, retries/restart,
release and larger same-slot reuse, all expiry windows, and uncertain cleanup.
