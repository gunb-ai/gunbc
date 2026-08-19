# Replacing GitHub Actions with the fabric: the plan

**Status: proposed, awaiting sign-off.** Operator direction (2026-08-19): do the migration as one
PR, no long shadow process. That is the replacement doctrine's *default* — delete-first, one atomic
authority transition — rather than the gap-intolerant carve-out I had proposed. The plan below is
written to that ruling.

## 1. What is actually being replaced

GitHub Actions is a **fused authority**, and the recut program has just spent four cuts learning to
recognise one. It is simultaneously:

```
event source      — a push or PR opened/synchronised
allocator         — which machine, when, and how many at once
executor          — checkout, toolchain, build, run
status sink       — the red or green a human and the merge button read
log store         — where the output goes
concurrency       — cancel the superseded run
```

The fabric has authority over **allocation and execution**. It has no authority over webhooks or a
log UI, and pretending otherwise is how an MVP becomes a project. So this is the same move the side
chat ruled for `ComputeOffer` in §16 of the recut program: decompose the fused carrier, replace the
parts that have a home, and leave the rest cited rather than absorbed.

## 2. The contract to preserve, measured from `.github/workflows/witnesses.yml`

**The minimum replacement is not the smallest thing that runs the floor.** It must preserve every
refusal, or it has erased a correctness distinction rather than completed a migration. Measured
from the live workflow rather than remembered:

| behaviour | current | must survive |
| --- | --- | --- |
| triggers | `workflow_dispatch`, push to `main`, PR to `main` (`opened`, `synchronize`, `reopened`, `ready_for_review`) | yes |
| runner | `[self-hosted, linux, arm64]` | yes — via the Cut C seam, not a literal |
| timeout | 180 minutes | yes, as a typed bound |
| concurrency | group per PR number or run id; **`cancel-in-progress` for PRs only** | yes — main runs never cancel |
| checkout | `fetch-depth: 0` (full history) | yes |
| toolchain | `setup-rust-toolchain@v1.16.0`, `cache: false` | yes |
| build | `claim_executor`, `gunbc`, `v1_src_dag_parse` | yes |
| gate 1 | `v1_src_dag_parse` — src/v1 `.dag` sources parse, **a separate step that must pass** | yes |
| gate 2 | `claim_executor --required-floor --source-root dag --source-root src/v2` | yes |
| env | `GUNBC_EXPECTED_RED_ROSTER_JOIN=expected_red_roster_join.tsv` | yes |
| failure | non-zero exit on any step is red | yes |

Two of these are easy to lose silently and are called out for that reason: **the v1 parse gate is a
separate step** (it was invisible for 98 minutes once already, gunbc#8466 → #8519), and
**cancellation is asymmetric** — cancelling a main run would lose the only unconditional signal
that main is green.

## 3. Why this is the right work, not a side quest

`product.fabric.*` is 80 declarations across six modules with **no production consumer**. Every
guarantee it carries is currently type-level. This replacement makes the fabric load-bearing on the
one workload we already own end to end, which is the difference between a modeled market and a
market.

It is also the **forcing function for Cut D**. Cut D stalled on two authorities that do not exist:
an execution class, and a fleet-derived runner authority (§17). Both are exactly what a CI
replacement must construct anyway — "what kind of machine does this run on" *is* an execution
class. So this work does not compete with the recut program; it grounds the part of it that had no
consumer to justify its shape.

## 4. The terminal loop

```
poll GitHub                         extdeps.github.push_event / pulls
  → Work                            "required floor at tree T", keyed by WorkContentKey
  → Demand                          admission terms, budget, satisfaction requirement
  → match eligible Offers           fleet hosts, projected per Cut D's D2
  → ExecutionGrant                  accepted offer revision + reservation + executor + LeaseEpoch
  → Attempt                         checkout, toolchain, build, v1 parse gate, required floor
  → Receipt                         verdict, counts, evidence
  → CheckRun PATCH                  extdeps.github.checks
```

**Polling, not webhooks**, deliberately: no inbound networking, no webhook secret, no listener to
secure. The fabric does not care how the event arrived, and a poller is strictly less
infrastructure than an endpoint.

**The `LeaseEpoch` from Cut A is load-bearing here**, not decorative: two pollers, or a poller
restarted mid-run, must not both hold a grant on the same work. That is the fencing the epoch
exists for, and this is its first real consumer.

## 5. What the one PR contains

1. **`product.fabric` gains its executor** — the fold from Demand to Receipt. Types exist; the loop
   does not.
2. **Fleet hosts project Offers** (Cut D's D2), with an explicit zero quote — owned supply.
3. **An execution class** for `linux/arm64 self-hosted`, which is what `RunnerSpec` should have
   been derived from all along (§16), retiring Cut D's blocked precondition.
4. **The poller as a modeled systemd unit**, following `live_deploy_systemd_unit_for` — the only
   genuinely new infrastructure, and the tree has the pattern.
5. **Check Run reporting** via the already-modeled `extdeps.github.checks` POST/PATCH.
6. **`.github/workflows/witnesses.yml` deleted**, and `gunbc.witness_floor_workflow` with it.

## 6. Bootstrap and rollback, since there is no shadow

**The hazard is real and must be stated rather than mitigated by optimism: after this merges, the
system that gates merges is the system that just changed.** If it is broken, the fix cannot be
merged through it.

Three things make that acceptable rather than reckless:

- The **cutover PR itself is validated by Actions**, because the PR is still under the old regime.
  The last green Actions run is on the exact tree that contains the replacement.
- **Rollback is `git revert`**, and the operator merges manually today, so a revert can always be
  landed by a human who can see the check is not reporting. This is the actual escape hatch, and it
  is a person rather than a flag — which is the correct shape.
- The **fleet is already the execution substrate.** The runners are self-hosted on our hosts today,
  so this changes the control plane, not the machines.

## 7. What gets weaker — declared, not discovered

**Independence of the control plane.** Today a fleet outage costs execution but GitHub still
queues the work and reports the status. Afterwards, a fleet outage means *nothing notices the
commit at all*. That is a genuine reduction and it belongs on the ladder as a declared rung, with
its trigger: it climbs when the poller has a liveness signal whose absence is itself observable —
a check that goes red when no poll has happened, not merely green when one has.

**Log retention and the run UI** move from GitHub's storage to ours. The MVP keeps the Check Run
output as the human-readable surface; whole logs need a home before this is at parity, and naming
that gap is part of the plan rather than a follow-up someone discovers.

**Concurrency semantics are re-implemented rather than inherited**, which is where a subtle
regression is most likely — specifically the asymmetry in §2.

## 8. Open, for sign-off

1. **Scope of the first cut: main pushes only, or PRs too?** PRs bring cancellation, merge-queue
   semantics and per-PR concurrency. Main-only is dramatically smaller and still replaces a real
   thing — but it leaves PR gating on Actions, which means the workflow file cannot be deleted, and
   the doctrine's whole point is that the root goes in one motion. I do not think main-only is
   coherent with the no-shadow ruling. I want that checked rather than assumed.
2. **Where does the poller run?** A fleet host is the obvious answer and the wrong one if that host
   is also under test. Does the control plane need to be off-fleet to be honest?
3. **Is the execution class built here or in Cut D?** Building it here makes this PR bigger and
   unblocks Cut D; building it in Cut D blocks this. I lean toward here, because a consumer-less
   authority is what the recut program keeps refusing.
4. **What is the Work identity over?** The tree hash, so an identical tree does not re-run — which
   would be a real gain over Actions, or a real hazard if the floor is not actually a pure function
   of the tree. It reads pure; I have not proven it.
