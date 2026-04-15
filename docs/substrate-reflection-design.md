# Substrate Reflection Design

> Part of: [THESIS.md](../THESIS.md), [INVARIANTS.md](../INVARIANTS.md),
> [lens-library-design.md](lens-library-design.md)
>
> **Purpose:** make v3's substrate self-describing. Move substrate
> types (`Dag`, `Node`, `Behavior`, `Declaration`, …) into `dsl/std/`
> as declarations in the substrate's own language, expose query
> primitives that let a `.dag` program walk compiled substrate
> data, and migrate the existing lenses from Rust bootstrap form to
> their canonical `.dag` form. After this work, adding a new lens
> is a new `.dag` file — no Rust, no compiler edits, no
> `kernel_lens_set` forming.

---

## §1. Motivation: the untested thesis claim

The thesis claims the substrate is **self-describing**: every
concept the compiler reasons about lives in the DAG, reachable
by walking typed edges. That claim has been tested on every
frontier except one — **the substrate's own shape**. Today, the
types that describe the substrate (`Behavior`, `NodeId`, `Dag`,
etc.) live only in Rust source. They're not DAG declarations.
Nothing in `.dag` can observe them. This is the last untested
load-bearing claim in the experiments log.

**The concrete failure mode this work prevents.** Lenses are the
thesis's invariant-enforcement primitive. The four lenses that
ship today (`lens_provenance`, `lens_depth`, `lens_cost`,
`lens_unused_parameters`) are all Rust modules. Every new lens
added as a Rust module grows a registry of known lens names, a
dispatch pattern keyed on those names, and shared Rust-lens
infrastructure. That is the same failure class as v2's
`kernel_type_set` — a name-roster leak at a different layer. The
project calls this the **`kernel_lens_set`** failure: by the time
you have five or six Rust lenses, the compiler has a hardcoded
list of known lens names and a special handler per lens, and
migrating them to `.dag` is a meta-refactor. The rule from
[`lens-library-design.md`](lens-library-design.md) §1.5 — "stop
and build the reflection primitive before the scaffold ossifies"
— is this work's charter.

**Composability was the user-facing framing.** The second reason
this matters: a `.dag` lens can be analyzed by another `.dag`
lens (including itself). A Rust lens cannot — analyzing it would
require a meta-language separate from the substrate. Once lenses
live in the substrate, `lens_unused_parameters` can run against
its own source, `lens_layer_opacity` can check that other lenses
don't dispatch on names, `lens_cost` can estimate the cost of
running a lens. This is the "DAG that looks inward" the user has
been pointing at — and the composition property lenses need if
they're ever going to be used to analyze each other.

**The third reason: cost of change.** The thesis's load-bearing
metric is "one file edit per new feature." For lenses, the
current metric is: new Rust file + build graph edit + test module
registration + documentation update. That's a four-file minimum.
In canonical form, adding a new lens is one `.dag` file loaded
by the manifest. One file. That's the thesis target.

---

## §2. Scope of this document

**In scope:**

1. Moving the substrate's structural type layer into
   `dsl/std/substrate.dag` (a regular `std/` file, not a
   subdirectory) as ordinary `.dag` declarations — `Dag`,
   `Declaration`, `Behavior`, `TypeConnective`, `ArrowBody`,
   `Field`, `AtomPayload`, `CardinalityBound`, `TemplateArgument`.
   **Not declared:** the atomic identity handles (`NodeId`,
   `PortId`, `DeclarationId`, `SourceSpan`) stay as Rust-seed
   primitives (§3.0). Proves via inhabitance that the Rust
   substrate structs satisfy the `.dag` declarations.
2. Declaring the **query primitive set** — the minimal `.dag`
   functions that walk substrate data (e.g., `declarations(d: Dag)
   -> List<Declaration>`, `params(b: Bind) -> List<PortId>`).
   These are Arrows with `ArrowBody::ExternalRealization` bodies
   that resolve at lower time to Rust implementations in
   `src/v3/spec/rust.dag`.
3. Extending `emit_rust` so a compiled `.dag` lens that calls a
   query primitive produces Rust code that calls the corresponding
   Rust method on a `Dag` value.
4. Migrating `lens_unused_parameters` from its Rust form to its
   `.dag` form as the proof point. Same algorithm, host language
   changes. The Rust lens is deleted.
5. Landing the **self-analysis test**: `lens_unused_parameters`
   analyzes its own `.dag` source and reports zero unused
   parameters.
6. Landing the **no-kernel-lens-set** structural check: a lens
   (or CI gate) that verifies the compiler does not dispatch on
   hardcoded lens names.
7. Thesis section: **§"Self-inspection"** — the substrate is its
   own subject.
8. `INVARIANTS.md` addition: **"Lenses are substrate declarations"**
   — after this lands, new lenses must be `.dag` programs over the
   reflection surface.
9. Rewriting `lens-library-design.md` §1.5 and §2.3 to reflect
   the canonical form as the current state, not aspiration.

**Out of scope (deferred):**

- Migrating `lens_provenance`, `lens_depth`, `lens_cost` to
  `.dag`. The migration template is proven by
  `lens_unused_parameters`; the other three are mechanical ports
  left as a follow-up.
- Building `lens_complexity` as the v2/v3 comparison vehicle.
  That depends on this work landing first but is a separate
  substrate audit + new lens (tracked as follow-up).
- `lens_structural_duplicates` and `lens_layer_opacity`. Both
  still valuable future lenses; both land after the reflection
  substrate as straightforward `.dag` programs.
- Runtime interpretation of `.dag` lenses. This design
  compiles lenses to Rust via the existing `emit_rust` pipeline
  and runs them as compiled Rust. A true runtime interpreter is
  not a prerequisite.
- The lens manifest format and `v3-compiler lens run` CLI
  subcommand. Both still deferred.
