## Modeling Faithfulness Invariant

The compiler and the `.dag` source share one governing principle:
every construct must be grounded in an identifiable external fact
(axiom, specification, standard, or structural derivation). Constructs
without factual grounding are not valid authorities in this codebase.

Full modeling guidelines: [`MODELING.md`](MODELING.md).

**The compiler's role:** enforce faithfulness mechanically. When the
compiler encounters a type, coercion, or structural claim for which no
grounding fact is declared, it must produce a diagnostic error. Silent
defaults, fabrication fallbacks, and placeholder emissions are violations
of this invariant — they allow ungrounded claims to propagate through
the pipeline as if they were facts.

**Annotations are not facts.** When a structural gap requires new
information, the fix is to extend `.dag` structure (types, edges,
functions), not to add metadata or annotations. The `.dag` language is
the meta-language for expressing intersubjective agreements; there is
no meta-language above it.

**No annotation mechanisms at any layer (ruled out through M3).** The
rule against annotations applies at every layer of the stack, not
just at the `.dag` source level:

- **Language level.** `.dag` has no annotations, attributes, pragmas,
  semantic comments, or side-channels for attaching metadata to
  declarations. Every fact is a first-class structural piece of the
  language. If a feature feels like it wants an annotation, that is
  a signal the core language is missing a structural primitive.
  The fix is to add the primitive, not the annotation.

- **Compiler data model level.** The DAG substrate (Port, Behavior,
  Declaration, diagnostics) carries only structural facts load-bearing
  for causal correctness — types, spans, `produced_by` edges, port
  state, declared signatures. It does NOT grow "annotation tables,"
  "attribute maps," or side storage for lens-produced derived facts.

- **Lens level.** Lenses are pure functions from `&Dag` to derived
  values, not annotation mechanisms. A lens reads the DAG and
  computes its answer on demand. It does not write results back into
  the Dag for later consumers. Cross-lens queries combine per-lens
  call results at the call site, not via a shared annotation store.

**Why this is ruled out now:** the core language is still being
discovered. Until it is clear what structural primitives the language
actually needs, allowing annotations as an escape hatch would fill
real gaps with a decorative layer and make the gaps invisible. The
annotation layer would accrete consumers, become load-bearing, and
prevent the language from growing the structural primitives it needs.
This is exactly the pattern v2 hit — metadata bolted on to cover
missing type declarations, then impossible to remove because downstream
code depended on the metadata. The ruling: if it feels like it wants
an annotation, that is a discovery opportunity, not an implementation
question. Bring the discovery back to the language design, not to the
compiler's data model.

**When this may be revisited:** at M3 (self-hosting) or later, once
the core language has survived contact with real code and the
structural primitives have stabilized. Before M3, the default answer
is always "no annotations." Contributors should not propose annotation
mechanisms, annotation-like side tables, or "just a small metadata
map for this lens" extensions without explicit authorization that
references this rule.

**Concrete implications for the v3 substrate (M0 – M3):**

- The Dag carries only Port, Behavior, Declaration, diagnostics. No
  `annotations: HashMap<NodeId, _>` field, no `lens_results:`, no
  attribute system. The diagnostics table is not an annotation
  system — it is the fail-closed channel for compile failure, linked
  to ports by a biconditional invariant, and it is the only kind of
  side-lookup allowed in the substrate.
- The provenance lens reads `produced_by` and classifies by behavior
  kind. The depth lens walks ports. Any future lens (cost, ownership,
  effect, termination) is also a pure function of `&Dag`. None of
  them write back.
- The success bar "adding a new analysis is trivial" is measured
  concretely: write a pure function from `&Dag` to your derived
  type, in its own file, with zero substrate modifications. If a
  proposed lens cannot be built this way, the failure mode is to
  revisit the substrate's structural facts (is a needed fact missing
  from Port/Behavior/Declaration?), not to add an annotation layer.

This invariant is upstream of all others. Performance invariants assume
the model is faithful. Decidability proofs assume the structures are
well-grounded. Sustainability rules assume facts have single authorities.
If the modeling is unfaithful, the downstream invariants are protecting
the wrong thing.

