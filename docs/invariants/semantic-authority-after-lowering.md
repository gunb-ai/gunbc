### Semantic authority after lowering

A sibling invariant to §"Layer opacity," pinned as a direct
response to the PR #445 meta-review (2026-04-15). Where layer
opacity says "consumers cannot observe below-boundary names,"
this invariant says the stronger thing about WHICH representation
IS authoritative after the lowering boundary:

**After lowering, declaration identity (`DeclarationId`) is the
ONLY semantic authority.** No name recovery. No parallel
`TypeShape` kernel. No bridge adapters from names back to
declarations. No "parsed façade" that carries strings alongside
the declaration table. Every semantic decision downstream of
lowering reads `Dag.declarations` directly via `DeclarationId`
and makes its choices on the structure that table carries.

**Why this invariant is necessary separately from layer opacity.**
Layer opacity forbids a specific pattern (consumers reading
below-boundary identifiers by name). Semantic authority forbids
a broader pattern: the existence of ANY secondary representation
that the compiler consults as an authority after lowering. A
compiler can satisfy layer opacity locally — never grep a name —
while still maintaining a parallel `TypeShape` that's really a
shadow copy of `Dag.declarations` with different identity. The
layer-opacity lens would not catch that parallel authority
because no individual consumer is reading a below-boundary name;
the authority is just duplicated. Semantic authority closes that
gap.

**Historical motivation.** The PR #445 meta-review identified
the pattern by walking 16 review events across ~17 hours and
finding the SAME root question surviving every round:

> Is `Dag.declarations` the consumed authority, or only a parsed
> façade beside name recovery, bootstrap special cases, and a
> parallel type kernel?

The wording changed between rounds — "bootstrap fixtures plus
`declaration_to_type_shape`" → "parallel `TypeShape` authority"
→ "global-name resolver" → "opaque `ExternalRealization` arm"
→ "imports parsed then discarded" → "lower→infer boundary still
too weak" — but the disease didn't change. Each round named a
new surface of the same underlying violation: the lowering
boundary wasn't settled as a hard authority commitment, so every
downstream stage ended up consulting some secondary
representation to fill in gaps.

The review loop was doing real work (each round closed a real
bug) but accumulating debt faster than it dissolved it. The
meta-review's verdict: **PAUSE_AND_REGROUP and graduate the
recurring pattern into an invariant instead of patching its
next instance.** This invariant is that graduation.

**The rule:** after `lower_bodies_phase` + `resolve_pending_
identifiers` complete, every downstream stage (`infer`, `lens_*`,
`emit_*`, the interpreter, any future analysis tool) reads facts
about types, functions, variants, operators, and realizations
**only** from `Dag.declarations` via `DeclarationId` lookups and
structural walks. Specifically forbidden:

- **Parallel type representations.** A `TypeShape` struct that
  carries any information that isn't reachable from
  `Dag.declarations` via its contained `DeclarationId`. The
  `declaration_to_type_shape` adapter that PR #445 introduced
  and then removed is the canonical counterexample.
- **Global name recovery.** Any function that takes a `String`
  and returns a `DeclarationId` as a post-lowering recovery
  path. Name-based lookup is a parse-time or bootstrap-time
  activity; after lowering, names exist only in diagnostic
  display paths.
- **Bootstrap-only special cases in post-lowering code.** Any
  `if current_range < bootstrap_range { special_behavior }`
  branch in a downstream stage. Bootstrap-range declarations
  may carry scaffolded state (per §"Layer opacity" exception 2),
  but downstream stages do not need to distinguish them — they
  consume the same `Dag.declarations` table and the scaffolded
  state is either handled uniformly or fails closed.
- **"Parsed façade" patterns.** Carrying a `SurfaceModule` or
  pre-lowering representation past the lowering boundary so
  downstream stages can fall back to re-reading the parse
  tree. The lowering boundary is the transition point where
  the parse tree's information must be fully absorbed into
  `Dag.declarations`.
- **Secondary authorities for realization.** An
  `ExternalRealization` target whose body is an opaque Rust
  handle or a compiler-internal lookup table instead of a
  declaration-table lookup. Realizations are declarations; they
  participate in `Dag.declarations` like any other declaration.

**The test.** This invariant has two mechanical checks, both
lens-shaped:

1. **Post-lowering name-reference audit.** A lens that walks
   every Rust function in `src/v3/compiler/src/` (or equivalent
   source tree), identifies the subset that runs after
   `lower_bodies_phase`, and flags any that read a `String`
   field from a declaration as input to a dispatch decision.
   The subset is determined structurally: functions called from
   `infer`, `lens_*`, `emit_*`, `interp`, etc. Violations are
   sites where post-lowering code reads a name, not a
   DeclarationId. This is a refinement of `lens_layer_opacity`
   scoped to post-lowering consumers.

2. **Parallel authority audit.** A lens that walks every struct
   definition in `dag.rs` and flags any that contains fields
   with names like `name: String` or `kind: String` that appear
   alongside a `DeclarationId`. The heuristic catches the
   declaration-shadow-copy pattern. Violations are sites where
   the compiler is carrying a name through the post-lowering
   data flow instead of relying on `DeclarationId` alone.

The two lenses together form the enforcement mechanism. Both are
in scope for the initial lens library (see
`docs/lens-library-design.md`) as post-MVP additions — the three
lenses currently specified (unused_parameters,
structural_duplicates, layer_opacity) are the starting point;
semantic authority lenses extend the pattern once the framework
is in place.

**The rule for consumers writing new downstream code:**

> If you are writing code that runs after `lower_bodies_phase`
> and you find yourself wanting to read a `String` field from a
> declaration, STOP. Either (a) the fact you need is available
> as a typed edge from the declaration you already have the
> `DeclarationId` for, or (b) the declaration table is missing
> a fact that should be there and you need to add the typed
> edge upstream before writing the consumer. There is no third
> option.

**The fix when you find an existing violation.** Do not patch
the consumer. Do not add a name-recovery helper. Do not add a
parallel table for the one fact you need. Instead:

1. Identify the fact the consumer wants as a declaration field.
2. Extend the declaration table to carry the fact as a typed
   edge (usually a `DeclarationId` field or a `Vec<DeclarationId>`
   child list).
3. Update `lower_bodies_phase` and `resolve_pending_identifiers`
   to populate the field.
4. Rewrite the consumer to read the typed edge.
5. The old name-reading path is deleted, not gated, in the same
   PR.

This is the standard bridge-dissolution pattern applied
specifically to the lowering boundary. Same rules as §"No
bridges," with the extra specificity that the bridge is always
between `String` names and `DeclarationId` identity.

**Historical examples of the violation:**

- `declaration_to_type_shape` adapter in PR #445's early rounds
  (dissolved in R5).
- Bootstrap-range vs user-range distinction as a post-lowering
  dispatch criterion (still under review in R13/R14).
- `kernel_type_set` as a post-lowering authority for
  "is-primitive" checks (v2, tracked as Part B).
- PR-B's `emit_rust.rs` `lookup("Int", "")` pattern (the Rust-
  level version of the same leak at the emit boundary, flagged
  in the 2026-04-15 layer-opacity review).

Each of these is the same pattern in a different layer. The
meta-review's insight: **the invariant would have prevented each
of these at introduction time** if it had existed from M0. The
review loop kept finding new instances because no general rule
named the class.

**Relationship to other invariants:**

- **Layer opacity** is a local consequence. Every semantic-
  authority violation is also a layer-opacity violation at the
  post-lowering boundary, but not every layer-opacity violation
  is a semantic-authority violation (a user project's layer
  opacity doesn't involve `Dag.declarations` directly).
- **Boundary sufficiency** says each stage's output must carry
  enough structural data for downstream consumers. Semantic
  authority is a specific application: the lowering boundary's
  output must carry every fact downstream consumers will need,
  referenced by `DeclarationId`, with no secondary authorities
  allowed as fallback.
- **No bridges** forbids adapters between two representations.
  Semantic authority forbids a specific class of adapter:
  String → DeclarationId at or after the lowering boundary.
- **Scaffold boundaries** (R16) requires scaffold variants to
  have boundary gates. Semantic authority extends the idea:
  not just scaffold variants, but ALL post-lowering reads of
  declaration data must use typed identity.

**Dissolution target.** When the semantic-authority lenses are
written and applied to `src/v3/compiler/src/`, violations
become visible in the CI output. Each violation names a
specific consumer that needs the bridge-dissolution pattern
above applied. Over time, the violation count should trend
monotonically downward; new violations are blocked at PR time;
existing ones are fixed as each downstream stage is touched.
The endgame is zero violations, at which point the invariant is
enforced by construction (there's no code that could violate
it because every declaration-reading site reads by
`DeclarationId`).

