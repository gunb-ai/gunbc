# DOWNSTREAM_REQUIREMENTS — v3 substrate consumer enumeration

**Status: all 14 original gaps resolved at M1(2.7); class 5
remains open with M1(3) PR-B partial closes.** This document was
originally drafted diagnostic-only during PR #445 review. It
catalogued every structural question that v3's consumers ask about
a `Node`, `Declaration`, or `Port`, and cross-referenced each
question against the fields the substrate exposed. The gap list
became the spec for the M1(2.7) fix PR, which resolved all 14
entries structurally in one coherent substrate change (rather than
reactive per-reviewer fixes). The enumeration sections below are
preserved verbatim as historical record; the resolution summary
maps each gap to its fix.

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

## Resolution summary (M1(2.7))

| Gap | Location (pre-M1.2.7) | Resolution |
|-----|-----------------------|------------|
| **Q1** Literal → primitive TypeShape | `infer.rs:107` `primitive_shape(dag, "Int\|Bool\|String")` | `Dag::int_shape()` / `bool_shape()` / `string_shape()` cached accessors populated at bootstrap |
| **Q2** Branch input must be Bool | `infer.rs:134` `primitive_shape(dag, "Bool")` | Same as Q1 — cached accessor |
| **Q3** Is target an operator? | `infer.rs:449` `unresolved_operator_name(decl)` inspecting `AtomPayload::UnresolvedIdentifier` payload | `TransformTarget { Callable(DeclarationId), Operator(OperatorKind) }` coproduct on `TransformNode.target`; operators never allocate stub declarations |
| **Q4** Arithmetic vs comparison operator? | `infer.rs:503` `is_comparison_operator` string match | `OperatorKind { Arithmetic(ArithmeticOp), Comparison(ComparisonOp) }` variant split encodes output-type rule |
| **Q5** Callable Arrow signature | `resolve_arrow_walk` structural walk | Already structural — no change |
| **Q6** ArrowBody variant dispatch | enum match | Already structural — no change; `Unparsed` variant added for QW1 |
| **Q7** TypeParam substitution | `SubstStack` | Already structural — no change |
| **Q8** ExternalRealization target validation | `is_realization_shape` name compare to `"Realization"` | Cached `Dag::realization_meta_id()` DeclarationId compare |
| **Q9** Transform target display name | rendering-only | No change; new `transform_target_display_name` helper dispatches on `TransformTarget` variant |
| **Q10** Port state | `PortState` enum | Already structural — no change |
| **QW1** Block-body `fn` discarded | `parse.rs:513-525` → `lower.rs:150,305-313` fail-open skip | `SurfaceItem::FnExternalBody` sibling variant; lowers to `ArrowBody::Unparsed(body_span)` declaration; signature flows forward |
| **QW2** `data` decls absorbed | `parse.rs:426-434` `absorb_data_item` throws facts away | `SurfaceItem::Data { name, ty, body_span }` emitted; lowers to declaration whose connective resolves from `ty` |
| **QW3** `module`/`import` absorbed | `parse.rs:391-419` | `SurfaceItem::Module { path }` / `SurfaceItem::Import { path, names }` emitted; lower to no-op; parsed facts preserved |
| **QW4** TemplateArgument stub self-ref | `lower.rs:664-672` `TemplateArgument { parameter: value, value }` | Stub branch deleted; `build_template_arguments` returns `Vec::new()` for stub templates |
| **QW5** `lower_type_for_port` whitelist | `lower.rs:988-1014` `"Int"\|"Bool"\|"String"` | Routed through `type_to_declaration_id`; fresh-stub fail-closed check preserves port diagnostic anchoring |

**Test coverage.** Eight `m17_*` regression tests in
`tests/m1_substrate_test.rs` verify the primitive cache, operator
dispatch shape (arithmetic + comparison + user-callable), block-body
scaffold, data declaration, module/import preservation, and the
TemplateArgument stub-branch deletion. Plus five `m17_r9_*` tests
for round 9 (see below). Total: 65/65 green, clippy clean.

## Round 9 — ChatGPT review, post-M1(2.7) landing

After the initial M1(2.7) commit landed on PR #445, a deeper
ChatGPT review flagged two structural gaps that the first fix
didn't fully close:

### R9-A: Operators still bypass declaration-backed arrows

**Pre-R9 state.** `resolve_operator_arrow` fabricated
`(T, T) -> T` / `(T, T) -> Bool` from `OperatorKind` without
consuming `std/algebra.dag`. The structural coproduct
`TransformTarget::Operator(OperatorKind)` was better than the
old string bridge, but Rust still owned the signature semantics.

**R9 fix.** Two parts:

1. **Extended `std/algebra.dag`** — `OrderedRing<T>` gained
   direct operator fields (`sub`, `div`, `eq`, `ne`, `lt`, `le`,
   `gt`, `ge`) so every arithmetic/comparison operator maps to
   a named algebra field with a declared Arrow signature. No
   grammar extension — just additional field declarations using
   the existing `name: fn(T, T) -> T / Bool` shape. The runtime
   semantics (sub = add + negate, lt = compare == Less) still
   live in the realization layer; the field declaration is the
   compiler's *signature authority*, not the implementation.
2. **Rewrote `resolve_operator_arrow` as a structural walk.**
   It walks the LHS type's declaration chain through
   `Instantiation`/`ResolvedIdentifier` edges to an algebra
   `Conj`, looks up the operator's field by name (via
   `OperatorKind::algebra_field_name()`), reads the field's
   `Arrow` from the declaration graph, and substitutes the
   algebra's receiver type parameter to the source declaration
   (not the template argument) so user-facing ports match. See
   `infer.rs::read_algebra_field` and `substitute_receiver`.

