# Lens-fold prerequisites — substrate audit before complexity-lens migration

**Status:** AUDIT (Director-approved option C on parent inbox #1130
2026-04-29). Authored at the stop-and-ping point of the
T-Substrate-Lens-Primitive complexity-lens migration slice. Names the
exact substrate/grammar work required before
`data complexity_lens: Lens<Int> = { ... }` (or any other
`Lens<C>` instance) can lower honestly, and before
`complexity_lens_via_framework_correct` becomes a meaningful test.

This audit does not author code. It enumerates the missing constructs,
points at the existing class-5 / Stage-2 markers that already track
parallel slices of the same problem, sketches the smallest honest
unblock surface, and identifies the workflow-root question Director
flagged for the M2 semantic interpretation.

The audit is grouped by what each `Lens<C>` field needs from the
lowering pipeline. Items already partially closed (e.g.
`ValueBody::Scalar`, literal-only `ValueBody::Structural`) are
marked so the unblock surface is the *delta*, not a re-author.

---

## Target shape — what we want to lower

Per `src/v3/std/lens.dag` (Director-locked 6-field carrier from
#1186) and `docs/design-lens-framework.md` §"Migration plan" row for
`src/v3/lenses/complexity.dag`:

```dag
data complexity_lens: Lens<Int> = {
  name: "complexity",
  read: complexity_read,
  sequential: { op: int_add, identity: 0 },
  branch: int_max,
  iterate: complexity_iterate,
  validate: complexity_validate
}
```

with supporting top-level `fn` declarations:

```dag
fn complexity_read(d: Dag, b: Behavior) -> Witness<Int> = { ... }
fn int_add(a: Int, b: Int) -> Int = a + b
fn int_max(a: Int, b: Int) -> Int = if a > b then a else b
fn complexity_iterate(c: Int, bound: LoopBound) -> Int = { ... }
fn complexity_validate(d: Dag, c: Int) -> OptionalDiagnostic = NoDiagnostic
```

The audit walks each construct against the current lowering pipeline.

---

## Field-by-field constructor audit

### 1. `name: "complexity"` — String literal field value

**Status:** ✅ **closed** at M1(3) / PR #496.
`SurfaceExpr::Record { fields }` with `LiteralBits::String` field
values lowers via the `ValueBody::Structural { fields:
Vec<(String, FieldValue)> }` path. PR-B receipt confirms (per
`src/v3/DOWNSTREAM_REQUIREMENTS.md` §"Class 5 gap 3 — M1(3) PR-B
partial close").

No prerequisite work needed for this field.

### 2. `read: complexity_read` — declaration-reference field value

**Status:** ⏸ **DEFERRED** — class-5 gap #3, port-carried field
values branch.

Per `src/v3/DOWNSTREAM_REQUIREMENTS.md` §"Class 5 gap 3" closing
paragraph:
> The remaining gap is **port-carried field values** (a record
> field whose value is another declaration reference, a nested
> record, a list literal, or a `Var` of an outer binding). When
> the next consumer needs one of those, `ValueBody::Structural`
> upgrades from `Vec<(String, LiteralBits)>` to
> `Vec<(String, PortId)>` (or similar), and the literal-bits
> inline form becomes a special case — not a separate variant.

Substrate already grew `FieldValue::Reference(DeclarationId)` per
the `FieldValue` enum at `src/v3/compiler/src/dag.rs:443`. Lens
instances and the parallel `Dimension<SymbolicCost>` deferral at
`src/v3/lenses/cost.dag:268-298` are exactly "the next consumer
that needs one of those" — the upgrade is named but not authored.

**Required unblock:**
- Lower `SurfaceExpr::Var` (and `SurfaceExpr::Path` for
  module-qualified refs) inside record-literal field positions to
  `FieldValue::Reference(DeclarationId)` resolved via
  `lower_record_to_structural`.
- Inhabitance check: `FieldValue::Reference(id)` must inhabit the
  declared field type (resolves to a declaration whose
  `connective` matches the field's annotated type).
- Same path covers field references for `sequential.op`,
  `branch`, `iterate`, `validate`.

This is the single largest unblock; closing it dissolves both the
deferred `Dimension<SymbolicCost>` instance (cost.dag) and any
`Lens<C>` instance simultaneously.

### 3. `sequential: { op: int_add, identity: 0 }` — nested record literal

**Status:** ⏸ **DEFERRED** — class-5 gap #3, nested record branch.

Same closing paragraph in DOWNSTREAM_REQUIREMENTS:
> a record field whose value is **another nested record** [...]

Field shape is `Monoid<Int>` (a Conj from
`dsl/std/algebra.dag:110`) — must lower as a nested
`FieldValue::Record(Vec<(String, FieldValue)>)`.

**Required unblock:** same as #2 (port-carried field values),
plus:
- Recursive inhabitance: `FieldValue::Record(fields)` against a
  Conj-typed field must check each inner `(label, value)` pair
  against the corresponding inner field's type.

`FieldValue::Record` already exists at `dag.rs:446`. The lowerer
needs to emit it from nested record-literal surface forms.

### 4. `iterate: complexity_iterate` and `validate: complexity_validate` — same as #2

Same construct as `read`. Once #2 closes, all four declaration-
reference fields lower.

### 5. `complexity_read` body — `fn` block expression

**Status:** ⏸ **DEFERRED** — Stage-2 grammar gap, `fn`
block-body lowering.

Per `src/v3/lenses/cost.dag:284-288`:
> Record literals / match / lambdas inside `fn X { body }`
> block-bodied definitions. Constructing
> `Witness::Violates { reason, at }` — the
> MissingCost-to-violation shim the Dimension contract requires —
> hits the same block-body restriction.

`complexity_read` needs to:
- `match behavior { Value(_) => Inhabits(0), Transform(_) =>
   Inhabits(1), Branch(_) => Inhabits(1), Loop(_) =>
   Inhabits(1), Bind(_) => Inhabits(0) }` (or whatever per-Behavior
  weight the M2 semantic interpretation locks in — see §
  "Workflow-root semantic question" below).
- Construct `Witness::Inhabits(n)` and `Witness::Violates { ... }`
  variants, both nested-record / variant-constructor expressions.

**Required unblock:**
- Block-body fn lowering (`fn name(...) -> T = { match ... }` or
  `fn name(...) -> T { ... return ... }`) for record-literal /
  match / lambda inner expressions.
- Variant-constructor expression lowering — class-5 gap #4 (per
  `src/v3/DOWNSTREAM_REQUIREMENTS.md:204-237`). `Witness::Inhabits`
  and `Witness::Violates` are bare-variant-name constructions.
  `Inhabits` may dispatch through scrutinee-typing in a `match`
  arm, but the RHS construction `Inhabits(0)` is the same problem
  class as the `False` / `True` RHS case the gap-4 entry names.

`complexity_iterate` and `complexity_validate` have similar bodies;
same block-body + variant-constructor needs.

### 6. `int_add`, `int_max` — pure expression-body fns

**Status:** ✅ **closed**. Trivial `Int` → `Int` arithmetic /
comparison expression-body fns lower today. No prerequisite work.

---

## `fold_lens<C>` — generic fold machinery (I2)

`src/v3/std/lens.dag:6` documents `fold_lens<C>: Lens<C> → Dag →
DimensionReport<C>` as the framework's consumer entry point. **Not
authored yet.** Without it, even an honestly-lowered
`complexity_lens: Lens<Int>` value has no consumer to validate
equivalence against the existing per-port depth fold.

### What `fold_lens<C>` needs to express

```dag
fn fold_lens<C>(lens: Lens<C>, d: Dag) -> DimensionReport<C> {
  // 1. For each Behavior b in d.nodes:
  //      witness = lens.read(d, b)
  //      collect into List<Witness<C>>
  // 2. Fold the witnesses via lens.sequential / lens.branch /
  //    lens.iterate per the program structure (Bind sequence /
  //    BranchNode arms / LoopNode iteration).
  // 3. If any read returned Violates, short-circuit to
  //    DimensionFail with violations: List<Diagnostic>.
  // 4. Else call lens.validate(d, composed) for the aggregate
  //    side-condition; lift OptionalDiagnostic into
  //    DimensionFail.violations or pass through to DimensionOk.
  // 5. Return DimensionOk { dimension_name: lens.name, composed,
  //    witnesses } / DimensionFail { ... } per the dimensions.dag
  //    carrier shape.
}
```

### Lowering needs for `fold_lens<C>`

- **Higher-order function calls:** `lens.read(d, b)` is a
  field-call on a function-typed Conj field. v3 already lowers
  HO call expressions for `AnalysisDimension.witness_of`-style
  consumers in Rust, but `fold_lens<C>` needs the same pattern
  to lower from `.dag`. Verify parity.
- **Generic type parameter dispatch:** `fold_lens<C>` is
  parametric over the carrier. `Lens<C>` and
  `DimensionReport<C>` already exist; the fn needs proper
  type-param threading through the bootstrap inhabitance
  check.
- **Block body / match / variant-construction:** same as
  §"complexity_read body" above.

### Sequencing

`fold_lens<C>` is a separate authoring lane *after* the field-
value lowering unblocks (§§1-5 above). Without those, you can't
even author lens instances to fold over.

---

## Workflow-root semantic question (Director M2 note)

Director's stop-and-ping reply 2026-04-29:
> Semantic interpretation for later M2: preserve current behavior
> means the framework-composed whole-Dag `Int` should match the
> existing per-port depth at the workflow/root result port, not
> preserve the whole per-port lookup table as the carrier. If
> that interpretation exposes a mismatch in available
> root-result identification, include that in the prerequisite
> audit rather than guessing.

### Workflow root in current substrate

`Dag` (`src/v3/std/substrate.dag:410`) is `{ declarations,
nodes: List<Behavior>, ports, clusters }`. There is no explicit
"workflow root" field; the substrate invariant is that
`d.nodes` is topologically ordered (producers before consumers
— the same invariant the existing complexity fold relies on).

`Behavior::Bind { result_port, params, span, lane2_workflow,
emit_participation }` (line 393-401) carries
`lane2_workflow: WorkflowEffect?` — only some Binds are
"workflow" Binds. The "workflow root" the M2 interpretation
references is presumably the **last `Bind` in
topological order whose `lane2_workflow` is `Some`**, or the
**outermost user-facing `Bind`** (the last one with
`emit_participation: Some(BindEmitParticipation::UserCallable)`).

### Identification options for `fold_lens<C>` semantics

**(α) Last topological `Bind`.** Take
`d.nodes.last()` if it's a `Bind`; its `result_port` is
the workflow-root port. Existing fold's `cost_of(d, last_bind.
result_port)` becomes the equivalence target.

**(β) Last `lane2_workflow: Some(_)` Bind.** Filter `d.nodes`
for Binds with `lane2_workflow` populated; take the last.
More semantically aligned with the workflow-effect concept,
but `lane2_workflow` is currently `None` for most Binds in
typical fixtures (it's specific to lane-2 workflow effects).

**(γ) Last `emit_participation: Some(UserCallable)` Bind.** The
top-level user-facing function's result port. Closest to "the
program's output."

### Director disposition (2026-04-29, parent inbox #1130)

**Workflow-root = (α) last topological `Bind`, but only behind
a named `workflow_root_port(d: Dag) -> WorkflowRoot` helper/accessor.**
The accessor is the framework boundary; the implementation is
"last topological `Bind`" today, refinable to (γ)
last-`UserCallable`-Bind later behind the same accessor without
churning consumers. (β) rejected as too narrow — a
`complexity_lens` should work on any program, not only lane-2
workflow programs.

**Cross-reference (Director-requested):** `workflow_root_port`
is shared authority for **both** `fold_lens<C>`'s workflow-root
identification AND R2-Evaluator's runtime entry-point
identification (`evaluate(program, entry, args) -> Value` shape
from Items 4+5 / #1176 §3.2). Same root-identification problem,
one accessor surface. The accessor is therefore not a
lens-framework-private addition; it's a substrate-level
declared fact that the lens framework and the runtime evaluator
both consume.

### Prereq-3 carrier: `workflow_root_port` accessor

**Return-type fail-closed shape (per BLOCKING review on
PR #1207 at sha `d34ca4a1`):** the accessor must NOT return a
bare `PortId`. A total `Dag → PortId` return drops the
no-root and multi-entry cases — a Dag with zero `Bind` nodes
or multiple eligible roots would require fabricating a
`PortId` or asserting a positional convention, which violates
fail-closed substrate semantics. The accessor returns a
typed `WorkflowRoot` carrier:

```dag
type WorkflowRoot
  = SingleRoot(PortId)                               // exactly one eligible root; PortId is its result_port
  | NoRoot                                           // zero Binds in d.nodes; lens fold short-circuits to DimensionFail
  | AmbiguousRoot { candidates: List<PortId> }       // 2+ eligible roots under the active α/γ rule; consumer must disambiguate (R2-Evaluator picks via entry-arg; fold_lens reports DimensionFail)
```

Both consumers (`fold_lens<C>` and R2-Evaluator) handle the
three variants explicitly. `NoRoot` and `AmbiguousRoot` are
fail-closed surfaces, not fabrication points. The α
implementation populates `SingleRoot` only when exactly one
last-topological-`Bind` exists; γ refinement (last
`UserCallable` Bind) reuses the same `WorkflowRoot` partition.

The accessor lands as part of Prereq-3 (or as a separate
sub-slice of Prereq-3 that can lead, since the accessor doesn't
depend on `fold_lens<C>` machinery — only on substrate-level
`Dag.nodes` walk). Either:

- **Sub-slice 3a:** declare `workflow_root_port(d: Dag) ->
  PortId` in `src/v3/std/substrate.dag` (or sibling) with the
  α implementation. R2-Evaluator and `fold_lens<C>` both
  consume.
- **Sub-slice 3b:** author `fold_lens<C>` against the accessor
  from 3a.

3a can land before Prereq-2 closes (no fn-block-body needs).
3b is gated on Prereq-1 + Prereq-2 as before.

---

## Smallest honest unblock surface

The audit produces one **substrate prerequisite lane** with three
sub-slices. Each is a separate PR; each closes when its named
test lands.

### Prereq-1: port-carried field values in `data` record bodies

**Scope:** `lower_record_to_structural` accepts
`SurfaceExpr::Var` / `SurfaceExpr::Path` (declaration references)
and nested `SurfaceExpr::Record` / `SurfaceExpr::List` /
`SurfaceExpr::Map` as field values. Inhabitance check upgrades to
recursive structural typing.

**Acceptance:** a fixture
`data foo: SomeConj = { fn_field: some_fn, nested: { ... } }`
lowers to `ValueBody::Structural { fields: Vec<(String,
FieldValue)> }` with `FieldValue::Reference(_)` /
`FieldValue::Record(_)` populated, and the bootstrap snapshot
preserves it byte-stably across regen.

**Closes:** §§2, 3, 4 above. Unblocks `data complexity_lens:
Lens<Int> = { ... }` field assignments that reference top-level
`fn` declarations.

**Sizing estimate:** ~3-5 days. The substrate variants
(`FieldValue::Reference`, `FieldValue::Record`,
`FieldValue::List`, `FieldValue::Map`) already exist; the
authoring is in the lowerer's record-literal walker and the
inhabitance check.

### Prereq-2: `fn` block-body lowering with variant-constructor expressions

**Scope:** `fn name(...) -> T = { match ... }` and
`fn name(...) -> T { let ... ; return ... }` lower record /
match / variant-constructor expressions inside the body.
Class-5 gap #4 (variant-name resolution outside match scrutinee
context) closes here.

**Acceptance:** a fixture
`fn complexity_read(d: Dag, b: Behavior) -> Witness<Int> = {
match b { Value(_) => Inhabits(0), Transform(_) => Inhabits(1),
... } }` parses, lowers, and bootstraps without falling back to
`FnExternalBody`.

**Closes:** §5 above. Unblocks the bodies for `complexity_read`,
`complexity_iterate`, `complexity_validate`.

**Sizing estimate:** ~5-7 days. Larger because variant-
constructor resolution (class-5 gap #4 — three options listed
in the gap entry) is itself non-trivial.

**Director disposition (2026-04-29):** **infer-time
re-resolution** for variant constructors, per class-5 gap #4's
"biggest substrate lift but most general" framing. **No
per-type special cases for `Witness<C>` / `OptionalDiagnostic`
or any other named coproduct.** The variant-resolution path is
generic; lens-framework variant constructions inherit the
generic path without bespoke handling.

### Prereq-3: `fold_lens<C>` machinery + `workflow_root_port` accessor

**Scope (per Director disposition 2026-04-29):**
- **3a:** declare `workflow_root_port(d: Dag) -> WorkflowRoot` in
  `src/v3/std/substrate.dag` (or sibling) with the α
  implementation (last topological `Bind`). Shared authority —
  R2-Evaluator's runtime entry-point identification consumes
  the same accessor (per Items 4+5 / #1176 §3.2 cross-reference).
- **3b:** author `fold_lens<C>: Lens<C> → Dag →
  DimensionReport<C>` in `src/v3/std/lens.dag` (or sibling),
  consuming `workflow_root_port` from 3a for the
  per-port-equivalence test target.

**Acceptance:** `fold_lens<C>` lowers to bootstrap; a fixture
`Lens<Int>` instance + `fold_lens` call returns a
`DimensionReport<Int>` whose `composed` matches the existing
per-port depth at the workflow-root port.

**Closes:** the gap between `Lens<C>` declaration (already in
substrate from #1186) and `Lens<C>` consumption. Unblocks
`complexity_lens_via_framework_correct` as a meaningful
equivalence test.

**Sizing estimate:**
- 3a: ~1-2 days. Standalone — does not depend on Prereq-1 or
  Prereq-2; substrate-level `Dag.nodes` walk only. Can land
  in parallel with Prereq-1 / Prereq-2 to unblock R2-Evaluator
  early.
- 3b: ~3-5 days *after* Prereq-1 + Prereq-2 land. Cannot be
  authored before them — without lowered fn-body lens-field
  values, there's nothing for `fold_lens<C>` to fold over.

### Total

~11-17 days at gunbc velocity, partially parallelizable per the
Director-locked disposition above:
- Prereq-1 (~3-5 days) and Prereq-3a (~1-2 days) are
  independent — both can start immediately. Prereq-3a unblocks
  R2-Evaluator's runtime entry-point identification ahead of
  the lens migration.
- Prereq-2 (~5-7 days) depends on class-5 gap #4 closure
  (infer-time re-resolution per Director disposition).
- Prereq-3b (~3-5 days) depends on Prereq-1 + Prereq-2 + Prereq-3a.

Roughly the same total as the original "~5-8 days for 4-lens
migration" sized in `docs/design-lens-framework.md`'s migration
plan, plus the substrate lifts that the framework migration
implicitly assumed but were never separately scheduled.

---

## What this audit explicitly does NOT do

- Does not author `data complexity_lens: Lens<Int> = { ... }` —
  blocked on Prereq-1.
- Does not author per-field free functions (`complexity_read`,
  etc.) as a substitute — Director rejected this in option (D).
- Does not author `fn complexity_lens() -> Lens<Int>` callable —
  Director rejected this in option (B).
- Does not modify the existing per-port complexity fold —
  preserved as the equivalence oracle.
- Does not commit to an interpretation of "workflow root"
  beyond raising the question and naming three options.

---

## Cross-references

- `src/v3/std/lens.dag` — Lens<C> 6-field carrier (#1186 receipt).
- `src/v3/std/lens.dag:6` — `fold_lens<C>` documented but unauthored.
- `src/v3/lenses/complexity.dag` — current per-port-depth PROXY fold;
  preserved as M2 equivalence oracle.
- `src/v3/lenses/cost.dag:260-302` — parallel `Dimension<SymbolicCost>`
  data-binding deferral; same blocker, same dissolution trigger.
- `src/v3/DOWNSTREAM_REQUIREMENTS.md` §"Class 5 gap 3" — port-carried
  field values branch (Prereq-1).
- `src/v3/DOWNSTREAM_REQUIREMENTS.md` §"Class 5 gap 4" — variant-name
  resolution outside match scrutinee (Prereq-2).
- `docs/design-lens-framework.md` §"Migration plan" — original
  migration sizing assumed these prereqs were closed; this audit
  surfaces the implicit dependency.
- `docs/design-dimension-abstraction.md` §"User-declared dimension
  example" — same blocker tracked there ("blocked in part on
  class-5 `data` bodies for `Dimension` value declarations").

---

## Acceptance for this audit PR

This document is the deliverable. No code changes; no test additions;
no substrate edits. The next dispatch consumes this audit to scope
Prereq-1 / Prereq-2 / Prereq-3 as separate substrate lanes. If
Director chooses a different unblock sequencing or a different
workflow-root identification, this doc is the diff target.
