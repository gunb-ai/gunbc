### Compositional layering: below-boundary opacity by construction

The two-substrate design is load-bearing only if it gives the
project a property the substrate by itself can't provide: **layers
compose such that below-boundary changes are invisible to
consumers**. That property is the reason to generate code rather
than write it by hand, and it is the reason the substrate is worth
the discipline it demands. Without it, the substrate is just a
different storage layout; with it, the substrate is a composition-
preserving guarantee. This section pins the claim.

**The networking analogy is exact.** The OSI stack is the proof-
of-concept: HTTP does not know or care what transport it is running
over. You can swap TCP for QUIC, add TLS between the transport and
application layers, or change the link layer from Ethernet to WiFi
to fiber to cellular, and application code — literal HTTP requests
from a literal browser — is structurally unaware. HTTP has **no
API** for asking "which transport are we running over right now?"
Below-boundary details are not merely hidden, they are *unnameable*
to consumers. That unnameability is what makes the internet scale
N+M instead of N×M: each layer can evolve independently because
no layer above it can depend on below-boundary internals.

**gunbc's compositional modeling claims the same property for
general software composition.** A `rest / http / service`
composition in `.dag` should have the same layering guarantees as
`HTTP / TCP / IP`. You can insert a retry layer between `rest` and
`http`. You can swap `http` for a different transport. You can
replace `service`'s implementation entirely. Application code
**cannot observe the change** because the compiler never exposes
below-boundary identifiers to it. The user edits one layer; every
other layer keeps working without modification. This is what the
thesis means when it says "the cost of change is 1."

**Generated code is the mechanism.** Hand-written code embeds
composition assumptions at every call site: function names,
parameter shapes, ownership conventions, error channels, import
paths. Every line is a reference to specific names in specific
layers. A layer swap requires editing every consumer because every
consumer spelled out the composition manually. Generated code
inverts this: the compiler walks the structural graph and emits
target-language code from the walk. The consumer's `.dag` source
declares *intent*; the compiler resolves *references* by walking
the substrate; the emitted code is the result of that walk. When
an intermediate layer changes internally, the walk produces the
same result because the walk goes through structural edges, not
through memorized names.

**The test: rename any below-boundary identifier.** The cleanest
empirical check of the layering claim is to pick an identifier
that's internal to some layer (below its public boundary), rename
it, and see whether any consumer's generated output changes. If
consumers produce identical output, the layer was opaque. If any
consumer's output differs — or worse, if consumers fail to compile
— the layering leaked and the compiler was reaching below the
boundary to read the identifier by name. This is the **rename
test**, and it is the cheapest invariant check the compiler can
run.

**Empirical validation (2026-04-15).** The weather example in
`dsl/examples/weather/` was compiled to Rust with v2, then the
`Float` declaration chain in `dsl/std/float.dag` was edited three
ways. For each variant, `diff -r` was run against the baseline:

1. **Insert an intermediate layer.** `Float = PreciseScalar =
   Float64 = Field<Word64>` instead of `Float = Float64 =
   Field<Word64>`. Weather.dag unchanged. Generated Rust
   **byte-identical** to the baseline. The compiler walked the
   algebraic chain and did not notice the new intermediate alias.
   ✅ Layering holds.
2. **Rename internal layers below the boundary.** `Float64 →
   BinaryFloat64` and `Float32 → BinaryFloat32`, keeping `Float`
   as the boundary alias. Weather.dag unchanged. Generated Rust
   **byte-identical** to the baseline. The compiler walked
   through the renamed internal names and still produced `f64`.
   ✅ Layering holds.
3. **Rename the boundary identifier itself.** `Float → FloatingPoint`
   in std/float.dag, weather.dag updated to use the new name via
   an explicit `import`. The consumer update is expected: consumers
   depend on boundary names. The question is whether anything ELSE
   breaks. Answer: **yes**. Generated Rust is structurally
   different — fields change from `celsius: f64` to
   `celsius: Box<FloatingPoint>`, `Temperature` loses its `Copy`
   derive, every use site gains `Rc<Temperature>` wrappers, and
   function signatures return `FloatingPoint` instead of `f64`.
   The leak: v2's inference and emission have a **fast path** for
   types whose canonical name appears in `kernel_type_set` (a
   string-keyed map in `dsl/std/types.dag`), and a slow path for
   everything else. Renaming a primitive moves it from fast path
   to slow path without warning. ⚠ Layering leaks.

**The leak class.** What the weather experiment found in v2 is a
general pattern: **canonical-primitive-name rosters**. These are
tables keyed on `String` that map "known primitive names" to
behavioral decisions — which types get efficient emission, which
variants participate in fast-path dispatch, which declarations
count as kernel types. They appear in every compiler that stages
up from minimal parsing: the simplest way to recognize a primitive
is by its name, and adding a name table is cheap. The table
**dissolves** below-boundary opacity because it makes the primitive
name observable to the compiler's decision logic. Rename the
primitive and the dispatch changes even though the primitive's
algebraic structure is identical.

