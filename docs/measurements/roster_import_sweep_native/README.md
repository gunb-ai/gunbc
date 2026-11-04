# Roster import sweep, native route

**Goal:** measure the sweep through `gunbc test //gunbc/instruments:roster-import-sweep-native`, using `NativeClaimProgramProducer`. The interpreter is only the offline differential oracle.

**Corpus:** `manifest.txt` supplies `id|paths-csv` rows to an argument-free, environment-free child; comments and blank lines are skipped. `expected/<id>.txt` is the byte-exact oracle output. `refuse_escaped_path` must retain its refusal. The `infer.dag` oracle never completed within the runner limit and has no expected file; its commented manifest row must not become a vacuous case.

**Measure:** the native instrument owns `child_wall_ms`; the oracle entry is `gunbc.instruments.roster_import_sweep_native.roster_sweep_oracle_wire(paths_csv)`. Compare completed observations, not wiring or emitted-crate construction. Binding controls are `test.claim.roster_import_sweep_native_control_witness` and `roster_import_sweep_native_is_bound_to_the_native_claim_program_producer` in `src/v1/stage0/src/target_invocation_host.rs`.

**Recorded boundary, 2026-10-25:** the interpreted node sweep completed; the infer sweep timed out. Native execution first refused on the Word64 same-leaf `Copy` collision (E0204). Declaration-keyed derives removed that refusal, exposing E0282 in the emitted fold accumulator and E0425 for `start_detached`/`reap`. Both native dispatches remained exit 2, with no child timing. Binding controls and the sibling value-position instrument passed; neither proves this sweep executed. The same-runner base/head mutation of `shared_roster_follows_the_declaration` discriminated the derive repair.

The [original execution account](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/measurements/roster_import_sweep_native/README.md) retains dispatch readings. Re-run the instrument to establish current standing; this record is not a performance result or a current merge gate.
