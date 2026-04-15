# Substrate Reflection Design

> Part of: [THESIS.md](../THESIS.md), [INVARIANTS.md](../INVARIANTS.md),
> [lens-library-design.md](lens-library-design.md)
>
> **Purpose:** make v3's substrate self-describing. Declare
> substrate types (`Dag`, `Node`, `Behavior`, `Declaration`, …)
> as ordinary substrate-language declarations, expose query
> primitives that let a `.dag` program walk compiled substrate
> data, and migrate the existing lenses from Rust bootstrap
> form to their canonical `.dag` form. After this work, adding
> a new lens is a new `.dag` file — no Rust, no compiler edits,
> no `kernel_lens_set` forming.
>
> **File locations.** Per `src/v3/compiler/src/bootstrap.rs`,
> v3-only fixture files live in `src/v3/std/` and `src/v3/spec/`
> during the v2→v3 transition window, not in `dsl/std/`. v2's
> CI pipeline scans `dsl/` recursively and cannot parse v3's
> substrate types. The canonical logical home for these files
> is `dsl/std/`; staged location is `src/v3/std/`. See §3.1.

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

**Reflection is the prerequisite for anything that reads
substrate-about-substrate.** That's a bigger set than "lenses"
and the framing matters. The earlier draft of this doc pitched
reflection as "it enables canonical lenses," which is about 40%
of what the work actually buys. The other 60% is schema
migration: a program that compares two versions of the
substrate's declarations and emits a structural diff so
compatibility changes can be applied mechanically instead of
patched by hand. Both use cases — lenses AND schema diffs — are
consumers of the same primitive (a `.dag` program that reads
substrate declarations as data). Anything else that needs to
observe the substrate's own shape (cross-version regression
detection, automated type-migration bots, substrate-aware
refactoring tools) joins the same set.

**The two near-term consumers are not on the same critical
path, but they are the same mechanism.**

1. **Lenses as `.dag` programs** — the structural
   invariant-enforcement primitive. `lens_unused_parameters`
   migrates first as the proof point. The 6500+ lines of v2
   analysis code in §1.5 follow as L2 consumer migrations.
2. **Schema migration as `.dag` programs** — the mechanism by
   which v3 eliminates manual stage0 edits when the substrate
   schema changes. Per `SELF_HOSTING.md` §11: the schema-diff
   lens reads old and new substrate declarations, classifies
   each delta into a `ChangeKind` (AddField, AddVariant, Rename,
   RemoveField, etc.), and emits a structural patch over Rust
   source. The patch applier is a follow-up PR after reflection.

**The concrete failure mode this work prevents — for lenses.**
The four lenses that ship today (`lens_provenance`, `lens_depth`,
`lens_cost`, `lens_unused_parameters`) are all Rust modules.
Every new lens added as a Rust module grows a registry of known
lens names, a dispatch pattern keyed on those names, and shared
Rust-lens infrastructure. That is the same failure class as
v2's `kernel_type_set` — a name-roster leak at a different
layer. The project calls this the **`kernel_lens_set`** failure:
by the time you have five or six Rust lenses, the compiler has
a hardcoded list of known lens names and a special handler per
lens, and migrating them to `.dag` is a meta-refactor. The rule
from [`lens-library-design.md`](lens-library-design.md) §1.5 —
"stop and build the reflection primitive before the scaffold
ossifies" — is this work's charter.

**The concrete failure mode this work prevents — for schema
migration.** v2's bootstrap pain is documented: manual stage0
edits to bridge TwoPhase changes, fixed-point failures that
report at the wrong layer, `bootstrap.dag` with 195 lines of
decorative modeling and zero consumers. The structural answer
is a schema-diff lens + a patch generator, both as `.dag`
programs over the reflection surface. Without reflection, they
can't be written — the diff lens needs to walk two Dag values
from two substrate versions and compare them structurally,
which requires substrate declarations to be data. Reflection
is the unblocker.

**Composability was the user-facing framing.** The third reason
this matters: a `.dag` lens can be analyzed by another `.dag`
lens (including itself). A Rust lens cannot — analyzing it would
require a meta-language separate from the substrate. Once lenses
live in the substrate, `lens_unused_parameters` can run against
its own source, `lens_layer_opacity` can check that other lenses
don't dispatch on names, `lens_cost` can estimate the cost of
running a lens, and a schema-diff lens can run against the
substrate's own declaration set. This is the "DAG that looks
inward" the user has been pointing at.

**The fourth reason: cost of change.** The thesis's load-bearing
metric is "one file edit per new feature." For lenses, the
current metric is: new Rust file + build graph edit + test module
registration + documentation update. That's a four-file minimum.
In canonical form, adding a new lens is one `.dag` file loaded
by the manifest. One file. For schema migration, the current
metric is: hand-edit stage0 + hand-write a bridge + regenerate
+ debug the fixed-point failure by hand. In canonical form,
schema change becomes: write the new substrate declaration,
run the schema-diff lens, apply the generated patch, verify
the fixed point. Zero manual steps. Both are thesis targets.

## §1.5 The motivating consumers already exist

Reflection is not a decorative framework in search of users. It
is the mechanism by which **6500+ lines of existing v2 analysis
code plus the entire schema-migration pain class** become
structural operations in v3. The consumers are already written
(or, in the schema-migration case, already modeled in v2's
`bootstrap.dag` but without working implementations). They are
waiting for the substrate to host them.

**Consumer class A — v2 analyses that migrate to `.dag` lenses:**

| v2 consumer | Lines | What it does | v3 current state | Migration target |
|---|---|---|---|---|
| `src/v2/complexity.dag` | **5490** | Symbolic cost terms (work + span), termination proofs, descent evidence, iteration dimensions, cost shapes. v2's most mature analysis. | **`lens_cost.rs` at 80 lines** — a placeholder that counts structural Transform / Branch / Loop ops. Not complexity; just a first data point on the "lens lands in tens of lines" curve. | `dsl/lenses/complexity.dag` — **a port of the algorithm**, not a rewrite. The shape stays; the substrate changes. |
| `src/v2/ownership.dag` | 719 | Move / borrow / clone inference, lifetime tracking, ref-counting decisions. Feeds directly into Rust emission. | Nothing in v3. | `dsl/lenses/ownership.dag` |
| `src/v2/effect_derivation.dag` | 66 | Pure vs effectful function classification. Experiment 4 from the v3 validation experiments shipped a prototype version. | Nothing in v3 beyond the experiment. | `dsl/lenses/effects.dag` |
| `src/v2/trace.dag` | 223 | Execution trace extraction for debugging. | Nothing in v3. | `dsl/lenses/trace.dag` |

**Consumer class B — schema migration as a structural operation:**

| v2 consumer | Status in v2 | Purpose | v3 equivalent | Migration target |
|---|---|---|---|---|
| `dsl/gunbc/bootstrap.dag` types (`CompilerStage`, `TransformContract`, `ChangeClassification`, `BootstrapStrategy`, `FixedPointCheck`, `FieldPropagation`) | **Declared but unused.** 195 lines of vocabulary with zero consumers wired into the actual regen flow. | The *model* exists; the *dissolution* never happened. v2 ships with the blueprint for fixing its own self-hosting pain, not wired. | `dsl/lenses/schema_diff.dag` + `dsl/lenses/schema_patch.dag` + a driver in `regen.dag` | **Schema-diff lens** reads two substrate versions and produces a `ChangeClassification`; **schema-patch lens** reads the classification and emits a structural Rust-source patch; **regen driver** composes them. All three are `.dag` programs over the reflection surface. See `src/v3/SELF_HOSTING.md` §11. |

**The Consumer B class is the second 60% of reflection's value.**
The lens-migration story (class A) is about 40% of what
reflection buys because the four v2 analyses already work —
they just currently live in v2. Reflection moves them without
losing behavior. **Schema migration (class B) is new capability**:
v2 has the pain, has the model, does NOT have the dissolution.
Reflection is the first time gunbc can actually wire the
bootstrap model to its consumers.

**Both consumer classes are tests of the same claim.** The
class A migrations (complexity et al.) test "physics plus lens,
zero heuristics" on analyses. The class B schema-migration work
tests the same claim on the compiler's own bootstrap loop. Every
v2 `annotate_*` helper that dissolves in class A is a win; every
manual stage0 edit that becomes a structural operation in
class B is the same win for a different consumer. Both are
Experiment 2 at scope.

**The critical framing: `lens_cost.rs` (80 lines) is v3's
placeholder for `complexity.dag` (5490 lines).** The reflection
framework's job is to be the migration path. When complexity
lands as `dsl/lenses/complexity.dag`, it should read substrate
facts v2 currently reconstructs — termination evidence via
descent edges, iteration dimensions via Cardinality, cost
shapes via algebra-inhabitance — instead of re-deriving them.
**Every heuristic in v2's complexity that dissolves into a v3
substrate field is a physics-plus-lens win.** Every one that
doesn't dissolve reveals a substrate gap that a prerequisite PR
has to close.

This is a direct test of Experiment 2 from
`docs/v3-validation-experiments.md` at full scope. Experiment 2
validated "carry facts through bindings" for one reconstruction
pattern (`classify_let_value`). Migrating complexity to a v3
lens is the same experiment at 5490-line scope: every
reconstruction in v2's complexity.dag either (a) dissolves into
a v3 substrate field that the lens reads directly, or (b) reveals
a substrate gap that blocks the migration until the gap closes.
Both outcomes are valuable; either way the migration is forward
progress.

**What this means for scope.** The reflection PR itself
(migrating `lens_unused_parameters` as the proof point) is the
smallest end-to-end demonstration that the framework works. The
**motivating consumers** — complexity, ownership, effects,
trace — are scoped as follow-up work, but they are not optional.
Each one is tracked as an explicit milestone in §11.5, not as
"future ideas." The framework exists because they exist;
shipping reflection without a plan for their migration would be
exactly the "decorative lens library" the critique names.

**The "new consumers" critique answered.** The codex and
ChatGPT reviews of earlier rounds flagged "no new consumers —
this is accumulating debt, not retiring it." The structural
answer is: **the consumers are not new. They are existing v2
code that this framework is specifically designed to host.** A
PR that ships reflection without at least one consumer migration
is incomplete; a PR that ships the framework plus the
`lens_unused_parameters` migration plus a scoped migration plan
for complexity/ownership/effects/trace is retiring debt.

---

## §2. Scope of this document

**In scope:**

1. Moving the substrate's structural type layer into
   `src/v3/std/substrate.dag` (a regular `std/` file, not a
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
  — new `.dag` files in `src/v3/std/` (bootstrap staging), new
  Rust query module, extended `emit_rust` path. No new
  `TypeConnective` variants, no new `Behavior` variants, no new
  `AtomPayload` variants.
- Any change to the `lens_unused_parameters` algorithm. Same
  walk, same output, different host language.

---

## §3. The reflection surface

The reflection surface is the set of `.dag` declarations a lens
needs in order to express its walk over the substrate. **There
are no "query primitives" as a separate category.** If the
substrate is declared correctly — every type is a record or a
sum, every field is addressable by name — then reading those
fields IS the query. A lens is just a normal `.dag` function
that takes a `Dag` value as input and reads its fields, the same
way any other `.dag` function reads fields on a record parameter.

**Where "query primitives" came from and why they're dissolved.**
An earlier draft of this design doc proposed a separate
`substrate_query.dag` file with functions like `fn declarations
(d: Dag) -> List<Declaration>` as explicit query primitives.
Those functions were re-declaring the structural fields of `Dag`
under different names — a parallel representation of data that
already exists on the declared record. That's exactly the
failure class everything else in this project is trying to
prevent: naming the same fact twice in two different places.
The cleaner model: declare `Dag` once, with its fields, and let
every consumer read those fields directly. This section reflects
the cleaner model.

**What the "query primitive layer" collapse reveals.** When you
try to write `lens_unused_parameters` as a `.dag` function that
reads fields on a `Dag` argument, several grammar features turn
out to be prerequisites: field access on local variables,
pattern matching with payload binding, higher-order function
calls (for `fold`/`map`/`filter`), lambda expressions, and a
`List<T>` standard library. These are parser/lowering extensions
that don't yet exist in v3's M1(3) grammar. §11 of this document
enumerates them as a prerequisite slate — each one is its own
PR, reflection lands after the slate is complete. The reason
for this ordering is **architectural honesty**: collapsing the
query-primitive layer into field access is only meaningful if
the grammar actually supports field access. Without the
prerequisites, we'd either ship a compromised design (the
query-primitive scaffold we just rejected) or ship a design
that lies about being compositional.

