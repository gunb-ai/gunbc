# CI on per-job microVMs

**Goal:** eliminate shared-workspace eviction by moving both repositories' CI to attempt-specific Firecracker guests. This plan neither changes runner selection nor authorizes cutover. DESIGN's gap-intolerant replacement rule governs the transition; [launch displacement](microvm-launch-displacement-analysis.md) owns the source-path analysis.

**Evidence:** [run 34562801087](https://github.com/gunb-ai/gunbc/actions/runs/34562801087) exposed runner re-registration into an occupied slot's workspace. Per-attempt guests remove that shared write path. This does not establish the cause of memory thrash, cure timeout/cancel behavior, or replace typed environment-versus-code classification. Repairing existing host registrations is separate work.

**Method:**

1. Resolve `runner_host_image_placeability` with architecture-correct boot execution. BMC reachability or an access-refused work item does not measure image placeability.
2. Make the modeled attempt-launch path execute from tmpfs at host boot, with an image built from that model. Converge-time execution presupposing a runner/checkout cannot replace PID1. Keep the replacement in shadow until the switch; it must not resolve through the old path.
3. Exercise the lifecycle controller's attempt-bound JIT mint/staging, nft installation/readback, workspace-format failures and PID1 missing-secret/stage2 refusals. Retain grant/prior-resource/envelope/signature/unit/boot/nonce/attempt/digest/floor/ceiling/VM-config admission. CI is not granted weaker FCI-3. Live cgroup census may defer for first internal dogfood, not commercial tenancy.
4. Rebuild and redirect to the measured image digest; switch the production root and retire the recovered init together. No dual selection or “replacement when available.” Bind `host_boot_cutover_frontier` only when old init has zero production consumers; compiling the planner or keeping it beside init is insufficient.
5. Move private workflow selection first, then public lanes, using modeled attempt-specific labels and guest memory/job envelopes. Capacity must be named: serialization, another canary, or an explicit parallelism drop with restoration trigger. Only public CI through the qualified canary starts the historical dogfood observation clock.

**Cost assumption:** historical whole-job comparisons supported accepted-cold builds after removing repeated world acquisition. That is a working assumption, not an Aarch64 cold-build measurement: the cited amd64 lower bound retained Cargo cache. Reopen the cache gate if an Aarch64 run with wiped `CARGO_HOME` pushes whole-job time toward the recorded cancellation class. External cache access needs its own admitted guest-egress capability; it is no default. A prebuilt image binary must not hide the private workflow's public-authority changes. Network access required for Actions is distinct from optional fleet-cache access.

**Validate:** on the actual host-boot jailer path, absent/empty credential creates zero VMMs and a live admissible grant creates exactly one; VMM exit zero remains `WorkNotEstablished`. Measure JIT push, vCPU-minutes and attempt-bound destruction. Report the minimum rung across paths until displacement completes. Retire `toolchain_filesystem_probe_dissolution_condition` only when both public and private required selections make shared-filesystem eviction impossible; identifying its deleter is insufficient. Preserve the single-process roster improvement and environment/code classification.

The original account retains the dated frontier states, job comparisons and lower-bound invocation. Those readings are not current merge, host or dogfood status; no cutover occurs in this cleanup.

[Prior design and dated receipts](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/ci-per-job-microvm-migration.md).
