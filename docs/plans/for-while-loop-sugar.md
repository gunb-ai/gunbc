# for / while — frontend sugar over Loop, one canonical meaning per spelling

**Status:** design brief for the contributor who implements `for` and `while` in the v2 `.dag` frontend. Nothing here is implemented. **Consumer of this document:** that contributor. **Authority for every judgement about a change:** `DESIGN.md`. §12 below lists the sections of it you need, in the order you will need them.

The goal is small on purpose. `for` and `while` add **no** new core behavior, no new node kind, no new evaluator path and no new emitter path. They are ergonomic spellings in the surface grammar, and each one lowers to the existing `Loop` behavior. If a spelling cannot be given exactly one such meaning, it is refused, and the refusal names the construct the author should write instead.

This revision follows a review of the first draft. The review found that the draft lowered `for` through `list_map`, which reintroduces the very dependency chain the design exists to avoid (§3.1), and it moved long-lived services out of `while` altogether (§6). Every claim below names the module and symbol it rests on (DESIGN §3, *cite the symbol, not the position*), so you can check it before you build on it.

## 1. The rule

> **Surface syntax chooses one canonical meaning. Dependency analysis may change how that meaning is scheduled; it may never reinterpret the source.**

So there is no optimizer that guesses whether a given `for` *meant* a map or a fold. A `for` always means one thing, a `fold` always means another, and a `while` always means a state transition. When the source does not determine one of those meanings, the compiler refuses the program and says which explicit construct to write.

What makes the rule cheap to hold here is that the language has **no mutation**. A loop body cannot write a variable the next iteration reads, so an iteration can reach the next one by exactly two routes: a **declared carrier** (the `^loop_carrier_edge` of a `Loop` node, at most one per loop by `v2.std.node` `loop_behavior_edges_conform`), or an **effect**. With neither, the iterations are independent, and that is structural (DESIGN §4b rung 4): a hidden dependence has no constructor.

### 1.1 A variable is not a dependency

**A variable declaration does not establish loop-carried dependence. Consuming a value a previous iteration produced does.** In `for i in 0..<n { work(i) }` each `i` is derivable from the range alone, so it is a *coordinate*, not state, and no edge runs from iteration *i* to *i+1*. A carrier exists only when the next state is defined from the previous one: `s1 = next(s0)`, `s2 = next(s1)`.

Real dependencies between members still count, and they come from data, never from position. If `job_b.input` names an artifact `job_a` produces, the data reference creates the edge `job_a -> job_b`. Their being adjacent in `jobs` creates nothing. The loop exposes a repeated shape; the graph expresses causality.

### 1.2 Zero ambiguity means one graph, not one interleaving

If `A -> C` and `B -> C`, the program is unambiguous even though `A` and `B` may run in either order or at once: `C` waits for both, neither waits for the other, and no observable result may depend on which finishes first. Demanding one physical order would mean adding an edge between `A` and `B`, which is exactly the false dependency this design removes. So the contract fixes the **permitted execution relation** completely and leaves irrelevant order unspecified, and it requires every *observation* (values, diagnostics, receipts) to be the same under every permitted order (§7).

## 2. The forms

The spellings are proposals; the grammar row decides them (§12, Q2). The *meanings* are the design.

| surface form | canonical meaning | carrier | termination |
| --- | --- | --- | --- |
| `for x in xs { e }` | finite, pointwise iteration: a `Loop` over `xs` with a member template `x => e`, results in domain order | **none** | the domain is finite (`^dag_surface_fold_iteration_measure`, `Strict` in `v2.std.cardinality` `measure_descent_fact_registry`) |
| `fold(xs, init: i, f: step)` | ordered state transition over `xs` (exists today) | `acc` | same fold-iteration measure |
| `while s = init while cond(s) decreases m(s) { next(s) }` | repeated state transition while a `Bool` guard holds | `s` | a **proven** strictly decreasing measure (§5), or refused |
| `repeat s = init up_to: N { step(s) }` | the same transition under an explicit finite step budget | `s` | the budget `N`, written at the site; exhaustion is a typed outcome (§6) |

