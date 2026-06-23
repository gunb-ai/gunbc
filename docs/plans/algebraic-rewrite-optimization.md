# Plan — algorithmic-cost reduction by construction (rewrite suboptimal patterns)

**Status:** design spec · **POST-STABILITY EXPANSION** (operator 2026-06-21: budget-gate validation holds the stability §3 for now; this rewrite-*construction* design is relocated to **`ROADMAP.md` §5**, homed with self-hosting — IR-rewrite/canonicalization is most natural once `.dag` is the self-hosted truth). Do not build the rewrite catalog in the stability window; the cost-lens *foundation* (#5437 `symbolic_max` + per-fn subject) stays in-window as §3 budget-gate plumbing. **DESIGN.md + the carriers remain the authority** (DESIGN §6). A rule's real state is its branch/PR + the rule-row carrier.

## 0. What this is — and what it is NOT

**The goal:** catch the *common* algorithmic-cost mistakes a normal developer makes (`O(n²)→O(n)`, `O(2ⁿ)→O(n)`, `O(n)→O(log n)`) and **rewrite** them to the cheaper equivalent form. This is *construction, not validation* (DESIGN §5): a lint that warns "you wrote O(n²)" concedes the redundant form is writable; a **canonicalizer that rewrites it to the O(n) form** dissolves the redundancy — the §2 "minimize redundancy" master move turned on the **cost axis** instead of the representation axis.

**It is NOT an optimality oracle.** "Could *some* faster algorithm exist?" is undecidable in general (Rice) — that stays the advisory `synthesis.dag` residue (§7 below), a *ratchet forever*, never a wall. This lane is the decidable, wallable subset: a finite library of pre-proven rewrite rules.

**The two axes — keep them apart (this is the whole calibration):**

- **Bulletproof = soundness, absolute.** Every rewrite it *makes* is provably correct: a rule fires only when its precondition is *provable*, and stays silent otherwise. Zero false rewrites, ever. This attaches to *correctness when it fires* — never to coverage.
- **Honest-incomplete = coverage, published.** It catches a **named, finite catalog** of patterns, and the catalog is shown to the developer. So **silence has a published meaning: "no catalog rule matched" — NOT "your code is optimal."**

Conflating the two ("bulletproof, therefore catches everything") is the oversell to avoid. The failure modes map onto the axes: a *false rewrite* (unsound) is catastrophic and must be **0**; a *missed* optimization (incomplete) is fine and honest — the ratchet tail.

## 1. Why this subset is decidable (the real leverage)

Program **equivalence** is undecidable, so we build no universal oracle. What *is* decidable: **"does this Node subtree match a known pattern `P`, whose pre-proven rewrite `R` yields a cheaper equivalent, with `R`'s preconditions provable on this instance?"** Three substrate facts make that decidable here when it is not in C/Python:

1. **Effects are modeled, not inferred.** "the inner test is pure" / "the collection is not mutated mid-loop" is a *structural query on the Node's `EffectShape`*, not a dataflow guess. **This is the source of the power** — the preconditions that are undecidable elsewhere are structural here.
2. **One core form.** Recursion / `for` / `while` desugar to the `Loop` primitive (DESIGN §4), so the matcher matches one shape, post-desugar, up-to alpha-renaming.
3. **The cost oracle already exists.** `src/v2/lens/complexity.dag` (`complexity_lens`) projects the asymptotic class of a `Node`; it is the arbiter of "did this rule actually drop the class," and the discriminating test (input class `X` → output class `Y`, `Y ≺ X`).

Soundness is discharged **once, abstractly** (prove `R` turns any instance of `P` into an equivalent), not per-instance; the per-instance obligation is just the structural precondition check, **fail-closed** (can't prove it ⇒ don't fire). Sound, incomplete, grows by rules.

## 1a. Detection vs enforcement — the containment (grounded in the actual models)

The set of complexities we **detect** and the set of violations we **enforce/rewrite** are different sets, and the second is a subset of the first. Verified against `src/v2/lens/cost.dag` + `complexity.dag` (2026-06-21):

- **Detection is TOTAL by construction.** `cost.dag` U2 (lines 6–17): `SymbolicCost` is defined at the **closed kernel** (6 connectives × behaviors), composed by a total `fold_node` — `base_cost_for_connective` / `base_cost_for_behavior` match *every* kernel case, so **every well-formed program has a cost / asymptotic class by construction**. No per-function or per-feature opt-in; a new surface feature dissolves to already-costed kernel nodes (a new *connective/behavior* is a STOP). `complexity.dag` is purely the asymptotic *projection* of that one carrier (`complexity_lens = asymptotic_projection ∘ cost_lens`) — it never re-derives. **So: arbitrary functions ARE detectable.**
- **The verdict alphabet** (`AsymptoticClass`): `Constant · Log · Linear · Linearithmic · Polynomial{degree} · PolyLog{exponent} · Exponential · Factorial · Unknown`.
- **The boundary is precision, not coverage.** When a loop bound is not witnessable, the verdict is `ClassUnknown` — the **fail-closed lattice top** (`asymptotic_class_dominates(Unknown, _) = true`). So detection has *total coverage*, *partial precision*. Two model-grounded precision limits worth naming:
  - `PolynomialDegree` is **integer-only** (`NonZeroNat`), so `n√n` (n^1.5) is **literally unrepresentable** — a concrete reason it is excluded from the catalog (§3), not mere taste.
  - log **base** is not distinguished (asymptotically correct) — so "binary vs ternary search" is *not* a class distinction; both are `Log`. (Another reason ternary search is out.)

**The three nested sets:**

| set | definition | bound |
| --- | --- | --- |
| **D** — detected | all programs (total fold); verdict ∈ alphabet incl. `Unknown` | **unbounded** (total) |
| **D′** — precisely detected | verdict ≠ `Unknown` (bound witnessable, no unmodeled construct) | D′ ⊊ D |
| **E** — enforced / rewritten | the rewrite catalog (§3) | **E ⊆ D′** |

`E ⊆ D′ ⊆ D` is not just expected — it is **structurally guaranteed by the DONE bar** (§5): witness (b) requires `complexity_lens` to confirm a *strict* class drop `class_after ≺ class_before`, which is impossible if either side is `Unknown`. A rule literally cannot land outside D′.

**Why today's gate runs a small roster — and why that is NOT a detection limit.** The ROADMAP's "complexity lens gates a curated roster (add/bind/branch/loop)" is about which *subjects* are fed to the gate, not what cost *can* analyze. The roster is small because subject-*production* is not whole-corpus yet (the fn-body reflection gap — `enumerate_concepts()` covers type decls, not fn bodies). Detection is total; the gate's *reach* is subject-bound. Wire whole-corpus fn-body reflection and the gate runs over **all** of D. (This is the same reflection dependency the corpus hit-rate gate (§6) carries.)

## 1b. `Unknown` is an anemic atom — dissolve it over time (never a false pass)

Two invariants are fixed up front; the taxonomy is **not** — it grows by dissolution.

- **INVARIANT 1 — never a false pass (already holds).** `cost_lens` emits `Holds` only for a *determined precise class*; `UnknownCost → Violates{diagnostic}` (fail-closed, located). A "pass" always means "determined," never "gave up." This is the §5 win already in the model — do not regress it.
- **INVARIANT 2 — every `Unknown` is on the dissolution frontier.** `Unknown` is **not** a fixed terminal; it is an **anemic atom**, dissolved over time like any anemic leaf (DESIGN §2 `decompress → map → reduce`; a `String` part-number → grounded fields). The carrier already has the hook: `UnknownCost { diagnostic: Diagnostic }` *carries its reason* — the anemia is that the reason is a free-form symbol, not modeled reason-structure. Each decomposition resolves an `Unknown` to one of:
  - **construction** — a now-determined precise class (a witness gap / lattice-precision limit closed); net `Unknown`s *shrink*, it becomes new D′;
  - **a grounded `Terminal`** — genuinely atomic / provably undecidable (Rice), *positively recognized* (we exhibit *why*). The honest ratchet — "less frequent but important enough to comment on" (advisory).
  - *Soundness:* label `Terminal` only with exhibited evidence; the default for an undecomposed reason stays the fail-closed `Violates`. We never *claim* undecidability without proof (that would be a false terminal — "giving up" masquerading as "provably impossible").

**DFS the concept DAG first — reuse `Disposition`, do not fork.** An `Unknown` is structurally "a skipped/unmodeled decision that resolves to construction or a justified `Terminal`" — exactly the `Disposition` carrier (ROADMAP §0 / [disposition-carrier.md](disposition-carrier.md)). So Unknown-dissolution becomes one *consumer* of the Disposition ratchet, not a parallel "unknown-reason" enum (the §2 test: net concepts must not grow by re-invention). This **supersedes** any fixed `Undecidable | Undetermined` split — the set of named terminal reasons grows over time (the 🟡-backlog discipline), it is not designed up front.

**Gated (operator):** enriching `UnknownCost`'s reason is a coproduct change to `cost.dag` (its header: "splitting requires explicit operator ratification — substrate extension = stop signal"), and `Disposition` is currently *parked* in §0. So the *vehicle* (un-park / align Disposition) and the *cost.dag enrichment* are both operator-ratification points; the two invariants above hold regardless.

## 2. The model — a rewrite rule is a row (§2 horizontal)

Each rule is a content-addressed row carrying:

- **pattern** — a `Node`-tree shape (matched on the canonical post-resolve tree, up-to renaming);
- **precondition** — a structural predicate over `EffectShape` / refinement type / provenance;
- **rewrite** — the `Node→Node` transform to the cheaper form;
- **soundness proof** — the once-and-for-all equivalence obligation (`R(P) ≡ P`);
- **claimed transition** — `class_before → class_after`, checked against `complexity_lens`.

The pass itself is a `fold_node` catamorphism (DESIGN §6 — the one fold reused by every stage) over the canonical tree, trying each rule, firing the sound ones. Mechanically this is **not a new subsystem**: finding a cost-reducing equivalent is the same `find_witness` / coercion-as-homomorphism machinery §4 already has (DESIGN §4) — optimization = a cost-reducing homomorphism to a canonical form. **Termination:** each rule strictly decreases the cost measure, so the rewrite system terminates by a well-founded order (the `DescentEvidence` discipline, DESIGN §4/§5); pick a deterministic strategy so there is one normal form.

## 2a. Generalization — key on structural redundancy, NOT on asymptotic degree

The catalog must generalize (`O(n³)→O(n²)`, not only `O(n²)→O(n)`) without edge cases and without firing wrongly. The resolution: **a rule's match key is the structural redundancy; the degree drop is the *consequence*, never the key.**

- **Why not key on degree.** A rule keyed on "is `O(n^x)`" would fire on programs that are *genuinely* `O(n^x)` — e.g. three independent nested loops each doing real work (matmul-shaped). No sound rewrite exists there, so it would either rewrite to something inequivalent (unsound) or emit a false suboptimality claim. **Degree is not evidence of redundancy.**
- **Key on the redundancy → generalization is free.** `nested-membership→set`, written structurally as *"a pure linear membership/search over a collection invariant in the enclosing loop,"* fires wherever that shape appears, **regardless of how many *outer* loops wrap it.** So `O(n³)→O(n²)` is the *same rule* firing on the inner levels, and **fold-to-fixpoint** peels successive levels (`O(n³)→O(n²)→O(n)`) when each carries the redundancy. The §2 "one concept, every scale" move — the rule is degree-agnostic *by construction*, so there is no per-degree edge case to enumerate.
- **The cost model supports any degree.** `PolynomialCost{degree}` / `ClassPolynomial{degree}` carry an arbitrary integer degree, and `multiply_classes` increments/decrements it, so witness (b) (strict class drop) holds at every peel for any x. **Detection is not the constraint; soundness of the peel is.**
- **Termination/determinism of the fixpoint:** each peel strictly decreases the degree (well-founded); a deterministic peel order (innermost-first) gives one normal form.

**OPEN generalization problem (operator-flagged 2026-06-21): `O(n^x) → O(n log n)` is algorithmic SUBSTITUTION, not redundancy elimination.** Replacing nested pairwise comparison with a sort-based pass is *not* a local peel — the before/after are *different algorithms with the same I/O relation*, which verges on the undecidable equivalence problem. It does **not** generalize as "arbitrary x → n log n." Tractable form: enumerate specific high-value *idioms* (sort-based dedup, sort-based pair/closest-finding, repeated-linear-min → heap) as individual rules, each with its own once-proven soundness — bounded and honest, not a parameterized rule. **DECISION PENDING** operator input on whether a cleaner shared framing exists; until then, the n-log-n class is per-idiom and lands *after* the polynomial-peel seed.

## 3. The catalog — the common cases, by transition

Tiered by how cleanly the precondition is structural (this is also the build order):

**Tier 1 — precondition is a pure-effect query (build first):**

| pattern | transition | precondition (structural) |
| --- | --- | --- |
| nested membership / find-a-pair → hash-set/map | **O(n²)→O(n)** | inner test pure; collection not mutated in-loop |
| repeated `contains`/`indexOf` in a loop → precomputed index | **O(n²)→O(n)** | lookup pure; source not mutated in-loop |
| string/array concat built up in a loop → accumulator/builder | **O(n²)→O(n)** | append target not aliased/observed mid-loop |
| naive overlapping recursion → memoize | **O(2ⁿ)→O(n)** | callee is a pure function of its args (clean `EffectShape` query) |

**Tier 2 — precondition needs a refinement type or provenance (later):**

| pattern | transition | precondition |
| --- | --- | --- |
| linear scan of **sorted** data → binary search | **O(n)→O(log n)** | sortedness carried in the type, or traceable to a `sort` with no intervening mutation; silent otherwise |
| repeated linear min/max extraction → heap | **O(n²)→O(n log n)** | extraction pure; structural recurrence recognized |

**Deliberately excluded (model-grounded, not just taste):** `n·√n` is excluded because the cost model **cannot represent it** — `PolynomialDegree` is integer-only (`NonZeroNat`), so n^1.5 ∉ D′ and no rule targeting it could satisfy witness (b) (§1a). Ternary search is excluded because log base is not a class distinction — it is `Log`, identical to binary search (§1a); and sqrt-decomposition / ternary search are *deliberate techniques*, not accidental mistakes. Including either is the exotic-coverage trap.

**Deferred — constant-factor (class unchanged), in scope later, labeled as such:** loop fusion (two sequential loops over one range → one) and loop-invariant hoisting. Real wins, but they do not move between asymptotic classes, so they are kept out of the seed to keep the class-reduction headline clean.

## 3a. The seed rules, fully specified up front (input → output → controls)

Every rule lands as a row carrying *exactly* these fields; the worker builds to these, no interpretation. (Surface shown in pseudo-syntax; the matcher operates on the desugared `Loop` Node.)

### Rule 1 — `nested-membership → set` · **O(n²) → O(n)** · D′: `Polynomial{2} → Linear`

- **INPUT (slow form it fires on):**

  ```
out = []
for x in a:                 // |a| = n
  if contains(b, x):        // linear scan of b = O(|b|) → whole loop O(n·m)
    out.append(x)
```
- **OUTPUT (rewritten):**

  ```
bset = set(b)               // O(|b|)
out = []
for x in a:
  if bset.contains(x):      // O(1) amortized → whole loop O(n + m)
    out.append(x)
```
- **precondition (structural):** the membership test is pure (`EffectShape` pure) **and** `b` is not mutated anywhere in the loop body.
- **non-firing control (witness d):** the same shape where `b` *is* mutated in the loop (e.g. `b.append(...)`) — the precomputed set would go stale, so the rule **must not fire**; code untouched.
- **discriminating equivalence input (witness c):** `a=[1,2,3]`, `b=[2,3,4]` → both forms yield `[2,3]`.
- **author structurally (per §2a):** key on "pure invariant inner membership in a loop," *not* on degree-2 — so the same rule peels one level wherever it matches and fold-to-fixpoint gives `O(n³)→O(n²)→O(n)`. Add a **depth-2 witness** (a triply-nested instance reduced to doubly-nested) to prove the generalization, alongside the depth-1 witnesses above.

### Rule 2 — `naive-recursion → memoize` · **O(2ⁿ) → O(n)** · D′: `Exponential → Linear`

- **INPUT (slow form):**

  ```
fn f(n):
  if n < 2: return n
  return f(n-1) + f(n-2)    // overlapping subproblems → O(2ⁿ)
```
- **OUTPUT (rewritten):** `f` memoized on its argument — each `f(k)` computed once → O(n).
- **precondition (structural):** `f` is a **pure function of its arguments** (`EffectShape` pure — a *different precondition shape* from Rule 1, which is why D2 seeds with both: it proves the framework on both "no-mutation" and "pure-fn" preconditions).
- **non-firing control (witness d):** `f` has an observable effect (prints / mutates a global) — memoizing would change observable behavior, so **must not fire**.
- **discriminating equivalence input (witness c):** `f(10)` → both forms yield `55`.

This is the level of up-front specificity every later catalog row must reach **before** it is built — an under-specified rule is how the project never finishes (DESIGN §5: "done" = a real consumer green by execution + a discriminating input that goes red when wrong).

## 4. Decisions (resolved in-conversation 2026-06-21)

- **D1 — canonical-form-is-truth** (not emit-time-only). The minimal-cost form *is* the truth: a provably-equivalent slow/fast pair is the *same idea at different cost* (DESIGN §6 — a program is the canonical idea; surface syntax is one medium), so the slow surface normalizes to the fast canonical Node. Rewrites only ever change *cost*, never semantics (the §2 soundness obligation), so no meaning is ever lost — only redundancy. *Revisit to emit-time-only if real adoption friction appears (a developer wanting to keep a deliberately-slow-but-clear form); the rule library is unaffected either way.*
- **D2 — seed = TWO rules:** `nested-membership→set` (**O(n²)→O(n)**, the headline) **and** `naive-recursion→memoize` (**O(2ⁿ)→O(n)**). Two, not one, because their preconditions have *different shapes* — "no mutation" vs "pure function of args" — which de-risks the framework abstraction early. A *set* of rules built at once is how you get a half-done set; two contrasting rules proven end-to-end is the framework proof.
- **D4 — constant-factor rewrites OUT of the seed**, in scope later (see §3 Deferred).

## 5. DONE — four witnesses per rule (the anti-half-done bar)

No rule lands without all four, **by execution** (not typecheck, not grep — DESIGN §5):

1. **(a) rewrites** the slow form to the fast form;
2. **(b) class drop** — `complexity_lens` confirms `class_after ≺ class_before`;
3. **(c) equivalence by execution** — rewritten output ≡ original output on a *discriminating* input (a real run, not an assertion);
4. **(d) non-firing control** — a look-alike input whose precondition *fails* (e.g. an impure inner test, or an aliased append target) is left **untouched**.

**(d) is the witness half-done versions always skip** — they rewrite when they must not. Making it mandatory is what makes "done" mean done. Each rule is `*_test.dag` with `test fn`s so the CI floor auto-discovers it (DESIGN "Building & checks").

## 6. Acceptance — "surprised if it missed something," made measurable

Before the lane is declared useful: **run the catalog over a real corpus** (this project's own seed, or a sampled real codebase) and report **% of files with ≥1 finding**. Two honest outcomes, both useful: high hit-rate ⇒ as useful as hoped, *measured*; low hit-rate ⇒ either add catalog rows or the true finding is "well-factored code has few of these" — worth knowing, and it prevents shipping a toy that never fires. The gut target becomes a number; no silent caps (DESIGN §6 — `log` what was dropped).

## 7. Honesty surface (the anti-oversell, in the developer's face)

Because D1 rewrites silently, emit a **transparency report** of what was canonicalized:

> rewrote N patterns; classes dropped `X→Y`; **this pass verifies the rewrites it made are correct; it does NOT verify your code is globally optimal.**

The diff *is* the detection; that last sentence is the calibration, every run. The published catalog (§3) is the other half: the developer can read exactly what is and is not looked for.

## 8. The undecidable residue (stays advisory, never gates)

`src/v2/lens/synthesis.dag` already surfaces the lower-bound *gap* (realization cost vs a declared relation's structural lower bound) — but it is `AdvisoryLens`, not a fail-closed `Diagnostic`, by design (its header: "algorithm choice is design-tier — programmer decides"). It is the honest home for "you may be above the theoretical floor but I have no sound rewrite" — the *ratchet forever* half of the trichotomy (DESIGN §5). This lane never promotes that to a gate.

## 9. Sequencing

1. **Foundation — the cost oracle must be trustworthy.** `complexity.dag` already gates a curated roster (add/bind/branch/loop), but the cost-lens zero-absorption fix (`symbolic_max` floor) and a subject-producer for every fn are prerequisites — the class-delta oracle (witness **b**) is meaningless if the cost fold lies. *(These were the original §3 worker items; they survive as the foundation this lane stands on.)*
2. **Framework + the two D2 seed rules**, each with the four-witness DONE bar. This proves the rewrite-rule-as-row abstraction carries the cost axis on two contrasting precondition shapes. *⚠ canonical-form-is-truth (D1) adds a normalization pass to the pipeline — a load-bearing change. Escalate before editing pipeline stages.*
3. **Corpus hit-rate acceptance gate** (§6).
4. **Grow the catalog by rows** — remaining Tier-1, then Tier-2 (binary search, heap), then the deferred constant-factor rules.

## Dissolution trigger (DESIGN §6)

Delete this doc when the rewrite framework + the seed rules have landed (each carrying its four-witness test), the catalog lives as discoverable rule-rows, and the corpus hit-rate gate runs in CI. At that point the rule carriers + their witnesses tell the whole story and this design doc is redundant.
