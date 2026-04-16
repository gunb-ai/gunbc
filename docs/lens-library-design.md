# Lens Library Design

> Part of: [v3-spec.md](v3-spec.md), [../THESIS.md](../THESIS.md),
> [../INVARIANTS.md](../INVARIANTS.md)
>
> **Purpose:** specify the initial lens library — structural
> invariant enforcement via reader lenses over the DAG — and the
> opt-in application mechanism that lets the same lens run against
> compiler source, user projects, or any other DAG with
> project-specific boundaries.

## What this document is

v3 already has three reader lenses working: `lens_provenance`,
`lens_depth`, and `lens_cost` (the first three added in M0 and
PR-B). Each is a pure function over `Dag` returning a structural
answer. This document specifies a **lens library** — a growing
set of reader lenses that enforce thesis-level invariants over
arbitrary DAG segments.

The library is the structural answer to the question we were
initially going to solve with grep gates: how does the compiler
enforce its own design invariants on its own source code? The
answer is: **write a lens for each invariant, apply it to the
scope that should be subject to the invariant, fail PRs whose
lens output is non-empty.**

Each lens in the library is:

- **Pure.** Reads `Dag` + a configuration object, returns a list
  of violations. No persistent state, no side effects, no
  internal mutation.
- **Opt-in.** Applied to specific files or folders via an
  application manifest (see §3 below). Not every invariant is
  enforced on every source file; each lens has a declared scope.
- **Structurally typed.** Violations carry enough information
  for a reviewer or CI system to understand what the violation
  is, where it is, and why it's a violation.
- **Thesis-native.** Uses existing v3 infrastructure (reader
  lens shape, `Dag` API, declaration walking) rather than
  inventing new machinery. Adding a new lens = new file + tests,
  zero substrate changes.

The library is itself a test of the thesis's extensibility claim:
**"adding a new analysis should be trivial — proportional to the
analysis's conceptual complexity, not proportional to substrate
modifications required."** Each lens in this document is an
instance of that claim.

## §1. Framing: lenses as invariant enforcement

The thesis originally framed lenses as cost, ownership, and
complexity analyses — structural questions about user programs
that the compiler answers on demand. That framing undersold them.
**A lens is the invariant-enforcement primitive**, and the
"analysis" framing is a special case of the general mechanism.

Most testable invariants over a DAG have the shape:

> Walk the DAG. For each site of kind `K`, check property `P`.
> Return the list of sites where `P` fails.

This is the shape of a reader lens. The "analysis" examples
(cost, ownership) are lenses where the returned value is a
derived quantity (per-node cost) instead of a list of
violations. The "invariant" examples (layer opacity, scaffold
boundaries, duplicate records) are lenses where the returned
value IS a list of violations. Same mechanism, different return
types.

Once you see invariants as lenses, the lens library grows along
two axes:

1. **Per-invariant lens.** For each invariant in
   `INVARIANTS.md` whose check has a structural shape, write a
   lens that returns violations.
2. **Per-application scope.** For each lens, declare which
   source trees it applies to, with what configuration.

The library replaces:

- **Grep gates** (lexical, brittle, false-positive-prone, needs
  hand-maintained manifests)
