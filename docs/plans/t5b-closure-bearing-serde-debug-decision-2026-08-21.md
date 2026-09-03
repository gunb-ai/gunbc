# T5b decided: closure-bearing declarations split into two dispositions, not one repair

**Session `tidy-dove-648`. Work item:** `node://adhoc-c3017faf-ad6` — "E0277 root T5b (35 sites,
largest): serde/Debug is demanded over closure-bearing values and NO derive can be added — decide
the modeling question, do not repair." This document is that decision. It does not touch
`v1.trait_derive_emit`, `v1.04_emit_info`, `v1.05_emit_rust`, or `v2.std.compilers.target_model` —
the repairs it hands off are named at the end, scoped but not implemented here.

**Inputs.** Three prior receipts, re-verified against the live `.dag` declarations rather than
trusted as transcribed numbers (§3's citation rule — symbols below are grep-checked by name, not
carried as positions):

- `docs/probes/e0277_partition_2026-08-21.md` (PR #8712, on `main`) — first framed root 1 ("serde/Debug
  derive on a function-valued carrier") as impossible-to-derive and structural.
- `docs/plans/self-host-cargo-refusal-root-partition.md` §11.19 and §11.23 (on `main`) — named the
  same root T5b, attributed 44 sites to ~12 enclosing declarations, and posed the open question
  verbatim: *"should a declaration whose value contains a function be serializable at all?"*
- `docs/probes/e0277_root_partition_2026-08-21.md` (session `bright-moth-92`, PR #8731, **not yet
  merged**) — re-measured T5b at E0277-only, site grain: **35 sites**, concentrated in
  `v2_std_runtime.rs` (13), `v2_std_compilers_target_model.rs` (11), and three compiler-stage files
  (11). Cited here as evidence, not as merged fact — its branch is `origin/session/bright-moth-92`.

## The decision

**T5b is not one modeling question with one answer. It is two populations that look identical at
the trait-bound grain and require opposite treatment.** "Drop serde/Debug wherever a closure
appears" is wrong for the second; "keep serde/Debug and find a wrapper" is wrong for the first.
Distinguishing them is the modeling work this root needed.

### Population 1 — process-local realization values: drop serde/Debug/PartialEq, keep Clone only

`v2.std.runtime` `ValueInterpreter`, `TransformInterpreter`, `BranchInterpreter`, `LoopInterpreter`,
`BindInterpreter`, `MatchInterpreter`; the `v2.std.runtime` `RuntimeBehaviorInterpreter` coproduct
wrapping them; `v2.std.runtime` `InterpretationAlgebra`; `v2.compiler.01_tokenize`
`CompiledLexRule`/`LexWalkAcc`; `std.algebra` `PartialFunction<K,V>`; and `v2.compiler.05_eval`
`EffectIoEvalBundle` / `EffectIoEvalContext` / `EffectIoYieldOutcome` are, by construction,
**evaluator plumbing that exists only inside one compile process.** None is written to disk, sent
across a process boundary, or logged as a debug artifact — each is built from a live
`EvaluationAlgebra`/handler table at run start and discarded at the end. `Serialize`/`Deserialize`/
`Debug` on them was never a consumer need; it was the derive roster applied uniformly regardless of
what the declaration is.

**This is not a new call.** `v1.trait_derive_emit` `v1_emit_struct_derives` already special-cases
`has_fn_fields` to emit `std.trait_derive_shape` `fn_field_derive_traits()` — Clone, nothing else —
and `docs/plans/self-host-cargo-refusal-root-partition.md` §11.19 states it: *"`CompiledLexRule`
emits `#[derive(Clone)]` and nothing else, because `fn_field_derive_traits()` is Clone-only —
correctly, since `Rc<dyn Fn>` is not serializable and not `Debug`."* Population 1's decision:
**apply that existing, correct rule consistently.** Per DESIGN §5, whether a declared type
transitively reaches a function value is decidable and structural, so this is a construction wall,
not per-declaration validation — one authority (reachability), derived once.

**Why 32 of these 35 sites exist despite the correct rule: it is wired for structs but not
coproducts, and the transitive walk cannot see through either.** Confirmed by reading
`v1.04_emit_info`, not inferred from the site count:

- `v1.05_emit_rust` `enum_derives` calls `v1.trait_derive_emit` `v1_emit_enum_derives` with **no
  `has_fn_fields` parameter at all** — `v1_emit_struct_derives`'s `has_fn_fields` branch has no
  enum counterpart, so a coproduct is never routed to `fn_field_derive_traits()` whatever its
  variants carry. Hence `RuntimeBehaviorInterpreter`, a coproduct whose every variant wraps a
  closure-bearing struct, still derives the full `payload_coproduct_derive_traits()` roster
  (Debug/Clone/PartialEq/Serialize/Deserialize) — 18 of the 44 occurrences / a majority of the 35
  E0277 sites by itself.
- `v1.04_emit_info` `build_type_summary`'s enum branch hardcodes `field_type_map: empty_map()` for
  every enum, where its struct branch populates `field_type_map` from the real fields.
  `v1.04_emit_info` `type_summary_reaches_fn` walks exactly that map for transitive
  closure-reachability, so enum variant payload types are **structurally invisible** to the
  existing fixpoint (`v1.04_emit_info` `close_fn_fields`). `v2.std.runtime`
  `InterpretationStructureWitness` (6 sites, holding only `Symbol` fields) is not a second modeling
  case but this blind spot's collateral, most likely misattributed by the census's 200-line
  proximity heuristic to a neighboring declaration that legitimately fails. It is expected to
  dissolve once the two gaps above close — not a third disposition.

**Handoff (repair, not modeling — do not do this in this PR):** thread `has_fn_fields` through
`v1_emit_enum_derives` as `v1_emit_struct_derives` already consumes it, and populate
`field_type_map` for enum variant payload fields so `type_summary_reaches_fn` sees through a
coproduct as it does a struct. Both are decidable, mechanical, and covered by the existing fixpoint
infrastructure — completing an existing wall, not authoring a new one.

### Population 2 — dispatch fused into an interface record: split it out, don't strip the record

`v2.std.compilers.target_model` `ProducedDeclSupport` is different in kind, not degree:

```
type ProducedDeclSupport
  = ProducedDeclUnwired
  | ProducedDeclWired {
      render: fn(Node) -> Outcome<TargetBodiedArrowStatementScaffold>
      scaffold_relation_rule_name: Symbol
      scaffold_base_row: Node
    }
```

`ProducedDeclSupport` is a field of `v2.std.compilers.target_model` `TargetModel`, which is not
process-local plumbing but the **per-target-language configuration record** built once per
language (`rust_target_model()`, `python_target_model()`, and eight more call sites across
`src/v2/extdeps/languages/*.dag`, `src/v2/extdeps/{github,bmc}/*`, `src/v2/extdeps/formats/*`)
and consumed pervasively through the emit pipeline. Stripping `Debug`/`Serialize`/`Deserialize`
from `TargetModel` would be a real loss — comparing, inspecting, and (per the `06_translate`
consumers) potentially persisting target-model facts is what this record exists for, unlike a
`ValueInterpreter` closure table nobody prints. Population 1's answer does not transfer; this is
where §11.23's "operator question" lives.

**The field is redundant with a fact already on the same variant.** `ProducedDeclWired` already
carries `scaffold_relation_rule_name: Symbol` — a named, resolvable identity for *which* rule
renders the scaffold. Its `render` field is that rule's own dispatch, embedded a second time as a
live closure beside the name that identifies it — the shape DESIGN §3 rules out: *"the dispatch
that selects a realization is itself realization... A pure spec is dispatch-free; a `std`
projection that matched over its realizations would have to name them, fusing dispatch back in."*
`ProducedDeclWired` should be the *interface* fact (which rule, over which base row) that
`TargetModel` carries as configuration; embedding the closure makes it carry the *realization*
too, which is why it alone among its siblings cannot be part of a serializable record.

**Decision: remove `render` from `ProducedDeclWired`.** The variant keeps
`scaffold_relation_rule_name: Symbol` and `scaffold_base_row: Node` — plain data, derivable under
the standard record roster. The `fn(Node) -> Outcome<TargetBodiedArrowStatementScaffold>` behavior
moves to a peripheral, non-serialized dispatch table (a `Map<Symbol, fn(Node) -> Outcome<...>>` or
equivalent registry) keyed by `scaffold_relation_rule_name`, resolved only where a scaffold is
rendered — never carried inside `TargetModel`. This is the named-resolvable-reference answer from
§11.23's three options, chosen over "drop serde/Debug" (wrong here — throws away a real
requirement) and over "split into description + realization as two fields on the same record"
(redundant — the description, the rule name, already exists; a second field beside it would
nickname one identity twice, which DESIGN §3 forbids). With `render` gone, `ProducedDeclSupport`,
`TargetModel`, and everything containing them keep full Debug/Clone/PartialEq/Serialize/Deserialize
— the fix removes a misplaced fact, not the record's strength.

