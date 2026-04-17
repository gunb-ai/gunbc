# DOWNSTREAM_REQUIREMENTS — v3 substrate consumer enumeration

**Status:** active substrate-consumer gap tracker. All 14 original gaps resolved at M1(2.7); R9 and M1(2.8) addendums landed. **Remaining classes below are still live** — Class 5 (structural gaps after R9 + M1(2.8)), class-6 tokenizer escapes, class-7 realization narrowing.

This document catalogues structural questions that v3 consumers (lenses, inference, emission) ask about `Node`/`Declaration`/`Port`. When a consumer must reconstruct a fact the substrate doesn't expose, that reconstruction is a **substrate-consumer gap** — fix the substrate, don't bridge in the consumer.

Historical enumeration (M1(2.7) → M1(2.8) — landed work, methodology, R16 audit, Consumer 1–4 walkthroughs) moved to [`docs/history/v3-substrate-consumer-enumeration-m1-2-7-to-2-8.md`](../../docs/history/v3-substrate-consumer-enumeration-m1-2-7-to-2-8.md) on 2026-04-17. Retained there for audit traceability.

Several Class 5 entries now have dedicated DB docs or ROADMAP deferrals — cross-referenced inline where applicable. Where a gap is fully tracked elsewhere, the entry here is a one-line pointer; where it's genuinely still open and uncaptured, the full description remains.

## PR-B validation summary (M1(3))

PR-B was the first real downstream consumer to read structurally
through the M1(2.8) substrate. Each open class-5 gap was checked
against its pipeline; the verdicts:

| Gap | PR-B verdict |
|-----|--------------|
| #1 Bool operator grounding | Did not block PR-B. Operator dispatch in PR-B's success program (`let x: Int = 1 + 2`) reads OrderedRing's `add` field directly via the existing §8.9 walk; Bool operator emission is exercised by the `if 1 > 0 then ...` test and works through the same path with `Classical` as the scrutinee. |
| #2 Collection-level algebra receivers | Did not block PR-B. PR-B's emit scope is scalar Int/Bool/String — no FreeMonoid/Set/Map operations. |
| #3 Data body parsing | **Partially closed by PR-B.** The literal-only slice landed via `ValueBody::Structural { fields: Vec<(String, LiteralBits)> }`, `SurfaceExpr::Record`, and `lower_record_to_structural`. The remaining gap is port-carried field values (nested records, list literals, declaration references, Var references). See the gap entry below. |
| #4 Variant constructor expressions | Did not block PR-B. PR-B's emitter doesn't construct sum-type values; the `if/else` branch test uses ordinary if-expression lowering. |
| #5 `where` clause refinement facts | Did not block PR-B. PR-B's Realization meta-type uses no refinement clauses. |
| #6 Declaration references as values | Did not block PR-B. The Realization meta-type was deliberately designed to use scalar field values instead of declaration references; M2+ would let `target: Declaration` replace `target_name: String`. |

Net: **PR-B validated that the M1(2.8) substrate is sufficient for
the first downstream consumer, with one substrate addition
(`ValueBody::Structural`) and zero new behavior variants.** The
remaining class-5 gaps are correctly characterized as M2+ extension
work, not missing substrate facts that PR-B revealed.

### M1(3) PR-B class-6 gap: tokenizer escape sequences

Surfaced during Phase 4 of PR-B when `rust_main_wrap`'s carrier
needed to emit `println!("{}", x)` and a literal `"` couldn't go
inside a string literal. The v3 tokenizer at M1(3) explicitly has
**no escape sequences** — `\"` inside a string is impossible
because the closing `"` terminates the literal at the backslash.
The PR-B workaround uses `%Q` as a literal-quote placeholder that
the emitter substitutes at render time, alongside `%N`, `%T`, etc.

This is a tokenizer extension, not a substrate gap. Dissolves
when the v3 tokenizer learns standard escape sequences (`\"`,
`\\`, `\n`, etc.). The `%Q` placeholder in `rust.dag` is the
visible scaffold; remove the placeholder and rewrite the carrier
when the tokenizer extension lands.