**Result for Int.** `let x = 1 + 2` walks
`Int → Int64 → OrderedRing<Word64>`, finds
`OrderedRing.add: fn(T, T) -> T`, substitutes `T → Int`,
returns signature `(Int, Int) -> Int` matching user ports. The
compiler consumes `std/algebra.dag` as authority rather than
fabricating in Rust. Same for `-`, `*`, `/`, `==`, `!=`, `<`,
`<=`, `>`, `>=`.

### R9-B: Data items dropped their body

**Pre-R9 state.** `lower_data_item` set the declaration's
`connective` = the annotated type, making
`data foo: Int = {...}` structurally identical to
`type foo = Int`. Reviewer's thesis point: values are
structural inhabitants, not another copy of the type connective.

**R9 fix.** Added `Declaration.value_body: Option<ValueBody>`
with `ValueBody::Unparsed(SourceSpan)` as the M1(2.7) scaffold
form. `lower_data_item` sets both:
- `connective` = the resolved type annotation (so the type fact
  is accessible).
- `value_body = Some(Unparsed(body_span))` (so the body is
  preserved AND the declaration is structurally distinct from a
  type alias).

`ValueBody::Unparsed` has an explicit M2+ dissolution trigger
(record/map/list literal `SurfaceExpr` parsing). Until then the
body span is preserved and consumers can discriminate
"data value" from "type alias" by reading `value_body`.

## M1(2.8) — match expressions + Branch generalization

After R9 landed, the next increment tackled match expression
parsing so v3's parser begins catching up to v2's grammar surface.
The work was framed by the user as "parser catch-up, not design
discovery" — v2 has match parsing; v3 didn't yet; writing it is
just the next increment.

### M1(2.8) fix

- **Branch input generalization.** The M0-era "Branch input must
  be `Bool`" check is the degenerate special case of "Branch
  input must be a `Disj`." Bool, Classical, user-defined sum
  types, etc. all satisfy `TypeConnective::Disj`. String, Int,
  Float, etc. don't — so `if "foo" then ...` still fails the
  same way it did at M0. This is a widening, not a compromise:
  Branch becomes "dispatch on any sum type," which is what it
  should have meant from the start.
- **`BranchPattern` + `Path.pattern`.** Each `Path` now carries
  an explicit variant discriminator. Phase coproduct:
  `UnresolvedVariant { name, span }` at lower time →
  `ResolvedVariant(DeclarationId)` after a new infer-time
  pattern resolution pass. Same shape as
  `AtomPayload::Unresolved/ResolvedIdentifier`. Replaces the
  positional convention (`paths[0] = then`, `paths[1] = else`)
  with structural labels.
- **`if`/`else` lowering rewired.** Generates explicit
  `UnresolvedVariant { name: "True" }` and `{ "False" }` arms
  instead of relying on positional convention. Resolution walks
  `Bool`'s `Disj` children.
- **`SurfaceExpr::Match`, `SurfacePattern::BareVariant`, parser
  for `match <expr> { <Ident> => <expr> ... }`.** At M1(2.8)
  patterns are limited to bare variant constructors — wildcards
  (`_`), record destructure (`Some { value: x }`), and nested
  patterns are deferred to M1(2.9)+.
- **Pattern resolution pass** in `infer.rs`. After the main
  inference loop converges, walks each Branch and rewrites each
  `UnresolvedVariant.name` against the scrutinee's `Disj`
  children (scoped, so `Classical.True` and `Bool.True` don't
  conflict). Unknown variant names fail-closed with a diagnostic
  on the Branch's output port.

Coverage: 41 M0 + 22 M1 substrate (17 R7/R9 + 5 new M1(2.8)
match tests) + 7 real-stdlib smoke + 1 realization smoke = 71
green. Clippy clean.

### M1(2.8) does NOT dissolve logic.dag's FnExternalBody scaffolds

`logic.dag` has three functions (`classical_not`, `classical_and`,
`classical_or`) whose bodies use match. Match parsing alone
doesn't dissolve their `FnExternalBody` scaffolds because the arm
bodies contain **bare variant expressions** — the RHS `False` in
`True => False` is a free expression that needs to resolve to
`Classical.False`, but R7's variant anonymization made variant
declarations unfindable via `declaration_by_name`. This is a
separate resolution problem from match patterns (which resolve
scoped against the scrutinee type); variant RHS expressions have
no scrutinee context and need either a variant-constructor
surface syntax, a targeted anonymization rollback, or infer-time
re-resolution against the Branch output type.

This is tracked as **class-5 gap #4** below.

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

**Current status.** Literal-field bodies: ✅ closed at M1(3).
Port-carried field bodies: ⏸ deferred to the M2+ consumer that
forces the upgrade.

### Class 5 gap 5: `where` clause refinement facts

**What's missing.** `type_alias` items in `std/*.dag` can carry
a `where <constraint>(...)` clause that refines the alias's
valid value space — e.g. `type CommitSha = String where sha1(.)`
or `type Port = Int where in_range(1, 65535)`. The parser
accepts the syntax via `skip_where_clause` at
`src/v3/compiler/src/parse.rs:801` but drops the entire clause
span without lowering anything. The refinement fact doesn't
survive parse → lower, and downstream consumers treat
`CommitSha` as a plain alias for `String` with no validation.

**Why it's hard.** Refinement is a Tier-2 safety concept — the
declaration needs to carry a predicate reference, lowering
needs a place to put it on the `Declaration` struct, and
inference needs to check the predicate at every value-boundary
where the refined type is produced. It's neither a 🟡 scaffold
(the fact is lost, not tracked) nor a simple parser extension
(lowering has no target shape for "refinement predicate"
yet).

