**6c.6b The launch-side marker contract: the attempt is decided by markers, never by exit codes.** The keeper's
pre-launch checks all refuse through one function that prints `PRE_SCOPE_REFUSAL: <why>` to stderr (and to
`drv.log` once it is open) and **exits 0 by design**. Reading a refusal's exit code as success is therefore the
default failure mode, and this packet's contract is: **key on the marker**. The states, in the keeper's own words:

| marker | meaning | what it means for the one attempt |
|---|---|---|
| `PRE_SCOPE_REFUSAL: …` | hard refusal **before** launch; `systemd-run` was never invoked, so the workload never ran | **NO-ATTEMPT** — proven no-launch, so the budget stays intact (the keeper treats it this way; whether it spends a granted attempt is keen-otter's call, not mine) |
| `FROZEN COPY SHA MISMATCH` | the staged copy's sha256 ≠ field 8 | **NO-ATTEMPT** (nothing ran); prevented outright by 6c.3 pinning field 8 to the published bytes |
| `LAUNCH_PROPERTY_MISMATCH` | a bounds difference found by readback **after** start | does **not** stop work already running — which is exactly why the packet gates its own cgroup (G1/G2) instead of trusting the launcher |
| `SCOPE_START_UNPROVEN`, `SCOPE_END_UNPROVEN` | ambiguous; the keeper reconciles | never recorded as a verdict here; reported to the keeper, and this packet's rows stay uncollected until reconciled |
| `SCOPE_END_OBSERVED` | clean end of the scope | the only state in which the frame's own completion gate (G6) and collection rule (G7) are allowed to produce rows |

There are **no warnings** — every refusal is hard. Note the asymmetry this creates and how the frame answers it:
the launcher cannot stop work that has already started, so `LAUNCH_PROPERTY_MISMATCH` is informational only, and
the five limits and the cgroup leaf are enforced *inside* the script (6c.6 G1/G2) at exit 2, before any claim runs.