- Moving the computation substrate (L1 behaviors) itself into
  `.dag`. The type layer is enough to prove reflection works;
  behaviors can follow.

**Non-goals (explicitly out of the thesis target for this work):**

- Parser or grammar extensions beyond what the existing
  expression-body `fn` form already supports. If the lens needs
  a parser feature that doesn't exist, the feature is scoped as a
  prerequisite PR, not bundled here.
- Any substrate variant additions. This work is purely additive
  to `dsl/std/`; no new `TypeConnective` variants, no new
  `Behavior` variants, no new `AtomPayload` variants.
- Any change to the `lens_unused_parameters` algorithm. Same
  walk, same output, different host language.

---

## §3. The reflection surface

The reflection surface is the minimum set of `.dag` declarations
and query primitives a lens needs to express its walk over the
substrate. Everything is an ordinary declaration in `dsl/std/` —
there is no separate "substrate subdirectory" or meta layer.
The substrate is self-describing by virtue of declaring its own
shape as data **inside** itself.

### §3.0 Seed minimality as an invariant

The bootstrap seed is deliberately narrow:

- **The parser.** Written in Rust. Produces `SurfaceItem` / `SurfaceExpr`
  trees from `.dag` source text.
- **Atomic-identity primitives.** `NodeId`, `PortId`, `DeclarationId`,
  `SourceSpan` — opaque Rust atoms that `.dag` programs compare
  for equality and pass around but never structurally inspect.
  Each has a realization entry in `src/v3/spec/rust.dag` the same
  way `Int` does.
- **The existing arithmetic primitives.** `Int`, `Bool`, `String`
  — already shipped, already realized.
- **The inhabitance / resolve sweep.** `resolve_pending_identifiers`
  + the forward-reference resolution v3 already has working for
  `std/algebra.dag → std/types.dag::Bool`.

**Everything else is declared in `.dag`.** `Dag`, `Declaration`,
`TypeConnective`, `Behavior`, `Field`, `ArrowBody`, `AtomPayload`,
`CardinalityBound`, `TemplateArgument`, and every other structural
substrate type lives as an ordinary `Conj` / `Disj` declaration
in `dsl/std/`. The Rust structs in `src/v3/compiler/src/dag.rs`
are **realizations** of these declarations — kept consistent by
an inhabitance check, not by parallel representation.

**The invariant this PR codifies** (new `INVARIANTS.md` entry —
see §7.2): the seed MUST NOT grow. Every new substrate concept
that could be declared in `.dag` must be declared there, not
added as a Rust-only primitive. New Rust primitive additions
are blocked unless (a) the concept is truly atomic (can't be
decomposed further) or (b) it's an ID-like handle that programs
pass around without inspecting. The seed is five atomic handles
plus the parser plus the sweep — and that's the ceiling.

