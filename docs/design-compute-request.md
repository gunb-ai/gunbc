# Design: ComputeRequest — the "give me compute" primitive (fabric brief A)

> **Status: DESIGN — map, not territory.** Brief A of the compute-fabric set (A request ·
> B providers · C policy selection · D dispatch). A is pure declarative model — designable
> and prototypable now. **E-10 note up front:** a request type with nothing that resolves it
> is spec-without-consumer, so **A's slice is consumed by C's slice** — A and C land
> together (B supplies the candidates); A never lands alone.

## 1. Problem

The "give me compute" button: a user (or the fabric itself) declares *what compute is
needed* — resources, duration, capabilities, constraints — as typed `.dag` data, and the
fabric derives a provider for it (C) or refuses with a located diagnostic. Brief A is the
request's shape. The bar: **every field is a declared fact**; nothing free-form, nothing a
provider must parse or guess at. A request that can't be satisfied produces a located
refusal downstream — never a silent fallback (C-8).

## 2. What already exists (M9 DFS)

| Concept | Where | Use here |
|---|---|---|
| Fact-bundle pattern (`PrimitiveFactBundle`, named-edge `Conj`) | `src/v2/std/model_core.dag`, extdeps languages | the request *is* a fact-bundle — same construction, new domain |
| Typed quantities: `Dimension<Unit, Carrier>` family (Cost/Duration precedent) | INVARIANTS P1 Step-1 worked examples; `src/v3/std/dimensions.dag` lineage | cpu/mem/gpu/duration are dimensioned quantities, never bare `Int` |
| Interval/bounds vocabulary | `v2.std.integer` interval specs; `design-value-set-lattice.md` | resource asks are **ranges** (min required / max useful), so satisfaction is interval containment — the same containment shape the value-set lane builds |
| Platform vocabulary (`OperatingSystem`, `Architecture`, `Vendor`) | `src/v2/std/platform.dag` | locality/affinity constraints reference declared platform facts, not strings |
| Network boundary carriers | `src/v2/std/network.dag` | data-residency / locality constraints anchor here |
| Effect partition `IsIdempotent \| IsBreaking` | `src/v2/std/effects.dag` | the workload's declared effect shape — what B's obligations (e.g. preemptible ⇒ re-runnable) are checked against |
| RFC-3339 instant facts | `src/v2/std/datetime.dag` | duration/deadline facts |

**Substrate target (P1):** one new `std/` concept module, `src/v2/std/compute_request.dag`
(the *concept* lives in std; B's provider *instances* live in extdeps — same layer split as
`TargetModel` vs `extdeps/languages/*`). No connective/behavior changes.

## 3. The shape

`ComputeRequest` = a `Conj` fact-bundle with named edges, all typed:

- **resources** — per-dimension asks as **min-required / max-useful pairs** of dimensioned
  quantities: cpu cores, memory, gpu count/class, storage. The two ends carry **different
  modal force, stated at the type** (review r3385120097): the minimum is a hard
  requirement (a provider below it fails satisfaction); the maximum is an allocation
  preference (how much of the surplus is worth granting — **never** an eligibility bar).
  An exact need is min = max; "as much as available up to N" is min = floor, max = N. The
  selection (C) tests providers against the minimum and carries the granted quantity in
  `SelectionResult`.
- **duration** — expected runtime bound + (optional) deadline, as datetime/duration facts.
  **The wait-bound is structurally required at the dispatch boundary** (review
  r3384872272): an ask without a finite duration/timeout fact is representable for
  *analysis and selection*, but the dispatch effect-request constructor **requires the
  bound fact** (the same constructor-requires-the-fact pattern as effect signatures,
  `design-v2-runtime-architecture.md` §4.3/§5.4) — so unbounded work cannot reach a
  waiting handler regardless of how permissive a policy is. P4's bounded-waiting guarantee
  is enforced by construction at the boundary that waits, not by policy goodwill.
- **capabilities** — the *required* capability set (e.g. `Gpu(class)`, `Docker`,
  `Checkpointable` as a property the *workload offers*, network egress). Closed coproduct
  vocabulary in wave 1 (Step 3: substrate-declared — satisfaction must be decidable per
  capability kind; an open string set would forfeit that). Growing the vocabulary is a
  substrate change with a per-kind satisfaction rule, by design.
- **constraints** — locality (platform/region facts), data-residency (network boundary
  facts), co-location/affinity (reference to another request's placement).
- **workload facts** — the effect shape (`effects.dag` partition) and re-runnability facts
  the obligations in B/C consume. The request declares what the workload *is*, so the fabric
  can check what a provider *demands* (preemptible ⇒ idempotent re-execution) structurally
  instead of trusting a flag.

Decision-procedure cites: Step 1 — attaches to fact-bundle + Dimension + interval ancestors
(table above); no new parents coined. Step 2 — capabilities are alternatives per entry
(sum), the request's sections are coordinates (record): both applied. Step 3 — capability
vocabulary substrate-declared (decidability), constraints reference existing platform/
network facts rather than a new label space.

## 4. Refusal posture (the requirement, stated precisely)

The request type itself cannot fail — it is data. The *fail-closed surface* is C's
satisfaction relation, and A's contribution to it is: every field is shaped so
"satisfiable?" is decidable per field (interval containment, capability-set inclusion with
per-kind rules, constraint-fact compatibility). Anything not expressible in the closed
shapes is **not authorable** — there is no free-text escape hatch to smuggle an ungrounded
requirement through (the C-5/C-8 posture applied at the authoring boundary).

## 5. Consumers and slice (E-10)

Consumed by C's selection (`compute_select` claims) — the A+B+C slice in
`design-compute-policy-selection.md` §6 is the single landing; A contributes the request
fixtures (one satisfiable, one GPU-requiring, one residency-constrained). No request field
lands without a satisfaction rule in C reading it (E-6's same-PR-consumer discipline,
field-by-field).

## 6. Open questions

- **Q-A1 — capability vocabulary seed.** Wave-1 closed set: propose `Gpu(class)`, `Docker`,
  `SshExec`, `NetworkEgress`, `Checkpointable-workload`. Operator trims/extends; each entry
  ships with its satisfaction rule.
- **Q-A2 — placement reference shape** for co-location constraints (request-to-request
  reference) — defer to wave 2 unless the homelab prototype needs it (it likely doesn't).

## 7. Non-goals

No pricing/billing model (B carries cost *facts*; accounting is out of scope). No runtime
semantics (D). No open/user-extensible capability strings.