**Options.** Add a `Declaration.refinement: Option<RefinementSpec>`
field that carries the predicate name + span; lower
`where <name>(.)` clauses into it; downstream inference emits
a diagnostic at value-producing boundaries when the refinement
isn't enforceable. Or: lower refinements into a structural
`Conj` whose only child is the refined carrier type plus a
`Predicate` child — but that reshapes how refined types render.

**Current status.** `skip_where_clause` is the bridge.
Bootstrapped aliases like `CommitSha` and `Port` currently
behave as plain aliases; no tests exercise refinement
validation at M1(2.8). Flagged by codex in earlier reviews
(`5d6030a8`, `5d0fc6dd`) as a previously-flagged blocker that
remains open — tracked here as a class-5 gap so the next
substrate pass doesn't skip it.

## Scaffold boundary audit (M1(2.8) R16)

After R14 closed the `ArrowBody::Unparsed` / `ValueBody::Unparsed`
user-range leak, the natural follow-up question is whether
*every* scaffold variant in the substrate has an unreachability
gate. The new **Scaffold Boundaries** invariant in
`INVARIANTS.md` formalizes the rule; this audit walks every
scaffold variant in the v3 substrate and reports its boundary
status, so the next substrate PR that adds a variant has a
concrete checklist to extend.

The audit covers every "transient" or "scaffold" variant in
the substrate — variants that represent "this position has a
fact the compiler cannot yet validate" or "this position is
awaiting a later resolution pass." Terminal variants
(`UserDefined`, `ExternalRealization`, `ResolvedIdentifier`,
`ResolvedVariant`, `Literal`, `TypeParam`, etc.) are not in
scope.

| Scaffold | Producers | User-reach? | Boundary | Test |
|---|---|---|---|---|
| `ArrowBody::Unparsed(SourceSpan)` | `lower_fn_item_unparsed` | Yes (block-body fn in user source) | R14's `reject_user_unparsed_scaffolds` sweep | `m18_r14_user_block_bodied_fn_is_rejected` |
| `ValueBody::Unparsed(SourceSpan)` | `lower_data_item` | Yes (data item in user source) | R14's sweep (same function) | `m18_r14_user_data_with_opaque_body_is_rejected` |
| `AtomPayload::UnresolvedIdentifier(String)` | `alloc_identifier_stub` | Yes (unknown type / callable name in user source) | `resolve_pending_identifiers_strict` fails-closed for user-range stubs that cannot resolve | `reviewer_unknown_type_name_is_rejected` |
| `BranchPattern::UnresolvedVariant { name, span }` | `lower_expr::Match`, `lower_expr::If` | Yes (match arm / if-then-else) | `resolve_branch_patterns` inside the infer fixpoint; unknown variant fails closed, coverage check fails closed | `m18_match_with_unknown_variant_is_rejected`, `m18_r11_non_exhaustive_match_is_rejected`, `m18_r11_duplicate_match_arm_is_rejected` |
| `ArrowBody::Pending` | `type_to_declaration_id` (Arrow arm), `resolve_operator_arrow` fallback, algebra field signatures | No (latent at M1(2.8)) | Implicit via grammar — first-class function values are not callable at M1(2.8), so Pending arrows produced by user type annotations are never reached by `decide_transform`; Pending arrows produced by the operator fallback bridge return through `decide_transform`'s Pending arm which legitimately accepts "no body needed"; algebra field arrows are only walked structurally, never dispatched | N/A (no user-reachable scenario) |

### Findings

1. **`ArrowBody::Pending` dissolution ledger was inaccurate.**
   The earlier ledger described Pending as "bootstrap realization
   lag" that would dissolve via a §8.11 monotonic-decrease ratchet
   when every Pending arrow binds to `ExternalRealization`. But
   production bootstrap has zero realization-lag arrows and
   hundreds of Pending arrows in three legitimate "no concrete
   body required" roles (bootstrap algebra field signatures,
   user Arrow type annotations, operator fallback bridge). Fixed
   in this pass: ledger in `dag.rs` now honestly describes all
   three roles. No code change — the semantics are unchanged,
   only the receipt.

2. **`ArrowBody::Pending` has no explicit user-range rejection
   sweep**, but the boundary is implicit: the three production
   roles are either structurally not reachable via user dispatch
   (user type annotations — no first-class fn calls at M1(2.8))
   or legitimately scaffold by design (algebra field signatures,
   operator fallback). If M2 adds first-class fn call syntax,
   this becomes a reachability concern and the variant must
   either split (to distinguish "type annotation pending" from
   the other two uses) or grow a user-range rejection gate.

3. **`BranchPattern::UnresolvedVariant` has a latent edge case.**
   `resolve_branch_patterns` skips Branches whose input port is
   not Resolved. If an input port never resolves (cascaded
   upstream failure), the Branch's paths remain in
   `UnresolvedVariant` state. The Branch's output port cascades
   to Unresolved via `decide_transform`'s Branch arm, so
   downstream consumers don't read the stale pattern. But a
   future emitter reading `Path.pattern` directly for a Branch
   whose output is Unresolved would see stale data. **Mitigation:**
   emit consumers must gate on the Branch's output port state
   (`PortState::Resolved`), not on `Path.pattern` directly.
   Documented here so the emit-stage PR has a pre-flight check.

4. **Four of five scaffold variants now have explicit
   user-range rejection gates.** Only `ArrowBody::Pending` is
   bounded implicitly, and its implicit boundary is legitimate
   at M1(2.8). The Scaffold Boundaries invariant in
   `INVARIANTS.md` makes this ratchet permanent: any future
   substrate variant must land with its gate and test in the
   same PR.