**Why the ceiling matters.** The thesis's self-hosting claim
("the project is a meta-compiler — the substrate can describe
any computational system including itself") only holds if the
substrate actually hosts itself. Every Rust primitive the seed
contains is a fact `.dag` cannot analyze. A ten-primitive seed
leaves ten blind spots; a five-primitive seed leaves five. The
direction is toward zero, and the seed ratchet ensures we don't
drift the other way under pressure.

**How this answers the bootstrap question.** The circular
dependency `Dag → Declaration → TypeConnective → Declaration` is
the same shape as `OrderedRing<T> → Ring<T> → Monoid<T>`. v3
resolves the latter today via `resolve_pending_identifiers`.
Applying the same mechanism to substrate-type declarations is
zero new bootstrap machinery — it's one more file in the
`include_str!` list.

### §3.1 Substrate types as `.dag` declarations

New file: `dsl/std/substrate.dag` — sibling to `std/algebra.dag`,
`std/types.dag`, `std/integer.dag`, etc. No subdirectory. Loaded
by the bootstrap alongside the other `std/` files, in dependency
order after `std/types.dag` (which it depends on for `Bool`,
`String`, etc.).

```
module std.substrate

// Substrate types — declared here, realized by the Rust structs
// in src/v3/compiler/src/dag.rs, kept consistent by the inhabitance
// check that runs at bootstrap time.

// Type connective — six-way sum matching Rust's TypeConnective.
type TypeConnective
  = Atom(AtomPayload)
  | Conj { fields: List<Field> }
  | Disj { variants: List<Declaration> }
  | Arrow { inputs: List<DeclarationId>, output: DeclarationId, body: ArrowBody }
  | Cardinality(CardinalityBound, DeclarationId)
  | Instantiation { template: DeclarationId, args: List<TemplateArgument> }

type ArrowBody
  = UserDefined(NodeId)
  | ExternalRealization(DeclarationId)
  | Pending
  | Unparsed(SourceSpan)

// L1 behavior — five-way sum matching Rust's Behavior.
type Behavior
  = Value(ValueNode)
  | Transform(TransformNode)
  | Branch(BranchNode)
  | Loop(LoopNode)
  | Bind(BindNode)

// Declaration — the unit of the type substrate.
type Declaration {
  id: DeclarationId
  name: String
  connective: TypeConnective
  meta_tag: DeclarationId?
  inhabits: List<DeclarationId>
  type_params: List<DeclarationId>
  span: SourceSpan
}

// The Dag itself — a declaration like any other. Its realization
// is the Rust struct src/v3/compiler/src/dag.rs::Dag.
type Dag {
  declarations: List<Declaration>
  nodes: List<Behavior>
}
```

**`NodeId`, `PortId`, `DeclarationId`, `SourceSpan` are NOT
re-declared here.** They are seed primitives (§3.0) — atomic
handles that the substrate passes around but never inspects.
Their realization entries in `src/v3/spec/rust.dag` bind them to
their Rust counterparts (newtype-wrapped integers and a span
struct), but there is no `type NodeId` in `.dag`. If a lens
needs to compare two `NodeId` values it uses equality; if it
needs to turn one into a `Behavior` it uses a query primitive
(`node_by_id(dag, id)`).

**Inhabitance check.** A post-bootstrap pass walks every
declaration in `std.substrate` and asserts the corresponding
Rust type satisfies the declared shape. For `type Dag`, the check
walks `Dag`'s Rust fields and asserts they match the declared
fields by name and type. Drift between the two representations
becomes a fail-closed diagnostic at bootstrap time. This is the
same inhabitance mechanism `src/v3/spec/rust.dag` uses today to
bind `.dag Int → Rust i64` — the only new thing is applying it
to the compiler's own struct types.

### §3.2 Query primitives

New file: `dsl/std/substrate_query.dag`. Declares the minimal
walking primitives every lens needs. Each is an Arrow with
`ArrowBody::ExternalRealization` — the body is declared
structurally but realized by a Rust method call at emit time.

```
module std.substrate_query

import std.substrate { Dag, Declaration, Behavior, Node, Port, Bind, NodeId, PortId, DeclarationId }

// Whole-DAG readers.
fn declarations(d: Dag) -> List<Declaration>
fn nodes(d: Dag) -> List<Behavior>
fn declaration_by_id(d: Dag, id: DeclarationId) -> Declaration
fn node_by_id(d: Dag, id: NodeId) -> Behavior

// Per-declaration readers.
fn connective(decl: Declaration) -> TypeConnective
fn declaration_name(decl: Declaration) -> String

// Per-behavior readers.
fn behavior_id(b: Behavior) -> NodeId
fn is_bind(b: Behavior) -> Bool
fn is_transform(b: Behavior) -> Bool
// … one per Behavior variant

// Bind readers.
fn bind_params(b: Bind) -> List<PortId>
fn bind_value(b: Bind) -> PortId

// Port readers.
fn port_producer(d: Dag, p: PortId) -> NodeId?

// Transform readers.
fn transform_inputs(t: Transform) -> List<PortId>

// Branch readers.
fn branch_input(br: Branch) -> PortId
fn branch_paths(br: Branch) -> List<Path>
fn path_output(p: Path) -> PortId

// Loop readers.
fn loop_source(l: Loop) -> PortId
fn loop_init(l: Loop) -> PortId
fn loop_body(l: Loop) -> NodeId

// Primary-result readers — the single substrate authority for
// "which port carries the semantic result of a behavior."
// Consumers that walk sub-DAGs (cost lens, unused_parameters,
// complexity, any future lens) MUST go through this query rather
// than reimplementing the match-on-variant locally. See §3.2.1
// below for the reason this primitive is load-bearing.
fn behavior_output_port(b: Behavior) -> PortId

// (Complete list matches the walking primitives every lens needs.
// New primitives go through a design review — the set is
// deliberately small.)
```

### §3.2.1 `behavior_output_port` as a canonical substrate query

**Why this primitive is called out specifically.** During the
PR #451 review cycle, a ChatGPT review of a `lens_unused_parameters`
iteration flagged that its private `behavior_output_port(b:
&Behavior) -> PortId` helper — a match over all five L1 behavior
variants returning "the primary result port" — was the beginning
of a canonical substrate query, not a lens-local convenience.
The review's point: if a second lens, interpreter helper, or
emitter walker ever needs the same mapping and reimplements it,
the project immediately has competing authorities for one
substrate fact. That is a `lens_authority_drift` failure mode
in the same family as `kernel_type_set` and `kernel_lens_set` —
a single fact duplicated across consumers because the substrate
didn't host it in the first place.

**The fix this design doc commits to.** `behavior_output_port`
lands in `std.substrate_query` as a first-class query primitive
in this PR's reflection work. Every lens (and every future
consumer that walks sub-DAGs) reads the primary-result port
through this one query. The rule is: if you find yourself
writing `match behavior { Value(_) => ..., Transform(_) =>
..., Branch(_) => ..., Loop(_) => ..., Bind(_) => ... }` to
answer "which port is the result," stop — call
`behavior_output_port` instead. This makes the substrate the
single authority per §"Semantic authority after lowering" in
`INVARIANTS.md`.

**Structural enforcement.** A grep-style layer-opacity check
(eventually a lens, for now a CI gate) verifies no consumer
source file outside `substrate_query.rs` contains a five-arm
match on `Behavior` variants that extracts a PortId. The single
exception is `behavior_output_port`'s own implementation. This
is the same shape as the rename test for `kernel_type_set` —
one place holds the authority, everyone else reads through it.

**Why this is a reflection-design concern, not a patch.** The
review flagged the pattern on a PR that predates the reflection
primitive. Without reflection, fixing it means adding yet another
public method to the Rust `Dag`/`Behavior` API, which grows the
Rust-side scaffold surface the §1 motivation argues against.
With reflection, the primitive lands in `.dag` — once — and
every lens, compiled from `.dag`, reads through it. The design
point the review made is answered by the reflection primitive
itself: the substrate hosts the fact, lenses walk it through
the substrate interface, and no consumer outside the substrate's
own realization layer can reinvent the mapping.

**Minimality.** The query primitive set is derived from what
the existing four lenses actually need. Each primitive corresponds
to a specific Rust method on `Dag` or one of its sub-types. The
set is not open-ended — adding a new primitive is a design
decision, not a free action. Primitives are the substrate's
public interface to `.dag` lens code; a smaller set is better.

**Realization.** Each query primitive is backed by a Rust
function in `src/v3/compiler/src/substrate_query.rs` (new file).
The realization declarations for each primitive live in
`src/v3/spec/rust.dag` alongside the existing type and operator
realizations — same mechanism, same `ArrowBody::ExternalRealization`
pattern.

