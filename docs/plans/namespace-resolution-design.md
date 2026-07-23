# Namespace-only resolution — the graph is the sole naming authority

**Provenance.** Operator ruling, 2026-07-06 (session loyal-dove-903), two governing rules
for name resolution (below). This **supersedes** the import-name-universe half of
`resolver-graph-major-design.md` §1c (the "name universe = own declarations ∪ direct
import lists" model) and **carries forward** §1c's resolution *mechanics* (constructor
owner via the binding edge; context-bare in typed positions; no expected-type owner-*picker*).
It absorbs and retires the selective-import mask thread (`source_visible_names`,
`resolve_node_bounded_masked_boundary`, `UnlistedImportUse`, family-closure) — see §7.

---

## 1. The two governing rules

Stated by the operator as the acceptance invariants for this feature (both are instances
of the DESIGN axioms — Rule 1 is §2/§3, Rule 2 is §4/§5 — restated as the operative bar):

- **Rule 1 — no dual representation; minimal representation possible, at every layer**
  (including *between* features and recursively in the frontend).
- **Rule 2 — no ambiguity, at any layer.**

Everything below is a *consequence* of these two, not an independent choice.

## 2. What the rules force (the derivation)

1. **Imports are deleted as an authority.** An `import M { A }` list is a *second* encoding
   of what a module uses — the references in the body are the first. Two encodings of one
   fact is a Rule-1 violation. → the reference is the **sole** representation of usage; the
   module's dependency set is *derived* (the set of resolved targets), never declared.
2. **Reachability is structural, not declared.** A "visible set" (`source_visible_names`,
   an import surface) is itself a declared dual-representation of reachability. Rule 1
   forbids it. → reachability is a node's **position in the graph** (its `module` path is
   its place in the namespace tree); the tree *is* the scope. Nothing to declare.
3. **Qualification is minimal.** Writing more path than is needed to make a target unique
   is redundant representation (Rule 1). → you write the **shortest path suffix that
   resolves uniquely**, and no more. `Node` if unique in scope; `node.Node` if not;
   `std.node.Node` if *that* still collides; up to the full path.
4. **Ambiguity refuses, never guesses** (Rule 2). A name resolving to ≥2 nodes is a typed,
   located `Ambiguous(candidates)` error — never a flat-namespace pick, never an
   expected-type tie-break (§1c rule 5 stays deleted). You resolve it by qualifying.

## 3. The model — ONE structure: the containment tree (operator, 2026-07-06)

There is not a namespace mechanism *and* a scoping mechanism *and* a field-access mechanism to
reconcile. There is **one relationship — syntactic containment — and it induces all of them at
once.** Model that tree cleanly and resolution has no knobs to tune; the tuning risk (this
section's earlier "nearest-wins / smallest-enclosing-subtree" drafts) came entirely from
treating resolution as a fuzzy search over a flat index instead of a walk up one tree, and from
treating the module-namespace tree and the value-nesting tree as *separate* — they are one tree.

**The containment tree.** `module v2.std.node` places a file's contents at a position;
`T { a { b } }` extends the path *below* it (`…T.a.b`); a coproduct's arms sit under it
(`…CheckConclusion.Success`). Module-nesting and value-nesting are the same tree. Four facts
fall out of a node's position in it, none separately specified:

1. **Qualified name = containment position.** `a` inside `T` *is* `T.a`; the inner `a` in
   `T { a { a } }` *is* `T.a.a` — distinct from `T.a` because its *position* differs. Identity
   comes from position, so **Rule 2 is satisfied by construction** — there is nothing to
   disambiguate that the nesting has not already disambiguated.
2. **Reference = lexical lookup up the ancestor chain.** A bare `name` resolves to the nearest
   enclosing binding on `position`'s ancestor chain; inner shadows outer (different positions =
   different names, no conflict). A sibling *module* is visible (it is a child of your parent, an
   ancestor); that sibling's *members* are **not** bare — you project into it (`node.Symbol`).
   Kernel is the root, so it is on every chain (bare everywhere).
3. **`.` = projection one level down that tree** — field (`t.a`), module member (`node.Symbol`),
   variant (`CheckConclusion.Success`): the *same* operation, descend into a named child.
4. **`.` is also the sub-value relation.** `t.a` makes `a` a strict-sub-value of `t`
   (`std.induction SubValueRelation`), which is how structural recursion is proven to descend.
   So containment → name → scope → `.` → descent-evidence are **one structure**, and
   `type-env-single-authority`'s `ScopeBinding { provenance }` already carries exactly it.

