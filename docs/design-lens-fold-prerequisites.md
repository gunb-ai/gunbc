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

### 2. `read: complexity_read` — declaration-reference field value (Arrow-typed)

**Status:** ⏸ **PARTIALLY OPEN** — narrower than the original
class-5 gap #3 framing. **Per BLOCKING review on PR #1207
sha `d34ca4a1`:** `lower_record_to_structural` at
`src/v3/compiler/src/lower.rs:3273` already lowers nested
`Record` / `List` / `Map` field values; `lower_structural_field_value`
at `:3369` already resolves `SurfaceExpr::Var` and
`SurfaceExpr::Path` to `FieldValue::Reference(DeclarationId)`
(line 3399 / 3409) when the field's `expected_type` walks to a
`DeclarationRef` marker or the resolved decl's `meta_tag` matches.

The actual residual gap is **inhabitance checking for
function-typed (Arrow) fields**: `Lens<C>.read: fn(Dag, Behavior)
-> Witness<C>` is an Arrow type, not a `DeclarationRef` and not
matched via `meta_tag`. The current
`lower_structural_field_value` paths cover decl-ref and meta-tag
shapes; assigning a `fn complexity_read(d: Dag, b: Behavior) ->
Witness<Int>` decl to a field of declared Arrow type goes through
`declaration_ref_types_equivalent`, which checks marker-type
equivalence, not Arrow-signature equivalence.

**Required unblock:**
- Arrow-signature inhabitance: when an `expected_type` resolves
  to an `Arrow { inputs, output }` type, allow
  `FieldValue::Reference(decl_id)` if the resolved decl is itself
  an Arrow (a `fn`) whose input/output declarations match
  structurally.
- Verify that `FieldValue::Record` / `FieldValue::List` /
  `FieldValue::Map` paths (already lowered) interact correctly
  with Arrow-typed fields nested inside `Lens<C>` (e.g.,
  `sequential: Monoid<C>` is a Conj whose inner `op` field is
  Arrow-typed; the recursive lowering needs Arrow-inhabitance at
  the inner step).

Closing this dissolves both the deferred
`Dimension<SymbolicCost>` instance (cost.dag) and any `Lens<C>`
instance simultaneously, but the change set is smaller than the
original "port-carried field values" framing — the substrate-
level lowering paths already exist; only Arrow-inhabitance is
missing.

### 3. `sequential: { op: int_add, identity: 0 }` — nested record literal

**Status:** ✅ **closed** for the record-literal lowering itself.
`FieldValue::Record` at `dag.rs:446` and the recursive
`lower_structural_field_value` walk at `lower.rs:3543-3562`
already emit nested record values with recursive inhabitance
against Conj-typed fields.

The remaining gap is the same Arrow-inhabitance issue from §2 —
`Monoid<C>.op: fn(C, C) -> C` is an Arrow-typed field inside
the nested record. Once §2's Arrow-inhabitance lands, the nested
`Monoid<Int>` literal `{ op: int_add, identity: 0 }` lowers
end-to-end (the `int_add` resolution against the `op` field's
Arrow type, plus the trivial `0` literal against `identity: Int`).

### 4. `iterate: complexity_iterate` and `validate: complexity_validate` — same as §2

Same Arrow-inhabitance gap as `read`. Once §2 closes, all four
Arrow-typed `Lens<C>` fields lower.

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

### Prereq-3b implementation plan (2026-04-30)

**Status:** PLAN ONLY until both implementation gates clear:

- **Gate 1:** PR #1232 (`workflow_root_port`) merges. `fold_lens<C>`
  consumes that accessor as the only workflow-root authority; it must not
  re-derive root selection from `Dag.nodes` directly.
- **Gate 2:** PR #1248 (brace-bodied `fn` / variant-constructor lowering)
  merges, or explicitly narrows to a shape that still lets real
  `Lens<C>` helpers author `Witness<C>` / `OptionalDiagnostic` /
  `DimensionReport<C>` constructors in `.dag`.

**Target signature and carrier authority:**

```dag
fn fold_lens<C>(lens: Lens<C>, d: Dag) -> DimensionReport<C> = ...
```

The result carrier is exactly `DimensionReport<C>` from
`src/v3/std/dimensions.dag`; `fold_lens<C>` does not introduce a
parallel `LensReport` or a lens-local failure carrier. `DimensionOk`
contains `dimension_name: lens.name`, the root-composed `C`, and the full
`List<Witness<C>>`. `DimensionFail` contains `dimension_name: lens.name`,
`violations: List<Diagnostic>`, and the same witness list; it never
fabricates a `composed` value when a root or witness failure occurs.

