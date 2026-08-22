# Proposing a variant's parent enum as a use-line candidate: FALSIFIED BY EXECUTION, inert

**Result:** built, measured, reverted. 66 defects before, 66 after. Published so no
lane rebuilds it, and because the reason it failed qualifies a carrier filed the
same day.

## What was built

`reference_derived_candidate_authored` already ADMITS a parent enum whose variant
is spelled in the module — `reference_derived_variant_induced_parent_spelled`
exists and works. Nothing ever PROPOSED such a parent, so the admission was
unreachable. The obvious repair is a producer: for every collected reference
name that `variant_to_enum` maps, propose the parent.

## Why it did not work

It fires, and its parents are admitted — measured, not assumed:

```
VIP-HIT vn=CheckpointAssertion parent=CoercionAssertion authored=true
VIP-HIT vn=Dag              parent=RenderTarget         authored=true
VIP-HIT vn=Realized         parent=TypeRealizationDecision authored=true
```

But for the target it never fires at all. Instrumenting both arms, `Resolved`
appears as neither a hit nor a miss in `v1.compiler.coercion` — it is **not in the
collected reference names in the first place**. The reference lives in a match
pattern:

```dag
Present { value: v1.std.core.Resolved { node: rt } } => decl_identity_file(item: rt)
```

and the reference collectors do not walk pattern heads. The producer was added
one stage downstream of where the name is lost.

Net effect: three mirrors gained `Quantity`, `Scale`, `FermiDepth` use-lines that
fixed no diagnostic. A change that alters emitted output without repairing
anything is worse than inert, so it was reverted rather than kept as harmless.

## What this qualifies

`emitter_synthesized_reference_use_line_blindness_2026-08-22.md` states the E0425
residue is a CEILING: a source-derived collector is structurally blind to
emitter-synthesized references, because the name is not in source for any walk to
find. **That claim is too strong for the variant-induced subclass.** For
`InferredNode::Resolved`, the enum name is indeed absent from source — but the
VARIANT is present, and `variant_to_enum` is already in the emitter. The
reference is therefore DERIVABLE from source plus a map the emitter already
holds, which is not a ceiling; it is an uncollected position.

The ceiling claim stands for the generated sort comparator, the turbofish on a
generated empty map, and the emitter-produced generic instantiation — those have
no source antecedent of any kind. It does not stand, unqualified, for a
reference whose variant is authored in a pattern.

## Next move, for whoever takes it

Collect variant references from PATTERN position, not just value and type
position. Then the existing admission path and this producer both become
reachable. Until that lands, the E0433 population of 12 over 5 identities
(`VarBindingKind`, `MatchPattern`, `InferredNode`, `GlobalBareLookupState`,
`CollectionSizeEffect`) is blocked on the collector, not on the candidate rules.
