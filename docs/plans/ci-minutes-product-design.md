# CI minutes as a product: GitHub-native runner SKUs over the fabric

**Status: design note. No code lands from this document.** It records the product shape, the boundary between fabric / provider binding / execution cell, and the decisions that are the operator's rather than the author's. Its terminal shapes become obligations on the fabric contract and the execution-cell lane; its sequencing is subordinate to the replacement-cut doctrine of the day (DESIGN §3 — an existing design is quarry, never authority over the cut).

Doctrine anchors: DESIGN §3 (single authority; external upstream decomposition; replacement migrations cut at the root), §4b (guarantee ladder — construction over validation), §5 (fail-closed; a failure arm refuses, never widens), §6 (denominate the benefit in displaced cost).

## 1. The product contract

The whole customer-visible surface:

1. Install the GitHub App on an organization or repository.
2. Select an execution class and a spend cap.
3. Change one line:

```yaml
runs-on: gunbai-8c32g-short
```

Their existing workflow, Actions UI, logs, artifacts, secrets, checks and rerun controls keep working. A customer never sees Demand, Work, Attempt, Offer, ExecutionGrant, lease generation, reservation price, or which host won.

**The interface is deliberately narrow, and the narrowness is the load-bearing asset:** a job goes in, artifacts come out, egress is outbound-only, nothing persists — no inbound connection, no service discovery, no customer-visible topology, no state surviving the job. That is why per-cell isolation is tractable here and not for general hosting or a long-lived serving surface.

## 2. Three programs, three owners

The word "runner" names three different things; conflating them is the first available modeling error.

- **The fabric** (`product.fabric.*`) decides admission, allocation, budget and settlement. It does not know GitHub exists. **If `AcquireJobs`, `runnerRequestId` or `scale set` appears anywhere in `product.fabric.*`, a provider has been fused into the core** — the §3 violation this decomposition exists to prevent, doubly wrong because the scale-set surface is public preview and will churn.
- **The GitHub binding** — one long-lived listener process per scale set, holding the App credentials, speaking the scale-set protocol and calling `AcquireJobs`. Provider-specific by construction. A GitLab product is a second binding, never a core enum arm.
- **The execution cell** — our host agent materializes an ephemeral VM, injects a JIT runner configuration, and starts GitHub's stock runner binary unmodified. The runner is GitHub's program; the fabric leases the *cell*, not the runner — which is what lets a GitLab product reuse the same cell with a different agent inside and change no fabric carrier.

## 3. The sequence

```
JobAvailable observed            binding, from the scale-set session
  -> Demand                      binding projects into generic fabric facts
  -> fabric admits or refuses    budget, fungibility, isolation, network profile
  -> reserve + build the cell    compute, money, network attachment; readback
  -> AcquireJobs([requestId])    the commitment point, and therefore LAST
  -> JIT config, boot runner     stock binary joins the scale set
  -> GitHub pairs job -> runner
  -> JobStarted                  binds external job to observed runner and cell
  -> JobCompleted                elapsed, measurements, teardown readback
  -> SettlementReceipt
```

**Acquire is last, and the ordering is fail-safe, not stylistic.** A booted cell with no acquired job idles and costs us a little; an acquired job with no cell hangs the customer's build. So nothing is acquired until we can certainly serve it.

**What GitHub still decides.** Within one scale set, GitHub pairs any acquired job with any idle runner. This is not a scheduler to defeat: we create the scale sets, so we choose the partition. **One scale set per `customer scope x execution class x duration class x isolation profile x network profile x pricing basis`** makes every job in a queue identical in every respect the grant depends on, so GitHub's choice is a distinction without a difference; heterogeneity inside one scale set is what would let its choice violate a grant.

**Unverified by this repository:** the scale-set message fields, `AcquireJobs`, and the absence of a job-timeout field in `JobAvailable` come from external research recorded in the side review, not from execution here. The binding must establish them against the live API before anything depends on them, and must pin the preview surface.

## 4. ExecutionClass is the stable product seam

The direction matters and today's model runs backwards: `gunbc.runner_spec_from_offer` derives labels **from a physical Offer**, so replacing a host would change the public interface and leak supply topology into customer workflows.

Terminal direction:

```
ExecutionClass  ->  scale-set name (the customer's runs-on)
Offers          ->  CLAIM they can satisfy an ExecutionClass
```

One class name resolves to one typed requirement record — environment, CPU and memory entitlement, ephemeral storage, isolation profile, network profile, maximum runtime, price — never to a bag of independently meaningful labels. Adding a supplier, changing a backing host, or fulfilling from rented capacity changes no customer workflow.

## 5. Isolation: the terminal construction

