# Prereq-X1.b — Evaluator Impact Audit

Docs-only audit of evaluator-side impact of the Prereq-X1.b
runtime-callee dispatch slice described in
`docs/design-prereq-x-ho-field-call.md` §Prereq-X1 / L1.b. Authored
by R3 Evaluator (sunny-hawk-622) at the request of R3 Evaluator
parent snappy-moth-795. Companion to the implementation-side static
slice already landed as PR #1699 (E6-G0b / X1.a).

The audit answers: from the evaluator's seat, what does the
`TransformDispatch` / `CalleeRef` / `ArrowPortRef` /
`input_ports()` substrate collapse cost, what stays Substrate
authority, and how should the implementation slice be split so that
it does not race E6-G0c (`fierce-bear`'s evaluator
`FieldProject` / `Callable` work).

The audit deliberately stops short of authoring an implementation
brief. The substrate carrier change is cross-cutting; per the
adjacent-authority rule (`feedback_audit_adjacent_authority_first`),
the substrate slice belongs to Substrate/Director, not Evaluator.

---

## 1. Evaluator consumers that read `TransformTarget` / `TransformNode.inputs` directly

Two classes of evaluator surface read these fields today; this
audit only catalogs evaluator-class consumers (not infer / lower /
emit / generated lens code, which are out of scope for this brief).

### Class E1 — host-Rust evaluator (`src/v3/compiler/src/lib.rs`)
`eval_transform_node` at `lib.rs:471-563`:
- iterates `t.inputs` to evaluate operands at `lib.rs:477-480`.
- matches `&t.target` against `Operator(Arithmetic | Comparison |
  Logical)`, `FieldProject`, `Callable`. Only `Operator(Arithmetic
  ring ops)` and `Operator(Comparison)` are implemented. `Logical`,
  `FieldProject`, `Callable`, and `ArithmeticDiv` all fail closed
  via `EvalError::UnsupportedTransformTarget { kind }` —
  `lib.rs:553-561`. The fail-closed surface predates X1; it is the
  contract that E6-G0c is currently extending.

### Class E2 — lens-apply evaluator (`src/v3/compiler/src/lens_apply.rs`)
- `eligibility_walk_transform` at `lens_apply.rs:71-89`: walks
  `t.inputs` and matches `t.target` (`Logical` ineligible; `Operator(_)`
  / `FieldProject` eligible; `Callable` recurses through
  `eligibility_walk_callable`).
- `eval_transform` at `lens_apply.rs:525-612`: dispatches on
  `t.target` for all four variant families. `Callable` arms do
  std-fold detection (`is_fold_instantiation`,
  `find_fold_step_bind_via_instantiation`) and then recurse into
  `eval_callable`; `Operator(Arithmetic|Comparison)` arms check
  arity directly via `t.inputs.len()`; `FieldProject` arm reads
  `field_label` and projects from a single-port input.
  `Logical` fails closed.
- `reflect_transform_target` at `lens_apply.rs:1996-2030`:
  reflects a `&TransformTarget` into a substrate `FieldValue` for
  lens authoring. Uses `sum_variant_payload(... "TransformTarget"
  ...)` per variant. This is the substrate-reflection surface; it
  reads the variant identity but not `t.inputs`.

### Class E3 — adjacent evaluators that walk `inputs` only
- `dimension.rs:expand_behavior_backward` at `dimension.rs:108-126`
  pushes `transform.inputs` onto a frontier; never matches on
  `target`. This is dependency walking, not dispatch.

Out-of-scope for this audit (substrate / lowerer / emitter / infer
/ generated-lens consumers): `dag.rs`, `dag/builder.rs`,
`infer.rs`, `lower.rs`, `emit.rs`, `emit/rust_target.rs`,
`emit/python_target.rs`, `regen_bootstrap_emit.rs`,
`lens_effect_enumeration_generated.rs`, and the other
`lens_*_generated.rs` files. Those consumers are listed in
`design-prereq-x-ho-field-call.md` §Prereq-X1 "Migration cost"
and live under Substrate/Lowerer/Emitter authority, not
Evaluator.

---

## 2. Evaluator surface that would move to `TransformDispatch` /
   `CalleeRef` / `input_ports()`

If the substrate collapse described in §X1.b lands as written,
the evaluator-class consumers above migrate as follows:

