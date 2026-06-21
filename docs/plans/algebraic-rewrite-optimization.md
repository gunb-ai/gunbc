# Plan — algorithmic-cost reduction by construction (rewrite suboptimal patterns)

**Status:** design spec (decisions resolved in-conversation 2026-06-21) · **DESIGN.md + the carriers
remain the authority** (DESIGN §6 "no parallel-ledger docs"). A rule's real state is its branch/PR +
the rule-row carrier. Linked from `ROADMAP.md` §3 *Algorithmic-cost reduction*.

## 0. What this is — and what it is NOT

**The goal:** catch the *common* algorithmic-cost mistakes a normal developer makes
(`O(n²)→O(n)`, `O(2ⁿ)→O(n)`, `O(n)→O(log n)`) and **rewrite** them to the cheaper equivalent form.
This is *construction, not validation* (DESIGN §5): a lint that warns "you wrote O(n²)" concedes the
redundant form is writable; a **canonicalizer that rewrites it to the O(n) form** dissolves the
redundancy — the §2 "minimize redundancy" master move turned on the **cost axis** instead of the
representation axis.

**It is NOT an optimality oracle.** "Could *some* faster algorithm exist?" is undecidable in general
(Rice) — that stays the advisory `synthesis.dag` residue (§7 below), a *ratchet forever*, never a wall.
This lane is the decidable, wallable subset: a finite library of pre-proven rewrite rules.

**The two axes — keep them apart (this is the whole calibration):**
- **Bulletproof = soundness, absolute.** Every rewrite it *makes* is provably correct: a rule fires
  only when its precondition is *provable*, and stays silent otherwise. Zero false rewrites, ever. This
  attaches to *correctness when it fires* — never to coverage.
- **Honest-incomplete = coverage, published.** It catches a **named, finite catalog** of patterns, and
  the catalog is shown to the developer. So **silence has a published meaning: "no catalog rule
  matched" — NOT "your code is optimal."**

Conflating the two ("bulletproof, therefore catches everything") is the oversell to avoid. The failure
modes map onto the axes: a *false rewrite* (unsound) is catastrophic and must be **0**; a *missed*
optimization (incomplete) is fine and honest — the ratchet tail.

## 1. Why this subset is decidable (the real leverage)

Program **equivalence** is undecidable, so we build no universal oracle. What *is* decidable: **"does
this Node subtree match a known pattern `P`, whose pre-proven rewrite `R` yields a cheaper equivalent,
with `R`'s preconditions provable on this instance?"** Three substrate facts make that decidable here
when it is not in C/Python:

1. **Effects are modeled, not inferred.** "the inner test is pure" / "the collection is not mutated
   mid-loop" is a *structural query on the Node's `EffectShape`*, not a dataflow guess. **This is the
   source of the power** — the preconditions that are undecidable elsewhere are structural here.
2. **One core form.** Recursion / `for` / `while` desugar to the `Loop` primitive (DESIGN §4), so the
   matcher matches one shape, post-desugar, up-to alpha-renaming.
3. **The cost oracle already exists.** `src/v2/lens/complexity.dag` (`complexity_lens`) projects the
   asymptotic class of a `Node`; it is the arbiter of "did this rule actually drop the class," and the
   discriminating test (input class `X` → output class `Y`, `Y ≺ X`).

Soundness is discharged **once, abstractly** (prove `R` turns any instance of `P` into an equivalent),
not per-instance; the per-instance obligation is just the structural precondition check, **fail-closed**
(can't prove it ⇒ don't fire). Sound, incomplete, grows by rules.

## 2. The model — a rewrite rule is a row (§2 horizontal)

Each rule is a content-addressed row carrying:
- **pattern** — a `Node`-tree shape (matched on the canonical post-resolve tree, up-to renaming);
- **precondition** — a structural predicate over `EffectShape` / refinement type / provenance;
- **rewrite** — the `Node→Node` transform to the cheaper form;
- **soundness proof** — the once-and-for-all equivalence obligation (`R(P) ≡ P`);
- **claimed transition** — `class_before → class_after`, checked against `complexity_lens`.

The pass itself is a `fold_node` catamorphism (DESIGN §6 — the one fold reused by every stage) over the
canonical tree, trying each rule, firing the sound ones. Mechanically this is **not a new subsystem**:
finding a cost-reducing equivalent is the same `find_witness` / coercion-as-homomorphism machinery §4
already has (DESIGN §4) — optimization = a cost-reducing homomorphism to a canonical form. **Termination:**
each rule strictly decreases the cost measure, so the rewrite system terminates by a well-founded order
(the `DescentEvidence` discipline, DESIGN §4/§5); pick a deterministic strategy so there is one normal
form.

## 3. The catalog — the common cases, by transition

Tiered by how cleanly the precondition is structural (this is also the build order):

**Tier 1 — precondition is a pure-effect query (build first):**

| pattern | transition | precondition (structural) |
|---|---|---|
| nested membership / find-a-pair → hash-set/map | **O(n²)→O(n)** | inner test pure; collection not mutated in-loop |
| repeated `contains`/`indexOf` in a loop → precomputed index | **O(n²)→O(n)** | lookup pure; source not mutated in-loop |
| string/array concat built up in a loop → accumulator/builder | **O(n²)→O(n)** | append target not aliased/observed mid-loop |
| naive overlapping recursion → memoize | **O(2ⁿ)→O(n)** | callee is a pure function of its args (clean `EffectShape` query) |

**Tier 2 — precondition needs a refinement type or provenance (later):**

