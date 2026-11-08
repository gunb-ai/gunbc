## CORRECTION to the walk-cost model record (6707290503) - hierarchy and labels

Append-only; 6707290503 is left unedited. Raised by keen-otter (relayed by silent-bee-845, who took the missed arithmetic check on themselves). Nothing was compiled, claimed, tested, regened, run, dispatched, pushed or landed by this session; every figure below is a re-reading of the retained log of main run 42292182286.

### 1. What I got wrong

In section 4 of 6707290503 I decomposed each emit step as "compile transaction + cargo + post-build controls", and described `process_unattributed` as the post-build part. That is wrong in two ways:
- **The two rows are exhaustive, not additive with cargo.** For each step, transaction + process floor IS the process total, exactly: self-host 1,218,660 + 432,804 = 1,651,464 ms; v2-native-cli 1,349,729 + 586,498 = 1,936,227 ms (each log line reports `transactions=1`). So cargo's ~4m39 (self-host) and ~4m24 (native-cli) cannot be extra components; they sit INSIDE the process floor.
- **The floor is not "post-build".** `process_unattributed` spans the whole interval between the two sampling instants that the transaction did not occupy, so it CONTAINS the cargo build as well as the door/refusal controls, fault/restore/spawn and the calibration suite. Calling it "post-build" understated what is unattributed and invented a partition the receipts do not give.

### 2. The hierarchy as the receipts give it, with labels

Verbatim rows, self-host step (subject `src/v2/compiler/source_root_eval.dag`):
- `[pre-entry] phase=process_invocation_total ms=1651464 scale=closure attribution=inclusive transactions=1 boundary=main_entry_to_pre_exit_sampling` — INCLUSIVE, spanning main entry to the sample taken at exit-sampling.
- `[pre-entry] phase=process_unattributed ms=432804 scale=closure attribution=floor boundary=main_entry_to_pre_exit_sampling` — EXCLUSIVE remainder of that same span: total minus the transactions. 432,804 ms = 7m13s.
- `[pre-entry] phase=compile_transaction ms=1218660 scale=closure attribution=inclusive subject=src/v2/compiler/source_root_eval.dag` — INCLUSIVE, 20m19s.
- `[pre-entry] phase=compile_transaction_unattributed ms=1196151 scale=closure attribution=floor raw_ms=27278` — EXCLUSIVE remainder of the transaction: 98.15% of it.
v2-native-cli (subject `src/v2/cli/compile_cli.dag`): total 1,936,227 ms INCLUSIVE (32m16s), floor 586,498 ms EXCLUSIVE (9m46s), transaction 1,349,729 ms INCLUSIVE (22m30s), its floor 1,326,813 ms EXCLUSIVE = 98.30%, `raw_ms=27407`.

Nested one level down, and NOT additive with the transaction total — the receipts carry the hierarchy in `parent=` and `depth=`:
- `phase=demand_layout_acquire ms=22509 scale=closure attribution=inclusive parent=none depth=0` (native-cli: 22,915)
- `phase=pool_source_tokenize ms=14295 ... attribution=recorded parent=demand_layout_acquire depth=1` (native-cli: 14,270), and the other `pool_*` rows likewise `depth=1` under the same parent; `phase=selection_tier_closure_files ms=8115 ... attribution=recorded parent=none depth=0` (native-cli: 8,110).
The transaction's floor is total minus what the caller's attributed accumulator explains: 1,218,660 - 1,196,151 = 22,509 ms, matching that single depth-0 inclusive child. `raw_ms` (27,278 / 27,407) is a different quantity - the raw sum of the recorded rows reported alongside, which nest inside that child - so `raw_ms`, the floor and the transaction total must not be added together. Where exactly the `raw_ms` rows sit relative to the attributed accumulator is not something these lines settle; I am not claiming more than the labels give.

Cargo's containment IS established by the receipts, as containment rather than as a component: the self-host route line `emitted 266 files ... cargo build` is timestamped 21:26:04.9 and the built crate returns 21:30:44.0, both inside the floor interval 21:26:03.5 -> 21:33:16.2; for native-cli 21:55:47.6 -> 22:00:13.1, inside 21:55:46.1 -> 22:05:32.6.

### 3. Retractions

- RETRACTED: "cutting the emit cost means reversing the fresh-product policy". The `GUNBC_NATIVE_PRODUCT_REUSE=fresh` policy forecloses one specific mechanism (a stored product standing in for the build), not the whole cost. It does NOT rule out a semantics-preserving optimization inside an independently fresh compile - fewer redundant traversals, better data layout, whatever measurement would reveal.
- RETRACTED: "headroom must therefore come from the parse-population side". That was a false dichotomy built on the first retraction.
- No such optimization is established. The gap is ATTRIBUTION: about 98% of each transaction lies outside every recorded phase, so today there is no basis to claim any emit-side saving.
- Change 2's savings remain UNMEASURED, and they stay unmeasured: no instrumented run is authorized, and I have not requested one beyond noting earlier that attribution is what is missing.

### 4. What stands from 6707290503, unchanged

The 12 instrumentation call sites; the corrected emitter citation `src/v1/stage0/src/cli_run/pre_entry_phase.rs:302`/`:309` (transaction and its floor) and `:368`/`:378` (process total and its floor), boundary documentation at `:201-221`, and the correction of my own earlier reference to a nonexistent `compile_telemetry.dag`; the parse-side joins (the hit for `49d4bc36` with `reused=10101`, `miss_digest_unseen=4`, about 7 s; the full-walk miss in 42339827972 with `reused=0 miss_identity_mismatch=10094 miss_entry_absent=0` after 22m46s); the identity-era finding, which is unaffected by this correction because it rests on the reuse counters, not on these timings.

keen-otter rates final Change 1 (`e3ba345342f0046711b904adc71ab2964de3a3d2`, tree `8dedb58c302b982654b5c6f4fbd0acdb31dd879a`) independently SOURCE READY. Execution, generation, push and LAND remain withheld, and remain so as far as this session is concerned: nothing has been pushed, there is no PR, and the working tree is clean at that commit.
