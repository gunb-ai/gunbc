# for / while — frontend sugar over Loop, one canonical meaning per spelling

**Status:** design brief for the contributor who implements `for` and `while` in the v2 `.dag` frontend. Nothing here is implemented. **Consumer of this document:** that contributor. **Authority for every judgement about a change:** `DESIGN.md`. §12 lists the sections of it you need, in the order you will need them.

`for` and `while` add **no** new behavior to the closed vocabulary of six. Each is an ergonomic spelling that lowers to the one `Loop` behavior. What they do need is a `Loop` that can represent *what it iterates over*, and today it cannot: §2 (S0) fixes that representation first, and every later section is written against it. If a spelling cannot be given exactly one canonical encoding, it is refused, and the refusal names the construct to write instead.

This is the third cut. The first lowered `for` through `list_map`, which reintroduces a dependency chain. The second was held in review because it described a `Loop` the substrate cannot represent, gave the carrier edge two meanings, specified a `repeat` that could not execute, and cited a termination proof path that does not exist. Each of those is corrected here, and the corrections are stated where they apply. Every claim names the module and symbol it rests on (DESIGN §3, *cite the symbol, not the position*).

## 1. The rule

> **Surface syntax chooses one canonical meaning. Dependency analysis may change how that meaning is scheduled; it may never reinterpret the source.**

No optimizer guesses whether a `for` *meant* a map or a fold. A `for` always means pointwise iteration, a `fold` always means an explicit recurrence, and a `while` always means a state transition. When the source does not determine one of those meanings, the program is refused.

### 1.1 What the construct guarantees, and what it does not

The guarantee `for` makes is narrow and structural: **the construct contributes no implicit positional predecessor edge.** Member *i+1* is not ordered after member *i* because they are adjacent in the list. The language has no mutation, so there is no way for a `for` body to write something the next member reads, and a carrier (the one route a loop has for passing state) is simply absent from its encoding.

That is **not** the same as saying the members are independent. Members can still be coupled by data, effects, resources or barriers, which are the relations `v2.lens.common.algebraic_composition` `ParallelismRelation` already distinguishes. Full independence is a *derived* fact: it holds only once the members exist at identity grain, dependency extraction over them is complete, no cross-member edge of any kind was found, and no coupling is left unresolved (§7).

### 1.2 A variable is not a dependency

**Declaring a variable does not create loop-carried dependence. Consuming a value a previous iteration produced does.** A list position is a *coordinate*, derivable on its own, not state. A carrier exists only when the next state is defined from the previous one: `s1 = next(s0)`, `s2 = next(s1)`. Dependencies between members come from data, never from adjacency: if one member's input names an artifact another member produces, that reference is the edge.

### 1.3 Zero ambiguity means one graph, not one interleaving

If `A -> C` and `B -> C`, the program is unambiguous even though `A` and `B` may run in either order or at once. Demanding one physical order would mean inserting an edge between them, which is exactly the false dependency this design removes. So the contract fixes the **permitted execution relation** and requires every *observation* (values, diagnostics, receipts) to be identical under every permitted order (§7).

## 2. S0 — the canonical loop encodings

### 2.1 What exists today

Be precise about the starting point, because the earlier cuts were not:

- **The shape.** `v2.std.node` `loop_behavior_edges_conform`: at least one positional body child, exactly one `^loop_bound_edge`, at most one `^loop_carrier_edge`. There is no domain, no member binder, no member identity and no result-assembly rule.
- **The carrier edge means a binder.** `v2.std.node` `loop_carrier_binder_target` reads it, `v2.compiler.fold_lowering` `fold_call_seam_loop` puts the accumulator's binder atom there, and `v2.lens.complexity_accumulator_copy` consumes it as a symbol. It never holds an initial value.
- **The only source form is unary.** The grammar is `loop <expr>` (`v2.extdeps.languages.dag` `dag_grammar_loop_expr_expr`); `v2.std.compilers.body_lowering` `lower_loop(value, bound)` puts the expression in the one positional slot and the bound is hard-coded to `dag_loop_measure_node()`. Its Rust realization is `loop { break e; }`: one evaluation, one result. There is no `loop N e` form and no budget.
- **No interpreter executes a `Loop`.** `v2.extdeps.runtimes.v2_evaluator` `v2_eval_step_loop` and `v2.program` `program_step_loop` both reject.
- **The fold seam is an analysis seam, not an executable fold.** `fold_call_seam_loop` records the step body, the fold-iteration bound and the accumulator binder. It carries neither the list nor `init`.

