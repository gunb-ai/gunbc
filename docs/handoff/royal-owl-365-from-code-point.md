# Handoff: from_code_point removal (re-derives #13378/#13392), WIP

Branch royal-owl-365/from-code-point, base origin/main 0eefa98974. No PR yet.

## Decision (parent-approved, option A)
Delete the v1 seed builtin `from_code_point` and migrate every caller in one PR. The interpreter dispatches
builtins BEFORE module fns, so a same-named declaration is unreachable while the builtin exists. No rung drop.
Declared route: std.unicode.scalar `from_code_point` (Int -> Result<String, NotUnicodeScalar>) and total `char_text`
(Char -> String); list-of-scalars through std.coercion `unicode_scalar_fold`. No Err arm may return a fabricated value.

## State
- Source migration of all callers: done (48 changed dag modules compile with 0 NEW blocking errors against a seed built from this tree;
  yaml/ingest.dag:1329 Nat-vs-Int and stage0_rust_host_observation git `Inspect` errors are identical on the #13699 tree, pre-existing).
- Seed arm (v1_interpreter.rs free_call.from_code_point) and v1_rt::from_code_point deleted by hand; mirrors regenerated.
- `claim_executor --source-root dag --source-root src/v2 --required-regen` => RC 0, first_generation_equal=true (fixed point).
- claim_batch over all changed claim files: started (/tmp/fcp-claims.out on the lane container; rerun if lost).

## Next steps
1. Read claim_batch results; fix reds. Add a discriminating red that `\u{D800}` refuses on the EMITTED v2 route (src/v2/extdeps/languages/dag.dag
   uses `match from_code_point`) and on the interpreter route; or name the unexecuted path in the PR body.
2. Run `gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet`
   on the exact head; it must leave no diff (it also rewrites provisioning/srv1/gunbc-ghrunner.sudoers on this base: unrelated main drift, `git checkout` it).
3. Both instruments on srv1 under `systemd-run --user --scope -p MemoryMax=22G -p MemorySwapMax=0`:
   `gunbc test //gunbc/instruments:self-host` and `//gunbc/instruments:v2-native-cli`. Put exits in the PR body.
4. PR body: v1-maintenance purpose admission (serves the v2 self-host program); why std.unicode.scalar is a sibling of std.types (cycle: std.types
   imports it); the three disagreeing realizations of the old builtin; old PRs #13378/#13392 as history only; unexecuted paths.
5. Half 2 (git fixture over percent-coding) only after #13699 lands on main: merge main, add it to this same PR.

## Traps found
- Seed build needs lib.rs of stage0_std_core/emit_core WITHOUT the std_unicode_scalar `#[path]` mod until the mirror exists
  (bootstrap order: build seed, regen, then re-apply). The `src/v1/stage0/src/lib.rs` candidate must be taken from regen verbatim.
- Add std_unicode_scalar edges to src/v2/workflow/rust_crate_partition.dag, then regenerate dag/gunbc/stage0/stage0_crate_partition_generated.dag via main_wet,
  or required-regen refuses Stage0EmittedEdgesNotCovered.
- Delete gitignored dag/gunbc/{recurring_failure_mode,rung_drop}/roster.dag before every run; rebuild the seed after the last edit (StaleBinary).
- The seed does not enforce Char's refinement at args/fields: Char is shape, not proof.
