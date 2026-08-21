# T5b decided: closure-bearing declarations split into two dispositions, not one repair

**Session `tidy-dove-648`. Work item:** `node://adhoc-c3017faf-ad6` — "E0277 root T5b (35 sites,
largest): serde/Debug is demanded over closure-bearing values and NO derive can be added — decide
the modeling question, do not repair." This document is that decision. It does not touch
`trait_derive_emit.dag`, `04_emit_info.dag`, `05_emit_rust.dag`, or `target_model.dag` — the
repairs it hands off are named at the end, scoped and ready to dispatch, but not implemented here.

**Inputs.** Three prior receipts, read and independently re-verified against the live `.dag`
declarations rather than trusted as transcribed numbers (§3's citation rule — symbols below are
grep-checked, not carried over):

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
the trait-bound grain and require opposite treatment.** Collapsing them into one answer — "drop
serde/Debug everywhere a closure appears" — would be wrong for the second population; "keep
serde/Debug and find a wrapper" would be wrong for the first. Distinguishing them is the actual
modeling work this root needed.

### Population 1 — process-local realization values: drop serde/Debug/PartialEq, keep Clone only

`ValueInterpreter`, `TransformInterpreter`, `BranchInterpreter`, `LoopInterpreter`,
`BindInterpreter`, `MatchInterpreter` (`src/v2/std/runtime.dag:308-336`), the
`RuntimeBehaviorInterpreter` coproduct wrapping them (`:356-362`), `InterpretationAlgebra`
(`:338-345`), `CompiledLexRule`/`LexWalkAcc` (`src/v2/compiler/01_tokenize.dag:232,421`),
`PartialFunction<K,V>` (`dag/std/algebra.dag:130`), and `EffectIoEvalBundle` /
`EffectIoEvalContext` / `EffectIoYieldOutcome` (`src/v2/compiler/05_eval.dag:230,237,242`) are, by
construction, **evaluator plumbing that exists only inside one compile process.** None is ever
written to disk, sent across a process boundary, or logged as a debug artifact in its own right —
each is built from a live `EvaluationAlgebra`/handler table at the start of a run and discarded at
the end. Requiring `Serialize`/`Deserialize`/`Debug` on them was never a real consumer need; it was
the derive roster applying uniformly to every record and coproduct regardless of what the
declaration actually is.

**This is not a new call.** The codebase has already answered this exact question, once, for the
direct case: `v1.trait_derive_emit` `v1_emit_struct_derives` (`src/v1/trait_derive_emit.dag:541`)
special-cases `has_fn_fields` to emit `fn_field_derive_traits()` — Clone, and nothing else
(`dag/std/trait_derive_shape.dag:257`) — and `docs/plans/self-host-cargo-refusal-root-partition.md`
§11.19 already states it in words: *"`CompiledLexRule` emits `#[derive(Clone)]` and nothing else,
because `fn_field_derive_traits()` is Clone-only — correctly, since `Rc<dyn Fn>` is not serializable
and not `Debug`."* Population 1's decision is: **apply that existing, already-correct rule
consistently**, not invent a new one. Per DESIGN §5, whether a declared type transitively reaches a
function value is a decidable, structural property, so this is a construction wall, not a
per-declaration validation call — one authority (reachability), derived once, not re-litigated
per type.

**Why 32 of these 35 sites exist despite the rule already being correct: it is wired for structs
but not for coproducts, and the transitive walk cannot see through either.** Confirmed by reading
`src/v1/04_emit_info.dag`, not inferred from the site count:

- `enum_derives` (`src/v1/05_emit_rust.dag:5607`) calls `v1_emit_enum_derives`
  (`src/v1/trait_derive_emit.dag:572`) with **no `has_fn_fields` parameter at all** — the struct
  path's `has_fn_fields` branch (`trait_derive_emit.dag:549`) has no enum counterpart, so a
  coproduct can never be routed to `fn_field_derive_traits()` regardless of what its variants
  carry. This is why `RuntimeBehaviorInterpreter`, a coproduct whose every variant wraps a
  closure-bearing struct, still derives the full `payload_coproduct_derive_traits()` roster
  (Debug/Clone/PartialEq/Serialize/Deserialize) — accounting for 18 of the 44 occurrences /
  a majority of the 35 E0277 sites by itself.
