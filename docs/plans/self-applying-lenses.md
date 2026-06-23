# Self-applying lenses — detect → generalize → emit → write

**Crux (fractal intent-linearity).** A program's *description length should equal its irreducible
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
to reference (the Ergonomics lane is what widens this wall).

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

**Deletes executed this turn: none.** All gated. The map is the deliverable; the triggers are the
schedule.

## Retrofit path (fix all current lenses)

Each existing analytical lens is upgraded from `-> Bool`/`-> count` to *also* produce a corrected
`Node` and (behind a flag) write it. Order by displaced cost, not taxonomy. The detect-only form stays
valid where the fix is undecidable (the ratchet residue) — produce-and-apply is for the **decidable
wall** classes, where the generalization is unambiguous.

### Survey of every `src/v2/lens/*.dag` (the classification)

Every lens in the tree falls into exactly one of three retrofit classes. The split is **not** by
taxonomy but by *who owns the fix*:

- **(A) — redundancy row.** Detect is an anti-unification/congruence read over some representation; the
  **generalization it computes _is_ the minimal-form fix** (§4: one read, two directions). These fold
  into `v2.lens.intent_linearity` as registry **rows** of one engine — they do not get a bespoke
  apply-half, they get the registry's. Tag each row with its *binder* (what the varying hole becomes),
  its *scheme* (foldl/scan/reduce/table-lookup — refinement 3), and its *§1 axis* (change-time = source
  redundancy; run-time = computation redundancy).
- **(B) — produce-and-apply (non-redundancy wall).** Not anti-unification, so not a registry row, but
  the flagged state has a **unique, decidable corrective `Node` rewrite the lens can emit and write
  itself** via `v2.lens.application` (`apply_diff`/`substitute_at`). This is the genuine "upgrade the
  lens from flag to fix" case outside the redundancy family.
- **(C) — detect-only (no self-applying upgrade).** The lens stays a flag. Three reasons:
  - **c1 — ratchet residue:** the fix is undecidable or needs content a human must author (global
    optimality by Rice; leaf-side decomposition / domain knowledge; choosing which of two *diverged*
    forks is canonical; supplying a real upstream URL — fabricating any of these is the §5 fail-open).
  - **c2 — dissolves by an _upstream_ construction:** the bad state becomes unwritable when a substrate
    fact lands **elsewhere** (the resolver forbids the edge, `emit(intent, Bash)` owns shell, each
    medium becomes `Medium<R>`, corpus-as-type makes the gap a totality error). The fix is not a
    lens-emitted rewrite, so there is nothing to *produce-and-apply* — it is a projection awaiting its
    authority (§3 "never a second authority").
  - **c3 — mechanism / meta / fixture:** not a detector at all. Notably `application` +
    `application_serializer` are the **write/serialize engine the whole program depends on** (the
    apply-half itself), and `intent_linearity` + `registry` are the registry engine that (A) rows plug
    into — these are *enablers*, never targets.

