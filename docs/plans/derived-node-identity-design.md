# Derived-node identity in v1 infer

Status: model, step 1 of 2. No infer code changes under this note until step 2 is scheduled.
Owner: clever-lynx-801. Ruling: quiet-gull-780, 2026-10-01; scheduled by neat-boar-16.
Class row: `gunbc.recurring_failure_mode` `generic_identity_decided_by_spelling`.

## 1. The question

When `v1.compiler.infer` **builds** a type node after resolution, what identity does that node
carry, and who says so?

"Builds" covers:
- substituting a generic's binding into a signature,
- instantiating a generic record's fields or an alias chain,
- forming a call plan's formals,
- minting a kernel container type for a value.

Today the answer is: whatever spelling the node happens to have. Every consumer that needs to know
whether a node *is* a generic parameter, rather than merely spelled like one, has nothing else to
read. This note models the identity those nodes must carry.

## 2. What was measured (not inferred)

These facts locate the problem. Their magnitudes are deliberately not transcribed (DESIGN §6);
§7 names the instrument that re-derives them.

- **The spelling collides by construction.** The kernel container type names its element child
  after its own template parameter. Every `List<M>` therefore carries a child spelled `T` and
  resolved to `M`, and it collides with any generic parameter also spelled `T`.
- **Three sites decide generic identity by spelling:**
  - `unify_generics` binds a bare formal by its label;
  - `substitute_generics_apply` replaces a bare node by its label;
  - that function's self-binding guard compares labels.
- **Two identity facts already exist after resolution.** Neither reaches the nodes infer builds:
  - `TypeVariable` (inferred) is bound to each declared parameter by
    `v1.compiler.infer_env` `env_with_type_variable_bindings`. Its id is the bare parameter name.
  - `Node.declaration = (owner, TypeParameter { name })` is written by `v1.compiler.resolve`
    `binder_marked_type` (#12690) on authored signature positions: parameter types, returns, type
    arguments, declaration fields and arrow positions.
- **Coverage, measured over the whole-corpus compile.** A large share of generic bind and
  substitute events act on nodes that carry neither fact in a usable form. They come from about a
  dozen constructors, listed on the class row, led by `infer_call_arguments_generic_pass` and
  `build_call_application_plan`. No magnitudes are given here: per DESIGN §6 they belong to the
  instrument (§7), not to prose.
- **What happened when each fact was read alone.** Reading the `TypeVariable` mark made the
  compiler's next generation refuse its own sources: generic record fields arrive unmarked. Reading
  the declaration mark would stop substituting at every event the constructors above produce.

## 3. Hypotheses step 2 must settle first (labelled as bets, DESIGN §4d)

1. **Read before marking, not lost in copying.** The unmarked call-plan formals are read from a
   signature representation built before `binder_marked_type` runs, not lost while copying.
   `v1.compiler.infer` `bind_local_func_conformance` builds `ResolvedFormal.declared_type` from
   `param_node_type_expr` of the func-env signature. If that signature predates marking, the repair
   at those two sites is to read the marked signature, not to add a carrier. The test is the same
   instrument, tagged by whether the node is reachable from a marked signature.
2. **`substitute_generics_apply` keeps its input's declaration.** It already copies
   `n.declaration` into the node it rebuilds, so it is not a loss site. Its *output* node, the
   binding, carries the binding's identity, which is correct.
3. **The kernel element child needs no parameter spelling.** Its identity is "element position
   of this container". The open question is whether any reader depends on the child being spelled
   `T`. The instrument answers it by deleting the spelling and listing what breaks; quiet-gull-780
   asked for that reader to be named.

## 4. The model

A derived type node carries exactly one **origin**, named by the constructor that builds it, never
reconstructed from the node's shape (bold-fox-455's constraint):

```dag
type DerivedTypeOrigin
  = CarriedDeclaration { declaration: DeclarationRef }    // a copy keeps the identity it was given
  | ParameterBinding { parameter: DeclarationRef }         // the node a generic parameter was bound to
  | KernelElementPosition { container: KernelContainer, position: Int }
  | DerivationRefused { constructor: NonEmptyStr, cause: NonEmptyStr }
```

- **`CarriedDeclaration`** is the default for every copy. A constructor that rebuilds a node from
  another node carries the input's declaration unchanged. This is the copy law, and most of the
  inventory falls under it.
- **`ParameterBinding`** names the generic parameter a substituted node stands in for. It is
  needed only where a consumer asks "which parameter did this come from" (cross-owner keying, §6).
- **`KernelElementPosition`** replaces the spelled `T`. A container's element child is identified
  by its position in a kernel container and is never mistaken for a `TypeParameter`.
- **`DerivationRefused`** is the fail-closed arm. A constructor that cannot name an origin refuses,
  located at the constructor. It never falls back to the spelling.

