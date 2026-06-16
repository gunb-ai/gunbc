# Design: Policy-Driven Provider Selection — the coercion flip applied to compute (fabric brief C)

> **Status: DESIGN — map, not territory.** Brief C of the compute-fabric set, and the most
> daglang-native piece: *declare a request, derive the provider, refuse if none qualifies.*
> This is `find_witness` over a new candidate domain — not an analogy: the same fold, the
> same closedness invariant, the same fail-closed multiplicity discipline, extended at a seam
> that already exists for exactly this purpose.

## 1. Problem

Given a `ComputeRequest` (A), the declared provider set (B), and a **Policy** (cost-
minimizing, latency-bounded, locality-preferring), select the provider whose contract
satisfies the request *and* is best under the policy — or refuse with a located,
per-candidate explanation. The bets that make this worth doing in daglang instead of Go
policy code: the selection is **derived from declared facts** (no per-provider branches),
**decidable** (closed candidates, total declared order — a verdict, not a heuristic), and
**fail-closed** ("safety is what it refuses": nothing runs if nothing qualifies).

## 2. What already exists (M9 DFS — the seam is already in the tree)

| Concept | Where | Use here |
|---|---|---|
| `find_witness` fold: closed `CandidateSet`, preservation predicate, fail-closed 0/≥2 | `src/v2/std/find_witness.dag` | the selection engine, reused as-is |
| **`MultiplicityPolicy = UniqueOnly \| TargetSelection { policy }`** and **`TargetSelectionPolicy = TargetDeclaredPriority { priority: Node } \| UserSelected { selection }`** | `find_witness.dag:62-67` | **the selection seam, reused with its existing semantics unchanged** — both arms mean "this exact candidate" (`find_witness_realized` routes either into `resolve_selected_candidate` as `selection:`). C does **not** overload `priority` with an ordering meaning (review r3384872268 — one carrier with two type-indistinguishable meanings violates P2); the ordering lives entirely in the domain wrapper (§4.2), which hands `find_witness` the already-computed argmax as the selected candidate |
| Domain-wrapper precedent: `constraints.dag` = "canonical source grounding via find_witness + UniqueOnly (T-9 solve_constraints wrapper)" | `src/v2/std/constraints.dag` | the pattern to copy: a thin domain module wrapping the shared fold — `compute_select` is the compute twin of `solve_constraints` |
| Interval/ordering vocabulary | `v2.std.integer` interval specs; `design-value-set-lattice.md` | capacity facts are dimensioned intervals; satisfaction tests the **hard minimum** (`available ≥ request.min`, §3 — not whole-interval containment, per review r3385120097) |
| Effect partition | `src/v2/std/effects.dag` | obligation discharge (preemptible ⇒ re-runnable) is a structural check on workload effect facts |
| Rejection-priority / best-rejection fold | `find_witness.dag:291-320` | the refusal-report mechanism generalizes from "keep best rejection" to "keep all, located" (§4.3) |

**Substrate target (P1):** `src/v2/std/compute_select.dag` — the satisfies-predicate, the
policy carriers, and the `compute_select` wrapper. The `find_witness` core is **untouched**
(no fifth fold variant; the operator's fewer-variants ruling and dep-graph Q2 both bind
here).

## 3. The relation, precisely

```
compute_select(request, providers: CandidateSet, policy) -> Outcome<SelectionResult>
```

1. **Satisfaction predicate** (the preservation predicate for this domain), decomposed
   field-by-field over A's shape — every clause decidable, every failure located:
   - capacity: provider available capacity **meets the hard minimum** of the requested
     range, per dimension — `provider.available ≥ request.min`. The range's upper end is
     **max-useful, an allocation preference, never an eligibility bar** (review
     r3385120097: requiring containment of the whole interval would refuse a provider that
     satisfies the requirement — a P1 modal-force error). The **granted allocation**
     `min(provider.available, request.max_useful)` is computed here and carried in
     `SelectionResult` per dimension — the dispatcher (D) needs the grant anyway, and
     "can grant more of the useful range" is available to policies as an *objective*
     (§3.2), cleanly separated from satisfaction;
   - capabilities: required ⊆ offered, with a per-kind rule for parameterized entries
     (gpu class compatibility is a declared fact table, not string matching);
   - constraints: platform/residency facts compatible (equality/membership over declared
     vocabularies);
   - **obligation discharge**: every obligation the provider's availability class imposes is
     discharged by the request's workload facts — `Preemptible` demands the effect shape
     admit re-execution; a `IsBreaking` workload without checkpoint facts fails *this
     clause*, with the obligation named.
2. **Policy** = a **declared total preference ordering** over satisfying providers: an
   objective projection (a dimensioned quantity read from provider facts — cost rate,
   expected latency, locality distance) plus a direction. Composite policies are
   lexicographic lists of such orderings (the same lexicographic discipline as termination
   proofs — order by first objective, tie-break by next).
