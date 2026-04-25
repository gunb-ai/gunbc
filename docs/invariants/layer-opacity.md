### Layer opacity

The whole point of gunbc's compositional modeling is that **layers
compose such that below-boundary changes are invisible to
consumers**. This is the thesis's load-bearing claim — see
`THESIS.md` §"Compositional layering: below-boundary opacity by
construction" for the motivation. This invariant is its CI-gate
enforcement: **no consumer of the substrate may observe a
below-boundary identifier by name**. Consumers read across layer
boundaries, and below-boundary identifiers are unnameable to them
by construction.

**Below-boundary identifiers are forbidden to appear as hardcoded
string literals in compiler source code** (outside of diagnostic
display paths). This includes:

- User-facing type names from `dsl/std/` (`"Int"`, `"Bool"`,
  `"Float"`, `"String"`, `"List"`, `"OrderedRing"`, `"FreeMonoid"`,
  every algebra name, every primitive alias)
- User-facing field names from `dsl/std/` algebras (`"add"`,
  `"sub"`, `"mul"`, `"lt"`, `"eq"`, every algebra-field name)
- User-facing variant labels from `dsl/std/` sum types (`"True"`,
  `"False"`, every Disj variant name)
- Canonical operator symbols when used as semantic discriminators
  (`"+"`, `"-"`, `"*"`, `"=="`, `"<"`, ...)

Hardcoding any of these in compiler source code makes the
identifier observable to the compiler's decision logic, which
means renaming the identifier changes compiler behavior — the
consumer is reaching below the layer boundary to read a name.
That is the leak. Every use of such a hardcoded string is a layer
violation regardless of whether the code currently "works."

**The rename test.** The cheapest empirical check of layer
opacity is the rename test: pick a below-boundary identifier,
rename it everywhere in its declaring module (and any `import`
statements that explicitly reference it), recompile a test
consumer, and compare the generated output against the baseline.

- If the generated output is **byte-identical**: the layer was
  opaque. Layer opacity holds for this identifier.
- If the generated output **differs structurally** (different
  types, different wrappers, different dispatch): the compiler
  was reading the identifier by name somewhere. Layer opacity is
  violated. Find the violation and fix it.
- If compilation **fails**: there is a compiler-internal
  dependency on the identifier that has no opaque resolution
  path. Same verdict — find and fix.

**Historical example (2026-04-15).** The weather example in
`dsl/examples/weather/` was compiled to Rust with v2, then the
`Float` declaration chain in `dsl/std/float.dag` was edited three
ways to probe layering. Inserting an intermediate alias
(`Float → PreciseScalar → Float64`) produced byte-identical output.
Renaming internal layers below the boundary (`Float64 →
BinaryFloat64`) produced byte-identical output. But renaming the
boundary identifier itself (`Float → FloatingPoint`) produced
structurally different generated Rust: fields became
`Box<FloatingPoint>` instead of `f64`, `Temperature` lost its
`Copy` derive, every use site gained `Rc<...>` wrappers. The
leak: v2's inference and emission have a fast path for types whose
canonical name appears in `kernel_type_set` (a string-keyed map
in `dsl/std/types.dag`, mirrored into
`src/v2/stage0/src/std_types.rs`), and a slow path for everything
else. Renaming a primitive moves it from fast path to slow path.
The leak is tracked in v2 as "Part B pending" — when inference
resolves methods from type fields structurally, `kernel_type_set`
dissolves. Until then, the v2 compiler fails the rename test on
any of the eight names in that table.

v3 PR-B's `emit_rust.rs` reproduced the same leak at the emit
layer: `index.lookup("Int", "")`, `index.lookup("Bool", "")`,
`match label.as_str() { "True" => ..., "False" => ... }`. The
mechanism is identical — string-keyed dispatch against below-
boundary identifiers — even though the compiler had spent 14
review rounds removing this pattern elsewhere. The rename test
would have caught it at PR-B introduction time had it been an
enforced gate.

**The rule:** any compiler source file that contains a hardcoded
string literal matching a below-boundary identifier from `dsl/std/`
is a layer violation and must be reworked to dispatch by
`DeclarationId` instead. The substitution mechanism is the same
as every other bridge dissolution: replace the string key with a
typed edge, walk the substrate to resolve the edge at lookup
time, and let renaming propagate through DeclarationId identity
rather than through name matching.

**The fix when you've already written one:** extend the upstream
data structure (substrate field, language-spec schema, or fact
table) to carry the DeclarationId directly instead of a string.
Resolve the identifier at parse/lower time when the name is
known, carry the DeclarationId forward, and dispatch on ID. The
specific shapes this takes:

