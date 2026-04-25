### The substrate: two coordinated shapes

The epistemic stacking section above tests any candidate compiler
substrate against a concrete question: *can it host the Node-tree
form of `dsl/std/algebra.dag`?* The thesis commits here to two
coordinated substrate shapes that pass the test. Neither is
invented in this document; both are inherited from earlier design
work. `MODELING.md` already gives the architecture explicitly:

```
Surface sugar:      service, fn, type, operation    (user intent)
Composition layer:  Node, children, edges           (how things connect)
Semantic kernel:    types, effects, contracts       (what flows through)
Foundation:         logical algebra                 (why it's sound)
```

The thesis's "type substrate" is the composition layer of that
architecture — `service`, `fn`, `type`, and `operation` are
surface sugar on top of it, and any compiler representation that
treats them as distinct substrate-level kinds (e.g.,
`DeclKind::Service`, `DeclKind::Operation`) is carrying the
parallel-representation disease the modeling discipline forbids.
See also `docs/design-lineage.md` §"Stage 3: gunbc v2" and
§"Stage 5: v3 spec," and the core-invariant memory
`feedback_compiler_is_dag_processor.md`. The thesis pins both
substrates down together because the epistemic chain rests on
them, and M1 substrate decisions need a single authoritative
statement to measure against.

**Types are Node trees with connectives.** Every type declaration
— a primitive, a record, a sum type, a function signature, a
parameterized generic, a service, an algebra, a correctness
dimension, a user-defined domain meta-model — is a Node with:

- A **connective**: `Atom | Conj | Disj | Arrow | Cardinality | Instantiation`
- **Children**: labeled (`Conj`, `Disj`) or positional
  (`Cardinality`) edges to other Nodes in the type substrate
- An optional **inhabits** edge: a reference to an algebra
  declaration whose children become derived accessors on this Node
  via the §"Epistemic stacking" mechanism

Connective meanings:

- **Atom.** Terminal at the user-input boundary — identifiers,
  literals, source spans from outside the closed world. Atoms
  carry a payload (the literal bits or identifier name). Every
  other connective decomposes; Atoms do not.
- **Conj.** Product / record with named children. Hosts type
  declarations (`Monoid<T> { op: fn(T, T) -> T; identity: T }`),
  services (`service shell.Exec { operation Run { ... } }`),
  operation definitions, user-defined domain meta-models,
  correctness dimension declarations, and **value construction**
  like `transport shell { argv: ["sh", "-lc", "{script}"] }` —
  which is a Conj whose fields are bound to specific values, with
  an optional `inhabits` edge pointing at the meta-type the Conj
  satisfies.
- **Disj.** Coproduct / sum with labeled alternatives. Hosts
  explicit sum types (`type Result<T, E> = Ok(T) | Err(E)`), exit
  patterns (`exit { 0 => Unit; nonzero => String }`), and the
  elimination forms consumed by the computation substrate's Branch
  behavior.
- **Arrow.** Function signature from input Nodes to an output
  Node. The `op: fn(T, T) -> T` field inside `Magma<T>` is an
  Arrow Node with three positional type children. An Arrow
  declaration's **body** is a typed edge to either a sub-DAG of
  L1 behaviors (for user functions) or a realization declaration
  in `dsl/extdeps/languages/` (for primitives). See §"Two
  groundings" below for how the user-defined vs external-
  realization distinction works; the concrete `ArrowBody` enum
  is defined in `src/v3/compiler/src/dag.rs` with four variants
  (`UserDefined`, `ExternalRealization`, `Pending`, `Unparsed`).
  In neither case is the body a name-based lookup — both are
  structural typed edges.
- **Cardinality.** Repetition over a child Node with a (possibly
  symbolic) count. `argv: ["sh", "-lc", "{script}"]` is a
  `Cardinality(3)` with three String-Atom children. `List<T>` is
  `Cardinality(n)` over `T`. Bounded iteration falls out of
  `List<T>` having a finite `n`. Unifies v2's Required / Optional
  (presence) with repetition (lists, arrays) in one connective.