### What this audit does NOT cover

- **Terminal variants.** `ArrowBody::{UserDefined,
  ExternalRealization}`, `AtomPayload::{Literal, TypeParam,
  ResolvedIdentifier}`, `BranchPattern::ResolvedVariant` — all
  require no boundary because they represent the fully-resolved
  state. The 4-pattern dissolution check on these is in the
  respective declaration ledgers.
- **Non-v3-substrate scaffolds.** Parse-surface scaffolds like
  `FnExternalBody`, `SurfacePattern::BareVariant`, etc. are
  pre-substrate and don't need boundary gates — they lower into
  substrate forms that do.
- **Data-body scaffolds beyond `ValueBody::Unparsed`.** When
  M2 adds `ValueBody::Structural(NodeId)`, that variant will
  be terminal and needs no boundary — but the transition away
  from `Unparsed` itself will be covered by the ratchet step
  in `INVARIANTS.md`.

---

Motivation: every review round on PR #445 has found the same bug
shape — **one field doing multiple jobs downstream**. The pattern
repeats because we have been editing the substrate reactively, one
reviewer at a time, without a single place to see what the consumers
actually need. An enumeration pass ahead of the next substrate
change breaks the cycle: fix the whole class, once, against a
visible requirement.

## Methodology

For each consumer file, walk every function end-to-end. Record each
distinct structural question the code asks: "is this port resolved?",
"what behavior produced this value?", "what is this transform
calling?", etc. For each question, record:

1. **The substrate path** — the sequence of field accesses (or name
   lookups) that answer it.
2. **Structural or bridge** — structural means the answer flows
   forward from substrate fields only. Bridge means the answer
   requires reading a string, scanning a name table, or doing a text
   match.

Gaps are the bridge rows. A gap is not necessarily an emergency — some
are localized, documented, and M2+ triggers exist. But every gap is a
place where one structural fact is spread across two representations
(the field and the text the consumer reads to interpret the field),
and that is the shape that generates review rounds.

## Consumer 1: `lens_depth.rs`

74 lines. One query: `DepthLens::depth_of(PortId) -> usize`.

| Question | Substrate path | Kind |
|----------|----------------|------|
| Who produced this port? | `port.produced_by: Option<NodeId>` | ✓ structural |
| What kind of behavior produced it? | `dag.node(id)` variant match | ✓ structural |
| For `Value`, what is the depth? | short-circuits to 0 | ✓ structural |
| For `Transform`, which inputs contribute? | `t.inputs: Vec<PortId>` | ✓ structural |
| For `Branch`, which ports contribute? | `b.input`, `b.paths[].output` | ✓ structural |
| For `Loop`, which ports contribute? | `l.source`, `l.init` | ✓ structural |
| For `Bind`, which port contributes? | `b.value` | ✓ structural |

**Zero name bridges. Zero string inspection. The whole lens is a
depth-first walk over typed id edges.** This is the v3 success bar
working as intended.

## Consumer 2: `lens_provenance.rs`

77 lines. One query: `ProvenanceLens::origin_of(PortId) -> Origin`.

| Question | Substrate path | Kind |
|----------|----------------|------|
| Who produced this port? | `port.produced_by: Option<NodeId>` | ✓ structural |
| What kind of origin is this? | `dag.node(producer)` variant match | ✓ structural |

Identical shape to `lens_depth.rs`. Zero bridges.

The lens carries a judgment-call enum `Origin { Source, Computed,
Selected, Accumulated }` which its own dissolution receipt already
flags as a Pattern-1 candidate (four variants are redundant with
`Behavior` variants that consumers can query directly). Kept at M0
for readability. Not a substrate gap — a lens-side compression debt
tracked in the file's own ledger.

## Consumer 3: `infer.rs`

654 lines. Four interlocking queries:

- `infer(dag)` — fixpoint driver. Calls `decide` per node.
- `decide(dag, index) -> Decision` — dispatches on `Behavior` variant.
- `decide_transform(dag, t) -> Decision` — resolves the called
  arrow's signature and checks arity + per-input types.
- `resolve_arrow_walk` / `walk_to_type_shape` — structural walks
  across type connectives.

The distinct structural questions, ordered by where they appear:

### Q1. Value literal → primitive type identity

`src/v3/compiler/src/infer.rs:113-131`

```rust
Behavior::Value(v) => {
    let name = match &v.data {
        LiteralBits::Int(_) => "Int",
        LiteralBits::Bool(_) => "Bool",
        LiteralBits::String(_) => "String",
    };
    let Some(ty) = primitive_shape(dag, name) else { ... };
    Decision::Set(v.output, ty)
}
```

**Substrate path:** the `LiteralBits` variant is pattern-matched to
produce a string literal (`"Int"`, `"Bool"`, `"String"`), which
`primitive_shape` then scans through `Dag::declaration_by_name` to
recover the `DeclarationId`. Two hops: `variant → string literal →
HashMap lookup`.

**Gap type:** NAME BRIDGE. The fact "an Int literal has type Int" is
spread across (a) the `LiteralBits::Int` variant tag and (b) a string
comparison of the declaration table. The substrate has no direct edge
from the literal-kind to the primitive declaration it inhabits.

### Q2. Branch input type must be Bool

`src/v3/compiler/src/infer.rs:134`

```rust
let Some(bool_ty) = primitive_shape(dag, "Bool") else { ... };
```

**Substrate path:** same as Q1. A string literal scans the
declaration table every time a branch is checked.