**Outcome (the total function is a tree walk, not a search):** lexical lookup finds exactly one
nearest binding → **resolve** (the reference *is* the dependency edge); two bindings at the same
nearest level → **`Ambiguous`** (qualify by projecting from a container that disambiguates); none
up to the root → **`Unresolved`**. Qualification is projection (`node.Symbol`), minimal by
Rule 1 (project from the shallowest container that resolves uniquely; over-qualification is a
lint, not unsafe). No expected-type *picker* (§1c rule 5); (Y) supplies the *container* in a
typed position (a variant's expected type), which is projection-with-the-container-implicit, not
a tie-break.

**Front/back split (the operator's core ask — model it once, cleanly):**
- **Frontend builds the tree** from nesting: each node's qualified path = its position. Pure
  structure, *zero resolution logic*. This is precisely what `SymbolIndex` should be — **the
  containment tree materialized** (path → node), derived from nesting, not a separate index with
  its own reach rules.
- **Backend walks it**: lexical lookup up (references), projection down (`.`). Trivial and
  rigid, because "which nodes can I see" is just "which nodes are on my ancestor chain" — the
  tree already answers it; there is no heuristic to tune.

**Namespace root.** A file declares its own path (`module v1.compiler.resolve`, 03_resolve.dag:1)
= its position in the tree. The source-root→namespace-root binding (`src/v1/**` ↔ `v1.*`,
`dag/**` ↔ `v2.*`) lives at that boundary; §9 flags confirming the exact map and whether the
`module` declaration is recoverable from the path (if so, the declaration is itself a Rule-1
candidate to derive rather than author).

## 4. The uniqueness test — DECIDED: (Y), position-information (operator, 2026-07-06)

"Resolves to exactly one node" — *given what?* Two readings; **(Y) is the ruling.**

- **(X) Pure structural.** A name resolves iff unique in the structural subtree; the expected
  type is never consulted. Simple, uniform, no expected-type anywhere. **Cost:** every use of a
  shared arm name (the `github` `Success`/`Failure`/`Cancelled` family; `Unit`) must qualify
  *at every site*, including in fully-typed positions where the type already determines it.
- **(Y) Position-information — THE RULING.** A reference resolves to exactly one node given
  *everything known at its position* — the structural subtree **and** the expected type when the
  position is typed (a return annotation, a match scrutinee, a field type). `Success` in a
  `-> CheckConclusion` position: 6 structural matches, but the expected type filters to one →
  **unambiguous, bare**. `let x = Success` (untyped) → 6 survive → **`Ambiguous`, qualify**.

**(Y) honors both rules.** It is still "exactly one node" (Rule 2) — the expected type is a
*filter that narrows to uniqueness*, not a *picker among survivors*, so it is **not** the §1c
rule-5 pathology (unbound name → scan all types → tie-break). And it is minimal (Rule 1): you
don't restate what the position already fixes. It generalizes §1c rule 4 (patterns resolve via
scrutinee type) from pattern to any typed position. **Decided (Y)** (operator, 2026-07-06); §5
shows why the cost of (X) is real and concentrated. Implementation consequence: `resolve` takes
an optional `expected: Node?` — present in checking positions (annotation, scrutinee, field
type), absent in synthesis positions; when present *and* the structural candidate set is >1, it
filters to variants/members of `expected` before declaring `Ambiguous`.

**RE-ADJUDICATED — SUPERSEDED BY §13 (operator ruling, 2026-07-22).** The (X)/(Y) menu above
presupposed the `global_bare` pool tier: bare `Success` only ever "has 6 structural matches" if
the whole pool is a candidate set. §13 deletes that tier — a bare name's candidates are the
binders on its **ancestor chain**, never the pool — so the pool-filter reading of (Y) is *dead,
not chosen against*. Operator (2026-07-22): similar instances across the repo are **not**
associated by shared literal name — *"there has to be some kind of logical connection"*; a bare
name with zero chain binders is `Unresolved` ("what are you referring to?"), never `Ambiguous`
among pool homonyms. What survives of (Y) is one residue, **demoted to sugar** (§6 amendment):
in a typed position whose container type is **syntactically local** (a written annotation — an
annotated scrutinee, `let x: T`, a field/return type), a bare member name desugars to
`T.<name>` — a grammar-level rewrite, decidable without inference, loud on any edit to the
local annotation. Flow-typed positions (the container arrives by inference from another
declaration) get **no sugar** and must qualify or alias: that is exactly where desugaring would
become type-directed elaboration, and where the silent-rebind hole lives (editing `f`'s return
type would silently re-target `match f() { Success => .. }` — the §13 edit-stability invariant
forbids it). The sugar boundary and the safety boundary coincide — which is *why* the residue
is sugar, not semantics. Implementation consequence updated: the sugar is a **pre-resolve
desugar pass** (bare member name + syntactically-local container annotation → `T.<name>` before
`resolve` runs); `expected: Node?` is **deleted from the resolver contract** — `resolve` takes
`(name, position)` only, and no expected-type consultation exists in the core resolver (one
authority; the §6 mechanics bullet states the same).

## 5. The collision census (empirical grounding, 2026-07-06)

Declared-name census over 942 `.dag` files (`dag/**` + `src/v1/**`), leaf name × declared
module (dir-proxy for the declared `module` path — §9 for the precise pass). The number Rule 2
actually forces to qualify:

| | count | share |
|---|---|---|
| distinct declared names | 13,326 | |
| **globally unique → always bare, zero qualification** | **13,078** | **98%** |
| names with >1 definition (collision candidates) | 248 | 2% |
| — same-dir siblings (genuinely co-visible) | 92 | |
| — shared-prefix subtree split | 107 | |
| — disjoint top-level (ambiguous only from a high ancestor) | 49 | |

The 2% decomposes into four kinds, and only a sliver is genuine forced verbosity:

- **§3 forks — Rule 2 is a Rule-1 detector.** `Deterministic`/`NonDeterministic`
  (`std.behavioral` vs `std.determinism`) are a fork the corpus *already documents*: both
  files carry scaffold markers ("honest §3 fork until projection lands"; "ONE core authority
  … not re-declared per domain"). `True`/`False` (`std.logic Classical` vs `std.types Bool`),
  `Unit`/`Credential`/`ApiKey`/`Json`/`Empty`/`IdentityEffect` (`std.types`/`std.*` twins) are
  the same shape. For these, the Rule-2 collision *forces the consolidation Rule 1 wanted
  anyway* — you fix them by deleting the fork, not by qualifying.
- **Genuine homonyms.** `Unit` = `TestClass` arm (`fidelity`) vs the unit *type* (`types`):
  genuinely different concepts sharing a leaf. These *do* qualify (`fidelity.Unit` /
  `types.Unit`) — decidable, few.
- **Shared arm names across distinct enums.** The `github` conclusion families (`Success`,
  `Failure`, `Cancelled`, `Queued`, `InProgress`, `Skipped`, `TimedOut` across
  `actions`/`checks`/`workflow_runs`). Under **(Y)** these resolve by context in typed
  positions (the overwhelming majority — a conclusion is always used at a typed site) and
  qualify only in the rare untyped position. Under **(X)** they qualify everywhere — this is
  the concentrated cost of (X), and it lands squarely on the extdeps/github + workflow code.
- **File-local helpers.** `main`, `nid`, `child_node`, `h2`/`p`/`li`/`ul` (doc emit),
  `list_contains_string`. Under lexical scope each is used *within its own file* — never
  actually ambiguous. Zero qualification. (The `h2`/`p`/`li` doc-helper repetition across
  plan files is itself a Rule-1 fork worth a shared authority, but that is orthogonal.)

**Reading:** the true "verbosity forced by Rule 2" is far below 248 — it is (homonyms) +
(untyped-position uses of shared arm names), a few dozen at most under (Y). Everything else is
either already bare, a fork that Rule 1 deletes, or file-local. The census doubles as a
**fork worklist** for the operator's broader §3 audit.

### 5.1 Precise census — declared `module` paths (2026-07-06)

Re-run on the declared `module X.Y.Z` path (not the dir-proxy), split by decl-kind. 2,092 type
names / 2,673 variant names / 8,340 fn names; **collisions: 26 types, 103 variants, 23 fns.** The
sharper split changes the reading in one important way — **most collisions are the v1-seed-vs-v2-std
fork, which nearest-wins resolves for free:**

- **v1-seed re-declarations** dominate. `v1.compiler.languages` re-declares `std.languages`
  (`ForEachSyntax`, `ImportRule`, `LanguageSpec`, `NamingCase`, `ReservedWords`, `ProjectScaffold`…);
  `v1.std.core` re-declares `std.constructors` (`Cardinality`); `v1.compiler.{parse,infer,resolve}`
  share internal result types (`FieldResult`, `ItemResult`, `VariantResult`). These are the
  DESIGN §7 shrinking-seed duplication, **not consolidation targets now** — and crucially they
  are in **disjoint subtrees** (`v1.*` vs `std.*`), so nearest-wins resolves each at its use-site
  (v1 code sees v1's, std code sees std's) with **zero qualification and zero ambiguity**. They
  dissolve as the seed shrinks, not by qualifying.
- **The 2 same-name pairs the census flagged are NOT consolidate-now forks** (verified by reading
  the decls, 2026-07-06): `ImportTrigger` (`dag/std/languages.dag` vs `src/v1/languages.dag`) is a
  genuine duplicate but one of **8 types in a whole-file v1-seed mirror** — DESIGN §7 seed-shrink
  territory, dissolves with the seed, not standalone (and editing the seed is regen-risky).
  `Reconciliation` is a **homonym, not a fork**: `std.realization_reconcile.Reconciliation<A,E>`
  (`Converged|NotConverged`, a generic effect-grounding outcome) vs `product.budget_tree.Reconciliation`
  (`AllSatisfied|Evicted|GuaranteedShortfall`, a budget-admission outcome) — *different concepts and
  variant sets*; the "identical variant-set" was a census variant-extraction artifact, and
  consolidating them would be wrong. Nearest-wins resolves each in its own subtree. **Net
  consolidate-now forks the census forces: zero** — the same-name residue is homonyms (resolve by
  subtree) and seed-file duplication (dissolves with the seed).
- **Same-subtree variant collisions** — the `github` conclusion families (`Success`/`Failure`/
  `Cancelled` across `extdeps.github.{actions,checks,workflow_runs}`, `CheckConclusion` itself
  actions-vs-checks with *different* variant sets) — are genuinely co-visible in github code and
  are the **(Y) context-resolution** population: bare at typed sites, qualify only in untyped ones.

**Net:** under (Y) + nearest-wins the forced-qualification residue is a **handful** — the
same-subtree homonyms with no typed context — because (a) cross-tree seed forks resolve by
subtree, (b) github-style collisions resolve by expected type, (c) file-local helpers never
collide. The census's standing value is the clean quantification of the v1→v2 seed-duplication
surface (dissolves with the seed) — it forces no consolidate-now forks; its same-name flags are
homonyms and seed-file duplication, not genuine cross-tree §3 forks.

### 5.2 Tier-2 homonym triage receipt (2026-07-16, quiet-carp-507)

**Deliverable:** tier-2 triage proved `ConstructionProtocolNoAction` on the two
highest-rank roster ghosts; consolidating either would destroy information.

- **`live_tree_disposition`** — entry-grain witness stamp (`v2.std.live_tree`); 691 modules
  each declare their own row with differing values (633 `SubstrateInputsOnly` / 67
  `ReadsLiveTree`). Not one concept forked 691 ways — per-entry construction protocol.
- **`extdeps_external_authority_anchor`** — per-extdeps-module upstream cite row (same fixed
  leaf name as `construction_justification` on lenses); 219 distinct locator values across
  285 modules. Not a §3 fork.
- **`emit`** — genuine homonym (four fn authorities) but zero cross-subtree bare `emit` sites;
  nearest-wins subtree resolution handles today. Policy home: §5 nearest-wins / §4 projection,
  not a per-fn construction stamp.

**Enforcement:** `src/v1/tests/src/global_bare_corpus_census_test.rs`
`tier2_namespace_homonym_invariants` — two legs: (1) census ambiguity — UNIQUE
proceeds, AMBIGUOUS reds, ABSENT refuses the census leg (not treated as non-ambiguous;
precedent: `fold`); (2) reachability — 0 ambiguous bare refs for construction-protocol
names, 0 cross-subtree bare `emit` refs (always evaluated). Not declaration counts.

## 6. Resolution mechanics carried from §1c (unchanged, re-homed)

`resolve(name, position)` *is* the §1c constructor-owner ruling generalized from constructors
to all names:

- **Binding edge, not scan.** A constructor's owner still rides its binding (§1c rule 1);
  under namespace-only the "binding" is just the resolved node's parent edge in the tree.
- **No expected-type picker** (§1c rule 5) — and per the §4 re-adjudication (2026-07-22) the
  expected-type *filter* is dead with the pool tier; no expected-type consultation survives in
  the core resolver.
- **Patterns via scrutinee** (§1c rule 4) is **demoted to sugar** (§4 re-adjudication): valid
  only when the scrutinee's container type is syntactically local (a written annotation), where
  it is a grammar-level desugar to `T.<name>`; a flow-typed scrutinee's arms qualify or alias.
- **Collision at env construction** (§1c rule 3) becomes the general `Ambiguous` outcome.

## 7. What dissolves (all Rule-1 dual-representations)

- `import` blocks — dual rep of usage (kept only as optional **`alias p = a.b.c`** sugar: pure
  path abbreviation, carries *no* visibility, decoupled from dependency).
- `source_visible_names` and the whole visibility side-map — dual rep of reachability.
- the `masked` bit, the `resolve_node_bounded_masked_boundary` invariant, `UnlistedImportUse`,
  the diagnostic-collect advisory, family-closure — *all* were machinery to reconcile the
  import list with usage. No import list → nothing to reconcile → they are deleted, not
  hardened. (This retires the memory thread `resolve-selective-import-fail-open`.)
- `get_exported_names` re-export of `specific_names` (the two-hop leak) — no export list.
- **`parameterized_use_site_prefers_parameterized_decl` (04_resolve.dag:231)** — the retry that
  prefers an imported generic `Optional<T>` over the paramless kernel `Optional` (a param-count-
  keyed kernel-vs-import precedence patch). This is a *second precedence path*, and nearest-wins
  subsumes it uniformly: the imported generic is module-local (nearer) and wins over root-kernel
  by scope depth, no param-count special-case. **Dissolve-on: the pivot** — remove it *with* the
  policy flip, so it is not left as a competing precedence mechanism (Rule 1). (flagged
  lively-raven-355, 2026-07-06; same root as the kernel-`Nat` shadow bug below.)

The original "fail-open" (`Symbol` resolving in `diagnostic.dag` without an import) **was not a
bug** under these rules — it is correct unique resolution. The bug was the *import declaration*
that made it "undeclared"; Rule 1 deletes that layer, and the wall moves from *"did you import
it?"* to *"is it unique?"* (Rule 2).

## 7.5 Substrate — this rides on the `SymbolIndex` (do NOT fork the index)

This design is the **resolver half** of a whole whose **index half** already exists:
`docs/plans/type-env-single-authority-design.md` (owner cool-hawk-899; realization lane
lively-raven-355). That doc builds `SymbolIndex : Map<QualifiedName, Node>` (04_env.dag:36) as
the *single* qualified-name authority, filled once by a topo-order DFS prepass, replacing the
O(M²) `ancestry_str_bindings` materialization. **That `SymbolIndex` is exactly the index
`resolve(name, position)` queries** — Rule 1 forbids a second one. The two docs compose cleanly:

- **`SymbolIndex` = the containment tree materialized** (qualified path → Node, one authority —
  §3: the frontend builds it from nesting, it is not a separate index with its own reach rules).
  Shared. Not mine to re-build; I consume it.
- **`resolve(name, position)` = the semantics over it** (unique-on-chain lookup, §13 +
  `Ambiguous`/`Unresolved`; no expected-type consultation — the (Y) filter formerly named here
  died in the §4 re-adjudication (2026-07-22), and its locally-annotated residue is a
  pre-resolve desugar pass, never a resolver input). Mine.
- **The one genuine difference is a policy value, not a conflict.** type-env-single-authority
  keeps the *import list as the visibility gate* ("a module's import list says which qualified
  names are visible" — its §3). Namespace-only **deletes** that gate and resolves by structural
  position instead. These are the two values of the §8 `ResolutionPolicy` row —
  `import-scoped` (their v1 perf fix, ships first) and `namespace-only-Y` (this pivot, on top).
  Same `SymbolIndex` underneath both.

**The invariant that makes "one index, two policies" sound (lively-raven-355, 2026-07-06):**
the **fill stays policy-agnostic** — the topo prepass fills *everything import-DAG-reachable* —
and the `ResolutionPolicy` gates **lookup only, never fill.** `import-scoped` filters the filled
map by the import list at lookup; `namespace-only-Y` filters by structural position at lookup;
both read the *same* fully-filled `Map<QualifiedName, Node>`. If a policy ever narrowed the
*fill*, the two policies would each materialize a different index — the exact dual-representation
Rule 1 forbids. So the guard is explicit: **policy is a lookup filter; fill is one, complete, and
shared** — which also keeps the O(M²) fix policy-agnostic (fill-once stands under either policy).

**Sequencing consequence:** `SymbolIndex` lands first (it is the substrate *and* the O(M²) fix).
**LANDED — #6809** ("SymbolIndex / type-env-single-authority — the containment-tree naming
authority"), carrier `src/v2/std/symbol_index.dag`:
`SymbolIndex { entries: Map<QualifiedName, Node>, global_bare: Map<Symbol, GlobalBareBindingState> }`
with `symbol_index_lookup` / `symbol_index_global_unique_lookup` / `symbol_index_lexical_lookup`,
and `GlobalBareLookup = GlobalBareHit | GlobalBareLookupAmbiguous | GlobalBareLookupUnbound`.
The loyal-heron scaling gate is **discharged**; this precondition is satisfied and the lane is no
longer blocked on it. Namespace-only is now a policy layer over a *settled, in-tree* index — not a
from-scratch resolver. Resolution is a **walk over the
containment tree** the `SymbolIndex` materializes (§3: lexical lookup up the ancestor chain, `.`
projection down), not a fuzzy key search; the kernel-`Nat` / `:231` / family-closure /
`source_visible_names` collapse (§7) all happen *at* this seam. Reconciliation owed: fold the `import-list-visibility` assumption in
type-env-single-authority §3 into the policy row so it reads as one setting, not the law.

## 8. Migration — per-subtree, behind a swappable policy

Model the choice as a **`ResolutionPolicy` row** (§7 DESIGN: language design is a row) with
values `{ import-scoped (§1c, today) , namespace-only-X , namespace-only-Y }`. Then:

1. Land `resolve(name, position)` + the `Ambiguous`/`Unresolved` outcomes beside the current
   resolver, gated by the policy row (import-scoped stays the default → zero corpus churn).
   **LANDED 2026-07-22** in the executing v1 seed under the §13 unique-on-chain semantics
   (which supersede this section's original nearest-wins wording) — see the §13 "STEP 1
   LANDED" entry for the mechanism, edit sites, and the discriminating witness.
2. The **precise census is already run** (§5.1, declared `module` paths): the exact
   forced-qualification residue and the fork worklist are its outputs — steps 3–4 act on them,
   no re-run.
3. Consolidate the §3 forks the census surfaced (Rule 1 work that stands on its own).
4. Flip the policy to `namespace-only-Y` **per subtree** as each converges (drop its imports,
   let bare resolve, qualify the genuine homonyms) — not big-bang. A subtree is converged when
   it resolves clean with zero `Ambiguous`.

   **Discriminating witness (live red today, lively-raven-355):**
   `cross_representation_equality_test` (`src/v1/tests/src/cross_representation_equality_test.rs`),
   fixture `RECEIPTS_SOURCE`, `import v2.std.nat { Nat, Succ, Zero, nat_add }` where
   `v2.std.nat` = `type Nat = Zero | Succ { prev: Nat }`. It quadruple-discriminates the policy:
   - **today (kernel-first + kernel_type_set has "Nat"):** `Err(NoSuchVariable{Zero})` — ambient
     kernel `Nat` wins globally, variant family silently dropped (the fail-open);
   - **`namespace-only-Y`:** lexical lookup binds `v2.std.nat.Nat` (nearer on the ancestor chain
     than kernel root, which it shadows) → `Zero`/`Succ` bound → `Bool(false)`/`Bool(true)`;
   - **same-depth two-`Nat`:** `Ambiguous`, never silent;
   - **fork-removed (no kernel "Nat"):** trivially the one coproduct → same greens.
   Reproduce the red in isolation by adding `"Nat"` to `kernel_type_set` on main (lively-raven's
   #6325 delta). This is §8's acceptance witness — it must go from `Err(NoSuchVariable)` to the
   coproduct greens under the flip, with a same-depth control asserting `Ambiguous`.
5. **TERMINAL STEP — delete the import syntax itself (operator directive, 2026-07-06).** Once
   every subtree has flipped and no module depends on import-scoped resolution, **delete the
   `import` grammar production and its supporting resolve/env code** so `import …` becomes an
   *actual parse error*, not an inert-but-tolerated form. Imports do not "retire to alias-only"
   — they are removed. The dependency graph (build order, `SymbolIndex` fill traversal) is then
   **derived from the `container.member` references themselves** (each reference names its
   container → that IS the edge), so the import statement is redundant on *both* its axes
   (visibility *and* dependency-declaration) and leaves nothing behind. Rule 1 end-state: one
   representation (references), zero vestigial syntax. Sequencing: this is LAST — deleting the
   production while any unmigrated module still uses `import` would parse-error that module, so
   the grammar drop lands only after corpus-wide flip is green.

The current selective-import mask is thereby the **interim realization** of the
`import-scoped` policy value — it is not thrown away mid-flight, it is the thing the policy
flip retires, subtree by subtree, until step 5 deletes the syntax outright.

**Import-from-definer migration (PR-4 scope; census seeded 2026-07-07).** The corpus currently relies on
**re-export transitivity** — `import M { X }` where `M` re-exports `X` from the module that actually defines it
(proven by execution in `type-env-single-authority-design.md` §3.1: `compile.dag` imports `EmitResult` from
`v1.compiler.emit`, which re-exports it from `emit_core_support`). The PR-2 perf reform *preserves* this (own
bindings + a memoized re-export-chain walk), so it is byte-identical. This step (PR-4) **migrates each import to
name the *defining* module**, eliminating re-export reliance so `container.member` references derive the true
dependency edge (Rule-1 end-state). First concrete census rows from the PR-2 refutation receipt: `EmitResult`,
`parse_with_table`, `default_artifact_plan`, `Rust` — extend by re-running the direct-import experiment and
collecting every unresolved-name error.

**PR-5b import-line strip (interim host bridge).** Deletes `import` lines from
`src/v2/workflow/**` and `src/v2/extdeps/**` while bare refs stay; the v1 host keeps
compile and witness execution working via the **census bare-reference closure**
(`extend_with_bare_reference_closure` in `src/v1/stage0/src/cli_run.rs`): an
import-stripped module's bare names resolve through the per-tree census (own tree
first, whole pool on miss — the same layering typecheck lookup uses), and each
resolved name pulls its declaring module into the entry closure. Precision guards,
both declaration-grain: names the referencing file itself binds anywhere (params,
named-arg keys, `let`/`data` binders) never pull, and a name whose resolved
declaration is a `test fn`/`test data` row never pulls (an execution root is not a
dependency). A pulled module missing from module-graph-facts provenance refuses
loudly. An earlier draft of this paragraph described a git-show import-replay
scaffold (`CLI_RUN_NAMESPACE_IMPORT_SYNTHESIS_SCAFFOLD_MARKER`) that was never
built — the census closure above is the mechanism that actually shipped, and the
known residue (the reference producer over-collects; the closure is a second
authority beside the compile-clean import closure) is tracked as the
reference-derived dependency-edge lane, the same lane that dissolves this bridge.
→ [layering-imports gate repoint scoping](layering-imports-reference-repoint-design.md) (CI `LayeringImportsGate` fact producer; Phase 1 lands before import strip reaches `std`/`extdeps` scan roots).
**Dissolve-on:** `^migrate_when_namespace_only_resolution_lands` (terminal step 5
above — delete import grammar; container.member references become the sole
dependency authority).
**AMENDED 2026-07-22 (execution-diagnosed, merry-heron-629):** the closure's adequacy is
**pool-membership coincidence** — a stripped file's bare names resolve iff the target module
happens to be in the shared pool via *some unrelated unstripped import in the closure*; there
is no per-file binding mechanism, so already-stripped files (batch-1's ~74) are green only by
ambient coverage that any later unrelated strip can erode. **All further stripping is blocked
corpus-wide** until a closure-independent binding mechanism (substantively: namespace-only
resolution reaching the typecheck env — the §8 flip) or a provable-coverage construction check
lands; additionally a strip wave must be closed under the imports-from relation (or PR-4 land
first) — partial strips sever re-export chains at hub files. Mechanism receipts, controlled
experiments, and the consolidated wave rule:
[import-strip witness-discovery cascade diagnosis](import-strip-witness-discovery-cascade-diagnosis.md)
§12–13 (PR #7061).

## 9. Open / to-verify

- **Root-prefix map** (`src/v1/**` ↔ `v1.*`, `dag/**` ↔ `v2.*`). ~~Whether the `module`
  declaration is derivable from the path~~ — RESOLVED THE OTHER WAY (operator, 2026-07-19, §10
  anti-goal): deriving the namespace position from the file path would harden the
  storage/identity fusion the module-identity lane dissolves. The declaration stays; its
  *extent* becomes syntactic (§10). Note `04_infer.dag:718` already special-cases the `"v2."`
  prefix — that logic folds into the root map.
- **Non-locality property (accepted, not a bug):** adding `v2.std.other.Symbol` later makes a
  previously-bare `Symbol` in-scope `Ambiguous` — a compile error until qualified. Loud and
  fail-closed (Rule 2), the honest cost of "no ambiguity." The alternative (narrower per-module
  scope) kills the sibling-access ergonomics the operator asked for; not taken.

## 10. Namespace scope is syntactic — the header declares a position AND an extent (operator, 2026-07-19; routed from sharp-bee-290, addendum on #6848)

The defect: `module examples.cost_estimate` declares WHERE the file's declarations sit but not
HOW FAR that claim reaches — the grammar row (`dag_grammar_module_header_expr`,
`src/v2/extdeps/languages/dag.dag`) is `kw_module × qualified_name`, no extent token; scope is
inferred from end-of-file, i.e. from the storage medium's boundary. Operator framing verbatim:
"there is no scoping on modules right now — we're inferring it from the file boundaries; I'd
like to make this all very explicit up front." This is the storage/identity fusion
(module-identity-storage-binding lane) surfacing at the naming layer. Verified downstream
symptoms of the special-casing: 03_resolve's dedicated metadata edge
(`dag_surface_module_header_metadata_edge`, `ctx.under_module_root`) instead of ordinary
containment fill; SymbolIndex fill carrying the qualified name as a string prefix bolted onto
keys (`module_qn.fn.param`) rather than literal containment nodes; "one file = one module"
enforced as a file-grain collision key standing in for the tree-grain rule it means.

The explicit form is this design's own semantics applied one level up: `module a.b.c` + body ≡
the file's declaration forest grafted under `a { b { c { … } } }` — literal containment nodes,
extent = the brace pair, nothing inferred from storage. In the model there is then no "module"
concept: containers all the way down (the header is C#'s file-scoped `namespace X;` — a
placement declaration, never a `using`; that role belongs to `import`, which §7's terminal step
deletes).

**Staged (each with its trigger):**
1. **Header as sugar — zero surface migration.** `module a.b.c` desugars at ingest to the
   containment graft. Frontend builds the tree from nesting (this design's "pure structure,
   zero resolution logic"); the resolver metadata-edge special case and the SymbolIndex prefix
   bolt-on dissolve into ordinary containment fill. Every file keeps its header; only the model
   changes. After this step "modules have no scoping" is false in the model: the scope IS the
   subtree extent.
2. **Admit the explicit braced container form at the surface** — a grammar row for top-level
   containers (a production + fold row per DESIGN.md §4 — rows, never a fold edit).
   Composition rule (no-ambiguity, operator 2026-07-06): a header, if present, is common-prefix
   sugar — the forest grafts under its position, explicit braces nest below it; the same
   position declared from two storage fragments is a typed refusal. The file-grain 1:1
   collision key becomes the tree-grain rule: one position, one authority. Multi-container
   files and finer-than-file fragments become expressible (the many-to-many model the
   module-identity lane stages — `ModuleStorageBinding` is the storage half; this is the naming
   half; they meet at the binding authority, never fuse). Prefer keyword-free bare containers
   (`a { … }`, matching `T{a}` notation) so nothing ever needs renaming — the keyword dies with
   the sugar header (§11).
3. **Dissolution — existing lanes, no new work.** Import deletion removes the lookup half;
   delta-first files-as-projections removes the storage half; the header becomes one projection
   policy's rendering convenience, then deletes. "Where does a module end" is moot: a
   namespace's extent is its subtree.

**Anti-goals:** never derive the position from the file path (hardens the fusion); never leave
both the metadata-edge mechanism and containment nodes live past step 1 (dual representation);
the desugar is exactly-one (header × explicit-brace composition defined; ambiguity refuses).

**Sequencing vs #6848 (this lane's call):** step 1 touches exactly the resolver/SymbolIndex
machinery the salvage stabilized — it lands as the FIRST follow-up PR on top of the merged
#6848, not inside it. Step 2 follows once step 1's dissolves (metadata edge, prefix keys) are
receipted.

**Step-1 landed vs receipted — post-#6968 capture (2026-07-22, from stern-newt-142's closeout
consult; this subsection is the durable authority for what was previously only in lane-plan
session messages).** #6968 merged the graft carrier + normalize/`validate_module_roots` ingress
+ a partial C5 dissolve (symbol_index_fill metadata branch and the name_resolve
export-admission header filter removed; graft-aware QN reader sweep), witnessed green
(zero-metadata graft output, graft shape, marker strip, normalize long-lane). **The step-1
receipt bar is NOT yet met**: the bar is both bolt-ons *dissolved into ordinary containment
fill*, not merely dead-in-corpus behind the graft. Open dissolves, in stern-newt's leverage
order:

1. **QN-reader collapse ("C6" second half)** — `qualified_name_from_module_node_graft_aware`
   is an interim dual-arm reader (`Scaffold → SingleAuthority`); dissolve = route grafted roots
   through the nesting-position spine walk inside `v2.extdeps.languages.dag`
   `qualified_name_from_module_node`, break the `std ↔ extdeps` import cycle (lift spine
   helpers to std OR split pre-graft ingress to a parse-only boundary), delete the wrapper +
   `namespace_graft_pre_graft_module_qn`. **Independent of the strip lane / pool-membership
   blocker** (confirmed: touches only namespace_graft + the extdeps QN fn; no PR-5b, no
   ci_layer_roots, no census machinery).
2. **"C6" first half — v1 seed regen** after the wrapper deletion. Previously scoped ONLY in
   the dashboard C1–C7 lane sequence (sunny-wolf green-light; "C6 regen + interim wrapper
   dissolution remain on plan under nimble-owl shepherding"); #6968 merged WITHOUT it. This
   paragraph is its capture.
3. **"C7" — completion receipts**: wet `source_root_ingest_gate_passes` + real-ingest RED
   controls green = the wrapper-collapse completion signal (named in the carrier trigger).
4. **Metadata-edge chain cleanup**: `dag_surface_module_header_metadata_edge` predicate + the
   `03_resolve` `under_module_root` preserve OR-chain
   (`dag_resolve_preserve_module_metadata_subtree`, `namespace_graft_parse_projection_edge`)
   still live as pre-graft/ModuleShell backstops; `body_lowering_fold` still calls the
   preserve fn at 3 sites (gated on body-lowering consuming grafted trees).
5. **SymbolIndex prefix-key dissolution**: `symbol_index_fill_module_root` still seeds the
   path from the `module_qn` string (`qualified_name_snoc` bolt-on);
   `symbol_index_containment_disposition` is `Scaffold` → `symbol_index_fill_containment_node`
   — the containment walk exists but pure nesting-position keys are not yet the authority.
6. **Review-41316 follow-ups (non-blocking)**: `try_admitted_export_binding` admission-entry
   construction hardening (refuse non-grafted admission); the
   `ends_with(.., "_node_projection")` suffix heuristic → declared grammar-projection rows.

**Post-C5 interim census protocol** (durable pointer): the 83-row exclude set is pinned
git-visible at `docs/probes/census_extra_excludes.txt` + `docs/probes/
still-hawk-row-coordination.txt` — ephemeral CLI application only, **never** baked into
`ci_layer_roots`; fierce-heron oracle = `derived(tip) == recovered-83`. Coordination note:
stern-owl-401's §13 containment-walk binding (#6979) was sequenced to coordinate after C5 on
the same files — clear to proceed since #6968 merged.

## 11. Terminology: "module" → "namespace" (operator-ruled, 2026-07-19)

A namespace is pure naming — a position that qualifies names. A module is classically a
namespace plus some subset of four extras, and here every extra is absent or deliberately homed
elsewhere: interface/export boundary — absent (keyword set has no pub/export/private;
reachability is lexical); dependency unit — being deleted (terminal step removes `import`);
compilation unit — already de-fused (`CompilationUnit` is its own concept; crate layout derived
by partition); storage/distribution unit — the fusion being dissolved (`ModuleStorageBinding`
models it honestly). So in gunbc "module" names the fusion itself — the thing this design
dissolves. Precedent: `ModulePath` → `QualifiedName` (§3 nickname, renamed);
`NamespaceOnlyY` / "effect grants over namespaces" are the live vocabulary.

**Ontology (three lines):**
- **containment** — the single relation; interior nodes include types, fns, params
  (`T{a}` → `a` is `T.a`; `.` is one projection op).
- **namespace** — a node that is only a container (no other semantics). What the file header
  actually declares.
- **module** — a namespace subtree fused to one storage file. Dissolving; survives, if
  anywhere, only as the storage-side binding's name.

**Priced consequences:** adopt "namespace" in design language and NEW carriers now; reserve
"module" strictly for the storage-side binding or retire it as carriers migrate. NO fleet-wide
keyword rename as its own change (2,258 headers for a dying word displaces no cost, §6). NO
alias period (two live spellings of one production is the §3 nickname the no-ambiguity rule
forbids) — if the sugar keyword is renamed at all, it rides the import-strip terminal wave,
which already touches every header region.

## 12. Runtime dispatch is another consumer of the containment tree — the fork is three-way, not two (zesty-deer-446, 2026-07-19)

**Finding, proven by execution: the import list is already inert. NEITHER typecheck NOR runtime
binds through it.** This is not "typecheck resolves against imports, runtime against registration
order" (the framing this lane inherited) — it is *two different arbitrary rules, neither of which
is the declared import*, plus a declaration nothing reads. §2.1's argument that imports are a
dual representation is therefore **empirically confirmed and stronger than stated**: they are not
a redundant-but-working authority to retire, they are a **dead** one.

### 12.1 The evidence

Instrumented probe (not inferred from diagnostics — see §12.2 for why that was necessary). Two
modules each declare `twin_sig`, distinguishable by arity; a third imports it **from `twin_p`**
and calls it. Reading what the frontend's own resolver returns:

```
planted module imports: test.claim.sigprobe.twin_p { twin_sig }     // 1 param
                        test.claim.sigprobe.twin_q { twin_q_anchor } // twin_q's twin_sig: 3 params

func_env.parents = [twin_q, v2.std.live_tree, std.types, std.algebra,
                    std.error_primitives, twin_p]
lookup_resolved_sig(planted.func_env, "twin_sig") -> 3 PARAMS      // == twin_q
```

`lookup_resolved_sig` (`v1_compiler_infer_sigs.rs:108-119`) checks `local`, then folds `parents`
taking the **first hit**. `parents` carries *which modules are imported*, never the selective name
list — `import M { f }` and `import M { g }` are indistinguishable in it — and its order is
incidental (note the two twins sit at opposite ends with std modules between them). So the
frontend binds `twin_q`, which the source never imported.

The three claimants:

| claimant | actual rule | honors the import list? |
|---|---|---|
| `import M { f }` | declared | **nothing reads it** |
| typecheck (`lookup_resolved_sig`) | first hit in `func_env.parents`, incidental order | no |
| runtime (`v1_interpreter fn_nodes`) | last insert into a bare-name map, file-path order | no |

They diverge because the *collections* differ, not because one is right. Consequence for the
incident that opened this lane (`import_closure` in `effect_reach.dag`): main was green on that
pair by coincidence at **two** layers, not one.

### 12.2 Why "no diagnostic" proved nothing (method note — do not repeat)

Three black-box probes were run first and all failed to discriminate, because the compiler does
not reject the mismatches they relied on: a call labelled with a parameter the bound signature
does not declare resolves clean, **and** a `test fn` declaring `-> Bool` whose callee returns
`Int` also resolves clean (caught only by the claim harness at runtime). With both label and
return-type checking inert for this shape, *absence of a diagnostic carries no information about
which declaration was bound.* Only direct observation of the resolver's return value settles it.
Two adjacent gaps this surfaced, both independent of naming and worth their own lanes:
`module_skips_direct_call_arg_check` (`v1_compiler_infer.rs:1233-41`) exempting all `v2.*` and
`v1.compiler.*` from compile-time direct-call argument checking — the exemption under which 33
silently-dropped-`diagnostics:` call sites accumulated (fixed #6896) — and the unchecked
declared-vs-actual return type above.

### 12.3 What "fold runtime dispatch into this lane" means, concretely

DESIGN.md's namespace open thread names three consumers of the one containment tree — resolution
walks it, content-addressing hashes it, termination reads its sub-value edges (this doc's §3
derives the tree itself, not that consumer list). **Dispatch is a further consumer, and was
unmodeled.** No ordinal is asserted here on purpose: effect grants independently claim "fourth"
(`DESIGN.md:100`, `docs/plans/effect-namespace-grants.md:22`), and an ordinal maintained in three
places is exactly the §3 second-encoding this lane exists to delete — consumers are *named*, and
the count is read off the list, never restated. Naming dispatch here is the whole consolidation: it converts component 1 of the bare-name dispatch fork
from *build a fourth resolver* into *delete a fork*.

- **`v1_interpreter fn_nodes` dissolves.** It is not fixed, re-keyed, or taught about imports — it
  is deleted. Dispatch reads the same `SymbolIndex` the resolver reads.
- **Ambiguity already has its typed refusal.** `GlobalBareLookup`'s `GlobalBareLookupAmbiguous` is
  exactly the §2.4 outcome ("a name resolving to ≥2 nodes is a typed, located error — never a
  flat-namespace pick"), and it is the direct answer to the dispatch fork's requirement that a
  bare name ambiguous in the loaded pool refuse rather than pick a winner. Nothing new is minted.
- **Cost shape is preserved.** Dispatch stays one map hit — on `QualifiedName` into
  `SymbolIndex.entries` rather than on a bare `String` into `fn_nodes`. It is NOT a per-call
  ancestor walk; the tree is materialized once by fill (§7.5's fill-once discipline), so §6's
  "bare minimum cost" is satisfied by construction rather than by measurement. *This is the
  claim most worth falsifying before implementation — it is the difference between a cheap
  substitution and a rewrite.* **FALSIFICATION RUN — the claim held (2026-07-22, §12.4
  delivery):** lexical_steps_histogram={1: 17175, 2: 1} over 37,521 containment hits — a
  substitution, not a rewrite.
- **Migration rides §8 unchanged.** When a subtree flips to `namespace-only-Y`, its runtime
  dispatch keys on the same index in the same motion. The two cannot drift because there is only
  one thing; a flip that moved resolution without moving dispatch would re-open the fork.

**Why it must NOT ship standalone.** Every standalone fix threads a *fourth* rule into a system
that already has three, and two of the three are wrong:
- threading `ResolvedImport.specific_names` (resolve-stage, `v1_compiler_resolve.rs:37-40`) into
  the interpreter makes **runtime** honor imports while typecheck still does not — converting
  shared arbitrariness into a genuine new divergence. Strictly worse.
- stamping the call site (the shape `MethodSemantics` already uses for algebra/service calls,
  where plain `CallSemantics` carries no target) is the right *shape*, but a stamp is only as good
  as the resolver producing it — so it would first require hardening `lookup_resolved_sig` to
  honor a mechanism §7 deletes. Paying twice.

**Interim, if the flip is not immediate.** The only defensible holding position is *visibility, not
a fix*: extend the existing binding-fork ledger (`cli_run.rs:5187-98`, which already counts
declaration-side multiplicity at typecheck, out-of-band, overlay-wins preserved) to also record
what runtime actually dispatched. Counted and typed, never an arm that widens (§5). The loud
subset is already closed — the application-site contract wall (#6896) turns the
mismatched-signature case into a typed `CallContractMismatch`; what remains silent is the
same-signature case, which only this lane's flip can catch.

### 12.4 NEXT STEP — the divergence census (operator-directed 2026-07-19)

Both resolution mechanisms now exist in-tree simultaneously — `lookup_resolved_sig`
(`v1_compiler_infer_sigs.rs:108-119`, first-hit over `func_env.parents`) and the containment walk
over the landed `SymbolIndex` (`symbol_index_lexical_lookup` / `symbol_index_global_unique_lookup`).
So the question *"which call sites do the two bind differently?"* is **computable today, read-only,
with no policy flip.** That set IS the blast radius of §8 step 4, and nobody has it.

This is deliberately NOT the §5.1 census. That one counted *declaration* collisions (23 fns, 26
types, 103 variants). This counts **resolution divergences** — the number that actually decides
migration risk, and which could be far smaller (most collisions are file-local helpers that never
co-occur in one closure) or far worse (one divergence on a hot path).

**Method — direct observation, NOT diagnostics.** §12.2 is a standing warning: the compiler
rejects neither label nor return-type mismatches for the relevant shapes, so "it compiled clean"
proves nothing about which declaration was bound. Read each mechanism's *return value* and compare
identity (node pointer, or qualified path). Three probes were wasted learning this.

**Deliverable — a counted, typed inventory,** whole corpus, both source roots. Per call site:
callee name, calling module, what `lookup_resolved_sig` binds, what the containment walk binds.
Bucket every site (§5 — a site must land in a named bucket, never be silently skipped):

- `Agree` — both bind the same declaration. The expected bulk; report the count, not the rows.
- `Diverge` — different declarations. **The deliverable.** Every row listed, with both bindings.
- `ContainmentAmbiguous` — the walk returns `GlobalBareLookupAmbiguous` where first-hit silently
  picked. These are the sites §2.4 turns into typed refusals; each needs qualifying before its
  subtree can flip. Expected to be the main source of migration work.
- `ContainmentUnresolved` — the walk finds nothing. Either a genuine gap in the index fill or a
  name that only first-hit reaches; a fill gap is a `SymbolIndex` bug and must be reported as one,
  not absorbed into the divergence count.

**Second output, nearly free: falsify §12.3's cost claim.** The census exercises exactly the
lookup dispatch would use, so record its cost shape. §12.3 asserts one map hit on `QualifiedName`,
not a per-call ancestor walk; if that is wrong, the fold is a rewrite rather than a substitution
and the lane's shape changes. This is the cheapest moment to find out.

**Do not** flip any policy, edit any resolver, or change dispatch as part of this. The census is an
artifact the rest of the lane consumes; §8 step 1 starts after it, informed by it.

**DELIVERED — and §8 step 1 has consumed it (2026-07-22).** The census shipped as
`resolution_divergence_census` (#6936 slice 1, #6967 slice 2); the fresh pre-step-1 run
(whole tree, `dag` + `src/v2`; raw rows in
`docs/probes/resolution_divergence_census_2026-07-22.tsv`): modules_resolved=1309,
sites_checked=40592 — **agree=33422 / diverge=0 / containment_ambiguous=38 /
containment_unresolved=0**, import_unresolved=4099 (walk-vs-infer cross-check
mismatch=0), neither_bound=3033 (builtin_or_intrinsic=1422, local_or_param=1607,
genuinely_unbound=4), agree-global-unique owner_mismatch=0. Cost shape (the §12.3
falsification): containment hits=37521, lexical_steps_histogram={1: 17175, 2: 1},
global_unique=20345 — the walk is O(chain depth) map hits with depth almost always 1,
so the fold is a **substitution, not a rewrite**; the claim held. One genuine §13
fail-open surfaced by the silent-pick join (both whole-tree and closure-scoped scopes,
pre-existing on main): `gunbc.falsifier_workflow` bare `ci_repo_root_shell` first-hits
`gunbc.ci_spec` over the identical duplicate decl in `gunbc.merge_admission_produce`
(fn_parent_first_hit=1) — a real §3 fork needing consolidation or qualification, and
the live specimen of the class §8 step 1's refusal arm makes loud under the flip.

Related: [type environment: single import authority + scope cursor](type-env-single-authority-design.md) — the type-env/SymbolIndex lane design this walk-rule migration rides on · [interface summaries and the declared↔use arity family](interface-summary-declared-use-arity.md) — the `std.interface_summary` carrier consumed by interface-grain resolve.

## 13. Resolution is unique-on-chain, not nearest (operator ruling, ratified 2026-07-21)

**Ratification.** The operator ruled the amended lookup semantics in-session (2026-07-21): *"fail
loudly"*; *"full path required at all times + users can alias"*; *"familiar frontend, very
strict/precise underneath"* — and on the shape below, *"this looks good."* Wording here is the
lane's; the semantics are ruled. This section **supersedes** the nearest-wins reading the executing
seed still carries (§7.5, §12) and tightens §3/§6 from "exactly one *nearest*" to "exactly one, full
stop."

**The rule.** Resolve a reference's **first segment** to the **unique binder on its ancestor chain**:

- **zero** binders → `Unresolved` — a loud, located refusal, never a fabricated bind;
- **two or more** → a located, typed `Ambiguous` carrying the **full candidate list** plus the fix
  menu (qualify by containment path / introduce an alias / rename);
- **exactly one** → project the remaining segments downward (§3's one projection op).

A bare name is the zero-extra-segments case of the *same* rule — not a special "global" tier; this
is the §7.5 seed invariant's own framing ("the shallowest level of the same walk") made terminal by
removing the nearest-wins fallback beneath it. **Amends §3/§6:** "finds exactly one nearest binding"
→ "finds exactly one binding on the chain; multiple = refusal." **No nearest-wins; shadowing is a
refusal, not a silent rebind** (Elm precedent). The refusal is raised at the **reference site**, not
the declaration site.

**The invariant (the why).** No edit anywhere in the program can silently **change what an existing
reference means** — it can only loudly break it. That is exactly the property nearest-wins forfeits:
adding a *nearer* homonym silently rebinds every reference below it. Fallback chains — nearest-wins,
global-unique-fallback, first-hit, any silent-pick — are rejected **as a class**; a uniqueness
constraint replaces a priority order (Rule 2 of §1, applied to the ancestor chain). This is §1/A3 at
the reference layer: a reference's meaning must stay stable under edits elsewhere.

**Aliases — binding-not-gate.** An alias is an **ordinary binding node**, not a visibility gate —
§6's "binding edge, not a scan" taken literally. `alias A = m.path.A` is a node at the declaring
position; references resolve *through* it by the same unique-on-chain rule. It is Rule-1-clean: the
target is encoded **once** (no dual representation), an unused alias is lintable dead code, and the
surface form is a grammar row (DESIGN.md §4, one grammar read both directions). **Import→alias
transmutation** is the migration mechanism — an `import` becomes an alias node at the importing
position, and the **source of truth is the walk's resolved target**, with the old import list
demoted to a cross-check. Grounded by the census buckets (#6936 first run: agree=38138 /
import_unresolved=1854 / neither_bound=739 / ambiguous=53; refreshed 2026-07-22 pre-step-1:
agree=33422 / import_unresolved=4099 / neither_bound=3033 / ambiguous=38, diverge=0 —
`docs/probes/resolution_divergence_census_2026-07-22.tsv`).

**`global_bare` dies as a mechanism.** The terminal state **deletes**
`symbol_index_global_unique_lookup` / `GlobalBareLookup` as *resolution* mechanisms — they survive
only as a migration oracle (computing the §12.4 divergence census), then are removed. This
supersedes §7's "the wall moves to *is it unique?*": the wall is *is it unique **on your chain?***
The executing seed's two interim gaps are named, not hidden: the fn path (`lookup_resolved_sig`,
`v1_compiler_infer_sigs.rs` — first-hit over `func_env.parents`, **no refusal arm at all**)
and the type/data LCP nearest-arm (`global_bare_lookup`, `v1_compiler_infer_env.rs` — refuses only
on an exact tie). The §5 interim backstop plus still-hawk-65's slice-2 per-class silent-pick counts
gate any widening (§12.4; do not widen blind — refusing where nothing refuses today reds latent
homonyms at scale, and fn is the larger class since it has *no* refusal today).

**§8 STEP 1 LANDED (2026-07-22) — both gaps now have policy-gated strict arms in the executing
seed.** A thread-local `NameResolutionPolicy` gate (`name_resolution_policy_is_namespace_only`,
`v1_rt`, host-setter only, **default OFF = ImportScoped byte-for-byte**) forks both paths:
the type/value path (`global_bare_lookup` / `global_bare_is_ambiguous`,
`src/v1/04_env.dag`) resolves a homonym to the **unique binder on the ancestor chain**
(exactly-one resolves; zero-or-multiple refuses — zero-on-chain whole-pool homonyms are
`Ambiguous`, census-walk parity, never a fabricated bind), and the fn path is rewritten to the
3-state `FuncSigLookup = FuncSigResolved | FuncSigUnresolved | FuncSigAmbiguous{candidates}`
(`src/v1/04_sigs.dag` / `04_lookup.dag`) so a refusal finally **has somewhere to go** — the
`Absent`-as-"keep-looking" straddle is dead as a class, and `FuncSigAmbiguous` never falls
through to the census fallback. Refusals surface as the typed, located
`AmbiguousReference { name, candidates, span }` diagnostic (`00_core.dag`) at the reference
site with the full candidate fix-menu. Analysis-only consumers (provenance/descent) project
through `func_sig_if_resolved` — conservative no-enrichment, never a fabricated bind; the
semantic bind sites (`04_infer.dag` ExprVar/ExprCall) match the full outcome. Discriminating
witness: `src/v1/tests/src/namespace_unique_on_chain_policy_test.rs` — the same homonym
fixtures compile clean under ImportScoped and refuse (typed, full candidate list) under
NamespaceOnlyY on both paths, with unique-on-chain-still-resolves and
unbound-stays-`UnresolvedType` controls. No subtree is flipped; the flip (step 4) still
gates on import→alias transmutation landing with-or-before it.

**Builtins bind at root.** The root namespace is on **every** chain, so builtins bound at root are
unique-on-chain everywhere — this **structurally dissolves** the prelude-shaped `neither_bound`
class (#6936, 739) rather than special-casing it, pending still-hawk's confirmation of the (c)
subclass.

**RULED — the (Y) expected-type filter (operator, 2026-07-22; recorded in §4).** The
re-adjudication this paragraph asked for is done: the pool-filter reading of (Y) dies with the
`global_bare` tier (there is never a pool candidate set to filter), and the surviving residue —
bare member names in typed positions — is **sugar gated on a syntactically-local container
annotation**, never type-directed elaboration from flow-typed positions. The sugar boundary
coincides with this section's edit-stability invariant, so nothing of the rejected fallback
class survives into the flip. See §4's re-adjudication note for the full statement and the §6
mechanics amendment for patterns-via-scrutinee's demotion.

**Sequencing.** §10 step-1 (header-as-sugar → containment graft) is in flight (stern-newt-142,
#6968); slice-2's per-class counts gate the §5 backstop widening; import→alias transmutation lands
**with or before** the strict flip, **never** imports-deleted-first (else references break before
their aliases exist — the ~20k `agree` sites are the empirical weight, §12.4); the flip is
per-subtree behind the §8 policy. This section is the authority the alias-grammar model PR and the
§3/§6 walk amendment consume.
