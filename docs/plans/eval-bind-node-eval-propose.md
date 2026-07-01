# PROPOSE: `eval_bind_node_eval` exhaustive `RuntimeBehaviorInterpreter` match

**Branch:** `session/eval-bind-propose-655`  
**Escalation:** loyal-bee-794 (load-bearing `05_eval.dag`) — **do not merge without review**  
**Coordinate:** sunny-crab (interpreter layer)

## Change

Replace `match interpreter { BindRuntimeInterpreter {…} => … _ => reject }` with **six explicit arms** on closed coproduct `RuntimeBehaviorInterpreter` (`src/v2/std/runtime.dag:335-341`):

| Arm | Behavior |
|-----|----------|
| `BindRuntimeInterpreter` | Existing bind eval path (unchanged) |
| `ValueRuntimeInterpreter` | `outcome_rejected(eval_rejected_type_node)` |
| `TransformRuntimeInterpreter` | reject |
| `BranchRuntimeInterpreter` | reject |
| `LoopRuntimeInterpreter` | reject |
| `MatchRuntimeInterpreter` | reject |

Wildcard `_ =>` is **unwritable** — exhaustiveness by construction.

## Dissolution receipts (required before merge)

1. **Roster −1:** remove `src/v2/compiler/05_eval.dag::eval_bind_node_eval` from `NON_FOLD_MIGRATION_DEBT_ROSTER` + full roster (staged in PROPOSE commit).
2. **Syntactic witness:** `complexity_linearity_audit` no longer fires `syntactic_match_wildcard_arm` on `eval_bind_node_eval` (`eval-interpreter-debt` 6 → 5).
3. **Resolved census:** `non_fold_residue` live site count −1; `live_tree_residue_roster_has_no_stale_entries` green.
4. **Discriminating eval receipt:** fixture corpus eval unchanged on bind nodes; **RED** on a test that feeds `ValueRuntimeInterpreter` to `eval_bind_node_eval` and expects reject (proves non-Bind arms are not fail-open).

## Scope equivalence claim

Reject path uses the **same** `eval_rejected_type_node` diagnostic as the prior `_ =>` arm — behavior identical on all six variants; only the escape hatch is removed.