**This section deliberately does not design around the fleet's current configuration** — that would premise the product on assumptions already scheduled for death (DESIGN §3, the consequence catalogue). Current state is recorded in §9 as migration debt only.

Terminal: **one execution cell per job, VM-grade boundary, its own network namespace and attachment, an explicit outbound-only egress allow-list, and no route to management, BMC, host services, the operator tailnet, or another cell.**

The construction claim, stated precisely: with a per-cell namespace and an explicit allow-list, *cannot reach the BMC* is **structural — no route exists** — not a firewall rule someone maintains. That is a climb from validation to construction on the §4b ladder, and why a global reachability answer is the wrong shape, not merely the wrong values.

Three things stay hard at any boundary strength:

- **Memory bandwidth and LLC are shared silicon.** No namespace, VLAN or cgroup partitions them. The entitlement-versus-variance question in §7 therefore SURVIVES perfect isolation; the two are orthogonal axes.
- **Egress cannot be literally air-gapped.** The cell must reach GitHub and package registries, so that one channel is both the exfiltration path and the abuse path (mining, DDoS source, spam relay). Rate caps, byte caps and a declared destination set are design items, not footnotes — it is the one place a customer can cost real money without running long.
- **Teardown is a proof obligation, not a cleanup step.** A cell not actually destroyed is a persistent, invisible foothold — the boundary meant to protect the tenant now protects it. Destroy-and-proceed is the absorbing fallback: the honest form observes the attachment absent, the disk gone and the registration deregistered, and **anything unproven holds the reservation** — the rule settlement already applies to money, applied to hardware. A host with an unverified teardown is not available capacity.

Two smaller declared facts: the JIT runner configuration is the one secret crossing into the cell (short-lived, scoped, never landing on a persistent customer-visible filesystem), and artifacts and caches are a declared outbound data path with their own integrity question.

Intra-host boundaries compose `std.access` into principal-scoped least privilege. They do not mint bespoke allow/deny folds — DESIGN's one-authorization-kernel rule applies inside a host exactly as it does across one.

## 6. Fragmentation is a starvation problem, not a packing problem

The question "how do we defragment mixed job sizes across hosts" decomposes into four, of which only the last is fragmentation:

1. **Feasibility** — can this ever run on this fleet? If no host is large enough the demand is infeasible and must refuse immediately and typed, never queue looking like it is waiting.
2. **Admission** — can it run now, given commitments? yes / not-yet / never.
3. **Placement** — where.
4. **Fragmentation** — free capacity in aggregate, none of it usable by one large job.

**Ephemeral single-job cells make fragmentation self-healing:** every allocation releases within one job duration, so a fragmented host is empty soon by construction; capacity is not lost. The real hazard is **starvation** — CI job sizes skew heavily small and short, so small jobs keep filling gaps and a large job never gets a window while the fleet looks busy and healthy.

The mechanism against starvation is a **draining reservation**: stop admitting to a chosen host, hold it, and **backfill** only jobs that provably finish before the drain completes. The safety arithmetic covers the whole cell lifecycle, not just execution:

```
now + provisioning_bound + declared_max_runtime + teardown_bound  <=  drain_deadline
```

**Maximum runtime belongs to the ExecutionClass, not to a per-job prediction.** A declared bound makes the arithmetic exact and fail-closed; overrun is a typed kill (`CustomerRuntimeBoundExceeded`, carrying declared and elapsed), never a silent widening. A handful of bounded duration classes — short / standard / long — gives the allocator exact arithmetic without one scale set per unique number, and **duration class must be part of scale-set homogeneity**, or GitHub may pair a long job with a cell on a draining host.

**Historical duration ranks; it never authorizes.** A distribution may order backfill candidates, estimate queue time, plan capacity and recommend a class to a customer. It cannot admit a non-preemptible job into a deadline-constrained hole while preserving an exact claim. This is §5 for scheduling: a heuristic may optimize, never authorize.

**At four hosts, do not build a scheduler — enumerate.** NP-hard bin packing is irrelevant at n=4; exhaustive placement search is microseconds. Get the cost shape right (no quadratic fold, no copied accumulator) and let the algorithm be exact. Heuristics become interesting at tens-to-hundreds of hosts, by which time the binding pressure will have been measured rather than guessed.

Fragmentation reasoning is **impossible against a scalar supply bound**: 64 free cores split 8+8+48 across NUMA domains is not 64 contiguous, and a scalar cannot tell the difference — hence exact resource member sets are a prerequisite for this section, not an adjacent nicety. `std.machine_shape` `ExecutionDomain` already models the NUMA axis and is consumed by nothing.

## 7. What a SKU promises: entitlement, or a variance bound

An unresolved product decision with direct technical consequences, recorded rather than decided.