### 2.2 The one extension: a domain edge

Pointwise iteration needs something to iterate. No existing structure instantiates a body once per member: `Cardinality` and `Instantiation` are *type* connectives, not computations. So the choice is between a seventh behavior and one more named edge on `Loop`. A seventh behavior would break the closed vocabulary of DESIGN §4, so the answer is the edge:

- **`^loop_domain_edge`** (new, optional, at most one): its target is the value being iterated. Its presence is what makes a `Loop` iterate members rather than evaluate once.

Nothing else is added, and no existing edge changes meaning:

| fact | where it lives |
| --- | --- |
| the domain | `^loop_domain_edge` → the domain value |
| the member binder | the positional body child is a member template, a function node whose parameter binds the member (the same lambda shape the fold seam already carries as its step) |
| member identity | the member's ordinal in the domain's canonical enumeration. In S1 the only admitted domain is `List<T>`, whose ordinal is its index (§3) |
| loop-carried state's binder | `^loop_carrier_edge`, unchanged: a binder, never a value |
| loop-carried state's initial value | an enclosing `Bind` that binds the carrier's binder to the initial value; the `Loop` is that `Bind`'s body |
| the termination measure | `^loop_bound_edge`, unchanged |
| the per-loop termination proof | derived from the node, never stored on it (§5.2) |
| result assembly | a rule fixed by which edges are present (next table), not an edge |

### 2.3 The four encodings

| form | edges present | encoding | result |
| --- | --- | --- | --- |
| existing unary `loop e` | bound; no domain; no carrier | unchanged | the one value of `e`, as today |
| `for x in xs { e }` | domain; bound; no carrier | `Loop { domain: xs, bound: fold-iteration measure, body: x => e }` | the list of member results in domain order |
| `fold(xs, init, step)` | domain; bound; carrier | `Bind { acc := init, body: Loop { domain: xs, carrier: acc, bound, body: (acc, x) => step } }` | the final carrier value |
| stateful `while` | bound; carrier; no domain | `Bind { s := init, body: Loop { carrier: s, bound: measure, body: s => WhileStep } }` (§5) | the value carried by the `WhileComplete` arm |

S0 is the substrate work those encodings need, done once in the one `Loop` path: admit the domain edge in `loop_behavior_edges_conform` and in every consumer of the loop's edges (the hash canonicalization `canonicalize_loop_labeled`, `loop_edge_contributes_to_iteration_fold`, `edge_contributes_to_cost_fold`), and give the `Loop` interpreter step the three executing rules above in place of its rejection. S0 changes no surface syntax, and the unary `loop` keeps its meaning.

**The fold seam migrates later, at its root.** Today's seam is the fold encoding with its domain and initial `Bind` missing. Bringing it to the full encoding is a replacement migration (DESIGN §3): slice S3 moves `fold_call_seam_loop` to the full encoding in one change, together with its consumers. The carrier edge keeps its binder meaning, so `v2.lens.complexity_accumulator_copy` is unaffected. Until S3 lands, the seam stays what it is: an analysis seam whose missing edges are this row's stated gap, not a second meaning.

## 3. S1 — the `for` contract

`for` is **finite, pointwise iteration over a canonically ordered domain**, and in S1 that domain is `List<T>` only.