**Gap type:** NAME BRIDGE. Same root cause as Q1. The fact "Branch
input is Bool-typed" is enforced by having the consumer re-derive
`Bool`'s `TypeShape` at every check site.

### Q3. Is the transform target an operator?

`src/v3/compiler/src/infer.rs:244, 449-456`

```rust
let target_decl = dag.declaration(t.target);
let signature = if let Some(op_name) = unresolved_operator_name(target_decl) {
    ...
};

fn unresolved_operator_name(decl: &Declaration) -> Option<&str> {
    if let TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) = &decl.connective {
        if operator_field_name(name).is_some() {
            return Some(name.as_str());
        }
    }
    None
}
```

**Substrate path:** the consumer reaches into the declaration's
connective, pattern-matches on `AtomPayload::UnresolvedIdentifier`,
extracts the String, and consults `OPERATOR_FIELD_MAP` to determine
whether the string names an operator.

**Gap type:** NAME BRIDGE, and the **load-bearing one**. It is
doing two jobs with one field:

1. `AtomPayload::UnresolvedIdentifier(String)` is meant to represent
   *"a name reference not yet resolved by the two-phase lowering
   sweep."* Downstream `resolve_pending_identifiers` either promotes
   it to `ResolvedIdentifier` or emits a fail-closed diagnostic.
2. `AtomPayload::UnresolvedIdentifier(String)` is also used as a
   *permanent* dispatch target for operators, because the resolution
   sweep has an exception that leaves operator strings unresolved and
   infer.rs resolves them on its own path at dispatch time.

The two uses share the same shape but have opposite lifecycles.
"Unresolved" means "waiting for lowering" for user identifiers and
"dispatched at a different layer" for operators. The string is the
*only* thing distinguishing the two cases downstream — the consumer
inspects the string content to decide which code path applies.

This is the pattern the user flagged as recurring: **one field
holding two facts, with text as the discriminator**.

### Q4. Operator output type: operand or Bool?

`src/v3/compiler/src/infer.rs:491-505`

```rust
let output = if is_comparison_operator(op_symbol) {
    primitive_shape(dag, "Bool")?
} else {
    *lhs_type
};

fn is_comparison_operator(op: &str) -> bool {
    matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=")
}
```

**Substrate path:** another string match, this time partitioning
the operator symbols into "arithmetic" (returns the operand type)
and "comparison" (returns Bool). `primitive_shape(dag, "Bool")`
resurfaces on the Bool branch.

**Gap type:** NAME BRIDGE, layered on top of Q3. Having identified
that the symbol is an operator, the consumer now asks "which family
of operator?" and answers by another hardcoded string match. The
fact "arithmetic operators return the operand type, comparisons
return Bool" has no structural expression in the substrate.

### Q5. User callable → Arrow signature

`src/v3/compiler/src/infer.rs:525-577`

```rust
fn resolve_arrow(dag: &Dag, target: DeclarationId) -> Option<ResolvedArrow> {
    let mut subst = SubstStack::new();
    resolve_arrow_walk(dag, target, &mut subst, 0)
}

fn resolve_arrow_walk(...) -> Option<ResolvedArrow> {
    match &decl.connective {
        TypeConnective::Arrow { inputs, output, body } => { ... }
        TypeConnective::Instantiation { template, arguments } => {
            subst.push(arguments.clone());
            let result = resolve_arrow_walk(dag, *template, subst, depth + 1);
            subst.pop();
            result
        }
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
            resolve_arrow_walk(dag, *next, subst, depth + 1)
        }
        _ => None,
    }
}
```

**Substrate path:** walks `TypeConnective::{Arrow, Instantiation,
Atom(ResolvedIdentifier)}` by typed id, with `SubstStack` handling
parameterized templates.

**Gap type:** ✓ structural. This is the clean half — the walk
dispatches on enum variants only and follows typed edges the whole
way down. No name comparisons. `SubstStack` is the substrate answer
for "what is this TypeParam bound to?" and it works.

### Q6. ArrowBody variant dispatch

`src/v3/compiler/src/infer.rs:302-357`

```rust
match &signature.body {
    ArrowBody::UserDefined(bind_id) => { ... check bind's value port ... }
    ArrowBody::ExternalRealization(realization_id) => {
        if !is_realization_shape(dag, *realization_id) { ... }
    }
    ArrowBody::Pending => { /* scaffold, skip body-walk */ }
}
```

**Substrate path:** enum variant match + per-variant follow-up
queries (`dag.node(bind_id).as_bind()`, `is_realization_shape`).

**Gap type:** mostly structural. The variant match is clean. The
follow-up query `is_realization_shape` is a partial bridge (see Q8).
`ArrowBody::Pending` is scaffold slated for dissolution via the
§8.11 ratchet by M3 — already tracked, not a new gap.

### Q7. TypeParam substitution

`src/v3/compiler/src/infer.rs:413-438, 604-607`

```rust
struct SubstStack {
    frames: Vec<Vec<TemplateArgument>>,
}

impl SubstStack {
    fn lookup(&self, param_id: DeclarationId) -> Option<DeclarationId> {
        for frame in self.frames.iter().rev() { ... }
    }
}

// walk_to_type_shape:
TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
    let bound = subst.lookup(current)?;
    walk_to_type_shape(dag, bound, subst, depth + 1)
}
```

**Substrate path:** `TemplateArgument { parameter, value }` edges
stored on `Instantiation.arguments`, read via the substitution
stack.

**Gap type:** ✓ structural. The substrate correctly carries the
parameter → value binding as a typed id pair. TypeParam lookup uses
the stack; no name comparison.

### Q8. ExternalRealization target validation