| Consumer | Today | After X1.b |
|---|---|---|
| `lib.rs:eval_transform_node` operand walk (`for port in &t.inputs`) | iterates `t.inputs: Vec<PortId>` | iterates `t.dispatch.input_ports()` (single-authority iterator over runtime ports across every variant) |
| `lib.rs:eval_transform_node` variant match | matches `&t.target` against four `TransformTarget::{Callable, FieldProject, Operator, …}` arms | matches `&t.dispatch` against five `TransformDispatch::{Callable, FieldProject, FieldCall, Operator, Indirect}` arms; `Indirect` is new and currently fail-closed-eligible |
| `lib.rs` arity checks (`operands.len() != 2`) | reads `operands.len()` derived from `t.inputs.len()` | unchanged — arity is structural in `OperatorCall::Unary | Binary` and the host-Rust evaluator only implements `Operator` arms today, where the arity is fixed by op-kind |
| `lib.rs` `EvalError::UnsupportedTransformTarget { kind: "Callable" | "FieldProject" | "Logical" }` | reads `t.target` variant for the kind label | becomes `kind: "Callable" | "FieldProject" | "FieldCall" | "Indirect" | "Logical"` (string surface widens; new `FieldCall` and `Indirect` are also fail-closed initially, until G0c+ extends them) |
| `lens_apply.rs:eligibility_walk_transform` input walk | `for input in &t.inputs` | `for input in t.dispatch.input_ports()` |
| `lens_apply.rs:eligibility_walk_transform` variant match | three explicit arms (`Operator(Logical)`, `Operator(_) | FieldProject`, `Callable`) | five-variant match. `FieldCall` collapses to "carrier eligible AND args eligible AND callable-target eligible" (project-then-call composition); `Indirect` is **ineligible** for the bounded lens-apply walker today — runtime-sourced callee leaves the static-resolution domain that `eligibility_walk_callable` operates in |
| `lens_apply.rs:eval_transform` variant match | four arms over `t.target` reading `t.inputs` directly | five arms; `FieldCall` arm projects the field then dispatches the resolved callee (composing the `FieldProject` and `Callable` arms); `Indirect` arm fails closed via `LensApplyError::UnsupportedConstruct("indirect dispatch in lens apply")` |
| `lens_apply.rs:eval_transform` `Callable` fold-detection (`is_fold_instantiation`, `find_fold_step_bind_via_instantiation`, `t.inputs[0]`/`t.inputs[1]`) | reads `t.inputs[0]` / `t.inputs[1]` positionally | reads `d.args()` on `CallableDispatch`; `t.inputs[0]/[1]` becomes `d.args()[0]/[1]`. Semantics unchanged. |
| `lens_apply.rs:eval_transform` `FieldProject` arm (single-port arity check + `project_field`) | reads `t.inputs.len()` and `t.inputs[0]` | `FieldProjectDispatch` is single-carrier (`carrier: PortId`), no `args` — arity check retires; `project_field(self.eval_port(d.carrier())?, d.field_label())` |
| `lens_apply.rs:reflect_transform_target` | reflects `&TransformTarget` into a substrate sum | reflects `&TransformDispatch` into the substrate sum. Substrate-side `TransformTarget` declaration in `dsl/std/` retires or is rewritten; the reflection surface follows whatever substrate name the X1.b slice settles on |
| `dimension.rs:expand_behavior_backward` `transform.inputs` push | `for input in &transform.inputs { frontier.push(*input); }` | `for input in transform.dispatch.input_ports() { frontier.push(*input); }` — pure mechanical replacement |

**Single common shape after migration.** Every evaluator-class
consumer of `t.inputs` collapses to `t.dispatch.input_ports()`;
every consumer of `t.target` collapses to `&t.dispatch`. There
are no evaluator consumers of `t.inputs` that *cannot* be served
by `input_ports()` — every read above is either a flat dependency
walk (E1, E3) or a positional lookup that becomes a per-variant
accessor (`d.args()`, `d.carrier()`).

**`ArrowPortRef` does not appear on the evaluator surface
directly.** Evaluator code reads `IndirectDispatch::callee()`
through the dag accessor; the proof that the port is Arrow-typed
is established at construction (`Dag::resolve_arrow_port`) and
preserved through the typed handle. The evaluator never
reconstructs an `ArrowPortRef` and never witnesses
`NonArrowPortError`. This is consistent with §X1.b's
"type-checking happens at the type-handle's construction
boundary" claim.

---

## 3. What must remain blocked until G0c lands; what is Substrate
   authority rather than Evaluator authority

