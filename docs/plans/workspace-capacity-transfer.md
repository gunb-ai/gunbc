# Transfer existing fleet capacity to the first workspace

The operator confirmed on 2026-09-28 that setting aside capacity is part of this
allocation work, not an external prerequisite expected from another session.
Implement the change through existing fleet convergence. Do not add an independent
workspace memory pool or increase host/attempt ceilings.

## Proposed first transfer

Inspect `srv1-13`, the existing explicitly budgeted microVM shakedown member, as the
first candidate. Its authored identity is retained even when the ordinary runner
width changes. A transfer reuses its existing 28 GiB attempt envelope; a 4 or 8 GiB
guest does not give the remainder back to another allocator. Controller cost stays
separately accounted. This is a candidate, not a claim that the slot is currently
free or admitted. Website requests still select any eligible host.

The transfer needs these connected steps in the existing convergence plan:

1. Read the current host budget and slot purpose, active attempt/hold, and readiness
   history on the actual host. An absent store or failed service is not a free slot.
2. Withdraw competing shakedown/runner demand for this member. Check both offer
   production and direct reservation/launch entries; filtering a UI roster alone
   cannot establish exclusivity. Preserve any already elected acquisition until it
   completes or is fenced/drained.
3. Finish prior obligations and observe sanitation. Preserve existing generations;
   publish initial generation zero only for a proven new lineage.
4. Keep the same host capacity commitment and install the workspace image,
   controller/access, protected-state binding, and bounded observation producers
   through the administrator-owned convergence path.
5. Admit workspace purpose from the completed transfer/readback. Only then may
   workspace offer production expose this capacity. Failure in a preceding step
   leaves the transfer outstanding rather than making the slot available.

The source already contains `workspace_commissioning_admit`, which requires a
matching authorized plan, conserving host budget, withdrawn prior purpose,
accounted history, and sanitation. Extend and connect those authorities; do not
replace them with a flag saying the host is ready.

## Latest read-only census

The srv1 microVM service reports failed with exit status 1 and no control group.
The expected allocation and readiness directories were not present in the census.
The modeled `fabric-cell-srv1-13.slice` reports inactive, no unit fragment, and
unbounded memory values. Those readings do not establish an installed, enforced
28 GiB reservation. No service, file, slot purpose, or live deployment was changed.

A supply-observer repair additionally checks the executing hostname before probing
supply or reading readiness. The remote coordinator must invoke that observer on
the selected host; a same-spelled local path cannot stand in for its authority.

## Still to execute

Produce the actual convergence transfer plan and qualify its withdrawal and budget
controls, commission protected storage/access, apply the qualified plan, and run
request → reservation → SSH → cleanup → same-slot reuse acceptance. The current
empty workspace designation is retained until the transfer is established.


## Direct-controller withdrawal boundary

Source `b8f407d` connects the direct CI controller to the same capacity sanction
used by the broker. A workspace-purpose slot cannot mint a new GitHub registration
through the fallback controller. Prior CI recovery still runs the shared teardown
with the recorded attempt; it needs neither a new registration nor the current CI
image. A workspace in-flight record refuses the CI path so owner/quota settlement
cannot be skipped. Unknown purpose, foreign executor evidence and unready cells
refuse before minting.

Four focused controls pass under 6 GiB/no swap. The complete controller module
loaded and typechecked from normal source roots under its existing 24 GiB modeled
controller budget, then reached the deliberate missing-entry refusal without
executing effects. Receipts: `receipts/allocation-purpose-2026-09-28/`.

This closes one launch bypass, not the whole transfer. It does not fence an elected
acquisition, establish a commissioned slot, or change the empty designation.
The latest read-only srv1 census finds the storage and approval services running,
the slot service failed, and the modeled cell slice absent with unbounded reported
memory values. Credential/allocation/readiness paths were absent or inaccessible
to the current account; this does not distinguish absence from denied traversal.
The HTTPS listener returned 405 to GET; that is reachability, not authenticated
protected-state acceptance.


## Protected-state commissioning readback and credential boundary

Subsequent root metadata readback on srv1 establishes that these paths are absent,
not merely inaccessible to the login account:

- `/etc/gunbc-fabric-storage/state-writer-mac-key`
- `/var/lib/gunbc/fabric/allocation`
- `/var/lib/gunbc/microvm-cell-readiness`

No secret contents were read and no paths were created. The absent key file does
not establish absence of the version-pinned GCP secret.

`fabric_storage_state_provision` contains an app approval request and apply helper,
but repository search found no workflow invoking them. The apply helper currently
expects one credential for both initial secret material and accessor policy
reconciliation. That is not the existing fleet principal split: the convergence
principal cannot modify policy, while `iam-converge` explicitly denies secret
version creation and access. Do not widen either identity or substitute an
operator bearer token to make this helper run.

Before commissioning, connect the existing secret actuator and accessor policy
convergence under their respective admitted authorities, with approval over the
concrete effects and independent readback. First observe whether the pinned secret
already exists; a missing host file is not permission to create replacement
material. This credential connection is part of the allocation vertical, not a
claim that the broader GCP consumer migration has been completed.

The restricted socket dependency #12482 remains open. Its transport was integrated
and locally qualified in #12505, but neither the dependency nor the allocation
controller has been installed by this work. Preserve the review/commissioning gate.