### §3.0 Seed minimality as an invariant

The bootstrap seed is deliberately narrow. **The seed is the
parser plus the realization mechanism**, NOT a Rust-only type
registry. Every type that appears in a `.dag` file is declared
in some `.dag` file — including the substrate's own types. What
makes an "atomic identity handle" special is that its declaration
is **minimal** (no fields, no variants), and its authority lives
in a realization entry that binds it to a Rust backing type.

The seed by component:

- **The parser.** Written in Rust. Produces `SurfaceItem` /
  `SurfaceExpr` trees from `.dag` source text. This is the only
  thing that has to exist before any `.dag` file parses.
- **The realization mechanism.** Already shipped for the
  arithmetic primitives. A `.dag` declaration binds to a Rust
  backing type via a `TypeRealization` entry in `rust.dag`.
- **The inhabitance / resolve sweep.** `resolve_pending_identifiers`
  + the forward-reference resolution v3 already has working for
  `std/algebra.dag → std/types.dag::Bool`. Handles the circular
  references in the substrate's self-description.

**Atomic identity handles as minimal declarations.** `NodeId`,
`PortId`, `DeclarationId` are declared in `src/v3/std/substrate.dag`
as opaque atoms (no RHS, no fields, no variants) and each has
a `TypeRealization` entry in `src/v3/spec/rust.dag` binding it
to a Rust newtype. The declaration form is necessary so name
resolution can find the identifier when it appears elsewhere
in substrate.dag (e.g., `fn port_producer(d: Dag, p: PortId)
-> NodeId?`). The declaration form is minimal because `.dag`
programs cannot pattern-match on opaque atoms — they can only
compare for equality and pass them around.

`SourceSpan` is already declared in `std/types.dag` as a record
and is reused — no new declaration in `substrate.dag`.

`Int`, `Bool`, `String` are already declared in the existing
`std/` fixtures and are reused unchanged.

**Everything else is a structural `.dag` declaration.** `Dag`,
`Declaration`, `TypeConnective`, `ConjField`, `ArrowBody`,
`Behavior`, `AtomPayload`, `CardinalityBound`, `TemplateArgument`,
and every other substrate type lives as an ordinary `Conj` /
`Disj` declaration in `substrate.dag`. The Rust structs in
`src/v3/compiler/src/dag.rs` are **realizations** of these
declarations — kept consistent by an inhabitance check, not
by parallel representation.

**The invariant this PR codifies** (new `INVARIANTS.md` entry —
see §7.2): every type that *could* be declared as a structural
`Conj` / `Disj` in `.dag` MUST be declared that way. The only
permitted "minimal-declaration" forms are opaque atomic identity
handles that programs never structurally inspect. New substrate
types that ship as "Rust-only primitives with no `.dag`
declaration" are a **fail-closed blocker**; new minimal-atom
declarations with realization backing are permitted but the
count is ratcheted downward over time.

**The ratchet.** A CI check counts minimal-atom declarations in
`substrate.dag` that exist only as name targets for realization
entries. The count is stored in a ratchet file (same mechanism
as the `Pending` ratchet) and must monotonically decrease or
stay flat. A PR that adds a new minimal-atom declaration fails
CI unless it also deletes an equal or greater count of existing
atoms, OR the new atom meets the narrow "opaque identity handle
programs never inspect" exception and the PR inline-receipts
that justification.