### Blocked-on-G0c
- `lib.rs:eval_transform_node` arms for `Callable` and
  `FieldProject` are currently fail-closed
  (`UnsupportedTransformTarget`). G0c is the slice that turns
  `Callable` and `FieldProject` from fail-closed into evaluated.
  X1.b cannot meaningfully extend the host evaluator into
  `FieldCall` / `Indirect` until `Callable` and `FieldProject`
  themselves evaluate — `FieldCall` is project-then-dispatch
  (composes both), `Indirect` is parameter-sourced dispatch (a
  generalization of `Callable` over runtime ports).

  Concretely: the X1.b implementation slice **must not** extend
  `eval_transform_node` to evaluate any new dispatch variant. The
  host evaluator stays fail-closed for `FieldCall` and
  `Indirect` after the substrate collapse; G0c+ owns the
  evaluation extension.

- `lens_apply.rs:eval_transform` `Callable` arm already evaluates
  via `eval_callable` (the bounded lens-apply walker is the
  evaluator that *does* dispatch user functions today). Its
  `FieldCall` arm post-X1.b composes `FieldProject` + `Callable`
  using existing primitives; this is genuinely an Evaluator
  follow-up, not a G0c gate. It is, however, a follow-up to the
  substrate collapse — it should not land in the same slice.

- `lens_apply.rs:eval_transform` `Indirect` arm is *not*
  unblocked by G0c. Runtime-callee dispatch in the bounded
  lens-apply walker requires `eligibility_walk_callable` to
  resolve a port-sourced callee back to a static decl, which is
  an additional evaluator design question (port→`ArrowPortRef`→
  `Bind` resolution under the depth budget). This is an
  Evaluator follow-up, but downstream of both X1.b substrate
  carrier and G0c host-eval extension.

### Substrate (not Evaluator) authority
The following all stay with Substrate/Director and **must not**
land inside an Evaluator-authored implementation slice:

1. `TransformTarget` → `TransformDispatch` carrier collapse on
   `TransformNode` (`dag.rs:1874-1912`).
2. `TransformNode.inputs: Vec<PortId>` retirement (collapsed into
   per-variant payload structs).
3. New variants: `FieldCall(FieldCallDispatch)`,
   `Indirect(IndirectDispatch)`. Module-private payload fields;
   public accessors only.
4. `ArrowPortRef` typed handle + `Dag::resolve_arrow_port`
   constructor.
5. New `Dag::push_*_transform` builders that co-construct
   args-against-target proofs.
6. `TransformDispatch::input_ports()` single-authority dependency
   iterator.
7. Lowerer (`lower.rs`) emission of `Indirect` dispatch when the
   callee resolves to a parameter port rather than a decl.
8. Substrate-side `dsl/std/` declaration of the renamed sum (the
   reflection surface in `lens_apply.rs:reflect_transform_target`
   follows whatever substrate name lands).
9. Generated-lens regen (`lens_effect_enumeration_generated.rs`,
   `lens_cost_generated.rs`,
   `lens_cost_symbolic_generated.rs`,
   `lens_unused_parameters_generated.rs`,
   `bootstrap_generated.rs`,
   `bootstrap_generated_without_parse_surface.rs`).
10. Bootstrap-regen rendering
    (`regen_bootstrap_emit.rs:render_transform_node` /
    `render_transform_target`).
11. Emitter consumers (`emit.rs:1545+`,
    `emit/rust_target.rs:2814+ / 3499+`, `emit/python_target.rs`).
12. Infer consumers (`infer.rs:1109+ / 1617+ / 1810+ / 2159+ /
    2899+ / 2957+ / 3011+ / 3951+`), including
    `refinement_targets_equal` and the non-callable
    `bind_non_callable_target_arguments` path.

These belong to Substrate/Director routing because each is either
a substrate-shape edit (1–6, 8) or a non-evaluator consumer of the
substrate shape (7, 9–12). An Evaluator worker must not migrate
them; doing so is parallel-representation debt
(`feedback_parallel_representation_debt`).

---

## 4. Proposed implementation split for X1.b

Four ordered slices. Each is independently mergeable; sequencing
avoids racing G0c.

### Slice S1 — substrate carrier + builders (Substrate authority)
Owner: Substrate (not Evaluator).

- Author the `TransformDispatch` sum on `TransformNode`. Module-
  private payload fields. Public accessors only.
- Add `ArrowPortRef` + `Dag::resolve_arrow_port`.
- Add `Dag::push_callable_transform`,
  `push_field_project_transform`, `push_field_call_transform`,
  `push_indirect_transform`. Co-construct args-against-target
  proofs.
