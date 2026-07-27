# Namespace unique-on-chain — operational plan for lexical binders

**Status:** proposed implementation plan for review; no implementation authority.

**Authority:** [namespace-resolution-design.md §13](namespace-resolution-design.md#13-resolution-is-unique-on-chain-not-nearest-operator-ruling-ratified-2026-07-21).
That operator-ratified section supersedes the older nearest-wins wording in §§3/6:
resolution considers the full visible ancestor chain, `0` refuses unbound, `1` binds,
and `2+` refuses with the full candidate population. In particular, **shadowing is a
refusal, never a silent rebind**.

**Trigger:** PR #7328 attempted to preserve exact match-pattern binder identity for the
generic-match ownership prerequisite. Review 43794 found that ordinary `let` and lambda
scope construction retained a stale, name-keyed `match_bound_names` marker. The first
proposed repair removed the marker when a nearer binder arrived, thereby making
nearest-wins shadowing explicit. That repair contradicts §13 and is not ratified.

This plan operationalizes the existing namespace authority. It does not create a
match-shadowing subsystem, change `std.occurrence_binding`, or authorize edits to
load-bearing resolver/emitter files.

---

## 1. One question, one authority

There is one semantic question:

> Which declarations are valid candidates for this occurrence at this containment
> position?

The answer is derived from the containment tree and the occurrence's syntactic
reference category. It is not assembled independently by `locals`,
`match_bound_names`, function maps, global-bare maps, or import reachability.

For a reference whose first segment is `s`:

1. Classify the occurrence position using existing syntax:
   `ValueReference`, `TypeReference`, or the already-ruled typed
   pattern/member projection.
2. Walk the occurrence's complete chain of enclosing containment scopes.
3. Collect every declaration exposed by a scope on that chain whose declared
   segment is `s` and whose declaration category is admissible at that occurrence
   position.
4. Preserve exact declaration containment identity and deterministic source order.
5. Collapse duplicate discovery of the *same full containment identity* only.
6. Call the one canonical `occurrence_binding_from_candidates` fold exactly once.
7. Continue only through `OccurrenceBound`.

`OccurrenceUnbound` and `OccurrenceAmbiguous` are terminal, located refusals.
`OccurrenceAmbiguous` retains the complete ordered population. No nearest-level
selection, priority ordering, fallback lookup, name re-lookup, or expected-type
winner is permitted.

“Shadowing” therefore disappears as a semantic operation. It may survive only as
explanatory language for an ambiguity diagnostic. No helper may suppress, overwrite,
or bypass a declaration because another same-spelled declaration is nearer.

“Category” is a property of a declaration/reference position in the one namespace
tree, not a second namespace mechanism. A value occurrence does not admit a type
declaration merely because its text is equal. Within the value category, function
parameters, lambda parameters, `let` declarations, pattern binders, and callable
value declarations are not separate priority tiers.

Projection after the first segment is unchanged: once the first segment is uniquely
bound, remaining segments descend through the one `.`/containment projection.

---

## 2. What counts as visible

Visibility is structural:

- A declaration exposed by an enclosing scope on the occurrence's ancestor chain is
  visible. This includes, for example, a function parameter exposed by the function
  scope and a sibling function exposed by the enclosing module scope.
- A declaration exposed only by a sibling arm or sibling nested scope is not on that
  chain and is not a candidate.
- A declaration below the occurrence is not a candidate.
- A declaration made reachable only by an import list is not a namespace candidate;
  imports are transitional dependency/loading data, not naming authority.
- A builtin is a root declaration and is therefore on every applicable chain.
- An alias is an ordinary declaration node and participates by the same rule.

Structural visibility may exclude off-chain declarations. It may **not** select the
nearest declaration from two same-category declarations exposed by two scopes that
are both on-chain.
That distinction corrects the phrase “structural rung before cardinality” in the P1
charter: structural admissibility defines one population; cardinality is then folded
once over the full admitted population.

### Sibling-arm example — legal and unique

```dag
match input {
  Left  { value: y } => y
  Right { value: y } => y
}
```

Each `y` reference has only its own arm's declaration exposed along its enclosing
scope chain. The equal text does not create ambiguity because the declaration paths
are disjoint.

### Nested `let` example — ambiguous at the reference

```dag
match input {
  Present { value: y } => {
    let y = 1
    y
  }
  Absent => 0
}
```

The final `y` has both the pattern declaration and the `let` declaration exposed by
scopes on its ancestor chain. It must produce `OccurrenceAmbiguous` with both exact
containment paths. Removing either candidate changes the result to
`OccurrenceBound`.

### Nested lambda example — ambiguous at the reference

```dag
match input {
  Present { value: y } =>
    map(items, fn(y) { y })
  Absent => []
}
```

The lambda-body `y` sees both declarations and refuses. A lambda parameter must not
silently clear the pattern candidate.

### Reference-site versus declaration-site refusal

The ratified §13 rule places the refusal at the **reference site**, not at declaration
construction. Therefore an unused nested homonym is not rejected by this plan:

```dag
match input {
  Present { value: y } => {
    let y = 1
    0
  }
  Absent => 0
}
```

Changing this to declaration-time rejection would be a stricter language amendment,
not an implementation detail. It requires an explicit operator amendment to §13 and
is not smuggled into the identity prerequisite or canonical resolver successor.

---

## 3. Exact identity flow — fresh prerequisite boundary

#7328 is superseded because it made `SourceSpan` part of the semantic binding
carrier. Its fresh current-main successor establishes only the exact declaration and
reference identity facts:

```text
authored pattern binder
  → dedicated declaration Node
  → declaration ContainmentPath<Node>

authored reference
  → reference ContainmentPath<Node>
  → canonical resolver boundary
```

`SourceSpan` remains diagnostic evidence carried by the authored `Node`. It is never
the identity of a binder, binding candidate, ownership target, or deduplication key,
and it is never joined by raw name.

The exact current representation boundary is now known:

- `ExprLet` is already a declaration `Node`.
- Lambda parameters already have distinct `parsed_name_leaf` child `Node`s with
  authored spans.
- `MatchPattern.Bind` carries only `name: String`; direct and nested pattern binders
  therefore have no declaration `Node` yet.
- `infer_expr` carries no ancestor `ContainmentPath`, while
  `InferScope.locals: Map<String, TypeBinding>` overwrites equal-name declarations
  and `TypeBinding` carries no declaration identity.

Accordingly, the upstream identity prerequisite has two facts: materialize exact
pattern-binder declaration nodes, and preserve/thread existing declaration and
reference node containment through inference to the canonical occurrence-binding
boundary. A `SourceSpan + origin` record is not a substitute: it would create
another identity universe and restate syntax already carried by the declaration
node.

The fresh identity prerequisite must:

- materialize declaration `Node`s for every direct, alias, shorthand, positional,
  tuple, and nested pattern binder;
- preserve existing let, function-parameter, and lambda-parameter declaration
  `Node`s unchanged;
- thread full declaration and reference containment paths through inference;
- retain same-name active declarations as a population rather than overwriting;
- prove equal-text binders in sibling arms remain isolated;
- prove distinct authored occurrences never deduplicate merely because their nodes
  are structurally alike;
- prove restoring map overwrite or span identity makes the witness red;
- use authored `.dag` witnesses and normal regeneration only.

The fresh identity prerequisite must not:

- remove an outer candidate when a nearer `let` or lambda parameter appears;
- decide 0/1/many namespace cardinality in a second helper;
- make `match_bound_names` a binding authority;
- use `SourceSpan` for binding, ownership, lookup, or deduplication;
- add a shadowing-specific diagnostic or resolver;
- hand-edit or add handwritten stage0 Rust witnesses;
- implement resolution, generic-match ownership, emitter, or runtime behavior.

A shadowing witness that expects ambiguity belongs to the executing namespace
consumer, not to the identity-carrier prerequisite. The fresh prerequisite may
retain a structural fixture containing equal names only if it observes declaration
identity without claiming a winner for the ambiguous reference.

---

## 4. Fresh canonical resolver consuming edge

#7321 remains audit evidence and is not rebased or rescued. Its fresh current-main
successor is the first production v1 consumer of `std.occurrence_binding`. Its
candidate producer must:

- For each ordinary type, value, and callee occurrence, construct the exact
  `BindingOccurrence<Node>` containment path.
- Collect the complete structurally admissible declaration population before
  folding.
- For lexical/value occurrences, “complete” means the full applicable ancestor
  chain, not the nearest nonempty lexical rung.
- Exclude sibling/off-chain declarations structurally.
- Dedupe only identical full `ContainmentPath<Node>` identities.
- Preserve same terminal nodes at different containment paths as distinct
  candidates.
- Call `occurrence_binding_from_candidates` once.
- Project `TypeBinding`/function signature/value semantics only from the accepted
  declaration terminal in `OccurrenceBound`.
- Emit the existing located `AmbiguousReference` diagnostic from the typed
  `OccurrenceAmbiguous` population. Diagnostic strings are projections, never
  candidate identity.
- Never recover through `match_bound_names`, `locals`, function maps,
  `global_bare`, import reachability, nearest-level selection, or a second lookup.

Builtins remain declarations at the structurally recognized root. Grounded builtin
operations that are not declarations remain their already-ruled separate authority;
they may neither enter the declaration population nor fabricate a declaration.

The current #7321 branch remains no-edit audit evidence. The fresh resolver successor
starts only after the exact-identity prerequisite lands; match ownership is a
downstream consumer of that successor, not its prerequisite. P-derive proceeds on
its separately ruled model and source-binding dependencies.

### Source-read checkpoint — #7321 current head is not the terminal consumer

Audit of draft head `6c7027f` found that `OccurrenceBindingResult` is currently
downstream of the old priority resolver:

- `structurally_visible_declaration_paths` chooses local declarations first, then
  import-parent declarations, then the global-bare pool.
- Its strict branch widens an empty on-chain population back to the whole global
  population, while its legacy branch selects the nearest LCP rung.
- `lexical_param_declaration_paths` retains only the nearest matching parameter
  population.
- `ExprVar` folds those parameters first, then reads `scope.locals`, and only after
  both miss reaches `declaration_binding_result`.
- `declaration_call_lookup` bypasses the declaration fold when `body_locals` or the
  builtin registry contains the spelling.

That shape is:

```text
legacy priority selectors
  → reduced candidate population
  → OccurrenceBindingResult
```

It must be replaced, not merged as-is:

```text
exact occurrence path
  + complete category-admissible ancestor-chain population
  → OccurrenceBindingResult exactly once
  → Bound-only projections
```

In particular, an empty occurrence chain is `OccurrenceUnbound`; it never authorizes
a whole-corpus search.

---

## 5. Generic-match ownership boundary

The ownership lane consumes an accepted binding result; it does not resolve names.

The exact-declaration-identity prerequisite lands declaration `Node`s and
`ContainmentPath<Node>` transport only. It does not create an accepted binding edge,
and `SourceSpan` remains diagnostic evidence rather than binder identity. After the
fresh canonical resolver successor produces `OccurrenceBound`, the match-ownership
implementation may consume that result's exact candidate containment path. It must:

- consume only uniquely bound pattern uses;
- treat absent, ambiguous, or conflicting ownership proof rows as typed refusal;
- never infer a binder from raw text;
- never make nearest-scope selection;
- never re-resolve through `match_bound_names`;
- derive target `Borrow | Move | Refuse` through the ruled ownership proof plus
  target representation evidence.

This preserves the sequencing without creating a cycle: the fresh identity
prerequisite lands exact declaration/reference paths; the fresh resolver successor
executes the canonical candidate fold and attaches the accepted edge or typed
refusal; only then may generic-match ownership consume the accepted declaration and
fix the emitter's invented clone.

---

## 6. Discriminating witness matrix

Every witness must execute the production candidate producer and canonical fold.
Algebra-only `std.occurrence_binding` claims are necessary but insufficient.

| Case | Candidate population | Required result |
| --- | --- | --- |
| one pattern binder, one use | `[pattern.y]` | Bound to exact pattern path |
| same text in sibling arms | each arm sees only its own `[arm.y]` | Both Bound, distinct paths |
| nested distinct names | `[outer.y]` or `[inner.z]` | Bound |
| pattern `y`, nested `let y`, referenced below | `[pattern.y, let.y]` | Ambiguous, ordered full population |
| pattern `y`, lambda parameter `y`, referenced in lambda | `[pattern.y, lambda.y]` | Ambiguous, ordered full population |
| same-scope duplicate value declarations, if grammar admits them | both exact declarations | Ambiguous at every affected reference |
| exact duplicate discovery path | one identity after producer dedupe | Bound |
| same terminal under different containment paths | two identities | Ambiguous |
| value `x` and type `x` where syntax admits both | category-admissible population only | No cross-category pick/collision |
| local value and callable value with same segment on-chain | both value declarations | Ambiguous |
| unused nested homonym | no occurrence to fold | No reference diagnostic under current §13 |
| unbound name | `[]` | Located Unbound |

For every 0/1/many case, perturbing the population must change the verdict. RED
controls remove the full-chain collection, restore nearest-wins, restore a
name-keyed lookup, or drop one ambiguous candidate; each must fail.

The corpus census before the strict consumer flips must count, by occurrence
category:

- unique full-chain bindings;
- unbound occurrences;
- two-candidate full-chain ambiguities;
- three-or-more full-chain ambiguities;
- candidate pairs split by declaration kind;
- sibling/off-chain equal-text declarations excluded structurally;
- exact duplicate discovery paths collapsed by identity.

The census is a migration worklist, never permission for a fallback. Repairs are
qualification, an ordinary alias, or rename; never import additions or priority
rules.

---

## 7. Codebase-wide dual-authority census

The shadowing defect is one instance of a broader migration problem: the compiler
still contains several structures that can independently answer “what does this
name mean?” A map or cache is not automatically a second authority. It becomes one
when a consumer can use it to choose a declaration without consuming the accepted
`OccurrenceBindingResult`.

This section is a bankruptcy charter, not another resolver plan beside the old
namespace. Every existing mechanism receives exactly one terminal disposition:
candidate enumerator, Bound-only projection, temporary loader with a dissolution
trigger, or deletion.

Every naming-related surface is classified into one of four dispositions:

1. **Authority** — the containment graph supplies exact declaration candidates and
   `OccurrenceBindingResult` supplies the one cardinality decision.
2. **Projection** — data computed only after `OccurrenceBound`, unable to choose or
   recover a declaration.
3. **Transitional scaffold** — import/dependency machinery still needed to load the
   corpus, with an explicit dissolve-on.
4. **Competing decision mechanism** — a second lookup, fallback, priority tier, or
   name-keyed identity that must dissolve.

The initial authored-source census follows. Generated stage0 Rust is an execution
projection of these `.dag` sources and is not a separate row or repair surface.

### 7.1 Binding-decision mechanisms that must converge

| Surface | Current independent decision | Terminal disposition / dissolve-on |
| --- | --- | --- |
| `src/v1/04_env.dag` — `TypeEnv.bindings`, `str_bindings`, `ancestry_str_bindings`, and `parents` | Ordered map/parent lookup can select a type declaration before the containment population is known | Retain only representation/index data needed to enumerate exact declaration paths. Delete their declaration-choice role when the fresh resolver successor's type occurrences consume the one full-chain fold. |
| `src/v1/04_env.dag` — `SymbolIndex.global_bare`, `GlobalBareLookupState`, LCP/nearest helpers, strict-policy branch | A bare-name pool plus policy chooses or refuses independently of lexical candidates | Delete `global_bare` as a resolution mechanism when all ordinary v1 type/value/callee occurrences enumerate from containment. No nearest or global-unique compatibility arm remains. |
| `src/v1/04_env.dag` — `Scope`, `ScopeBinding`, `scope_lookup`, `scope_push` | A second lexical abstraction exists without an executing consumer | Do not expand it into another resolver. Delete it if the consumer census confirms it remains hollow; otherwise convert its sole use to candidate enumeration and give it the fresh resolver successor's dissolve-on. |
| `src/v1/04_sigs.dag` and `04_lookup.dag` — `ResolvedFuncEnv`, `FuncSigLookup`, global-bare callable fallback | Function/value lookup has its own local/parent/global 0/1/many and fallback behavior | Function signatures become post-`OccurrenceBound` projections from the accepted declaration path. Delete the independent lookup result and fallback after all call/value consumers cross that seam. |
| `src/v1/04_infer.dag` — `InferScope.locals`, `body_locals`, `match_bound_names`, `call_locals_shadow_note` | Separate name maps and an explicit locals-before-functions-before-global priority can select a value | Preserve scope facts only to enumerate exact on-chain declarations. Delete raw-name winner selection when the fresh resolver successor's value/callee witnesses execute the full-chain fold. |
| `src/v1/04_infer.dag` — `spine_root_is_shadowed`, `body_shadow_aware_func_sig`, `lookup_variant_parent_enum`, `infer_var_binding_kind`, and every `shadow*` helper | A raw-name presence check suppresses a qualified projection, converts a function to unresolved, or recovers declaration kind/owner after lookup | Delete the decision helpers. Bind a qualified spine's first segment once; derive parameter/local/function/variant/namespace kind from the accepted declaration path. “Shadowed” may remain only in diagnostic prose for `OccurrenceAmbiguous`. |
| `src/v1/04_method.dag` — `builtin_function_registry` and builtin preemption in call lookup | One bare-name registry mixes declaration-like builtins, grounded operations, host/compiler hooks, and legacy policy instrumentation; a match can bypass declaration binding | Root declaration-like builtins in containment and include them in the ordinary candidate fold. Give genuine non-declaration operations a separate typed operation authority that cannot fabricate, suppress, or displace a declaration. Delete policy instrumentation entries with the policy gate. |
| `src/v1/04_infer.dag` — variant-local/census helpers and `VariantCollision` selection | Constructor ownership and collision handling can be inferred through a variant-name side population | Enumerate constructor declaration paths in the applicable syntactic category. Keep collision text only as a diagnostic projection of an ambiguous typed result. |
| `src/v1/04_infer.dag` and `04_service.dag` — `ItemInfo`/service registries, `collect_called_func_names`, `expand_transitive_services`, receiver-string operation lookup | Calls and service effects are propagated through caller/callee short names; service operations take the first textual match | Build accepted caller-declaration → accepted callee-declaration edges. Key summaries by declaration identity; derive service identity from the accepted service declaration and resolve operations only within it with explicit cardinality. |
| `src/v1/04_resolve.dag` — `parameterized_use_site_prefers_parameterized_decl` | Type arguments trigger a second direct-import retry after the primary lookup chose a declaration | Delete when generic type occurrences bind once by containment and project arity only after `OccurrenceBound`. |
| `src/v1/04_resolve.dag` — `source_visible_names`, `masked`, `UnlistedImportUse` family | An import-derived side map independently filters whether an otherwise resolved name is admitted | Delete when namespace binding is wholly structural and accepted binding edges drive dependency loading. This is already required by the parent namespace design §7. |
| `src/v1/04_env.dag`, `04_lookup.dag`, and emit helpers — owner/qualification recovery by short name | Global-bare owner lookup, borrowed-name lookup, variant-owner search, alias-target lookup, or registry provider lookup rediscover an owner after a plausible bind | Owner/provenance/qualification/display are projections of the accepted full containment path. Delete every short-name owner recovery. A shortest display spelling may be computed only from that accepted path. |
| `src/v1/05_emit.dag` and target emitters — reconstructed `InferScope`, `lookup_item`, `lookup_func_sig_in_scope`, textual argument ordering/provider selection | Emission replays scope and call lookup, so it can choose a different declaration from typechecking | Typed call/value nodes carry or reference accepted declaration identity, callable signature, parameter/argument association, and accepted variant/service ownership. Every emitter consumes that edge and may cache rendering only by identity. |
| collision-wall designs and `VariantCollision`/type-name collision mechanisms | Global or import-closure short-name equality is treated as a declaration conflict even when declarations are off-chain siblings | Split the concerns: duplicate full containment identity is a construction error; 2+ same-category candidates on one occurrence chain are `OccurrenceAmbiguous`; two declarations intentionally modeling one concept belong to the consolidation lens; unrelated equal leaves are legal; duplicate discovery of one path dedupes by full identity. |
| `src/v1` runtime policy — `NameResolutionPolicy` and silent-pick census recording | A compatibility gate permits both namespace-only and legacy nearest/import-scoped answers | Delete the policy fork and legacy instrumentation when the strict consumer and corpus repair are green; the terminal language has one policy. |
| v1 policy/census/precedence witnesses | Tests preserve off-chain global ambiguity, local/kernel/direct-import/last-import winners, global-unique binding, or “global bare is the naming authority” | Rewrite as on-chain ambiguity, structural exclusion, loading-only, or Bound-projection witnesses. In particular, the current zero-on-chain homonym fixture becomes Unbound, not Ambiguous. Keep corpus-global homonym counts only as non-authoritative migration inventory. |
| `src/v2/std/symbol_index.dag` — `LexicalLookup` | A separate bound/unbound/ambiguous result is produced without exact reference-occurrence identity | Keep `SymbolIndex.entries` as materialized containment storage. Once the v2 resolver carries exact occurrences, project candidates from the index and fold through `OccurrenceBindingResult`; then dissolve `LexicalLookup` as a parallel result carrier. Do not fabricate an occurrence during the interim. |
| `src/v2/std/symbol_index.dag` and `src/v2/compiler/03_resolve.dag` — `global_bare`, `GlobalBareLookup`, `SymbolIndexAtomLookup` fallback | Lexical unbound can widen into global-unique resolution | Delete the global-unique fallback when the v2 exact-occurrence consumer lands. An empty on-chain population is unbound, not permission to search a second universe. |
| `src/v2/compiler/03_resolve.dag` — `ScopeFrame.locals`/`lookup_chain`, symbol-index lookup, and canonical-symbol fallback | Nearest-frame lookup gets priority over an independent lexical result, which may widen global, followed by another canonical-symbol acceptance path | One exact-occurrence candidate producer replaces all three decision routes. Exact symbol-index storage may remain; frame data may enumerate candidates but cannot select the nearest frame. |
| `src/v2/compiler/03_name_resolve.dag` — `Namespace.bindings`, `canonical_symbols`, import admission | These maps currently mix canonical storage, import admission, and possible declaration choice | Audit each consumer. Canonical-symbol storage may remain, but no map read may select a declaration independently; import admission dissolves with Dispatch 2. |

This inventory is intentionally by semantic entry point rather than by type name.
Renaming a lookup or moving it into a registry does not retire an authority.

### 7.2 Dependency/loading representations with a separate dissolve-on

These rows do not decide declaration identity in the intended model, but they still
duplicate the evidence used to decide which modules and Rust `use` lines are needed:

| Surface | Transitional role | Dissolve-on |
| --- | --- | --- |
| `ResolvedModule.resolved_imports` and import overlays | Load/admit modules from authored import declarations | Reference-derived loading reaches closure without imports and the Dispatch 2 refusal matrix is green. |
| `src/v2/lens/module_graph.dag` — `ImportResolutionFact` | Import-derived module dependency evidence | Every required module edge is projected from accepted reference binding or another explicitly typed non-reference dependency. |
| `src/v2/lens/reference_deps.dag` — `ReferenceResolutionFact` | Qualified-name approximation of reference dependencies | Exact accepted binding paths directly project the reference dependency edge; parsed-name approximation is deleted. |
| `src/v2/lens/module_graph.dag` — union of import and reference facts | Compatibility closure over two evidence sources | Reference-only closure is complete, deterministic, and witnessed; the import half and union disappear together. |
| `src/v1/05_emit_rust.dag` — reference-derived/import-derived `use` synthesis and registry provider lookup | Keeps emitted Rust compiling while authored imports and approximate reference facts coexist | Emission consumes the accepted declaration/module edge and cannot choose a provider by bare registry name. Import extraction and compatibility synthesis then delete. |
| host loader/runtime registries, including any bare `fn_nodes` or item-provider map | Load or dispatch by a name-derived provider | Dispatch is keyed by the accepted declaration identity from the same SymbolIndex/containment authority. A metadata cache may remain only if it cannot choose among declarations. |

This separation matters: deleting import syntax before reference-derived loading is
complete would break execution, while retaining import facts as naming evidence
would preserve the dual authority that Dispatch 2 is meant to remove.

### 7.3 Legitimate projections that must not become resolvers

The following data may remain, subject to a construction wall:

- `TypeBinding`, `DeclaredFuncSig`, and `ResolvedFuncSig` are semantic payloads
  derived directly from the accepted declaration terminal/path.
- `AmbiguousReference` strings and other diagnostic labels are exhaustive ordered
  renderings of typed candidates, never identity or input to another lookup.
- `DependencyView.BindsTo` is a downstream projection:
  `source = candidate.containment.terminal` and
  `dependent/usage_site = occurrence.containment.terminal`.
- intern tables, emitter registries, method tables, and caches may accelerate
  representation lookup only when keyed from an accepted declaration identity and
  incapable of choosing a declaration.
- genuinely grounded builtin operations may retain their separate typed authority;
  a same-spelled user declaration cannot enter or be displaced by that operation
  rung. Declaration-like builtins instead live at the namespace root.
- field and method tables may select a member only after the receiver's first segment
  is uniquely bound; they may not repair an unbound or ambiguous receiver.

If any projection performs a raw-name fallback, priority pick, provider guess, or
candidate dedupe weaker than full containment-path equality, it is reclassified as
a competing decision mechanism.

The following are yellow-zone data pending caller classification, not mechanical
deletion targets: `recursive_type_set`, `inductive_fields`,
`lambda_param_provenance`, field/method summary maps, and serialization/display
qualification caches. A raw-name key is a defect only when equal-spelled declarations
can make one row affect the other's semantics.

### 7.4 Census and cleanup gate

Before the namespace migration is declared complete, an executing source-graph
census must classify every function that:

- reads a name-keyed declaration map;
- walks lexical parents or module ancestors;
- chooses a type, value, function, constructor, alias, or provider;
- falls back after a lookup miss;
- converts import or registry data into visibility;
- emits an unbound/ambiguous/collision diagnostic.

For every such function, the census records its authored `.dag` location, occurrence
category, input identity, output type, callers, one of
`Authority | Projection | Transitional | Competing`, and a named dissolve-on. The
gate fails on an unclassified entry point. It also fails if two classified entries
can both decide the same occurrence, even when their current corpus answers happen
to agree.

Seed discovery vocabulary includes `shadow`, `nearest`, `first_hit`, `global_bare`,
`overlay`, `*_by_name`, `lookup_*`, `fallback`, `provider`, `variant_owner`,
`source_visible`, `Ambiguous`, `Unresolved`, and `Collision`. This vocabulary finds
candidates for classification; it is not itself the semantic gate.

The terminal call graph has a simple shape:

```text
containment/index storage
  → complete category-admissible candidate population
  → occurrence_binding_from_candidates (exactly once)
  → Bound-only semantic projections
  → diagnostics / dependency edges / emission / runtime dispatch
```

No cleanup row is satisfied merely by deleting a type. Its executing callers must
either consume the accepted edge or disappear, and a RED witness must prove that
restoring the old fallback or priority changes the verdict.

The cross-stage identity receipt must prove:

```text
typechecker accepted declaration identity
  == emitter target identity
  == runtime dispatch identity
```

---

## 8. Sequencing and ownership

1. **This docs-only plan** is reviewed against §13 and the operator's no-parallel-
   subsystem requirement, including the dual-authority census and dispositions.
2. **Pattern-node and containment identity prerequisite** gives every binder an
   exact declaration node and threads declaration/reference containment through
   inference without a name-keyed overwrite or parallel identity carrier.
3. **#7328 is superseded.** A fresh current-main identity prerequisite implements
   only the ruled declaration-node/path fact, `.dag` witnesses, and
   population-preserving lexical index; it carries no span identity, resolver,
   ownership, emitter, or runtime behavior.
4. **Fresh canonical resolver successor** implements the single full-chain
   occurrence-binding consumer, including the shadowing ambiguity matrix, and
   attaches `OccurrenceBound` or a typed refusal. #7321 remains audit evidence and
   is not rebased or rescued.
5. **Generic-match ownership prerequisite** consumes the exact declaration from
   `OccurrenceBound` and removes emitter-invented clone behavior.
6. **P-derive model authority** lands independently; its emitter consumer waits for
   both the corrected static contract and exact source declarations accepted by the
   canonical binding edge.
7. **Executing dual-authority census** classifies every naming decision entry point
   and turns the table above into checked cleanup rows with named consumers.
8. **Bound-edge consumer migration** rewires, in order, generic/type projection,
   function signatures, constructor ownership, emitter call/argument association,
   call/service effects, and runtime dispatch before their name-keyed structures
   delete.
9. **Corpus census and generated repair** eliminate unbound/ambiguous residues by
   qualification/alias/rename, with no import additions.
10. **Reference-derived dependency/loading convergence** consumes accepted binding
   edges and deletes the transitional import/reference union.
11. **Compatibility bankruptcy** deletes `NameResolutionPolicy`, silent-pick
   telemetry, global-bare resolution, import visibility, generic retry, shadow
   helpers, collision-as-resolution walls, and obsolete precedence tests only after
   their executing callers consume accepted edges or disappear.
12. **Dispatch 2** deletes import lines and grammar only after the reference-only
   authority, authority census, and full refusal matrix are green.

No step may use a default-off escape, nearest-wins compatibility arm, raw-name
identity, hand-generated Rust, or a second resolution carrier.

---

## 9. Done lines

This plan is implemented when all of the following are true:

- Exact pattern declaration identity survives parse → infer without raw-name joins.
- The production v1 resolver folds the full applicable ancestor-chain population
  exactly once through `OccurrenceBindingResult<Node>`.
- A nearer homonym cannot silently change an existing reference's meaning.
- Sibling-arm equal names remain independently and uniquely bound.
- Ambiguity diagnostics carry all exact candidates in deterministic order.
- Type/value/callee consumers continue only from `OccurrenceBound` and never
  re-resolve.
- Every naming-decision entry point is classified, and every competing mechanism
  has dissolved at its executing callers rather than merely being renamed.
- Dependency, diagnostic, emitter, and runtime projections are constructed from the
  accepted declaration edge and cannot choose or recover a declaration.
- `global_bare`, import visibility, generic retry, local/function priority, and v2
  lexical/global fallback no longer exist as resolution mechanisms.
- No `shadow*`, collision wall, builtin registry, emitter, service/call registry, or
  owner-recovery helper can suppress, select, widen, or re-resolve a declaration.
- Typecheck, emission, effect propagation, and runtime dispatch agree on the exact
  accepted declaration identity by execution.
- The generic-match ownership consumer reads the accepted binding edge and has no
  name or clone heuristic fallback.
- The ordinary-compile matrix, v1 build, regen fixed point, diagnostic accounting,
  and full CI are green.

Only after those walls and the remaining reference-derived dependency gates are
green does import deletion become the mechanical Dispatch 2 operation.