**Why the ceiling matters.** The thesis's self-hosting claim
("the project is a meta-compiler — the substrate can describe
any computational system including itself") only holds if the
substrate actually hosts itself. Every minimal-atom declaration
is a fact `.dag` cannot structurally analyze — a lens can walk
it by equality but not by field shape. A substrate with five
opaque atoms is a substrate with five blind spots; one with
zero has none. The direction is toward zero, and the seed
ratchet ensures we don't drift the other way under pressure.

**How this answers the bootstrap question.** The circular
dependency `Dag → Declaration → TypeConnective → Declaration` is
the same shape as `OrderedRing<T> → Ring<T> → Monoid<T>`. v3
resolves the latter today via `resolve_pending_identifiers`.
Applying the same mechanism to substrate-type declarations is
zero new bootstrap machinery — it's one more file in the
`include_str!`-based `V3_SPECS` list that `src/v3/compiler/
src/bootstrap.rs` already uses for `rust.dag` and `v3_l1.dag`.

### §3.1 Substrate types as `.dag` declarations

New file: `src/v3/std/substrate.dag`. Sibling to the existing
`src/v3/spec/rust.dag` and `src/v3/spec/v3_l1.dag` (both of which
live outside `dsl/` for the same reason). Loaded by v3's bootstrap
via the build-script-generated `V3_SPECS` include list, parsed in
dependency order after the bootstrap `std/` fixtures.

**Why not `dsl/std/substrate.dag`?** Per `src/v3/compiler/src/
bootstrap.rs:58-66`: v2's CI pipeline scans `dsl/` recursively
and attempts to resolve every identifier in every record-literal
field. v2 doesn't know about v3's substrate types
(`TypeConnective`, `Behavior`, `NodeId`, etc.) and would flag
every reference in `substrate.dag` as an undefined-variable
error. Keeping v3-only spec files outside the v2-scanned tree
is the existing pattern for `rust.dag` and `v3_l1.dag`; it
applies identically to `substrate.dag`.

**Canonical-home note.** The canonical logical home for
`substrate.dag` — once v2 retires — is `dsl/std/substrate.dag`
or similar, sibling to `dsl/std/algebra.dag` and friends. The
`src/v3/std/` location is bootstrap staging for the v2→v3
transition window. Same shape as `src/v3/spec/rust.dag` staging
for the eventual `dsl/extdeps/languages/rust.dag` canonical
location. The `THESIS.md` §"Two groundings" bootstrap-staging
note applies to both files.

```
module std.substrate

// Substrate types — declared here, realized by the Rust structs
// in src/v3/compiler/src/dag.rs, kept consistent by the inhabitance
// check that runs at bootstrap time.

// Atomic identity handles. These are declared but minimal — no
// fields, no variants. The declarations exist ONLY so that name
// resolution can find them when they appear in type expressions
// elsewhere in this file. The authoritative backing is the
// TypeRealization entry in src/v3/spec/rust.dag that binds each
// atom to its Rust newtype. A .dag lens can pass these values
// around and compare for equality; it CANNOT pattern-match on
// them because they have no structure. The substrate-honest
// pattern: identity without structure.
type NodeId
type PortId
type DeclarationId
// SourceSpan already declared in std/types.dag — reused, not
// redeclared.

// Type connective — six-way sum matching Rust's TypeConnective.
type TypeConnective
  = Atom(AtomPayload)
  | Conj { fields: List<ConjField> }
  | Disj { variants: List<Declaration> }
  | Arrow { inputs: List<DeclarationId>, output: DeclarationId, body: ArrowBody }
  | Cardinality(CardinalityBound, DeclarationId)
  | Instantiation { template: DeclarationId, args: List<TemplateArgument> }

// ConjField — a named field in a Conj. Named this way to avoid
// a collision with algebra.dag's `Field<T>` (the mathematical
// field / division ring, a wholly different concept).
type ConjField {
  name: String
  declaration: DeclarationId
}

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

**On opaque atoms being declarations, not Rust-only primitives.**
An earlier draft of this section said "there is no `type NodeId`
in `.dag`." That framing was wrong. For any identifier to resolve
at parse time, the substrate needs SOMETHING for name resolution
to bind against. The `Int` precedent is authoritative: `type Int`
is a `.dag` declaration in `integer.dag` even though the Rust
backing is `i64`; the realization in `rust.dag` is what connects
the two. `NodeId`/`PortId`/`DeclarationId` follow the same
pattern — they are declared in `substrate.dag` as minimal atoms
(no fields, no variants), and their realization in `rust.dag`
binds each to its Rust newtype. The "seed" is the parser plus
the realization mechanism, not a Rust-only type registry.

**Name collisions with existing v3 code:**

- **`type Declaration` in `src/v3/spec/v3_l1.dag`** — an empty
  `Conj` sentinel used by `lower.rs` and `rust.dag` as a type
  marker for "this field carries a typed reference to another
  declaration." Collides with `substrate.dag`'s central
  `Declaration` record type. **Resolution:** rename the sentinel
  → `DeclarationRef` (~10 mechanical edits across `lower.rs`,
  `dag.rs`, `rust.dag`, and a test). The substrate's central
  record type keeps the natural name `Declaration`.
- **`type Field<T>` in `dsl/std/algebra.dag:197`** — the
  mathematical field (division ring), completely unrelated to
  record fields. **Resolution:** substrate's record-field type
  is named `ConjField` above to avoid the collision. Algebra's
  `Field<T>` keeps its correct name.

**Inhabitance check.** A post-bootstrap pass walks every
declaration in `std.substrate` and asserts the corresponding
Rust type satisfies the declared shape. For `type Dag`, the check
walks `Dag`'s Rust fields and asserts they match the declared
fields by name and type. Drift between the two representations
becomes a fail-closed diagnostic at bootstrap time. This is the
same inhabitance mechanism `src/v3/spec/rust.dag` uses today to
bind `.dag Int → Rust i64` — the only new thing is applying it
to the compiler's own struct types.

### §3.2 Queries ARE field access

**There is no `substrate_query.dag` file.** Queries are not
primitives. If `Dag` is declared with a `nodes: List<Behavior>`
field, then a lens writes `d.nodes` directly — the substrate
already carries the fact, no separate accessor function is
needed. The earlier draft of this section proposed an explicit
query-primitive layer (`fn declarations(d: Dag) -> List
<Declaration>`, etc.); those functions were all re-declarations
of fields on already-declared records. They were the scaffold,
not the model.

**What a lens ACTUALLY needs from the substrate** (assuming the
grammar catches up — see §11 for the prerequisite slate):

1. **Field access on local variables.** `d.nodes`, `bind.params`,
   `behavior.span`. Today v3 parses `Ident.Ident.…` as
   `SurfaceExpr::Path` but only lowers it for declaration
   references, not for field-reads on local variables. **Prereq 1**
   in §11 — small lowering extension.
2. **Pattern matching with payload binding.** `match b { Bind
   (bind) => …, Transform(t) => … }`. Today v3 only supports
   `BareVariant` patterns that don't capture payloads. **Prereq 2**
   in §11 — adds `SurfacePattern::VariantWith`.
3. **Higher-order function calls.** `fold` takes a `fn(U, T) -> U`
   parameter and calls it inside its own body — `f(acc, head)`
   where `f` is a parameter. Today v3's `TransformTarget::Callable
   (DeclarationId)` requires the call target to be a declaration,
   not a local port carrying a function value. **Prereq 0** in
   §11 — the deepest substrate question. See §3.5 for the three
   options.
4. **Lambda expressions.** `|acc, x| acc + x`. Today v3 has no
   lambda surface form. Per v3-spec.md §Principle 5 and
   Experiment 1, lambdas lower to ordinary `Bind` declarations
   with captures as explicit additional parameters (no special
   "capture" concept). **Prereq 3** in §11 — parser + lowering
   extension.
5. **`List<T>` standard library.** `fold`, `map`, `filter`,
   `length`, `enumerate`, `contains`. `List<T>` is modeled as
   the **free monoid on T** (a Disj of `Empty | Cons(T, List<T>)`
   inhabiting `Monoid<List<T>>`). See `src/v3/std/list.dag` — the
   file has been committed as a design artifact alongside this
   doc; it ships once **prereqs 0, 2, 3** land. **Prereq 4** in
   §11 — declaration only, depends on 0, 2, 3.

**All five prereqs are parser/lowering/std extensions, NOT new
substrate variants.** The v3 substrate is already expressive
enough to express lenses; it just doesn't have the surface
grammar or standard library yet.

**The `behavior_output_port` question — resolved compositionally.**
The ChatGPT review of an earlier `lens_unused_parameters` iteration
flagged that a five-arm match over all `Behavior` variants,
returning "the primary result port," was the start of a canonical
substrate query — if duplicated across consumers, it becomes a
`lens_authority_drift` leak in the same family as `kernel_type_set`.

The earlier draft of this doc answered the concern by adding
`behavior_output_port` as a query primitive. With the compositional
model, there's a cleaner answer: **make the primary-result port a
declared field on every `Behavior` variant.**

Instead of:
```
type Behavior
  = Value(ValueNode)
  | Transform(TransformNode)
  | Branch(BranchNode)
  | Loop(LoopNode)
  | Bind(BindNode)
```

…each variant's payload carries `result_port: PortId` as a field:

```
type ValueNode { result_port: PortId, … }
type TransformNode { result_port: PortId, … }
type BranchNode { result_port: PortId, … }
type LoopNode { result_port: PortId, … }
type BindNode { result_port: PortId, … }
```

A lens that needs the primary-result port reads `b.result_port`
after pattern-matching the variant — just field access. No helper
function. No five-arm match. No drift risk, because any consumer
that re-implements the mapping is just re-reading a field that
already exists on the declared record.

**This is an inhabitance constraint on the Rust substrate.** The
Rust `BindNode`/`TransformNode`/etc. structs all already have a
field carrying the primary result port (`bind.value: PortId`,
`transform.output: PortId`, etc.). The constraint that comes out
of the compositional model is: **the inhabitance check MUST
verify that every Behavior variant's payload has a reachable
`result_port: PortId` edge**, either as a direct field or as a
structurally-derivable one. If Rust's `BindNode` has `value:
PortId` and the `.dag` `BindNode` declaration names it
`result_port`, the inhabitance check either (a) fails with a
field-mismatch diagnostic, pushing Rust to rename to `result_port`,
or (b) the `.dag` declaration uses a structural alias matching
the Rust name. The cleanup direction is "rename the Rust field to
match the `.dag` declaration's canonical name" — one-time
structural rename, no runtime cost.

**Why this answer is strictly better than the query-primitive
answer.** A query primitive `fn behavior_output_port(b: Behavior)
-> PortId` is a function that is definitionally `b.result_port`
under a different name. Anyone can inline it, and every inlining
is a parallel representation. A declared field, in contrast, is
the authority — there's nothing to inline because there's no
alternative representation. "Field access on a declared record"
is exactly the shape of authority the substrate already uses
for everything else.

### §3.3 Lens output type

A lens returns a value of its own lens-specific output type. For
`lens_unused_parameters`, that's `List<UnusedParameter>`; for
`lens_cost`, a per-node cost structure; for `lens_provenance`, a
per-port origin map; for `lens_structural_duplicates`, a list of
duplicates. The `.dag` form declares the output type as a `Conj`
(or `Disj`, or `List<Conj>`) in the lens's own module:

```
// src/v3/lenses/unused_parameters.dag
// (bootstrap staging — canonical home is dsl/lenses/ post-v2)

module lenses.unused_parameters

import std.substrate { Dag, Behavior, BindNode, NodeId, PortId, SourceSpan }
import std.list { List, fold, filter, map, enumerate }

type UnusedParameter {
  function: NodeId
  parameter: PortId
  parameter_index: Int
  function_span: SourceSpan
}

fn check(d: Dag) -> List<UnusedParameter> =
  // See §5 below for the full compositional form.
  // (Uses field access + pattern match + higher-order calls.)
  ...
```

The returned `List<UnusedParameter>` is compiled to a Rust
`Vec<UnusedParameter>` where the Rust struct is derived from the
`.dag` record declaration — same mechanism PR #445 uses for data
bodies via `ValueBody::Structural`. No new substrate paths; the
existing record-emission path handles the lens return type the
same way it handles any other record in return position.

### §3.5 The deepest substrate question: higher-order function calls

**Problem.** `fold`, `map`, `filter`, and every other compositional
list operation takes a function-typed parameter and **calls** it
inside its body. For example:

```
fn fold<T, U>(list: List<T>, init: U, f: fn(U, T) -> U) -> U =
  match list {
    Empty            => init
    Cons(head, tail) => fold(tail, f(init, head), f)
  }
```

The call `f(init, head)` — where `f` is a parameter, not a
declared function — does not fit v3's current substrate shape.
`TransformTarget::Callable(DeclarationId)` requires the call
target to be a declaration with a known `DeclarationId`. When
`f` is a parameter, `f` is carried by a PortId inside the Bind's
body, not a DeclarationId.

This question is **not specific to reflection** — it's a general
property of higher-order functions. It shows up the moment any
`.dag` program takes a function as a parameter and calls it.
Reflection's lens layer happens to need it because lenses walk
lists, but `lens_complexity` / `lens_ownership` / any future
collection-walking lens hits the same wall.

**Three options.** Each has a different substrate/lowering cost.

**Option 1a — Substrate extension: `TransformTarget::PortValue
(PortId)`.** Add a new `TransformTarget` variant so `Transform`
can dispatch on a function value carried by a port. The port's
type is an `Arrow` connective, and at emit time the call becomes
an indirect call through whatever the port carries.

- **Pro:** clean, direct, matches the thesis's "functions are
  first-class values" claim. A lens walking a Transform with
  `TransformTarget::PortValue(p)` sees "this Transform calls
  whatever function the port `p` carries" — structurally
  honest.
- **Con:** new substrate variant. Must pass `INVARIANTS.md`
  §"Scaffold boundaries" and the §8.10 substrate-extension
  audit. The thesis explicitly says terminal forms shouldn't
  grow casually.
- **Pro on the con:** the 4-pattern dissolution check passes
  straightforwardly. Pattern 1 (fact placement) — the fact "call
  a port-carried function" doesn't fit anywhere else. Pattern 2
  (variant-is-data) — the call target IS the variant
  distinction, not data on a common shape. Pattern 3 (algebraic
  form) — Callable vs PortValue is dispatch-on-origin, not
  dispatch-on-algebra. Pattern 4 (dimensional) — the two
  variants are categorically different (static declaration
  reference vs runtime value). Verdict: **TERMINAL.**

**Option 1b — Monomorphization at lowering time.** No substrate
change. Every call site where a function is passed as an
argument gets specialized into a declaration. `filter(xs,
is_bind)` at the call site generates `filter_is_bind(xs)` as a
specialized declaration with `is_bind` inlined. The generated
declaration is a `Transform` with `TransformTarget::Callable`
pointing at the real `is_bind` declaration.

- **Pro:** zero substrate change. Uses existing mechanism.
- **Pro:** no runtime indirect calls — every call resolves to a
  specific declaration at compile time. Rust-idiomatic.
- **Con:** substrate grows by the specialization factor. Every
  unique `(filter, predicate)` pair becomes a distinct
  declaration in the Dag. Three calls to `filter` with three
  different predicates produce three `filter_*` specializations.
- **Con:** a lens analyzing "what does `filter` do" sees many
  `filter_*` variants instead of one canonical `filter`. That's
  a layering opacity leak — the specialization is a compiler
  artifact, not a program fact.
- **Con:** higher-order functions with function-valued returns
  are harder to express — you'd need to materialize the return
  value as a specialized declaration too, which cascades.

**Option 1c — Template instantiation extension.** Extend
`TemplateArgument` (which v3 already uses for type parameters)
to bind function-typed parameters alongside type parameters.
When `filter<T, P>` is called with a specific predicate, the
Instantiation carries `T := Behavior, P := is_bind` as template
arguments. Inference walks the Instantiation at the call site
and substitutes the function reference the same way it
substitutes a type parameter.

- **Pro:** reuses existing Instantiation mechanism — no new
  substrate variant, no parallel dispatch path. The thesis's
  "generics and higher-order functions share one mechanism"
  claim lands directly.
- **Pro:** a lens analyzing `filter<T, P>` sees ONE canonical
  declaration with two template slots, one type, one function.
  No specialization sprawl. Template instantiation at call
  sites is a structural fact, not a compiler artifact.
- **Pro on thesis alignment:** `TemplateArgument` already
  carries `parameter: DeclarationId` and a value slot. Extending
  the value slot to accept function-typed arguments (as
  `DeclarationId` references to declared functions) is a
  minimal change to the existing field, not a new variant.
- **Con:** requires `TemplateArgument`'s value slot to accept a
  union of (literal constant, function reference) or to treat
  function references as first-class template values. The
  inference walker needs a new case: when walking an Arrow with
  a function-typed parameter, look up the template binding
  instead of the parameter's port.
- **Con:** when a function parameter is **not** at a template
  instantiation boundary (e.g., a callback passed through
  several layers of functions), the template machinery may not
  have a way to thread the binding. This case needs more
  thought.

**Decision: 1c.** Locked in. Three reasons:

1. **Thesis alignment.** The substrate already treats template
   arguments as "the parameter is filled in at the call site
   with a specific value." Function parameters naturally fit
   that frame — the value is a `DeclarationId` of the function
   being passed. Reusing Instantiation makes "generics" and
   "higher-order functions" share one substrate mechanism,
   which is the thesis's compression principle.
2. **No layering opacity leak.** Option 1b (monomorphization)
   creates compiler-artifact declarations that aren't
   program-visible facts. A lens walking them has to know they
   came from specialization — which is exactly the kind of
   "walk-the-compiler-internal-state" the thesis forbids.
3. **No substrate variant addition.** Option 1a is clean but
   adds `TransformTarget::PortValue`, which has to pass the
   §8.10 audit. More importantly, 1a enables runtime function
   values — a feature v3 doesn't need at the substrate level
   because every higher-order call in lenses is compile-time
   monomorphizable. Adding a substrate variant for a feature
   we don't need is accumulating a scaffold.

**The call-chain threading question is answered by SubstStack
propagation.** When a higher-order function like `fn twice<T>
(x: T, f: fn(T) -> T) -> T = apply(apply(x, f), f)` is
instantiated with `twice[T := Int, f := negate]`, the inner
calls to `apply(x, f)` inherit the outer instantiation's
`SubstStack` frame. Inside `apply`'s body, `f(x)` resolves
through the SubstStack chain: `apply.f → twice.f → negate`,
arriving at `negate(x)` as a concrete Transform with
`TransformTarget::Callable(negate_decl_id)`. This is the same
substitution machinery v3 already uses for type-parameter
threading; extending it to function-typed parameters is a
SubstStack-frame addition, not new substrate.

**Prereq 0 codifies this as the implementation target.** The
implementation extends `TemplateArgument`'s value slot to
accept function references (as `DeclarationId`), extends the
inference walker to resolve function parameters through
SubstStack instead of through Transform input lookup, and
leaves Transform's existing `TransformTarget::Callable` shape
unchanged. No substrate variant growth.

---

### §3.6 The second substrate question: field access as projection

**Problem.** A lens walking the substrate needs to read
children of structural records: `decl.nodes`, `node.params`,
`port.value_type`. The surface form is dotted-path field
access (`p.first`). The semantic is: given a port of Conj type,
extract the labeled child. Every lens writeable in `.dag` hits
this on its first line.

This looks like "pure lowering extension" — the parser already
produces `Path` for `a.b.c`, so lowering just needs to route
field access somewhere. **That framing is the smuggling
claim.** It assumes the substrate already expresses "project a
labeled child of a Conj" somewhere. §11's initial Prereq 1
write took this at face value without naming the substrate
element. PR #453 is the implementation hitting what the design
should have hit — every lowering route attempted there reduces
to one of four substrate-smuggling patterns, because no clean
existing form exists.

**The ground-truth check.** The thesis §"Structural
decompression" suggests "FieldAccess = Product elimination,
from `std/algebra.dag`." That framing is sloppy. Product
elimination (fst/snd, labeled projection) is intrinsic to
products via the universal mapping property — it's a property
of the shape itself, not an operation a type gains by
inhabiting an algebra. algebra.dag is where types gain
*user-facing* operations via inhabitance (Int inhabits
OrderedRing → gets `+`/`−`/`×`). Structural projection on
Conj isn't gained by inhabitance; it's intrinsic to being a
Conj. Verifying `dsl/std/algebra.dag` directly confirms the
drift: there is no general Conj-projection operation in
algebra.dag. What exists is `FreeMonoid<T>.fold` (list-specific
catamorphism, used by Prereq 4's `list.dag`), not a shared
product eliminator.

**Conclusion:** the substrate target for field access cannot
be "call an algebra.dag eliminator," because the premise is
false and conceptually miscategorized. The question is how
projection is expressed in the substrate itself.

**Five options.** Each tagged with its smuggling route
(none / #1 synthesized declarations / #2 flat coproduct
growth / #3 surface state preserved / #4 parallel
representation).

**Option 1a — New `TransformTarget::FieldProject` variant.**
Add a TransformTarget variant so `Transform` can dispatch on
projection. Shape:

```
TransformTarget
  = Callable(DeclarationId)
  | FieldProject { parent_type: DeclarationId, field_label: String }
  | Operator(OperatorKind)  // existing scaffold
```

- **Smuggling route: none.** Genuine substrate extension; the
  compiler sees "this Transform is a projection" as a typed
  variant, not a meta-tag or a label recovered from name.
- **Pro:** structurally honest. A lens walking a Transform
  with `FieldProject { parent_type, field_label }` sees "project
  the `field_label` child of `parent_type`" directly — no
  indirection through synthesized declarations.
- **Pro:** reduces downstream recovery work. emit_rust dispatches
  `FieldProject` directly to the field-binding lookup in the
  parent type's `TypeRealization`, no `meta_tag == FieldAccessor`
  filter needed.
- **Con:** grows `TransformTarget` by one variant. Requires the
  §9 "scope boundary" receipt for a substrate extension and a
  §8.10 4-pattern check at the commit.
- **4-pattern check (sketch):**
  - *Pattern 1 (fact placement):* the fact "this call projects
    a Conj child" doesn't have another natural home. Not on
    Callable (projection isn't an Arrow), not on port shape
    (the projection targets a specific child of the parent,
    not the parent itself).
  - *Pattern 2 (variant-is-data):* FieldProject and Callable
    are dispatch-on-origin — a declared callable vs an
    intrinsic projection. Different data shapes by construction.
  - *Pattern 3 (algebraic form):* projection is not an algebra
    field (see ground-truth check above).
  - *Pattern 4 (dimensional):* Callable/FieldProject don't
    decompose into a record-with-sub-coproduct; they're
    categorically distinct dispatch targets.
  - *Verdict:* TERMINAL.

**Option 1b — New `Behavior::FieldAccess` variant.**
A 6th behavior alongside Value/Transform/Branch/Loop/Bind.

- **Smuggling route: none.**
- **Pro:** most honest. Field access IS a distinct kind of
  computation from function application, and the "behaviors
  are terminal categories" framing would gain structural
  clarity.
- **Con, blocking:** violates the "Behaviors terminal at 5"
  invariant unless the invariant is re-opened. More
  importantly, fails the 4-pattern check at *Pattern 4
  (dimensional)*: FieldAccess and Transform both consume one
  input, produce one output, and dispatch on a target. They
  are dimensionally related, not categorically distinct. The
  behavior layer is for categorically distinct computations;
  a new behavior for each dispatch flavor is flat coproduct
  growth at a higher level.
- **Verdict:** rejected.

**Option 1c — Synthesized accessor declarations, per-use.**
Each occurrence of `p.field` in user code allocates a fresh
Arrow declaration with a `FieldAccessor` meta-tag and a
`Pending` body. Transform's target points at the freshly
allocated declaration. (PR #453's attempted form.)

- **Smuggling route: #1 (synthesized declarations).**
- **Con:** declaration-table bloat linear in field-access
  occurrences. The `feedback_no_metadata_markers` principle is
  nearly violated — the `FieldAccessor` marker is a typed
  meta-tag, not a string, which saves it from being a hard
  violation, but the (meta_tag + name + Arrow shape) recovery
  pattern is exactly what the principle warns against.
- **Con:** a lens walking the declaration table sees many
  `(parent, label)` accessor declarations that are compiler
  artifacts, not user-authored facts. Layering opacity leak.
- **Con:** the field identity has to be recovered at emit time
  from `(meta_tag, name, Arrow.inputs[0])` rather than being
  visible as a typed dispatch.
- **Verdict:** rejected — #453's experience proves the smuggling
  cost is real.

**Option 1d — Canonical accessor declarations, one per
(parent_type, field_label).**
A variant of 1c that deduplicates: at bootstrap or first use,
allocate one canonical accessor per unique (parent, label)
pair. Every `p.first` in the program targets the same
`Pair_first_accessor` declaration.

- **Smuggling route: #1 (synthesized declarations) with
  deduplication.**
- **Pro vs 1c:** bounded growth — O(total fields declared)
  rather than O(occurrences).
- **Con:** still synthesis. The accessors are derived, not
  authored. A lens walking the declaration table still sees
  compiler-artifact declarations mixed with user-authored
  ones. The layering opacity concern applies, just at lower
  density.
- **Verdict:** rejected — deduplication doesn't dissolve the
  structural concern, it only reduces its volume.

**Option 1e — Preserve `Path` at surface level, handle in
`emit_rust`.**
Don't lower field access at all. The substrate carries
`SurfaceExpr::Path` through the pipeline, and emit_rust
produces the Rust field-access syntax at rendering time.

- **Smuggling route: #3 (surface state preserved in the
  substrate).**
- **Con, blocking:** violates the substrate / surface
  separation. The substrate exists to hold structural facts
  about the program; preserving raw surface syntax through
  to emit means the substrate doesn't fully represent the
  program's structure and every downstream consumer has to
  re-parse the surface form.
- **Con:** breaks the "lenses walk a Dag, not a parse tree"
  principle. A lens asking "what does this Transform do?"
  would have to distinguish "it's a real Transform" from
  "it's a leftover SurfaceExpr::Path." Layering collapse.
- **Verdict:** rejected.

**Decision: 1a.** Locked in. Elimination walk lands on 1a by
process of removing 1b (Behavior terminal-at-5), 1c/1d
(smuggling route #1), and 1e (smuggling route #3). The reasons
for committing:

1. **It's the only clean option left.** The elimination walk
   above is exhaustive on the categories. Each rejected option
   is rejected for a substrate-level reason, not a
   convenience-level one.
2. **It's the minimal extension.** One new TransformTarget
   variant is the smallest substrate commitment that honestly
   expresses projection. 1b grows the most-constrained
   substrate layer (Behavior); 1a grows a less-constrained
   one (TransformTarget's target coproduct).
3. **The §9 receipt and §8.10 audit are cheap.** The 4-pattern
   check above is a paper exercise; the §9 receipt is a single
   explicit entry in the scope-boundary section. Both are
   doable in the Prereq 1 PR itself.
4. **Downstream work gets simpler.** emit_rust's field-binding
   dispatch becomes a `match` arm on `TransformTarget::FieldProject`,
   not a `meta_tag == FieldAccessor` filter. A lens walking
   field access reads the typed variant directly.

**Prereq 1 codifies this as the implementation target.** The
re-implementation (replacing #453's synthesized-accessor
approach) extends `TransformTarget` with `FieldProject
{ parent_type: DeclarationId, field_label: String }`, updates
`lower_expr`'s Path arm to emit this variant, extends
`emit_rust` to dispatch it to the parent type's
TypeRealization's FieldBinding entry, and deletes the
`FieldAccessor` substrate.dag marker and the `lower_field_chain`
synthesis helper. Tests from PR #453 port over largely
unchanged — the Dag-shape assertions check a typed variant
instead of a meta-tagged Arrow.

**Non-goal.** Field access through template-parameterized
records (`p: Pair<Int>`) remains out of scope for Prereq 1.
That requires composing substitution resolution with projection
and is naturally handled as a follow-up that builds on
§3.5's SubstStack threading.

---

### §3.7 The third substrate question: match-with-payload binding

**Problem.** Lens code destructures sum variants with
patterns like `match b { Bind(bind) => bind.params }`.
The surface form binds a local name `bind` to the variant's
payload so the arm body can reference it. The semantic has
two sub-concerns that must be expressed in the substrate:

1. **Discrimination:** which variant does this arm match?
   (today's `BranchPattern::Resolved(DeclarationId)`)
2. **Extraction:** what local name carries the variant's
   payload in the arm body, and which port holds its value?

#453 grew `BranchPattern` to a 4-way flat coproduct
(`{Unresolved, Resolved} × {Bare, WithPayload}`) to express
both. That's smuggling route #2 (flat coproduct growth around
a dimensional distinction). This section walks the honest
option space.

**Ground-truth check.** The same thesis-drift observation
as §3.6 applies: "Disj elimination" would intuitively belong
to an algebra, but verifying `dsl/std/algebra.dag` confirms
there is no general Disj-elimination operation — and arguably
shouldn't be one. Each Disj has its own eliminator shape
(different variant count, different payload types); there is
no shared "DisjAlgebra" users inhabit. Disj elimination is a
substrate primitive (like Conj projection), and the existing
`Behavior::Branch` already IS the eliminator. What Prereq 2
needs is not a new eliminator — it's a way to express the
payload-extraction sub-concern inside the existing
`Branch` / `Path` machinery.

**Five options.**

**Option 2a — Flat 4-way `BranchPattern`.**
`BranchPattern = UnresolvedVariant | ResolvedVariant
| UnresolvedVariantWith | ResolvedVariantWith`. The shape
PR #453 landed.

- **Smuggling route: #2 (flat coproduct growth).**
- **Con, blocking:** fails the 4-pattern check at
  *Pattern 4 (dimensional)*. The variant axis
  `{Unresolved, Resolved}` and the binding axis `{Bare, With
  Payload}` are orthogonal; encoding them as a flat 4-way
  coproduct hides the dimensional structure. The 4 variants
  aren't categorically distinct — they're the product of two
  binary dimensions.
- **Con:** infer's `Unresolved → Resolved` rewrite has to
  preserve the binding sub-structure under a case-by-case
  dispatch (#453's `RewriteShape` enum exists exactly to
  work around this).
- **Verdict:** rejected.

**Option 2b — Dimensional `BranchPattern` record.**
Restructure `BranchPattern` into a record with two fields,
one per dimension:

```
struct BranchPattern {
    resolution: PatternResolution,  // Unresolved { name, span } | Resolved { decl }
    payload: Option<PayloadBinding>,
}
```

- **Smuggling route: none.** The dimensional structure is
  expressed directly.
- **Pro:** correct decomposition of the 4-way coproduct.
- **Con:** expands `BranchPattern`'s semantic role. Today
  BranchPattern answers "which variant does this arm match?"
  — a pure discrimination concern. Adding binding to the
  record extends the role to "which variant AND what to
  capture." Two concerns on one shape.
- **Con:** big refactor. Every consumer of the existing
  `BranchPattern` variants (emit_rust dispatch, infer
  rewrite, lower construction, tests) needs to change, and
  the change is a restructuring of a hot coproduct rather
  than an additive extension.

**Option 2c — New `Behavior::VariantBind` variant.**
A 6th behavior specifically for "discriminate a Disj variant
and bind its payload to a port."

- **Smuggling route: none.**
- **Con, blocking:** fails 4-pattern check *Pattern 4*
  (dimensional): VariantBind is clearly a sub-concern of
  Branch, not a peer behavior. Variant binding is a property
  of arm lowering, not a separate category of computation.
- **Con:** terminal-at-5 violation.
- **Verdict:** rejected.

**Option 2d — Disj elimination via algebra.dag.**
Hypothetical: if algebra.dag had a DisjAlgebra with a
`case(variants, handlers) → result` eliminator, Prereq 2
could lower match-with-payload to a call into that eliminator.

- **Blocking premise failure:** algebra.dag has no such
  operation, and the thesis-level argument above shows it
  shouldn't. Disj elimination is not an algebra inhabitance
  operation; it's a substrate primitive already expressed
  by `Behavior::Branch`.
- **Verdict:** rejected — premise false, miscategorized.

**Option 2e — `Option<PayloadBinding>` field on `Path`.**
Leave `BranchPattern` entirely unchanged (2 variants,
`Unresolved`/`Resolved`). Add a new field to `Path`:

```
struct Path {
    pattern: BranchPattern,         // unchanged, discrimination only
    binding: Option<PayloadBinding>, // NEW, extraction
    output: PortId,
    body: NodeId,
}

struct PayloadBinding {
    binding_name: String,
    payload_port: PortId,
}
```

- **Smuggling route: none.** Adding an Option field to a
  struct is a structural extension, not coproduct growth
  or synthesis.
- **Pro: correct role separation.** `BranchPattern`'s job is
  "which variant does this arm match?" — discrimination.
  `Path`'s job is "coordinate one arm of a Branch" — the arm
  container. Payload binding is a property of the arm, not
  of the discrimination, so it belongs on Path. Per
  `INVARIANTS.md` §"Minimal information per interface":
  design signatures from the job outward. BranchPattern gains
  nothing; Path extends its existing arm-coordinator role.
- **Pro: decoupled inference rewrite.** Infer's job is
  resolving variant names to DeclarationIds — pure
  pattern-level work. With 2e, infer rewrites `path.pattern`
  and leaves `path.binding` alone. No `RewriteShape` helper
  needed. Smaller surface, less to get wrong.
- **Pro: decoupled emit dispatch.** emit_rust reads
  `path.pattern` for discrimination and `path.binding` for
  extraction as two orthogonal reads on the same `Path`
  struct. Each dispatch is straightforward.
- **Pro: guards land naturally.** If guard clauses are ever
  added to v3 (`match x { Right(p) if p.first > 0 => ... }`),
  they fit 2e as another Option field on Path (`guard:
  Option<NodeId>`) without disturbing either BranchPattern
  or PayloadBinding.
- **Con:** two orthogonal reads on Path at each consumer
  (where 2b has one read on BranchPattern). Marginal
  ergonomic cost; structural clarity wins.
- **4-pattern check:**
  - *Pattern 1 (fact placement):* the binding is arm-level,
    `Path` is the arm container. Natural home.
  - *Pattern 2 (variant-is-data):* `PayloadBinding` is a
    record (two fields), not a discrimination.
  - *Pattern 3 (algebraic form):* no algebra involved.
  - *Pattern 4 (dimensional):* `Option<PayloadBinding>` is
    the natural dimensional form — the binding is present or
    absent, not a flat coproduct growth.
  - *Verdict:* TERMINAL.

**Decision: 2e.** Locked in. Three reasons:

1. **Role separation matches the substrate's existing
   factoring.** BranchPattern answers "which variant";
   Path answers "what happens in this arm." Adding binding
   to Path extends an existing role; adding binding to
   BranchPattern (2b) conflates two roles on one shape.
2. **Decoupled rewrite + dispatch.** Inference and emission
   each touch exactly the field they care about. No
   `RewriteShape` helper, no case-by-case dispatch, no
   coupled case table. Smaller surface area reduces the
   chance of a bug where a new variant in one dimension
   forgets to preserve the other.
3. **Minimal substrate delta.** 2e is one new field on an
   existing record + one new small record type. 2b is a
   full restructure of `BranchPattern`. The smaller delta
   ports easier and preserves more of the existing
   infrastructure.

**On `PayloadBinding.binding_name`.** The `binding_name` field
is load-bearing during lowering — the lowerer has to insert
the name into the arm-local scope so subsequent field accesses
inside the arm body (`p.first` inside `Right(p) => p.first`)
resolve to the payload port. Post-lowering, the name is
carry-forward: inference and type-checking consume
`payload_port` (a PortId), not the name. `emit_rust` may use
the name for readable generated output (Rust match arms like
`Right(p) => ...` with `p` as the variable name). This is the
same class as `SourceSpan` on every Behavior — source-level
information preserved for non-semantic reasons. Documented
explicitly so future readers don't treat it as a hidden
semantic channel.

**Prereq 2 codifies this as the implementation target.** The
re-implementation (replacing #453's 4-variant BranchPattern
approach) leaves `BranchPattern` at 2 variants
(`Unresolved`/`Resolved`), adds `binding: Option<PayloadBinding>`
to `Path`, adds a small `PayloadBinding` record type, updates
`lower_expr`'s match arm to populate the binding field when
the pattern is a `VariantWith`, and leaves `infer`'s
`resolve_branch_patterns` pass touching only `path.pattern`
(no `RewriteShape` helper needed). emit_rust grows a
branch-arm rendering path that reads the binding if present.
Tests from PR #453 port over largely unchanged — assertions
check `path.binding` for the payload-binding arms and
`path.pattern` for the variant discrimination.

**Non-goal.** Multi-positional and record-shaped variant
payloads (`Right(Int, Int)`, `Wrap { inner: Pair }`) remain
out of scope for Prereq 2, same as in #453. The
single-positional-unwrap shape is the minimum that unblocks
lens migration.

---

## §4. The mechanism, step-by-step

How a `.dag` lens becomes a running check over a `Dag` value at
CI time, assuming the §11 prerequisite slate has landed:

1. **Substrate types loaded.** `src/v3/std/substrate.dag` is
   added to the `V3_SPECS` include list and parses at bootstrap
   via `include_str!` (same pattern as `rust.dag` and `v3_l1.dag`
   today). Cross-file forward references resolve via
   `resolve_pending_identifiers`.
2. **Inhabitance check.** A startup pass walks every declaration
   in `std.substrate::*` and asserts the corresponding Rust
   struct in `src/v3/compiler/src/dag.rs` matches the declared
   field shape. Mismatches are fail-closed diagnostics. This is
   the same inhabitance mechanism `rust.dag` already uses for
   arithmetic primitives — applied to the compiler's own types.
3. **`std.list` loaded.** `src/v3/std/list.dag` is added to
   `V3_SPECS`. Declares `List<T>` as a Disj, `fold`/`map`/
   `filter`/`length`/etc. as higher-order functions. The
   higher-order functions work because **Prereq 0** (template
   instantiation for function-typed parameters — see §3.5)
   landed first.
4. **Lens file loaded.** `src/v3/lenses/unused_parameters.dag`
   is parsed into a Dag via the standard `compile_to_dag`
   pipeline. The lens's `check(d: Dag) -> List<UnusedParameter>`
   function becomes an Arrow declaration whose body is a Bind
   sub-DAG. Every field read in the lens (`d.nodes`, `bind.params`,
   `b.span`) lowers to the corresponding substrate field-access
   form (**Prereq 1** — field-access lowering).
5. **Lens compiled to Rust.** `emit_rust` runs on the lens Dag
   and produces a Rust function `fn check(d: &Dag) ->
   Vec<UnusedParameter>`. Field access on `Dag` / `Behavior` /
   etc. compiles to the corresponding Rust field access or
   accessor method via the inhabitance binding established in
   step 2. Pattern matching on `Behavior` variants compiles to
   Rust `match` arms. The lens becomes idiomatic Rust code that
   reads the compiler's own internal state through the same
   surface every other consumer uses.
6. **Compiled lens invoked.** A test or CI runner calls the
   compiled function with a `Dag` value and collects the
   returned violations. Zero Rust lens source code. The lens is
   `.dag`, compiled by v3's own pipeline, executed as Rust.

**What's new in emit_rust.** The existing pipeline handles
`OperatorKind` transforms (arithmetic, comparison) and
`FunctionRealization` transforms (via the `rust.dag` realization
index). The extension is: when a Transform reads a field off a
value of a realized substrate type (e.g., `d.nodes` where `d:
Dag`), emit the corresponding Rust field access. The binding is
declared in `rust.dag` — for `Dag.nodes`, the realization says
"the Rust `Dag` struct's `nodes` field has type `Vec<Behavior>`
and is read as `dag.nodes`." This is roughly ~100 lines in
`emit_rust.rs`, following the existing realization-dispatch
shape.

### §4.1 Field-access realization in `rust.dag`

A new realization category — **or, more cleanly, an extension
of the existing `TypeRealization` shape to carry per-field
bindings.** Every `TypeRealization` for a realized substrate
type declares, alongside its Rust type name, a mapping from
`.dag` field names to Rust field names / accessor methods.

```
// src/v3/spec/rust.dag — extended TypeRealization
type TypeRealization {
  for: Declaration              // the .dag type being realized
  rust_name: String             // the Rust type name
  fields: List<FieldBinding>    // per-field Rust mappings
}

type FieldBinding {
  dag_name: String              // field name in the .dag declaration
  rust_access: RustFieldAccess  // how to read it from a Rust value
}

type RustFieldAccess
  = DirectField(String)         // struct.field_name
  | AccessorMethod(String)      // struct.method_name()
```

Example entries (illustrative; real syntax TBD during
implementation):

```
data dag_realization: TypeRealization = {
  for: std.substrate.Dag
  rust_name: "Dag"
  fields: [
    { dag_name: "nodes",        rust_access: DirectField("nodes") },
    { dag_name: "declarations", rust_access: DirectField("declarations") },
  ]
}

data bind_node_realization: TypeRealization = {
  for: std.substrate.BindNode
  rust_name: "BindNode"
  fields: [
    { dag_name: "params",      rust_access: DirectField("params") },
    { dag_name: "value",       rust_access: DirectField("value") },
    { dag_name: "span",        rust_access: DirectField("span") },
    { dag_name: "result_port", rust_access: AccessorMethod("result_port") },
  ]
}
```

**Why this shape instead of a separate `FunctionRealization`
category.** Field access is the natural compositional form, and
a realization that says "field `X` on `.dag` type `T` maps to
Rust field `Y`" is exactly the bridge we need. Every lens that
reads a field goes through this mapping; nothing else does. A
separate category for "function calls as Rust methods" is only
needed if we have standalone functions that aren't field reads
— and once the compositional model replaces query primitives,
we don't.

**Exception for `List<T>` operations.** `fold`/`map`/`filter`/etc.
are NOT field reads — they're higher-order function calls that
walk the list structure. Their realization path goes through
**Prereq 0** (template instantiation for function-typed
parameters), not through field access. See §3.5.

### §4.2 No `substrate_query.rs` Rust module

The earlier draft proposed a dedicated `src/v3/compiler/src/
substrate_query.rs` module holding Rust impls for every query
primitive. That file goes away. **Field access binds directly
to existing Rust struct fields via the inhabitance check.** The
lens compiler doesn't call a Rust accessor function; it emits
`dag.nodes` (or whatever the `rust_access` binding says) inline.
No bridge module. No parallel Rust API surface to maintain.

**One caveat.** The inhabitance check may fail if the Rust
struct's field names or shapes don't match the `.dag` declaration
exactly. The cleanup direction is **rename Rust fields to match
the `.dag` declaration's canonical names**, one-time structural
rename, no runtime cost. If a Rust field genuinely cannot be
renamed (name collision, history reasons), `FieldBinding` carries
the Rust name via `DirectField("old_name")` — the translation
layer, but scoped to per-field aliasing, not a whole accessor
module.

### §4.3 Pipe operator (`|>`) — parser sugar, not a function

Per the §3 modeling discussion, pipe has a natural function form
for the unary case:

```
// unary pipe — in std.list or std.pipe
fn pipe<A, B>(x: A, f: fn(A) -> B) -> B = f(x)
```

…but the general multi-arg form `x |> f(y)` cannot be expressed
as a function without currying, which v3 deliberately does not
have. **Multi-arg pipe is parser-level sugar.** At parse time,
`x |> f(y, z)` desugars to `f(x, y, z)` — first-arg injection,
matching F# and Elm conventions. The desugaring happens during
parsing; by the time lowering runs, there is no pipe — just a
normal function call.

**Why first-arg injection (not last-arg).** Three reasons:
1. It treats `x |> f` and `x |> f(y)` uniformly. In both cases
   `x` is the "subject" being threaded through.
2. It matches method-chaining idioms: `x |> length` reads as
   "the length of x."
3. Last-arg injection (Elixir-style) is harder to read in the
   `x |> fold(0, add)` case, where the piped value ends up
   buried three arguments deep in the call site.

**Parser prerequisite.** Add a `SurfaceExpr::Pipe { left, right,
span }` variant that parses `x |> <call_expr>` with call-expr
having a hole for the first argument. Desugar in `parse_expr`
during parsing — not a lowering-time transform. This keeps the
grammar rule self-contained in parse.rs.

**Scope.** The pipe operator is a **cosmetic ergonomic**. Lenses
can be written without it (using nested `fold(filter(map(...)),
...)` expressions instead). Its place in the prereq slate is
**Prereq 5** — small parser extension, no substrate change, no
lowering change. Can land independently of the other prereqs.

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

**After** (`.dag`, `src/v3/lenses/unused_parameters.dag`):

```
module lenses.unused_parameters

import std.substrate { Dag, Behavior, BindNode, NodeId, PortId, SourceSpan }
import std.list { List, fold, filter, map, enumerate, concat, empty, contains }

type UnusedParameter {
  function: NodeId
  parameter: PortId
  parameter_index: Int
  function_span: SourceSpan
}

// Entry point: walk the Dag, check every function-shaped Bind.
fn check(d: Dag) -> List<UnusedParameter> =
  fold(d.nodes, empty(), |acc, behavior|
    concat(acc, check_behavior(d, behavior))
  )

// For each Behavior, return the violations it produces. Only
// function-shaped Binds produce violations; everything else is
// skipped.
fn check_behavior(d: Dag, b: Behavior) -> List<UnusedParameter> =
  match b {
    Bind(bind) =>
      if length(bind.params) > 0 then
        check_bind(d, bind)
      else
        empty()
    _ =>
      empty()
  }

// Walk the function body's sub-DAG, collect referenced ports,
// and return a violation for every parameter not in the
// referenced set.
fn check_bind(d: Dag, bind: BindNode) -> List<UnusedParameter> {
  let referenced = referenced_ports(d, bind.value)
  filter(
    map(
      enumerate(bind.params),
      |ie| UnusedParameter {
        function: bind.id,
        parameter: ie.value,
        parameter_index: ie.index,
        function_span: bind.span,
      }
    ),
    |violation| !contains(referenced, violation.parameter)
  )
}

// Iterative work-list walk from root_port backwards through
// produced_by edges. Collects every port the walk touches as
// "referenced."
fn referenced_ports(d: Dag, root: PortId) -> List<PortId> =
  // Structural form: recursive walk via pattern matching on
  // Behavior variants to find each producer's input ports.
  // Implementation details elided in this sketch — see the
  // Rust form for the exact algorithm. The point is: every
  // operation here is field access + pattern match + list
  // ops, no query primitives.
  ...
```

**What changed:**

- Field access replaces query primitives: `d.nodes`,
  `bind.params`, `bind.value`, `bind.span`, `bind.id`,
  `ie.index`, `ie.value` — all direct reads off declared
  record fields.
- `match` with payload binding replaces variant-inspection
  helpers: `match b { Bind(bind) => ... }` captures the
  `BindNode` payload as a local.
- Standard list operations (`fold`, `filter`, `map`,
  `enumerate`, `concat`, `length`, `contains`) come from
  `std/list.dag` — not from a `substrate_query` layer.
- Higher-order calls (`filter(list, predicate)`) resolve via
  the template instantiation mechanism from §3.5.
- The algorithm is identical to the Rust form — same walk,
  same output. The host language is the substrate itself.

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

### §6.2 Field-access binding parity

New test: `tests/m2_field_access_binding_test.rs`. For each
realized substrate type declaration (`Dag`, `BindNode`,
`TransformNode`, etc.), assert that every `.dag`-declared field
has a `FieldBinding` entry in `rust.dag` with either a
`DirectField` or `AccessorMethod` mapping. Compile a tiny `.dag`
snippet that reads each field, run it against a test Dag, and
assert the compiled Rust reads the corresponding Rust struct
field and produces the expected value. This validates the
inhabitance binding end-to-end: `.dag` declaration → realization
entry → Rust struct field → correct runtime value.

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
> `src/v3/std/substrate.dag` as ordinary `.dag` declarations,
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

Most of the original §10 questions are either **resolved** (Q1:
atomic handles are minimal declarations; see §3.1) or **moved**
to the prerequisite slate in §11 (higher-order functions →
Prereq 0; list ops → Prereq 4; match-with-payload → Prereq 2;
field access → Prereq 1; lambda → Prereq 3). What remains are
questions the implementer will hit during the actual work.

1. **Pattern 2 of the template-instantiation decision (§3.5).**
   This was open in earlier drafts; it is now **closed by the
   §3.5 decision**. Function-typed parameters bind through
   `TemplateArgument` and propagate through the `SubstStack`
   across nested higher-order calls. A direct-call-only first
   pass is no longer acceptable for Prereq 0; the acceptance bar
   includes the two-level `twice(apply(x, f), f)` propagation
   case.
2. **`result_port` canonical field name.** §3.2.1 proposes
   every `Behavior` variant's payload record carries a
   `result_port: PortId` field for the primary-result query.
   Today Rust's `BindNode.value`, `TransformNode.output`,
   `BranchNode` (no direct field — the output is per-path),
   `LoopNode.source` (no, that's the input), etc. are not
   uniform. The cleanup direction: rename the Rust fields so
   every variant has `result_port` as its canonical name. Open
   question: does `BranchNode` have a well-defined primary
   result port, or is the answer "it depends on which path
   executed"? If the latter, `behavior_output_port` is not a
   structural property; it's a runtime fact that depends on
   control flow. Needs investigation before the prereq slate
   starts; affects inhabitance-check design.
3. **List realization — Disj vs Cardinality.** §3.1 declares
   `List<T>` as a Disj (`Empty | Cons(T, List<T>)`) because
   that's the structural shape for pattern matching. But v3's
   substrate also has `Cardinality(Unbounded, T)` as the
   repetition primitive for iterating over sequences. Are these
   two shapes the same at realization time, or is one the
   static grounding and the other the realization grounding?
   Per the two-groundings distinction in `THESIS.md`, I
   suspect: Disj is static (for pattern matching, L1 lens
   walks), Cardinality is realization (for contiguous-memory
   target types like `Vec<T>`). The L4 verification lens would
   check the two are consistent. Prereq 4 (`list.dag`) needs to
   pin this down.
4. **Lens entry point convention.** The `.dag` form declares
   `fn check(d: Dag) -> List<_>`. Is `check` the canonical
   entry-point name, or does each lens pick its own? Proposal:
   **the entry point is discovered by `meta_tag`, not by
   string match.** A lens declares itself via `data
   my_lens_entry: Lens = { check_fn: my_lens.check }` where
   `Lens` is a meta-type in `std/lens.dag`. The runner walks
   declarations whose `meta_tag == Lens` and calls `check_fn`
   structurally. Layer opacity is preserved: no string match
   on "check," no hardcoded registry.
5. **Parser handling of function-typed parameters at call
   sites.** When a call site writes `filter(xs, is_bind)`,
   how does the parser know `is_bind` is meant as a function
   reference rather than a variable reference? Probably: the
   type inference walks the call target's signature, sees the
   parameter is `fn(T) -> Bool`, and binds `is_bind` as a
   declaration reference. Straightforward in principle but
   needs the inference walker to handle function-typed
   arguments specifically. Prereq 0 implementation detail.
6. **v2 CI behavior during the transition.** `src/v3/std/
   substrate.dag`, `src/v3/std/list.dag`, and
   `src/v3/lenses/unused_parameters.dag` live outside the
   v2-scanned `dsl/` tree and so are invisible to v2 CI.
   But when reflection lands, the v3 compiler ships with these
   files bootstrap-loaded and parsed via `V3_SPECS`. The
   cross-check: v3's test suite must load all three files
   cleanly, while v2 CI stays green. Both conditions are
   expected to hold, but worth validating during implementation.

These resolve during the prerequisite slate or during the
reflection PR itself. None of them are design blockers; they're
implementation details that need care.

---

## §11. What counts as done

Reflection cannot ship in isolation. It depends on a slate of
**six prerequisite PRs** that lift the grammar and standard
library to the point where lenses can be written compositionally.
Each prereq is independently reviewable and lands as its own PR.
Reflection is the PR that closes the slate.

### Prerequisite slate (independent PRs, lands before reflection)

**Prereq 0 — Template instantiation for function-typed
parameters.** The deepest substrate question. **Decision locked
as 1c per §3.5:** extend `TemplateArgument`'s value slot to
accept function references (as `DeclarationId`) alongside type
arguments. The inference walker resolves function parameters
through the existing `SubstStack` machinery — same way type
parameters propagate. No new substrate variant, no runtime
function dispatch, no monomorphization sprawl. Land the
lowering + inference path, add test coverage for a higher-order
call (e.g., `fn apply<T>(x: T, f: fn(T) -> T) -> T = f(x)`
called as `apply(3, negate)`), and keep emit-side wiring as a
later commit. **Blocker for Prereq 4.**

- [x] §3.5 decision committed (1c — locked)
- [x] `TemplateArgument` value slot accepts function-reference
      bindings (`DeclarationId`) alongside type bindings
- [x] Inference walker threads function bindings through SubstStack
- [x] Test: a single-level higher-order call compiles cleanly,
      lowers through `Instantiation`, and keeps function-typed
      arguments out of the runtime input list
- [x] Test: a two-level nested higher-order call resolves via
      SubstStack propagation (the `twice(apply(x, f), f)` pattern)
- [x] No regressions on existing v3 tests

**Prereq 1 — Field-access lowering.** Extend `lower_expr` to
resolve `SurfaceExpr::Path` against local-variable Declarations
and walk to the named field. Today the parser already produces
`Path` for `a.b.c`; only lowering for user-code expression
position needs the new path. **Decision locked as 1a per
§3.6:** add a new `TransformTarget::FieldProject { parent_type:
DeclarationId, field_label: String }` variant. Field access
lowers to a `Transform` with this target, not a synthesized
accessor declaration. Requires a §9 receipt for the substrate
extension and a §8.10 4-pattern check (both completed in §3.6).
**Independent of other prereqs, can land first. Supersedes
PR #453's synthesized-accessor attempt.**

- [x] §3.6 decision committed (1a — locked)
- [ ] `TransformTarget::FieldProject` variant added to `dag.rs`
- [ ] Lowering extension in `lower_expr` emits the new variant
- [ ] `emit_rust` dispatches `FieldProject` via field-label
      string on the parent value (no FieldBinding needed until
      the reflection PR wires realized substrate types)
- [ ] `infer.rs` `decide_field_project` resolves output type
      from Conj children
- [ ] Test: field access on a record-typed parameter compiles
      and produces a `FieldProject` target
- [ ] Test: field access on a non-Conj type fails with a
      fail-closed diagnostic
- [ ] Test: nonexistent field fails with diagnostic naming the
      specific field
- [ ] Test: multi-hop field access (`p.inner.x`) compiles

**Prereq 2 — Match-with-payload-binding.** Add
`SurfacePattern::VariantWith { name, binding, span }` variant
and parser rule. Extend lowering so a match arm
`Bind(bind) => body` binds `bind` as a local in `body`'s scope,
pointing at the variant's payload port. **Decision locked as
2e per §3.7:** `BranchPattern` stays at 2 variants
(`Unresolved`/`Resolved`), unchanged. Add `binding:
Option<PayloadBinding>` field to `Path`, with a small new
`PayloadBinding { binding_name: String, payload_port: PortId }`
record type. Role separation: `BranchPattern` answers
discrimination, `Path.binding` answers extraction. Decoupled
rewrite (infer touches only `path.pattern`, leaves
`path.binding` alone) and decoupled emit dispatch. **Supersedes
PR #453's 4-variant BranchPattern attempt.**

- [x] §3.7 decision committed (2e — locked)
- [ ] `SurfacePattern::VariantWith` added to parse.rs
- [ ] Parser accepts `match b { Bind(bind) => …, … }`
- [ ] `PayloadBinding` struct added to `dag.rs`
- [ ] `Path` grows `binding: Option<PayloadBinding>` field
- [ ] `BranchPattern` left unchanged at 2 variants
- [ ] Lowering populates `path.binding` when the pattern is
      `VariantWith`, types the payload port from the variant's
      Disj child
- [ ] Infer's `resolve_branch_patterns` touches only
      `path.pattern` — no `RewriteShape` helper
- [ ] `emit_rust` reads `path.pattern` for discrimination and
      `path.binding` for extraction as two orthogonal reads
- [ ] Test: pattern match that captures a variant payload
      compiles and runs
- [ ] Test: `payload_binding.binding_name` documented as
      carry-forward — consumed by lowering for scope
      insertion, used by emit_rust for readable output, not
      consumed by inference

**Prereq 3 — Lambda parser + lowering.** Add `SurfaceExpr::Lambda
{ params, body, span }`. Parser rule: `|ident (, ident)*| body`
(Rust-ish, avoids overlap with v3's existing block-body
syntax). Lowering: per v3-spec.md §Principle 5, lower to an
ordinary Bind declaration with captures from the outer scope
materialized as additional positional parameters. Captures are
detected by walking the body's free variables. The construction-
site Bind supplies those captured inputs; later call sites pass
only the lambda's declared parameters. **Implementation scope for
the first landing:** lambdas lower only in positions that
already provide an expected function type (for example a
function-typed argument position or an annotated `let`
binding). Bare unannotated lambda values fail closed until
function-value inference is designed explicitly. No new
substrate variants.

- [x] `SurfaceExpr::Lambda` added to parse.rs
- [x] Parser rule accepts `|x| x + 1` and multi-param forms
- [x] Lowering produces a Bind with captures as explicit inputs
- [x] Call-site rewrite: callers pass only declared runtime
      params; captures are baked into the lambda Bind
- [x] Unannotated lambda values fail closed instead of guessing a
      function type from insufficient context
- [x] Test: a lambda that captures from outer scope compiles,
      the captured value flows through as a typed edge, and the
      lens walking the resulting Bind sees params = [declared +
      captured] with no distinction
- [x] Test: calling a captured lambda lowers to a normal
      Transform whose runtime inputs are only the declared
      parameters; the captures are already satisfied by the
      lambda's construction-site Bind
- [x] Test: unannotated standalone lambda fails closed
- [x] **Scope note (first landing):** contextual lambdas only —
      lambdas lower when an expected function type is available
      (annotated let, HOF argument position). Unannotated
      standalone lambda values fail closed. Zero-arg/block-body
      lambdas and the callback-rule lens test (v3-validation-
      experiments Experiment 1) are follow-ups.
- [ ] Test: the callback rule from v3-validation-experiments
      Experiment 1 — a lambda body that flows into a Loop gets
      fan-out = Loop's bound in the ownership lens, termination
      bounded by Loop in the termination lens

**Prereq 4 — `src/v3/std/list.dag` ships.** Add the design file
(already committed to this PR as `src/v3/std/list.dag`) to
`V3_SPECS`. Depends on Prereqs 0 (for higher-order calls), 2
(for pattern matching), 3 (for lambdas). Once these three are
in, `list.dag` parses, type-checks, and becomes available to
subsequent declarations.

- [ ] `list.dag` added to `V3_SPECS`
- [ ] Bootstrap loads `list.dag` cleanly after prereqs 0, 2, 3
- [ ] Test: `fold([1, 2, 3], 0, |acc, x| acc + x)` evaluates to
      6 via v3's compile → emit → run pipeline
- [ ] Test: `filter([1, 2, 3, 4], |x| x > 2)` produces `[3, 4]`
- [ ] Test: `map([1, 2, 3], |x| x * 2)` produces `[2, 4, 6]`

**Prereq 5 — Pipe operator `|>` parser sugar.** Add
`SurfaceExpr::Pipe` or a desugar-at-parse-time transform.
`x |> f(y)` becomes `f(x, y)` (first-arg injection). Parser-only
change, no substrate, no lowering. **Independent of other
prereqs, can land in parallel with any of them.**

- [ ] `|>` tokenizes
- [ ] Parser rule handles `x |> f` and `x |> f(args…)` via
      first-arg injection
- [ ] Test: `5 |> negate` produces `-5`
- [ ] Test: `[1, 2, 3] |> length` produces `3`

### Reflection PR itself (lands after all prereqs)

- [ ] `src/v3/std/substrate.dag` exists as a flat file
      (no subdirectory), parses cleanly, inhabitance check
      passes against the Rust substrate structs in `dag.rs`.
- [ ] Atomic identity handles (`NodeId`, `PortId`,
      `DeclarationId`) declared in `substrate.dag` as minimal
      opaque atoms with `TypeRealization` entries in
      `rust.dag`. `SourceSpan` reused from `std/types.dag`.
- [ ] Every Behavior variant's payload record has a canonical
      `result_port: PortId` field (either directly or via
      per-field aliasing in `rust.dag`). The earlier
      `behavior_output_port` query function is NOT added;
      field access on `result_port` replaces it.
- [ ] Seed ratchet file exists, CI gate counts minimal-atom
      declarations in `substrate.dag`, a PR that adds a new one
      without a receipted exception fails CI.
- [ ] `src/v3/spec/rust.dag` grows per-field `FieldBinding`
      entries for every realized substrate type. Each field
      maps to either `DirectField` or `AccessorMethod`.
- [ ] `emit_rust.rs` handles field access on realized substrate
      types via the `FieldBinding` lookup. No `FunctionRealization`
      category, no `substrate_query.rs` module.
- [ ] `src/v3/lenses/unused_parameters.dag` exists, compiles
      cleanly, produces the same output as the Rust form for
      every test fixture. Uses field access + match-with-payload
      + higher-order calls + `std.list` — all via the prereqs.
- [ ] `src/v3/compiler/src/lens_unused_parameters.rs` is
      deleted.
- [ ] `m2_substrate_inhabitance_test.rs` passes (every
      `.dag` declaration in `substrate.dag` has a matching Rust
      struct).
- [ ] `m2_field_access_binding_test.rs` passes (every field on
      every realized type reads correctly through the
      `FieldBinding` mapping).
- [ ] `m2_lens_unused_parameters_migration_test.rs` passes
      (byte-for-byte with the Rust form's previous output).
- [ ] `m2_lens_self_analysis_test.rs` passes (the lens analyzing
      itself reports zero findings).
- [ ] `THESIS.md` §"Self-inspection" landed.
- [ ] `INVARIANTS.md` §"Bounded substrate seed" landed.
- [ ] `INVARIANTS.md` §"Lenses are substrate declarations"
      landed.
- [ ] `INVARIANTS.md` §"Every dependency is a substrate fact"
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

## §12.5 Consumer migration as first-class scope

Per §1.5, the motivating consumers for reflection are v2's
existing analyses (complexity, ownership, effect derivation,
trace). They are not optional follow-ups; they are the
structural answer to the "this framework is decorative"
critique. Each one is tracked here with an explicit migration
milestone. **The reflection PR is complete when at least one
migration milestone is in flight; the framework is justified
when all four have a scoped plan.**

### §12.5.1 Migration M1 — `dsl/lenses/complexity.dag` (the big one)

**Source:** `src/v2/complexity.dag` (5490 lines).

**Blocker:** the reflection PR itself, plus Prereq 0–5. Cannot
start until lenses-as-`.dag`-programs is real and `std/list.dag`
is loaded.

**Scope:** port the algorithm in shape. Every reconstruction in
v2's complexity either dissolves into a v3 substrate field
(termination evidence on bindings, iteration dimensions on
Cardinality, cost shapes via algebra inhabitance) or reveals a
substrate gap. The migration is therefore an **audit disguised
as a port**: walking v2's complexity line by line and asking
"does v3 carry this structurally?" for each reconstructed fact.

**Acceptance criteria:**

- [ ] Every `annotate_*` helper in v2's complexity that
      reconstructs a fact from the IR has a disposition: either
      (a) DELETED because v3 carries the fact as a substrate
      field, or (b) TRACKED as a prerequisite substrate extension
      in a new design note.
- [ ] `dsl/lenses/complexity.dag` exists and parses under the
      grammar available after the reflection PR's prereq slate.
- [ ] For a canonical test input (e.g., a small v2 sub-module
      both compilers can parse), complexity's v2 output and
      v3 output match on the subset of facts v3 currently
      carries structurally.
- [ ] Substrate gaps identified during the port are either
      closed in prerequisite PRs before complexity lands, OR
      tracked in `DOWNSTREAM_REQUIREMENTS.md` with a concrete
      dissolution plan.

**Expected substrate extensions surfaced by the port** (each
tracked as its own prerequisite if the migration needs it):

- **Termination evidence as typed edges on bindings.** v2's
  complexity walks `TypeBinding` and reconstructs `sub_value_vars`
  via `annotate_descent()`. v3's thesis says bindings should
  carry a typed `provenance` edge. Experiment 2 validated the
  shape partially; full migration forces the remaining work.
- **Iteration dimension on Cardinality / Loop.** v2's complexity
  computes iteration dimensions from loop source + body shape.
  v3's `Cardinality(bound, element)` + `Loop { source, init, body }`
  should carry the dimension structurally, not by reconstruction.
- **Cost shape as algebra-inhabitance.** v2's complexity enumerates
  cost shapes (`ShapeConstant`, `ShapeLinearScan`, etc.). v3's
  algebra inhabitance should carry the shape via which algebra
  a declaration inhabits (e.g., `inhabits Monoid → ShapeConstant`
  for the monoid op).

Each of these is a substrate question that complexity's migration
surfaces empirically. The migration isn't "port the 5490 lines;"
it's "port what ports cleanly, pin what doesn't, let the
reconstruction dissolution direct the remaining substrate work."

### §12.5.2 Migration M2 — `dsl/lenses/ownership.dag`

**Source:** `src/v2/ownership.dag` (719 lines).

**Blocker:** reflection PR, plus complexity migration (M1)
because ownership reads several of the same substrate facts
complexity reads.

**Scope:** move/borrow/clone inference as a pure reader over
v3's substrate. The callback rule from Experiment 1 (closures
in Loops get fan-out = N, not 1) is the ownership lens's
responsibility to implement; v3 currently has no ownership code
at all.

**Acceptance criteria:**

- [ ] `dsl/lenses/ownership.dag` parses.
- [ ] For every function in the test corpus, the lens's
      move/borrow/clone decisions match v2's for the shared
      substrate subset.
- [ ] The callback rule is tested: a closure body flowing into
      a Loop reports fan-out = N, not 1.

### §12.5.3 Migration M3 — `dsl/lenses/effects.dag`

**Source:** `src/v2/effect_derivation.dag` (66 lines). Small.
Experiment 4 from v3-validation-experiments already shipped a
prototype "purity lens" as a `.dag` function.

**Blocker:** reflection PR (to give it the lens-framework
handle). Experiment 4 proved the shape works at v2 level; v3
migration is a straight port.

**Acceptance criteria:**

- [ ] `dsl/lenses/effects.dag` parses.
- [ ] Classifies functions as pure / effectful consistently
      with v2.

### §12.5.4 Migration M4 — `dsl/lenses/trace.dag`

**Source:** `src/v2/trace.dag` (223 lines).

**Blocker:** reflection PR. Low priority — trace is debug
tooling, not load-bearing for correctness.

**Acceptance criteria:**

- [ ] `dsl/lenses/trace.dag` parses and produces readable trace
      output for at least one v3 test fixture.

### §12.5.5 Migration order and gating

The four migrations don't need to be done in any specific
order after reflection lands, BUT:

1. **M1 (complexity) is the highest priority** because it's the
   biggest, the most substrate-dependent, and the one that most
   directly tests Experiment 2. Starting it reveals substrate
   gaps that might also affect M2.
2. **M2 (ownership) blocks any v3 work on code generation
   optimization.** v3 currently emits Rust with no ownership
   analysis (everything is owned or cloned uniformly). Shipping
   M2 is the path to idiomatic Rust emission.
3. **M3 (effects) and M4 (trace) are small and independent.**
   Either can ship on any cadence.

**The framework is justified empirically when M1 lands.** Until
then, reflection is "we expect the compression to work." When
complexity.dag's 5490 lines come into v3 as a .dag lens and
demonstrably reads substrate facts instead of reconstructing
them, the physics-plus-lens claim moves from "thesis" to
"measured result."

---

## §12.6 Self-hosting as the eventual horizon

Reflection is not the endpoint. It is **the unblocker for full
self-hosting** — the milestone at which v3's compiler pipeline
(parse, lower, infer, emit) is itself rewritten in `.dag`, with
Rust code reduced to a bootstrap stage. Self-hosting is
explicitly a **post-reflection** concern; this section exists
to name where it fits in the progression so the reflection PR's
relationship to M3 is load-bearing visible.

**The dependency order, start to endpoint:**

```
L0 — Substrate stable                      [SHIPPED at M1(3)]
       │
L1 — Reflection (this PR + prereqs 0–5)    [in design]
       │    substrate types in .dag
       │    field access, pattern match, lambda, higher-order calls
       │    first lens (lens_unused_parameters) migrates
       │
L2 — v2 consumer migrations                [§12.5 — M1/M2/M3/M4]
       │    complexity (5490 lines)
       │    ownership (719)
       │    effects (66)
       │    trace (223)
       │
L3 — Pipeline stages in .dag               [post-L2 — see SELF_HOSTING.md]
       │    parse.dag   — tokens → SurfaceItems
       │    lower.dag   — SurfaceItems → Declarations + Behaviors
       │    infer.dag   — Dag → Dag with port state populated
       │    emit.dag    — Dag → target source strings
       │
L4 — Full self-hosting (M3)                [long-term]
            v3's compiler is .dag code
            Rust stage0 is vestigial (kept as bootstrap seed)
```

**Why reflection is prerequisite for L3, not a parallel track.**
Pipeline stages operate over substrate data. The parser
produces `SurfaceItem`/`SurfaceExpr` trees; lowering transforms
those into Declarations and Behaviors; inference walks the Dag
and populates port state; emission reads the Dag and produces
target source. **Every single pipeline stage reads or writes
substrate types.** Without reflection, `SurfaceItem` and `Dag`
aren't expressible in `.dag`, so a `.dag` parser can't produce
them and a `.dag` lowering pass can't consume them. L3 cannot
start until L1 ships.

**Why L2 (consumer migrations) comes before L3 (pipeline
migrations).** Consumer migrations are less integrated — porting
complexity from v2 to v3 doesn't break the compile loop. Pipeline
migrations are self-referential — porting the parser means the
compiler that reads `.dag` code needs to run through the stage
that's being ported. That's the meta-circular evaluator problem.
Doing consumer migrations first gives us a test corpus for
pipeline migrations (the migrated lenses exercise the substrate
in the same ways pipeline stages will) and keeps the bootstrap
loop from turning meta-circular too early.

**The four pipeline stages, in the order they should port:**

| Stage | Current Rust file | Lines | Why this order |
|---|---|---|---|
| **1. `emit.dag`** | `emit_rust.rs` | ~340 | Easiest to port because it's already structured as a walk over substrate + realization lookup. The rust.dag realization mechanism already exists; emit's `.dag` form reads substrate via reflection + concatenates strings via `std/string.dag` (prerequisite). Proves the pipeline-in-dag model on the simplest stage. |
| **2. `lower.dag`** | `lower.rs` | ~2000 | Second cleanest. SurfaceItem → Declaration is a walk + construction. The tricky part is the cross-file forward-reference resolution (`resolve_pending_identifiers` sweep), which has its own substrate shape. Requires `std/string.dag` and pattern matching on SurfaceItem variants (which themselves become substrate types). |
| **3. `infer.dag`** | `infer.rs` | ~1100 | The trickiest stage. Inference is mutation-shaped in Rust (populates `Port.state` in place). The functional `.dag` form takes a Dag and returns a new Dag with inferred state. This requires the substrate to treat inference state as a first-class field OR requires each inference step to produce a new Dag value (copy-on-write model). Open substrate question. |
| **4. `parse.dag`** | `parse.rs` | ~1600 | Hardest AND most transformative. The end state is grammar-as-data: parser rules become `.dag` declarations, the parser is a generic rule-interpreter. At that point, "ingest Python" becomes a new declaration file, not a compiler rewrite. But the intermediate step (port `parse.rs` as-is to `.dag` functions on a List<Token>) is its own substantial project. |

Each stage is a multi-PR project. Each gates on substrate
extensions that its port surfaces (same pattern as §12.5's
expected-substrate-extensions for complexity). None of them
starts during the reflection PR; they start after the reflection
PR lands and at least M1 (complexity migration) is underway.

**Full detail is deferred to `src/v3/SELF_HOSTING.md`** — a
design note that expands this section with per-stage acceptance
criteria, blocker dependencies, expected substrate extensions,
and the meta-circular bootstrap model. The reflection design
doc stays scoped to reflection; `SELF_HOSTING.md` carries the
pipeline-migration plan so it can evolve independently without
making this doc unwieldy.

**For warm-elk's current work context:** the prerequisite slate
in §11 is L1 foundation work. Prereq 1 (field-access lowering)
lands toward L1. Prereqs 2/3/5 land toward L1. Prereq 4
(`list.dag`) lands toward L1. Every prereq PR is a brick in the
L1 foundation; reflection itself is L1's capstone. After
reflection lands, L2 migrations begin, and the four pipeline
stages in L3 start once L2 has enough running code to prove the
model. No single PR touches L3; every PR in the slate is L1.

**The structural claim self-hosting validates.** Once v3
compiles itself — when `parse.dag` parses its own source,
`lower.dag` lowers it, `infer.dag` walks the result, `emit.dag`
produces Rust that matches the hand-written stage0 byte-for-byte
— the thesis's "every facet is a substrate fact" claim moves
from direction to empirical fact. Every consumer the project
ever needs (analyses, lenses, emitters, even the compiler
itself) lives in the substrate the substrate describes. That's
the endgame.

---

## §13. Recent-conversation threads captured

Cross-reference for the user's "make sure nothing is missed" ask.
Every thread below is either addressed in-doc or explicitly
deferred with a note.

| Thread | Status |
|---|---|
| **Where does self-hosting fit in all of this?** | §12.6 — pipeline stages (parse/lower/infer/emit) are L3 post-reflection; reflection is the unblocker, not a parallel track; full plan in `src/v3/SELF_HOSTING.md` |
| **Existing consumers (complexity, ownership, effects, trace) must be reframed in the new lens stuff** — structural answer to "accumulating debt, no new consumers" critique | §1.5 — motivating consumers enumerated; §12.5 — explicit migration milestones M1-M4 with acceptance criteria; reflection framework is justified empirically when complexity.dag migrates |
| **Queries fall out of compositional modeling** ("I expected the queries to fall out of proper compositional modeling") | §3.2 — collapsed the query-primitive layer, replaced with field access on declared records |
| **Higher-order function calls as the deepest substrate question** (from the List modeling exercise) | §3.5 — three options (1a substrate variant, 1b monomorphization, 1c template instantiation); **decision locked as 1c** with SubstStack propagation for call-chain threading; codified as Prereq 0 |
| **`behavior_output_port` resolved compositionally** | §3.2.1 — replaced with `result_port: PortId` field on every Behavior variant payload, accessed by field read |
| **`std.list` modeled as the free monoid on T** | `src/v3/std/list.dag` (committed alongside this doc), §3.2 prereq list |
| **Pipe operator `\|>` — function or sugar?** | §4.3 — unary form is a function, multi-arg form is parser-level sugar with first-arg injection |
| **Lambda = Bind + Define from Experiment 1** | §11 Prereq 3 — parser + lowering extension only, substrate already complete per v3-spec.md §Principle 5 |
| **Callback rule for closures-in-Loops** | §11 Prereq 3 test requirement — lens-level validation of ownership fan-out and termination bounding |
| **Captures as anonymous inputs** | §11 Prereq 3 — captures become additional positional parameters, no special capture concept |
| **Grammar-as-data / omni-ingestion** | Flagged in §1 as a parallel work stream, not blocking reflection |
| "Everything is Dag/Node" / project as meta-compiler | §3.0, §3.1, §7.1 — substrate is self-describing, no meta layer, no substrate subdirectory |
| Primitives vs declarations ("l0 primitives vs declare them") | §3.0 — declare everything structural; atomic identity handles are minimal declarations backed by realization entries, not Rust-only primitives |
| Bootstrap problem ("how does this not cycle?") | §3.0 — same mechanism v3 already uses for algebra.dag → types.dag forward refs |
| Seed-minimality invariant | §7.2a — new `INVARIANTS.md` entry with ratchet |
| Reflection primitive as canonical form | §1, §3, §4 — the core of this doc |
| Rust lenses as bootstrap scaffolds | §1 motivation, §5 migration |
| Thin-wrapper-vs-deepening-scaffold gate | §7.3 (retrospective after this PR) |
| `kernel_lens_set` failure class | §1, §7.1 thesis, §7.2 invariant |
| Substrate-as-data | §3.1 substrate types in substrate.dag |
| Interpreter binding (.dag receives Dag) | §4 step-by-step, §4.1 field-access realization |
| Composition Opacity = Layer Opacity | already reconciled (PR #445), cross-ref in §8 |
| Semantic authority after lowering | §8 — this PR extends it |
| Scaffold boundaries | §8 — Rust lenses are tracked scaffolds |
| `lens_unused_parameters` as first migration | §5 |
| `lens_complexity` as v2/v3 comparison | §8 — deferred follow-up, unblocked by reflection |
| **v2 CI scanning constraint on file location** | §3.1 — `src/v3/std/` is bootstrap staging; `dsl/std/` is canonical post-v2 |
| **Name collisions — v3_l1.Declaration sentinel, algebra.Field\<T\>** | §3.1 — rename sentinel → `DeclarationRef`, use `ConjField` for substrate record-field type |
| **Every dependency is a substrate fact** (module/import/buffer lifting, user_start elimination) | §8, prerequisite to reflection — scoped as separate PR, not yet designed in full detail |
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