**Files to touch once the gates clear:**

- `src/v3/std/lens.dag` — author `fold_lens<C>` plus any small private
  helpers needed to fold `Behavior` and convert `WorkflowRoot` /
  `Witness<C>` / `OptionalDiagnostic` failures into `DimensionReport<C>`.
- `src/v3/std/substrate.dag` — import/use only the
  `workflow_root_port(d: Dag) -> WorkflowRoot` accessor from #1232. 3b
  should not add another root carrier; if #1232 does not expose
  `NoRoot` / `AmbiguousRoot` in the shape above, STOP+PING instead of
  inventing a second partition.
- `src/v3/std/dimensions.dag` — no shape changes expected; only import
  `DimensionReport`, `Witness`, and `OptionalDiagnostic`.
- `src/v3/std/diagnostics.dag` — only if #1232 does not already provide
  reusable diagnostics for `NoRoot` / `AmbiguousRoot`. Prefer reusing
  existing `Diagnostic` constructors; STOP+PING before adding a new
  diagnostic substrate carrier.
- `src/v3/lenses/complexity.dag` — first real instance/equivalence
  consumer after 3b lands; keep current `cost_of` / `compute_costs` fold
  as the migration oracle until `complexity_lens_via_framework_correct`
  passes.
- `src/v3/lenses/cost.dag` — second consumer; map its current
  `SymbolicCost` combinators and `Lookup<SymbolicCost>` failure behavior
  onto a `Lens<SymbolicCost>` instance after the framework fold exists.
- `src/v3/lenses/idempotency.dag` and `src/v3/lenses/parallelism.dag` —
  readiness probes only in the first 3b PR unless their prerequisite
  carriers are already live. Do not pretend the hard stub in
  `parallelism.dag` is a completed lens instance.
- `src/v3/compiler/src/bootstrap_std_generated.rs` /
  `src/v3/compiler/src/bootstrap_generated.rs` — regenerated only through
  the existing bootstrap flow if `.dag` edits require it.

**Workflow-root consumption and fail-closed behavior:**

1. `fold_lens<C>` calls `workflow_root_port(d)` before it attempts to
   report a root-composed value.
2. `SingleRoot(root_port)` selects the report target. The fold composes
   per-Behavior witnesses across the DAG structure and reports the value
   associated with `root_port` as `DimensionOk.composed`.
3. `NoRoot` returns `DimensionFail { dimension_name: lens.name,
   violations: [Diagnostic(... no workflow root ...)], witnesses }`.
   The fold must not use `lens.sequential.identity` as a fabricated whole
   program result for an empty / rootless DAG.
4. `AmbiguousRoot { candidates }` returns `DimensionFail` with a
   diagnostic naming the candidate root ports. The fold must not choose
   first, last, or "best" candidate; entry disambiguation belongs to the
   evaluator consumer, not to the generic lens fold.

**Generic fold mechanics:**

- `read`: call `lens.read(d, behavior)` for each `Behavior`, preserving
  every `Witness<C>` in order for the report.
- `sequential`: compose `Bind` / topological sequence values with
  `lens.sequential.op`, using `lens.sequential.identity` only as the fold
  unit for legitimate empty substructures, never as a failure fallback.
- `branch`: compose exclusive `BranchNode` arms with `lens.branch`.
- `iterate`: compose `LoopNode` bodies with `lens.iterate(body_c,
  loop.bound)`.
- `validate`: after a successful root composition, call
  `lens.validate(d, composed)`. `SomeDiagnostic` converts to
  `DimensionFail`; `NoDiagnostic` produces `DimensionOk`.
- Failure partition: any `Witness::Violates` or root/validation
  diagnostic produces `DimensionFail`. The first implementation may
  either accumulate all diagnostics or preserve a deterministic prefix,
  but it must not drop witness evidence or synthesize a carrier.

**Substrate vs lens-instance-specific boundary:**

- Substrate/framework (`src/v3/std/lens.dag`) owns only traversal,
  structural composition dispatch, root handling, witness retention, and
  final `DimensionReport<C>` partitioning.
- Lens instances own `read`, `sequential`, `branch`, `iterate`, and
  `validate`. Cost/complexity/idempotency/parallelism must not add
  special cases to `fold_lens<C>`.
- `workflow_root_port` remains shared substrate authority between the
  lens framework and R2-Evaluator. `fold_lens<C>` is a consumer, not a
  second root-selection authority.

**Minimal first implementation tests:**

