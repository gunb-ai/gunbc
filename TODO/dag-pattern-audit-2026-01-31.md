# DAG Pattern Audit Findings

**Status**: Draft
**Date**: 2026-01-31

## Goal

Capture acute pattern issues found in the audit and document why they have not been detected so far.

## Design

### Findings (Acute)

1. **Upsert output mismatch**
   - Outer node exposes `was_created`, but the inner subdag has no boundary output named `was_created`.
   - Expected failure: `lower()` fails with `NoInnerBoundary` for `was_created`.

2. **Loop output mismatch**
   - Outer node exposes `iterations`, but the inner subdag never produces an `iterations` boundary output.
   - Expected failure: `lower()` fails with `NoInnerBoundary` for `iterations`.

3. **Loop API/doc drift**
   - Docs mention `with_element_output`, but no such method exists.
   - Loop body output type is always forced to the element input type, so element-type transforms are not representable.

4. **Repeat/While/Poll config is unused**
   - Retry policy + classifier never influence the DAG.
   - While `max_iterations` is stored but unused.
   - Poll `interval` and `timeout` are stored but unused.

5. **Pattern internals depend on `T::default()` with no contract**
   - Branch/Loop/Repeat inject controller/merge/unpack/pack nodes using `T::default()`.
   - There is no enforced semantic contract that the default op implements these internal behaviors.

6. **Pattern interface contracts are not validated**
   - SubDag interface matching (parent ports -> inner entrypoints/boundaries) is only enforced during lowering.
   - No builder-time or test-time validation for these contracts today.

7. **TransportOps::Execute missing skip handling** (fixed)
   - `TransportOps::Execute` unconditionally required a `request` input without checking the `skip` flag.
   - When `PrepareTestCommand` set `skip=true` (due to build failure) and omitted the optional `request`, `execute_test` crashed: `missing or invalid 'request' input`.
   - Root cause: the skip-aware port pattern (`request?` + `skip: Bool`) was wired in the CI graph but never implemented in `TransportOps::Execute`. The existing `CIGraphOp::CliTool` handled it correctly — `TransportOps` was missed.
   - Fixed in `lib/transport/src/ops.rs`: check `skip` before reading `request`, return early with `{skip: true}` when skipped.

### Root Cause for Non-Discovery

- **No lowering tests for patterns**: pattern unit tests only check node/edge counts and guards; they never call `lower()`.
- **Testgen is structural only**: it analyzes boundaries, edge types, and cardinality but does not validate SubDag interfaces.
- **Limited execution coverage**: patterns may not be exercised in real workflows yet, so lowering is not triggered.
- **Config-only fields**: Repeat/While/Poll settings are stored but never wired, so tests don’t observe them.
- **Doc/API drift**: docs can reference methods that were never implemented.
- **Skip pattern not tested end-to-end**: the `skip` port pattern was wired at graph level but no test exercised the `build_success=false` path through `execute_test`, so `TransportOps::Execute` was never called with a missing `request`.

## Tasks

- [x] Fix Upsert: add `was_created` boundary output (or remove it from the outer interface).
- [x] Fix Loop: add `iterations` boundary output (or remove it from the outer interface).
- [x] Decide and implement Loop element-output typing (add `with_element_output` or update docs and behavior).
- [ ] Wire Repeat/While/Poll configuration into internal ops, or remove unused fields.
- [ ] Define and enforce a contract for internal pattern ops (avoid implicit `T::default()` semantics or formalize it).
- [ ] Add SubDag interface validation in `gunbc-ir` and run it from builders/testgen/exec.
- [ ] Add pattern tests that run `lower()` (or equivalent validation) to catch interface mismatches early.

## Notes

- The current detection point is `lower()` in `gunbc-exec`, which is too late for design-time feedback.
- A small IR-level validator would make this consistent across testgen, builder usage, and exec.
- Loop element-output typing remains fixed to the element input type for now; docs were updated accordingly.
- Upsert no longer exposes `was_created`; the create node has no outputs.
