# Fleet acceptance criteria — completion tiers and per-node operational acceptance

Home: ROADMAP §2 (compute fabric). Every §2 node's **Accept:** line summarizes a checklist here; this doc is the checklist, the node is the summary, and ROADMAP.md is generated from both — one authority, two zoom levels (operator-set, 2026-07-02).

> A roadmap item is not complete because its algebra exists. It is complete only when a **named consumer** uses it on the **live path** and produces a **receipt**.

This is DESIGN §5 applied to roadmap admission: a typecheck or a grep is not a consumer; done means a real consumer runs green **and** a discriminating perturbation goes red. The bar for real progress on this pillar: *can srv1/srv2 runner allocation be changed through the model, read back, re-run safely, and retracted?* Until yes, the work is scaffolding — valuable scaffolding, but not the operational milestone.

## Completion tiers

- **T0 — modeled only**: types and folds exist, nothing consumes them.
- **T1 — fixture/synthetic witness**: green by execution on synthetic rows; a fixture RED exists.
- **T2 — wired to the intended consumer**: the named consumer computes through the new carrier on its real code path (proof by consumption — remove the carrier and the consumer's witness fails to resolve — never by grep).
- **T3 — live dry-run receipt**: the real host/CI path runs read-only or plan-only and emits a receipt.
- **T4 — live apply + independent read-back receipt**: the mutation happened on the real host and an independent read (never our own write echoed back) proves the effective state.
- **T5 — periodic/CI consumer keeps it alive**: a timer or CI gate re-executes it on cadence, so a regression or hand-edit reds without anyone remembering to check.

Every dispatchable node names its required tier in its **Accept:** line. A roadmap checkbox does not close below the named tier. For fabric/control-plane items the required tier is almost always **T4 or T5** — anything below is a useful subtask, never completion.

## The acceptance block for dispatch briefs

Paste into every work item; a brief without one is not ready to dispatch:

- **Consumer:** the named live consumer that calls the new carrier.
- **Green:** the real command / CI job / host apply that succeeds and emits a receipt.
- **RED:** the specific perturbation that fails for the intended reason.
- **No inert landing:** no alternate path still uses the old mechanism.
- **Receipt location:** the artifact/log/file/commit recording observed before/after state.
- **Re-run behavior:** re-running is a noop or a typed conflict, never a blind rewrite.

## 2-keyed-delta-fold — required tier T2 (wired)

The fold/diff carrier does NOT close because a generic algebra exists. It closes only when `gunbc.host_converge` computes a real converge patch through it. T4 arrives only via 2-converge-reland. Complete only when ALL hold:

1. Reuse or consciously extend `v2.std.change` (`ChangeKind` = NodeChanged | NodeAdded | NodeRemoved | ProjectionChanged | ArtifactChanged; `ChangeSet` — already consumed by `v2.lens.affected_set`); a parallel Hunk/Patch vocabulary requires the PR to explain why attaching failed. The algebra anchor (Monoid/lattice/inverse) picks ONE std authority consciously — `algebra` is pillar-1's FIRST de-fork target (LIVE fail-open), and the standing ruling applies: coproduct = structural authority, grounded-realization wins; never a third surface.
2. Generic keyed delta fold exists with law witnesses: diff(A, A) = identity; unchanged keyed rows emit NO hunk by the monoid identity (absence by law, never by filtering); key order does not change the produced patch.
3. Three-way fold over (base, observed, desired): observed == base and desired != base gives an apply hunk; observed == desired gives already-applied (a clean noop, not an error); observed != base and observed != desired gives `Conflict`. `Conflict` has NO apply arm — structurally unreachable mutation, not a runtime guard.
4. The codomain carries NO Unknown element. Bottom is the BENIGN `Unchanged` (the safety inversion of the `DescentEvidence` precedent, whose bottom is the fail state): an Unknown at bottom would join-fold unreadable state to "no change" — a §5 fail-open. Unreadable state is refused upstream at the typed-read wall, never folded.
5. Hunk vocabulary covers Added, Removed, and Modified (three-way union, not modified-only).
6. apply(diff(A, B), A) = B is witnessed by execution.
7. invert(diff(A, B)) applied to B retracts to A — the inverse law. The structure is a groupoid, NOT a total Group: apply is partial (refuses off-base) and `Conflict` has no inverse; a total apply derived from Group inhabitance would silently compose through drift.
8. A `gunbc.host_converge` consumer uses the generic fold: base = current `ConvergeKnob` rows; desired = an srv2 override changing runner memory/swap; the patch contains exactly the expected memory/swap hunks and no unrelated hunks.
9. Per-target equality/normalization is supplied by `host_converge`, never the generic carrier — all SIX `ConvergeTarget` variants have their own semantics witness: `SliceProperty` (apply-value vs expected-effective split — written string != effective string even inside one variant), `PerSlotMemoryCap` (byte normalization), `RunnerWidth`, `JobserverTokens`, `VerifyOnlyCap` (no apply value at all), `GunbcPinnedTree` (converges on pin-coherence + marker, not value equality). One string-equality over these is a semantic flattening.
10. RED controls: reorder knobs → same patch; add/remove a knob → Added/Removed hunk appears; observed drift → `Conflict` and no apply plan; wrong per-target semantics → witness fails.
11. The closed-loop converge PR imports this carrier; NO bespoke differ remains on the converge path. Proof by consumption: remove the carrier and the re-land witness fails to resolve.

## 2-live-read-seam — required tier T3/T4 (read side live on both hosts)

The first slice (one srv2 unit's memory knobs, node 2-live-read-runner-memory) is its own narrow node and may close alone; it does NOT close the full seam. The full seam is complete only when:

1. Live reads run on srv1 AND srv2, not just fixtures.
2. Reads cover at least: active runner units (glob `actions-runner@<host>-*.service`), `MemoryMax`, `MemoryHigh`, `MemorySwapMax`, `TasksMax`, `CPUWeight`, runner count, and the jobserver token value (`CTRL_JOBSERVER_TOKENS`).
3. Read output is typed rows, never string blobs (no string flattening past the parse boundary).
4. Fixture RED: absent property → typed Absent; `infinity` where bytes are expected → drift verdict; wrong bytes → drift verdict.
5. Live green: one real srv2 runner unit read grounds into the `ConvergeTarget` semantics.
6. Full-host receipt: srv1 and srv2 receipts each list ALL runner units and their effective knobs.
7. NO mutation occurs anywhere in this node.
8. The closed-loop converge precondition consumes these live reads (T2-style proof by consumption).

## 2-converge-reland — required tier T4 (live apply + independent read-back)

Complete only when this end-to-end flow runs on a real host. Target: srv2, one real runner unit or the runner template drop-in; knobs `MemoryMax`, `MemoryHigh`, `MemorySwapMax` (preferred also: jobserver tokens). Names like the committed-base snapshot and the override edit layer are the #6096 re-land design's intent — no such carriers exist in-tree yet; the effective base is `host_converge`'s `ConvergeKnob` rows, never `fleet_intent` (physical inventory).

1. **Snapshot** — a committed base exists, derived from effective `host_converge` `ConvergeKnob` rows; its content hash is recorded.
2. **Edit** — the operator changes the srv2 runner allocation through the model's override layer; NO edit to raw emitted bash, NO edit to `fleet_intent` for mutable runtime knobs.
3. **Diff** — the three-way diff produces a reviewable patch listing ONLY the changed srv2 knobs; no srv1/srv3 hunks appear.
4. **Admission** — post-patch values feed RAM/swap/pids/CPU-token accounting; an unsound plan fails; the plan names Strict or Burst mode explicitly.
5. **Precondition live read** — observed srv2 state is read BEFORE mutation; if observed != base and observed != desired the plan returns `Conflict`, emits NO shell, and calls no host apply.
6. **Apply** — privileged mutation goes through the gated host-effect path: unprivileged actor refused before mutation, wrong host refused before mutation; the command writes the intended systemd drop-in or property.
7. **Independent read-back** — srv2 is read again after apply; green ONLY if observed effective values equal desired. Shell exit-0 is not sufficient.
8. **Commit** — the new base is committed/materialized; the receipt records old hash, new hash, host, unit, changed knobs, observed-before, observed-after.
9. **Re-apply** — running the same plan again is already-applied/noop; NO shell mutation is emitted on noop.
10. **Retract** — the inverse patch restores the previous values; read-back proves restoration; re-running retract is a noop.
11. **Persistence** — `systemctl daemon-reload` does not lose desired values; a restarted or newly spawned matching runner unit inherits the intended drop-in values.
12. **RED controls** — manually perturb srv2 `MemoryMax` before apply → `Conflict`, no shell; remove the live read → `NotConverged`, no shell; unprivileged actor → typed refusal, no shell; unsound desired allocation → admission rejects.

## 2-runner-allocation-v0 — required tier T4, held by T5 (the operational milestone)

This is the milestone that says: **srv1/srv2 are no longer hand-configured for runner allocation.** Complete only when both hosts are configured through the model and read back:

1. **Model** — `RunShape`/workload classes exist for at least ci_floor, rust_heavy, deploy_srv1; HostSupply exists for srv1 and srv2 (RAM, swap, pids, CPU tokens, reserved host/session headroom).
2. **Policy** — StrictLease is the default; runner count does NOT equal heavy-job concurrency; per-slot cgroup caps are containment ceilings; admitted concurrency derives from HostSupply × WorkloadClassDemand.
3. **Desired allocation declared** — per host: runner slots, labels, per-slot `MemoryMax`/`MemoryHigh`/`MemorySwapMax`/`TasksMax`, jobserver/build tokens.
4. **Apply** — converge applies the desired allocation to srv1 and srv2; no hand-edited systemd files are required after apply.
5. **Read-back** — all active runner units on BOTH hosts match desired effective values; a newly started runner unit inherits the desired template drop-ins; the jobserver token value matches desired.
6. **Backpressure** — GitHub cannot schedule more rust_heavy jobs than admitted heavy capacity: either only admitted heavy slots carry the heavy shape label, or a control-plane admission gate blocks excess; generic self-hosted labels cannot bypass heavy admission.
7. **Receipt** — records host, runner unit count, labels, effective memory/swap/pids values, jobserver token value, model hash, timestamp.
8. **RED controls** — hand-edit one srv2 cap → receipt reds or converge conflicts; remove/wrong label → dispatch witness fails; desired heavy concurrency above admission → plan rejected; unprivileged apply → refused before mutation.
9. **Workload proof** — one real CI run lands on the intended shape; the cgroup receipt shows no oom_kill increase; memory.swap.events shows no unexpected fail; peak memory is recorded and fed back as a measurement row.

## 2-host-admission — Σ-accounting, both modes enforceable

1. Admission computes from POST-PATCH desired values, never fleet-wide literals; feeding old/base values instead of post-patch values fails the witness.
2. Inputs include: host RAM, host swap, reserved host memory, reserved session memory, pids budget, CPU-token budget, per-workload measured RAM/swap/pids demand.
3. Guaranteed-mode witness: Σ(memory_max) ≤ RAM budget and Σ(swap_max) ≤ swap budget; an overcommitted plan fails.
4. Burst-mode witness: admitted_count × ram_margin + headroom ≤ host_ram; admitted_count × swap_margin + reserve ≤ host_swap; caps are ceilings, not reservations; a plan that passes ONLY by assuming every slot simultaneously consumes its burst ceiling is rejected or explicitly marked Guaranteed-mode.
5. Live receipt: one srv2 run records pre/post memory.events — oom_kill did not increase, memory.swap.events fail did not increase.
6. RED: admitted heavy jobs above the RAM budget → rejected; swap demand above the host swap reserve → rejected. This closes the exact failure that made single-slot-swap ≤ host-swap too weak in #6096.

## 2-periodic-actuation — required tier T5

A timer existing is not acceptance. Complete only when the timer catches or corrects real drift according to per-knob policy:

1. srv1 and srv2 each have an installed systemd timer (or ctrl thin-run) executing the emitted converge under the privilege model — no implicit root hope.
2. Each run emits a receipt: host, model hash, observed hash, applied/noop/conflict verdict, changed knobs — landing where CI/dashboard can read it.
3. A no-change run is a green noop.
4. A hand-edit to a managed runner cap is detected: an auto-correct knob is restored with read-back proof; a verify-only knob (the `VerifyOnlyCap` semantics) refuses with a conflict and does NOT mutate.
5. RED: broken sudo → typed refusal; removed live-read access → `NotConverged`; a hand-edit inside the managed keyspace is never silently ignored.

## 2-strictlease + 2-shape-labels + 2-provider-offer — dispatch without a scheduler

1. `RunShape` includes arch, OS, CPU tokens, RAM, swap, pids, trust class, workload class.
2. `ProviderOffer` rows exist for the srv1 strict slot, the srv2 strict slot, and the DORMANT Ubicloud ephemeral offer.
3. Allocation witnesses: srv1 and srv2 offers each satisfy a shape; the Ubicloud offer satisfies a shape WITHOUT host-mutation support.
4. Host mutation refuses Ubicloud offers structurally (no cgroup target to mutate).
5. CI YAML consumes shape labels, not host labels, for non-deploy jobs; deploy-to-srv1 stays host-proofed, never shape-only.
6. Backpressure: available shape labels equal admitted capacity; extra generic runners cannot accept heavy jobs.
7. RED: a shape needing host mutation on Ubicloud → refused; a deploy job on a non-srv1 shape → refused; a heavy job without the heavy label → not scheduled. One real CI job runs under a StrictLease allocation whose Receipt proves effective limits equal the offer's declared limits, read back from the live cgroup; a second job requesting more than remaining capacity is refused a lease.

## 2-service-receipt — required tier T4

Complete only when a real service on srv1/srv2 is applied AND retracted:

1. Apply starts the service; independent read-back proves: process running, expected port listening, route installed, served artifact digest equals desired digest; the health endpoint answers as expected.
2. Re-apply is a noop.
3. Retract removes route, service/process, and owned artifact; read-back after retract proves ABSENCE.
4. RED: process running with wrong digest → `NotConverged`; route present but port dead → `NotConverged`; apply exits 0 but read-back fails → `NotConverged`.

## 2-compile-clean-shard-b — the floor consumes the shards

Shard A's partition/composition proof + one exemplar is a first receipt, NOT "compile-clean is sharded". Complete only when:

1. The floor plan uses compile-clean shards, not only exemplar witnesses.
2. Every shard derives from the same source-root authority as the whole-tree gate.
3. compose(shard verdicts) is equivalent to the whole-tree gate on a green tree; a planted failure in one shard fails the composed verdict; an empty shard list is Unknown/fail-closed, never green.
4. CI receipt shows old batch-1 wall-clock, new batch-1 wall-clock, shard count, and no coverage loss.
5. RED: drop one shard from the roster → the coverage witness fails; a planted bad module in an omitted shard cannot pass.

## What "complete" honestly means today

Rails (honestly completable now): CD target-host proof first slice · privilege refusal before mutation · runner memory live-read carrier first slice · mechanism inventory baseline · resolver profile receipt · compile-clean shard algebra/exemplar.

NOT complete until their named tier: srv1/srv2 runner allocation · full live host read seam · keyed-delta fold consumed by converge · closed-loop converge · periodic actuation · compute-fabric dispatch. The first set builds rails; the second set is where infra actually changes.

## Dissolution trigger (DESIGN §6)

Dissolves when acceptance is carried as typed fields on RoadmapNode (a modeled completion tier + acceptance record) consumed by the dispatch surface, or when every §2 node named here has closed at its required tier — whichever comes first. Until then, an Accept: edit on a node and its checklist here land in the same PR (the node is the summary, this doc is the checklist; both project into generated artifacts).