- **Why only lists.** The promises about result and diagnostic order need an enumeration and an ordinal, and a cardinality supplies only a count: a finite set of five members has no canonical first member. `v2.std.refinement` already distinguishes `StructurallyOrdered` from `StructurallyUnordered`. A list is structurally ordered and its index is its ordinal. A range comes once a range carrier with its own canonical enumeration exists, and any other finite domain needs an ordering witness or an explicit ordering projection.
- **Why not `list_map`.** `v2.std.algebra` `list_map` is defined by `fold_list_right`, which threads an accumulator, and lowering through it or through `fold_call_lowering` installs a carrier. `for` lowers directly to the domain encoding of §2.3.
- **Pure bodies only.** An effectful body is refused in S1, until ordering or commutativity evidence for effects is modeled (§7).

| admitted in S1 | refused in S1 |
| --- | --- |
| a `List<T>` domain | any other domain, until its canonical enumeration exists |
| an immutable member binder and immutable captures | an effectful body |
| every member runs to completion | `break`, `continue`, early `return`: they have nothing to mean in a pointwise form |
| results assembled by list index | (nothing to refuse: see below) |

**There is no reduction-shaped-body diagnostic.** The second cut asked for a `for_body_requires_loop_carried_state` refusal. It is deleted, because its red cannot be authored: a `for` body sees only its member and immutable captures, so it has no name for a previous result, no accumulator, and no way to mutate one. `xs[i - 1]` reads input data and is not a recurrence; `acc = acc + x` is unwritable; an unbound `acc` gets the ordinary binding refusal, not a guess that the author meant a fold. That is the rung-4 result the design wants, and a check for a state with no constructor would be permanently green decoration (DESIGN §4b).

## 4. `fold` stays an explicit recurrence

A `fold` declares a recurrence: `step(acc0, x0) = acc1`, `step(acc1, x1) = acc2`. `docs/plans/demand-engine-program.md` says the same: such a fold stays a chain, and only established algebraic laws license regrouping it. So `fold` is carrier-bearing and ordered, and this brief changes its encoding (S3) but not its meaning.

Two constructs are deliberately **not** in this brief, because the substrate they need does not exist and minting them would add vocabulary with no consumer (DESIGN §3c):

- **a law-bearing `reduce`**, which could run as a tree. It needs a carried, checked associativity law. Today `std.algebra` `Monoid` and `CommutativeMonoid` are field-identical records, and `std.algebra_law` holds only a refutation receipt for float addition. The law evidence must also cover the full observation contract, meaning overflow and refusal behavior, diagnostics and their order, effects, provenance and termination, not just values. The spelling `acc + x` is not evidence.
- **`scan`**, the fold that exposes its prefix states. There is no `scan` in `std`; it is the fold encoding with a different result rule, and it comes when a consumer needs it.

## 5. S2 — the `while` contract

A `while` is a **state transition**, and its state is written in the syntax, not inferred from free variables. The fields:

- an **initial state**. Several pieces of state are one product record, because the core admits one carrier
- a **guard** of type `Bool` over the state. There is no collection or integer truthiness
- a **transition** returning exactly one next state of the same type
- a **measure**, proven to descend (§5.2)
- an explicit **result**

The encoding (§2.3) wraps the guard and the transition into a body that returns an explicit step. The binder is the carrier edge's meaning; the initial value comes from the enclosing `Bind`:

```
type WhileStep<S, R>
  = WhileContinue { state: S }
  | WhileComplete { result: R }

Bind { s := init,
  body: Loop { carrier: s, bound: measure(s),
    body: s => if guard(s) { WhileContinue { state: next(s) } }
               else        { WhileComplete { result: finish(s) } } } }
```

`WhileContinue` and `WhileComplete` were unused names in the corpus as of this writing; grep again before settling them. A colliding bare variant name is resolved by precedence rather than refused (the name-collision class in `gunbc.recurring_failure_mode`), which is why the obvious `Continue` is not used: `v2.std.runtime` `ControlTransfer` already has one.