`src/v3/compiler/src/infer.rs:511-520`

```rust
fn is_realization_shape(dag: &Dag, realization_id: DeclarationId) -> bool {
    let decl = dag.declaration(realization_id);
    if !matches!(decl.connective, TypeConnective::Conj { .. }) {
        return false;
    }
    let Some(meta_tag) = decl.meta_tag else {
        return false;
    };
    dag.declaration(meta_tag).name.as_deref() == Some("Realization")
}
```

**Substrate path:** shape check via `TypeConnective::Conj` variant
(structural) + `meta_tag` edge follow (structural) + the meta-type
declaration's `name` compared against `"Realization"` (name bridge).

**Gap type:** NAME BRIDGE, localized. The `Realization` meta-type is
defined in `dsl/std/types.dag` and its identity is read by the
consumer via the declaration's string name. This is a safety-net
check duplicating `bootstrap::assert_realization_shape`, but it
reaches for a name to ask "is this a Realization?" — the same
pattern as Q2 in miniature.

### Q9. Transform target display name (rendering only)

`src/v3/compiler/src/infer.rs:621-633`

**Substrate path:** `decl.name` → falls through to
`UnresolvedIdentifier` payload → falls through to `declaration#N`
fallback. Used only for diagnostic rendering, not dispatch.

**Gap type:** rendering-only. Not a bridge for typechecking; the
failure mode is a less-helpful error message, not a soundness hole.
Noted for completeness. A future `DiagnosticRenderer` could take
this over.

### Q10. Port state read / write

Throughout. `dag.port(p).state()` returns `PortState::{Uninferred,
Resolved(TypeShape), Unresolved}`. Writers use
`Decision::{Set, Fail, Retry}`.

**Gap type:** ✓ structural. The three-state enum + fail-closed
biconditional is the substrate's flagship correctness guarantee and
it holds by construction.

## Consumer 4: `lower.rs` (write-pipeline)