- **Custom test code** (one-off per invariant, tied to Rust,
  doesn't generalize)
- **Review-time vigilance** (does not scale, re-finds the same
  patterns every round)

Each is a fallback the project has used at various points. The
lens library is the structural replacement.

## §1.5 Canonical form: lenses as `.dag` programs

The load-bearing form of a lens is a `.dag` program that takes
a `Dag` (or a sub-DAG) as input and returns **lens-specific
output** — a list of violations, a per-port cost, a per-node
depth, a provenance map, whatever shape the analysis produces.
Lenses written as Rust modules are a **bootstrap shortcut**,
not the endgame.

**The canonical shape is "pure reader with lens-specific
output," not "pure reader returning violations."** The four
lenses shipping today demonstrate the range:

- `lens_provenance` returns per-port origin information.
- `lens_depth` returns per-port depth.
- `lens_cost` returns per-node cost.
- `lens_unused_parameters` returns a list of violations.

Three of the four are analyses; one is a violation list. The
common shape is **pure function, `Dag` in, structured output
out**. "Violation list" is one instantiation. The canonical
form must not bake "violation" into the signature, or else
analyses like cost and depth don't fit the template.

**Why `.dag` form is canonical:**

1. **Composability with the rest of the substrate.** A `.dag`
   lens is a Declaration like any other. It can be walked by
   other lenses, type-checked by inhabitance, inhabited by test
   data, grounded through the epistemic chain, and analyzed by
   every future tool the substrate gains. A Rust lens is a black
   box to the substrate; nothing in `.dag` can observe it.
2. **Self-inspection.** A `.dag` lens can be analyzed by another
   `.dag` lens — including itself. `lens_unused_parameters` could
   run against its own source and report unused parameters in its
   own algorithm. A Rust lens can't; you'd need a meta-language
   to analyze it, which means accumulating another scaffold.
3. **No `kernel_lens_set`.** Every compiler that ships
   analyses-as-built-in-modules eventually accumulates a hardcoded
   list of "known lens names." That list is the same failure
   class as v2's `kernel_type_set` — a name-roster leak at a
   different layer. Keeping lenses in the substrate prevents this
   from forming in the first place.
4. **Extension stays a data edit.** Adding a new lens in the
   canonical form is a new `.dag` file loaded by the manifest,
   not a new Rust module + build-graph edit + CI integration
   step. This is the thesis's "cost of change = 1 file" applied
   to invariant enforcement.

**The remaining missing primitive is execution, not reflection.**
The reflection surface now exists in staged form:

- `src/v3/std/substrate.dag` declares the substrate's structural
  types (`Dag`, `Declaration`, `Behavior`, behavior payload
  records, etc.).
- `src/v3/spec/rust.dag` carries `TypeRealization` entries for the
  reflected substrate types plus per-field `FieldBinding`
  mappings to Rust fields and accessor methods.
- `emit_rust.rs` can render reflected field access through those
  bindings.

What is still missing is the final step: a `.dag` lens receiving a
runtime `Dag` value as input and being invoked as compiled code. Until
that execution path lands, lenses still ship as Rust bootstrap modules.

**The reflection primitive is the subject of
[`docs/substrate-reflection-design.md`](substrate-reflection-design.md)**
(merged into this branch in commit 700dba6). That design doc
specifies substrate types as `.dag` declarations, a query
primitive set, and the migration path for the four existing
lenses. When the reflection primitive lands, the §1.5 "thin
wrapper" gate becomes a **structural invariant** enforced by
two ratchets:

1. **"Bounded substrate seed"** — the count of Rust-side
   primitive types monotonically decreases. New seed primitives
   are a fail-closed blocker unless they meet narrow exceptions
   (atomic identity handle or truly indivisible).
2. **"Lenses are substrate declarations"** — new lenses MUST be
   `.dag` programs. Rust lens modules for new lenses are
   forbidden; the three remaining Rust lenses after
   `lens_unused_parameters` migrates are tracked in the seed
   ratchet and each deletion is a ratchet tick.

Both invariants are now first-class entries in `INVARIANTS.md`; the
reflection design doc remains the design history and migration plan.

**Current Rust lenses are bootstrap scaffolds with narrower
remaining scope.** Reflection and field-binding realization are now in
place, so the remaining blocker is not "the substrate is invisible";
it is "compiled `.dag` lenses are not yet invocable on a runtime
Dag value." `lens_provenance`, `lens_depth`, `lens_cost`, and
`lens_unused_parameters` therefore remain bootstrap scaffolds, but
their migration target is now mechanically clearer: the substrate
surface and Rust realization bridge they need already exist.

**Reviewer-enforced gate until the ratchet lands.** Before
adding a new lens as Rust, ask: does this thicken the Rust-lens
scaffold, or is it a thin wrapper that will migrate cleanly
once reflection lands?

- **Thin wrapper.** Algorithm is ~50-100 lines, pure reader,
  follows the template of existing lenses, no cross-lens
  interaction, no state outside the lens-specific output type.
  Adding one is cheap and the migration cost is a mechanical
  port.
- **Deepening scaffold.** Algorithm grows Rust-specific infra
  (shared types across lenses, a per-lens trait hierarchy, a
  registration framework, dependency wiring between lenses).
  Stop and build the reflection primitive first — otherwise the
  scaffold ossifies and the canonical form is blocked.

This gate is reviewer-enforced until the substrate reflection
PR lands. After that, the ratchet takes over and the gate
becomes a structural CI check instead of a reviewer discipline.

## §2. The initial library: three lenses

The first three lenses in the library. Each is minimum-viable
and targets a specific, concrete failure class that has been
hand-found in existing reviews.

### §2.1 `lens_structural_duplicates`

**Purpose:** detect type declarations or data declarations with
the same structural shape, flagging duplicates regardless of
name.

**Motivation from the reviewer's std/extdeps audit (2026-04-15):**

- `FileClassification` and `FileEntry` in `dsl/std/filesystem.dag`
  are the same record shape (same fields, same types), each
  claiming to be "what Filesystem.probe returns."
- `wire_contract: VariantEncoding = StringVariant { naming:
  SnakeCase }` declared identically in `llm.dag`, `anthropic.dag`,
  and `openai.dag` — two redundant duplicates of the imported
  value.
- `default_edition: String = "2021"` declared in both `cargo.dag`
  and `rust/imports.dag`.
- `NonEmptyStr` and `NonEmptyString` — two aliases for the same
  refined type in `dsl/std/types.dag`.

All four were hand-found in a single review pass. A
`lens_structural_duplicates` applied to `dsl/std/` and
`dsl/extdeps/` would return all four automatically, plus any
others the reviewer missed.

**Signature:**

```rust
// src/v3/compiler/src/lens_structural_duplicates.rs

pub struct StructuralDuplicatesLens;

pub struct DuplicatesConfig {
    /// Include `type` declarations in the comparison.
    pub include_types: bool,
    /// Include `data` declarations in the comparison.
    pub include_data: bool,
    /// When true, identical declarations in modules with an
    /// explicit import relationship are ignored (a re-export
    /// is not a duplicate).
    pub ignore_re_exports: bool,
}

pub struct Duplicate {
    pub hash: u64,
    pub declarations: Vec<(DeclarationId, SourceSpan)>,
    pub kind: DuplicateKind,
}

pub enum DuplicateKind {
    /// Two type declarations with identical field shapes.
    IdenticalType,
    /// Two data declarations with identical bodies.
    IdenticalData,
    /// Two type declarations with compatible but differently-named
    /// structures (e.g., same fields in different order).
    CompatibleType,
}

impl StructuralDuplicatesLens {
    pub fn query(
        dag: &Dag,
        config: &DuplicatesConfig,
    ) -> Vec<Duplicate> { ... }
}
```

**Algorithm sketch:**

1. Walk every declaration in `dag.declarations()` that matches
   `config.include_types` or `config.include_data`.
2. For each type declaration, compute a structural hash over its
   field shape: `hash({ (field_name, field_type_declaration_id)
   for each field })`. For each data declaration, compute a
   structural hash over its body: `hash(body_connective)`.
3. Collect hashes into a `HashMap<u64, Vec<DeclarationId>>`.
4. Emit a `Duplicate` for every hash with more than one
   declaration.
5. If `ignore_re_exports` is set, filter out duplicates where one
   declaration imports the other transitively.

**Expected initial findings** (from the reviewer's audit):

- `FileClassification` / `FileEntry` in `dsl/std/filesystem.dag`
- `NonEmptyStr` / `NonEmptyString` in `dsl/std/types.dag`
- `wire_contract` in `llm.dag` vs `anthropic.dag` vs `openai.dag`
- `default_edition` in `cargo.dag` vs `rust/imports.dag`
- Probably others nobody has noticed yet

**Test plan:**

- Unit test: construct a Dag with two records of identical shape
  and different names, assert the lens reports them.
- Unit test: construct a Dag where one declaration re-exports
  another; assert the lens does NOT report it when
  `ignore_re_exports` is true.
- Unit test: construct a Dag with three data declarations of the
  same value; assert the lens reports all three.
- Integration test: run the lens against `dsl/std/` and
  `dsl/extdeps/`; assert the expected duplicates are in the
  output.
- Regression test: after the expected duplicates are fixed,
  re-run the lens and assert the output is empty.

**Shape notes:** ~50-100 lines of Rust plus tests. No substrate
changes. No new configuration schema beyond what the lens's own
struct defines.

---

### §2.2 `lens_layer_opacity`

**Purpose:** detect sites where a consumer of the DAG reads a
below-boundary identifier by name, violating the compositional
layering claim.

**Motivation from the review cycle:**

- v3 M1(2.6) rounds 5-7 eliminated name-dispatch bugs at the
  inference layer by hand.
- v3 M1(2.7)'s big fix closed 14 gaps, most of which were
  instances of the same pattern.
- v3 PR-B's `emit_rust.rs` introduced a fresh version of the
  same pattern at the emit layer: `lookup("Int", "")`,
  `lookup("Bool", "")`, `match label.as_str() { "True" => ... }`.
- v2's `kernel_type_set` (in `dsl/std/types.dag` and
  `src/v2/stage0/src/std_types.rs`) hardcodes eight primitive
  type names and is known to fail the rename test.
- `dsl/extdeps/transports/shell.dag` declares
  `ShellOutputChannel = Stdout | Stderr | StdoutLines |
  ExitSuccess`, but consumers still use `from "stdout"` / `from
  "stderr"` / `from "exit_success"` / `from "stdout_lines"`
  across `cargo.dag`, `shell.dag`, `git.dag`, `browser.dag`.
- `dsl/extdeps/llm/anthropic.dag` and `openai.dag` declare
  `AnthropicModel` / `OpenAiModel` enums but accept `model:
  String` at service boundaries.

Every one of these is a consumer reading a below-boundary
identifier by name. A single lens parameterized by the
appropriate `BoundarySpec` would catch every one.

**Signature:**

```rust
// src/v3/compiler/src/lens_layer_opacity.rs

pub struct LayerOpacityLens;

pub struct BoundarySpec {
    /// Declarations that count as below-boundary for this
    /// application. Typically "all declarations in dsl/std/" or
    /// "all declarations in the 'rest' layer of my project".
    pub below_boundary: Vec<DeclarationId>,
}

pub struct Violation {
    pub location: SourceSpan,
    pub identifier: String,
    pub origin: DeclarationId,
    pub consumer_kind: ConsumerKind,
    pub suggestion: Option<String>,
}

pub enum ConsumerKind {
    /// A Transform target resolving via string name rather than
    /// typed reference.
    TransformDispatch,
    /// A Branch path matching on a variant by string label.
    VariantStringMatch,
    /// A string literal in expression position that matches a
    /// below-boundary name.
    StringLiteral,
    /// A record field typed as `String` whose value is constrained
    /// to a closed set declared below the boundary.
    FlattenedDiscriminant,
    /// A channel declaration (`from "X"`) that references a
    /// variant name as a string.
    TransportChannelString,
}

impl LayerOpacityLens {
    pub fn query(
        dag: &Dag,
        boundary: &BoundarySpec,
    ) -> Vec<Violation> { ... }
}
```

**Algorithm sketch:**

1. Build a reverse index from below-boundary identifier strings to
   their `DeclarationId`. The index is derived from `boundary.
   below_boundary` by walking each declaration and collecting its
   name plus any variant names.
2. Walk every `Behavior::Transform` in the DAG. For each, check
   the `target` field. If the target resolves through a string
   name that matches a below-boundary identifier, emit a
   `TransformDispatch` violation.
3. Walk every `Behavior::Branch`. For each path, check the
   `pattern` field. If the pattern resolves via a string label
   that matches a below-boundary identifier, emit a
   `VariantStringMatch` violation.
4. Walk every `Behavior::Value` whose data is a `String`. For
   each, check whether the string matches a below-boundary
   identifier. If so, and the string is in expression position
   (not a diagnostic message), emit a `StringLiteral` violation.
5. Walk every record type declaration. For each `String`-typed
   field, check whether the field's values are constrained to a
   closed set that's declared below the boundary. If so, emit a
   `FlattenedDiscriminant` violation.
6. Walk every transport output declaration. For each `from "X"`
   clause, check whether `X` matches a variant name declared
   below the boundary. If so, emit a `TransportChannelString`
   violation.

**Expected initial findings:**

- v3 `emit_rust.rs` string dispatches (`lookup("Int", "")`, etc.)
- v2 `kernel_type_set` (the eight primitive names hardcoded as
  strings)
- `ShellOutputChannel` string tags across `cargo.dag`,
  `shell.dag`, `git.dag`, `browser.dag` (~30+ sites)
- LLM provider `model: String` fields in `anthropic.dag` and
  `openai.dag`
- Single-value string discriminants in `CacheControl.type`,
  `ToolCall.type`, STS grant fields
- Browser transport flattening (`Launch` input using `headless:
  String = "false"`)

**Test plan:**

- Unit test: construct a Dag where a Transform target is a
  `UnresolvedIdentifier("+")` and the boundary includes the
  arithmetic operator names; assert the lens reports the
  violation.
- Unit test: construct a Dag where a Branch pattern uses a
  string label and the boundary includes the parent Disj's
  variant names; assert the lens reports it.
- Unit test: construct a Dag with a string literal in expression
  position matching a below-boundary identifier; assert the lens
  reports it.
- Unit test: construct a Dag with a string literal in a
  diagnostic message matching a below-boundary identifier; assert
  the lens does NOT report it (diagnostic exception).
- Integration test: apply the lens to `src/v3/compiler/src/` with
  `boundary = dsl/std/` and assert the expected known violations
  are in the output.
- Integration test: apply the lens to `dsl/extdeps/` with
  `boundary = ShellOutputChannel variants` and assert the
  expected `from "stdout"` / `from "stderr"` / etc. violations
  appear.
- Regression test: the weather example's rename test (run by
  hand on 2026-04-15) should be reproducible by applying the
  lens with `boundary = [Float]` and asserting zero violations
  after the intermediate-layer and internal-rename experiments.