### 5.1 What exists, and what does not

The second cut claimed that `std.computation` `lower_call_pattern` already classifies a `while` body's updates. It does not. It maps an **already-selected** `CallPattern` to a `LoweringTarget`: it never inspects a transition expression, never connects the pattern to a measure, and its production callers are in the v1 complexity machinery, not the v2 frontend. And `v2.std.cardinality` `loop_multiplicity` takes its proof from `measure_descent_fact_registry`, a static list of three symbol-keyed rows for global measures. An authored measure such as `s.n` is not a registered symbol, so today it is `DescentUnknown` and refuses.

### 5.2 The four joins S2 must build

1. **Recognition** — from the resolved transition expression to a descent pattern. S2 admits exactly: the predecessor of a `v2.std.nat` `Nat` (`Succ { prev } => prev`); `m - k` for a positive integer literal `k`; `m / k` for an integer literal `k > 1`. Anything else is `DescentUnknown`.
2. **Relation** — the recognized update must apply to **the authored measure itself**: the same field path of the carried state that the `^loop_bound_edge` names. A descending expression elsewhere in the body proves nothing.
3. **Well-foundedness** — a strictly smaller `Int` is not termination, since integers descend forever. A `Nat` measure is well-founded by construction (`Zero | Succ`). An `Int` measure is admitted only when the guard implies a floor that keeps the update strict: for `m - k` the guard must imply `m >= k`, and for `m / k` it must imply `m >= 1`. In S2 the admitted guard shapes are literal comparisons on the measure itself (`m > c`, `m >= c`).
4. **Carriage** — one producer, read by every consumer. The producer is a single derivation in `v2.std.cardinality` over the `Loop` node: measure, transition and guard in, `TerminationProof` witness out. `loop_multiplicity` calls it for any loop whose measure is not a registry symbol (the three registry rows keep answering for their global measures). The witness that `v2.compiler.infer` exposes through `inferred_facts_descent` is that same function's answer, never a second computation. The cost and cardinality lenses therefore cannot disagree with inference about one loop.

**No fallback to a budget.** When no proof is found, quietly giving the loop a large step budget is DESIGN §5's absorbing fallback: it widens *unknown* into *bounded by something* and hides the missing proof. The loop is refused with `^cardinality_descent_not_proven`, naming the measure and the transition.

**No `break` or `continue`.** `WhileStep` already is the explicit exit, and early exit is spelled `WhileComplete`.

## 6. What is refused, and what is out of scope

| spelling | disposition | write instead |
| --- | --- | --- |
| `while (xs)`, a collection as a condition | **refused**. It has three incompatible meanings: each member independently, consume the head and carry the tail, or drain a worklist that can grow. No collection truthiness | `for`, `fold`, or a stateful `while` over an explicit worklist state with a proven measure |
| `while (true)` | **refused**. It claims unbounded computation, which the substrate cannot express | a stateful `while` with a proven measure, or, for a service, the lifecycle design below |

### 6.1 Why `while (true)` gets no hidden bound

The first cut desugared it to a counter of `std.computation` `forever_iteration_bound()` (2^63 − 1) iterations. That is withdrawn. `SizeBound.Forever` is a value in the **cost model**, not the runtime meaning of an authored loop. A hidden counter puts a bound in the program that its source never states, proves termination where the author meant non-completion, and leaves cancellation unrepresented. Worst for this design, `while true { conn = accept(); handle(conn) }` makes each `accept` wait for the previous handler: a **false dependency**, the exact thing §1 exists to prevent.

### 6.2 Budgeted repetition is a separate design

The second cut proposed `repeat s = init up_to: N`. It is withdrawn from this brief. As specified it could not execute: no `loop N e` form exists to carry `N`, and a step returning only a next state gives the author no way to finish, so its `RepeatCompleted` arm was uninhabited. A correct version needs an explicit terminal signal (the `WhileStep` shape) and a remaining-budget count inside the carried state: continue with fuel left advances and decrements, continue at zero is budget exhaustion, and complete is completion. That is straightforward once S0 and S2 exist, so it should be designed then, against a named consumer.