- **Language spec realizations:** instead of `target_name: String`,
  use `for: DeclarationId`. Walk the realization declaration to
  resolve its `for` field; the resolved `DeclarationId` is the
  identity key. This is the v3 class-5 gap #6 dissolution (extend
  `ValueBody::Structural` to support `LiteralBits::DeclarationRef`).
- **Canonical primitive rosters:** instead of
  `kernel_type_set: Map<String, Bool>`, use
  `kernel_types: List<Declaration>`. The list carries typed
  references to the primitive declarations; any consumer that
  needs "is this type a kernel primitive?" does DeclarationId
  containment rather than string lookup. This is v2's Part B
  dissolution of `kernel_type_set`.
- **Variant dispatch:** instead of `match label.as_str() { "True"
  => ..., "False" => ... }`, match on `BranchPattern::ResolvedVariant(DeclarationId)`
  and compare the variant's parent Disj against the scrutinee type
  structurally. Variant identity is a DeclarationId, not a string.

**Structural enforcement: the layer-opacity lens.**

Layer opacity is a structural query over a DAG and the natural
enforcement mechanism is a reader lens, not a grep gate. The
lens takes a `BoundarySpec` describing which declarations count
as below-boundary for the analysis, walks the DAG, and returns a
list of violations — every consumer site where a below-boundary
identifier is read by name instead of by typed reference. Lenses
are the thesis's intended extensibility point for invariant
enforcement (see `THESIS.md` §"Compositional layering" and the
lens library sketched in `docs/lens-library-design.md`).

```rust
// src/v3/compiler/src/lens_layer_opacity.rs
pub struct LayerOpacityLens;

pub struct BoundarySpec {
    /// Which declarations count as below-boundary for this analysis.
    /// Compiler-level application: all declarations from dsl/std/.
    /// User-level application: all declarations inside a named layer.
    pub below_boundary: Vec<DeclarationId>,
}

pub struct Violation {
    pub location: SourceSpan,
    pub identifier: String,
    pub origin: DeclarationId,
    pub consumer_kind: ConsumerKind,
}

pub enum ConsumerKind {
    TransformDispatch,
    VariantStringMatch,
    StringLiteral,
}

impl LayerOpacityLens {
    pub fn query(dag: &Dag, boundary: &BoundarySpec) -> Vec<Violation> {
        // Pure reader. Walk every Transform, Branch pattern, and
        // Value literal. For each, flag cases that observe a
        // below-boundary identifier by name.
    }
}
```

**Opt-in application.** The lens is opt-in via a `lens_config.dag`
file (or equivalent) that declares which lenses apply to which
source trees and what boundary each application uses:

```
module lens_config

data compiler_layer_opacity_spec: LensApplication = {
  lens: "layer_opacity"
  applies_to: ["src/v3/compiler/src/**/*.rs",
               "src/v3/compiler/src/**/*.dag"]
  boundary: "all declarations in dsl/std/**"
  severity: "error"
}

// A user opting into the same discipline for their own domain:
data my_domain_opacity_spec: LensApplication = {
  lens: "layer_opacity"
  applies_to: ["dsl/examples/my_app/app_code/**"]
  boundary: "all declarations in dsl/examples/my_app/rest/**"
  severity: "error"
}
```

CI runs every configured lens application on every PR that touches
its declared scope. A violation with `severity: error` blocks the
PR. The lens is a CI gate because it's a structural query, not
because it's a text search.

**Why the lens is strictly better than the grep gate we originally
proposed:**

- **Structural, not lexical.** The lens reads the substrate
  (Transform targets, pattern nodes, value literals), not source
  text. False positives from comments, documentation, or
  diagnostic strings don't exist.
- **Opt-in per application.** Multiple applications can coexist
  (compiler source, individual user projects, library boundaries)
  with different `BoundarySpec`s. A grep pattern would need
  project-specific pattern lists maintained by hand.
- **Structured output.** Each violation carries a location,
  identifier, origin, and consumer kind. Reviewers can triage by
  severity, consumer type, or origin layer. Grep output is a
  flat list.
- **Thesis-native.** The lens uses existing v3 infrastructure
  (reader lens over Dag, same shape as `lens_provenance`,
  `lens_cost`; `lens_depth` was a retired M0 experiment). No new machinery.
- **Generalizes to other invariants.** The lens framework
  extends to every testable invariant over a DAG — scaffold
  boundaries, fail-closed discipline, structural duplication,
  epistemic grounding termination, and more. See
  `docs/lens-library-design.md` for the full lens library plan.

