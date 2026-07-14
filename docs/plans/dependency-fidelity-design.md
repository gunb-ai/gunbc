# Dependency-Fidelity — making "CI green" mean *declared ≡ witnessed*

Status: design draft (operator-directed, 2026-07-14). Companion to the [*enforcement-intent* thread](enforcement-intent-design.md) (the same "ask once, compile forever" spine, made quantitative). Reasoned serially per DESIGN.md; each section is a consequence of the ones before it.

---

## 0. North star (the constraint everything serves)

> Change 5 stray lines → the system **generates exactly the coverage needed** to confirm those lines broke no relationship they could affect, and runs only that. A 5-line change costs 5-lines-worth of verification, not a corpus sweep.

Every choice below is judged against this. Coverage that is not **derived and minimal per-change** is the CI-blind / whole-corpus problem in a new coat.

## 1. The problem this closes

The int-literal keystone bug — an ingested `fn` body silently failing to become a substrate `Arrow` — passed CI green and collected two diff-level review APPROVEs while being **behaviorally dead**, because the one witness that exercised the capability lived in a CI-excluded directory. Reviewers and CI are both structurally blind to *relationships that are never witnessed*. "CI green" is therefore a proxy that lies exactly where it matters most: on capabilities no test drives.

The failure is not "a missing test." It is **the absence of a stated, witnessed relationship** between a stage's input and its output — and worse, sometimes the relationship was never in the model at all.

## 2. The law

A function's signature is a **claim about dependency structure**: which outputs depend on which inputs, and how. Correctness = the *declared* structure equals the *witnessed* (by-execution) structure over its valid domain. Everything reduces to enforcing:

> **declared dependency structure ≡ witnessed dependency structure.**

CI-green earns meaning exactly as it certifies this equivalence — and it is fractal (DESIGN §7): "declared ≡ witnessed" is itself a relationship the system must witness (the checker is subject to its own law, §6 below).

## 3. Three-way, not input-only: the ray-tracing frame

Coverage is **not** input permutations. It is the joint `(input, function, output)` relation, sampled like a renderer traces a scene. Witnesses are *rays* through the relation manifold; **strong typing is the scene geometry** — the type structure bounds the valid input/output space, so tracing is tractable and enumerable (unlike untyped fuzzing, which has no geometry to sample against). Three surfaces must be hit:

1. **Input reachability** — is every *meaningful* input region exercised? (An input permutation the type admits but no witness drives is a coverage hole.)
2. **Output reachability** — is every *declared-possible* output actually **witnessed**? A return shape the type says is producible that **no ray ever hits** is suspicious: either the return type **over-claims** the output space (a declared output that is in fact unreachable — the output-side dual of a vestigial input), or a **bug** prevents reaching it, or it is simply **un-covered**. All three are `declared ≠ witnessed` on the *output* side — and all three are worth a flag. (This is the operator's "if some outputs are impossible in witnesses, that is concerning too.")
3. **Function fidelity** — does the actual input→output map match the declared interaction structure? (The decomposition / single-authority analysis, §5.)

A renderer with geometry no ray ever hits has either dead geometry or under-sampling. Same here: **unreached declared outputs and unexercised declared inputs are the same class of defect seen from two ends**, and the interaction structure is the surface between them.

## 4. Coverage = mutation-adequate discrimination (the key primitive)

Coverage of a relationship R is:

> the **minimal** witness set such that **every single-edit mutation of R's declared structure is killed** by at least one witness going red.

Not line coverage, not the combinatorial product — the minimal set of rays that would catch the relationship being wrong *by one step* on any of the three surfaces (an input dropped, an output made unreachable, an interaction added/removed). This is DESIGN §5's "discriminating RED" made quantitative, and three properties fall out:

- **Generatable.** Testgen enumerates the one-step neighbor-mutations of the declared structure (bounded by the type geometry) and produces witnesses that separate R from each neighbor.
- **Minimal + per-relationship.** No witness that fails to kill a distinct mutant earns its place — which is what makes the north star reachable.
- **Domain-bounded.** Mutations and witnesses range only over what the type + preconditions actually admit, so functions with rich preconditions are not punished with impossible permutations.

## 5. The two fidelity rules (with soundness baked in)

- **Under-claim (missing authority):** a witnessed dependency edge with no declared authority → **located flag**. This is the int-literal class (value flows through the captured projection; nothing declared it, so a consumer took the identity edge). Fix-form is free — declare the authority, ground it, or typed-refuse; the law constrains correctness, not creativity.
- **Over-claim (false coupling):** decompose `foo(a,b)` **only if `a` and `b` sit in separate connected components of the output's dependency graph *transitively* — i.e. they never *meet* at any downstream join.** Local non-interaction is necessary but **not** sufficient; a constructor or fact-bundle creates a downstream join and is therefore globally coupled and *not* flagged. This local→global lift is what rescues every valid pattern in §6. Decomposition = "the dependency graph has a cut the signature ignores."

Both rules produce a **located flag with evidence + an exemption path** (declare the coupling / authority), never an auto-rewrite — because the ontological-coupling judgment must be *declared and remembered*, which is what §6 is for.

## 6. Soundness — the anti-false-positive corpus (load-bearing)

A checker that flags valid designs gets **worked around**, and a worked-around checker is worse than none. So the guard against over-constraint is a first-class, executable component, not an afterthought.

- A curated **golden corpus of valid patterns**, each `{ pattern, correct verdict }`. Categories that must **never** be flagged as defects:
  - **product constructors** (`pair`, record/tuple build) — independent inputs bundled *by design*;
  - **fact-bundles / atomicity** — behaviorally independent, ontologically co-produced;
  - **uniform interfaces** (§2 Realization) — `handle(request, ctx)` where the uniform shape *is* the point;
  - **combinators / projections** (`const`, adapters) — an argument's independence is the feature;
  - **recursion / fixpoints** — self-dependency the substrate encodes acyclically (§4); the "dependency graph" here must be the **bounded-forward** graph or self-reference reads as a cut it is not — this caveat is *held in the corpus*, and gates the over-claim arm from shipping until resolved;
  - **higher-order** — interaction is between a value and a *function* argument.
- **Every checker rule runs against this corpus as a RED control.** A rule that flags a valid entry, or misses a should-flag entry, is **rejected or refined before it ships.**
- **The only sanctioned way to silence a flag is to add the pattern to the corpus** (with justification), never to disable the check. A real-world false positive → a new corpus entry → the checker is refined → the workaround becomes unnecessary. Hacking-around is *designed out*: the escape valve strengthens the checker instead of bypassing it.

This is DESIGN §7 self-application: the checker's declared behavior (what it flags) must equal its witnessed behavior (the corpus), or the checker is the bug.

## 7. The engine: affected-set × testgen (the north star, mechanized)

```
change → affected set                (existing affected-set machinery)
       → declared structures of the affected elements
       → for each, testgen generates the mutation-discriminating witness set
         (bounded by the type geometry; three surfaces of §3)
       → run only those → observe witnessed structure
       → fidelity check: declared ≡ witnessed, per surface
       → green ⟺ every affected relationship still holds across its discriminating coverage
```

A 5-line change re-witnesses only the relationships those lines participate in, at exactly the resolution that would catch a break — nothing more. **CI-green now *means* "declared ≡ witnessed across the affected set's discriminating coverage."** That is the valuable indicator.

## 8. Staging

1. **Pure v2 functions first.** Interaction and reachability are well-defined and grid-testable; the seed's effectful code needs its state lifted into signatures before the law is even well-defined there (which is the right pressure anyway).
2. **Ship the under-claim / coverage arm + the §7 engine first.** It is what would have caught the initial bug, it is the north-star engine, and it is "make CI green valuable" directly. Deliver: (a) declared-structure extraction for pure v2 fns, (b) the mutation-neighbor testgen, (c) the affected-set-scoped runner, (d) the fidelity verdict with located discrepancies.
3. **The graph-cut over-claim (decomposition) arm second**, gated behind the soundness corpus reaching a measured confidence bar (N valid categories, zero false flags), and behind the recursion/bounded-forward caveat being resolved in the corpus.
4. **Output-reachability sweep** as an early, cheap win: for pure v2 fns already under witness, flag declared outputs no witness produces — likely to surface real dead branches / over-wide return types immediately.

## 9. Acceptance (self-witnessing, per the operator's two criteria)

- **Solves the initial problem** — the int-literal relationship is in the affected set of its own diff; the §7 engine generates its discriminating coverage and goes red → merge blocked automatically. Demonstrated on that exact PR as the first acceptance witness.
- **Does not rule out valid solutions** — each §6 corpus category is a live RED-control the checker must pass; the property is *witnessed*, not asserted, and grows monotonically as real patterns are added.

## 11. Enrollment receipts (verified on real code, 2026-07-14)

Landed and green-by-execution: the **verdict spine** `v2.lens.dependency_fidelity` (§3 single-authority `FidelityVerdict`/`FidelityDiscrepancy`, input surface consuming `unused_parameters`; #6567), the **§4 coverage primitive** `v2.lens.mutation_adequacy` (a mutation is *killed* iff its discriminating unit holds, grounded on `v2.lens.discrimination`; #6577), and the **under-claim arm** `v2.lens.identity_captured_navigation` projecting into `UnderClaimedEdge` (#6556). The under-claim corpus sweep runs **offline** (`filesystem_read`-based, so not affected-set-attributable — operator-ruled) and scans its curated high-risk roster clean (no live instances; the keystone was fixed by #6558).

**The §6 wall caught an unsound enrollment — on our own code.** First attempt to enroll input-reachability as a live tree-lens (run `unused_parameters` over each `FnArrowDecl` from `fn_arrow_decl_facts_live()`) was **green on synthetic tests but flagged false positives the moment it ran over real compiled fns** — the mature `unused_parameters` lens flagged *its own module*. Root cause: `FnArrowDecl` is `{ output: Node, params: List<FnArrowParam> }` — `output` is the **return node only**; the parameters are a **separate field**. Feeding `dependency_lens(decl.output)` to a *parameter*-usage analyzer analyzes the wrong tree. This is §1's keystone lesson recapitulated: synthetic-green lies exactly where a lens is fed the wrong real input; only *running over real code* exposes it. **Confirmed feasible:** a per-module bounded run is fast (~236 ms, no whole-corpus reconcile wedge — cf. `corpus_dependency_view`'s `WallPricedAbort` at ~53 min, which is *why* the subject must be affected-set-scoped, not whole-corpus). **Blocker for a sound enrollment:** a param-aware feed is required — reconstruct the full arrow (params ∪ body) so the analyzer sees both declarations and uses, or find a full-arrow-node accessor. No existing corpus driver of `unused_parameters` exists to copy (it has only ever been synthetic-tested). This is the concrete next increment; the unsound `dependency_fidelity_corpus` gate was **not** landed.

## 10. Open sub-threads

- Recursion / fixpoints under the graph-cut arm (must be the §4 bounded-forward graph). **Blocks** the over-claim arm; held in the corpus.
- The cross-function "same relationship computed twice" case is not per-function interaction — it needs the §3 single-authority sweep over the whole graph; sequence after the per-function engine.
- Minimal-witness-set generation: is mutation-neighbor enumeration always finite/bounded under the type geometry, or are there type shapes (open coproducts, higher-order) where the neighbor set is unbounded and needs a sampling policy? Decide before scaling past pure first-order fns.
