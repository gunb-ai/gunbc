# Design: Compute Dispatch — running the selected work (fabric brief D, gated)

> **Status: DESIGN — gated dependency statement + bridge contract.** Brief D of the
> compute-fabric set. A/B/C are pure model and prototypable now; D is where the fabric
> touches a runtime, so most of D is **deliberately deferred to the runtime-architecture
> design** (open decision #4) — this doc fixes the boundary, the bridge, and the bridge's
> dissolution trigger, and enumerates what the runtime design must answer for D. It does
> not design execution semantics.

## 1. The pipeline position

`ComputeRequest` (A) → `compute_select` (C) → `SelectionResult` → **dispatch**: bind the
work to the selected provider's transport facts (B) and run it, producing typed receipts.
Pieces already landed or in flight:

| Piece | Where | State |
|---|---|---|
| Runnable-frontier scheduler (`RunnableFrontier`, `SchedulerPlan`, readiness folds) | `src/v2/workflow/scheduler.dag` (#4566) | landed — decides *what is ready*, provider-agnostic |
| Executor batching | #4603 | in flight (rebasing) — decides *in what order/groups* |
| Host-run receipts (`ExecutionEvidence` host arm, `EmitHostRunReceipt`) | `src/v2/std/host_run.dag`; ROADMAP T-22 bridge row | landed receipt carriers — dispatch reuses these, not a new receipt family |
| **Dispatcher** — fire work on the chosen backend via its transport facts | nowhere | **the gap D names** |

## 2. The bridge (the operator-proposed shape, made contractual)

Wave 1 dispatch rides the existing execution substrate instead of waiting for the v2
runtime: **daglang emits the selected task graph; the `OaaS_v2` runner executes it.** This
is the unify direction (B §5) and it rides v2's working emission — it does **not** need
COMPREP. Per P5, a bridge is only legitimate with its receipts and trigger fixed up front:

- **Contract:** the bridge consumes `SelectionResult` + the provider's transport facts and
  emits the runner's task-graph input (proto/JSON at the boundary — a boundary artifact,
  not a parallel IR authority). Results return as `host_run`-family receipts attributed to
  the request, so claims can assert on dispatch outcomes with the same typed-verdict
  discipline as everything else.
- **Boundary discipline inherited verbatim** (INVARIANTS P2 host-process boundary (a)–(e)):
  typed outcomes not string probes; setup failure ≠ logical exit; **no implicit
  re-execution** — a failed or contract-divergent dispatch (B §4) surfaces fail-closed, and
  re-selection (go ask C again, excluding the offender) is an **explicit, bounded, declared
  policy step**, never a hidden retry.
- **Dissolution trigger:** the bridge deletes when the runtime-architecture design's native
  dispatch path executes the same task graph with the same receipts — tracked the same way
  as ROADMAP's T-22 eval-host bridge row (and it should land *as* a row in that table).
- **Out-of-clone honesty:** `OaaS_v2` is not reachable from this session; the task-graph
  input format is operator-supplied grounding to be pinned (schema cited in the bridge PR)
  when that repo is in scope.

## 3. What the runtime-architecture design must answer for D (its input spec)

D is the first concrete consumer of decision #4's design; these are the questions it needs
answered, so the runtime design can be checked against a real consumer rather than written
in the abstract (the seesaw discipline):

1. **Effects/IO model at the dispatch boundary** — how a dispatched workload's declared
   effect shape (A's workload facts) becomes an enforced contract rather than a description:
   what the runtime checks before/around execution, and what receipt proves it.
2. **Preemption semantics** — what `Preemptible` means operationally: checkpoint/resume
   hooks, re-execution windows, and how the obligation discharged at selection time (C §3)
   is *kept honest* at runtime.
3. **Receipt taxonomy** — whether `host_run.dag`'s evidence arms suffice for remote
   dispatch (exit/setup/divergence/preemption) or need bounded extension — extension lands
   in `std/host_run.dag`, single authority, not a dispatcher-private result enum.
4. **Cancellation and bounded waiting** — dispatch is the first place the system waits on
   external completion; the runtime design owns how waiting stays bounded/fail-closed
   (P4's premise does not get an exception because the loop is "just polling").
5. **Re-selection policy shape** — the explicit declared form of "provider diverged, pick
   the next" (count-bounded, policy-cited, receipt-leaving), so it exists as data, not as a
   retry loop someone writes under pressure.

## 4. Consumers and slice (E-10) — gated, with one honest interim

The real slice is: dispatch the A+B+C homelab selection — a trivial containerized workload
— through the bridge, receipts asserted by a `TestClaim` (green), plus the discriminating
red: a workload whose declared capability the homelab fixture *doesn't* offer never
dispatches (refused at C; the claim proves nothing leaks past a refusal into execution).
This slice is **gated on**: the runtime-architecture design (for §3's answers, even in
bridge form: items 3 and 5 are needed immediately), the `OaaS_v2` schema pin, and a session
scope that can see it. Until those, D correctly stays a dependency statement — building a
dispatcher ahead of them would be the spec-without-execution trap with a network attached.

## 5. Non-goals

Execution semantics (runtime design's), autoscaling/candidate generation (excluded by C's
closedness), secrets handling (transport facts name mechanisms only), multi-request
placement (Q-PS2), and any second receipt vocabulary.