| lens | class | binder/scheme · axis  (A) / corrective rewrite (B) / reason (C) |
|---|---|---|
| `simulated_relationship` | **A** | **row 1, landed.** list-element binder · foldl · change-time. Its `congruent`/`pair_is_unrolled` is the shared anti-unification **kernel** the other (A) rows reuse. |
| `structural_similarity` | **A** | type-parameter binder · generic-instantiation · change-time. *Keystone:* realize the `Unrealized` scaffold as a row reusing the kernel over the type-decl forest; DELETE the bespoke `TypeShape`/`FnShapeUnrealized` scaffold on the consolidation-map trigger. |
| `identical_variant_payload` | **A** | variant-tag binder (identical payloads → one tag-agnostic parameter) · change-time. Gated on the same producer as `structural_similarity`. |
| `languages_consumer_census` | **A** | data-row binder (~64 identical per-language rows → one row + language list) · change-time. Apply-half is the §3 de-fork migration (repoint consumers to `extdeps/languages/*`), not an in-`Node` fold. |
| `table_decision_tree` | **A** | **table-lookup scheme, _not_ a fold** (refinement 3): fn-encoded if/else ladder → cited rows in `extdeps/` + one generic dispatch (`TotalMap`). change-time. Currently `Unrealized`. |
| `cost` | **A** | run-time. Supplies the `fold_node` cost catamorphism that is the **detect of the run-time rows**; its decidable `(pattern → cheaper-form)` catalog entries become `RunTime`-axis rows. File stays (the catamorphism is the row); what dissolves is "a complexity engine distinct from the redundancy engine." |
| `complexity` | **A + c1** | run-time catalog → `RunTime` rows (A); the `RatchetForever` *global-optimality* residue stays detect-only (c1, Rice). |
| `idempotency` | **A** | run-time (algebraic redundancy: `op∘op → op` / cancellation). Per-edge law-witness today; the simplification rewrite is the row's apply, gated on the closed-algebra carrier. |
| `unused_parameters` | **B** | **flagship.** Corrective rewrite = delete the dead parameter and update every call site — a unique, decidable multi-edit `apply_diff`. Smallest blast radius → the right first produce-and-apply proof. |
| `layering_imports` | **B / c2** | the *delete-a-forbidden-edge* subcase is a unique mechanical rewrite (B); *re-homing* a shared decl to the right layer is advisory, and the class dissolves anyway once the resolver rejects the inverted edge by construction (c2). |
| `affected_set` | **C/c2** | re-exec frontier projection; dissolves when the scheduler consumes the frontier as a construction. |
| `coverage` | **C/c2** | missing-set is decidable, but the fix is a *handler body* a human writes (a stub arm would be §5 fail-open); dissolves at corpus-as-type. |
| `mock_totality` | **C/c2** | same as `coverage` — totality gap over declared-data corpus; dissolves at corpus-as-type. |
| `discrimination` | **C/c2** | the lens cannot synthesize a discriminating red *input* (needs the semantics); dissolves when the witness-authoring surface forbids a green-only unit. |
| `edit_locus` | **C/c2** | diff-path → `Node` projection over `NodeArtifactProvenance`; shim until the provenance carrier owns it. |
| `effect` | **C/c2** | per-edge effect projection; dissolves when the signature-derived effect-kind set is closed in the substrate. |
| `ownership` | **C/c2** | per-edge alias/ownership projection; dissolves when the closed access carrier lands on `InferredFacts`. |
| `parallelism` | **C/c2** | per-edge coupling projection; dissolves when the coupling carrier lands on `InferredFacts`. |
| `structural_resolution` | **C/c2** | projection-only over the resolver's binding facts — never a second authority. |
| `resolved_imports` | **C/c2** | dangling-import projection of the resolver's `UnresolvedImport` rule; fix = author the missing module (external). |
| `visibility` | **C/c2** | two valid fixes (restrict publication vs promote import) → ambiguous; dissolves into `access.dag`'s effective-public rule. |
| `extdeps_shape_transport_policy` | **C/c2** | the four tells dissolve when the §3 interface/transport/policy de-fusion is realized by construction. |
| `host_language_transport_script` | **C/c2** | literal-in-script-slot; dissolves when `emit(intent, Bash)` owns shell construction. |
| `realization_vocabulary_containment` | **C/c2** | target-AST sidecar import outside the realization edge; shrinking-roster ratchet → wall when the arc empties. |
| `medium_structure_containment` | **C/c2** | medium structure leaking to a raw string; shrinking roster → wall as each medium becomes `Medium<R>`. |
| `leaf_model_verification` | **C/c2** | emitted-code behavior correctness (R1–R3); dissolves when host-exec (T-22) replaces structural pair-readiness. *(Note the irony: its own ~10 fixture pairs are an (A)-shaped redundancy — a self-application target once the kernel runs on the lens corpus itself, §7.)* |
| `fact_cardinality` | **C/c1** | redundancy-*detect* (cross-tree coexistence), but choosing which of two **diverged** forks is canonical is a judgment → apply non-unique. (Contrast `languages_consumer_census`: identical copies → unique fold = A.) |
| `fact_density` | **C/c1** | hollow-alias detect is a wall, but *what to decompose the leaf into* is §2 leaf-side domain knowledge — undecidable. |
| `unit_modeling` | **C/c1** | whether a bare scalar denotes a modeled quantity needs domain knowledge (is `4926` a contact_count?) — `RatchetForever`. |
| `extdeps_external_authority` | **C/c1** | anchor-presence is decidable, but the fix is the *real upstream URL* a human supplies; fabricating one is the §5 phony-anchor fail-open. |
| `synthesis` | **C/c1** | lower-bound gap is provable, the closing program is not (`feedback_no_engine`; matmul-ω) — permanent advisory. |
| `testgen` | **C/c1** | test-generation / gap heuristics; `RatchetForever`, no constructive close-loop. |
| `application` | **C/c3** | **the apply engine** (`apply_edit`/`apply_diff`/`substitute_at`) — the produce-and-apply write substrate every (A)/(B) lens calls. Enabler, not a target. |
| `application_serializer` | **C/c3** | **the serialize half** (emit = grammar inverse → `DagSourcePatch`). Enabler, not a target. |
| `intent_linearity` | **C/c3** | the **registry engine** (A) rows plug into. Not a target — it is the host. |
| `registry` | **C/c3** | the lens-identity registry (interim `LensIdV0` map). Meta. |
| `subsumption` | **C/c3** | a meta ledger of which root-fix mechanically closes which leaf-fixes; not a code detector. |
| `affected_set_examples` | **C/c3** | example/fixture data for `affected_set`. |
| `visibility_test` | **C/c3** | witness file for `visibility` (a `*_test.dag`, floor-discovered). |

