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
- **Genuine cross-tree forks (identical variant-set → same concept): only 2** — `ImportTrigger`
  (`std.languages` vs `v1.compiler.languages`, same 5 variants — a seed fork) and `Reconciliation`
  = `Converged|NotConverged` (`product.budget_tree` vs `std.realization_reconcile` — a real §3
  fork to consolidate). That is the entire "consolidate now" worklist the census forces.
- **Same-subtree variant collisions** — the `github` conclusion families (`Success`/`Failure`/
  `Cancelled` across `extdeps.github.{actions,checks,workflow_runs}`, `CheckConclusion` itself
  actions-vs-checks with *different* variant sets) — are genuinely co-visible in github code and
  are the **(Y) context-resolution** population: bare at typed sites, qualify only in untyped ones.

**Net:** under (Y) + nearest-wins the forced-qualification residue is a **handful** — the
same-subtree homonyms with no typed context — because (a) cross-tree seed forks resolve by
subtree, (b) github-style collisions resolve by expected type, (c) file-local helpers never
collide. The census's standing value is the two-item fork worklist plus a clean quantification of
the v1→v2 seed-duplication surface.

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

**Sequencing consequence:** `SymbolIndex` lands first (it is the substrate *and* the O(M²) fix,
gated on loyal-heron's scaling receipt per that doc's §7.5). Namespace-only is then a policy
layer over the settled index — not a from-scratch resolver. Resolution is a **walk over the
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

## 9. Open / to-verify

- **Root-prefix map** (`src/v1/**` ↔ `v1.*`, `dag/**` ↔ `v2.*`) and whether the `module`
  declaration is derivable from the path (if so, the declaration is itself a Rule-1 candidate
  to delete). Note `04_infer.dag:718` already special-cases the `"v2."` prefix — that logic
  folds into the root map.
- **Non-locality property (accepted, not a bug):** adding `v2.std.other.Symbol` later makes a
  previously-bare `Symbol` in-scope `Ambiguous` — a compile error until qualified. Loud and
  fail-closed (Rule 2), the honest cost of "no ambiguity." The alternative (narrower per-module
  scope) kills the sibling-access ergonomics the operator asked for; not taken.