### 6.3 Long-lived services are a lifecycle, and a separate design

Keeping a socket open is not *one computation repeated many times*. It is a lifecycle: acquire the listener, hold it under a lease, accept until cancellation or closure, start one demand per connection, run handlers by their real dependencies, drain or cancel what is in flight, release, and publish a teardown receipt. That design should build on what exists: leases are `std.temporal_effect` `HeldLease`, and `gunbc.host.host_effect` `Drive` = `OneShot | ConvergeLoop` already separates re-driven work from one-shot work, though it is a host-effect drive mode and not a service lifecycle. Its natural first consumer is the dashboard accept loop that `docs/plans/gunbc-served-dashboard-design.md` runs as a sequential host Rust `std::net::TcpListener` under a declared scaffold.

## 7. Parallelism, and why observations must not race

Whether a loop *may* run in parallel is a fact about its graph. Whether it *does* is a realization choice (DESIGN §3: the dispatch that selects a realization is itself realization; §3d: a selection over cost). So there is no `parallel for` keyword.

- **In S1's admitted population, cross-member edges are not constructible.** A pure body over a list sees its member and immutable captures, and captures are the same for every member, so no member can reference another member's result. Independence therefore holds for that population, and the claim is scoped to it.
- **Everywhere else, independence is derived** after dependency extraction, and only when no cross-member data, effect, resource or barrier edge was found (§1.1).
- **Unresolved coupling is not admitted to a parallel realization.** It is refused for parallelism, with its diagnostic, the way `v2.lens.parallelism` already flags it (`parallelism_fact_unresolved`) rather than classifying it as any kind of coupling. It is not silently serialized: serializing an unresolved graph is DESIGN §5's absorbing fallback, and it would manufacture an ordering the source never stated.
- **Classify with the vocabulary that exists.** `ParallelismRelation` has no positive *independent* arm, and none is added: independence is the absence of any cross-member relation.

**Every permitted order produces the same observation.** Results are assembled by list index. Diagnostics are ordered by index, then by a canonical order within one member. *First error* means the lowest index, not the first worker to report. A runtime may cancel speculative later work, but that must not change the canonical answer.

**Claim the graph shape, not the speed-up.** No emitter emits parallel code. S1 establishes the graph (no carrier, no positional edges) and executes it in index order, which is one permitted schedule of an independent population, not a fallback. Until a parallel realization lands with its own evidence, describe a loop as *parallelizable*, never *parallelized* (DESIGN §4b, rung honesty).

## 8. The existing `EffectPlanStep` loop forms

`v2.std.effect_plan` `EffectPlanStep` already has `For { binder, over, body }` and `While { cond, body }`, and `docs/plans/host-effect-orchestration.md` says body lowering treats For, While, Retry and recursion as one `Loop` sugar. Both need a disposition (DESIGN §3b, conformance):

- **`EffectPlanStep.For` — a stated divergence.** It orders effectful steps in an orchestration plan; the surface `for` is pure, pointwise and unordered. The same word in two explicitly distinct naming surfaces is legitimate reuse under DESIGN §3, and the reason is recorded here: an effect plan's `For` promises sequencing, and the frontend `for` promises its absence. If the effect plan's `For` is ever lowered onto `Loop`, it lowers to the domain encoding **with** its effects, and it then falls under §7's effect rules. It does not borrow S1's purity.
- **`EffectPlanStep.While` — conform.** It has no measure field today, while the orchestration plan says a `While` without a measure is unconstructible. It must end with the same answer as the surface `while`: a measure whose proof comes from the §5.2 producer. That change is slice S4.

## 9. Where the change lands