**Scope note added after the first pass.** The initial enumeration
covered only *readers* of the substrate (`infer.rs`,
`lens_depth.rs`, `lens_provenance.rs`). A ChatGPT review of PR #445
surfaced five additional gaps in the **write pipeline** — the
boundary between `parse.rs` output (`SurfaceItem` / `SurfaceType`)
and `lower.rs` output (the declaration table + behavior nodes).
Writers are consumers too: each `SurfaceItem` variant represents a
question `lower.rs` must answer ("what declarations and nodes does
this surface form produce?"), and each answer is either structural
or a bridge. The enumeration discipline applies to both sides.

This section extends the original enumeration. The findings
catalogued here came from a reviewer; the methodology going
forward is to run the write-pipeline pass **proactively** before
shipping any substrate change.

### QW1. Block-bodied `fn` items drop at the lower boundary

`src/v3/compiler/src/parse.rs:513-525`:

```rust
TokenKind::LBrace => {
    let end = self.skip_brace_balanced()?;
    Ok(SurfaceItem::Fn {
        name,
        params,
        return_type,
        body: None,
        span: SourceSpan::new(self.file, fn_kw.span.byte_start, end),
    })
}
```

`src/v3/compiler/src/lower.rs:150`:

```rust
SurfaceItem::Fn { body: None, .. } => continue,
```

`src/v3/compiler/src/lower.rs:305-313`:

```rust
} else {
    // Block-body form (`fn f(x) -> T { body }`) — skipped
    // at collect_symbols time. `lower_item` should never
    // reach this arm because `is_first[idx]` is false for
    // block-bodied fns (no declaration was allocated). The
    // arm exists to keep the match exhaustive. If inference
    // reaches it somehow, return scope unchanged.
    scope
}
```

**Substrate path:** the parser turns a block-bodied fn into
`SurfaceItem::Fn { body: None, ... }`. `collect_symbols` pattern-
matches on `body: None` and *skips the item entirely* — no
declaration allocated, no diagnostic emitted. `lower_item` has a
dead arm for the same case that also returns silently.

**Gap type:** FAIL-OPEN DATA LOSS. The parser sees the fact "this
item had a block body" and throws it away. No declaration reaches
the substrate. No diagnostic reaches the user. A v3 program that
declares `fn foo(x: Int) -> Int { x + 1 }` compiles to a DAG with
no trace of `foo` and no error explaining why.

**Pattern:** identical to Round 3's
`Identifier { name, resolved: Option<_> }` shape. One field
(`body: Option<SurfaceExpr>`) carries a phase coproduct
(expression-body vs block-body-that-was-skipped), and the
discriminator lives in the `Option`. Downstream code reads the
discriminator to choose between two completely different code
paths (lower the body vs silently drop the item). The phase
distinction is already known at parse time — it was a branch on
`TokenKind::Eq` vs `TokenKind::LBrace` — but the surface form
collapses the distinction back into an Option that consumers then
re-inspect.

### QW2. `data` declarations absorbed at parse time

`src/v3/compiler/src/parse.rs:426-434`:

```rust
fn absorb_data_item(&mut self) -> Result<(), Diagnostic> {
    self.expect_kind(TokenKind::KwData)?;
    self.parse_ident()?;
    self.expect_kind(TokenKind::Colon)?;
    self.parse_type_expr()?;
    self.expect_kind(TokenKind::Eq)?;
    self.skip_brace_balanced()?;
    Ok(())
}
```

**Substrate path:** the parser consumes a complete `data name:
Type = { body }` declaration, throws every field away, and returns
`Ok(())`. The caller (`parse_item`) turns this into `Ok(None)` so
no `SurfaceItem` is emitted and `lower.rs` never sees the
declaration exists.

**Gap type:** FAIL-OPEN DATA LOSS, **load-bearing severity**. The
`dsl/std/*.dag` data declarations are the single-authority tables
the roadmap's FACTS FLOW FORWARD principle names directly:
`kernel_algebra_profile`, `kernel_type_set`,
`container_type_arity`, etc. These are exactly the hardcoded
tables the substrate rework is supposed to move out of Rust and
into `.dag` declarations. In the current state they are parsed,
then silently erased, then bootstrap re-hardcodes the equivalent
facts elsewhere. This is the shape the roadmap text prohibits:

> Bootstrap fixtures that parallel `dsl/std/*.dag` are debt —
> delete them the moment the parser can consume the real files.

The parser *can* consume the files. Consumption drops the facts.

**Pattern:** same class as Round 3/QW1 but worse: QW1 at least
stores a null option that the consumer sees; QW2 has no surface
representation at all. The fact is killed inside the parser before
any other code has a chance to route it.

### QW3. `module` and `import` items absorbed at parse time

`src/v3/compiler/src/parse.rs:391-419`. `absorb_module_item` and
`absorb_import_item` parse and discard the same way
`absorb_data_item` does.

**Gap type:** FAIL-OPEN, lower severity than QW1/QW2. Module and
import semantics genuinely don't do anything at M1(2.6) — the flat
namespace via `declaration_by_name` hasn't been partitioned into
modules yet, so absorbing them is *effectively* correct today. But
the pattern is the same: the parser knows facts it then drops.
When M2 adds module scoping, the absorbed names will have to be
recovered by re-parsing. Cleaner to emit a `SurfaceItem::Module`
/ `SurfaceItem::Import` now (with lowering as a no-op) so the M2
work is additive.

Flag this as a **low-severity gap, not a blocker.** The current
behavior is not actively wrong; it just isn't future-proof.

### QW4. `TemplateArgument.parameter` admits self-reference

`src/v3/compiler/src/dag.rs:281-287` (field contract):

```rust
pub struct TemplateArgument {
    /// The template parameter being bound. References a TypeParam Atom declared
    /// as a child of the template.
    pub parameter: DeclarationId,
    /// The concrete type the parameter binds to.
    pub value: DeclarationId,
}
```

`src/v3/compiler/src/lower.rs:664-672` (constructor violation):

```rust
let parameter = if template_is_stub {
    // Stub tolerance: self-reference. The stub itself is
    // caught by `resolve_pending_identifiers`; this
    // TemplateArgument is either dead code (bootstrap
    // dangling ref) or unreachable (user-code strict
    // mode catches the stub before inference walks it).
    value
} else {
    match template_param_id(dag, template, idx) { ... }
};
TemplateArgument { parameter, value }
```

**Substrate path:** when the template declaration is an
unresolved stub, `build_template_arguments` constructs
`TemplateArgument { parameter: value, value }` — a self-reference
where `parameter` does not point at a TypeParam Atom.

**Gap type:** ILLEGAL STATE REPRESENTABLE. The substrate admits a
state the field's own doc comment says should never exist. The
comment argues the state is unreachable because
`resolve_pending_identifiers` catches the stub first, but this is
convention enforcement — the type system does not prevent
construction of a self-referential `TemplateArgument`, and future
readers (including `resolve_arrow_walk`'s `SubstStack`) must
tolerate the state or crash.

**Pattern:** the same "illegal state made representable for
scaffolding convenience" shape as Round 6's deleted
`lower_fn_item_pending` path, but without §8.11-style tracking.
`ArrowBody::Pending` has a ratchet; `TemplateArgument` stub
self-references have no visibility beyond this comment.

### QW5. `lower_type_for_port` primitive whitelist vs `type_to_declaration_id`

`src/v3/compiler/src/lower.rs:988-1014`:

```rust
fn lower_type_for_port(ty: &SurfaceType, dag: &Dag) -> Result<TypeShape, Diagnostic> {
    match ty {
        SurfaceType::Named { name, span } => match name.as_str() {
            "Int" | "Bool" | "String" => { ... }
            _ => Err(Diagnostic::ResolveError {
                name: format!("unknown type `{name}`"),
                span: span.clone(),
            }),
        },
        SurfaceType::Parameterized { span, .. }
        | SurfaceType::Optional { span, .. }
        | SurfaceType::Arrow { span, .. } => Err(Diagnostic::ResolveError {
            name: "compound type annotations are not yet supported in user code"
                .to_string(),
            span: span.clone(),
        }),
    }
}
```

Meanwhile `type_to_declaration_id` (same file, called from
`lower_type_record`, `lower_type_sum`, etc.) handles **all**
`SurfaceType` variants structurally: walks `symbols` + local
scope, allocates identifier stubs for unknowns, builds
`TemplateArgument`s for parameterized types.

**Substrate path:** two different paths answer "what `DeclarationId`
does this `SurfaceType` correspond to?":

- **Declaration-internal**: `type_to_declaration_id` —
  structural. Accepts every `SurfaceType` variant. Stubs on
  unknowns. Used by type record/sum/alias lowering.
- **Port-internal**: `lower_type_for_port` — whitelist. Accepts
  three hardcoded primitive names. Rejects every compound type.

**Gap type:** PARALLEL AUTHORITIES. The substrate already decided
"types are identified by `DeclarationId` walked through the
six-connective substrate." But the port subsystem runs a second
authority that enforces "ports can only be named primitives." A
user can declare `type Foo = { x: Int }`, but
`let y: Foo = ...` fails with "unknown type `Foo`" because the
port-side authority doesn't know about `Foo` even though
`type_to_declaration_id` could have resolved it from the same
symbol table.

**Pattern:** Round 5's `TypeShape::Primitive(Prim)` in a
different layer. Round 5 dissolved the coarse-primitive type tag
from the type substrate. QW5 leaves the same coarse primitive
authority in the port-type subsystem. Round 5's fix doesn't
compose forward: the type substrate got structural, but
`lower_type_for_port` didn't follow.

This is the "declarations say 'structural type', ports say
'primitive whitelist'" split the reviewer named.

## Write-pipeline gap summary

| Gap | Location | Kind | Pattern |
|-----|----------|------|---------|
| QW1 Block-body `fn` discarded | `parse.rs:513-525`, `lower.rs:150, 305-313` | FAIL-OPEN | Option-wraps a phase coproduct |
| QW2 `data` decls absorbed | `parse.rs:426-434` | FAIL-OPEN, load-bearing | Facts don't flow forward |
| QW3 `module`/`import` absorbed | `parse.rs:391-419` | FAIL-OPEN, low severity | Facts don't flow forward |
| QW4 TemplateArgument stub self-ref | `lower.rs:664-672` | ILLEGAL STATE | Convention not type-enforced |
| QW5 `lower_type_for_port` whitelist | `lower.rs:988-1014` | PARALLEL AUTHORITY | Coarse tag parallel to structural |

## Substrate cross-reference

The substrate currently exposes these fields (from `src/v3/compiler/src/dag.rs`):

| Field | Purpose | Consumers that read it |
|-------|---------|------------------------|
| `Port.state: PortState` | type or failure state | infer (read/write), depth/prov (indirect) |
| `Port.produced_by: Option<NodeId>` | upstream producer | depth, provenance, infer |
| `Behavior` (5 variants) | computation kind | depth, provenance, infer |
| `TransformNode.target: DeclarationId` | what is being called | infer |
| `ValueNode.data: LiteralBits` | which literal kind | infer (Q1) |
| `Declaration.name: Option<String>` | human label, uniqueness key | infer (Q1/Q2/Q4/Q8 — bridges), parse (identifier lookup) |
| `Declaration.connective` | type-level shape | infer (Q3/Q5/Q6/Q7 — structural) |
| `Declaration.type_params` | generic slots | infer (Q7) |
| `Declaration.meta_tag` | "this is a meta-typed value" | infer (Q8) |
| `Declaration.inhabits` | "this declaration additionally inhabits algebra X" | unused today |
| `AtomPayload` (4 variants) | leaf content | infer (Q3 — overloaded) |
| `ArrowBody` (3 variants) | user-defined / external / pending | infer (Q6) |
| `TemplateArgument` | param → value binding | infer (Q7) |
| `TypeShape` (newtype `DeclarationId`) | port type identity | infer (Q1/Q2/Q5) |

### What the substrate does NOT expose (the gaps)

**Read-side gaps** (from Consumer 3 infer.rs enumeration):

1. **Primitive type identity** — there is no direct edge from a
   `LiteralBits` variant to its primitive `DeclarationId`, and no
   `Dag::int_shape()` / `bool_shape()` / `string_shape()` cached
   accessor. Q1, Q2, Q4 all reconstruct this via
   `primitive_shape(dag, name)` name scans.

2. **Operator dispatch classification** — there is no substrate
   edge from "an operator symbol" to "the algebra-field family it
   dispatches through" or "the output-type rule it follows." Q3 and
   Q4 cover this via string inspection + a hardcoded comparison
   set. `AtomPayload::UnresolvedIdentifier(String)` carries both
   meanings ("forward reference" and "operator") and downstream code
   must inspect the string content to tell them apart.

3. **Meta-type identity** — Q8 asks "is this a Realization?" by
   comparing `meta_tag`'s name to `"Realization"`. No structural
   "meta-type role" edge; the answer is a string compare on a name.

4. **`Declaration.inhabits` is declared but unread** — a substrate
   field with no current consumer. Either a stub for future use
   (fine, but document it as such) or a missed opportunity for
   Q8-style questions ("is this declaration a Realization?" could
   be a declared role rather than a name comparison).

**Write-side gaps** (from Consumer 4 lower.rs enumeration):

5. **Block-bodied fn items have no declaration scaffold.** The
   parser represents them as `SurfaceItem::Fn { body: None }`, but
   that shape has no corresponding declaration form (QW1). Needed:
   either a `Fn` variant that explicitly records "external body"
   and lowers to a declaration with `ArrowBody::Pending` (already
   the §8.11 scaffold variant designed for this case) or a
   parse-time diagnostic in user-code mode.

6. **`data` declarations have no declaration form at all.** They
   are parser-absorbed (QW2). Needed: either a full
   `SurfaceItem::Data` variant that lowers into a structural
   declaration (the shape is `name + ty + body`, which is a Conj
   over the body's field facts) or a scaffold declaration with a
   meta-tag pointing at the parsed-body span.

7. **`module` / `import` declarations have no surface
   representation** (QW3). Same shape as #6 but lower severity.

8. **`TemplateArgument` admits non-TypeParam parameters** (QW4).
   Needed: either a typestate that prevents the self-reference
   construction (`parameter: TypeParamId` where `TypeParamId` is a
   newtype only constructible from a TypeParam atom declaration),
   or deletion of the stub branch in `build_template_arguments`
   (emit nothing for stub templates; the stub error is already
   raised elsewhere).

9. **Port type annotations have a parallel authority to
   declaration-side type resolution** (QW5). Needed: dissolve
   `lower_type_for_port` into a call to `type_to_declaration_id`
   wrapped as `TypeShape`. The "which types exist" question then
   has one authority, the declaration table.

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