- **Instantiation.** Specialization of a parameterized template
  declaration by binding its template parameters to concrete
  types. `Int64 = OrderedRing<Word64>` is an Instantiation whose
  `template` is the `OrderedRing` declaration and whose
  `arguments` list contains a single binding `T := Word64`.
  Substitution happens lazily at walk time via a substitution
  stack: walkers push the arguments on entering the Instantiation,
  look up template-parameter references inside the template body,
  and pop on exit. This matches C++ template instantiation
  exactly — a template is a parameterized declaration (`Monoid<T>`,
  analogous to `template<typename T> class Monoid`), an
  Instantiation is the specialized form with concrete arguments
  (`Monoid<Int>`, analogous to `Monoid<int>`).

**Instantiation is ONLY for type parameterization.** Earlier drafts
of this section conflated three different things under the
"Instance" label: type parameterization (`OrderedRing<Word64>`),
value construction (`transport shell { argv: [...] }`), and
inhabitance witness (`Int inhabits OrderedRing`). These are
three different structural relationships and the conflation was
a coproduct hiding in plain prose. They resolve as follows:

- **Type parameterization** → `Instantiation` connective, as
  described above. Template parameters (the `T` atoms in the
  parameterized declaration) bind to concrete types at the
  Instantiation site.
- **Value construction** → plain `Conj` with the field values as
  children, and an optional `inhabits` edge pointing at the
  meta-type the Conj should satisfy. `transport shell { argv:
  [...] }` is a Conj with one labeled child `argv` (a Cardinality
  literal) and an `inhabits` edge to the `transport` meta-type.
  No Instantiation involved — it's just record construction with
  a type tag.
- **Inhabitance witness** → for pure aliases like `Int =
  OrderedRing<Word64>`, Instantiation IS the inhabitance. Int's
  declaration has connective `Instantiation` directly; there is
  no separate "Int declaration with an inhabits edge pointing at
  an Instantiation Node." The collapse is honest because Int has
  no structure of its own beyond the inhabitance. The optional
  `Declaration.inhabits` slot is reserved for the rare case of a
  primary-structure declaration (Conj/Disj/Atom) that
  additionally inhabits an algebra — like a hypothetical Money
  record that also inhabits a CurrencyLattice.

**Computation is five L1 behaviors.** Every function body is a
sub-DAG of Nodes in a separate substrate with exactly five
primitive behaviors:

- **Value.** A literal carried into the graph on a Port with no
  inputs.
- **Transform.** Apply a `FunctionRef` (a cross-substrate edge to
  an Arrow declaration in the type substrate) to input Ports,
  producing an output Port.
- **Branch.** Select between sub-DAGs based on a Bit-valued
  (Disj-eliminated) input Port; outputs are unified into a single
  output Port. Exhaustiveness is structural: a Branch whose cases
  don't cover the input's Disj is a compile-time unbound-child
  error.
- **Loop.** Bounded iteration over a source Port whose type is a
  `Cardinality` Node in the type substrate, with an accumulator
  Port and a body sub-DAG. Termination is structural rather than
  behavioral because the source's cardinality is finite.
- **Bind.** Name a sub-DAG's output Port so downstream Transforms
  can reference it.

No sixth behavior. M0 validated these five under three reviewer
rounds and never hit a stop signal (see
`src/v3/M0_RETROSPECTIVE.md`); the five-behavior claim is the
project's strongest empirical substrate commitment and is not
revisited here.

**The two substrates compose via FunctionRef and Arrow-body.** A
Transform Node in the computation substrate holds a FunctionRef,
which is a typed edge to an Arrow declaration in the type
substrate. The Arrow declaration's `body` child is a Node
reference into the computation substrate (for user functions) or
into `extdeps/` (for primitives). So the two substrates meet at
exactly two points: the `Transform → Arrow` lookup on one side
and the `Arrow → body` lookup on the other. Inference resolves
types across this boundary; emission projects across it. There is
no third substrate.