The fix is not to remove the distinction between fast and slow
paths — primitives really do map to different target representations,
and that mapping is the point of the language spec. The fix is to
**key the distinction on structural identity (a `DeclarationId`)
rather than on a string name**. A language spec declares "this
realization targets the declaration at `DeclarationId(X)`" via a
typed edge; the compiler walks the edge at resolution time; renaming
the declaration doesn't change its identity, and the walk still
finds the right realization. `kernel_type_set`'s replacement is the
dissolution of the canonical-name roster in favor of typed edges
from realizations to the declarations they realize.

**This is the same pattern the review cycle kept finding.**
Rounds 5–7 of M1(2.6) spent most of the milestone eliminating
name-keyed dispatch at the inference layer. M1(2.7)'s big fix
closed fourteen gaps, most of them instances of the same pattern.
PR-B of M1(3) shipped with a fresh version of the same pattern at
the emit/language-spec layer (`lookup("Int", "")`,
`lookup("Bool", "")`, `match variant.as_str() { "True" => ... }`).
Different layer, same leak. The root cause in every case is that
the primitive's identity was carried as a string, and the compiler
made a behavioral decision by string comparison. Layer opacity is
the invariant that would have caught every one of those cases at
introduction time if it had been enforced structurally from M0.

**Relationship to other thesis sections.**

- **Epistemic stacking** says concepts ground in primitives via
  a structural DAG. Layer opacity is the *runtime consequence* of
  epistemic stacking: if consumers walked the stack structurally,
  renaming any concept in the stack should not change the walk's
  result for consumers above it. A leak in layer opacity is
  always a leak in epistemic stacking — somewhere, a consumer
  short-circuited the walk by reading a name instead of following
  the structural edges.
- **The substrate: two coordinated shapes** provides the storage
  that makes the walk possible. The two substrates carry typed
  edges between declarations and between computation nodes, and
  those edges are what the walk traverses. Without typed edges,
  the walk collapses into name lookups.
- **Two groundings** (following this section) extends the layering
  claim to the target world: static grounding means concepts
  decompose to primitives structurally inside gunbc, realization
  grounding means primitives map to target representations
  structurally in the language spec. Both groundings must respect
  layer opacity — the static decomposition must not leak internal
  identifiers up to consumers, and the realization must not leak
  target-internal names (e.g., Rust's `i64`) into compiler
  decision logic.
- **Omni-emission** (far below) is what layer opacity enables at
  scale: one intent graph, many target artifacts, with no target-
  specific logic in compiler code. If any target emission has a
  hardcoded list of primitive names it knows about, adding a new
  target requires Rust edits; if every target's knowledge lives in
  a language spec referenced by typed edges, adding a new target
  is a single spec-file edit.

**Enforcement: lenses, not grep.** Layer opacity is a structural
query over a DAG — "walk every consumer site, flag any that reads
a below-boundary identifier by name" — and that makes it the
paradigmatic case for a **lens**. A lens is a pure reader over
the substrate that answers a structural question, and lenses are
the thesis's intended extensibility point for invariant
enforcement. The layer-opacity lens takes a `BoundarySpec` (which
declarations count as below-boundary for this application) and
returns a list of violations. Applied to compiler source with
boundary `= dsl/std/**`, it returns every place in the compiler
where a user-facing std/ identifier is read by name. Applied to
a user project with boundary `= project's internal layer`, it
returns violations in the user's own composition. **Same
mechanism, parameterized application.**

Lenses are the structural answer to invariant enforcement, and
layer opacity is the first invariant the project should use them
for. Grep gates were the earlier proposal in this document; they
are superseded by [`docs/invariants/layer-opacity.md`](../invariants/layer-opacity.md) pointing at a
lens with opt-in application. The rename test remains as a
regression safety net, but the primary enforcement is the lens
output.

**Lenses as the general enforcement mechanism.** The layer-opacity
case generalizes. Most testable invariants over a DAG can be
stated as "walk the DAG, flag sites that violate property X" —
which is the shape of a lens. Compositional layering, scaffold
boundaries, fail-closed discipline, structural duplication,
epistemic grounding termination, dataflow reachability — all are
lens-shaped. The thesis's early framing of lenses as "cost,
ownership, complexity" undersold them. Lenses are **the
invariant-enforcement primitive**: adding a new invariant becomes
adding a new lens, applied to whichever files or folders should
be subject to it, with no substrate changes required. The review
cycle's historical cost came from re-implementing per-round
invariant checks by hand; the lens library is the structural
answer that replaces the hand-work with a standing query.

**Operational commitment.** Every consumer of the substrate —
lens, emitter, interpreter, future analysis tools — must pass
the layer-opacity lens for every identifier below its consumed
boundary. The lens is cheap to run (walk the DAG, return a list
of violations), general (same lens handles compiler source, user
projects, or any DAG), and catches the leak class that has
historically accounted for the largest share of review-round
findings. The rename test is the regression smoke test; the
layer-opacity lens is the primary enforcement; substrate-level
structural constraints (e.g., a `DisplayName` type without `Eq`)
are the long-term target that makes the lens's findings impossible
to construct in the first place.

Layer opacity is the property the substrate exists to provide.
Without it, the substrate is structurally interesting but
operationally indistinguishable from any other storage layout. With
it, the substrate delivers what the thesis promises: independent
evolution of layers, one-file edits for new targets, and
compositional modeling that matches the OSI stack's empirical
scalability proof.