**SUPERSEDED, AND IN THE SAME DIRECTION.** The `render` field is gone, but not into a peripheral
registry keyed by `scaffold_relation_rule_name`: rendering is now DECLARED ROWS on the variant
(`v2.std.compilers.target_model` `ProducedDeclRenderRows`, read by the single shared fold
`produced_decl_render_from_rows`), and each target declares its own signature order as data
(`v2.extdeps.languages.rust` `rust_produced_decl_render_rows`, `v2.extdeps.languages.c`
`c_produced_decl_render_rows`). A registry would have kept the fn alive one indirection away — still
origin-bound, which is exactly what blocked the required floor from sharing one evaluation of a
target model across claim frames (`v2.workflow.floor_pure_producer_share`). Rows delete the fn
outright, so `TargetModel` is a pure content value; the interface/realization split is preserved,
the target still owns its order, and the compiler holds no per-target arm.

**The other four `v2_std_compilers_target_model.rs` sites are very likely the same collateral
pattern as `InterpretationStructureWitness`, not independent decisions.**
`v2.std.compilers.target_model` `TargetDeriveSupplementalGenericBoundContractAuthority`,
`TargetDeriveSupplementalGenericBoundContract`, `TargetCollectionRealization`, and
`TargetRepresentationParameterSlot` — confirmed by source read to be **an empty struct**,
`TargetRepresentationParameterSlot {}` — hold no function field directly or (by inspection of their
declared fields) transitively. Their likeliest cause is proximity to `ProducedDeclSupport` within
the same module/derive-refusal cluster — the 200-line-window attribution effect the prior receipt
flagged for `InterpretationStructureWitness`. **Do not author a fix for these four independently.**
Re-measure this file after the `render` removal lands; whatever remains is a real, distinct defect
and gets its own row.

