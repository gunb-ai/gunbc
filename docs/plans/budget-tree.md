# Resource budget tree — grounding notes

Carriers: [dsl/extdeps/accounting/budget.dag](../../dsl/extdeps/accounting/budget.dag) (the §3
authority), [dsl/product/budget_tree.dag](../../dsl/product/budget_tree.dag) (the memory
instantiation). Roadmap node: §1 `1-budget-tree`. PR #5582.

Rationale is homed here, not in-file: ctrl#1793 strips `.dag` comments tree-wide, so a
comment-heavy carrier would red main when that wall lands. These are the planning-level
grounding facts the carrier cannot carry; the model itself lives on the carrier (§6).

## 1. Grounded in real accounting (§1 reduce-convention-to-necessity, §3)

Budgeting is a real, well-developed framework, so the tree is grounded in it rather than in a
coined abstraction: `extdeps.accounting.budget` is the §3 authority, anchored to a real
`ExternalAuthority` (`Https en.wikipedia.org/wiki/Budget`), with the upstream's real names —
`Appropriation` ("max amount for a certain expenditure"), `LineItem`, `BudgetBalance`
(`Surplus | Balanced | Deficit`), `BudgetingMethod` (`ZeroBased | Incremental | ActivityBased`).
Two methods are adopted: **zero-based** (admission) and **appropriation-as-ceiling**
(`within_appropriation`). Being a proper anchored module, it does not trip the unrostered-module
anchor-completeness lens.

## 2. One concept over Measure<Q,S> (§2 one-concept-every-scale)

The authority is generic over `Measure<Q, S>`: **money is instantiation #1, memory is #2** (the
product tree binds `Measure<Memory, One>`). A future CPU / thread / energy dimension extends the
same surface — never a parallel budget tree. This is realized in code (the generic
`measure_add` / `measure_le` lifted to `std.measure`), not just documented. The QoS class
vocabulary (`Guaranteed` / `Burstable` / `BestEffort`, the Kubernetes QoS classes derived from
cgroup-v2 `memory.{min,low,max}`) follows the same extdeps-anchor grounding pattern as a named
follow-up.

## 3. The §5 trichotomy — construction vs residue-lens vs handler (never conflated)

A Bool conservation *check* is **validation**, not construction (it concedes the over-commit is
writable). So the three verdicts are kept distinct:

- **construction** — `admit_all` (zero-based "justified & approved") builds a committed set
  provably within appropriation; over-commit is **unwritable on the admission path**. This is the
  real §5 wall.
- **residue lens** — `node_conserves` is an honest Bool lens for **raw literals** that bypass
  admission (the unstructurable residue you cannot forbid structurally). It is *not* a wall.
- **handler** — runtime intent-vs-actual `reconcile` (capacity drop → evict by QoS / reschedule /
  loud-error on an unsatisfiable `Guaranteed`): a runtime fail-closed handler, not a compile wall.

The tree is recursive: a child's `Appropriation` is charged as a `LineItem` at its parent, so
`node_conserves` recurses and divide-once is structural.

## 4. Convergence survey (§3 single authority)

The budget concept was already forked: `realization_width` (`memory_bounded_fit_count`) and the
complexity gate's `EffortBudget` (op-count) are the SAME capped-resource→claims concept over
different measures. They are **convergence candidates** onto `extdeps.accounting.budget` — future
consumers, reported not refactored (the PR stays atomic).

## 5. Actuator dependency (the protective-only boundary)

`admit_all` / `node_conserves` / `reconcile` are MODEL computations: they **decide** budgets, they
do NOT by themselves prevent the kernel OOM. The tree is PROTECTIVE only paired with an
enforcement actuator:

- (a) admission control holding actual-run-count ≤ authored-claim-count (the merry-otter enforced
  cap), **or**
- (b) cgroup `memory.max` caps (operator-fenced).

On the uncapped fleet the physical OOM-killer pre-empts `reconcile`, so **interim** the authored
L1 run-claim count must be the conservative-HIGH ceiling, never an observed sample. This is the
honest §5 boundary: merging the tree is necessary but not sufficient to end the OOMs — the
actuator is the other half.

## 6. Next steps — from model to protective (the execution plan)

The carrier is merged (#5582), but per §3/§5 above the tree is descriptive until a consumer
derives a real decision from it **and** an actuator enforces actual ≤ authored. The path:

**consumer-0 — instantiate the real fleet tree.** Author the srv1 budget tree as a `data` instance
(root = host RAM ~125 GiB → fixed overhead → R concurrent runs → per-run rustc `N` / `.dag`-floor
spawn-width) with appropriations set from **measured** peaks (#5574 leaf-cgroup: per-run ~24 GiB
pessimistic, host ~125 GiB), not guesses. Each consumer **imports** its parent allocation
(divide-once); none re-divides host RAM. neat-dove owns the seam spec.

**Track A — `.dag`-floor spawn-width (in-tree, no fenced apply).** Convert
`std.realization_width.memory_aware_spawn_width` to take its memory budget from the tree's L2
allocation instead of an independently-divided host budget. Payoff: an over-wide spawn that would
OOM the floor becomes a **compile error** (the admission wall made real at the floor) and one of
the three §3 forks dissolves. Fully landable without touching the live fleet — the immediate step.

**Track B — CI run-concurrency R (the stern-otter OOM cure; needs the actuator).** Derive `R`
(runs/host) and `N` (compile-jobs) from L0/L1 divide-once (placement #5559 / compile-jobs #5546 as
the consumer leaves). This PREVENTS OOM only paired with the §5 actuator — an **operator decision**:
(a) admission control (enforced `R` cap; don't admit run R+1) or (b) cgroup `memory.max` on the
srv1 runners (live-fleet apply, fenced). Until one lands, `R` stays conservative-HIGH.

**Convergence (§3).** As each consumer imports its allocation, the forked budgets
(`realization_width`, complexity `EffortBudget`) collapse onto `extdeps.accounting.budget`; the
roadmap then re-homes #5444 / #5559 / #5546 as edge-children of `1-budget-tree` (reflecting reality,
not aspiration).