| pattern | transition | precondition |
|---|---|---|
| linear scan of **sorted** data → binary search | **O(n)→O(log n)** | sortedness carried in the type, or traceable to a `sort` with no intervening mutation; silent otherwise |
| repeated linear min/max extraction → heap | **O(n²)→O(n log n)** | extraction pure; structural recurrence recognized |

**Deliberately excluded (the "exotic" guardrail):** `n·√n` (sqrt-decomposition is a *deliberate
technique*, not an accidental mistake) and ternary search (niche unimodal optimization). Including them
is the exotic-coverage trap; binary search covers the search case developers actually hit.

**Deferred — constant-factor (class unchanged), in scope later, labeled as such:** loop fusion (two
sequential loops over one range → one) and loop-invariant hoisting. Real wins, but they do not move
between asymptotic classes, so they are kept out of the seed to keep the class-reduction headline clean.

## 4. Decisions (resolved in-conversation 2026-06-21)

- **D1 — canonical-form-is-truth** (not emit-time-only). The minimal-cost form *is* the truth: a
  provably-equivalent slow/fast pair is the *same idea at different cost* (DESIGN §6 — a program is the
  canonical idea; surface syntax is one medium), so the slow surface normalizes to the fast canonical
  Node. Rewrites only ever change *cost*, never semantics (the §2 soundness obligation), so no meaning is
  ever lost — only redundancy. *Revisit to emit-time-only if real adoption friction appears (a developer
  wanting to keep a deliberately-slow-but-clear form); the rule library is unaffected either way.*
- **D2 — seed = TWO rules:** `nested-membership→set` (**O(n²)→O(n)**, the headline) **and**
  `naive-recursion→memoize` (**O(2ⁿ)→O(n)**). Two, not one, because their preconditions have *different
  shapes* — "no mutation" vs "pure function of args" — which de-risks the framework abstraction early. A
  *set* of rules built at once is how you get a half-done set; two contrasting rules proven end-to-end is
  the framework proof.
- **D4 — constant-factor rewrites OUT of the seed**, in scope later (see §3 Deferred).

## 5. DONE — four witnesses per rule (the anti-half-done bar)

No rule lands without all four, **by execution** (not typecheck, not grep — DESIGN §5):
1. **(a) rewrites** the slow form to the fast form;
2. **(b) class drop** — `complexity_lens` confirms `class_after ≺ class_before`;
3. **(c) equivalence by execution** — rewritten output ≡ original output on a *discriminating* input
   (a real run, not an assertion);
4. **(d) non-firing control** — a look-alike input whose precondition *fails* (e.g. an impure inner
   test, or an aliased append target) is left **untouched**.

**(d) is the witness half-done versions always skip** — they rewrite when they must not. Making it
mandatory is what makes "done" mean done. Each rule is `*_test.dag` with `test fn`s so the CI floor
auto-discovers it (DESIGN "Building & checks").

## 6. Acceptance — "surprised if it missed something," made measurable

Before the lane is declared useful: **run the catalog over a real corpus** (this project's own seed, or
a sampled real codebase) and report **% of files with ≥1 finding**. Two honest outcomes, both useful:
high hit-rate ⇒ as useful as hoped, *measured*; low hit-rate ⇒ either add catalog rows or the true
finding is "well-factored code has few of these" — worth knowing, and it prevents shipping a toy that
never fires. The gut target becomes a number; no silent caps (DESIGN §6 — `log` what was dropped).

## 7. Honesty surface (the anti-oversell, in the developer's face)

Because D1 rewrites silently, emit a **transparency report** of what was canonicalized:
> rewrote N patterns; classes dropped `X→Y`; **this pass verifies the rewrites it made are correct; it
> does NOT verify your code is globally optimal.**

The diff *is* the detection; that last sentence is the calibration, every run. The published catalog
(§3) is the other half: the developer can read exactly what is and is not looked for.

## 8. The undecidable residue (stays advisory, never gates)

`src/v2/lens/synthesis.dag` already surfaces the lower-bound *gap* (realization cost vs a declared
relation's structural lower bound) — but it is `AdvisoryLens`, not a fail-closed `Diagnostic`, by design
(its header: "algorithm choice is design-tier — programmer decides"). It is the honest home for
"you may be above the theoretical floor but I have no sound rewrite" — the *ratchet forever* half of the
trichotomy (DESIGN §5). This lane never promotes that to a gate.

## 9. Sequencing

1. **Foundation — the cost oracle must be trustworthy.** `complexity.dag` already gates a curated roster
   (add/bind/branch/loop), but the cost-lens zero-absorption fix (`symbolic_max` floor) and a
   subject-producer for every fn are prerequisites — the class-delta oracle (witness **b**) is
   meaningless if the cost fold lies. *(These were the original §3 worker items; they survive as the
   foundation this lane stands on.)*
2. **Framework + the two D2 seed rules**, each with the four-witness DONE bar. This proves the
   rewrite-rule-as-row abstraction carries the cost axis on two contrasting precondition shapes.
   *⚠ canonical-form-is-truth (D1) adds a normalization pass to the pipeline — a load-bearing change.
   Escalate before editing pipeline stages.*
3. **Corpus hit-rate acceptance gate** (§6).
4. **Grow the catalog by rows** — remaining Tier-1, then Tier-2 (binary search, heap), then the deferred
   constant-factor rules.

## 10. Dissolution trigger (DESIGN §6)

Delete this doc when the rewrite framework + the seed rules have landed (each carrying its four-witness
test), the catalog lives as discoverable rule-rows, and the corpus hit-rate gate runs in CI. At that
point the rule carriers + their witnesses tell the whole story and this design doc is redundant.