## What this decision resolves and what it hands off

| population | declarations | disposition | this document | handoff |
|---|---|---|---|---|
| 1 — realization values | `ValueInterpreter`+5 siblings, `RuntimeBehaviorInterpreter`, `InterpretationAlgebra`, `CompiledLexRule`, `LexWalkAcc`, `PartialFunction<K,V>`, `EffectIoEvalBundle`/`EvalContext`/`YieldOutcome` | Clone-only; serde/Debug never legitimate | decides + grounds the rule already implicit in `fn_field_derive_traits()` | wire `has_fn_fields` through `v1_emit_enum_derives`; populate enum `field_type_map` in `v1.04_emit_info` |
| 1 (collateral) | `InterpretationStructureWitness` | expected to dissolve with population 1's repair | names it, does not fix it | re-measure after the repair, do not pre-emptively touch |
| 2 — dispatch-in-interface | `ProducedDeclSupport`/`ProducedDeclWired` | drop the embedded `render`; dispatch by the existing `scaffold_relation_rule_name` in a peripheral registry | decides the split and which existing fact survives as the identity | remove the field, add the registry, rewire the ~10 `TargetModel` construction sites that build a `ProducedDeclWired` |
| 2 (collateral, unconfirmed) | `TargetDeriveSupplementalGenericBoundContractAuthority`, `TargetDeriveSupplementalGenericBoundContract`, `TargetCollectionRealization`, `TargetRepresentationParameterSlot` | presumed collateral of population 2; not independently decided | names the four, declines to assume | re-measure after population 2's repair; triage what remains then |

**What is not claimed.** This document does not re-run the probe, so it confirms neither that the
35-site count drops to zero nor that population 2's four collateral declarations resolve on their
own — both are expectations grounded in the reachability mechanism being fixed, not measured
outcomes. Per DESIGN §16 of the shared partition doc, a site count measures where the compiler
pointed; closing this decision is a repair PR's job, verified by re-running
`docs/probes/curated_cargo_probe_one.sh` against the same entry set after both handoffs land.