And three spellings that are **refused**, each with a diagnostic that names the alternatives:

| refused spelling | why | write instead |
| --- | --- | --- |
| a `for` body that reads a previous iteration's result | that is a recurrence, and `for` never manufactures one | `fold`, or a stateful `while` |
| `while (xs)`, i.e. a collection as a condition | it has three incompatible meanings: each member independently, consume head then pass the tail, or a worklist that can grow. No collection truthiness | `for`, `fold`, or a `while` over an explicit worklist state |
| `while (true)` | it claims unbounded computation, which the substrate cannot express, and a hidden large counter would misreport what the program does (§6) | `repeat ... up_to: N`, or a lifecycle construct for services (§6.3) |

Worked example.

```
// independent: no iteration can see another
fn doubled(xs: List<Int>) -> List<Int> {
  for x in xs { x * 2 }
}

// explicit recurrence: the dependence is the accumulator
fn total(xs: List<Int>) -> Int {
  fold(xs, init: 0, f: (acc, x) => acc + x)
}

// state transition: explicit state, Bool guard, proven measure
fn halvings(n: Int) -> Int {
  (while s = Halving { n: n, count: 0 }
     while s.n > 1
     decreases s.n {
     Halving { n: s.n / 2, count: s.count + 1 }
  }).count
}
```

## 3. The `for` contract

`for` is **finite, pointwise, dependency-transparent iteration**. Conceptually it lowers to:

```
Loop
  domain:          xs
  bound:           CollectionSize(xs)
  carrier:         absent
  member-template: x => e
  result-order:    domain order
```

The keyword contributes no edge from member *i* to member *i+1*. Anything the body really depends on stays in the graph as an ordinary data, effect or resource edge (§1.1).

### 3.1 Do not lower `for` through `list_map`

This is the error the first draft made. `v2.std.algebra` `list_map` is defined by `fold_list_right`, which threads an accumulator, and `v2.compiler.fold_lowering` `fold_call_seam_loop` installs a `^loop_carrier_edge` on every fold-family call it lowers. Lowering `for` to `map(xs, x => e)` today therefore produces a **carrier-bearing** loop: the chain is baked into the graph before any analysis runs, and no later pass may remove it without reinterpreting the source. There are two lawful routes:

1. `for` lowers directly to a `Loop` with **no carrier edge**; or
2. a semantic `map` identity survives lowering as itself and receives its realization later, after demand derivation.

**Establish which one the substrate supports before choosing.** The only carrier-less `Loop` producer today is the `loop N e` form (`v2.extdeps.languages.dag` `dag_grammar_loop_expr_expr`, lowered by `v2.compiler.body_lowering_fold` `body_lower_try_loop_keyword_from_captured`), and it is not established that a carrier-less `Loop` collects one result per member in domain order, in `05_eval` (`eval_loop_node`) or in emission. If it does not, that capability is the first piece of work, and it is work in the one `Loop` path, not a `for`-specific one.

### 3.2 What the first version admits and refuses

