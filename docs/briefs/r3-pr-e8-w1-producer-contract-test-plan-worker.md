# PR-E E8 W1 - Producer Contract Test-Plan Worker

**Status:** READINESS PACKET - docs/test-plan only. This packet turns the
existing W1 blocker into later implementation fire criteria for
`DifferentialEquals(rust_emit_output, dag_eval_output, ProgramOutputBind)`.
It does not edit `test_runner.rs`, fixtures, substrate declarations, runner
predicates, target enumeration, stdout parsing, or evaluator behavior.

**Parent authority:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
E8, [`r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md)
W1, [`r3-pr-e8-runner-extensions-continuation-readiness.md`](r3-pr-e8-runner-extensions-continuation-readiness.md),
and [`r3-pr-e8-w1-output-producer-contract-blocker.md`](r3-pr-e8-w1-output-producer-contract-blocker.md).

**Post-E3 boundary:** [`../audit/r3-evaluator-phase5-post-e3-closure-handoff.md`](../audit/r3-evaluator-phase5-post-e3-closure-handoff.md)
keeps #1857 scoped to the E6-G1.a Option 3 mechanism. Do not use #1857 as
runner, E8, Q-Reification, or lens-over-`Dag` authority.

## Current-State Grep Receipt

Current `main` has already consumed the #1485 blocker through the narrow #1499
W1 path. The test-plan baseline is therefore "keep the landed slice narrow and
prepare its replacement path," not "fire the first W1 implementation."

| Surface | Current evidence | Readiness consequence |
|---|---|---|
| `DifferentialEquals` dispatch | `src/v3/compiler/src/test_runner.rs` `eval_differential_equals` handles the legacy cost pair `v3_program_cost` / `v2_oracle_cost` and the W1 pair `rust_emit_output` / `dag_eval_output`; unsupported pairs return `NotYetImplemented`. | Later work must preserve unsupported-pair fail-closed behavior and must not broaden W1 by accepting new producer names incidentally. |
| W1 producer identity | `w1_rust_emit_output_int`, `w1_dag_eval_output_int`, and `w1_differential_equals_lineage_int` dispatch by the exact lineage names `rust_emit_output` and `dag_eval_output`. | This is the approved transitional DeclarationRef-name contract from #1485 / #1499, not durable substrate producer authority. |
| Fixture shape | `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag` declares `fn rust_emit_output(...) -> Lookup<Int>`, `fn dag_eval_output(...) -> Lookup<Int>`, three `ProgramOutputBind` inputs, and three `DifferentialEquals` rows. Comments state the fixture stubs are not executed; the runner implements both sides. | The fixture is evidence for the current Rust / Int slice. Do not rewrite the stubs or treat them as semantic producer bodies. |
| Program observation carrier | `src/v3/std/runtime.dag` declares `ProgramObservation<Carrier> { observed: Carrier }` and comments that PR-B.2 W1/W3 should land the first declared consumer/producer path. | The carrier is producer-neutral. It still lacks producer lineage, target language, channel, exit-status, and parse policy. |
| Rust observation path | `w1_rust_emit_output_int` emits Rust, compiles with `rustc`, executes the binary, and parses one Int token via the W1 stdout carve-out. `emit/rust_target.rs` exposes the shared program-mode bind selector used by W1. | This is an Int-only stdout parse carve-out. It must dissolve into PB-Runtime generated target-language tests plus typed observation-channel authority. |
| DAG evaluation path | `w1_dag_eval_output_int` evaluates the producer node for the named top-level output bind through `evaluate_body` with `EvalStrategy::ApplicativeOrder { input_order: InputEvaluationOrder::LeftFirst }`, then accepts only `Value::LiteralValue(Int)`. | This is a no-memo eager evaluator use of the current body evaluator spine. It does not claim full memo, strategy, witness, or value-domain completeness. |
| Verification consumers | `src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs` expects three W1 L4 claims to pass and a mixed `rust_emit_output` / `v3_program_cost` pairing to stay deferred. | L4 may consume this narrow surface. L5 / W3 still needs typed structural observation and target capability authority. |

## Producer Identity Contract Options

Later runner-code work must choose one of these contracts explicitly.

### Producer Option A - transitional DeclarationRef-name contract

Accept only the exact DeclarationRef spellings `rust_emit_output` and
`dag_eval_output` at the `DifferentialEquals` subject/oracle sites.

Acceptance criteria:

1. The runner rejects every other subject/oracle producer pair fail-closed,
   including mixed W1/cost pairings.
2. The implementation comment names this packet, #1485, and #1499 as
   transitional authority.
3. The PR body names the dissolution target: producer-role markers or an
   equivalent typed producer contract replace name-keyed recognition.
4. Tests include both a passing supported W1 pair and an unsupported-pair
   `NotYetImplemented` control.

Rejection criteria:

- accepting aliases, target-prefixed names, fixture-local helper names, or
  producer declarations discovered by fuzzy name matching;
- treating the fixture function body as executable producer authority;
- adding a fixture-local producer enum or a new `TestPredicate` variant to
  avoid the existing `DifferentialEquals` envelope.

### Producer Option B - substrate producer role / marker contract

Add or consume a P1-owned producer-role surface so the runner selects
`rust_emit_output` and `dag_eval_output` by typed declaration role rather than
spelling.

Acceptance criteria:

1. Producer roles identify at least producer lineage and value domain.
2. The runner reads the role structurally from the compiled or fixture `Dag`.
3. Unknown roles and role/value-domain mismatches fail closed.
4. The contract keeps the existing `DifferentialEquals` and `ProgramOutputBind`
   surfaces.

Rejection criteria:

- a runner-local registry that mirrors substrate roles in Rust only;
- broad target enumeration bundled into W1;
- producer identity inferred from source text, comments, output bind names, or
  target file paths.

## Observation Channel Contract Options

Later runner-code work must also choose one observation-channel contract.

### Observation Option A - typed observation-channel carrier

Consume a P1-owned carrier that describes observation channel and expected
value kind before normalization into `ProgramObservation<Value>`.

Acceptance criteria:

1. The carrier distinguishes stdout, stderr, file, or evaluator-return channel
   without parsing by convention.
2. The carrier names the expected value kind, starting with Int if the slice
   stays W1-compatible.
3. The runner normalizes output into the shared structural value comparator
   path rather than a W1-only equality helper.
4. Nonzero exit status, channel absence, parse failure, and value-kind mismatch
   remain typed runner failures.

### Observation Option B - scoped Int-only stdout parse carve-out

Keep the #1499 carve-out for the existing Rust / Int slice only: capture
emitted Rust stdout, require exactly one trimmed integer token, and compare it
to `dag_eval_output`'s `Value::LiteralValue(Int)`.

Acceptance criteria:

1. The PR states this is W1 slice debt, not a general observation policy.
2. The fixture row has one named Int output bind and no target-language side
   effects.
3. The runner bounds stdout/stderr capture and fails closed on timeout,
   truncation, invalid UTF-8 policy violation, non-Int output, or extra tokens.
4. The dissolution target is PB-Runtime generated target-language tests plus a
   typed observation-channel/value-kind surface.

Rejection criteria:

- comparing raw stdout bytes as semantic equality;
- accepting Bool, list, record, Diagnostic, or multi-output observations under
  the Int carve-out;
- extending the carve-out to Python, Go, `ForAllTargets`, or L5 strict
  structural observation.

## `dag_eval_output` Evaluator Dependency

The current W1 oracle shape is exactly:

1. Compile the `TestClaim.source` program to a `Dag`.
2. Resolve `ProgramOutputBind.output_ref` to a top-level value bind name.
3. Find the bind's producer node in the compiled program `Dag`.
4. Call `evaluate_body(program_dag, entry, &mut EvalStateStack::with_root_frame(...), EvalStrategy::ApplicativeOrder { input_order: InputEvaluationOrder::LeftFirst })`.
5. Accept only `Value::LiteralValue(Int)` for W1 slice parity.

This is sufficient for the existing branch-only / Int L4 receipts. It does not
claim:

- full memoization completeness;
- lazy or alternate strategy completeness;
- Callable / fold / effect completeness;
- witness construction completeness;
- non-Int value-domain observation.

Any later implementation slice that needs those surfaces must either cite the
landing PR that made them live or STOP+PING with the missing evaluator symbol
and fixture shape.

## Implementation Fire Criteria

A later runner-code slice may fire only if all criteria below are true:

1. Producer identity is explicit: either Producer Option A's exact transitional
   names are still the authorized scope, or Producer Option B's substrate
   producer role has landed.
2. Observation channel is explicit: either Observation Option B's Int-only
   stdout carve-out remains scoped to Rust / Int W1, or Observation Option A's
   typed observation carrier has landed.
3. `dag_eval_output` uses the real eager evaluator entry named above and
   documents no-memo `ApplicativeOrder` / `LeftFirst` at the call site.
4. `rust_emit_output` uses the same program-mode output-bind selector as Rust
   emission; it does not rederive the printed bind from source spans or claim
   text.
5. Unsupported producer pairs, unsupported value shapes, Rust compile/run
   errors, stdout parse failures, evaluator errors, and bind-resolution
   failures all fail closed.
6. The PR body includes grep receipts for `eval_differential_equals`,
   `w1_rust_emit_output_int`, `w1_dag_eval_output_int`,
   `ProgramObservation<Carrier>`, and the W1 fixture rows it consumes.
7. Tests cover one passing W1 pair and at least one unsupported producer-pair
   control that remains `NotYetImplemented`.

## STOP+PING Conditions

Stop and ping the Evaluator Manager instead of editing runner code if the next
slice requires any of these:

- a new `TestPredicate`, `Value`, producer enum, target enum, or observation
  carrier outside a P1 substrate proposal;
- broadening W1 beyond Rust / Int / one named output bind before typed
  observation authority lands;
- treating #1857 as runner, E8, Q-Reification, or lens-over-`Dag` authority;
- using `lens_apply.rs`, reflected-program folding, or `ReflectedProgram<T>` to
  justify W1;
- executing fixture `rust_emit_output` / `dag_eval_output` stub bodies as
  producer semantics;
- comparing emitted source, raw stdout bytes, diagnostics, or exit code as
  semantic equality;
- adding target enumeration or `ForAllTargets` behavior under W1;
- requiring evaluator Callable, fold, effect, lazy strategy, memo, or witness
  behavior that is not live on `main`;
- weakening the mixed-lineage unsupported-pair ratchet.

## Verification Coordination

This packet is W1 readiness only.

- **Lane 1 / L4:** may consume the existing #1499 Rust / Int W1 surface by
  adding narrow `DifferentialEquals(rust_emit_output, dag_eval_output,
  ProgramOutputBind)` rows inside evaluator-supported programs.
- **E9 / L5:** remains blocked on LanguageSpec readiness, all Shape A targets,
  corpus-home, typed structural observation, and `ForAllTargets` authority.
- **W3 / `ForAllTargets`:** must not borrow W1's stdout carve-out or producer
  names as structural observation authority.
- **Phase 5 evaluator closure:** #1857 remains only E6-G1.a Option 3 mechanism
  evidence; it is not related to this runner producer-contract queue.

## Validation For This Packet

Minimum validation for edits to this packet:

```text
git diff --check -- docs/briefs/r3-pr-e8-w1-producer-contract-test-plan-worker.md
rg -n "W1|rust_emit_output|dag_eval_output|DifferentialEquals|ProgramOutputBind|ProgramObservation" \
  docs/briefs/r2-pr-b-2-runner-extension-bundle.md \
  docs/briefs/r3-evaluator-dispatch.md \
  docs/briefs/r3-pr-e8-w1-output-producer-contract-blocker.md \
  docs/briefs/r3-pr-e8-runner-extensions-continuation-readiness.md \
  docs/briefs/r3-pr-e9-post-w1-lane1-consumption-readiness.md \
  src/v3/std/runtime.dag \
  src/v3/std/verification.dag \
  src/v3/compiler/src/test_runner.rs \
  src/v3/compiler/src/emit/rust_target.rs \
  src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag \
  src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs
```
