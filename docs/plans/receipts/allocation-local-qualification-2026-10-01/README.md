# Local commissioning qualification

Subject: 0ae5218c819a2dc0731c394eb96b08b3390752e0. Its tree is identical to dd5df8d5b38;
the additional merge resolves the allocation parent's generated-ledger conflict.
The ledger was regenerated from gunbc.design_ledgers.expected_design_rung_drops_md.
GitHub subsequently reported the PR mergeable; draft status was retained.

All 86 currently declared fleet-plan tests passed in four fresh processes (22/22/22/20).
Every expected identity occurred exactly once; each process exited 0. The driver checked
the compiler digest and every snapshot module digest before and after each shard.
The byte-identical module snapshot contains 1159 modules. Each shard ran under 12 GiB,
no swap, with MALLOC_ARENA_MAX=2. This is the focused fleet suite, not the required CI floor.
The historical 87th test was moved into the workflow suite in the earlier qualification
repair; the current fleet entry declares 86, and no test is silently omitted here.

All 32 process-boundary executor controls passed over freshly exported model-emitted
Bash, including the temporary user-manager dispatcher-exit case. These use explicit
process doubles; no real controller, protected allocation store, or fleet unit was run.

The combined required witness lane OOM-killed under 12 GiB while concurrently preparing
the lane roster and primitive-runtime-body child: kernel readback showed about 6 GiB
resident in each process. The standalone floor also OOM-killed during preparation of
its 2744-module closure. Neither run reached a witness verdict or establishes complete
integrated qualification. No limit was raised and no requirement was removed.

Remote run 36913346016 separately reached producer warming and refused because
runner_microvm_boot_probe used Filesystem.Write without an explicit Filesystem import.
The accompanying source repair names extdeps.filesystem.filesystem_io, the service that
owns Write. It does not alter claim-scope refusal or remove the producer from its roster.
The fleet results above precede this one-line repair; its focused controls and fresh CI
are separate evidence.

No installation, commissioning, initial readiness, allocation, VM boot, SSH, release,
or reuse is claimed by this receipt.

The explicit boot-probe service import passed all 8 focused boot-probe controls,
exit 0, in a fresh 596-module byte-identical closure under 6 GiB/no swap.
These are model controls; they do not start a VM. See boot-probe-import.log.