**Shape notes:** ~150-200 lines of Rust plus tests. The main work
is the reverse-index construction and the walker for each
`ConsumerKind`. No substrate changes.

**Dependencies:** none — works today on the existing v3 Dag
shape. The tests that assert specific findings depend on v3
being able to parse the files they target (e.g.,
`dsl/extdeps/transports/shell.dag` must load successfully).

---

### §2.3 `lens_unused_parameters`

**Purpose:** detect function parameters that are declared in a
signature but not read in the body.

**Motivation from the reviewer's audit:**

- `fn content_upsert(content: String, path: String) -> { written:
  Bool } { let matches = content == ""; { written: !matches } }`
  in `dsl/std/patterns.dag:136-139`. The `path` parameter is
  unused. The function is a live stub that silently ignores half
  its declared inputs and returns a result that doesn't depend
  on the filesystem at all.

**This is the simplest dataflow lens** and is worth writing early
because:

- Stub functions often have unused parameters as a tell.
- Unused parameters indicate missing implementation or wrong
  signature.
- The check is structurally simple: walk the function body,
  collect referenced port IDs, compare against the function's
  parameter ports.

**Migration status after the reflection work.** The reflection
surface and field-binding realization path this lens needs now
exist:

- `src/v3/std/substrate.dag` declares `Dag`, `Behavior`, `BindNode`,
  `BranchPath`, and the rest of the walked substrate.