| admitted | refused |
| --- | --- |
| a finite `List<T>`, a finite range, or another domain with an established cardinality | a domain with no established cardinality |
| an immutable binder and immutable captures | reading a preceding iteration's result: `for_body_requires_loop_carried_state`, naming `fold` and stateful `while` |
| a pure body | a body with any effect, in the first version (effects arrive with §7's ordering evidence) |
| results assembled by domain position | a result that depends on which iteration finishes first |
| every member runs to completion | `break`, `continue`, early `return`: there is nothing for them to mean in a pointwise form |

The compiler must **not** silently turn a reduction-shaped `for` into a `fold`. That would be surface syntax reinterpreted by analysis, which the rule forbids.

## 4. `fold` stays visibly different

A `fold` is not a parallel loop. It is the construct that **declares a recurrence**: `step(acc0, x0) = acc1`, `step(acc1, x1) = acc2`, and so on. `docs/plans/demand-engine-program.md` says the same: a fold whose step reads the previous accumulator stays a chain, and only established algebraic laws (associativity with identity, and so on) license regrouping it. So a generic `fold` is carrier-bearing and serial, and nothing in this brief changes it.

Two constructs from the review are **not** part of this brief, because the substrate they need does not exist yet, and minting them here would create vocabulary with no consumer (DESIGN §3c):

- **a law-bearing `reduce`** that may run as a tree. It needs a carried, checked associativity law, and today `std.algebra` `Monoid` and `CommutativeMonoid` are field-identical records with no law carried; `std.algebra_law` holds only a refutation receipt for float addition. The law evidence would also have to cover the whole observation contract, not just values: overflow and refusal behavior, diagnostics and their order, effects, provenance and termination. Recognizing the spelling `acc + x` is not evidence.
- **`scan`**, the fold that exposes its prefix states. `std` has no `scan`. It is the same carrier-bearing `Loop` and can be added when a consumer needs it.

## 5. The `while` contract

A pure `while` whose condition can change must have changed something, so `while` is a **state transition**, and the state is written in the syntax rather than inferred from free variables. Whatever the final spelling, these fields are the design:

- an **initial state**. Several pieces of state are packaged into one product record, matching the core's single optional carrier
- a **guard** of type `Bool` over the state: no collection or integer truthiness
- a **transition** returning exactly one next state of the same type
- a **termination measure** proven to decrease strictly. An explicit step budget is the separate `repeat` form (§6), not an escape inside `while`
- an explicit **final result**

It lowers to the same `Loop`, now with a carrier:

```
Loop
  carrier: state
  bound:   remaining(state)
  body:    state =>
             if condition(state) { continue with next_state(state) }
             else                { complete with state }
```

**The measure is proven or the program is refused.** The machinery exists. `v2.std.cardinality` `loop_multiplicity` looks the measure up in `measure_descent_fact_registry`; an unregistered measure is `DescentUnknown` (`std.termination`) and refuses with `^cardinality_descent_not_proven`, per the module's own comment that an unproven measure refuses, never fabricates. `std.computation` `lower_call_pattern` already classifies the updates that descend: subtracting a positive literal (`ArithmeticSubtractCall`) and dividing by a literal greater than one (`ArithmeticDivideCall`) are both `Strict`. The `while` lowering checks that `next(s)` makes the measure descend by such a pattern and emits a `Strict` fact **derived from that check**, never a hand-written registry row per loop. `v2.test.lens_complexity` `while_external_condition` already shows a bound-less `Loop` failing closed; `while` gives that refusal a surface spelling.

**No fallback to a budget.** When a `while` cannot be proven to terminate, quietly giving it a large step budget is DESIGN §5's absorbing fallback: it widens *unknown* into *bounded by something* and hides the missing proof. The author who wants a budget writes `repeat ... up_to: N`, where the budget is visible and its exhaustion is typed.

**No `break` or `continue` in the first version.** They can later become sugar over an explicit transition coproduct (continue with a state, complete with a result, refuse with a cause), never target-language control-flow escapes.

## 6. `while (true)`, `repeat`, and long-lived services

### 6.1 Why `while (true)` is refused rather than bounded

The first draft desugared `while (true)` to a counter of `std.computation` `forever_iteration_bound()` (2^63 − 1) iterations. That is withdrawn. `SizeBound.Forever` and `forever_iteration_bound` are values in the **cost and complexity model**, not the runtime meaning of an authored loop, and making them one would:

- put a bound in the program that appears nowhere in its source, so a change to the sentinel changes runtime semantics invisibly
- prove termination instead of modeling non-completion, and feed a misleading figure into cost and resource demands
- leave cancellation and shutdown unrepresented
- most seriously for this design, turn a server into one sequential computation: `while true { conn = accept(); handle(conn) }` makes each `accept` wait for the previous handler. That is a **false dependency**, the exact thing §1 exists to prevent

So `while (true)` is refused, and the refusal names the two legitimate forms below.

### 6.2 Explicit finite repetition: `repeat`

`repeat s = init up_to: N { step(s) }` is a carrier-bearing `Loop` whose bound is the literal budget `N`, which the existing `loop N e` form (`dag_loop_measure_sym`, already `Strict`) can carry. Exhausting the budget is **not** completion, so the result says which one happened:

```
type RepeatOutcome<S, R>
  = RepeatCompleted { result: R, steps: Int }
  | RepeatBudgetExhausted { last_state: S, steps: Int }
```

Cancellation is not an arm here, because a pure computation has nothing that can cancel it; it belongs to the lifecycle form. A runtime may also impose emergency fuel or a wall-clock limit on any evaluation (`std.evaluation_budget`), but hitting it is reported as non-completion or a resource refusal. That is realization policy, never the program's declared meaning. Grep before settling the variant names: `Completed`, `Cancelled` and `BudgetExhausted` are already taken elsewhere in the corpus, and a colliding bare variant name is resolved by precedence rather than refused (the name-collision class in `gunbc.recurring_failure_mode`).

### 6.3 Long-lived services are a lifecycle, and a separate design

Keeping a socket open is not *doing one computation a very large number of times*. It is a lifecycle: acquire the listener, hold it under a lease, accept events until cancellation or closure, start one demand per accepted connection, run handlers according to their real dependencies (which are none, unless they share data, effects or resources), drain or cancel what is in flight, release the listener, and publish a teardown receipt. Conceptually:

```
serve listener until cancellation {
  on_connection conn { handle(conn) }
}
```

That construct is **out of scope for this brief** and gets its own design. It should build on what exists rather than mint a parallel vocabulary: leases are `std.temporal_effect` `HeldLease`, and the one-shot versus re-driven distinction already appears as `gunbc.host.host_effect` `Drive` = `OneShot | ConvergeLoop`. That type is a host-effect drive mode, not a service lifecycle, but it is evidence the repository already treats lifecycle drive as a different concern from bounded computation. Its natural first consumer is the dashboard's accept loop, which `docs/plans/gunbc-served-dashboard-design.md` runs as a sequential host Rust `std::net::TcpListener` under a declared scaffold.

## 7. Parallelism, and why observations must not race

Whether a loop *may* run in parallel is a fact about its graph. Whether it *does* is a realization choice (DESIGN §3: the dispatch that selects a realization is itself realization, and §3d: a selection over cost). So there is no `parallel for` keyword. A carrier-less `for` with a pure body permits any topological order; a carrier-bearing loop permits only carrier order.

**Every permitted order must produce the same observation.** For a `for`:

- results are assembled in domain order
- diagnostics are ordered by domain position, then by a canonical order within one member
- *first error* means the lowest domain position, not the first worker to report
- a runtime may cancel speculative later work, but that must not change the canonical answer

Effectful iteration follows the same principle for receipts, which is why the first version refuses effects in `for` (§3.2): where effect order is observable and no ordering or commutativity evidence is modeled, the only safe answers are *refuse* or *serialize*, and serializing by source position would quietly promise an order the construct does not mean.

**Classify with the vocabulary that exists.** `v2.lens.common.algebraic_composition` `ParallelismRelation` = `DataDependent | EffectCoupled | BarrierCoupled | UnresolvedCoupling`, consumed by `v2.lens.parallelism` through the `parallelism_classifier` row, classifies coupling edges. It has no positive *independent* arm, and none is added: an independent loop is one with no iteration-crossing edge. `UnresolvedCoupling` is serial, never optimistically parallel (DESIGN §5).

**Claim the graph shape, not the speed-up.** No emitter emits parallel code today. The first slice establishes the correct graph (no carrier, no positional edges) and runs it serially, which is one lawful schedule. What matters now is not baking a serial dependency into the graph that would make a later parallel realization impossible. Until one lands, describe a loop as *parallelizable*, never *parallelized* (DESIGN §4b, rung honesty).

## 8. The existing `EffectPlanStep` neighbor

`v2.std.effect_plan` `EffectPlanStep` already has `For { binder, over, body }` and `While { cond, body }` arms, and `docs/plans/host-effect-orchestration.md` states that body lowering treats For, While, Retry and recursion as one `Loop` sugar and that `While` must carry a measure, with no measure making it unconstructible. Today its `While` has no measure field. The surface `while` and `EffectPlanStep.While` must end with **one** answer to *what makes a while terminate*, the `std.termination` one: reuse it there, or record the divergence and its reason (DESIGN §3b, conformance).

## 9. Where the change lands

Everything is v2. v1 is under `gunbc.v1_maintenance_standing` `v1_seed_standing`: semantics frozen, maintenance only for the v2 self-host program, so adding loop forms to v1 is not admitted. v1 already reserves `for`/`in` and has an unused `for x in coll` path (`src/v1/02_parse.dag` `parse_for` → `ExprForEach`, emitted through the `emit_for_each` arm of `src/v1/05_emit.dag`) with no bound and no termination check, and zero corpus uses. Do not revive it; retiring it is a separate v1 change.

| piece | where, and what to reuse |
| --- | --- |
| keywords (`for`, `in`, `while`, `decreases`, `repeat`, `up_to`) | `v2.extdeps.languages.dag`, next to `dag_keyword_lex_rule(^dag_token_kw_loop, ...)`. Grep first: a keyword removes an identifier from every module. |
| one production per form | `v2.compiler.body_producer_forward`, one `BodyProducerForwardRow` per surface identity (see `body_producer_forward_row_loop_expr`). It is the planned successor to `body_lowering_fold` (DESIGN §4, one grammar read in both directions). If that cutover has not landed, coordinate with its owner rather than adding arms to the table being deleted. |
| `for` lowering | a carrier-less `Loop`, or a surviving `map` identity (§3.1). **Not** `list_map`, and not `fold_call_lowering`. |
| `while` lowering | `lower_loop` (`v2.std.compilers.body_lowering`) with a carrier edge and the measure as the bound edge; the descent check reuses `std.computation` `lower_call_pattern`. |
| `repeat` lowering | the existing `loop N e` path, widened from int-literal operands to a carrier and wrapped in `RepeatOutcome`. |
| the `while (true)` and `while (xs)` refusals | typed, located diagnostics raised where the production is recognized, each naming the alternatives in §2. |
| independence classification | `ParallelismRelation` over iteration-crossing edges (§7). No new relation vocabulary. |

## 10. How this differs from a traditional loop

Hand this table to anyone writing their first loop here. The one-line version: **a `.dag` `for` enumerates a finite domain; it does not assert or manufacture iteration order.** Anything that needs a predecessor's state uses a construct that visibly carries it.

| in C / Python / Java | here | why |
| --- | --- | --- |
| the body assigns to a variable outside the loop | impossible: no mutation. State that crosses iterations is `fold` or a stateful `while` | the dependence is structure, so the scheduler can see it |
| iterations run in order, always | `for` iterations may run in any order or at once; results and diagnostics are still reported in domain order | order is observable only through a carrier or an effect |
| `for` returns nothing; you push into a list | `for x in xs { e }` *is* a value: the list of `e`s | pointwise iteration has a result |
| an index `i` makes the loop sequential | a range index is a coordinate, not state | only consuming a previous iteration's value creates a dependence |
| `while (cond)` needs no termination argument | a `while` needs a proven decreasing measure, or it is refused | execution is bounded and forward (DESIGN §4) |
| `while (xs)` tests a collection | refused: write `for`, `fold`, or a worklist `while` | three meanings, no truthiness |
| `while (true)` runs forever | refused: write `repeat ... up_to: N`, or a lifecycle construct for services | a hidden bound misreports; a server loop manufactures a false dependency |
| `break` / `continue` | not in the first version; later, sugar over an explicit transition result | no non-local jumps |
| the compiler may auto-parallelize if it proves safety | the graph *is* the proof; running in parallel is a later realization, and today everything runs serially | *parallelizable* is a fact; *parallelized* is a realization |

## 11. Slices and the evidence each owes

Each slice lands with its consumer and with claims that evaluate a program and can go red (DESIGN §3c, §5). A test that only shows a production *parses* proves nothing.

1. **S1 — pure finite `for`.** First establish, by execution, whether a carrier-less `Loop` collects per-member results in domain order (§3.1); if not, add that to the one `Loop` path. Then the grammar row and lowering. Claims: the lowered node has **no** `^loop_carrier_edge`, and a mutation that inserts one fails the claim; no positional predecessor edge exists between unrelated members; a body whose input references another member's output gets exactly that edge; values equal the hand-written `map` on a non-commutative body; results and diagnostics are identical under reversed and shuffled evaluation orders; a reduction-shaped body refuses with `for_body_requires_loop_carried_state`; an effectful body refuses.
2. **S2 — stateful `while`.** Claims: halving and count-down loops are admitted with `Strict` derived from the descent check; `s + 1` and an unchanged `s` refuse with `^cardinality_descent_not_proven`, naming the measure; a non-`Bool` guard refuses; hand-stamping `Strict` must make the refusal claim fail; the lowered node has exactly one carrier.
3. **S3 — `repeat` and the two refusals.** Claims: `repeat` returns `RepeatBudgetExhausted` at its budget and `RepeatCompleted` otherwise, tested with a **small supplied budget** (DESIGN §3, a witness discriminates at one interface), paired with one claim on the real lowering; `while (true)` and `while (xs)` each refuse, and the diagnostic names the alternatives.
4. **S4 — conformance with `EffectPlanStep`** (§8).

**Not in this brief, each gated on a missing capability:** law-bearing `reduce` (gated on a carried associativity law covering the full observation contract), `scan` (gated on a consumer), effects in `for` (gated on modeled ordering or commutativity evidence), parallel emission (gated on S1, with its own serial-versus-parallel equality claims), and the service lifecycle construct (its own design, §6.3).

## 12. Open questions, and what to read in DESIGN.md

- **Q1 — the carrier-less `Loop`.** Does it already collect per-member results in domain order? The answer decides §3.1's route and the size of S1.
- **Q2 — head spellings.** `while s = init while cond decreases m` and `repeat s = init up_to: N` are proposals. Whatever you choose must make the carrier impossible to miss, because the carrier is the one thing that makes a loop serial.
- **Q3 — outcome variant names.** Grep the corpus before settling them (§6.2).

| before… | read |
| --- | --- |
| the first grammar row | §4, *the closed, grounded substrate*: six behaviors, sugar adds no power, one grammar read in both directions |
| writing any refusal | §5, *fail-closed*: the absorbing fallback and the workaround are the two traps this design is most exposed to |
| claiming a loop is parallel | §4b rung honesty, and §3d, *selection precedes convergence* |
| adding `reduce` or `scan` | §3c, consumption, and §4d, epistemic humility: a spelling is not a law |
| touching `EffectPlanStep` | §3b, conformance |
| your first claim | §3, *a witness discriminates at one interface* |

## Dissolution trigger (DESIGN §6)

Delete this brief when slices S1 through S4 are landed and green on the required floor: the for, while and repeat productions exist as body_producer_forward rows, each lowering and each refusal is exercised by a discriminating claim, and EffectPlanStep.While shares the std.termination measure. At that point the grammar rows and their claims are the authority for what the loop forms mean, and this document, whose only job was to hand off the design, would be a second statement of it.