Everything is v2. v1 is under `gunbc.v1_maintenance_standing` `v1_seed_standing`: semantics frozen, maintenance only for the v2 self-host program, so loop forms are not added to v1. v1 already reserves `for`/`in` and has an unused `for x in coll` path (`src/v1/02_parse.dag` `parse_for` → `ExprForEach`, emitted through the `emit_for_each` arm of `src/v1/05_emit.dag`) with no bound and no termination check, and zero corpus uses. Do not revive it.

| piece | where |
| --- | --- |
| the domain edge, and every consumer of loop edges (S0) | `v2.std.node` `loop_behavior_edges_conform` and the loop-edge helpers beside it, the hash canonicalization `canonicalize_loop_labeled`, `loop_edge_contributes_to_iteration_fold`, `edge_contributes_to_cost_fold` |
| executing a `Loop` (S0) | the `Loop` interpreter step: `v2.extdeps.runtimes.v2_evaluator` `v2_eval_step_loop` and `v2.program` `program_step_loop`, both of which reject today; and the Rust emission of the three rules |
| keywords `for`, `in`, `while`, `decreases` | `v2.extdeps.languages.dag`, beside `dag_keyword_lex_rule(^dag_token_kw_loop, ...)`. Grep first: a keyword removes an identifier from every module |
| one production per form | `v2.compiler.body_producer_forward`, one `BodyProducerForwardRow` per surface identity (see `body_producer_forward_row_loop_expr`). If its cutover from `body_lowering_fold` has not landed, coordinate with its owner rather than adding arms to the table being deleted |
| `for` lowering (S1) | the domain encoding. Not `list_map`, not `fold_call_lowering` |
| `while` lowering and the proof producer (S2) | `lower_loop` generalized in `v2.std.compilers.body_lowering`, and the derivation in `v2.std.cardinality` of §5.2 |
| fold seam migration (S3) | `v2.compiler.fold_lowering` `fold_call_seam_loop`, with its consumers in the same change |
| the two refusals | typed, located diagnostics where the production is recognized, each naming the alternatives in §6 |

## 10. How this differs from a traditional loop

The one-line version for a newcomer: **a `.dag` `for` enumerates a list; it does not assert or manufacture iteration order.** Anything that needs a predecessor's state uses a construct that visibly carries it.

| in C / Python / Java | here | why |
| --- | --- | --- |
| the body assigns to a variable outside the loop | impossible: no mutation. State that crosses iterations is `fold` or a stateful `while` | the dependence is structure, so the scheduler can see it |
| iterations run in order, always | `for` members may run in any permitted order; results and diagnostics are still reported by index | order is observable only through a carrier, an effect or another real edge |
| `for` returns nothing; you push into a list | `for x in xs { e }` *is* a value: the list of `e`s | pointwise iteration has a result |
| `for` works over any iterable | S1 iterates a `List` only | the order promises need a canonical enumeration, not just a count |
| an index makes the loop sequential | a position is a coordinate, not state | only consuming a previous iteration's value creates a dependence |
| `while (cond)` needs no termination argument | a `while` needs a measure proven to descend on a well-founded domain, or it is refused | execution is bounded and forward (DESIGN §4) |
| `while (xs)` tests a collection | refused | three meanings, no truthiness |
| `while (true)` runs forever | refused; services get a lifecycle construct | a hidden bound misreports, and a server loop manufactures a false dependency |
| `break` / `continue` | a `while` body returns `WhileContinue` or `WhileComplete`; a `for` has neither | no non-local jumps |
| the compiler may auto-parallelize if it proves safety | the graph is the evidence; running in parallel is a later realization, and today everything runs in order | *parallelizable* is a fact; *parallelized* is a realization |

## 11. Slices and the evidence each owes

Each slice lands with its consumer and with claims that evaluate a program and can go red (DESIGN §3c, §5). A test that only shows a production *parses* proves nothing.

