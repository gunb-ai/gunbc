# `shared_types` membership-authority audit (R1 producer trace, population-independent)

This is a producer trace, not a population census. It does not inherit the `E0308` partition's
R1 count, create a second classifier, or claim completion. A reference-layer mechanism can surface
under several rustc codes, and the cross-code classifier owns its membership. Symbols below are
the durable identities; generated Rust positions are deliberately not used as authority.

## What fact does `shared_types` represent?

The v1 Rust emitter builds one bare-name set in `v1.compiler.emit_rust` `build_shared_types`.
`maybe_mark_shared_type` admits a `TypeSummary` when the Rust target needs sharing and the summary
is a non-unit struct/coproduct that is neither a grounded native coproduct alias nor a type
constant. `build_shared_types` also admits target collection-template names. The closest faithful
reading of the constructed set is:

> For the selected target, this bare declared name is predicted from its source representation
> shape to need the target's shared indirection realization.

That sentence exposes the fusion. The producer starts from a **declaration fact** (`TypeSummary`
shape, recursive/type-constant exclusions), applies a **target realization policy**
(`sharing_for_target(target).needs_sharing` plus target collection-template rows), and stores only a
bare `String`. It does not retain the declaration identity from which the prediction came, the
target realization identity selected, the layer (`Rc`, `Box`, or owned), or the use-site position.
It is therefore not a general fact named “this type is shared.” It is a lossy, target-specific
Boolean projection whose name and carrier make it available as though it answered all of those
questions.

That set is not an absolute identity authority. Its key is `TypeSummary.name: String`, and
`rust_carrier_is_at_shared_layer` projects a resolved node through
`rust_fn_sig_leaf_name`/`qualified_last_segment` back to that bare leaf before membership lookup.
Two declarations with the same leaf therefore cannot carry different sharing facts. The source
already records this ceiling in `rust_carrier_is_at_shared_layer` and correctly calls the result
mitigatable, not structural.

## Consumer audit — three different information needs

The direct readers partition by what their decision actually requires. Passing `Set<String>` to
all three groups hides this distinction; it does not make their questions the same.

| required fact | symbolic consumers | why the present set is insufficient or excessive |
|---|---|---|
| **Declaration identity** | `render_node_type`; `render_rust_decl_type`; `render_rust_alias_rhs_type`; `emit_rust_expr_record_lit`; `emit_typed_record_lit`; `emit_field_value_with_context` | These resolve a type/constructor or recurse into its arguments. A bare leaf cannot distinguish equal spellings declared by different modules, and the correct layer must follow the resolved declaration through every nested position. They need a declaration-keyed lookup; the Boolean is a projection after that join. |
| **Realization identity** | `rust_render_checkpoint_scalar_bare`; `render_rust_shared_type_if_needed`; `needs_box_wrapping`; `v1_emit_struct_derives`; `v1_emit_struct_from_capability_table` | These decide Rust representation: native scalar vs structural carrier, `Rc` vs already-indirect/recursive `Box`, and heap-capable vs copy-capable derive surfaces. “Member” does not say which realization was selected or whether indirection is already supplied. The rendered-string prefix guard is evidence that realization identity was discarded and reconstructed from bytes. |
| **Boolean projection only** | `rust_carrier_is_at_shared_layer` consumers (`rust_field_carrier_final_type`, `analyze_rc_match`, `value_inferred_type_is_rc_wrapped`); `analyze_rc_pattern`; `variant_ref_self_wraps`; `emit_typed_expr_base`; `emit_discriminant_call_scrutinee_lowering`; `emit_typed_call`/`emit_rust_with_method_call`; `emit_rust_fold_method_call`; `emit_data_def` | Once declaration and realization have already been resolved, these only choose a local lowering: wrap a constructor, dereference/clone a value, suppress a second `Box`, or select an Rc-aware pattern. They should consume a total projection such as “this resolved value is at shared indirection,” not the membership set or a rendered spelling. Several currently redo the bare-name lookup themselves, so they can disagree even though their required output is only Bool. |

