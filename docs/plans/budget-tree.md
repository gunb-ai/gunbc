# Resource budget tree — grounding notes

Carrier: [dsl/product/budget_tree.dag](../../dsl/product/budget_tree.dag). Roadmap node: §1 `1-budget-tree`.

Rationale is homed here, not in-file: ctrl#1793 strips `.dag` comments tree-wide
(`dag_comment_wall_test.dag` holds the incoming wall), so a comment-heavy carrier would
red main when that lands. These are the planning-level grounding facts the carrier cannot
carry; the model itself lives on the carrier (§6).

## 1. Measure-generalization fence (realized in code)

`node_conserves` is `Σ child.measure ≤ parent.measure` over any `Measure<Q, S>` (via the
generic `measure_le` / `measure_add` added to `std.measure`); `ByteSize` is instantiation
#1. A future CPU / thread / energy dimension **extends this same surface**, never a parallel
`CpuBudgetTree` — the §3 single-authority fork is fenced by construction, not by a note.

## 2. QoS class grounding (§3 citation)

`Guaranteed` / `Burstable` / `BestEffort` are the Kubernetes QoS class names, themselves
derived from cgroup-v2 memory protection (`memory.min` = guaranteed floor, `memory.low` =
best-effort-protected, `memory.max` = hard limit). The trichotomy is the real upstream
taxonomy, not a from-scratch 3-way. The extdeps anchor relocation (new module +
`external_authority_anchor` row + allowlist) is a **named follow-up**, deferred to keep this
PR atomic and off the unrostered-module red-surface.

## 3. Actuator dependency (the protective-only boundary)

`node_conserves` + `reconcile` are MODEL computations: they **decide** budgets, they do NOT
by themselves prevent the kernel OOM. The tree is PROTECTIVE only paired with an enforcement
actuator:

- (a) admission control holding actual-run-count ≤ authored-claim-count (the merry-otter
  enforced cap), **or**
- (b) cgroup `memory.max` caps (operator-fenced).

On the uncapped fleet the physical OOM-killer pre-empts `reconcile`, so **interim** the
authored L1 run-claim count must be the conservative-HIGH ceiling, never an observed sample.
This is the honest §5 boundary: merging the tree is necessary but not sufficient to end the
OOMs — the actuator is the other half.
