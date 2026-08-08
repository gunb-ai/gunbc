# CI2-0 re-observation: batch-5 native bundle standing on main (2026-08-08)

Re-observed against origin/main `6f4bc24444` before any change, per the CI2-0
brief's first law (re-observe starting standing; never copy it from the brief).
The brief's "0/3-native figure is retrospective-era" expectation was **refuted by
measurement**: 0/3 is the *current* standing, on every observed run.

## What is enrolled

- `gunbc_pr_native_batch()` (`v2.workflow.ci_floor_plan`): one
  `NativeBundleWitnessKind` row →
  `src/v2/test/claim/execution/native_selected_witness_bundle_production.dag`
  :: `native_selected_logic_production_spec`; population = 3 members
  (meet / join / complement), bundle identity `09d7d2e53554c783`.
- Executed by `claim_executor` `run_native_bundle_unit`
  (`BatchUnit::NativeBundle`): cold native leg → warm native leg → planted-red
  native leg → interpreter oracle → `native_transition_decision`.

## Measured standing (GitHub Actions run logs, `[native-selected-bundle]` receipts)

Every observed main-branch floor run from the first enrolled run
(30774223741, 2026-08-03T00:19Z, the #7671 merge push) through 31233572432
(2026-08-08T01:50Z) reports the identical receipt:

```
selected_witness_count 3  native_count 0  interpreted_count 3  unavailable_count 3
fallback_count 3  verdict fallback:native_realization_refused
planted_red_equivalent false  cold_compile_wall_nanos ~70-77ms
transport causes: cold: process failed | warm: primary cold artifact unavailable
                  | planted-native: process failed
```

Sampled runs: 30774223741 · 30776356578 · 30778265303 · 30863019228 ·
30961317834 · 31058002966 · 31131446355 · 31156588100 · 31201159791 ·
31213648746 · 31219934483 · 31233572432. No run shows `accepted`.

**Native execution has never been authoritative on main.** The pre-merge
"outage" that motivated the counted-fallback arm (run 30764923923, review
47508's srv4-03 reading) shows the same signature — same workspace key
`2bc05a600ce0e4a0`, `compile_skipped=false`, ~133ms cold process-fail — so it
was never a transient toolchain outage: it is a deterministic cold-build failure
on every runner, absorbed since 2026-08-03 by the fallback arm, whose zeroed
failure frequency is exactly DESIGN §5's absorbing-fallback prediction.

## Located defects (each fixed by this capsule or the flip capsule)

1. **Cause dropped on the wire.** The transport result carries `stderr_octets`
   and `build_log`, but `claim_executor::native_transport_observation` read only
   success/stdout/nanos — every non-accepted verdict was located only as
   "process failed". Fixed here: bounded stderr tail + build log rendered into
   the transport causes (stderr/detail), never into the TSV receipt.
2. **Job-scoped env asymmetry.** `GUNBC_NATIVE_CACHE_ROOT` is derived in the
   `build` job (and falsifier), but `GITHUB_ENV` is job-scoped and the floor
   (`ci`) job never derived it — the bundle workspace fell back to the shared
   `/tmp/gunbc_…` scratch prefix instead of the toolchain-segmented durable
   root. Fixed here: `ci_floor_job_prelude_steps` carries
   `ci_native_cache_root_step()` (authority `gunbc.ci_workflow`; ci.yml and
   falsifier.yml regenerated).
3. **The emitted crate itself is healthy**: reproduced byte-for-byte locally
   from `native_selected_logic_production_spec` and `cargo build` succeeds
   (warnings only), so the failure is environmental to the floor job, not a
   defect in the emitted bundle.

## The cutover happens in this same PR (operator recut, 2026-08-08)

The diagnosis above is the first commit of CI-0, not a separately mergeable
precursor. In the same PR: the counted interpreter fallback is deleted — the
`NativeProductionTransitionFallback` verdict arm loses its constructor
(`std.selected_witness_bundle` `native_production_transition_no_fallback_note`)
and the executor's decision reduces to accepted-or-refused with
interpreted = fallback = 0 by construction. Native unavailability is a typed,
located refusal carrying build log + stderr. The build failure named by the
surfaced stderr (`error: unnecessary parentheses around match scrutinee
expression` under a deny-warnings build env) is fixed at its root: the emitted
crate's manifest declares its lint envelope
(`rust.dag` `rust_emit_host_lint_envelope_note`), because the one match shape
must keep scrutinee parens for the struct-literal form that requires them.
Receipt/status homes unchanged: `std.selected_witness_bundle`
`NativeWitnessBundleExecutionReceipt.member_equivalence`,
`gunbc.native_witness_transition_receipt`.