Two qualifications keep the table honest. First, `needs_box_wrapping` consumes realization identity
to establish *which* indirection satisfies sizedness, even though its immediate return is Bool;
classifying every Bool-returning function as “Boolean only” would confuse output shape with input
grain. Second, trait derivation does not ask whether a use site should be wrapped. It asks which
Rust capabilities the realized declaration supports. Its use of the same membership set is direct
evidence that `shared_types` is serving more than the R1 wrap question.

The audit result is therefore: **one set is answering at least three questions**. Declaration
identity is required to locate the subject, realization identity is required to derive the target
representation, and only then may use-site consumers safely read a Boolean layer projection. The
minimum information flow is not a wider `Set<String>` and not three replacement rosters: resolve a
declaration-keyed realization fact, then derive total projections for the narrower consumers. This
is a requirement exposed by the audit, not a promoted root or an authorization to mint that carrier.

## The projections are not one producer

Outer, generic-argument, and element-depth `Rc` differences must not be presumed to share a root.
The current emitter has several consumers of the bare-name membership, and they apply the layer at
different recursion points:

- `rust_carrier_is_at_shared_layer` is the carrier-level projection. Its current consumers include
  `rust_field_carrier_final_type`, `analyze_rc_match`, and
  `value_inferred_type_is_rc_wrapped`. This consolidated four former local opinions, including the
  struct-field and value-position forks, but remains a Bool projection over the bare-name set.
- `render_rust_shared_type_if_needed` is a rendered-text idempotence projection. It asks membership
  by a supplied name and separately asks `rust_type_is_rc_wrapped` whether the already-rendered
  string begins at the target's shared layer. `render_rust_applied_type`, alias-RHS rendering, and
  several specialized renderers call it directly. This is where an outer layer can be applied
  after a path-specific rendering decision.
- Recursive type rendering decides nested positions independently. `render_rust_applied_type`
  maps arguments through `render_rust_applied_type_arg`; alias RHS rendering recurses through
  `render_rust_alias_rhs_type`; generic `render_node_type` recurses over collection elements and
  generic children while recomputing `set_contains(shared_types, tn)` at each node. Element and
  generic-argument differences can therefore originate below the outer carrier even when the
  outer projection agrees.
- `emit_data_def` still computes `needs_rc` from
  `set_contains(shared_types, authored_name_at(type_node))` and then uses rendered-text prefix
  detection before wrapping its return type. This bypasses the carrier-level projection and is a
  distinct writable opinion.

Therefore the answer is neither “one producer at three depths” nor “three roots” yet. R1 is one
symptom family produced by multiple consumers of insufficient identity information. There is one
membership set, multiple projections of it, and recursive render paths that can consult membership
again below the outer carrier. A cross-code identity join must attribute observations to these
symbolic producer paths before any root lane can be sized.

## Already-consolidated boundary and remaining ceiling

`rust_carrier_is_at_shared_layer` is real convergence work: field declarations, variant record
fields (through `rust_field_carrier_final_type`), match scrutinees, and selected value positions now
ask one carrier question. It must not be described as though the old field-only
`authored_name_at` miss were still the whole mechanism.

The class remains writable because direct `set_contains(shared_types, ...)` and rendered-text
prefix checks survive on other paths. Copying the Bool predicate into every recursive renderer
would add validations to the seed and preserve the bare-name authority. The declared next rung is
the modeled `TargetReferenceLayer` plus `target_layer_transition` in
`v2.std.compilers.target_model`, consumed by `v2.compiler.wrap_decision`. That model is not
reachable from the v1 seed closure today; moving it to a root both compilers can consume is a
model-first carrier migration, not an edit justified by this probe.

This audit does not itself prove that carrier is the root. Per the promotion order, it is the
producer trace between classification and intervention: a single-axis change followed by an exact
site-conversion receipt is still required before any manifestation is promoted to a root.

## Reporting rule

This trace reports symbols and producer relationships only. It reports no board share, no root
rank, and no completion count. The code-local partition remains a historical projection; the
cross-code classifier supplies membership, and only that membership can define a closing census.