- `rust.dag` carries `TypeRealization.fields` bindings for those
  reflected record fields.
- `emit_rust` can render reflected field reads compositionally.

The remaining blocker is execution plumbing: the `.dag` form of the
lens can now be written against real substrate declarations, but the
project still needs the path that feeds a runtime `Dag` value into a
compiled lens and invokes the compiled function from Rust.

**Signature** (as shipped in `src/v3/compiler/src/lens_unused_parameters.rs`):

```rust
pub struct UnusedParametersLens<'a> {
    dag: &'a Dag,
}

/// Empty by design. The shipped lens takes no config. Earlier
/// drafts proposed `ignore_underscore_prefix: bool` and
/// `scope: Vec<DeclarationId>`; both were removed before ship
/// because neither has backing substrate data at the current
/// scope. See the "no ghost fields" rule below.
pub struct UnusedParametersConfig {}

pub struct UnusedParameter {
    pub function: NodeId,
    pub parameter: PortId,
    /// Positional index in the parameter list. v3's substrate
    /// does not carry per-parameter names past lowering.
    pub parameter_index: usize,
    pub function_span: SourceSpan,
}

impl<'a> UnusedParametersLens<'a> {
    pub fn query(&self, _config: &UnusedParametersConfig) -> Vec<UnusedParameter>;
}
```