### M1(3) PR-B-unwind R1 class-7 gaps: substrate-level realization narrowing

The PR-B-unwind R1 review on PR #445 raised three substrate-side
issues that are partially addressed and partially deferred. Each
is tracked here so the next round picks them up cleanly.

**Class-7 gap #1: substrate-level type narrowing for realization
fields.** The current shape uses `Declaration` (the universal
sentinel) as the field type for `target` and `op` in rust.dag's
realization meta-types. The substrate accepts any declaration in
those positions, including bad wirings like
`BehaviorRealization { target: Int }` (a primitive type targeted
at a behavior template) or `OperatorRealization { op: Bind }`
(a substrate marker used as an algebra field).

PR-B-unwind R1 added a **lower-time fail-closed narrowing check**
in `lower_record_to_structural` (see
`validate_realization_field_target` in `src/v3/compiler/src/lower.rs`).
The check fires at lower time, not at consumer time, so bad
wirings surface as fail-closed diagnostics anchored to the
offending data-body span. Two regression tests pin the behavior:
`m1_3_prb_unwind_r1_behavior_realization_with_primitive_target_is_rejected`
and `m1_3_prb_unwind_r1_type_realization_with_behavior_target_is_rejected`.

What the lower-time check does NOT do: encode the constraint at
the **type system level** (i.e. make bad states unrepresentable).
The fully structural fix needs one of:

- **`inhabits` syntax on `type` declarations**: e.g.
  `type Bind inhabits BehaviorMarker {}` so the substrate carries
  a typed edge from each behavior marker to a parent kind. Then
  rust.dag's `BehaviorRealization.target: BehaviorMarker` would
  type-check directly against the inhabits edge. Requires parser
  + lower work to support `inhabits` on type declarations.
- **`where` clause refinement** on field types: e.g.
  `target: Declaration where inhabits(BehaviorMarker)`. Class-5
  gap #5 already tracks `where` clauses; this gap is a specific
  consumer of that work.

**Dissolution trigger**: when v3's parser grows either `inhabits`
syntax or `where` clauses, the lower-time check in
`validate_realization_field_target` dissolves into the parser-
level constraint and is removed.

**Class-7 gap #2: field-as-declaration in `ValueBody::Structural`.**
The current payload shape is
`Vec<(String, FieldValue)>` where the `String` is the field label
("target", "carrier", etc.). PR #445's R1 review noted that this
keeps a per-field-label string in the data body. The proposed
fix reshapes Field substrate so each Conj field is its own
declaration with a stable id, and `ValueBody::Structural`
carries `Vec<(DeclarationId, FieldValue)>` — typed field
identity instead of a label string.

The blocker is that v3's `Field` struct (in `dag.rs`) is currently
`{ label: String, ty: DeclarationId }` — fields are NOT their own
declarations. Changing this touches `dsl/std/*` consumed by both
v2 and v3, so it requires v2 coordination.

**Dissolution trigger**: when v2 either deprecates or coordinates
with v3 on the field-as-declaration substrate change, the
`(String, FieldValue)` payload moves to `(DeclarationId, FieldValue)`
and consumer-side label lookups (e.g.
`field_decl_ref(fields, "target")`) become typed-id lookups. The
PR-B-unwind R1 round delivered the **fail-closed reading** of
the current shape: `require_field_decl_ref` /
`require_field_string` in `emit_rust.rs` panic on missing fields
or duplicate keys, so the label-string lookup fails loudly when
the spec is malformed.

**Class-7 gap #3: bootstrap fixture enumeration.** The pre-unwind
shape had `const RUST_DAG: &str = include_str!("../../spec/rust.dag");`
constants in `bootstrap.rs` plus a hardcoded fixture array.
PR-B-unwind R1 closed this gap via `build.rs` + the generated
`V3_SPECS` static array. Adding a new per-target spec file
(e.g. `python.dag`) is now a pure file-system change — drop the
file in `src/v3/spec/`, run `cargo build`, and the loader picks
it up. **Status**: closed at PR-B-unwind R1.

