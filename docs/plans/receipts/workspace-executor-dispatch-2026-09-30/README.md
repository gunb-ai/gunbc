# Fleet commissioning dispatch qualification

Source qualification only. No host service, protected store, fleet generation, initial readiness or VM was changed in this checkpoint.

## Production connection

The commissioning-only fleet scope now uses the sealed privileged observation and existing run-bound plan receipt. Apply captures all artifact bytes, validates sidecars and run/revision/repository binding, and sends one canonical bundle to the installed root staging entry. Existing durable CAS publishes all four artifacts atomically; same input retries, different input refuses.

The caller takes the fleet lock as its existing owner, stages or reattaches, captures the executor invocation, and releases the lock before waiting. The host executor retains the enclosing transaction. Completion requires protected generation-completed journal, durable generation receipt and successful exact unit terminal. A readable refusal, changed invocation, pending controller, timeout or cancellation cannot become `FullyApplied`. Ordinary fleet apply remains unchanged, including genuine commissioning no-op plans.

Three exact-release argument-free root entry points are installed through generated sudoers grants, syntax checked before publication, and read back with the installed adapter. Their inputs are data on stdin, not job-writable executable paths. No per-poll interpreter is introduced.

## Evidence and limits

Compiler SHA-256: `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`. Existing compiler, byte-identical dependency copies; not an exact-head compiler build or required CI floor.

Fourteen native dispatch controls pass against the unchanged exported Bash transport. They use executable boundary doubles and a real flock with an independent executor process. Cases include failure/refusal, missing or changed invocation fields, uncommitted generation, wrong bundle, cancellation/reattachment, lost response, concurrent callers and indefinitely pending completion. These do not execute the installed root helpers or live systemd/controller.

Integrated source results are recorded in the accompanying logs and source manifest. Host installation, protected CAS wet execution, commissioning and VM acceptance remain separate gates. Protected input replacement is deliberately refused; this checkpoint only supports retry of the same commissioning transaction.

Four combined integration controls passed on the final 1,992-module pool (1,154 demanded modules), under 12 GiB/no swap, reported peak RSS 9,206,616 KiB. This includes production fleet CLI typechecking, scope routing, exact helper grants, installer/bundle controls, and operator-versus-lock-owner dispatch.

## Qualification blocker

The full fleet regression entry typechecked after adding the new scope to two exhaustive test helpers. It passed 70 of its 87 controls before the approved 12 GiB/no-swap scope was OOM-killed (exit 137; systemd `Result=oom-kill`). No failing assertion was reported. The demanded closure contained 1,985 modules; pre-entry reported peak RSS 12,192,168 KiB. This is incomplete qualification, not a green full suite. No ceiling was increased and no live commissioning was attempted. The remaining controls and exact-head CI remain outstanding; reducing test/closure memory is required before calling this local gate complete.

## Compact historical manifest

`fleet-sources.json` uses `source-manifest-delta/v1` over `sources.json`: verify the base SHA-256, replace the declared prefix in each row’s `copy` field, then replace rows by `module` from `replace_rows`, preserving array order. Serialize as compact JSON plus one newline and verify `reconstructed_sha256`. The reconstruction was checked byte-for-byte against the original 543,601-byte manifest before compaction. Only one source hash differs; all 1,992 module identities and the original scratch locations remain recoverable. This keeps the judged diff below its existing 8 MiB bound.
