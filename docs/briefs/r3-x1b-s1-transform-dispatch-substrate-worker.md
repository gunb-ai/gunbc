---
status: draft (wait-window; awaits R3 host restoration AND E6-G0c merge before dispatch)
authority parent: R3 Substrate Manager (#1739)
input packet: R3 PB (neat-bear-351, #1742) at #issuecomment-4377461513
hard precondition: E6-G0c merged (R3 Evaluator merry-gull-128 #1743 owns)
authority docs:
  - docs/design-prereq-x-ho-field-call.md §Prereq-X1 / §L1.b (Director-approved 2026-04-30 #1130)
  - docs/briefs/x1b-evaluator-impact-audit.md §3, §4 Slice S1, §5
disambiguation: This is **Prereq-X1.b** (HO field-call / runtime-sourced callee dispatch), NOT TV2 retirement audit "Decision 2" (`kernel_algebra_profile`). Same-label collision; different track.
---

# R3 X1.b S1 — `TransformDispatch` substrate carrier + atomic `(target, inputs)` retirement

## Context

Today's `TransformNode` carries `target: TransformTarget` (3 variants:
`Callable | FieldProject | Operator`) plus a sibling
`inputs: Vec<PortId>` whose meaning depends on the variant — pure
positional dependency for `Callable` / `FieldProject`, or operator
arguments for `Operator`. The runtime-sourced callee case (HO
field-call, parameter dispatch) cannot be expressed in this shape:
the callee must be sourced from a port, but `TransformTarget` has no
port-valued variant. The current shape also leaks parallel authority
— `target` is the routing fact; `inputs` is the dependency-walk
fact; reflected consumers walk `inputs` directly to derive
dependencies (per audit §1 evaluator consumer catalog).

§Prereq-X1 / §L1.b prescribes collapsing both fields into a single
`TransformDispatch` sum carrying its own typed payloads, with
`input_ports()` as the single dependency-walk authority and
`ArrowPortRef` as a typed witness for port-valued callees.

S1 is the **Substrate authority** slice: lands the carrier and
mechanical consumer updates. S2 (lowerer produces `Indirect`),
S3 (evaluator semantics for `Indirect` / `FieldCall`), and S4
(emitter follow-up) are downstream slices in different ownership.

## Audit authority — confirmed live at HEAD

The load-bearing audit `docs/briefs/x1b-evaluator-impact-audit.md`
is in tree at HEAD (verified by wait-window scan). Worker MUST
read the audit's §1, §3, §4, §5 in full before authoring code;
key facts inlined below for self-containedness, but the audit
itself remains the authoritative source — if the inline summary
diverges from the audit at dispatch time, the audit wins:

- **§1 Evaluator consumer catalog (inlined summary):** three
  classes of consumers reading `TransformTarget` / `TransformNode.inputs`
  directly — E1 host-Rust evaluator (`src/v3/compiler/src/lib.rs::eval_transform_node`);
  E2 lens-apply evaluator (`src/v3/compiler/src/lens_apply.rs`
  including `eval_transform` / `reflect_transform_target` /
  eligibility check); E3 adjacent evaluators that walk
  `inputs` only (`dimension.rs`). Worker re-greps these
  paths at dispatch.
- **§3 Substrate-adjacent consumer catalog (inlined
  summary):** `src/v3/compiler/src/dag.rs` (carrier home),
  `dag/builder.rs`, `lower.rs`, `infer.rs`, `emit.rs`,
  `emit/rust_target.rs`, `emit/python_target.rs`,
  `regen_bootstrap_emit.rs`, generated `lens_*_generated.rs` /
  `bootstrap_generated*.rs`. All mechanical updates per slice
  step 6 below.
- **§5 STOP conditions (inlined verbatim):** (a) substrate
  redesign in flight invalidating §X1.b shape (e.g.,
  CalleeRef-collapse re-scope); (b) regen / strict-compile
  ratchet red at S1 start; (c) reflection mid-migration drift
  between `dsl/std/` reflection sum and Rust `TransformDispatch`;
  (d) `Vec<PortId>` cannot retire atomically (some consumer's
  rewrite needs a bridge); (e) G0c not merged.

## Slice

Per audit §4 Slice S1, with steps below mirroring the audit's
ordering. Worker reads the audit's §4 in full before authoring;
the inline summary here serves as a dispatch packet, not a
substitute for the audit.

1. Introduce `TransformDispatch` on `TransformNode` with variants
   `Callable | FieldProject | FieldCall | Operator | Indirect`.
   Module-private payload fields; public accessors only.

   **Practice 4 dissolution-ledger marks (mandatory on the live
   declaration).** Per
   `docs/modeling-discipline.md#4-coproduct-dissolution`
   (Practice 4) — *"Any new Rust enum with N ≥ 2 variants must
   have a checkpoint comment naming its classification (🟢/🟡/🔴),
   with a ledger entry if GREEN or a named trigger if YELLOW."*
   Each variant of the new `TransformDispatch` enum MUST carry its
   🟢/🟡/🔴 mark
   as a doc comment on the variant itself, copying the per-variant
   ledger from `design-prereq-x-ho-field-call.md` §Prereq-X1
   (verified live at HEAD `:440-488`). Worker transcribes verbatim:

   - 🟡 `Operator { op: OperatorCall }` — future-dissolve.
     Tracking gate: `std/{int,bool,float}/` declares operator-
     algebra witness functions + parser desugars operator tokens
     to `Call(FunctionRef)` at parse time, collapsing `Operator`
     into the unified `Call { callee: CalleeRef = Decl(...) }`
     shape.
   - 🟢 `FieldProject` — keep. Pure value-access fact preserved
     from current `TransformTarget::FieldProject`; no dispatch.
     Distinct state family from `FieldCall`.
   - 🟡 `Callable` — future-dissolve. Tracking gate: emitter
     splits callee-rendering from dispatch match
     (`lens_*_emitter_split` shape). Future: `Call { callee:
     CalleeRef, args }` with `CalleeRef = Decl(DeclarationId) |
     Field { ... } | Port(ArrowPortRef)`.
   - 🟡 `FieldCall` — future-dissolve. Same tracking gate as
     `Callable`. Future shape: `Call { callee: CalleeRef::Field
     { label, child, carrier }, args }`.
   - 🟡 `Indirect` — future-dissolve. Same tracking gate as
     `Callable`. Future shape: `Call { callee: CalleeRef::Port(
     ArrowPortRef), args }`.

   **No 🔴 (dissolve-now) variants** — the design's no-malformed-
   state invariants (Facts Flow Forward, illegal-states-
   unrepresentable, args-bound-to-target) all hold under the 🟡
   set above; verbatim from the design-doc ledger entry at
   `:484-488`. The PR body MUST also carry the dissolution-ledger
   summary per the audit's PR-body discipline; the per-variant
   doc comments are the in-source receipt that satisfies Practice 4.
2. Add `ArrowPortRef(PortId)` (private constructor) and
   `Dag::resolve_arrow_port(p: PortId) -> Result<ArrowPortRef,
   NonArrowPortError>`.
3. Add typed builders: `Dag::push_callable_transform`,
   `push_field_project_transform`, `push_field_call_transform`,
   `push_indirect_transform`. Co-construct args-vs-target proofs
   per design doc.
4. Add `TransformDispatch::input_ports() -> impl Iterator<&PortId>`
   as the single dependency-walk authority.
5. Migrate `dsl/std/` reflection sum to mirror the new variants
   so generated / lockstep substrate stays consistent.
6. **Atomic** mechanical update of all in-tree consumers per audit
   §3 list 7–12 + §1 evaluator consumers:
   `src/v3/compiler/src/dag.rs`, `dag/builder.rs`, `lower.rs`,
   `infer.rs`, `emit.rs`, `emit/rust_target.rs`,
   `emit/python_target.rs`, `regen_bootstrap_emit.rs`, generated
   `lens_*_generated.rs`, `bootstrap_generated*.rs`, evaluator
   `lib.rs` (`eval_transform_node`), `lens_apply.rs`
   (eligibility / `eval_transform` / `reflect_transform_target`),
   `dimension.rs`. All mechanical:
   `t.inputs` → `t.dispatch.input_ports()`,
   `&t.target` → `&t.dispatch`.
7. **No bridge**: `TransformNode.target` and `TransformNode.inputs`
   retire in the **same commit** that introduces `TransformDispatch`.
   Per `INVARIANTS.md` P2 (single-authority metadata) +
   `feedback_parallel_representation_debt`, an unbounded
   `(target, inputs) ∥ dispatch` bridge is disallowed; this brief
   does **not** authorize a temporary bounded bridge.

### Behavior preservation

S1 produces **zero net behavior change** relative to its starting
point. Because S1 lands **after** G0c (hard precondition), starting
point includes G0c's evaluated `Callable` / `FieldProject` arms in
`eval_transform_node`. S1 preserves that evaluation verbatim through
the rename to `TransformDispatch::Callable` / `FieldProject`; it
must **not** revert those arms to fail-closed. The only
newly-fail-closed arms S1 adds are `FieldCall` and `Indirect`,
which remain fail-closed until S3.

The lens-apply evaluator (`lens_apply.rs`) preserves its existing
`Callable` / `FieldProject` evaluation through the rename and gains
fail-closed arms for `FieldCall` and `Indirect`.

### Pre-flight (mandatory before authoring code)

- Read `docs/design-prereq-x-ho-field-call.md` §Prereq-X1 / §L1.b
  in full; carry the dissolution-ledger 🟢🟡🔴 marks into the PR
  body.
- Read `docs/briefs/x1b-evaluator-impact-audit.md` §1 (evaluator
  consumer catalog), §3 (substrate-adjacent consumer catalog), §4
  Slice S1, §5 (STOP conditions).
- Update `src/v3/DOWNSTREAM_REQUIREMENTS.md` with the consumer
  enumeration **before** S1 work starts, per
  `feedback_enumerate_before_substrate`.
- Verify E6-G0c is merged on `main` (`git log --oneline main --
  src/v3/compiler/src/lib.rs` and confirm host
  `eval_transform_node` `Callable` / `FieldProject` arms are
  evaluated, not fail-closed). If unmerged, STOP — wait.

## Acceptance

- Atomic commit: `TransformDispatch` introduced, `target` + `inputs`
  retired, all consumers updated, generated files regenerated.
- **Practice 4 classification receipts on the live declarations
  (mandatory; both Rust and `.dag` sides).** Each variant of the
  Rust `TransformDispatch` enum at `src/v3/compiler/src/dag.rs`
  carries an inline 🟢/🟡/🔴 doc comment matching the design
  ledger at `design-prereq-x-ho-field-call.md:440-488` (verbatim:
  🟡 `Operator`, 🟢 `FieldProject`, 🟡 `Callable`, 🟡 `FieldCall`,
  🟡 `Indirect`; no 🔴). The reflected `dsl/std/` mirror sum
  variants carry the same marks (kept in sync per slice §1's
  Migrate-reflection-sum step). The marks must be present on
  the source in both files, not only in the design doc, brief,
  or PR body — per `docs/modeling-discipline.md` Practice 4
  Step 4 the in-source receipt is the load-bearing artifact.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet (`v2_strict_compile_diagnostic_count -- --ignored`)
  remains at 0 diagnostics.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- G0c-evaluated `Callable` / `FieldProject` arms in
  `eval_transform_node` and `lens_apply.rs` preserve evaluation
  (regression: existing E6 tests still green).
- `FieldCall` and `Indirect` arms fail closed with a typed error
  (no fabrication, no panic).
- ROADMAP receives a new entry under the appropriate section noting
  S1 landed; X1 dissolution ledger in
  `design-prereq-x-ho-field-call.md` flips L1.b 🟡 → 🟢 (or per
  whatever flip the design doc names).

## STOP-AND-ESCALATE

Per `x1b-evaluator-impact-audit.md` §5 STOP conditions:

- **G0c not merged.** Wait. S1 cannot land first.
- **Substrate redesign in flight invalidating §X1.b shape**
  (e.g., a `CalleeRef`-collapse re-scope). Audit names this as a
  Director STOP branch — surface to Director #828; do **not**
  silently re-shape the variants.
- **Regen / strict-compile ratchet red** at S1 start. Wait for
  green; S1 touches generated files and must not land into an
  unstable regen state.
- **Reflection mid-migration drift** between `dsl/std/` reflection
  sum and Rust `TransformDispatch` after step 5. Surface; this is
  a substrate-internal coherence break and S1 cannot proceed
  without it being clean.
- **`Vec<PortId>` cannot retire atomically** (some consumer's
  rewrite turns out to require a bridge). Surface to R3 Substrate
  Manager (#1739); the no-bridge rule is load-bearing per the
  audit and this brief, and re-scoping requires Director input.

## Sequencing post-S1

- **S2** (Lowerer authority): produces `TransformDispatch::Indirect`
  / parameter field-call paths. Different worker.
- **S3** (Evaluator authority, post-G0c): evaluator semantics for
  `Indirect` (and `FieldCall` composition where applicable).
  Different worker.
- **S4** (Emitter / Lens-author authority): non-mechanical emitter
  / lens-author follow-up. Independent of S2/S3.
- **E6-G1** (R3 Evaluator merry-gull-128 #1743): generic
  `fold_lens<C>` parametric runtime callee — downstream of the
  **full X1.b chain**. S1 alone is necessary, not sufficient
  (per `r3-pr-e6-lens-fold-readiness-audit.md`).

## Authority audit receipt

1. **Substrate exists?** No — `TransformDispatch` is the new
   carrier this slice introduces; today's `TransformNode` carries
   `target: TransformTarget` + `inputs: Vec<PortId>`. Live anchors
   at the time of this brief authoring (re-verify at dispatch):
   `pub enum TransformTarget` at `src/v3/compiler/src/dag.rs:1874`;
   `pub struct TransformNode` at `:1900`. No existing carrier
   dissolves the HO field-call gap.
2. **Existing brief?** This brief is the X1.b S1 worker dispatch.
   Authority docs: design-prereq-x-ho-field-call.md (§Prereq-X1 /
   §L1.b) is the design authority; x1b-evaluator-impact-audit.md
   is the consumer catalog + slice split. No prior worker brief
   for S1 itself.
3. **Design-doc match?** Verified: §Prereq-X1 / §L1.b prescribes
   exactly the `TransformDispatch` sum, `ArrowPortRef`,
   `resolve_arrow_port`, builders, and `input_ports()` listed
   here. The audit's S1 spec at §4 matches the design-doc
   recommendation 1:1.
4. **Citations live?** `src/v3/compiler/src/dag.rs:1874`
   (`pub enum TransformTarget`) and `:1900` (`pub struct
   TransformNode`) verified at HEAD by direct grep on this brief's
   authoring tree (post-cursor-review fix; the pre-fix range
   `:1695-1712` was wrong — that's the `ordinal_param_label` /
   `BindNode` side-table region). Worker re-verifies at S1 start
   since line numbers will continue to drift; cite by symbol
   (`pub enum TransformTarget` / `pub struct TransformNode`)
   alongside the line number to make drift recoverable.
5. **Carrier dissolves the bridge?** Yes — `TransformDispatch`
   collapses `target` and `inputs` into a single typed sum;
   `input_ports()` is the single dependency-walk authority,
   eliminating the parallel `target` (routing) ∥ `inputs`
   (dependency) authority pair. The `Indirect` variant carries
   `ArrowPortRef` to express runtime-sourced callees the current
   shape cannot express.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per parent
session #828 instruction (T-Numeric-Construction parallel-author
posture extended to X1.b after R3 PB delivered the input packet
at #1742 #issuecomment-4377461513). Ratification pending host
restoration AND E6-G0c merge from R3 Evaluator (merry-gull-128
#1743). Two-precondition gate; this brief is throughput-prep, not
immediate dispatch.