### Priority order (by displaced cost, not taxonomy)

The prerequisite gate for **every** apply-half: emit (§6) and `application.dag` exist; what is missing is
(i) a **filesystem write effect** (the write twin of the lenses' `filesystem_read`) and (ii) the live
**resolve walk** producing the per-body / per-decl `Node` facts the detectors consume (today they run as
`lens_unit` over synthetic input). Until both land, the produce-half is provable on synthetic input but
not wired to the corpus.

1. **The §3 dup-decl family — `structural_similarity` + `identical_variant_payload` +
   `languages_consumer_census`.** Highest displaced cost: a fork is *always* consolidated later, and the
   debt re-duplicates through everything generated (testgen, emit, lenses). All three share **one
   producer** (parse/resolve type-/data-decl facts) and **one kernel** (the term-layer `congruent`
   generalized to the type-parameter / variant-tag / data-row binder) → land as one consolidation.
2. **`simulated_relationship` row 1** — already landed; the unblock is the live statement-chain walk so
   it fires on `dsl/**` instead of synthetic chains.
3. **`cost` / `complexity` run-time catalog rows** — the `O(n^x) → O(n log n)` substitution catalog;
   real run-time wins, but the rows must tag the run-time axis (the honest disanalogy: some source-DRY
   rewrites are O(1)-runtime and vice-versa).
4. **`unused_parameters` (B)** — the cleanest non-redundancy produce-and-apply: smallest, fully decidable
   multi-edit rewrite. The right proof-of-concept for the write effect end-to-end.
5. **`table_decision_tree`** — needs the decision-tree-shape producer **and** the `extdeps/` dispatch
   target (its minimal form is rows + one generic dispatch, not a fold).

Everything in **(C)** stays detect-only by design — c1 is the honest `RatchetForever` residue, c2 is a
projection that dissolves when its single authority lands upstream (not a lens-applied rewrite), and c3 is
the engine itself. None of these is a regression in the produce-and-apply program; they are the boundary
of where it correctly stops (§5: check decidability before claiming a wall).