- `build_type_summary`'s enum branch (`src/v1/04_emit_info.dag:395-404`) hardcodes
  `field_type_map: empty_map()` for every enum, where the struct branch
  (`:386-394`) populates it from the real fields. `type_summary_reaches_fn`
  (`04_emit_info.dag:409-421`) walks exactly that map to find transitive closure-reachability, so
  an enum's variant payload types are **structurally invisible** to the fixpoint that already
  exists to answer this question (`close_fn_fields`, `:426-`). `InterpretationStructureWitness`
  (6 sites, and it holds only `Symbol` fields — `runtime.dag:347-354`) is not a second modeling
  case; it is this same blind spot's collateral, most likely misattributed by the census's
  200-line proximity heuristic to a neighboring declaration that legitimately fails. It is
  expected to dissolve, unmeasured on its own, once the two gaps above are closed — not a third
  disposition.

**Handoff (repair, not modeling — do not do this in this PR):** thread `has_fn_fields` through
`v1_emit_enum_derives` the same way `v1_emit_struct_derives` already consumes it, and populate
`field_type_map` for enum variant payload fields so `type_summary_reaches_fn` can see through a
coproduct the way it already sees through a struct. Both are decidable, mechanical, and covered by
the fixpoint infrastructure that already exists — this is completing an existing wall, not
authoring a new one.

### Population 2 — dispatch fused into an interface record: split it out, don't strip the record

`ProducedDeclSupport` (`src/v2/std/compilers/target_model.dag:154-160`) is different in kind, not
degree:

```
type ProducedDeclSupport
  = ProducedDeclUnwired
  | ProducedDeclWired {
      render: fn(Node) -> Outcome<TargetBodiedArrowStatementScaffold>
      scaffold_relation_rule_name: Symbol
      scaffold_base_row: Node
    }
```

`ProducedDeclSupport` is a field of `TargetModel` (`:162-170`), which is not process-local
plumbing — it is the **per-target-language configuration record** built once per language
(`rust_target_model()`, `python_target_model()`, and eight more call sites across
`src/v2/extdeps/languages/*.dag` and `src/v2/extdeps/{github,bmc}/*`, `src/v2/extdeps/formats/*`)
and consumed pervasively through the emit pipeline. Stripping `Debug`/`Serialize`/`Deserialize`
from `TargetModel` the way population 1 drops them would be a real loss — comparing, inspecting,
and (per the `06_translate` consumers) potentially persisting target-model facts is exactly the
kind of thing this record exists for, unlike a `ValueInterpreter` closure table nobody ever prints.
So population 1's answer does not transfer here, and this is where the "operator question" in
§11.23 actually lives.

**The field is redundant with a fact already on the same variant.** `ProducedDeclWired` already
carries `scaffold_relation_rule_name: Symbol` — a named, resolvable identity for *which* rule
renders the scaffold. `render` is that rule's own dispatch, embedded a second time as a live
closure sitting beside the name that already identifies it. This is exactly the shape DESIGN §3
rules out directly: *"the dispatch that selects a realization is itself realization... A pure spec
is dispatch-free; a `std` projection that matched over its realizations would have to name them,
fusing dispatch back in."* `ProducedDeclWired` is supposed to be the *interface* fact (which rule,
over which base row) that `TargetModel` carries as configuration; embedding the closure makes it
carry the *realization* as well, in the same field set, which is why it — alone among its
siblings — cannot be part of a serializable record.