**Scope.** Covers both the read side (`infer.rs`, `lens_depth.rs`,
`lens_provenance.rs`) and the write side (`parse.rs` →
`lower.rs` boundary). The write side was added in a second pass
after a reviewer surfaced five write-pipeline gaps; the lesson is
that enumeration applies symmetrically to readers and writers.
Gap count at time of enumeration: **14 entries** across 4
fact-placement classes. **All 14 now resolved.**

## Class 5 — remaining structural gaps after R9 and M1(2.8)

These are substrate-level gaps that the R9 + M1(2.8) passes
surfaced but did not close. Tracked as the next substrate
milestone.

### Class 5 gap 1: Bool operator grounding

**What's missing.** `Bool` is `Classical = True | False` (a
`Disj`), with no structural link from `Classical` to
`BooleanAlgebra`. The R9 operator walk terminates at a `Disj`
with no field lookup path — the walker falls back to the
Rust-side `(T, T) -> T` / `(T, T) -> Bool` scaffold bridge for
any type whose chain doesn't land on an algebra `Conj`.

**Why it's hard.** The link "Bool inhabits BooleanAlgebra" is
commented in `algebra.dag` and encoded in the
`kernel_algebra_profile` data table, but the substrate doesn't
have an `inhabits` edge you can walk. Options:
- Add `inhabits TypeExpr` syntax to `type_item` parsing, then
  edit `logic.dag` to say
  `type Classical inhabits BooleanAlgebra<Classical> = True | False`.
  The walker consults `inhabits` as a second-chance edge when
  the Instantiation chain terminates without an algebra. This
  is a parser extension but structurally clean.
- Consume `kernel_algebra_profile`'s body at bootstrap, build
  an in-memory `name → algebra` map, and have the walker check
  that map when the chain doesn't find an algebra. This avoids
  parser work but requires record-literal body parsing (another
  deferred class).
- Treat Bool specially in the walker with a hardcoded
  `classical_algebra_id` cached at bootstrap. A bridge, not a
  structural fix.

**Current status.** Bool operators still dispatch through the
Rust scaffold fallback inside `resolve_operator_arrow`. The
fallback is explicitly documented and returns
`(Bool, Bool) -> Bool` / `(Bool, Bool) -> Bool` (arithmetic on
Bool is nonsensical but the path exists for safety). Tests
that exercise Bool operators still work because the
fallback signature is shape-compatible.

### Class 5 gap 2: Collection-level algebra receivers

**What's missing.** `FreeMonoid<T>`,
`BooleanAlgebra<T>` (for `Set<T>`), and `PartialFunction<K, V>`
(for `Map<K, V>`) have operator fields whose receiver is the
*parameterized algebra itself*, not the type parameter:
`concat: fn(FreeMonoid<T>, FreeMonoid<T>) -> FreeMonoid<T>`.

