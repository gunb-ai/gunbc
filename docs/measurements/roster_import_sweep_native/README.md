# Roster import sweep, native route

Measurement corpus for `gunbc test //gunbc/instruments:roster-import-sweep-native`: the
`roster_import_sweep_for_paths` sweep (the generator entry `v2.workflow.roster_import_sweep` runs
today only through the v1 interpreter) is emitted by the seed, built with cargo and EXECUTED as a
NativeClaimDriver program -- the same `gunbc.target_binding` NativeClaimProgramProducer route
`gunbc test` uses for instrument labels (DESIGN section 7). The interpreted pass below is the
OFFLINE DIFFERENTIAL ORACLE only; the native route never calls the interpreter.

## What is here

    manifest.txt              the work list the child reads (rows `id|paths-csv`; `#` and blank
                              lines skipped) -- the child spawns with no arguments and no
                              environment, so this fixed path is its only subject channel
    expected/<id>.txt         the sweep's edit lines for one case, committed ONCE from the
                              interpreter (the oracle), compared BYTE-IDENTICALLY by the child
    README.md                 this file

The seeded RED is `refuse_escaped_path`: its expected file IS the `REFUSE ../outside_repository.dag
...` row the admission prints, so a route that ever let it pass goes not-held by itself.

## Measuring the before/after, by the instrument itself

Before (interpreted, the displaced cost -- one dispatch per module; node.dag measured ~16m,
infer.dag ~50m at sunny-bee-153's measurement, past the runner's 1h cap):

    gunbc run --source-root dag --source-root src/v2 \
      --entry dag/gunbc/instruments/roster_import_sweep_native.dag \
      --function gunbc.instruments.roster_import_sweep_native.roster_sweep_oracle_wire \
      --arg "paths_csv=src/v2/std/node.dag"

After (native): the instrument's own rows print `child_wall_ms=<n>` per case, measured between the
two `observed_monotonic_nanos` readings around the sweep call, and the host plan line prints the
child's wall and the binding's deadline. The before/after pair is named by these instruments --
the interpreter dispatch's wall and the native row's `child_wall_ms` -- not transcribed from this
README.

    gunbc test //gunbc/instruments:roster-import-sweep-native

The controls that go red when the route is removed or falls back are
`dag/test/claim/roster_import_sweep_native_control_witness.dag` (the binding row must name
NativeClaimProgramProducer) and the Rust test
`roster_import_sweep_native_is_bound_to_the_native_claim_program_producer` in
`src/v1/stage0/src/target_invocation_host.rs`.
