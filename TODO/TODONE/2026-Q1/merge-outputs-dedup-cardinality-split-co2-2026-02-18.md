# CO2 Completion: MergeOutputs Dedup/Cardinality Split

Date: 2026-02-18
Task: `CO2`

## Change Summary

File: `lib/review/src/lib.rs`

Refactored `ReviewOps::MergeOutputs` execution path so responsibilities are separate:

- Cardinality/input decoding:
  - `collect_merge_outputs(inputs: &HashMap<String, Value>) -> Result<Vec<ReviewOutput>, ExecError>`
  - `decode_review_output_list(items: &[Value]) -> Result<Vec<ReviewOutput>, ExecError>`
- Dedup/conflict logic:
  - `dedup_findings_with_conflicts(outputs: &[ReviewOutput]) -> (Vec<Finding>, Vec<serde_json::Value>)`

`execute_merge_outputs` is now orchestration-only:
1. decode outputs with cardinality handling
2. dedup findings and collect conflicts
3. emit `bundle` + `conflicts`

Behavior and external interface were preserved.

## Tests Added

In `lib/review/src/lib.rs` test module:

- `test_collect_merge_outputs_rejects_non_list`
- `test_dedup_findings_with_conflicts_keeps_first`

## Validation

- `cargo test -p gunbc-lib-review merge_outputs -- --nocapture`
- `cargo test -p gunbc-lib-review dedup_findings_with_conflicts_keeps_first -- --nocapture`
