# Root-4 Measure missing-generics closure receipt

This receipt measures one emitter root-family closure. It is not a Gate-1 closure: all seven canonical emitted crates still refuse cargo compilation.

## Change and authority

`render_rust_decl_type_container_arg` now consults one structural predicate before replacing a declaration container argument with its resolved overlay. The predicate admits an overlay only when the authored argument has no connective children, is not a closed alias realization, and has non-empty applied or resolved children. It does not inspect the name `Measure`.

The source authority is `src/v1/05_emit_rust.dag`; `src/v1/stage0/src/v1_compiler_emit_rust.rs` is its mechanically regenerated bootstrap projection. No `cli_run` transport, interpreter-deletion carrier, or deletion-plan authority changes in this slice.

## Identical parent/child protocol

The full rows and binary stamps are in `root4_measure_missing_generics_canonical_seven_2026-07-26.tsv`.

- Parent: `9a112a30975be5b8e0203b4d2216459572b3c818`, one release `gunbc` binary with SHA-256 `1380ec700e49d3378230831f0d84a8d0bd6a65c143cdc4a3c2143f2617a40df2`.
- Child probe tree: `41e5ce2b4678a5db2a6c39bfd85b9d3e9b512452`, one release `gunbc` binary with SHA-256 `0019ffa282960ad36283866626c7fed9c3ece90d1e3ae0b21ba699ec194691c0`.
- Both sweeps used the same `cssl_assemble` binary, `CSSL_STD_SEED_LINK=1`, one empty shim, `docs/probes/curated_cargo_probe_one.sh`, a fresh temporary crate per module, and retained full cargo logs.
- Every coded rustc error plus uncoded `unreachable pattern` and `UNRESOLVED_CompilerError` row is counted.

## Result

The target `measure_missing_generics` family is `8 -> 0` across the canonical seven and `4 -> 0` in `materialization_carriers`. Aggregate E0107 is `114 -> 82`. The closest module moves from `328 errors / 18 live mechanistic families` to `320 / 17`; the Measure family is the one family closed.

The two named downstream consumers move:

- `05_eval`: `431 -> 427`
- `materialization_carriers`: `328 -> 320`

Across all seven, total errors move `2920 -> 2892`. E0599, E0277, E0369, every other coded error, and both uncoded classes are flat. E0308 moves `839 -> 843`: two errors in `emit_host` and the same two in `materialization_carriers`, all at `std_realization_measurement.rs:91`. Repairing the line-90 `Measure` signature exposes the already-live Semiring/Measure representation mismatch in the fold initializer and `time_measure` argument. These four rows therefore do not introduce a distinct root family.

The same structural overlay also removes 24 non-Measure missing-generic errors as measured collateral. That broader family remains live (`56 -> 32` aggregate; `8 -> 2` in `materialization_carriers`) and is not claimed closed. Wrong generic arity remains `50 -> 50`.

## Discriminating fixtures

`dag/test/claim/root4_measure_missing_generics_witness_test.dag` contains:

1. A deliberately synthetic `List<Root4AppliedCarrier<ProbeQuantity, S, ProbeMagnitude>>` positive fixture that exercises the same structural generic-container shape without re-declaring the canonical `std.measure.Measure` authority.
2. A synthetic closed zero-arity alias control over `Root4AliasCarrier`, likewise avoiding a second unit model.
3. An unrelated `Root4ZeroArityCarrier` leaf control, proving a bare leaf without applied or resolved children stays unchanged.

With exact-parent `claim_batch` built from `9a112a3097` (SHA-256 `6000e1e1432d94945cddd5588c5c5cf761672b374bd7a455ee6435e1d98a7a9a`), the positive fixture fails while both controls pass. With the child binary, all three pass. The canonical `std.measure` closure independently compiles through the real `gunbc compile` transport with zero blocking errors and emits `Vec<Rc<Measure<(), S, Nat>>>`; its 46 `UnlistedImportUse` rows are advisory under the existing compile-clean policy and are not hidden by a `cli_run` change in this PR.