### §3.3 Violation output type

A lens returns a list of violations. The violation type is a
per-lens record — `UnusedParameter` for `lens_unused_parameters`,
`Duplicate` for `lens_structural_duplicates`, etc. The `.dag`
form declares this record as a `Conj` in the lens's own module:

```
// dsl/lenses/unused_parameters.dag

module lenses.unused_parameters

import std.substrate { NodeId, PortId, SourceSpan }
import std.substrate_query as q

type UnusedParameter {
  function: NodeId
  parameter: PortId
  parameter_index: Int
  function_span: SourceSpan
}

fn check(d: Dag) -> List<UnusedParameter> {
  // ... algorithm walks the Dag via q.* primitives
}
```

The returned `List<UnusedParameter>` is compiled to a Rust
`Vec<UnusedParameter>` where the Rust struct is derived from the
`.dag` record declaration (same mechanism PR #445 uses for data
bodies via `ValueBody::Structural`).

---

## §4. The mechanism, step-by-step

How a `.dag` lens becomes a running check over a `Dag` value at
CI time:

1. **Substrate types loaded.** `dsl/std/substrate.dag`
   parses at bootstrap time via `include_str!` (same pattern as
   `dsl/std/algebra.dag`). The types are Declarations in the
   bootstrap Dag.
2. **Inhabitance check.** A startup pass walks the Rust substrate
   types (via `realization_meta_id`-style markers) and asserts
   each `.dag` declaration in `std.substrate::*` has a
   matching Rust struct. Mismatches are fail-closed diagnostics.
3. **Query primitives loaded.** `dsl/std/substrate_query.dag`
   parses. Each Arrow gets its `ArrowBody::ExternalRealization`
   resolved to a Rust function declaration in
   `src/v3/spec/rust.dag`. The bootstrap already has the
   `ExternalRealization` mechanism for `Int.add`, `Int.sub`,
   etc. — this is the same pattern applied to a different set of
   primitives.
4. **Lens file loaded.** `dsl/lenses/unused_parameters.dag` is
   compiled to a `Dag` via the standard `compile_to_dag` pipeline.
   The lens's `check` function becomes a Bind with body sub-DAG.
   The query primitive calls in the body are Transforms targeting
   the declarations from step 3.
5. **Lens compiled to Rust.** `emit_rust` runs on the lens Dag
   and produces a Rust function `fn check(d: &Dag) ->
   Vec<UnusedParameter>`. This is the same `emit_rust` pipeline
   that ships `let x: Int = 1 + 2` → Rust; no new path needed.
   The only extension: `emit_rust` must recognize Transforms
   targeting query-primitive declarations and emit Rust method
   calls (e.g., `d.declarations()`) instead of arithmetic
   operators.
6. **Compiled lens invoked.** A test, CI runner, or (eventually)
   the lens manifest subcommand calls the compiled function with
   a `Dag` value and collects violations. Zero Rust lens code
   involved. The lens lives in `.dag`, was compiled by v3's own
   pipeline, and runs as Rust at check time.

**What's new.** Only step 5's extension — `emit_rust` needs to
handle query-primitive Transforms. Everything else is pattern-
matching existing machinery. This is deliberately minimal: the
reflection primitive is small because most of the work is
already done.

### §4.1 `emit_rust` extension

The existing `emit_rust` handles `OperatorKind` transforms by
looking up `(target, op_name) → carrier` in the realization
index and substituting via template placeholders. The extension:
when a Transform's target is a declaration with
`ArrowBody::ExternalRealization` pointing at a Rust function
(as opposed to a type/operator/template), emit a direct method
call.

Concretely, `rust.dag` grows a new realization category:

```
// src/v3/spec/rust.dag

// ... existing TypeRealization, OperatorRealization, BehaviorRealization ...

// NEW: Rust function-call realization. Binds a .dag Arrow to a
// Rust method on a receiver value. The receiver is always the
// first Arrow input; subsequent inputs become call arguments.
type FunctionRealization {
  for: Arrow              // the .dag Arrow declaration
  receiver_type: Declaration  // which Rust type hosts the method
  method_name: String     // the Rust method name
}

data declarations_realization: FunctionRealization = {
  for: std.substrate_query.declarations
  receiver_type: std.substrate.Dag
  method_name: "declarations"
}

// ... one per query primitive in §3.2 ...
```

**Why a new realization category.** The three existing
categories (`TypeRealization`, `OperatorRealization`,
`BehaviorRealization`) each serve a distinct emit path. A
function-call realization is its own shape: it has a receiver
type + a method name + a structural Arrow declaration it
realizes. Packing it into the existing categories would overload
the meaning of their fields (layer opacity violation by analogy).

**Size estimate:** ~150 lines in `emit_rust.rs` — a new
`FunctionRealization` index, a dispatch arm in the Transform
handler, template generation for method calls. The algorithm
follows the same shape as the existing operator dispatch.

### §4.2 Query primitive Rust impls

New file: `src/v3/compiler/src/substrate_query.rs`. Each query
primitive becomes a method (or free function) on `Dag` /
`Behavior` / etc. The Rust impls are trivial — they're existing
field accessors wrapped in a stable public interface.

```rust
impl Dag {
    pub fn declarations(&self) -> Vec<Declaration> { ... }
    pub fn nodes(&self) -> Vec<Behavior> { ... }
    pub fn declaration_by_id(&self, id: DeclarationId) -> Declaration { ... }
    // ... ~20 methods total
}
```

**Why a dedicated module.** The Rust impls are the bridge between
the `.dag` reflection surface and the compiler's internal state.
Gathering them in one file makes the inhabitance check mechanical
(every method in `substrate_query.rs` corresponds to exactly one
query primitive in `std.substrate_query`) and gives future
reviewers one place to audit for leaks.