- Add `TransformDispatch::input_ports()`.
- Migrate `dsl/std/` reflection sum to match.
- Land atomically: `TransformNode.target` + `inputs` retire in
  the same commit that introduces `TransformDispatch`, with all
  in-tree consumers (§3 list 7–12 and §1 evaluator consumers)
  updated in lockstep. No "behind the existing `Vec<PortId>`
  shape" dual-authority bridge, no temporary parallel
  representation. Per `INVARIANTS.md` P2 Boundary Discipline
  (single-authority metadata) and
  `feedback_parallel_representation_debt`, an unbounded
  bridge between `(target, inputs)` and `dispatch` is
  disallowed; the only acceptable alternative to atomic
  migration would be a documented bounded bridge with a named
  dissolution trigger, and this audit explicitly does not
  authorize one — `Vec<PortId>` retires with `TransformDispatch`
  or S1 does not merge.
- Mechanical updates to all consumers in §3 list 7–12 plus the
  evaluator consumers in §1 (the evaluator updates here are
  pure mechanical — `t.inputs` → `t.dispatch.input_ports()`,
  `&t.target` → `&t.dispatch` — no semantic change). Renames
  fail-closed `kind` strings to match new variant names.

S1 produces zero net behavior change relative to its starting
point. Because S1 lands **after** G0c (per the hard precondition
below), "starting point" includes G0c's evaluated `Callable` and
`FieldProject` arms in `eval_transform_node`. S1 must preserve
that G0c-introduced evaluation verbatim through the rename to
`TransformDispatch::Callable` / `FieldProject`; it must **not**
revert those arms to fail-closed. The only newly-fail-closed
arms S1 adds are the X1.b-introduced variants `FieldCall` and
`Indirect`, which remain fail-closed until a later evaluator
slice (S3) implements them. The lens-apply evaluator preserves
its existing `Callable` / `FieldProject` evaluation through the
rename, and likewise gains fail-closed arms for `FieldCall` and
`Indirect`.

**Hard precondition:** S1 must not start until E6-G0c (host
evaluator `FieldProject`/`Callable` work) is far enough along
that S1's mechanical evaluator-arm renames do not conflict. The
cleanest sequencing is **S1 lands after G0c merges**, since G0c
edits the same `eval_transform_node` arms S1 renames. Director /
Substrate confirms.

### Slice S2 — lowerer support for runtime-sourced callee (Lowerer authority)
Owner: Lowerer (not Evaluator).

- Lower call-on-parameter-Arrow-typed-port to
  `TransformDispatch::Indirect(IndirectDispatch { callee:
  ArrowPortRef, args })` via `Dag::push_indirect_transform`.
