The `primitive-runtime-body` required phase belongs to the **witnesses** lane.
`claim_executor --required-ci --required-lane witnesses` runs it after `parse`
and before `floor`. Failure enters the phase-failure ledger; the existing
required-CI measurement adjudication and required-lanes aggregate refuse it.
The host/authority phase-roster joins and expected-versus-ran check include it.
No workflow job or CLI flag was added.

`gunbc.witness_floor_workflow.witness_floor_triggers` runs that lane on pull
requests targeting main (opened, synchronize, reopened), `merge_group`, and
`workflow_dispatch`. The runtime phase uses the parse sweep's universe, including
the producer root for declaration integrity, in an isolated child of the required
dispatcher, using its already-built sibling
`gunbc`. The child exits before `floor`, releasing the additional preparation
caches rather than retaining another corpus pool in the floor process. It does not widen the ordinary floor subject,
whose exclusion of src/v1 protects it from the documented name collisions.

The producer integration lives under `test/primitive_runtime_body`, outside both
the ordinary floor roots and the v1-only stage0 regeneration sweep;
its reusable census and mutation predicate remain under `dag`. The phase calls
`v1.compiler.primitive_runtime_body.check`. That door reads the current
`rt_function_registry` and `rust_runtime_source()` declarations, derives body rows
through the shared Rust item scanner, and joins **bridge_name**, not operation
spelling. No copied registry, body-name list, or population-count oracle exists.
The original five-surface D0 query keeps its historical scope;
`primitive_surface_census_with_runtime` provides the six-surface census.

The same required invocation always executes a permanent negative control:
replace the live `map_insert` row's bridge with `runtime_body_missing_control`,
keep its operation and calling convention, and require the checker to refuse
exactly that body. Missing control subject, unexpected missing rows, unreadable
source, or a control that unexpectedly passes all stop the phase. This control
remains enrolled when the wall is green. The same phase also checks the complete
combined-surface join and refuses an appended bridge with no body, sharing the
standing census. These producer controls execute here rather than appearing as
unexecuted test declarations outside ordinary discovery. The fixture witnesses additionally
exercise automatic enrollment, alias spelling, bodyless declarations, scanner
refusals, and preservation of multiple identity joins.

Run the production .dag door directly with a built `gunbc` and an enforceable
memory bound:

```sh
gunbc run --source-root dag --source-root src/v2 --source-root src/v1 \
  --source-root test/primitive_runtime_body \
  --entry test/primitive_runtime_body/producer.dag --function check
```

Required-regen is complementary. Its host `compare_generated_surfaces` compares
committed bytes to normalized candidate bytes; the `.dag` adjudicator
`required_regen_sync_admission` receives surface-match/drift and population
outcomes, not runtime definitions. `compile_stage0`'s emitted-edge admission
checks module/package coverage and reachability, not bridge function bodies.
The behavioral-receipt annotation in claim_executor explicitly records that
regen and the fixed point do not compile the candidate. Thus up-to-date mirrors
can still name a missing body; mirror freshness and body presence are two facts.
The new phase consumes neither a duplicated mirror comparison nor a second body
validator: both the live population and mutation control call
`std.primitive_identity.primitive_runtime_body_check_in`.

Qualification:

- At main base `b1b7aea9aa7`, all 62 bridge rows have emitted definitions.
- A detached probe at `lane/demand-integration` head `d691e985604` auto-enrolled
  all 63 rows, including `observed_monotonic_nanos`, using byte-identical census
  and gate implementations. Neither its registry nor runtime authority needed
  an enrollment edit. The version-specific count assertion was external probe
  data, not a committed acceptance rule.
- Removing only the clock definition in that disposable probe made the gate
  refuse `observed_monotonic_nanos -> v1_rt::observed_monotonic_nanos`; the file
  was restored afterward.
- The final dispatcher suite passed all 16 tests in 146.95 seconds, including
  the exact child-process phase adapter. That same adapter refused the missing
  clock body in 144.77 seconds. Both include source preparation; neither is a
  fleet measurement. The registry and runtime files in the detached probe were
  restored after the mutation.
- All 35 scoped model/regression witnesses passed. The changed dispatcher also
  compiled through Clippy with warnings denied; workspace formatting passed.

Scope: presence of file-level function bodies in the scanner's supported Rust
syntax. Signatures, visibility, cfg-specific availability and behavior are not
proved by this join. In particular, this does not substitute for the emitted
clock specimen. The check is now required for merge admission; it is not yet a
per-invocation compiler diagnostic. The producer controls in this directory execute inside the phase, which always
checks the live producer and its permanent negative control.

The merge-group build lane also qualifies this placement: putting the producer under
`src/v1` made stage0 regeneration import its v2 scanner dependencies into a v1-only
source sweep. The parse/declaration and runtime phase enrollment preserves enforcement without
widening that separate compilation subject.