**No ghost fields.** An earlier draft of this design proposed
two config fields that both got deleted before ship:

- **`ignore_underscore_prefix: bool`** — would have required
  reading parameter names, but v3's substrate doesn't carry
  per-parameter names past lowering. A consumer setting the
  flag would expect name-based filtering; the lens would
  silently no-op. Layer-opacity violation at the API surface.
- **`scope: Vec<DeclarationId>`** — would have restricted the
  scan to specific function declarations, but v3's substrate
  doesn't carry a `Bind → Declaration` back-edge at this scope.
  A consumer setting a non-empty scope would expect the scan
  to honor it; the lens would silently scan every Bind anyway.
  Same failure class as the underscore filter: a config field
  sounds enforceable but isn't.

Both are deferred until their backing substrate data exists:
`ignore_underscore_prefix` waits on a `BindNode.param_names`
field, `scope` waits on a `Bind → Declaration` typed edge.
Neither is in this PR; neither lands in the config until its
prerequisite substrate work does.

**Rule codified by this episode: a lens config field must not
be added before its backing substrate data exists.** A field
that "sounds enforceable but isn't" is a layer-opacity
violation at the API surface — the promise is behavioral, the
check is nothing. Forbid at introduction time, not at review
time.

**Positional identity, not name.** `UnusedParameter.parameter_index:
usize` is the substrate-honest way to name a parameter. Names are
part of surface syntax (`SurfaceItem::Fn`) and not preserved past
lowering. Indexing makes the violation addressable without
inventing a naming layer the substrate doesn't support.

**Algorithm** (as shipped in Rust bootstrap form, and unchanged in
the planned `.dag` migration):

1. For each `Behavior::Bind` in the Dag with non-empty `params`
   (function-shape filter — value bindings have no parameters),
   walk the body sub-DAG starting at `bind.value` backwards
   through `produced_by` edges.
2. Collect every `PortId` that appears as an input to any visited
   Behavior (Transform inputs, Branch scrutinee + path outputs,
   Loop source/init, nested Bind value). The root port is added
   too, which covers trivial-body functions whose body is
   literally a parameter port (`fn first(a, b) = a`).
3. Compare each parameter port against the referenced set. Any
   parameter port absent from the set is unused.

Note that the shipped form walks `Bind.value` directly rather
than routing through `ArrowBody::UserDefined` — ArrowBody is a
declaration-level concept; the lens operates at the Bind level
where the body sub-DAG already lives. When post-PR-B work adds
`Bind → Declaration` edges, a `scope: Vec<DeclarationId>` config
field can land with a real implementation that actually honors
the scope restriction — not before.

**Test plan** (as shipped in
`src/v3/compiler/tests/m1_3_lens_unused_parameters_test.rs`):