**Decision: remove `render` from `ProducedDeclWired`.** The variant keeps
`scaffold_relation_rule_name: Symbol` and `scaffold_base_row: Node` — both plain data, both already
derivable under the standard record roster. The actual
`fn(Node) -> Outcome<TargetBodiedArrowStatementScaffold>` behavior moves to a peripheral,
non-serialized dispatch table (a `Map<Symbol, fn(Node) -> Outcome<...>>` or equivalent registry)
keyed by `scaffold_relation_rule_name`, resolved only at the point a scaffold is actually rendered
— never carried inside `TargetModel` itself. This is the named-resolvable-reference answer from
§11.23's three options, chosen over "drop serde/Debug" (wrong here — throws away a real
requirement) and over "split into description + realization as two fields on the same record"
(redundant here — the description, the rule name, already exists; adding a second field beside it
would be nicknaming the same identity twice, exactly what DESIGN §3 forbids). Once `render` is
gone, `ProducedDeclSupport`, `TargetModel`, and everything that contains them keep full
Debug/Clone/PartialEq/Serialize/Deserialize, undiminished — the fix is removing a misplaced fact,
not weakening the record that held it.

**The other four `v2_std_compilers_target_model.rs` sites are very likely the same collateral
pattern as `InterpretationStructureWitness`, not independent decisions.**
`TargetDeriveSupplementalGenericBoundContractAuthority`, `TargetDeriveSupplementalGenericBoundContract`,
`TargetCollectionRealization` (`target_model.dag:9526-9549`), and `TargetRepresentationParameterSlot`
— confirmed by reading the source to be **an empty struct**, `TargetRepresentationParameterSlot {}`
(`:9524`) — hold no function field of their own, directly or (by inspection of their declared
fields) transitively. Their most likely diagnostic cause is proximity to `ProducedDeclSupport`
within the same module/derive-refusal cluster, the same 200-line-window attribution effect the
prior receipt already flagged for `InterpretationStructureWitness`. **Do not author a fix for
these four independently.** Re-measure this file after the `render` removal lands; whatever
remains at that point is a real, distinct defect and gets its own row, not an assumption now.

## What this decision resolves and what it hands off

| population | declarations | disposition | this document | handoff |
|---|---|---|---|---|
| 1 — realization values | `ValueInterpreter`+5 siblings, `RuntimeBehaviorInterpreter`, `InterpretationAlgebra`, `CompiledLexRule`, `LexWalkAcc`, `PartialFunction<K,V>`, `EffectIoEvalBundle`/`EvalContext`/`YieldOutcome` | Clone-only; serde/Debug never legitimate | decides + grounds the rule already implicit in `fn_field_derive_traits()` | wire `has_fn_fields` through `v1_emit_enum_derives`; populate enum `field_type_map` in `04_emit_info.dag` |
| 1 (collateral) | `InterpretationStructureWitness` | expected to dissolve with population 1's repair | names it, does not fix it | re-measure after the repair, do not pre-emptively touch |
| 2 — dispatch-in-interface | `ProducedDeclSupport`/`ProducedDeclWired` | drop the embedded `render`; dispatch by the existing `scaffold_relation_rule_name` in a peripheral registry | decides the split and which existing fact survives as the identity | remove the field, add the registry, rewire the ~10 `TargetModel` construction sites that build a `ProducedDeclWired` |
| 2 (collateral, unconfirmed) | `TargetDeriveSupplementalGenericBoundContractAuthority`, `TargetDeriveSupplementalGenericBoundContract`, `TargetCollectionRealization`, `TargetRepresentationParameterSlot` | presumed collateral of population 2; not independently decided | names the four, declines to assume | re-measure after population 2's repair; triage what remains then |

**What is not claimed.** This document does not re-run the probe, so it does not confirm the
35-site count drops to zero, nor does it confirm the four collateral declarations in population 2
resolve on their own — both are stated as expectations grounded in the reachability mechanism
being fixed, not as measured outcomes. Per DESIGN §16 of the shared partition doc, a site count
measures where the compiler pointed; closing this decision is a repair PR's job, verified by
re-running `docs/probes/curated_cargo_probe_one.sh` against the same entry set after both handoffs
land, not by this one.