- `lens_fold_no_root_fails_closed` — crate-private / synthetic-DAG
  coverage for a DAG with no eligible root returns `DimensionFail`, not
  `DimensionOk` with a monoid identity. Do not require an ordinary
  lowered-source fixture for this case if the source pipeline always
  produces at least one `Bind`.
- `lens_fold_ambiguous_root_fails_closed` — future enumerate-all-entry
  coverage, or crate-private helper/unit coverage over an explicit
  `AmbiguousRoot { candidates }`, returns `DimensionFail` with candidate
  evidence and no chosen `composed`. Under the accepted α
  last-topological-`Bind` rule in #1232, `AmbiguousRoot` is a drift arm,
  not a normal source-fixture result.
- `complexity_lens_via_framework_correct` — a `Lens<Int>` fixture reports
  the same root value as `src/v3/lenses/complexity.dag::cost_of(d,
  root_port)`. This is the first full fold correctness test.
- `cost_lens_framework_symbolic_root_matches_current_lookup` — a
  `Lens<SymbolicCost>` fixture reports the same root `SymbolicCost` as
  `src/v3/lenses/cost.dag::symbolic_cost_of(d, root_port)` on one
  sequential transform and one branch/loop case. Keep existing
  `Lookup<SymbolicCost>` Miss behavior as the oracle for read failures.
- `idempotency_lens_framework_readiness` — compile-only or
  structural-shape test that the current idempotency analyzer can expose
  a `Lens<WorkflowIdempotencyReport>`-shaped plan once its carrier
  becomes a monoid; do not migrate behavior in 3b unless the effect
  report algebra is already declared.
- `parallelism_lens_framework_readiness` — assert the current
  `parallelism.dag` hard stub is not migrated as a fake completed
  `Lens<C>` instance. The first real migration waits for the Stage 2e
  `.dag` walk / `lane2_workflow_at` port.
- Bootstrap/snapshot checks: run the focused new integration tests plus
  `cargo run -p v3-compiler --features bootstrap-regen-fresh --bin
  regen_bootstrap -- --verify`; run `regen_lens` snapshots only for lens
  files edited in that PR.

**STOP+PING criteria before implementation:**

- `workflow_root_port` lands without a typed `WorkflowRoot` partition for
  `SingleRoot` / `NoRoot` / `AmbiguousRoot`.
- `fold_lens<C>` needs a new stored DAG-side accumulator, root marker, or
  report carrier rather than consuming existing `Lens<C>`,
  `WorkflowRoot`, `Witness<C>`, `OptionalDiagnostic`, and
  `DimensionReport<C>`.
- #1248 leaves `.dag` unable to author the constructors needed for
  `Witness<C>`, `OptionalDiagnostic`, or `DimensionReport<C>` in real
  `read` / `validate` helpers.