1. **S0 — the encodings.** The domain edge, the three executing rules in the one `Loop` interpreter, and every loop-edge consumer. Claims: each of the four encodings in §2.3 is admitted by `loop_behavior_edges_conform` and two domain edges are refused; the node hash distinguishes a loop with a domain edge from one without; built directly as nodes (no surface syntax yet), a domain loop yields its member results in index order, a domain-and-carrier loop yields the fold of a non-associative step (subtraction), and a carrier loop with a `WhileStep` body yields its completing value; the unary `loop e` still yields `e`. Mutation controls: evaluating members in reverse fails the order claim, and folding right-to-left fails the fold claim.
2. **S1 — `for` over `List<T>`.** Claims: the lowered node has a domain edge and **no** carrier edge, and a mutation that inserts one fails the claim; its values equal a hand-built domain loop's; results and diagnostics are identical under reversed and shuffled member evaluation; the first error is the lowest index; an effectful body and a non-`List` domain are each refused with their own diagnostic.
3. **S2 — stateful `while`.** Claims: a `Nat` count-down, an `Int` count-down guarded by `m > 0`, and a halving loop guarded by `n > 1` are admitted, each with a proof from the §5.2 producer; `s + 1`, an unchanged `s`, an `Int` decrement with no guard floor, and a descent on a field other than the measure are each refused with `^cardinality_descent_not_proven`; `loop_multiplicity` and `inferred_facts_descent` return the same proof for the same loop. The mutation control has a real producer to mutate: making the producer answer `Strict` for an unrecognized update must turn the refusal claims red. `while (xs)` and `while (true)` are refused with the alternatives named.
4. **S3 — fold seam migration.** `fold_call_seam_loop` emits the full fold encoding in one change with its consumers; `v2.lens.complexity_accumulator_copy` is unchanged and its claims stay green.
5. **S4 — `EffectPlanStep.While` conformance** (§8).

**Out of scope, each gated on a named capability:** law-bearing `reduce` (a carried associativity law covering the full observation contract), `scan` (a consumer), ranges and other domains (a canonical enumeration), effects in `for` (modeled ordering or commutativity evidence), budgeted repetition (S0 and S2, then its own design, §6.2), parallel emission (S1, then its own serial-versus-parallel equality claims), and the service lifecycle (its own design, §6.3).

## 12. Open questions, and what to read in DESIGN.md

- **Q1 — head spelling of a `while`.** The fields in §5 are fixed; the surface order and keywords are not. Whatever is chosen must make the carried state impossible to miss, because it is the one thing that makes the loop ordered.
- **Q2 — `WhileStep` variant names.** Grep before settling them (§5).
- **Q3 — which measure lands first in S2.** `Nat` is well-founded by construction and needs no guard reasoning; the guarded `Int` forms are more ergonomic. Landing `Nat` first makes the smallest S2.

| before… | read |
| --- | --- |
| S0 | §4, *the closed, grounded substrate* (six behaviors; sugar adds no power), and §3, *replacement migrations cut over at the root* (for S3) |
| writing any refusal | §5, *fail-closed*: the absorbing fallback and the workaround are the traps this design is most exposed to |
| writing a check | §4b: ask whether its red is authorable before writing it |
| claiming a loop is parallel | §4b rung honesty, and §3d, *selection precedes convergence* |
| touching `EffectPlanStep` | §3b, conformance, and §3 on one name across distinct scopes |
| your first claim | §3, *a witness discriminates at one interface* |

## Dissolution trigger (DESIGN §6)

Delete this brief when slices S0 through S4 are landed and green on the required floor: Loop carries the domain edge and executes its three rules, the for and while productions exist as body_producer_forward rows, the per-loop termination proof has its one producer and every consumer reads it, the fold seam emits the full encoding, and EffectPlanStep.While shares the measure. At that point the node shape, the grammar rows and their claims are the authority for what the loop forms mean, and this document, whose only job was to hand off the design, would be a second statement of it.