---

## §5. Lens migration: `lens_unused_parameters` as the proof point

This is the migration template. Once it lands and the tests are
green, the other three existing lenses follow the same pattern
as follow-up work (not in this PR).

**Before** (current Rust, `src/v3/compiler/src/lens_unused_parameters.rs`):

```rust
pub struct UnusedParametersLens<'a> { dag: &'a Dag }

impl<'a> UnusedParametersLens<'a> {
    pub fn query(&self, config: &UnusedParametersConfig) -> Vec<UnusedParameter> {
        let mut violations = Vec::new();
        for node in self.dag.nodes() {
            let Behavior::Bind(bind) = node else { continue };
            if bind.params.is_empty() { continue; }
            self.check_bind(bind, &mut violations);
        }
        violations
    }

    fn check_bind(&self, bind: &BindNode, out: &mut Vec<UnusedParameter>) {
        let referenced = collect_referenced_ports(self.dag, bind.value);
        for (idx, &param) in bind.params.iter().enumerate() {
            if !referenced.contains(&param) {
                out.push(UnusedParameter { ... });
            }
        }
    }
}
```

**After** (`.dag`, `dsl/lenses/unused_parameters.dag`):

```
module lenses.unused_parameters

import std.substrate { Dag, Behavior, Bind, NodeId, PortId, SourceSpan }
import std.substrate_query as q

type UnusedParameter {
  function: NodeId
  parameter: PortId
  parameter_index: Int
  function_span: SourceSpan
}

fn check(d: Dag) -> List<UnusedParameter> {
  q.nodes(d)
    .filter(is_function_bind)
    .flat_map(fn(b) = check_bind(d, b))
}

fn is_function_bind(b: Behavior) -> Bool =
  match b {
    Bind(bind) => not(q.bind_params(bind).is_empty()),
    _          => false,
  }

fn check_bind(d: Dag, b: Bind) -> List<UnusedParameter> {
  let referenced = referenced_ports(d, q.bind_value(b))
  q.bind_params(b)
    .enumerate()
    .filter(fn((_, port)) = not(referenced.contains(port)))
    .map(fn((idx, port)) = UnusedParameter {
      function: q.bind_owner_node(b),
      parameter: port,
      parameter_index: idx,
      function_span: q.bind_span(b),
    })
}

fn referenced_ports(d: Dag, root: PortId) -> Set<PortId> {
  // Iterative work-list walking through q.port_producer + input sets.
  // Structurally the same as collect_referenced_ports in the Rust form.
  ...
}
```

**What changed:**

- The `.dag` form uses query primitives (`q.nodes`, `q.bind_params`,
  `q.port_producer`) instead of direct field access.
- The algorithm is identical — same walk, same output.
- The host language is the substrate itself.

**What didn't change:**

- The test fixtures in
  `src/v3/compiler/tests/m1_3_lens_unused_parameters_test.rs`
  stay. After migration, the tests invoke the compiled `.dag`
  lens (via a thin Rust shim that calls the `check` function) and
  assert the same output.
- The canonical-target blocker stays. `content_upsert` in
  `patterns.dag` is still blocked on class-5 parser gaps. Both
  the literal-fail test and the synthetic-equivalent test
  continue to pass. This migration does not depend on the parser
  catching up.

