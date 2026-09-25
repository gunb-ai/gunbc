# The execution-cell contract (owner directive 2026-09-25)

Supersedes the "crash-era residue" framing of the Group B start failure.

## The ruling, verbatim

Treat the Group B failure as a missing execution-generation dependency, not as
incidental residue. The new unit's cleanup hooks are downstream of systemd start
admission, so they cannot repair start-rate state or orphan resources that
prevent the start. Move prior-generation inventory, quiescence, disposal,
manager-state settlement, and readback ahead of preparation.

Prepare every rank before activating any. A four-rank arm is one cohort:
preparation failure starts none; partial activation rolls back all started
members; success requires one exact generation across every rank.

Extract this as a realization-neutral execution-cell contract. Keep
systemd/Docker as the current Group B realization and adopt Firecracker next for
disposable harness workers. Do not claim that microVMs provide idempotency by
themselves — the host controller must still own operation identity, CAS,
resource conservation, disposal, and readback.

Preserve the current wet run as measurement evidence only. Even if it succeeds
after manual cleanup, prove the repaired convergence route against planted
failed-unit and orphan-container residue before restoring persistent service.

## What this means structurally

### The phase order is law

```
inventory(prior generation)
  → quiescence(prior generation)
  → disposal(prior generation)
  → manager-state settlement (rate counters, failed state, unit-table)
  → readback(the prior generation is actually gone)
  → preparation(new generation, every member)
  → activation(cohort)
```

Anything in the new generation's cleanup hooks is unreachable for failures of
start admission; cleanup of the PRIOR generation must precede preparation of
the new one. The Sep 25 failure is the canonical witness: start-rate counters
and an orphan container blocked admission before the unit's own ExecStartPre
could run.

### Cohort semantics

An N-rank arm is ONE execution cell, not N units:

- preparation is per-member but all-or-nothing: any member's preparation
  failure activates none;
- activation is sequenced with rollback: a member that fails after others
  started disposes the started members;
- success is a single exact generation (identity of the rendered intent)
  observed across every member — never "three ranks on generation G, one on
  G-1".

### Realization neutrality

The contract names phases, not tools. systemd/Docker is the current Group B
realization; Firecracker is the next realization for disposable harness
workers. A realization supplies the mechanisms for inventory / quiescence /
disposal / settlement / readback / prepare / activate. Idempotency is NEVER a
property of the substrate: the host controller owns operation identity,
compare-and-set on admission, resource conservation (nothing leaks, nothing is
double-consumed), disposal, and readback, for every realization.

### Evidence discipline

The 2026-09-25 wet run (manual pre-cleanup by the operator session) is
measurement evidence only — its memory-profile receipts feed the fit proof,
but it does NOT discharge the convergence-route proof. Before persistent
service is restored, the repaired route must pass against PLANTED failure
state: a failed-unit (tripped start-rate counter) and an orphan container,
planted per host, with the route required to inventory, settle, and read back
their absence before activation. That planted-residue witness joins
`fleet-slot-retirement-wet-proof` as a reopening precondition.

## Work queued from this ruling

1. Execution-cell contract module (realization-neutral): the phase vocabulary,
   cohort type, operation identity + CAS, conservation laws.
2. systemd/Docker realization of the contract, replacing the current apply
   bracket's prepare-then-start shape (shared by the experiment bracket and
   the persistent serving apply).
3. Planted-residue witness: failed-unit + orphan container, per host, repaired
   by the route itself.
4. Conjunctive reopening gate amendment: + execution-cell route proven against
   planted residue.
5. Firecracker realization for disposable harness workers (the microVM node),
   owning identity/CAS/conservation/disposal/readback in the host controller.