**The vocabulary closes here.** Every `.dag` construct decomposes to
one of the six type connectives or one of the five behaviors, plus
references between them. `service`, `fn`, `type`, `operation`, `match`,
`fold`, `where`, `via`, `inhabits`, `transport` — the entire surface
syntax — is sugar over this composition layer. There is no other
syntactic vocabulary. There is no annotation slot, no opaque
intrinsic, no `unsafe` block, no escape into a foreign computation
model. **Anything that looks like a new construct is either a new
composition over existing connectives + behaviors, or it is substrate
extension** — and substrate extension is a C1-class stop signal (see
below). This is what makes lenses apply by construction: the lens is a
fold over six connectives and five behaviors, and there is no
program-shape outside that fold's domain. See §"Epistemic stacking" for
the corollary at the user-program level (case 1 vs case 2; no
opaque-intrinsic third case).

**The load-bearing test: host `dsl/std/algebra.dag`.** For any
candidate compiler-internal Declaration shape, the check is: *can
it host the Node-tree form of `std/algebra.dag` as-is?*
Specifically, the shape must carry four things:

1. **Parameterization over T.** `Magma<T>`, `Monoid<T>`, `Ring<T>`
   — type declarations whose children reference a free type
   parameter. The substrate must represent `T` as a child Node
   (probably an Atom with a parameter marker) that gets bound at
   Instantiation time, not as a string name with ad-hoc substitution.

2. **Structural children.** `Monoid<T>` as a Conj with children
   `{ op: Arrow(T, T, T); identity: T }`. The substrate must carry
   these children as typed Node edges, not as serialized metadata
   on a nominal declaration.

3. **The `fn(T, T) -> T` morphism shape.** An Arrow Node with
   three positional `T`-references. The substrate must represent
   function types as first-class Node shapes, not as separate
   Function declarations with inline signatures disconnected from
   the algebra they belong to.

4. **The inhabitance relation.** `Int inhabits OrderedRing<Word64>`
   — realized as an Instantiation node whose `template` is
   `OrderedRing` and whose `arguments` are `[T := Word64]`. Int's
   declaration IS this Instantiation; no separate edge needed for
   the pure-alias case. The substrate must carry this relation
   structurally so that `Int.add` resolves by walking `Int →
   Instantiation → OrderedRing (template) → add child →
   Arrow(T, T, T)` with `T := Word64` substituted from the
   arguments stack at each parameter reference — with no separate
   `add` declaration attached directly to Int.

If the substrate hosts `algebra.dag` through this walk, it
automatically hosts `shell.dag`, domain meta-models, user-defined
correctness dimensions, `service OrderAPI via rest::server`
declarations, and every other compositional declaration in the
project — because they are all Node-tree compositions in the
same substrate with the same six connectives. **If it cannot
host `algebra.dag`, the substrate is too narrow** and is carrying
parallel-representation debt that will force special-case handling
in every lens and every emission target. The test is not "what is
convenient to ship first"; it is "does the substrate preserve the
epistemic chain?"

**Substrate extension is a C1-class stop signal.** Adding a
seventh connective to the type substrate, or a sixth L1 behavior
to the computation substrate, is a substrate-level commitment
that requires the same discipline as any coproduct extension:
*all four dissolution patterns from §"Structural decompression"
must be attempted, and extension is only valid if all four fail
with structural arguments.* "A new kind of thing we need" is
never sufficient — the new thing must be shown to not decompose
into existing connectives or behaviors. This stop signal applies
to both substrates symmetrically.

**Future candidate (not committed): unified substrate.** A
stronger redesign that dissolves the five L1 behaviors into
patterns over the type substrate — treating Value as an Atom
carrying a literal, Transform as an Instantiation of an Arrow,
Branch as an Instantiation of Disj elimination, Loop as an
Instantiation of `fold`, Bind as a local-scoped declaration —
has been discussed as a candidate for concept-unification applied
to the substrate itself. **The thesis does NOT commit to this
collapse.** M0's validation of the five behaviors as substrate
primitives stands; revisiting them requires new failure pressure,
not just aesthetic or theoretical preference. Recorded here as a
candidate for future consideration only.