**Rust lens deletion.** After the `.dag` form lands and tests are
green, `src/v3/compiler/src/lens_unused_parameters.rs` is
deleted. The `.dag` form is the only lens. No parallel
representation, no "bootstrap vs canonical" dual state (per
`INVARIANTS.md` §"No bridges" and §"Semantic authority after
lowering").

---

## §6. Testing strategy

Four test categories, all adding to the existing
`cargo test -p v3-compiler` suite.

### §6.1 Substrate inhabitance

New test: `tests/m2_substrate_inhabitance_test.rs`. Asserts that
every declaration in `std.substrate::*` has a matching
Rust struct in `src/v3/compiler/src/dag.rs`, field-by-field.
Mismatches fail the test with a diagnostic pointing at the
specific field. This is the binding contract — if the Rust
substrate drifts from the `.dag` declaration, the test catches
it immediately.

### §6.2 Query primitive parity

New test: `tests/m2_query_primitive_test.rs`. For each query
primitive in `std.substrate_query::*`, asserts (a) it has an
`ExternalRealization` body, (b) the realization resolves to a
Rust method in `substrate_query.rs`, (c) calling the primitive on
a test Dag returns the same value as calling the Rust method
directly on the same Dag.

### §6.3 Lens migration parity

New test: `tests/m2_lens_unused_parameters_migration_test.rs`.
For every fixture in the existing
`m1_3_lens_unused_parameters_test.rs`, compile the `.dag` lens
form, run it against the same input Dag, and assert the output
matches the Rust form byte-for-byte. This is the migration
correctness proof. After this lands, the Rust form is deleted.

### §6.4 Self-analysis

New test: `tests/m2_lens_self_analysis_test.rs`. Compiles
`dsl/lenses/unused_parameters.dag` to a Dag, then runs the
compiled lens against *itself* — the lens's own Dag as input.
Asserts the output is empty (zero unused parameters in the
lens's own source). This is the composability claim made
empirical: a lens can analyze a lens.

---

## §7. Thesis integration

### §7.1 New `THESIS.md` section: "Self-inspection"

Between §"Compositional layering" and §"Two groundings", a new
subsection:

> **Self-inspection: the substrate is its own subject.** The
> substrate's structural type layer — `Dag`, `Declaration`,
> `Behavior`, `TypeConnective` — is declared in
> `dsl/std/substrate.dag` as ordinary `.dag` declarations,
> sibling to `std/algebra.dag` and `std/types.dag`. There is no
> separate "meta substrate" and no subdirectory: the substrate
> describes itself as data **inside** itself, and the
> self-reference is a fixed point, not a stratification. The
> Rust structs in `src/v3/compiler/src/dag.rs` are realizations
> of these `.dag` declarations, kept consistent by inhabitance.
> A `.dag` program can receive a compiled `Dag` as input and
> walk it via the query primitives in `std.substrate_query`.
> Lenses — the invariant-enforcement primitive — are `.dag`
> programs over this reflection surface. This closes the last
> self-reference gap in the thesis: there is no code the
> substrate cannot observe. Adding a new lens is a new `.dag`
> file in `dsl/lenses/`, not a Rust module. The thesis's "cost
> of change = 1 file" claim applies to lenses by the same
> mechanism it applies to types and behaviors.
>
> **The seed is bounded.** The bootstrap seed is the parser,
> the resolve sweep, and four atomic identity handles
> (`NodeId`, `PortId`, `DeclarationId`, `SourceSpan`) — plus
> the existing arithmetic primitives (`Int`, `Bool`, `String`).
> Everything else — every structural substrate type — is a
> declaration, not a primitive. The seed is a ratchet: it can
> shrink (if a seed primitive becomes expressible as a
> declaration) but cannot grow. See `INVARIANTS.md`
> §"Bounded substrate seed" for the enforcement rule.
>
> **The failure mode this prevents** is a `kernel_lens_set` — a
> hardcoded registry of known lens names in compiler source with
> per-lens dispatch. Every compiler that ships analyses-as-built-in-
> modules eventually accumulates one. v3 does not, because lenses
> live in the substrate itself; there is no registry to accumulate.
>
> **The composability property** is a consequence: a lens can be
> analyzed by another lens, including itself. `lens_unused_parameters`
> runs against its own `.dag` source at test time and reports zero
> findings. If it regressed, the test would catch it before merge.

### §7.2a New `INVARIANTS.md` entry: "Bounded substrate seed"

```
### Bounded substrate seed

The bootstrap seed — the set of Rust primitives the compiler
ships before any `.dag` declaration loads — is a ratchet. It
can shrink, it cannot grow. A new Rust primitive landing in
the seed is a **fail-closed blocker** unless it meets one of
two exceptions.

**The seed, as of the substrate reflection PR:**

1. `NodeId` — opaque identity handle for behavior nodes.
2. `PortId` — opaque identity handle for ports.
3. `DeclarationId` — opaque identity handle for declarations.
4. `SourceSpan` — opaque source-location handle.
5. `Int`, `Bool`, `String` — existing arithmetic primitives.

Plus: the parser, the `resolve_pending_identifiers` sweep, the
inhabitance check mechanism. None of these are types; they are
the machinery that lets declarations parse and resolve.

**The two exceptions** under which a new primitive can land:

1. **Atomic identity.** The concept is a handle that programs
   pass around and compare for equality but never structurally
   inspect. `NodeId` is the template. If the concept has fields
   users can read, it is not a handle and does not qualify.
2. **Truly indivisible.** The concept is at the floor of its
   decomposition chain (Classical Bit, raw machine word). If
   the concept decomposes further, it belongs in `.dag` as a
   declaration, not in the seed.

**Every other substrate concept must be declared in `.dag`.**
This includes every structural type the substrate uses to
describe itself: `Dag`, `Declaration`, `TypeConnective`,
`Behavior`, `ArrowBody`, `Field`, `AtomPayload`,
`CardinalityBound`, `TemplateArgument`, and any successor
concepts. The Rust structs in `dag.rs` are realizations, not
authorities — the authority is the `.dag` declaration.

**Why the ceiling matters.** Every seed primitive is a fact
`.dag` cannot analyze. A lens cannot ask "does `Behavior` have
a `Loop` variant?" if `Behavior` is a Rust-only primitive. The
thesis's self-hosting claim requires the substrate to host
itself; every seed primitive is a hole in that claim. The
ratchet ensures the holes shrink over time, never grow.

**Enforcement.** The CI check counts seed primitives by grepping
the realization entries in `src/v3/spec/rust.dag` for types that
have no `.dag` declaration counterpart. The count is stored in
a ratchet file (same mechanism as the `Pending` ratchet) and
must monotonically decrease or stay flat across PRs. A PR that
adds a new seed primitive fails CI unless the PR also deletes
an equal or greater count of existing seed primitives, OR the
addition meets one of the two exceptions above and is receipted
inline in the PR.
```

### §7.2b New `INVARIANTS.md` entry: "Lenses are substrate declarations"

```
### Lenses are substrate declarations

Every lens added to the project after the substrate reflection
primitive lands MUST be a `.dag` program in `dsl/lenses/`
operating over the query primitives in `std.substrate_query`.

Rust lens modules (e.g., `src/v3/compiler/src/lens_*.rs`) are
forbidden for new lenses. Exception: the migration of the three
existing Rust lenses (`lens_provenance`, `lens_depth`, `lens_cost`)
is tracked as followup work; each deletion happens as its own
PR, and no new Rust lens can land in the meantime.

**Why this invariant is necessary.** A compiler that permits
Rust-form lenses will accumulate them. Each Rust lens grows
dispatch patterns, shared infrastructure, and implicit knowledge
of where lenses "live" in the source tree. That accumulation is
how a `kernel_lens_set` forms. The only way to prevent the
failure class is to forbid Rust-form lenses at the invariant
level — the substrate IS the lens host.

**The enforcement lens.** A structural check (via
`lens_layer_opacity` applied to `src/v3/compiler/src/`) verifies
no compiler source file contains a Rust function whose signature
matches the lens template (`fn check(dag: &Dag) -> Vec<_>` or
similar). The check runs in CI as part of the standard lens
invocation.

**Exception scope.** Bootstrap helpers that accept a Dag and
return a derived value but are NOT lenses (e.g., the realization
index builder, the symbol table builder) are exempt. The test is:
is the function's output a list of findings about the Dag? If
yes, it's a lens and must be `.dag` form. If it's a derived data
structure used internally, it's bootstrap and is exempt.
```

### §7.3 `lens-library-design.md` rewrites

- **§1.5 "Canonical form"** — demoted from "future state" to
  "current state." The section header changes to "Canonical form:
  lenses are `.dag` programs." All "once reflection lands" hedge
  language deleted. The thin-wrapper-vs-deepening-scaffold gate
  stays but becomes retrospective: any lens that doesn't fit the
  `.dag` form is a bug, not a bootstrap allowance.
- **§2.3 `lens_unused_parameters`** — the signature example
  rewrites to `.dag`. The Rust form disappears. The test plan
  references the `.dag` fixtures.
- **§7 "Existing infrastructure"** — the four-lens list updates
  to reflect migration status. When this PR lands, the list is
  `lens_unused_parameters (.dag), lens_provenance (Rust —
  migration pending), lens_depth (Rust — migration pending),
  lens_cost (Rust — migration pending)`.

---

## §8. Relationship to other ongoing work

**`project_node_to_std`** (tracked in memory): this work IS that
work. The tracked note should be resolved / deleted when this
PR lands. The scope in the memory note was narrower (just move
Node) — this design expands it to the full type layer, which is
what's actually needed for reflection.

**`lens_complexity` as the v2/v3 comparison:** blocked on this
PR. After reflection lands and the migration template is proven
by `lens_unused_parameters`, building `lens_complexity` is
straightforward: audit what substrate facts v3 carries (loop
bounds, dimensions, descent evidence via `produced_by` walks),
build a `.dag` lens that reads them via the query surface, compare
line count and algorithmic clarity against v2's `complexity.dag`.
This becomes a follow-up PR with a clean scope.

**`lens_structural_duplicates`, `lens_layer_opacity`:** both
still-valuable future lenses; both land as `.dag` programs after
reflection. Neither is in this PR.

**`INVARIANTS.md §"Layer opacity"`:** the long-term target
(§"Rust type-level enforcement") still applies. Reflection is the
next step toward it, not the endpoint. Layer opacity's ultimate
form is a `DisplayName` type that makes the violation impossible
at the Rust level; this PR makes lens-based enforcement
structurally cheaper, which buys time for the type-level work.

**`INVARIANTS.md §"Scaffold boundaries"`:** the Rust lenses are
tracked scaffolds from this PR's perspective. The tracked
`kernel_lens_set` count starts at 3 (the remaining Rust lenses
after `lens_unused_parameters` migrates) and must monotonically
decrease. Each Rust-lens deletion is a ratchet tick.

**`INVARIANTS.md §"Semantic authority after lowering"`:** this
PR extends the invariant's reach. Lenses currently read substrate
state through Rust field access; after reflection, they read
through query primitives that go through the same authority path
as user code. One authority for "what does this Dag say" — the
query primitive set — not two (Rust fields + user-facing `.dag`).

---

## §9. Scope boundary — what does NOT land in this PR

- Other lenses' migration. Only `lens_unused_parameters` moves.
- A runtime interpreter for `.dag` lenses. Lenses compile to Rust
  via the existing pipeline.
- New substrate variants. Additive only: new declarations in
  `dsl/std/`, new Rust file for query impls, new file for the
  lens.
- The lens manifest format or `v3-compiler lens run` CLI. Both
  still deferred.
- Moving L1 behaviors themselves into `.dag`. The type layer is
  enough for reflection to work; the computation layer can
  follow.
- Parser or grammar extensions. If the lens needs a feature the
  parser doesn't support, the feature is a blocker prerequisite
  PR, not bundled.

---

## §10. Open questions

1. **How are atomic identity handles (`NodeId`, `PortId`,
   `DeclarationId`, `SourceSpan`) seeded?** Per §3.0 they stay
   as Rust primitives and get realization entries in
   `src/v3/spec/rust.dag` — the same mechanism `Int → i64` uses.
   The question to audit: does the existing `TypeRealization`
   shape support opaque-handle types (not just arithmetic
   primitives)? If yes, no new work. If it requires a
   `HandleRealization` sibling category, the addition is
   scoped as part of this PR.
2. **Higher-order functions in `.dag`.** The `.dag` lens form
   uses `filter`, `flat_map`, `enumerate` — do these exist in
   `dsl/std/list.dag` today? If not, they're prerequisite
   declarations. Audit before writing the lens.
3. **`Set<PortId>` semantics.** The Rust lens uses a `HashSet`;
   the `.dag` form needs a set-shaped collection type. Does v3
   have `Set<T>` in `std/collections.dag`? If not, it's a
   prerequisite.
4. **Lens entry point convention.** The `.dag` form declares
   `fn check(d: Dag) -> List<_>`. Is "check" the canonical name,
   or does each lens pick its own? Convention: `check` is the
   entry point every lens must define. The runner looks up
   `{lens_module}.check` by structural path, not by string
   match (layer opacity).
5. **How does the lens result's violation record type get wired
   to Rust?** When `emit_rust` compiles `lenses.unused_parameters`,
   the `UnusedParameter` record needs a Rust type. PR #445's
   `ValueBody::Structural` handles literal data; this is a
   structural record type in function-return position. Does the
   existing path support it, or is there a small extension?
   Audit during implementation.
6. **Does the check function need any configuration?** The Rust
   form takes `&UnusedParametersConfig`; the `.dag` form has no
   parameter beyond `Dag`. Config for future lenses (e.g.,
   `BoundarySpec` for `lens_layer_opacity`) can be a second
   parameter to `check`: `fn check(d: Dag, cfg: Config) ->
   List<_>`. Leave the single-parameter form for lenses that
   don't need config.

These resolve during implementation. None are blockers for
starting.

---

## §11. What counts as done

- [ ] `dsl/std/substrate.dag` exists as a regular `std/` file
      (no subdirectory), parses cleanly, inhabitance check
      passes against the Rust substrate structs in `dag.rs`.
- [ ] Atomic identity handles (`NodeId`, `PortId`,
      `DeclarationId`, `SourceSpan`) have realization entries
      in `src/v3/spec/rust.dag` and are **not** declared in
      `.dag`. The seed is four handles plus the existing
      arithmetic primitives.
- [ ] Seed ratchet file exists, CI gate counts seed primitives,
      a PR that adds a new seed primitive without a receipted
      exception fails CI.
- [ ] `dsl/std/substrate_query.dag` exists, all primitives have
      `ExternalRealization` bodies resolved at lower time.
- [ ] `src/v3/compiler/src/substrate_query.rs` exists; every
      primitive has a Rust impl; the impls are called through a
      stable interface and never via direct field access.
- [ ] `src/v3/spec/rust.dag` grows `FunctionRealization` entries
      for each query primitive.
- [ ] `emit_rust.rs` handles `FunctionRealization` Transforms
      and emits Rust method calls.
- [ ] `dsl/lenses/unused_parameters.dag` exists, compiles
      cleanly, produces the same output as the Rust form for
      every test fixture.
- [ ] `src/v3/compiler/src/lens_unused_parameters.rs` is
      deleted.
- [ ] `m2_substrate_inhabitance_test.rs` passes.
- [ ] `m2_query_primitive_test.rs` passes.
- [ ] `m2_lens_unused_parameters_migration_test.rs` passes
      (byte-for-byte with the Rust form's previous output).
- [ ] `m2_lens_self_analysis_test.rs` passes (the lens analyzing
      itself reports zero findings).
- [ ] `THESIS.md` §"Self-inspection" landed.
- [ ] `INVARIANTS.md` §"Bounded substrate seed" landed.
- [ ] `INVARIANTS.md` §"Lenses are substrate declarations"
      landed.
- [ ] `lens-library-design.md` §1.5 rewritten, §2.3 rewritten,
      §7 updated.
- [ ] `project_node_to_std` memory resolved / deleted.
- [ ] Clippy clean, cargo test clean.

---

## §12. What this supersedes

- `lens-library-design.md` §1.5 "Canonical form" changes from
  aspirational to authoritative.
- `lens-library-design.md` §2.3 `lens_unused_parameters` Rust
  signature is deleted; the `.dag` form replaces it.
- The `kernel_lens_set` failure class (named in §1 of this doc)
  is prevented by construction rather than prevented by gate.
- `project_node_to_std` memory (narrower scope; subsumed).
- The `lens-library-design.md` §5 implementation-order ordering
  moves `lens_unused_parameters` migration to the top of the
  list; `lens_structural_duplicates` and `lens_layer_opacity`
  become `.dag`-form-from-the-start tasks.

---

## §13. Recent-conversation threads captured

Cross-reference for the user's "make sure nothing is missed" ask.
Every thread below is either addressed in-doc or explicitly
deferred with a note.

| Thread | Status |
|---|---|
| "Everything is Dag/Node" / project as meta-compiler | §3.0, §3.1, §7.1 — substrate is self-describing, no meta layer, no substrate subdirectory |
| Primitives vs declarations ("l0 primitives vs declare them") | §3.0 — declare everything structural, four seed handles only, ratcheted |
| Bootstrap problem ("how does this not cycle?") | §3.0 — same mechanism v3 already uses for algebra.dag → types.dag forward refs |
| Seed-minimality invariant | §7.2a — new `INVARIANTS.md` entry with ratchet |
| ChatGPT review: `behavior_output_port` as canonical query | §3.2, §3.2.1 — landed as a first-class query primitive |
| Reflection primitive as canonical form | §1, §3, §4 — the core of this doc |
| Rust lenses as bootstrap scaffolds | §1 motivation, §5 migration |
| Thin-wrapper-vs-deepening-scaffold gate | §7.3 (retrospective after this PR) |
| `kernel_lens_set` failure class | §1, §7.1 thesis, §7.2 invariant |
| Substrate-as-data | §3.1 substrate types in std/ |
| Query primitives for `.dag` programs | §3.2 |
| Interpreter binding (.dag receives Dag) | §4 step-by-step, §4.1 emit_rust extension |
| Composition Opacity = Layer Opacity | already reconciled (PR #445), cross-ref in §8 |
| Semantic authority after lowering | §8 — this PR extends it |
| Scaffold boundaries | §8 — Rust lenses are tracked scaffolds |
| `lens_unused_parameters` as first migration | §5 |
| `lens_complexity` as v2/v3 comparison | §8 — deferred follow-up |
| `content_upsert` blocked on parser gaps | §5 — migration preserves the existing pinned tests |
| Positional identity (parameter_index) | §5 — preserved |
| No ghost config fields | §3.3, §10 Q6 — lens config is a real parameter |
| Node → std/ (project_node_to_std memory) | §8, §12 — subsumed and resolved |
| Self-hosting loop | §1, §7.1 — reflection is a stepping stone |
| Two groundings (static + realization) | §4.1 — realization pattern extended to compiler-internal types |
| Canonical target test requirement | §5 — tests preserved |
| Checkpoint dissolution default | (general project doctrine — not specific to this PR) |
| Compositional layering (thesis-level) | §7.1 — self-inspection is a new subsection |
| Rename test as regression check | (covered by existing `INVARIANTS.md §"Layer opacity"`) |

If a recent thread is missing from this table and it belongs
in-scope, flag it and this section updates.
