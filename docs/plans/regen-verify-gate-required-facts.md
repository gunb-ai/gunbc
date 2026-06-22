# regen --verify gate — required facts (carried from the deleted emit_rust header)

Status: parked design row. Pins the load-bearing operational facts that the
`v1_compiler_emit_rust.rs` header carried before the codebase-wide comment ban
deleted it (deletion was operator-directed; git history holds the full original
at `git show <pre-ban-sha>:src/v1/stage0/src/v1_compiler_emit_rust.rs`). A
comment described the regen hazard but enforced nothing — the structural home for
the fact is this gate. Breadcrumb: `node://adhoc-80af9ff8-40f` (#5514 deferral).

## Why this row exists

`src/v1/stage0/src/v1_compiler_emit_rust.rs` is a **hand-synced mirror** of the
`.dag` authority, **not** a clean regen fixpoint. There is no `regen --verify`
gate, so a blind regen would silently overwrite the two hand-resolved drifts
below and regress the wire-policy fix. The protection was always absent; this row
makes the fact durable so it rides the gate when it lands.

## The two hand-resolved local drifts (must be reproduced by a faithful regen)

1. **cargo header inlined** — the file inlines its cargo header rather than
   importing the deliberately-unwired `extdeps_cargo_version` orphan module.
2. **policy ref/value alignment at `wire_value_serialize.rs`** — the wire-policy
   reference/value alignment is hand-resolved here; a naive regen loses it.

All other emitted modules stay byte-identical to the committed seed.

## Deferral provenance (#5514, verbatim)

"regen-write still emits a non-building seed from ~150 emitter-completeness gaps;
the clean green regen fixpoint is deferred, converges with the Route-A emitter
self-host." A faithful full regen wires the deliberately-unwired orphan std
modules (`std_measure`, `std_algebra`, `std_realization_schedule`,
`std_machine_constraints`, `std_integer`, `extdeps_version_semver`,
`extdeps_cargo_version`) and surfaces ~150 latent emitter-completeness gaps
(std numeric / measure-tower generic emission).

## Dissolution trigger

§7 fixpoint convergence with the Route-A emitter self-host — when
wire-all-emitted-modules → regen-equals-committed is in reach, the
`regen --verify` gate (a wall: a clean regen must reproduce the committed seed
bit-identically) becomes buildable and this row dissolves into it.
