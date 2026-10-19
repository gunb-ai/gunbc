# Pre-registered prediction — emit-and-build gate (stern-bat-401 msg_db49ee15 ruling (2)=A)

Run: `gate_emit_build_14152.sh <sha> <label> <kind>` on the runner, fresh clone, printed SHA.

P1. HEAD e5cc5d903e1 (repair, PR-1 head), kind=repair: SELF_HOST_RC != 0. The log carries
    `EmittedCompilerBuildFailed` and `error[E0277] ... S: Clone ... std_realization_measurement.rs:341:72`
    (reproducing CI). The retained emitted file, if kept, shows a generic S whose declaration or a
    use-site type argument is NOT a bare `S`. Counters: S_AT_COUNT > 0 or UNRESOLVED_*_COUNT > 0 in
    that file — i.e. the mint id or an unresolved render is what changed at that line. If instead all
    three counters are 0, the mechanism is NOT my mint id and I must look elsewhere (record which).
P2. HEAD origin/main, kind=main: SELF_HOST_RC = 0, EMITTED_BUILD_FAILED_LINES=0. The red is
    attributable to my change, not pre-existing. If main is also red, this is not mine — stop and report.
P3. The gate itself: both runs print MEASURED_AT equal to the requested SHA and pass their GUARD line.