Generic identity at the three sites becomes: a node is generic parameter `p` exactly when its
origin is `CarriedDeclaration` with `field: TypeParameter`, or a `ParameterBinding` to `p` whose
binding is `p` itself. A self-binding is identity equality. Spelling never enters the decision.

**Laws** (the step-1 carrier states these and its control executes them):
- **L1, copy:** a copied node's origin equals its input's origin.
- **L2, no shape inference:** a constructor that builds a node names its origin, and two nodes
  that are structurally equal but have different origins stay distinct.
- **L3, kernel distinctness:** a `KernelElementPosition` origin is never a `TypeParameter` identity,
  whatever its spelling.
- **L4, refusal:** `DerivationRefused` is never read as a generic parameter or as a concrete type;
  a consumer reaching it refuses.

## 5. Its own relation, or an extension of an existing one (DESIGN §3b)

This is **not a new keying relation**, and **not an extension of `std.occurrence_identity`**.

- **Not #12790.** Occurrence identity answers "which authored occurrence". The nodes in question
  are built by infer and have no authored occurrence, so allocating one would infer their cause
  from their shape. That is option A, which 3e already rejected for the same reason.
- **Not the facts site key (3e).** That key answers "which site of one inferred tree a fact is
  about". It is positional inside one tree and says nothing about what a node denotes.
- **The same relation as `std.decl_ref`, carried further.** Its relation is "this node denotes this
  declaration-level entity". The model extends where that identity is *carried*, through infer's
  constructors, and adds the two arms `DeclarationRef` cannot express: a kernel element position
  and a refusal.

**Reconciliation with A′** (`keying-relation-design.md` §3e, declared model item): A′ is "identity
for every node at infer's input, with each builder naming its cause". This note is A′'s
type-level interior: the same constraint, that builders name their cause, applied to the type
nodes infer builds *inside* the stage. They are one model item, not two. If A′ lands, its
input-side origins and these interior origins are arms of one carrier. This note's carrier is
named and placed so A′ extends it rather than forking it.

**Scope:** type nodes built by `v1.compiler.infer` after resolution, covering the dozen
constructors inventoried in §2. The following are outside this note:
- value-expression nodes;
- other stages;
- v2 infer, which has its own carrier work (#12763 and its line).

## 6. Sequencing

1. **Step 1, in two PRs, with no infer edits:**
   - **1a (#12913):** this note and the class row.
   - **1b (#12914):** the std carrier `std.derived_type_origin`, which declares `DerivedTypeOrigin`
     and `GenericIdentityVerdict`, and its control `test.claim.derived_type_origin_witness_test`,
     which executes L1 to L4 over supplied origins. Until step 2 lands, that control is the
     carrier's only consumer, a declared frontier whose trigger is step 2 (DESIGN §3c).
2. **Step 2:** first settle §3's three bets with the instrument. Then carry origins through the
   inventoried constructors, switch the three sites to identity, and delete their name branches.
   The evidence is:
   - the compiler's next generation builds and compiles its own sources (the control that caught
     the first attempt);
   - a whole-corpus census in both directions at identity grain;
   - the srv3 witness, the `00_compile` native-lane site, and `head_of(xs: [1], d: 2)` into a
     String parameter.

   Step 2 starts only after eager-newt-412's facts work lands its last stage, because both rewrite
   infer's copying paths.
   **Residue admitted to step 2** (quiet-gull-780, 2026-10-01, via clever-newt-773): a generic
   variant literal with no expected type, such as `Cons { head, tail }` at
   `gunbc.spark.host_commitment`, is typed as its parent's *unapplied* declaration in
   `v1.compiler.infer` `infer_record_lit_structural` (`raw_resolved`). Instantiating it from its
   field values needs an applied-type constructor that carries identity, which is this model's
   `ParameterBinding` origin. It is left to step 2 rather than added as another spelling-keyed
   instantiation site.
3. **Then:** re-key the substitution map by the `TypeParameter` declaration
   `(owner, declaration, name)`, so two owners' parameters spelled alike cannot share a key. This
   is a separate PR, and the first production reader of #12690's field for this population.

## 7. Instrument

- **What it measures.** For each event in which `unify_generics` or `substitute_generics_apply`
  binds or substitutes a generic, whether the node carries a `TypeParameter` declaration, and its
  two nearest infer callers. It runs over the whole-corpus compile, `gunbc compile --source-root dag
  --source-root src/v2 --target dag --repository gunbc --measured-root-demands
  tools/whole_corpus_compile_measured_root_demands.json`.
- **Not yet an entry point.** The one measurement taken so far was a local, uncommitted probe on
  main `b793732902`. It is a one-off, so its numbers are not quoted in this note or on the row.
  Step 2's first task is to make it an instrument: a `gunbc test` label over the same compile,
  reporting by constructor. The step-2 decisions rest on its output, not on any figure written
  here.
