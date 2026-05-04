# R3 PR-E E6-G0 First-Gate Narrowing

<!-- PROPOSAL — docs-only narrowing of the E6-G0 first-gate. No code, no substrate edits, no test_runner changes. -->


**Status:** docs-only narrowing. Picks the first concrete E6-G0 slice that is
truly unblocked on current `main` after `workflow_root_port` / `WorkflowRoot`
(Prereq-3a, #1232) landed. Reconciles the four candidate "first gates" that
already appear across the E6 docs and names exactly which one ships first,
which sequences after it, and what STOP/GO conditions move the gate.

**Parent authorities:**
[`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) §E6,
[`r3-pr-e6-lens-fold-readiness-audit.md`](r3-pr-e6-lens-fold-readiness-audit.md),
[`r3-pr-e6-post-blocker-gate-packet.md`](r3-pr-e6-post-blocker-gate-packet.md),
[`r3-pr-e6-lens-value-authoring-stop.md`](r3-pr-e6-lens-value-authoring-stop.md),
[`../design-prereq-x-ho-field-call.md`](../design-prereq-x-ho-field-call.md),
[`../design-lens-fold-prerequisites.md`](../design-lens-fold-prerequisites.md).

**Non-goals:** does not implement `fold_lens`, does not edit
`Lens<C>` / `Witness<C>` / `OptionalDiagnostic` / `DimensionReport<C>` /
`TransformTarget` / `TransformNode`. Does not author a `Lens<C>` value, a
Rust lens registry, per-lens callbacks, or a `TestPredicate`. Does not narrow
or remove `lens_apply.rs::fold_lens_over_reflected_program`; that seam stays
the reflect→apply compatibility surface.

**Closes #1532 debt receipt:** N/A — no hand Rust under `src/v3/` changes; no
Per-PR dissolution gate required.

---

## Can `fold_lens<C>` be authored in `.dag` today?

**No.** Three structural facts hold simultaneously on current `main`:

1. **Parser surface.** `lens.read(d, b)` parses as a field-access expression,
   then fails with `expected ..., got LParen`. Recorded by the
   `prereq_x_call_on_field_access_ratchet_test::x1_direct_field_call_blocked`
   ratchet at
   `src/v3/compiler/tests/integration/prereq_x_call_on_field_access_ratchet_test.rs`.
   Every `Lens<C>` consumer dispatch (`lens.read`, `lens.sequential.op`,
   `lens.branch`, `lens.iterate`, `lens.validate`) is a call-on-field-access.
2. **Lowering substrate.** Per
   [`../design-prereq-x-ho-field-call.md`](../design-prereq-x-ho-field-call.md)
   §Prereq-X1.b, a parametric body `fn fold_lens<C>(lens: Lens<C>, …) = …
   lens.read(…)` cannot lower today: `lens` is a parameter, so the callee is
   a runtime-port-sourced Arrow value with no `TransformTarget` variant that
   carries a `PortId`. The audited unblock is the
   `TransformNode.target+inputs → TransformDispatch` collapse with an
   `Indirect(ArrowPortRef, args)` variant. That is a substrate carrier edit
   and is not in scope for any docs-only PR.
3. **Lens-value authoring.** Per
   [`r3-pr-e6-lens-value-authoring-stop.md`](r3-pr-e6-lens-value-authoring-stop.md),
   no honest `Lens<C>` value can be authored on `main` because structural
   data-body checking does not yet substitute instantiated generic Conj
   fields (`Lens<C> → Lens<Int>`, `Monoid<C> → Monoid<Int>`). Without one
   declared lens **value**, every "fold consumer" path is fictional.

These three facts are independent. Each blocks `fold_lens<C>` independently.
The post-blocker gate packet's "E6-G0 = evaluator `FieldProject`/`Callable`
execution" is downstream of all three: even if the evaluator could execute
those targets, lowering produces neither (1) without parser X1 nor (2)
without `TransformDispatch::Indirect` for the parametric case, and there is
no declared lens **value** without (3).

## The four candidate first-gates and their actual ordering

The E6 docs name four candidate "first gates." Listed in topological order:

| # | Gate | Layer | Live on `main`? | Authority |
| --- | --- | --- | --- | --- |
| C | Generic Conj field substitution in structural data-body checking (`Lens<C> → Lens<Int>`, `Monoid<C> → Monoid<Int>`) | type-checker (no substrate edit) | NO | [`r3-pr-e6-lens-value-authoring-stop.md`](r3-pr-e6-lens-value-authoring-stop.md) §Next Dispatch (`lens_value_generic_conj_field_substitution_lands`) |
| A | Prereq-X1.a static call-on-field-access (`data v: WrapFn = { f: double }; v.f(x)`) | parser + lowerer (reuses existing `TransformTarget::Callable`) | NO | [`../design-prereq-x-ho-field-call.md`](../design-prereq-x-ho-field-call.md) §Prereq-X1.a |
| D | Evaluator `TransformTarget::FieldProject` and `TransformTarget::Callable` execution | evaluator (host Rust) | NO — both fail closed at `src/v3/compiler/src/lib.rs:556-561` | [`r3-pr-e6-post-blocker-gate-packet.md`](r3-pr-e6-post-blocker-gate-packet.md) §E6-G0 |
| B | Prereq-X1.b `TransformNode.target+inputs → TransformDispatch` collapse + `Indirect(ArrowPortRef, args)` variant | substrate carrier shape | NO | [`../design-prereq-x-ho-field-call.md`](../design-prereq-x-ho-field-call.md) §Prereq-X1.b |

**Key observation:** A and D are not interchangeable orderings of the same
gate. Without (A), lowering never produces a `TransformTarget::Callable` for
`v.f(x)`, so executing it in the evaluator is dead code. Without (C),
no `data` binding exists whose `f` is an Arrow-typed field, so even (A)
cannot be exercised on a Lens-shaped value. Without (B), `fold_lens<C>`
parametric body never lowers regardless of A/C/D.

Therefore the truly first unblocked actionable slice is (C). It is a
type-checker change, not a substrate-shape change. It does not require any
of (A), (B), or (D) to land first.

## E6-G0 selected: lens-value substitution-aware structural data-body check

**Gate name:** `lens_value_generic_conj_field_substitution_lands`
(name preserved verbatim from the lens-value-authoring-stop receipt — this
brief picks it as the first E6-G0 slice; it does not rename or fork it).

**Scope (substrate read-only):**

- Structural data-body checking substitutes instantiated generic Conj fields
  before validating field values. Specifically: when checking a
  `data <name>: Lens<Int> = { … }` body, `Lens<C>`'s field signatures are
  rewritten under `C ↦ Int` before `read`, `sequential`, `branch`,
  `iterate`, `validate` are typed; when checking `sequential: Monoid<Int>`,
  `Monoid<C>`'s `op` and `identity` are rewritten under the same
  substitution.
- The carrier shapes of `Lens<C>`, `Monoid<C>`, `Witness<C>`,
  `OptionalDiagnostic`, and `DimensionReport<C>` are unchanged. No new
  variant, no new field, no `Lens<Int>` mirror type.
- One minimal `Lens<Int>` fixture lowers to a `ValueBody::Structural` whose
  `name`, `read`, `sequential`, `branch`, `iterate`, and `validate` are
  present as typed fields, with `sequential.op` and `sequential.identity`
  read from the nested `Monoid<Int>` value.
- No evaluator change. No parser change. No new substrate carrier. No
  changes to `lens_apply.rs::fold_lens_over_reflected_program`.

**Owner boundary:** R2 substrate / type-checker authority (same lane that
landed Prereq-1 #1230 / #1239 and Prereq-3a #1232). Not the Evaluator
worker; E6-G0 in the post-blocker gate packet renamed to "evaluator
`FieldProject`/`Callable` execution" is sequenced **after** this slice and
keeps its name as **E6-G0b** in the schedule below to avoid renaming the
existing receipt.

**Acceptance tests:**

1. A minimal `data complexity_lens_seed: Lens<Int> = { … }` fixture (or the
   existing `src/v3/lenses/complexity.dag` body, if the slice un-defers it)
   type-checks and lowers without diagnostic. The `read`, `branch`,
   `iterate`, `validate`, and `sequential.op` fields each resolve as Arrow
   values with substituted `Int`-typed signatures. `sequential.identity`
   resolves as `Int`.
2. A negative fixture supplies `read: fn(Dag, Behavior) -> Witness<String>`
   for a `Lens<Int>` and is rejected with a typed substitution-mismatch
   diagnostic — not a generic-Conj-not-supported fail-closed.
3. The existing `prereq_x_call_on_field_access_ratchet_test` ratchets stay
   red. They are not retired by this slice; they are retired by Prereq-X1.
4. `fold_lens_over_reflected_program` and the four PROXY/STUB lens emitters
   are unchanged.
5. `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
   and the existing E1–E5 evaluator tests continue to pass.

**STOP+PING:**

- if the slice would add a `Lens<Int>` mirror, a `Monoid<Int>` mirror, a
  Rust lens registry, or any new substrate variant;
- if structural data-body checking would compile by inferring a host default
  for `sequential.identity`, `read`, or `validate` rather than reading the
  declared Conj-field after substitution;
- if removing `fold_lens_over_reflected_program` becomes part of the slice;
- if a downstream consumer (`fold_lens<C>` body in `.dag`, evaluator
  `FieldProject`/`Callable` execution) is bundled in.

## Sequencing after E6-G0

Once E6-G0 lands, the next gates in order are:

- **E6-G0b (evaluator field/call API).** Make `TransformTarget::FieldProject`
  and `TransformTarget::Callable` execute through the existing
  `evaluate_body` / `eval_node` / `eval_port` entry points in
  `src/v3/compiler/src/lib.rs`. Inputs are now non-empty because (C)
  produced at least one declared `Lens<Int>` value whose static field
  references can be projected and called. This is the "E6-G0" the existing
  post-blocker gate packet names; renamed to **E6-G0b** here only for
  ordering clarity. No substrate edit.
- **E6-G0c (Prereq-X1.a static call-on-field-access surface).** Parser +
  lowerer extension so `complexity_lens.read(d, b)` parses and lowers to
  `TransformTarget::Callable(decl_id_of_complexity_read)` via static field
  projection on the `data` binding's `ValueBody::Structural`. Reuses the
  existing static-callee dispatch path; no `TransformDispatch` collapse, no
  `Indirect`. Acceptance test matrix is `T1.1` from Prereq-X §X1.a.
- **E6-G1 (non-parametric report lifting).** A non-parametric per-lens fold
  (e.g., `fn fold_complexity(d: Dag) -> DimensionReport<Int> = …`) authored
  in `.dag` consuming `complexity_lens` as a static `data` binding. Lifts
  `Witness<Int>` and `OptionalDiagnostic` into `DimensionOk` /
  `DimensionFail` using the existing `DimensionReport<C>` shape. Cardinality
  loops only; `LoopBound::Descent` is an explicit residual per the
  post-blocker gate packet's test-7. **This is the first executable fold
  slice that does not require Prereq-X1.b** because the lens is statically
  bound, not a parameter.
- **E6-G2 (parametric `fold_lens<C>`).** Requires Prereq-X1.b
  `TransformDispatch::Indirect` substrate collapse. Sequenced separately;
  its substrate edit is not in this brief's scope.

The `lens-fold-file-path-semantics` ROADMAP item (file-path participation
semantics) is **not** promoted by this brief. The compatibility seam
`fold_lens_over_reflected_program` continues to filter by `SourceSpan.file`;
generic fold scope is settled by E6-G1's non-parametric concrete fold
(whole-Dag traversal) rather than by promoting the file-path filter.

## Why this narrowing is honest

- It does not duplicate `lens_apply.rs::fold_lens_over_reflected_program`.
- It does not fabricate a `DimensionReport<C>` value, a host default
  `identity`, or a Rust-only lens registry.
- It does not promote the file-path filter to a structural authority.
- It picks the slice that is **truly unblocked first** — a type-checker
  change with no substrate-carrier edit, no parser edit, no evaluator edit.
- It preserves both `fold_lens_over_reflected_program` (compatibility seam)
  and the existing E6 readiness/post-blocker/lens-value-authoring receipts;
  it only resolves the ordering question they leave implicit.
- It declares why E6-G2 (parametric fold) cannot ship in this lane and
  routes Prereq-X1.b substrate collapse to its existing audit owner.

## Cross-references

- `src/v3/std/lens.dag` — `Lens<C>` 6-field carrier (live).
- `src/v3/std/dimensions.dag` — `Witness<C>`, `OptionalDiagnostic`,
  `DimensionReport<C>` (live).
- `src/v3/std/substrate.dag:585-616` — `WorkflowRoot` / `workflow_root_port`
  (Prereq-3a, live).
- `src/v3/compiler/src/lens_apply.rs::fold_lens_over_reflected_program` —
  reflect→apply compatibility seam (preserved).
- `src/v3/compiler/src/lib.rs:556-561` — `TransformTarget::FieldProject` /
  `TransformTarget::Callable` evaluator fail-closed sites (E6-G0b).
- `src/v3/compiler/tests/integration/prereq_x_call_on_field_access_ratchet_test.rs` —
  parser ratchet pinning Prereq-X1 / X3 (red until E6-G0c / E6-G2 land).
- `src/v3/lenses/complexity.dag`, `src/v3/lenses/cost.dag` — lens instances
  deferred pending E6-G0.