- Empty result for `fn add(a, b) = a + b` (every param used).
- Single violation at index 1 for `fn first(a, b) = a`.
- All-params violation for `fn always_one(x, y, z) = 1` (constant
  body).
- Zero violations for `let x = 1 + 2` (value binding skipped).
- Branch-body coverage: `fn pick(a, b) = if a > 0 then a else b`
  vs `fn always_a(a, b) = if a > 0 then a else a`.
- Bootstrap baseline test (zero violations against v3's
  bootstrap-loaded std/).
- **Canonical target test.** Lens library spec names
  `content_upsert` in `dsl/std/patterns.dag:136-139` as the known
  concrete finding. v3's M1(3) parser cannot yet handle the literal
  source (anonymous record returns, block bodies, record literals
  in expression position — all class-5 gap follow-ups), so the test
  suite pins both: (a) the literal source fails to parse today,
  and (b) a v3-parseable synthetic equivalent (`fn
  content_upsert(content: Int, path: Int) -> Int = content + 0`)
  produces the expected violation at parameter index 1. When the
  parser grows the missing features, test (a) flips to a positive
  assertion against the literal source. **Rule codified by this
  episode: every lens that names a canonical target in its design
  must have a test that either reaches the literal target or pins
  the parser gap that blocks it.**

---

## §3. Lens application manifest format

Lenses are opt-in via a declarative manifest. The manifest is
itself a `.dag` file (same format as everything else) that
declares which lenses run against which file sets with which
configurations.

### §3.1 Proposed format

```
// dsl/lens_config.dag

module lens_config

// Each application declares:
//   - lens: which lens (name matches a lens in the library)
//   - applies_to: glob patterns identifying the files subject
//   - boundary: the lens's configuration (BoundarySpec or equivalent)
//   - severity: "error" (block PR) or "warning" (surface but don't block)

data compiler_layer_opacity: LensApplication = {
  lens: "layer_opacity"
  applies_to: [
    "src/v3/compiler/src/**/*.rs",
    "src/v3/compiler/src/**/*.dag"
  ]
  boundary: {
    below_boundary_modules: ["dsl/std/**"]
  }
  severity: "error"
}

data extdeps_layer_opacity: LensApplication = {
  lens: "layer_opacity"
  applies_to: ["dsl/extdeps/**/*.dag"]
  boundary: {
    below_boundary_modules: [
      "dsl/extdeps/transports/shell.dag",   // ShellOutputChannel variants
      "dsl/extdeps/llm/anthropic.dag",      // AnthropicModel variants
      "dsl/extdeps/llm/openai.dag"          // OpenAiModel variants
    ]
  }
  severity: "error"
}

data std_duplicates: LensApplication = {
  lens: "structural_duplicates"
  applies_to: ["dsl/std/**/*.dag", "dsl/extdeps/**/*.dag"]
  config: {
    include_types: true
    include_data: true
    ignore_re_exports: true
  }
  severity: "error"
}

data stub_detection: LensApplication = {
  lens: "unused_parameters"
  applies_to: ["dsl/std/**/*.dag", "dsl/extdeps/**/*.dag"]
  // No config fields: lens_unused_parameters ships with a scope
  // restriction only, no name-based filters (see §2.3).
  severity: "warning"
}
```

### §3.2 Interpretation

At CI time (or local invocation), the compiler loads
`dsl/lens_config.dag`, iterates over every `LensApplication`,
runs the named lens on files matching `applies_to`, and collects
violations. Applications at `severity: error` block the build;
applications at `severity: warning` surface in the output but
don't fail.

The manifest is itself a set of `data` declarations, so it is
subject to the same inhabitance checking and structural analysis
as any other data. A bad manifest (wrong lens name, invalid
config shape) fails to load. The manifest is also subject to
the `structural_duplicates` lens — if two applications are
identical, they're flagged as duplicate manifest entries.

### §3.3 Why this shape

- **Data-driven, not code-driven.** The manifest is
  declarative; adding or removing a lens application is a spec
  edit, not a code edit. This is the thesis's "one spec file
  edit per new feature" claim applied to invariant enforcement.
- **Opt-in by default.** A file is only subject to a lens if an
  application explicitly names it. No source file is subject to
  every invariant; each invariant has a declared scope.
- **Parameterized.** The same lens runs with different boundaries
  for different applications. `layer_opacity` applied to
  compiler source uses `boundary = dsl/std/`, applied to a user
  project uses whatever boundary the user declares. One mechanism,
  multiple applications.
- **Self-validating.** The manifest is parsed and checked by the
  compiler itself. A misconfigured application fails at load
  time, not at runtime.

## §4. CI integration

The lens library runs as a CI step on every PR. The integration
is minimal:

```yaml
# .github/workflows/lens_check.yml (or equivalent)

name: Lens Library
on:
  pull_request:
    paths:
      - 'src/**'
      - 'dsl/**'
      - '.github/workflows/lens_check.yml'

jobs:
  run_lens_library:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build v3-compiler
        run: cargo build -p v3-compiler --release
      - name: Run lens library
        run: |
          cargo run --release -p v3-compiler -- \
            lens run --config dsl/lens_config.dag
```

The command `v3-compiler lens run --config <path>` is a new
subcommand that the lens library adds. Implementation:

1. Parse the manifest at `<path>`.
2. For each `LensApplication` in the manifest:
   a. Compile every file in `applies_to` into a `Dag` (one Dag
      per file for isolation, or one Dag for all files if the
      lens needs cross-file visibility).
   b. Run the named lens with the declared configuration.
   c. Collect violations.
3. Emit a summary (JSON + human-readable) of all violations,
   grouped by application and severity.
4. Exit with non-zero status if any error-severity application
   has violations.

The subcommand is ~100 lines of Rust in a new
`src/v3/compiler/src/bin/v3-compiler.rs` module (or wherever the
CLI lives).

## §5. Implementation order

The recommended order for building the initial library:

1. **`lens_unused_parameters`** — simplest (shipped as of PR #445).
   The algorithm is trivial (set difference between declared
   params and referenced ports) and the dataflow it exercises is
   the foundation for more sophisticated lenses later. Catches
   `content_upsert`'s stub via the synthetic-equivalent test; the
   literal target is blocked on three class-5 parser gaps (anon
   record return, block body, record literal in expression position)
   and pinned by a parse-failure test that flips when the parser
   grows those features.
2. **`lens_structural_duplicates`** — next. Simple structural hash
   + collision detection. Expected to catch the
   `FileClassification`/`FileEntry` duplicate, the `wire_contract`
   duplicates, `default_edition` duplication, and probably others.
   Gives immediate value on existing std/ and extdeps/ code.
3. **`lens_layer_opacity`** — third, the most sophisticated of the
   three because it needs the `BoundarySpec` reverse index and
   multiple consumer kinds. The one that validates the thesis's
   layering claim end-to-end.
4. **Manifest parser + CLI subcommand** — fourth. Ties everything
   together. Before this lands, each lens is runnable directly via
   a hand-written test binary; after it lands, the lens library is
   invocable as a CI gate.

Ordering is a recommendation, not a requirement — if one lens is
structurally easier than another given the current substrate, the
order can change. After this ships, the project has:

- Three working structural-invariant lenses
- A declarative application manifest
- CI integration
- The template for every future lens (add a new file,
  implement `query`, declare an application in the manifest)

## §6. Future lenses the library should grow toward

The initial three are the highest-leverage starting point. The
library should grow as new invariants surface. Candidates:

- **`lens_inhabitance`** — walks `data X: T = { ... }`
  declarations and verifies that the body matches T's declared
  fields by name and type. Catches the `mock_response` /
  `GcpErrorShape` / `GitHubErrorShape` mismatches from the
  2026-04-15 reviewer audit. Depends on v3 class-5 gap #3
  (record-literal parsing).
- **`lens_scaffold_boundaries`** — walks every scaffold variant
  declared in the substrate and verifies that each has an
  unreachability gate in user-range code. Replaces R14's
  hand-written `reject_user_unparsed_scaffolds` with a
  systematic check for every scaffold variant.
- **`lens_fail_closed`** — walks every failure path in the
  compiler and verifies that each attaches a diagnostic.
  Catches silent skips and missing diagnostics.
- **`lens_algebra_inhabitance`** — walks type declarations and
  flags any that implement a lattice shape (meet + join) without
  declaring `inhabits Lattice<T>`. Catches the `FermiDepth` and
  `Encoding` manual-lattice duplications.
- **`lens_cross_authority_consistency`** — walks declarations
  of the same conceptual function in multiple language
  authorities, verifies they agree on signature. Catches the
  `String.chars` semantic drift between Rust emit.dag and
  runtime.dag.
- **`lens_grounding`** — walks every declaration's epistemic
  chain and verifies it terminates at primitive roots.
- **`lens_determinism_inputs`** — walks the compiler's own
  source for reads of `HashMap` (non-deterministic iteration)
  or `SystemTime::now` (environment-dependent). Lint-adjacent
  but structural.

Each of these follows the same template as the initial three.
Every future lens should pass the §1.5 canonical-form gate (thin
wrapper, not a deepening Rust-lens scaffold) before landing.

## §7. Relationship to the existing lens infrastructure

v3 already has four reader lenses:

- `lens_provenance.rs` (M0) — reads `produced_by` edges,
  returns the origin of each port.
- `lens_depth.rs` (post-M0) — reads the substrate walker,
  returns port depth.
- `lens_cost.rs` (M1) — reads substrate + language spec,
  returns per-node cost.
- `lens_unused_parameters.rs` (M1, PR #445) — walks function
  body sub-DAGs and reports parameter ports the body never
  references. First library lens to ship; same reader-lens
  template as the three above.

The new lenses in this document follow the same shape: pure
function over `Dag` + configuration, returns structured output.
They're sibling modules to the existing ones and can be added
without touching existing lens code. All of the above are Rust
bootstrap form (§1.5); canonical `.dag` form now awaits only the
runtime invocation path for compiled lenses, not the reflection
surface itself.

The substrate already carries enough information for all three
remaining initial lenses. No substrate changes are required —
the lenses are additive.

## §8. What the lens library is NOT

- **Not a replacement for the type system.** The type system
  catches what it can at compile time. Lenses catch what the
  type system doesn't express. Over time, successful lens
  patterns migrate into the type system (e.g., the `DisplayName`
  refactor that would make layer opacity impossible to violate
  structurally, as mentioned in `INVARIANTS.md` §"Layer opacity").
- **Not a grep replacement for arbitrary source patterns.**
  Lenses operate on the DAG, not on source text. Patterns that
  don't have a structural form (determinism, license headers,
  code style) are lint territory, not lens territory.
- **Not universal.** Each lens has a declared scope. Not every
  file is subject to every lens. The opt-in mechanism is the
  point.
- **Not infallible.** A lens can only enforce a property that
  has a structural form. Properties that require human
  judgment (modeling faithfulness, root-cause depth, forward
  progress) stay review-level.

## §9. Success criteria for this library

The library is successful when:

1. **Each of the three initial lenses has shipped** with its
   test suite green and its expected initial findings
   reproducible in CI.
2. **The manifest parser and CLI subcommand** work end-to-end —
   a developer can run `v3-compiler lens run --config
   dsl/lens_config.dag` locally and see the same output the CI
   sees.
3. **At least one high-value finding is closed by a lens-driven
   fix.** Concrete target: the `FileClassification`/`FileEntry`
   duplicate is surfaced by `lens_structural_duplicates`, fixed
   by merging the two declarations, and the fix's regression
   test is the lens returning empty on subsequent runs.
4. **The existing layer-opacity grep-gate proposal in
   `INVARIANTS.md` is fully superseded** — the §"Layer opacity"
   invariant points at `lens_layer_opacity` as its primary
   enforcement, with the rename test as a regression check.
5. **Adding a fourth lens is measurably cheap** — the next
   invariant that becomes a lens should land in under 2 days
   of work, with no substrate changes. That demonstrates the
   extensibility claim.

## §10. Open questions

1. **Per-file vs per-module compilation for lens input.** Some
   lenses (unused parameters) can run on one file at a time.
   Others (structural duplicates) need cross-file visibility.
   The lens framework should support both; the question is
   whether the manifest declares the scope per application or
   whether the lens declares its own visibility requirements.
2. **Incremental lens execution.** On large repos, running every
   lens against every file on every PR may get slow. The
   framework should support incremental re-run based on which
   files changed. Open design question: how does the lens
   framework know which violations depend on which files?
3. **Lens output format standardization.** Should every lens
   return `Vec<Violation>` with a shared `Violation` trait, or
   should each lens have its own output type? Trade-off between
   uniformity (shared trait, easier CI parsing) and expressiveness
   (per-lens types, richer output).
4. **Manifest validation at lens-library version bumps.** When
   a new lens is added, existing manifests should either
   gracefully ignore references to unknown lenses (with a
   warning) or fail loudly. Open design question per the
   project's general version-drift philosophy.

These are resolvable during implementation. They are not blockers
for starting.

## §11. Relationship to `INVARIANTS.md` and `THESIS.md`

This document is the implementation spec for the enforcement
mechanism that `INVARIANTS.md` §"Layer opacity" and
`THESIS.md` §"Compositional layering" reference. The docs
describe the principle and the invariant; this document describes
how the invariant is enforced.

When the first lens ships, update `INVARIANTS.md` to replace
any remaining "lens-to-be-built" language with a reference to
the shipped lens. Similarly, when the manifest format stabilizes,
update `THESIS.md` to use the final format in its examples.

The document is structured so that each section can evolve
independently: §2 (the three lenses) gets updated as new lenses
are added to the library; §6 (future lenses) gets items moved
to §2 as they're implemented; §10 (open questions) gets items
deleted as the questions are resolved.

---

**This document is a design spec, not a ship target.** Each
section is written to be small enough that an implementer on
`free-cod-972` can pick up any lens and build it without needing
to read the entire document. The implementation order in §5 is
a recommendation, not a requirement — if one lens is easier than
another to land first, the order can change.
