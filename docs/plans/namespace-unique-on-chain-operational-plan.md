# Namespace unique-on-chain — lexical binder plan

**Goal:** one declaration decision per occurrence, as ruled by [namespace-resolution-design §13](namespace-resolution-design.md#13-resolution-is-unique-on-chain-not-nearest-operator-ruling-ratified-2026-07-21). Proposed plan; no implementation authority. The [original audit](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/namespace-unique-on-chain-operational-plan.md) records #7328's stale `match_bound_names` defect and #7321 head `6c7027f`'s priority selectors. Both branches remain superseded audit evidence; start successors from current main.

**Method:** exact occurrence/declaration containment → complete admissible ancestor-chain population → `occurrence_binding_from_candidates` once → Bound-only consumers. **Validate:** production witnesses, an exhaustive decision-entry-point census, and agreement of typecheck, emission, effect propagation and runtime identity. This plan neither changes `std.occurrence_binding` nor authorizes load-bearing resolver/emitter edits.

## Candidate and identity contract

Classify each occurrence as `ValueReference`, `TypeReference`, or the ruled typed pattern/member projection. Collect matching first-segment declarations exposed by every enclosing scope, preserving exact `ContainmentPath<Node>` identity and deterministic source order. Dedupe only identical full paths. `0` gives located `OccurrenceUnbound`; `1` gives `OccurrenceBound`; `2+` gives located `OccurrenceAmbiguous` with the complete ordered population. Only Bound continues; remaining segments use ordinary `.` containment projection.

Sibling/off-chain and descendant declarations are excluded. Imports supply transitional loading data, never naming authority. Aliases are ordinary declarations; declaration-like builtins live at the root. Grounded non-declaration operations retain their separately ruled typed authority without fabricating, suppressing or displacing declarations. Categories follow syntax and declarations, not parallel namespaces: parameters, lets, pattern binders and callable values are one value population, with no priority tiers.

No nearest-level selection, expected-type winner, global widening, fallback, name re-lookup or candidate suppression. Nested `let y` or lambda `y` beneath a pattern `y` makes a referenced `y` ambiguous; sibling-arm `y`s remain isolated. An unused nested homonym has no reference refusal. Declaration-time rejection would require an operator amendment to §13.

The identity prerequisite materializes declaration `Node`s for direct, alias, shorthand, positional, tuple and nested pattern binders; preserves existing let/function/lambda declaration nodes; threads declaration and reference containment through inference; and retains same-name active populations. `SourceSpan` remains diagnostic evidence, never binding, ownership or dedupe identity. Structurally alike authored nodes at distinct paths remain distinct.

Its witnesses prove sibling isolation, distinct occurrences and RED restoration of map overwrite or span identity. Use authored `.dag` witnesses and regeneration. It adds no cardinality helper, shadow diagnostic, resolver, ownership, emitter/runtime behavior or hand-written stage0 Rust. An ambiguity-result witness belongs to the executing resolver; the identity prerequisite may only observe identities.

## Owners and order

Roadmap node boundaries govern if this description diverges:

| Owner | Obligation |
| --- | --- |
| `namespace-reference-derived-closure` | Exact `BindingOccurrence<Node>` and complete type/value/callee candidate population; structural exclusion and full-path dedupe. |
| `namespace-binding-kernel` | Fold the supplied population exactly once. |
| `namespace-bound-only-consumers` | Project type, signature and value semantics only from the accepted declaration terminal. |
| `namespace-ambiguity-discharge` | Render existing located `AmbiguousReference` from the exhaustive typed population; strings never identify candidates. |
| `namespace-canonical-binding` | Delete old decision routes after their consumers migrate. |

Land exact identity first, then the fresh canonical resolver, then generic-match ownership. Ownership consumes only the accepted pattern declaration; absent, ambiguous or conflicting proof rows refuse. Derive `Borrow | Move | Refuse` from the ruled proof and target representation, never names, nearest scope, `match_bound_names` or invented clone behavior. P-derive's model proceeds independently; its emitter waits for both corrected static contract and accepted exact source declarations.

Next execute the authority census; migrate generic/type projections, signatures, constructors, emitter call/argument associations, service effects and runtime dispatch; repair corpus occurrences by qualification, ordinary alias or rename; converge reference-derived loading; delete competing mechanisms. Dispatch 2 removes import lines/grammar only after reference-only closure, census and refusal gates pass. No default-off escape, compatibility nearest-wins arm, raw-name identity or second result carrier.

## Decision-entry-point dispositions

A map is competing authority only if it can choose a declaration without the accepted result. These source symbols are audit seeds, not a current implementation receipt. Every executing caller must migrate or disappear; moving or renaming a type proves nothing.

| Surface | Required disposition |
| --- | --- |
| v1 `TypeEnv.bindings`, `str_bindings`, `ancestry_str_bindings`, `parents` | Exact-path enumeration/index storage only; delete declaration choice once type occurrences consume the fold. |
| v1 `SymbolIndex.global_bare`, `GlobalBareLookupState`, LCP/strict-policy helpers | Delete bare/global/nearest resolution after all ordinary occurrences use containment. |
| v1 `Scope`, `ScopeBinding`, `scope_lookup`, `scope_push` | Delete if caller census confirms hollow; otherwise enumerate candidates with the resolver successor's dissolve-on. |
| v1 `ResolvedFuncEnv`, `FuncSigLookup`, callable fallback | Bound-only signature projections; delete independent results/fallbacks. |
| v1 `InferScope.locals`, `body_locals`, `match_bound_names`, `call_locals_shadow_note` | Preserve exact candidate facts; delete locals/function/global winner selection. |
| `spine_root_is_shadowed`, `body_shadow_aware_func_sig`, `lookup_variant_parent_enum`, `infer_var_binding_kind`, every `shadow*` helper | Delete suppression/recovery decisions; derive declaration kind from the accepted first-segment path. “Shadowed” may describe ambiguity only. |
| `builtin_function_registry` and call preemption | Separate root declarations from typed grounded operations; delete policy instrumentation with the policy gate. |
| Variant-local/census helpers, `VariantCollision` | Enumerate constructor paths in the syntax category; collision text only renders typed ambiguity. |
| `ItemInfo`/service registries, `collect_called_func_names`, `expand_transitive_services`, receiver-string operation lookup | Accepted caller→callee identity edges and identity-keyed summaries; service operations resolve inside the accepted service with explicit cardinality. |
| `parameterized_use_site_prefers_parameterized_decl` | Delete import retry; project arity only after Bound. |
| `source_visible_names`, `masked`, `UnlistedImportUse` | Delete import-derived visibility when structural binding drives loading. |
| Short-name owner/variant/alias/provider/borrowed-name recovery | Delete; derive owner, provenance, qualification and shortest display spelling from the accepted full path. |
| Emitters' reconstructed `InferScope`, `lookup_item`, `lookup_func_sig_in_scope`, textual argument/provider selection | Carry accepted identity, callable signature, parameter/argument association and variant/service ownership in typed nodes; cache rendering by identity only. |
| Collision walls | Distinguish duplicate full identity (construction error), many on-chain candidates (ambiguity), one concept modeled twice (consolidation lens), unrelated equal leaves (legal), repeated discovery (full-path dedupe). |
| `NameResolutionPolicy`, silent-pick telemetry, precedence witnesses | Delete policy fork/instrumentation after strict consumers and corpus repairs pass. Replace import/global/nearest winner tests with structural/Bound tests; zero on-chain is Unbound, not global Ambiguous. Global homonym counts remain non-authoritative inventory only. |
| v2 `LexicalLookup` | Retain index storage; dissolve parallel result once exact occurrences feed the canonical fold. Never fabricate an interim occurrence. |
| v2 `global_bare`, `GlobalBareLookup`, `SymbolIndexAtomLookup` | Delete global-unique fallback with exact-occurrence consumer. |
| v2 `ScopeFrame.locals`/`lookup_chain`, index lookup and canonical-symbol fallback | One candidate producer replaces all decision routes; storage/frame enumeration may remain. |
| v2 `Namespace.bindings`, `canonical_symbols`, import admission | Classify every caller; keep storage only, delete import admission with Dispatch 2. |

Loading has a distinct dissolution boundary:

| Surface | Dissolve when |
| --- | --- |
| `ResolvedModule.resolved_imports`/overlays | Reference loading closes without imports and Dispatch 2 refusal matrix passes. |
| `ImportResolutionFact` | Every dependency edge comes from accepted binding or an explicitly typed non-reference dependency. |
| `ReferenceResolutionFact` | Exact accepted paths replace parsed-name approximations. |
| Import/reference union | Reference-only closure is complete, deterministic and witnessed; remove both compatibility pieces together. |
| Rust `use` synthesis/provider registry | Emission consumes accepted declaration/module edges; remove import extraction and guessed provider synthesis. |
| Host `fn_nodes`/provider registries | Dispatch uses accepted declaration identity; caches cannot choose declarations. |

Allowed projections: `TypeBinding`, `DeclaredFuncSig`, `ResolvedFuncSig`; exhaustive diagnostic renderings; `DependencyView.BindsTo` with source=candidate terminal and dependent/usage_site=occurrence terminal; identity-keyed intern/emitter/method/cache representation; member selection after a uniquely bound receiver. None may repair an unbound/ambiguous receiver. Raw-name fallback, priority, provider guesses or weaker dedupe reclassify a projection as competing.

Classify callers before deleting yellow-zone `recursive_type_set`, `inductive_fields`, `lambda_param_provenance`, field/method summaries and serialization/display caches. A raw-name key is defective when equal-spelled declarations can affect each other's semantics.

## Validation

Execute the production candidate producer and fold; algebra-only claims are insufficient:

| Witness | Result |
| --- | --- |
| One pattern binder/use; nested distinct names | Bound to exact path. |
| Equal names in sibling arms | Separate unique paths. |
| Pattern plus nested let/lambda of same name | Ambiguous with both ordered paths. |
| Same-scope duplicates, where grammar permits | Ambiguous at affected references. |
| Repeated discovery of one full path | One Bound candidate. |
| Same terminal at different paths | Ambiguous. |
| Equal type/value spelling | Only syntax-admissible category participates. |
| Local and callable values with same on-chain segment | Ambiguous. |
| Unused nested homonym | No reference diagnostic. |
| Empty population | Located Unbound. |

Perturb every 0/1/many population. RED controls restore nearest-wins/name lookup, remove full-chain collection or drop an ambiguous candidate. Census each occurrence category's unique, unbound, two-way and 3+-way results, candidate-kind pairs, off-chain exclusions and duplicate-path collapse; this is a repair worklist, never fallback permission.

The executing source-graph census covers every name-map reader, ancestor walker, declaration/provider chooser, miss fallback, import/registry visibility conversion and ambiguity/unbound/collision diagnostic. Record authored location, occurrence category, input identity, output, callers, `Authority | Projection | Transitional | Competing` and named dissolve-on. Fail unclassified entries or two decisions for one occurrence, even when answers agree. Discovery words (`shadow`, `nearest`, `first_hit`, `global_bare`, `overlay`, `*_by_name`, `lookup_*`, `fallback`, `provider`, `variant_owner`, `source_visible`, `Ambiguous`, `Unresolved`, `Collision`) seed the census; they are not its semantic gate.

Completion requires exact pattern identity through parse→infer; one full-chain fold; no silent homonym rebinding; exhaustive ordered diagnostics; all semantic consumers Bound-only; every competing caller gone; accepted-edge dependency/diagnostic/emitter/runtime projections; ownership without name/clone fallback; and executed equality of typecheck, emitter and runtime declaration identity. Ordinary compile matrix, v1 build, regeneration fixed point, diagnostic accounting and full CI must pass before mechanical import deletion.