- Type-param lowering cannot express `fn fold_lens<C>(...) ->
  DimensionReport<C>` without hand-Rust scaffolding.

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
  | AmbiguousRoot { candidates: List<PortId> }       // multi-entry programs (R2-Evaluator entry-name disambiguation case); reserved for the enumerate-all-eligible-entries rule, NOT producible under α/γ "last X Bind" rules over linear d.nodes
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
  WorkflowRoot` in `src/v3/std/substrate.dag` (or sibling) with
  the α implementation. R2-Evaluator and `fold_lens<C>` both
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

### Prereq-1: Arrow-signature inhabitance for fn-typed `data` record fields

**Scope:** narrower than the original framing (per BLOCKING
review). The substrate-level field-value variants
(`FieldValue::Reference`, `FieldValue::Record`,
`FieldValue::List`, `FieldValue::Map`) already exist;
`lower_record_to_structural` and `lower_structural_field_value`
already lower nested record/list/map literals AND resolve
`SurfaceExpr::Var` / `SurfaceExpr::Path` to
`FieldValue::Reference(DeclarationId)` for `DeclarationRef`-typed
and meta-tag-matching fields. The residual gap is
**Arrow-signature inhabitance** for fields whose `expected_type`
resolves to an Arrow (`fn(...) -> ...`):

- Extend `lower_structural_field_value`'s
  `declaration_ref_types_equivalent` path (or add a sibling
  `arrow_inhabitance` check) so that when `expected_type` is an
  Arrow `{ inputs, output }` and `expr` resolves to a top-level
  `fn` declaration, the resolved fn's input/output declarations
  are structurally compared and accepted as
  `FieldValue::Reference(fn_decl_id)`.
- Cover the recursive case: `Monoid<C>.op: fn(C, C) -> C` is
  Arrow-typed nested inside a `FieldValue::Record` lowering;
  the existing recursive walk reaches it but bails on
  Arrow-inhabitance before §2's gap closes.

**Acceptance:** a fixture
`data foo_lens: Lens<Int> = { name: "foo", read: foo_read,
sequential: { op: int_add, identity: 0 }, branch: int_max,
iterate: foo_iterate, validate: foo_validate }` lowers to
`ValueBody::Structural { fields }` where every Arrow-typed field
is a `FieldValue::Reference(fn_decl_id)` resolved against the
corresponding top-level `fn` declaration's signature. Bootstrap
snapshot preserves byte-stably across regen.

**Closes:** §§2, 3, 4 above. Unblocks `data complexity_lens:
Lens<Int> = { ... }` field assignments — except for the bodies
of `complexity_read` / `complexity_iterate` / `complexity_validate`
themselves, which §5 / Prereq-2 covers.

**Sizing estimate:** ~1-3 days. Smaller than the original
framing because most of the lowering pipeline is already in
place; the change is a narrower inhabitance refinement.

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

**3a acceptance (per BLOCKING review on PR #1207
sha `42bf818a4`):** since 3a may land before 3b — and per the
reflected-facts invariant every substrate addition needs a
generated-consumer proof — 3a carries its own acceptance
without piggy-backing on 3b's `complexity_lens_via_framework_correct`:

- `workflow_root_single_bind_returns_single_root` — a fixture
  Dag with exactly one `Bind` returns `SingleRoot(p)` where `p`
  is that Bind's `result_port`.
- `workflow_root_zero_bind_returns_no_root` — a fixture Dag
  containing only `Value` / `Transform` / `Branch` / `Loop`
  behaviors (no `Bind`) returns `NoRoot`.
- `workflow_root_multi_bind_returns_ambiguous` — neither α
  (last topological `Bind`) nor γ (last `UserCallable` `Bind`)
  can produce ambiguity over a linear `Dag.nodes`: both rules
  pick exactly one "last" element by definition. The
  `AmbiguousRoot` arm is reserved for a separate
  **enumerate-all-eligible-entries** rule, which the
  R2-Evaluator runtime consumer needs (per the
  `evaluate(program, entry, args)` shape from Items 4+5 / #1176
  §3.2 — the runtime takes an entry-name arg precisely because
  multi-entry programs need user-supplied disambiguation). That
  rule returns the full list of UserCallable `Bind` result_ports
  as `AmbiguousRoot { candidates }`; R2-Evaluator picks the one
  whose owning bind name matches `entry`.

  α / γ acceptance for this claim is therefore vacuous-but-wired:
  any fixture exercises α / γ to `SingleRoot(last_*_bind.
  result_port)`. The realizable `AmbiguousRoot` case lands when
  the enumerate-all rule is wired (a separate consumer path
  alongside α / γ behind the same accessor); its acceptance
  pins (a) the `candidates` list contains every UserCallable
  Bind's result_port; (b) R2-Evaluator's entry-name match
  reduces a multi-candidate result to a single chosen Port;
  (c) absence of `entry` against `AmbiguousRoot` fails closed
  with a typed diagnostic.
- `workflow_root_consumed_by_runtime_entry_point` (R2-Evaluator
  cross-consumer proof) — a smoke test driving R2-Evaluator's
  `evaluate(program, entry, args)` shape (or its substrate
  equivalent) through the same accessor; on `SingleRoot`,
  evaluation proceeds; on `NoRoot` / `AmbiguousRoot` without an
  entry-arg disambiguator, evaluation fails closed with a typed
  diagnostic.

These four claims pin the accessor's behavior and prove the
shared-authority cross-reference Director requested at the
substrate-load boundary, not at the consumer-folding boundary.
3b's downstream `complexity_lens_via_framework_correct` is then
purely a `fold_lens<C>` correctness test, not a re-test of the
accessor partition.

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

~9-15 days at gunbc velocity (revised down ~2 days after the
PR #1207 BLOCKING review correctly narrowed Prereq-1 from
"port-carried field values" to "Arrow-signature inhabitance" —
the substrate already has the broader lowering paths). Partially
parallelizable per the Director-locked disposition above:
- Prereq-1 (~1-3 days) and Prereq-3a (~1-2 days) are
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
- ~~Does not commit to an interpretation of "workflow root"
  beyond raising the question and naming three options.~~
  **Updated 2026-04-29:** Director-locked α (last topological
  `Bind`) behind a `workflow_root_port(d: Dag) -> WorkflowRoot`
  accessor — see §"Director disposition" above. This bullet
  is preserved as a strikethrough so the audit history shows
  the question-raising stance the doc shipped with before
  Director answered.

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