The R9 walk's substitution rule ("algebra's first type
parameter is the receiver, substitute T → source") doesn't
cover this shape — for `FreeMonoid`, the receiver is
`FreeMonoid<T>`, not `T`. The walker would need to identify
the receiver position differently (e.g., "the first input
parameter's declaration *is* the receiver, replace all of its
occurrences in the Arrow with source").

**Why it's deferred.** At M1(2.7), `type_to_declaration_id`
allocates a fresh declaration for each syntactic occurrence
of `FreeMonoid<T>` in the algebra field, so they don't share
a DeclarationId and can't be matched structurally. Hash-consing
or a "first input IS the receiver" convention would work but
requires more thought.

**Current status.** String operators (`"a" + "b"`) fall back
to the Rust scaffold bridge. No existing user-facing tests
exercise String operators, so the bridge is latent.

### Class 5 gap 4: Variant constructor expressions

**What's missing.** Bare variant names in expression position
(e.g. the `False` in `match a { True => False, False => True }`'s
arm bodies) don't resolve. R7's anonymization set
`decl.name = None` on sum variant children, so
`declaration_by_name("False")` returns None even when
`False` is a declared constructor of `Classical`. The
name-resolution path in `lower.rs` falls through to an
identifier stub, which the strict sweep flags as unresolved.

**Why it's hard.** Match patterns resolve scoped against the
scrutinee's Disj children, which sidesteps the anonymization —
but variant RHS expressions have no scrutinee context. Options:
- **Variant-constructor surface syntax** (`Classical.False` or
  similar) so the resolution is explicit. Parser extension,
  user-visible.
- **Targeted anonymization rollback**: make sum variants
  findable by name (reverting that part of R7) while keeping
  TypeParams anonymous. Adds namespace ambiguity
  (`Classical.True` vs `Bool.True` share the name `True`;
  first-match-wins or fail-closed on duplicates).
- **Infer-time re-resolution**: for `Var` expressions that
  didn't resolve at lower time, infer could re-try resolution
  against a contextual type (the Branch output type, a
  surrounding let annotation, etc.). Biggest substrate lift
  but most general.

**Current status.** Match parses, type-checks, resolves
patterns. Arm bodies can use literals, primitives, user function
calls, operators — anything that doesn't require bare variant
expressions. `logic.dag`'s `classical_not` / `classical_and` /
`classical_or` still load as `FnExternalBody` scaffolds because
their RHS uses `True` / `False` as expressions.

### Class 5 gap 3: Data body parsing

**What's missing.** `ValueBody::Unparsed` preserves the body
source span but doesn't make the body structurally consumable.
`kernel_algebra_profile` / `kernel_type_set` / etc. — the
load-bearing data tables — still aren't readable by the
compiler at M1(2.7).

**Why it's deferred.** Parsing record/map/list literal
`SurfaceExpr`s is a full M2 parser extension: new `SurfaceExpr`
variants, new `lower_expr` paths, new `Value`-shaped behavior
nodes for composite values, infer support for structurally
typing record values against their annotation. Substantial.

**M1(3) PR-B partial close.** PR-B added the **literal-only**
slice of this gap: `SurfaceExpr::Record { fields }`,
`ValueBody::Structural { fields: Vec<(String, LiteralBits)> }`,
`lower_record_to_structural` inhabitance checking, and the
`src/v3/spec/rust.dag` consumer that reads
`(target_name, op_name, carrier, cost)` literal fields off
realization records. The substrate now structurally consumes
data bodies whose field values are scalar literals (Int / Bool
/ String). PR-B's Realization meta-type was deliberately
designed to fit inside that scope. The remaining gap is **port-
carried field values** (a record field whose value is another
declaration reference, a nested record, a list literal, or a
`Var` of an outer binding). When the next consumer needs one of
those, `ValueBody::Structural { fields }` upgrades from
`Vec<(String, LiteralBits)>` to `Vec<(String, PortId)>` (or
similar), and the literal-bits inline form becomes a special
case — not a separate variant.

**Current status.** Literal-field bodies: ✅ closed at M1(3). Scalar `data x: T = v` bodies: ✅ closed at PR #496 (3a.2 via `ValueBody::Scalar`; see DB-10 in [`design-m2-feature-parity.md`](../../docs/design-m2-feature-parity.md)).
Port-carried field bodies: ⏸ deferred to the M2+ consumer that
forces the upgrade.

### Class 5 gap 5: `where` clause refinement facts

**Status (as of 2026-04-17):** partial — parser foundation + `Declaration.refinement: Option<DeclarationId>` substrate edge landed via PR #496. Full semantics (predicate lowering, call-site structural-DAG check, Branch-arm narrowing extension) deferred; now tracked as **DB-11** in [`docs/design-m2-feature-parity.md`](../../docs/design-m2-feature-parity.md) and as **Deferral: 3a.3-full** in [`ROADMAP.md` §Active deferrals](./ROADMAP.md).

The original entry is preserved in the historical enumeration file; the active tracker for remaining work is the DB/ROADMAP pair. This is the canonical pattern: once a class-5 entry lands a dedicated DB doc, its live description lives there — not duplicated here.

## Historical enumeration — preserved verbatim

The original substrate-consumer gap enumeration (Consumer 1 through Consumer 4, Methodology, M1(2.7) Resolution summary, Round 9 / M1(2.8) / R16 audit sections, Write-pipeline gap summary, Substrate cross-reference) has been moved to [`docs/history/v3-substrate-consumer-enumeration-m1-2-7-to-2-8.md`](../../docs/history/v3-substrate-consumer-enumeration-m1-2-7-to-2-8.md) as part of the docs-pruning pass on 2026-04-17. All 14 original gaps closed structurally at M1(2.7); R9 and M1(2.8) addendums landed; the R16 scaffold-boundary audit completed. Retained there for audit traceability — not for active consumption.

Active substrate-consumer gaps (Class 5 items above, class-6 tokenizer escapes, class-7 realization narrowing, PR-B validation summary) remain in this file. When a new lens or surface form joins the pipeline, run the enumeration discipline against it; the history file shows the methodology.

## The meta-pattern

Every review round on PR #445 has flagged a variant of the same
shape: **one substrate field carrying multiple downstream jobs, with
the discriminator living in a sibling field (usually a string) — or
the dual: two fields carrying parallel authorities for one fact, with
the discriminator being which call site you happen to be in.**

- Round 3: `Identifier { name, resolved: Option<_> }` — one variant
  held both the unresolved-phase and resolved-phase state, with
  `Option` as the discriminator. Fix: split into
  `UnresolvedIdentifier` / `ResolvedIdentifier`.
- Round 5: `TypeShape::Primitive(Prim)` — an M0-inherited coarse tag
  parallel to the full `DeclarationId`-based type identity. Fix:
  replace with the newtype.
- Round 6: `ArrowBody::Pending` (partial) — a mixed-lifecycle slot
  held both "user body not yet lowered" and "external realization
  lag." Fix: delete the `lower_fn_item_pending` path; body-less fns
  have no declaration at all.
- Round 7: flat namespace via `declaration_by_name` — a single
  HashMap held both user and bootstrap declarations with no module
  layer. Fix: deferred to M2.

The *next* occurrences, visible in the combined enumeration above:

- **`AtomPayload::UnresolvedIdentifier(String)` holds both the
  forward-reference job (Q3 intake) and the operator-dispatch job
  (Q3 + Q4 consumption).**
- **`SurfaceItem::Fn.body: Option<SurfaceExpr>` holds both "has
  expression body" and "had block body which was skipped" (QW1).
  The consumer uses the Option discriminator to decide whether to
  lower into a declaration or silently drop the item.**
- **`TemplateArgument.parameter: DeclarationId` holds both
  "references a real TypeParam atom" and "self-reference stub
  tolerance" (QW4). The consumer distinguishes by checking
  whether the template was a stub at construction time.**
- **Port type resolution authority is split between
  `type_to_declaration_id` (structural, used inside declarations)
  and `lower_type_for_port` (whitelist, used for port annotations),
  discriminated by the call site rather than by type (QW5).**

Both the Round 3 split (Identifier → Unresolved/Resolved) and the
Round 5 dissolution (TypeShape::Primitive → newtype around
DeclarationId) were *localized* fixes. They resolved the specific
field they targeted. They didn't prevent the same pattern from
re-appearing in an adjacent field a round later.

Zooming out, the full gap list partitions into four fact-placement
classes:

1. **Primitive type identity** — Q1, Q2, Q4, QW5 all ask "given
   that something is of a primitive kind, what is its
   `TypeShape`?" Each site answers via a string name lookup or a
   hardcoded whitelist. Root cause: there is no substrate-level
   edge from primitive-kind to DeclarationId.

2. **Operator dispatch classification** — Q3, Q4 ask "is this
   Transform target an operator, and if so which family?" Root
   cause: operators share a variant shape
   (`AtomPayload::UnresolvedIdentifier`) with user-code forward
   references, so the consumer must re-parse the string at
   dispatch time.

3. **Scaffold honesty** — QW1, QW2, QW4 all involve substrate
   positions that *should* carry a scaffold form but instead
   silently discard the underlying fact. `ArrowBody::Pending` is
   the prototype of a scaffold done right (named variant,
   tracked ratchet, dissolution trigger). The substrate needs
   analogous scaffold-with-trigger forms for block-bodied fns,
   data decls, and stub template arguments.

4. **Parallel authorities** — Q8, QW5 both involve two code
   paths answering the same question ("what exists in the type
   system?") with different levels of structural sophistication.
   Q8's is local (a safety net on ExternalRealization); QW5's
   is systemic (every port annotation sees a different type
   system than every declaration annotation).

Each class has a different structural fix. The next substrate PR
addresses them as classes, not as individual symptoms.

**This document does not propose a fix shape.** There are several
plausible directions for each gap, and the value of enumerating now
is to hold all the gaps in one place so the fix can address the
class rather than the symptom. Candidates to consider when the
substrate PR is drafted:

**Class 1 — primitive type identity** (Q1/Q2/Q4/QW5)
- A `Dag` primitive cache with `dag.int_shape()` / `dag.bool_shape()`
  / `dag.string_shape()` populated at bootstrap (cheap, partial).
- A `LiteralBits → DeclarationId` structural edge on
  `Dag` or on `LiteralBits` itself.
- Dissolve `lower_type_for_port` into a call to
  `type_to_declaration_id` wrapped as `TypeShape` — ports inherit
  the declaration authority.

**Class 2 — operator dispatch** (Q3/Q4)
- Promote operator calls to a distinct `TransformTarget` or
  `Behavior` shape at lowering time (commits to a parser change).
- Desugar operators to fully-qualified algebra field calls at
  parse time (`1 + 2` → `std::algebra::add(1, 2)`), dissolving the
  operator concept entirely.
- Split `AtomPayload::UnresolvedIdentifier` into
  `UnresolvedUserIdentifier(String)` +
  `OperatorSymbol(&'static str)` so the phase distinction is in
  the type.

**Class 3 — scaffold honesty** (QW1/QW2/QW4)
- Split `SurfaceItem::Fn` into two variants (expression-body vs
  external-body-scaffold) so the type system distinguishes them.
- Promote `absorb_data_item` to a real `SurfaceItem::Data` variant;
  lower either to a full declaration or to a scaffold declaration
  with tracked dissolution.
- Delete the `TemplateArgument` stub branch: `template_is_stub`
  means the template already has a diagnostic attached, so
  emitting nothing is correct. Or: wrap `parameter` in a
  `TypeParamId` newtype constructable only from a TypeParam atom.
- Extend §8.11-style ratchet tracking to every scaffold form.

**Class 4 — parallel authorities** (Q8/QW5)
- Declare meta-type roles as substrate edges rather than named
  declarations (addresses Q8, may dovetail with unused `inhabits`
  field).
- Delete the port-type whitelist; route through the declaration
  authority (addresses QW5, also resolves Class 1 for compound
  types).

Each has tradeoffs the reviewers have not yet had a chance to weigh
in on, and the right answer is probably a combination rather than a
single move. The next substrate PR picks from this menu with the
full gap list visible.

## Scope note

This enumeration covers both halves of the pipeline:

- **Read side** (Consumers 1–3): `infer.rs`, `lens_depth.rs`,
  `lens_provenance.rs` — readers of the substrate.
- **Write side** (Consumer 4): `lower.rs` viewed as a consumer of
  `parse.rs` output, with its `SurfaceItem` → declaration /
  behavior / port lowering treated as a question-and-answer
  boundary.

The write side was added after the initial pass because a reviewer
surfaced five gaps there; the lesson is that the enumeration
discipline applies symmetrically to readers and writers, and the
next enumeration pass (new lens or new surface form) should cover
both halves from the start.

Not yet covered: `bootstrap.rs` (another producer), the test files,
and the §8.11 `Pending` ratchet consumer (doc-only today). When the
cost lens (M1(3)) and ownership lens land, re-run the enumeration
end-to-end: each new consumer tests whether the substrate's answers
to its questions are structural or bridged. A lens that needs a
bridge is the substrate's signal to extend.
