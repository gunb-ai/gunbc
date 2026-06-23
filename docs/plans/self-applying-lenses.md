# Self-applying lenses — detect → generalize → emit → write

**Crux ([fractal intent-linearity](intent-linearity-design-draft.md)).** A program's *description length should equal its irreducible
information content* — the minimal generative template plus the genuinely-distinct data — **recursively
at every nesting level**. Equivalently: the intent is 1:1 with its own inputs; the description grows
*only* with distinct information, never with repetition. This is the decidable, enforceable shadow of
§1's limit ("replace convention with necessity until nothing arbitrary survives"): redundancy is
convention surviving where a reference was available, and super-linear description is exactly that
convention made visible. Enforcing it over the code-as-its-own-input — fractally — is §2's master move
turned into a construction wall, and the micro-scale twin of §7's seed-shrink (a compiler 1:1 with its
inputs is the seed at its irreducible core). **Bound (do not let "linear" become a universal wall):** it
is a *wall up to the substrate's expressible abstraction* (anti-unification computes the structural
minimum relative to the available combinators) and a *ratchet beyond it* (true minimal description is
Kolmogorov-uncomputable). The frontier advances as the catalog/combinators grow — which is §7's
"language design opens up," measured. Prerequisite: the minimal form must be *expressible and ergonomic*
to reference (the Ergonomics lane is what widens this wall). The candidate DESIGN.md articulation of this
crux — held against §1/§2/§7 for the operator's "model §1's axioms + enforce the syllogism" thread — is
drafted in [intent-linearity design draft](intent-linearity-design-draft.md) (operator-review, not yet a
DESIGN.md edit).

**Thesis.** A lens that only *flags* concedes the bad state is writable and leaves the fix to a
human (who pays the §1 time, and re-introduces fail-open arms by hand). The next form of the lens
**produces the correct pattern and applies it through our own write API** — it does not report a
violation, it removes it. This is the §7 recursion (the dedup principle applied to the dedup-*tools*)
and the apotheosis of the Ergonomics lane ("make the fold the path of least resistance" → the lens
*writes* the fold).

## The unifying concept: redundant intent

Every member of this lens family detects **specification complexity above the essential minimum**:
the *intent* has a minimal generative description; the *code* spells it out at higher complexity.

- hand-unrolled fold — intent "do X to each of N" (O(1) intent + O(n) data), code O(n) statements.
- 2-D / nested unroll — intent "fold over a grid" (O(1) + O(n²) data), code **O(n²)** statements.
- if/else-ladder dispatch — intent "look up key→value" (a table: O(n) data + O(1) dispatch), code O(n) branches.
- duplicate type decls (`structural_similarity`) — intent "one parameterized type", code N type decls.

This is §1/§2 made measurable. Anti-unification yields the **generalization** (template + per-element
substitution); the redundancy is `spec_size − generalization_size`. The generalization *is* the
minimal-intent code — so the same read that **measures** the gap **produces** the fix (§4: one
decision procedure run in different directions, N models not N×M).

## The engine: anti-unification (one kernel, two binders)

`congruent`/`anti_unify` (seeded in `v2.lens.simulated_relationship`) is the shared kernel. It serves:

- **term layer** — N near-identical statements → the varying part becomes a **list element** (a fold).
- **type layer** (`structural_similarity`, currently an unrealized scaffold) — N near-identical type
  decls → the varying part becomes a **type parameter** (a generic; the `Int8…Int128` = one `Compose`
  axis). Same move, different binder. These should be **one engine imported by both lenses**, not two.

## Three refinements proven by stress test (scratch S1–S4)

1. **Fractal recursion (S2).** A 2-D unroll is a fold-of-folds; a single application removes one layer
   (outer flagged *and* inner-row flagged independently). The producer-and-applier must **recurse into
   its own output's holes** until the residue is irreducible (§6: a finished stage is one fold; the
   bottom is a named irreducible kernel). The recursion is the O(n²)→O(n)→O(1)-spec reduction.
2. **Type-homogeneous holes need resolve, not parse (S4).** A fold over a heterogeneous coproduct —
   `handle(Read{path}); handle(Write{path,data}); handle(Close{})` — is **missed** by structural
   congruence (arms differ in shape). The real criterion is "the hole ranges over inhabitants of one
   type" (§2-deep, §3-grounding), which requires the **resolved type** at each hole. (Producer scope
   note sent to the parse/resolve-walk lane.)
3. **The species are distinct schemes; one isn't a fold (taxonomy).** foldl (gate chain) · mapAccumL/
   scan (interleaved byte-decomp) · reduce (n-ary binary) · **table-lookup (if/else ladder)**. The
   ladder's correct fix is "these cited rows belong in `extdeps/` + one generic dispatch" (§3: dispatch
   lives in extdeps, not std) — **not** a fold. The lens must name the scheme so it emits the right form.

## Collapse into complexity analysis (the same move, two axes)

`cost.dag` computes runtime cost as a **`fold_node` over the AST** (`SumCost` for a sequence,
`ProductCost` for a nested loop) → an `AsymptoticClass`. That is the *same catamorphism shape* this
lens uses, measuring a different §1-time axis:

- **complexity = run-time** (§1 *cost* axis — time to run).
- **redundant intent = change-time** (§1 *complexity* axis — time to change; §2 "redundant work defers
  cost into the future").

Both are §2 *minimize redundancy*: DRY on the **execution** (don't compute the same thing twice) vs DRY
on the **source** (don't write the same shape twice). So the unification is one engine:

> **redundancy = (actual − minimal) along a §1 time-axis, computed by a catamorphism, closed by an
> anti-unification `(pattern → minimal-form)` rewrite catalog, applied via the write API.**

Parameterized by: (a) the **representation walked** — source-AST (redundant intent) vs cost-recurrence
(complexity, the `SumCost`/`ProductCost` shape); (b) the **§1-axis minimized** — change-time vs run-time.
Instances: `simulated_relationship` (unroll→fold) · `structural_similarity` (dup type→generic) · the §5
`O(n²)→O(n)` catalog (redundant *computation* → memo / single-pass). One engine, N catalog rows
(§2-horizontal) — not the N×M "per-idiom rules" §5 feared.

**Decidability split (the expressibility frontier, §0):**
- **WALL / self-applying** — source-redundancy + the *finite* rewrite catalog ("bulletproof where it
  fires", §5). Decidable: pattern-match + rewrite.
- **RATCHET / advisory** — *global* optimality (Rice: "is there ANY cheaper equivalent?"). `complexity.dag`
  stays `RatchetForever` for that residue (§3 "synthesis stays advisory").

**Honest disanalogy — the axes genuinely diverge, so a row must tag which one it improves:**
- elf 8-byte unroll: high spec-redundancy, **O(1) runtime** (8 ops either way) — redundant-intent fires,
  cost does **not**.
- compact `for i: for j:`: **O(1) spec**, O(n²) runtime — cost fires, redundant-intent does **not**.

The "minimal form" is axis-relative; some rewrites improve both axes, some only one. (And complexity
anti-unifies the *cost-recurrence*, a derived representation — you fold to cost first, then match —
whereas redundant-intent anti-unifies the source AST directly.)

## Dependencies

- **emit** (§6, `serialize_target ∘ translate`) to render the generalized `Node` back to source.
- a **filesystem write effect** to apply it (the write twin of the lenses' existing `filesystem_read`).
- **resolve** facts on the corpus walk (refinement 2) — the shared grounding authority.

## Consolidation map (what merges, what deletes, and when)

The registry must not *itself* be a fork (a linearity enforcer that is non-linear would be the §7 irony).
So the redundancy lenses become **rows of one engine**, not peers. Nothing below deletes now — each
deletion is gated on a named trigger (§6); executing them before the trigger would strand the floor/lens
wiring.

- **`v2.lens.intent_linearity` is the registry** (landed, lens_unit-green). `simulated_relationship`'s
  `chain_is_simulated` is **row 1** (consumed, not duplicated). No deletion — it's the row-1 detector.
- **`v2.lens.structural_similarity` → a registry row.** It is today an *unrealized scaffold*
  (`verdict: Unrealized`, no predicate) — the type-decl-layer twin of the same anti-unification move.
  **Consolidate:** realize it as an `intent_linearity` row over the type-decl forest (binder = type
  parameter, not list element), reusing the shared kernel.
  **DELETE on trigger** `structural_similarity realized as an intent_linearity row`: the bespoke
  `StructuralSimilarityVerdict` / `TypeShape` / `FnShapeUnrealized` scaffold (`structural_similarity.dag`
  lines 12–45) — once the row exists, that empty machinery is dead. *(Not before — it carries a live
  `ConstructionJustification` the hygiene backstop counts.)*
- **`cost.dag` / `complexity.dag` → run-time rows.** The decidable `(pattern → cheaper-form)` catalog
  entries become `RunTime`-axis rows in the same registry; `complexity.dag` keeps **only** its
  `RatchetForever` global-optimality residue. **No file deletion** — `cost.dag`'s `fold_node` cost
  catamorphism is the row's `detect`; what dissolves is the *separate* notion of "a complexity rewrite
  engine distinct from a redundancy engine" (they were one).
- **The flagged instances** (`run_ci_gates_sequential`, `elf/encode` unrolls, the dispatch ladders) are
  **rewritten, not deleted as files** — the apply-half replaces each hand-unrolled body with its fold/
  table. Those line-deletions are the *output* of the enforcer, gated on emit + write (§6 + the write
  effect), not a manual sweep.

**Pre-audit fork-check findings (the audit starts from these, doesn't rediscover them):**

- **`fact_density.dag` is the OTHER HALF of intent-linearity** — it detects a *hollow* carrier (too few
  facts; an anemic/under-decomposed alias). That is §2-**deep** / leaf-side; intent-linearity is
  §2-**horizontal**. Together they are the two inequalities of `description == information`: fact_density
  catches *under* (hollow), intent_linearity catches *over* (redundant), and **1:1 is the equality**. The
  linearity wall is two-sided. **Consolidate** under one `intent-linearity` umbrella; this subsumes *both*
  DESIGN §2 open threads (horizontal redundancy + the parked leaf-side decomposition diagnosis).
- **`table_decision_tree.dag` is the dispatch-ladder species' existing home** (today Unrealized) — it
  already targets "a fn-encoded total table over a closed vocabulary → a substrate `TotalMap`/`TotalPolicy`
  data row." That IS the if/else-ladder→table minimal form (a lookup, not a fold). **Realize it as the
  dispatch-ladder registry row; do NOT fork a new lens.**
- **`registry.dag` is the lens-identity authority** (`LensIdV0`), a registry *of lenses*, not of rules —
  not a fork of the `LinearityRule` registry. But `intent_linearity` rows should **reference `LensIdV0`**
  for lens identity rather than re-coin names (§3).

**Deletes executed this turn: none.** All gated. The map is the deliverable; the triggers are the
schedule.

## Retrofit path (fix all current lenses)

Each existing analytical lens is upgraded from `-> Bool`/`-> count` to *also* produce a corrected
`Node` and (behind a flag) write it. Order by displaced cost, not taxonomy. The detect-only form stays
valid where the fix is undecidable (the ratchet residue) — produce-and-apply is for the **decidable
wall** classes, where the generalization is unambiguous.
