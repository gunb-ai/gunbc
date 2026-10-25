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

Before (interpreted, the displaced cost -- one dispatch per module; node.dag measured 1197 s by
the dispatch's own `==NODE_T0==`/`==NODE_T1==` pair, infer.dag never completed inside the runner's
hard 1 h cap -- see Executed results):

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

## Executed results (remote runner, per-module dispatches, 2026-10-25)

BEFORE (the displaced cost, named by its instrument -- the interpreted oracle dispatch's wall):
`==NODE_T0=1792899699== ... ==NODE_T1=1792900896==` = **1197 s** for one interpreted pass over
`src/v2/std/node.dag` (138,516 B), from the `run_case` dispatch around the
`roster_sweep_oracle_wire` call. For `src/v2/compiler/infer.dag` (585,795 B) the interpreted pass
could not be completed inside the runner's hard 1 h cap at all: three dispatches carrying the
`==INFER_T0==`/`==INFERONLY_T0==` timer (started 1792901399, 1792913545, 1792919886) were each
killed past 50 min with no rows printed. The consumer's
second subject therefore has NO committed expected file and is listed only in the manifest
comment below -- a missing oracle must not become a silent vacuous case.

AFTER (the native route's own instrument): the baseline `gunbc test` run did not reach the child
-- the emitted crate's cargo build refuses with the exact emission refusal recorded in
`dag/gunbc/instruments/roster_import_sweep_native.dag`'s header:
`V2-NATIVE REFUSAL cause=EmittedCompilerBuildFailed first_error=error[E0204]: the trait `Copy`
cannot be implemented for this type @ .../std_bit.rs:40:12` (Word64 leaf collision between
`dag/std/bit.dag` and `dag/std/machine_word.dag`; the documented still-leaf-keyed `shared_types`
RFM `emitter_index_keyed_on_a_type_leaf_takes_the_last_declarer`). So `child_wall_ms` rows print
only after that emitter gap closes; until then the native cost instrument is the REFUSAL line
itself, and the route is proven WIRED (the seed rendered the crate and cargo ran it) rather than
executed. The refusal was reproduced three consecutive dispatches (baseline, poison,
no-manifest -- all exit 2 with the same first_error), and it is NOT papered over: the poison and
manifest-absence controls cannot report through it, which is why the control witnesses below carry
the binding assertions instead.

GREEN controls, executed: all twelve `test fn`s of
`dag/test/claim/roster_import_sweep_native_control_witness.dag` HELD under `gunbc test
//test/claim/...:all` (WITNESS_RC=0, cpu_ms<=2 per row), and the Rust control passed on the
remote (`roster_import_sweep_native_is_bound_to_the_native_claim_program_producer ... ok`;
`test result: ok. 1 passed`). The sibling native instrument `//gunbc/instruments:value-position-
block-nests` exited 0 on the same dispatch, proving the producer route itself is live; the other
sibling `native-materialization-store-closure` is red on main at the SAME emitted line (pre-existing,
"run by name, not a merge gate"), which is what discriminates the collision: its closure carries
both Word64 declarers, VBP's carries only std.bit.