- **Entitlement** — you get 8 vCPU and 32 GiB. A resource-access promise. Variance unbounded. Packing is nearly free and tight packing is honest.
- **Entitlement plus a variance bound** — and additionally it performs within X% of a dedicated machine of that size. Requires headroom, contention-aware placement, lower density, higher cost per sellable minute.

**Wall-clock performance is not promisable for arbitrary customer code** — the customer's own program determines runtime — so "performance" can only mean a *variance* bound relative to dedicated hardware, never an absolute time. And a variance bound is sayable only once measured per host and per co-tenancy state; promising an unmeasured one is fabricated plausible output at the product layer.

Utilization is the wrong KPI (DESIGN §6, do not anchor on one): tighter packing raises utilization and contention, which slows jobs, which lowers sellable-minutes-delivered-at-promised-quality per hour. The optimum is interior.

## 8. Logs and the data plane

Customer workflow logs are GitHub's delivery responsibility: the runner communicates outward to the Actions service and the customer reads them in the ordinary Actions UI. Customer build output does not traverse our data plane.

What we own: runner and platform diagnostics, provisioning and teardown receipts, resource and network measurements, and billing evidence.

**This is disjoint from the floor semantic-artifact lane.** That lane exists because *our own* floor verdict must not be recovered from a foreign executor's log; it is internal correctness work that neither gates nor is gated by this product.

## 9. Current fleet state: migration debt, not design input

Recorded so the terminal design is not read as today's description, and so each item carries a trigger.

- `gunbc.fleet_intent` `ExecutionSurface` declares `PerJobFilesystem` with a shared job user and persistent runner installations. Adequate for our own trusted repository; not a boundary for arbitrary customer code. **Dissolves when** the execution-cell lane lands VM-grade per-job cells.
- `product.network_topology` `network_reachability` is a global two-argument matrix returning `Bool`, and it answers true for container-runner to BMC-management and container-runner to container-runner. Beyond the values, the shape is wrong: it takes no profile parameter, so it cannot express a per-cell policy at all. **Dissolves when** reachability derives from a profile applied to an attachment.
- `product.network_topology` `NetworkEgressClass` is vocabulary only — four arms, one optional consumer in `gunbc.fleet_container` `NetworkRequirement`, with no rate, byte count, counter, reservation or settlement anywhere. It is **not** a billing axis today and must not be cited as one. **Dissolves when** network appropriation lands.
- `gunbc.runner_spec_from_offer` derives labels from a physical offer — the inverted direction of §4. **Dissolves when** ExecutionClass becomes the projection source.
- `product.compute_fabric` `Shape` carried a thread count only, and `shape_covers` compared only threads; the module is now deleted at the root, so no shape authority stands here at all. The public name `gunbai-8c32g` names two axes it cannot express before reaching storage, isolation or network. **Dissolved with** the resource-root replacement cut, which deleted the module rather than growing `Shape`.

## 10. Open decisions (operator)

1. **Entitlement or entitlement-plus-variance-bound** (§7). Decides packing policy, density, and price. Author's recommendation: entitlement first, instrument variance from day one, promise a bound only once measured.
2. **The acceptable boundary for a private alpha**, before the terminal per-cell construction lands. A sequencing question, not an architectural one: the terminal answer is settled, the interim is not. Author's note: stripping fleet authority from an execution host is the cheap move that changes the risk class at every boundary strength, worth doing first regardless.

## 11. Acceptance controls

1. One `JobAvailable` creates exactly one Demand naming the exact external job.
2. No budget or no admissible Offer means `AcquireJobs` is never called.
3. Network and cell readback pass before acquisition; failure refuses rather than proceeding.
4. The cell reaches GitHub and public package registries.
5. The cell cannot reach the operator LAN, any BMC address, the operator tailnet, the host namespace, or another cell — and cannot because no route exists, not because a rule denied it.
6. Two simultaneous jobs receive distinct disks, attachments, JIT configurations and runner identities.
7. `JobStarted` binds the exact external job to the observed runner and cell.
8. Resource and money reservations release only after teardown readback; an unproven teardown holds them.
9. Reusing a cell, disk, attachment or JIT configuration for a second job refuses.
10. Changing a customer network profile to permit management reachability makes admission red.
11. A job exceeding its class maximum runtime is killed with a typed cause carrying declared and elapsed.
12. Adding a supplier changes no customer workflow; adding a second SCM adds a binding, not a core arm.

## 12. What this note does not claim

No code lands from it. No fabric carrier is authorized by it. The GitHub scale-set surface is unverified here and is public preview. No isolation claim is measured. No variance figure exists. The fragmentation analysis is reasoned from job-size skew and ephemerality, not from observed fleet queues — the starvation hazard is currently hypothetical at four hosts and should be measured before draining reservations are built.