- Wire the parser (X1.a already parses the surface form for
  field-calls; see PR #1699) to produce the parameter-callee
  shape that the lowerer routes through `Indirect`.
- Lowerer-side test matrix `T1.2` (parameter field-call) +
  `T1.4` (non-Arrow callee diagnostic).

S2 is purely a *production* of new dispatch shapes; consumers
fail-closed by construction from S1.

### Slice S3 — evaluator extension into `Indirect` (Evaluator authority, post-G0c)
Owner: Evaluator (post-G0c).

This is the *first* genuinely Evaluator-authored slice in X1.b,
and the only one that should be filed against this audit.

- Extend `lib.rs:eval_transform_node` `Indirect` arm to evaluate
  via the runtime-port's bound value. The `ArrowPortRef` proof
  guarantees only that the producing port carries an
  Arrow-*typed* value; how that Arrow value resolves to an
  evaluatable callable body (a top-level `Bind`, a closed-over
  `Bind` reference, a parameter that itself was bound to a
  callable elsewhere, …) is the open Evaluator design question
  for this slice. Composes with G0c's `Callable` evaluation
  once the resolution path is settled.
- Extend `lens_apply.rs:eval_transform` `Indirect` arm.
  Eligibility walker (`eligibility_walk_transform`) gains a
  port-resolution step: chase the `ArrowPortRef`'s producing
  bind under the depth budget. This is the Evaluator design
  question deferred from §3.
- Same composition for `FieldCall`: `eval_field_project →
  eval_callable`. (`FieldCall` may be merged into S3 if G0c lands
  `FieldProject` evaluation; otherwise it splits as S3a.)

**Hard precondition:** G0c merged.

### Slice S4 — emitter / lens consumer follow-up (Emitter / Lens-author authority)
Owner: Emitter / lens-author (not Evaluator).

S1 already updates emitters and generated lens code mechanically
to read `t.dispatch.input_ports()` etc. S4 is the *non-trivial*
emitter work that S1 stubs out:
- Render `Indirect` dispatch in `emit/rust_target.rs` as a
  closure call against the resolved Rust binding for the
  `ArrowPortRef`.
- Render `FieldCall` as `<carrier>.<field>(<args>)` (or the
  per-target equivalent) once carrier rendering is structural.
- Update lens authoring (`dsl/std/` lenses that match
  `TransformTarget`) to pattern over the new sum.

S4 is downstream of S1 and independent of S2/S3. It can land in
parallel with S2 once S1 merges.

---

## 5. Non-goals and STOP conditions

### Non-goals
- This audit does **not** author an X1.b implementation brief. The
  S1 slice is Substrate-cross-cutting and routes through
  Substrate/Director per §3.
- This audit does **not** size G0c. G0c is `fierce-bear`'s slice
  and owns the host evaluator `FieldProject`/`Callable`
  surface. The audit only names the sequencing dependency.
- This audit does **not** prescribe whether `FieldCall` and
  `Indirect` should land in the same substrate slice or in two
  slices. Substrate worker decides; both shapes are unrelated
  enough at the type level (`FieldCall` is a structural
  dissolution of "project + call"; `Indirect` adds runtime-callee
  capability) that they could split if S1 is too large.
- This audit does **not** attempt to dissolve the
  `Callable` / `FieldCall` / `Indirect` triple into a single
  `Call { callee: CalleeRef, args }` variant. Per §X1.b's 🟡
  ledger entry, that collapse requires `CalleeRef` to carry the
  same args-bound-to-target proof and is "not a blocker for X1; a
  follow-up modeling slice." This audit honors that deferral.

### STOP conditions
Halt the X1.b implementation slice and route back to Director if
any of the following hold at S1 authoring time:

- **Substrate authority unresolved.** If §X1.b's substrate-shape
  claims (module-private fields, `ArrowPortRef` proof bound to
  `resolve_arrow_port`, `input_ports()` as single dependency
  authority) are challenged by an in-flight substrate redesign
  (e.g., a `CalleeRef` collapse landing earlier than X1.b),
  S1 must wait. The audit's variant naming presumes the §X1.b
  shape; if Director has converged on the future
  `Call { callee: CalleeRef, args }` shape directly, S1 should
  author *that* shape and skip the transitional `Indirect`
  spelling.
- **G0c not merged at S1 start.** S1's mechanical evaluator-arm
  renames conflict with G0c's `Callable` / `FieldProject`
  evaluation. Wait for G0c to merge, then start S1.
- **Generated-lens regen unstable.** S1 touches
  `bootstrap_generated.rs` plus four `lens_*_generated.rs`
  files. If a regen-stability ratchet is open
  (`v2_strict_compile_diagnostic_count` or equivalent regen
  receipts), pause until that ratchet is green.
- **Reflection authority drift.** If `reflect_transform_target`
  in `lens_apply.rs:1996-2030` is mid-migration to a different
  reflection surface (e.g., a structural `assert_sum_variant_payload_matches_substrate`
  pivot), align with that migration before adding the new
  variants.
- **Audit collapses to "already covered."** If a future
  substrate slice lands `CalleeRef` directly and X1.a's
  `PathCall` retires (per the X1.a worker brief's tracking
  gate, `r3-pr-e6-g0b-x1a-static-field-call-worker.md:226-232`),
  re-evaluate whether X1.b is still needed at all — the
  parameter-callee form may lower directly through
  `CalleeRef::Port(ArrowPortRef)` without an intermediate
  `Indirect` variant.

---

## Cross-references

- `docs/design-prereq-x-ho-field-call.md` — parent design,
  §Prereq-X1 / L1.b / `TransformDispatch` / `CalleeRef` ledger.
- `docs/briefs/r3-pr-e6-g0b-x1a-static-field-call-worker.md` —
  the X1.a static-callee worker brief (PR #1699 already merged
  per `git log`).
- `src/v3/DOWNSTREAM_REQUIREMENTS.md` — substrate carrier-shape
  ledger (the canonical place to file the §3 list-7–12 consumers
  before S1 starts, per `feedback_enumerate_before_substrate`).
- `src/v3/compiler/src/lib.rs:471-563` — host evaluator
  `eval_transform_node`.
- `src/v3/compiler/src/lens_apply.rs:71-89, 525-612, 1996-2030` —
  lens-apply evaluator transform consumers.
- `src/v3/compiler/src/dimension.rs:108-126` — adjacent
  dependency walker.

— sunny-hawk-622 (R3 Evaluator), 2026-05-04
