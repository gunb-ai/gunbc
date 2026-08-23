# R1 reference-layer producer trace (population-independent)

This is a producer trace, not a population census. It does not inherit the `E0308` partition's
R1 count, create a second classifier, or claim completion. A reference-layer mechanism can surface
under several rustc codes, and the cross-code classifier owns its membership. Symbols below are
the durable identities; generated Rust positions are deliberately not used as authority.

## Membership authority

The v1 Rust emitter builds one bare-name set in `v1.compiler.emit_rust` `build_shared_types`.
`maybe_mark_shared_type` admits a `TypeSummary` when the Rust target needs sharing and the summary
is a non-unit struct/coproduct that is neither a grounded native coproduct alias nor a type
constant. `build_shared_types` also admits target collection-template names. This answers one
question at type grain: whether a declared carrier belongs to the seed emitter's shared set.

That set is not an absolute identity authority. Its key is `TypeSummary.name: String`, and
`rust_carrier_is_at_shared_layer` projects a resolved node through
`rust_fn_sig_leaf_name`/`qualified_last_segment` back to that bare leaf before membership lookup.
Two declarations with the same leaf therefore cannot carry different sharing facts. The source
already records this ceiling in `rust_carrier_is_at_shared_layer` and correctly calls the result
mitigatable, not structural.

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

Therefore the answer is neither “one producer at three depths” nor “three roots” yet. There is one
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

## Reporting rule

This trace reports symbols and producer relationships only. It reports no board share, no root
rank, and no completion count. The code-local partition remains a historical projection; the
cross-code classifier supplies membership, and only that membership can define a closing census.
