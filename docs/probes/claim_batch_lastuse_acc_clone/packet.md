# Frozen claim_batch packet — last-use by-value-argument probe (PR #15988)

STATUS: DRAFT for byte review (manager clever-crane-830) then keen-otter grant. Not executed by the author; the required lanes green at this head evaluate NO witness, so this packet is the executed receipt the probe needs.

## Subject

- Repo/PR: gunb-ai/gunbc PR #15988, branch `session/crisp-bee-97-lastuse-acc-clone`.
- The packet freezes the two claim files by content, not by head:
  - `dag/test/claim/emit_last_use_acc_clone_control.dag` — git blob `dd7e40887acea8a6a8aaddc73a7b7a7803b21f89`, sha256 `ad3e51baf0ed21b42e1ba6e7b2ca3fed90d3d49cd7724de73464146caa4e46c1`
  - `dag/test/claim/emit_last_use_acc_clone_fixtures.dag` — git blob `94406f1a9ab589f7b045e5cd6859cdf97e0d55ef`, sha256 `0de3c1bd9311bfbe0855df0c0512c50f6f66e71851c833f09e5e92795a92d155`
- Run head: the PR head whose tree carries those exact bytes (`ceaa130c5c52f30c01418156c0d541a045488fb8` at drafting; a later head is valid only if both blobs are unchanged). Verify before eval: `git rev-parse HEAD:dag/test/claim/emit_last_use_acc_clone_control.dag` equals the pinned blob.
- The probe reads the EMITTER COMPILED INTO THE BINARY IN USE (the stage0 mirror). At a heal-green head the mirror is seed-converged, which is what makes the reading attributable to the emitter that lands.

## Build (one dispatch, release, fleet build configuration)

From a clean checkout of the run head:

```
cargo build -p v1-compiler --release \
  --features text_lookup_work_counter,interp_test_witness,test_hooks \
  --bin claim_batch
```

(same bin/feature configuration as the fleet release-bins roster in fleet-converge.yml; record `target/release/claim_batch --help`-absent: record the build log and `sha256sum target/release/claim_batch` in the receipt.)

## Command (one end-to-end run, hermetic default, one finalization path)

```
target/release/claim_batch \
  --source-root dag --source-root src/v1 --source-root src/v2 \
  --hermetic \
  --entry dag/test/claim/emit_last_use_acc_clone_control.dag \
  --functions the_move_module_emits_holds,the_second_use_module_emits_holds,the_vec_module_emits_holds,the_capture_module_emits_holds,the_branch_module_emits_holds,the_shadow_module_emits_holds,the_last_use_by_value_parameter_emits_bare_holds,the_second_use_keeps_its_clone_holds,the_last_use_vec_accumulator_emits_bare_holds,the_captured_parameter_keeps_its_clone_holds,the_both_arms_parameter_keeps_its_clone_holds,the_shadowed_parameter_keeps_its_clone_holds \
  --outcomes-out <workdir>/lastuse_acc_clone_outcomes.json
```

- 12 witnesses, one entry group, resolve-once — the run goes end to end through the pre-fetch phases the bin performs in-process (roster → resident shared index → whole-tree published-mock precompute → warm edge index → single resolve → per-witness eval) and finalizes through the ONE path: `--outcomes-out` renders `v2.workflow.floor_terminal_ledger` from the outcomes (`render_ledger_from_outcomes`). No `--print-entry-closure`, no splits, no `--wet`/`--record`, no env overrides (optionally `GUNBC_FLOOR_PHASE_PROFILE=1` for diagnostics only).
- Exit composition: **0** every witness Pass · **1** any finding (Fail / NotBool / RuntimeError / TimedOut) · **2** no observations (unresolved entry group, no witnesses).

## Expected cells — the probe's contract

Each behaviour cell travels with its MARKER-ONLY pair; a red behaviour cell beside a TRUE marker is **emitted-but-different**, the only red the probe claims. A red marker cell, or a cell whose outcome is NotBool/RuntimeError/TimedOut, is a REFUSAL reading: report the exact outcome text, do not read it as evidence about the move.

| # | cell | expected at this head | meaning of that reading |
|---|---|---|---|
| 1 | `the_move_module_emits_holds` | Pass | emission happened; #7 is text-comparison, not refusal |
| 2 | `the_second_use_module_emits_holds` | Pass | marker |
| 3 | `the_vec_module_emits_holds` | Pass | marker |
| 4 | `the_capture_module_emits_holds` | Pass | marker |
| 5 | `the_branch_module_emits_holds` | Pass | marker |
| 6 | `the_shadow_module_emits_holds` | Pass | marker |
| 7 | `the_last_use_by_value_parameter_emits_bare_holds` | **Fail — the probe's finding** | the motivating shape (String accumulator, generic declared callee) still clones at its sole use |
| 8 | `the_second_use_keeps_its_clone_holds` | Pass | non-last use keeps its clone (frozen boundary holds) |
| 9 | `the_last_use_vec_accumulator_emits_bare_holds` | **Fail — the probe's finding** | the non-Copy accumulator shape also clones |
| 10 | `the_captured_parameter_keeps_its_clone_holds` | Pass | capture keeps its clone |
| 11 | `the_both_arms_parameter_keeps_its_clone_holds` | Pass | two-arm use keeps its clone |
| 12 | `the_shadowed_parameter_keeps_its_clone_holds` | Pass | shadowed parameter keeps its clone (falsifies a repair that moves the shadow's occurrence) |

**Expected exit: 1** — findings confined to cells #7 and #9. Readings and consequences:
- exit 1 with exactly #7+#9 Fail (markers Pass): probe CONFIRMS the frozen pattern should move and does not; the repair is written against the one existing route (`user_call_lambda_emit_info` → `build_moved_argument_occurrences` → `emit_rust_expr_var`), suspect localized from the emitted text in the finding (suspects 2/3 stand; suspect 1 already structurally excluded — `rust_call_arg_function_value_adapt` is a pure string post-processor).
- **exit 0 (all 12 Pass, move cells green)**: NOT read as stale mirror and NOT as "the move already happens on main" without more — it first means a fixture-shape miss; the next packet extends the fixture toward the real-site shape further (emitted `impl Fn` callable wrap on fn-typed parameters, instantiation depth) before any conclusion.
- exit 2, or any marker red, or NotBool/RuntimeError anywhere: refusal channel — the exact outcome text is reported and front-end gaps routed per the standing rule; no repair is written from it.

## Receipts wanted back

Build log tail + binary sha256; the run's stderr phase lines (`claim_batch: N entry group(s), 12 witness(es) total`, `resolving ... once`, per-witness lines) as printed; the `--outcomes-out` JSON (or its 12 per-witness rows); final exit code; the checkout line proving the run head and the two pinned blob hashes.

— drafted by crisp-bee-97 at head `ceaa130c5c52f30c01418156c0d541a045488fb8`