3. **Selection** = the wrapper folds over the closed candidate list keeping the policy-max
   among satisfiers — **argmax-uniqueness under the declared order**: exactly one maximum ⇒
   hand that candidate to `find_witness` as the selection (§4.2) and return the
   `SelectionResult` with its witness; zero satisfiers ⇒ refuse; **two-way tie ⇒ refuse as
   ambiguous** (P4 determinism — a tie is a policy underspecification surfaced to the
   author, never coin-flipped; the fix is one more lexicographic level, declared).

### 4.2 Where it plugs in (and what it deliberately does not change)

`compute_select` owns the ordering end-to-end as its **own typed carrier**
(`ComputePolicy`: objective projection + direction + lexicographic tail — a compute-domain
type, not a `find_witness` type). The flow: filter satisfiers → fold for lexicographic
argmax (ties refuse) → invoke `find_witness` with `TargetSelection { UserSelected {
selection: argmax } }`, which re-validates the predicate on the chosen candidate and mints
the witness + closedness result through the shared discipline. **`find_witness` and its
`TargetSelectionPolicy` carriers are untouched and keep their existing exact-candidate
semantics** — no second meaning is smuggled into `TargetDeclaredPriority.priority`
(review r3384872268), no new variant, no fold variant. The shared fold's closedness
invariant carries unchanged: **providers are
never generated, only declared** — the moment anything synthesizes a candidate (spin up a
new VM shape to fit), it has left the decidable fragment; that capability, if ever wanted,
is a different relation with its own design.

### 4.3 Refusal is a report, not a boolean

On zero satisfiers, the result carries a **per-candidate located failure**: provider X
failed capacity(memory), provider Y failed obligation(preemptible-re-execution), provider Z
failed residency. This is the generalization of `find_witness`'s best-rejection fold from
"keep one" to "keep all, each located" — and it is the fabric's product surface ("why can't
I have compute?" answered structurally). On ambiguity, the report names the tied providers
and the exhausted ordering. DB-1 discipline: typed carriers, no diagnostic prose-parsing.

## 5. Decidability (the carve)

Finite closed candidate list × per-field decidable clauses × fold for lexicographic argmax
under a declared total order = a decision procedure that always terminates with a verdict.
No solver, no scoring heuristics, no float-weighted utility blending (a "score" that mixes
incommensurable dimensions is exactly the ungrounded-heuristic shape P1 forbids —
lexicographic declared orderings keep every comparison inside one dimension's algebra).

## 6. Consumers and the A+B+C slice (E-10) — prototypable now

All machinery exists (`find_witness`, intervals, effects, claims runner), so the slice is
executable without any runtime work:

1. `std/compute_request.dag` + `std/compute_provider.dag` + `std/compute_select.dag`;
   `extdeps/compute/{homelab,gcp}.dag` real instances (B §6).
2. `TestClaim`s under `src/v2/test/claim/compute_select/`, run via `--claim-run`:
   - **green — derive:** cost-minimizing policy selects the homelab over GCP for a small
     CPU request, with the selection witness carried;
   - **green — obligation discharge:** an idempotent workload accepts the preemptible GCP
     arm; the witness records the discharged obligation;
   - **red — refuse (no candidate):** a GPU-class request neither provider offers refuses
     with both per-provider reasons located;
   - **red — refuse (obligation):** a breaking workload against preemptible-only capacity
     refuses naming the undischarged obligation;
   - **red — refuse (ambiguous):** two identical fixture providers under a single-level
     policy tie and refuse — the discriminating case proving ties never resolve silently.
3. This slice is the consumer for A and B (their docs bind to it); D consumes
   `SelectionResult` later and is explicitly **not** needed for the slice to be real.

## 7. Open questions

- **Q-PS1 — objective vocabulary wave 1.** Propose: cost-rate and a locality preference;
  latency-bounded needs a latency *fact* on providers that B wave 1 doesn't declare —
  add the objective only with its fact (E-6 field-with-consumer, applied in reverse).
- **Q-PS2 — multi-request placement.** Selecting for N requests jointly (bin-packing) is a
  different, harder relation — explicitly out of wave 1; one request, one selection.
  Flagged so nobody backs into an undeclared scheduler inside the selector.
- **Q-PS3 — does `SelectionReport` belong in `std/verdict.dag`'s family?** Likely yes
  (M9 — check at implementation; the report is verdict-shaped data).

## 8. Non-goals

No candidate generation/autoscaling, no utility scoring, no online re-selection loops
(D's re-dispatch policy owns that, explicitly), no change to `find_witness` core or the
coercion rule vocabulary.
