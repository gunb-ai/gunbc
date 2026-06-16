# Design: Compute Providers — modeled backends with behavioral contracts (fabric brief B)

> **Status: DESIGN — map, not territory.** Brief B of the compute-fabric set. Designable
> now; lands inside the A+B+C slice (`design-compute-policy-selection.md` §6).
>
> **Out-of-clone grounding, marked honestly:** the operator reports gunb.ai already has
> ExtDeps-with-behavioral-contracts in Go (`tools/extdeps`) and provider content in
> `tools/infra/dag` / `gcp-config` / `OaaS_v2`. None of that is reachable from this session
> (repo scope is gunbc only), so this design grounds the *shape* in the gunbc substrate and
> cites the gunb.ai-side convergence as operator-supplied; the convergence claims should be
> re-verified the first time that repo is in a session's scope.

## 1. Problem

Each compute backend — the homelab box, a GCP MIG, anything later — becomes a **modeled
provider**: an extdeps fact-bundle declaring what it offers (capacity, capabilities,
availability class, cost) and what it **demands in return** (obligations, e.g. "preemptible
⇒ the workload must be safely re-runnable"). Selection (C) then reads declared facts; no
provider-specific code path exists anywhere in the selection logic — the same model-local /
derive-global move as language targets.

## 2. What already exists (M9 DFS)

| Concept | Where | Use here |
|---|---|---|
| The concept/instance layer split: concept in `std/`, per-target instances in `extdeps/` | `std/target_model.dag` vs `extdeps/languages/*.dag` (P2 layer DAG) | `ComputeProvider` concept in `std/`; `extdeps/compute/homelab.dag`, `extdeps/compute/gcp.dag` instances |
| Fact-bundle + algebra-inhabitance declaration pattern | extdeps languages; `model_core.dag` | provider files are fact-bundles, same construction discipline (no hollow aliases — a provider that drops its backend's spec facts fails the fact-density gate like any other bundle) |
| Effect partition `IsIdempotent \| IsBreaking` | `src/v2/std/effects.dag` | the **obligation mechanism**: a preemptible provider's obligation is discharged by the *workload's* declared effect facts (A §3), checked structurally — not by a "checkpointable: true" flag on faith |
| Platform/network vocabulary | `std/platform.dag`, `std/network.dag` | locality/residency facts on the provider side |
| Dimensioned quantities + intervals | as in brief A | capacity and cost-rate facts are typed quantities; capacity is an interval (free capacity may be a range when shared) |
| Host-run receipts (`ExecutionEvidence` host arm) | `src/v2/std/host_run.dag` | the *future* enforcement seam: declared contract vs observed behavior reconciles through dispatch receipts (D), not through trust |

**Substrate target (P1):** `src/v2/std/compute_provider.dag` (concept: provider fact-bundle
shape, availability classes, obligation carriers) + `src/v2/extdeps/compute/` instances. No
substrate primitives.

## 3. The shape

A provider instance declares, all as typed facts:

- **capacity** — per-dimension intervals (cores, memory, gpu inventory by class, storage).
- **capabilities** — the offered side of A's closed capability vocabulary (same coproduct,
  offered vs required — one vocabulary, two roles; never two vocabularies).
- **availability class** — closed coproduct, wave 1: `OnDemand | Preemptible | Scheduled`
  (Step 2: genuine alternatives). Each class carries its **obligation set**.
- **obligations** — what accepting this provider demands of the workload. The wave-1
  load-bearing one: `Preemptible ⇒ workload effect shape admits re-execution` — discharged
  structurally against A's effect facts (`IsIdempotent`, or breaking-with-checkpoint
  facts). An obligation nobody can discharge is a refusal in C, with the obligation as the
  located reason.
- **cost facts** — rate per dimensioned unit (the cost lens's `Dimension` discipline; no
  bare floats). C's cost-minimizing policy orders by these.
- **transport facts** — how dispatch reaches it (`SshExec`/`Docker` for the homelab; MIG
  semantics for GCP). Consumed only by D; selection never branches on transport.

## 4. The contract boundary, stated honestly (this is the design's hard edge)

A provider bundle declares facts about **external reality the model does not control** —
the P1 closed-system argument stops at this boundary (compute hardware is an open system:
the homelab can be unplugged). So:

- The declared contract is the **authoring-time authority** — selection (C) reasons over it
  totally and decidably.
- **Divergence between declaration and reality is detected at dispatch, fail-closed** (D):
  a dispatch receipt (`host_run` evidence) that contradicts a declared fact (no GPU found;
  capacity exhausted) is a typed dispatch failure attributed to the *provider model*, never
  silently absorbed by retrying elsewhere — re-selection after a contract violation is an
  explicit policy step (D §"no implicit re-execution", inheriting INVARIANTS
  [P2 host-process boundary] (d)).
- No liveness/health modeling in wave 1: availability class is a static fact; dynamic
  capacity tracking is a later wave with its own consumer (likely D's receipts feeding back
  — flagged, not designed).

## 5. Convergence with gunb.ai (the unify-vs-greenfield input)

This design is written to make **unify** the natural outcome: the provider model is the
declarative front half of what `OaaS_v2`'s runner already executes — daglang declares and
derives; the existing runner remains the dispatch substrate behind D's bridge. Greenfield
would rebuild a working runner to get facts we can declare today — the wrong direction
under P5 (progress is dissolution, not duplication). The Go `tools/extdeps` behavioral
contracts, per the operator, carry the same content this design types; when that repo is in
scope, the convergence move is to **generate or check one from the other**, not to maintain
both by hand (no parallel authority).

## 6. Consumers and slice (E-10)

Lands inside the A+B+C slice: two real instances — `homelab.dag` (the prototype target:
SSH/Docker capabilities, on-demand, real capacity numbers) and `gcp.dag` (MIG-shaped,
preemptible arm with the re-runnability obligation). Both are fixtures for C's claims and
become D's first dispatch targets unchanged.

## 7. Open questions

- **Q-B1 — capacity as shared/dynamic.** Wave 1 declares static capacity; concurrent
  requests against one provider need a reservation story — defer to D/runtime design, do
  not model speculatively.
- **Q-B2 — provider identity.** Instances are module-level data (like target models); if
  multiple homelab boxes appear, identity is per-bundle — naming/brand discipline applies
  (each box is its own bundle, not a list entry with a string name).

## 8. Non-goals

Health checking, autoscaling, billing, secrets/credentials modeling (transport facts name
*mechanisms*; credential material never enters the model), and any selection logic (C's).
