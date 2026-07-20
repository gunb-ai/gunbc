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
- **No expected-type picker** (§1c rule 5) — (Y)'s expected-type *filter* is distinct (§4).
- **Patterns via scrutinee** (§1c rule 4) is the pattern-position instance of (Y).
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
- **`resolve(name, position)` = the semantics over it** (nearest-enclosing-subtree search +
  (Y) expected-type filter + `Ambiguous`/`Unresolved`). Mine.
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
authority"), carrier `dag/std/symbol_index.dag`:
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

## 9. Open / to-verify

- **Root-prefix map** (`src/v1/**` ↔ `v1.*`, `dag/**` ↔ `v2.*`) and whether the `module`
  declaration is derivable from the path (if so, the declaration is itself a Rule-1 candidate
  to delete). Note `04_infer.dag:718` already special-cases the `"v2."` prefix — that logic
  folds into the root map.
- **Non-locality property (accepted, not a bug):** adding `v2.std.other.Symbol` later makes a
  previously-bare `Symbol` in-scope `Ambiguous` — a compile error until qualified. Loud and
  fail-closed (Rule 2), the honest cost of "no ambiguity." The alternative (narrower per-module
  scope) kills the sibling-access ergonomics the operator asked for; not taken.

## 10. Runtime dispatch is the FOURTH consumer — the fork is three-way, not two (zesty-deer-446, 2026-07-19)

**Finding, proven by execution: the import list is already inert. NEITHER typecheck NOR runtime
binds through it.** This is not "typecheck resolves against imports, runtime against registration
order" (the framing this lane inherited) — it is *two different arbitrary rules, neither of which
is the declared import*, plus a declaration nothing reads. §2.1's argument that imports are a
dual representation is therefore **empirically confirmed and stronger than stated**: they are not
a redundant-but-working authority to retire, they are a **dead** one.

### 10.1 The evidence

Instrumented probe (not inferred from diagnostics — see 10.2 for why that was necessary). Two
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

### 10.2 Why "no diagnostic" proved nothing (method note — do not repeat)

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

### 10.3 What "fold runtime dispatch into this lane" means, concretely

§3 names three consumers of the one containment tree (resolution walks it, content-addressing
hashes it, termination reads its sub-value edges). **Dispatch is a fourth, and was unmodeled.**
Naming it here is the whole consolidation: it converts component 1 of the bare-name dispatch fork
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
  substitution and a rewrite.*
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

### 10.4 NEXT STEP — the divergence census (operator-directed 2026-07-19)

Both resolution mechanisms now exist in-tree simultaneously — `lookup_resolved_sig`
(`v1_compiler_infer_sigs.rs:108-119`, first-hit over `func_env.parents`) and the containment walk
over the landed `SymbolIndex` (`symbol_index_lexical_lookup` / `symbol_index_global_unique_lookup`).
So the question *"which call sites do the two bind differently?"* is **computable today, read-only,
with no policy flip.** That set IS the blast radius of §8 step 4, and nobody has it.

This is deliberately NOT the §5.1 census. That one counted *declaration* collisions (23 fns, 26
types, 103 variants). This counts **resolution divergences** — the number that actually decides
migration risk, and which could be far smaller (most collisions are file-local helpers that never
co-occur in one closure) or far worse (one divergence on a hot path).

**Method — direct observation, NOT diagnostics.** §10.2 is a standing warning: the compiler
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

**Second output, nearly free: falsify §10.3's cost claim.** The census exercises exactly the
lookup dispatch would use, so record its cost shape. §10.3 asserts one map hit on `QualifiedName`,
not a per-call ancestor walk; if that is wrong, the fold is a rewrite rather than a substitution
and the lane's shape changes. This is the cheapest moment to find out.

**Do not** flip any policy, edit any resolver, or change dispatch as part of this. The census is an
artifact the rest of the lane consumes; §8 step 1 starts after it, informed by it.

Related: [type environment: single import authority + scope cursor](type-env-single-authority-design.md) — the type-env/SymbolIndex lane design this walk-rule migration rides on · [interface summaries and the declared↔use arity family](interface-summary-declared-use-arity.md) — the `std.interface_summary` carrier consumed by interface-grain resolve.