**The rename test as regression safety net.** Even with the lens
in place, the rename test remains a valuable regression check:
pick a below-boundary identifier, rename it, recompile, diff the
output byte-by-byte. If the lens ever misses a violation, the
rename test catches it at the behavioral level. The two
mechanisms compose — the lens catches violations at introduction
time, the rename test catches any that sneak past the lens.

**The long-term structural target: Rust type-level enforcement.**
Even the lens is a reader over a substrate that currently allows
the violation to exist. The endgame is to make the violation
structurally impossible via Rust types: a `DisplayName` type
without `Eq`/`Ord`/`Hash` so dispatching on a below-boundary name
fails to compile at the Rust level. That's a bigger refactor
(touches every `decl.name` read) and is tracked as compiler-
architecture work rather than a per-file lens application. The
enforcement progression is: grep → test → lens → Rust types, each
step strengthening the invariant from "detected after the fact"
toward "impossible to write."

**Exception 1: diagnostic display paths.** The compiler's
diagnostic layer legitimately mentions user-facing names when
producing error messages — "unknown type `Int`" is a useful error
even though "Int" appears as a literal. Diagnostic display is an
exception because it's emitting text for the user, not making a
compiler decision. Test: if the string literal flows into a
diagnostic message, it's display. If it flows into a
`match`/`if`/`lookup` that determines compiler behavior, it's
dispatch and is forbidden.

**Exception 2: tracked scaffolds with active dissolution.** Some
scaffolds temporarily need hardcoded names during a transition.
The v2 `kernel_type_set` is the canonical example — it exists as
a documented scaffold waiting for Part B. Such scaffolds are
allowed only if (a) they have an active `INVARIANTS.md`
§"Scaffold boundaries" receipt with a numeric ratchet or explicit
dissolution trigger, (b) the trigger is documented inline in the
scaffold, and (c) the scaffold count is tracked and monotonically
decreasing across milestones. Tracked scaffolds do not exempt the
lens — they appear as violations in the lens output and require
an inline `// lens:layer_opacity_exception` comment linking to
the dissolution receipt. The lens cross-references its violations
against the receipt file; violations without receipts are errors,
violations with receipts are warnings that surface the scaffold
count for the monotonic-decrease ratchet.

**Exception 3: substrate-internal enum variants that are not
user-renameable.** Rust enum variants on compiler-internal types
(`Behavior::Bind`, `TransformTarget::Callable`, `ArrowBody::Pending`)
are not in `dsl/std/` and cannot be renamed from user code. String
or enum-pattern matches on these are NOT layer violations because
the names are compiler-internal, not below-boundary. Test: if the
name appears in a `.dag` source file anywhere in `dsl/`, it's
below-boundary and the `lens_layer_opacity` lens applies (with
`boundary = dsl/std/**` or equivalent); if it appears only in
`src/v3/compiler/src/*.rs` as an enum discriminant, it's
compiler-internal and exempt from the lens. (This exception will itself dissolve
when `project_node_to_std` moves Node and L1 behaviors into std/
as structural declarations — at that point the behavior names
become below-boundary and the lens applies to them too.)

**Relationship to other invariants:**

- **No bridges** forbids adapter functions between two
  representations of the same fact. Layer opacity is a specific
  class of bridge: one where the adapter is "match on a string
  from below-boundary data and produce a compiler-internal
  dispatch decision." Every string-dispatch leak is also a
  no-bridges violation; the two invariants catch the same
  failures from different angles.
- **Boundary sufficiency** says stage boundaries must carry
  enough structural data that downstream stages don't need
  name-proxy reads. Layer opacity is a specific diagnostic for
  boundary insufficiency: when a consumer reads a name, the
  upstream boundary didn't carry the structural fact the
  consumer needed.
- **Emission is translation, not decision-making** says the
  emitter must not make target-language decisions via
  hardcoded logic. Layer opacity generalizes this from emission
  to every consumer (lens, interpreter, future tooling); the
  emitter is the most common offender but not the only one.

**Operational commitment.** Every consumer of the substrate that
crosses a layer boundary must pass the layer-opacity lens for
every identifier below that boundary, AND must pass the rename
test as a regression check. The lens is applied to `src/v3/
compiler/src/` by default; user projects opt in via their own
`lens_config.dag`. New consumers are audited at introduction
time (the lens runs as a CI gate); existing consumers are
audited whenever their upstream layer gains new identifiers
(i.e., whenever `dsl/std/` grows — the lens's `BoundarySpec`
auto-updates by walking the canonical std/ declaration set). The
lens is the primary enforcement; the rename test is a regression
smoke test; the long-term goal (`DisplayName` type refactor) is
the structural target that makes violations impossible to
construct. Together they form the enforcement progression from
detection toward prevention.

