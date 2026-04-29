# design-reflection-completeness.md

**Director-authored design lock — PR-C (Item 2 of pre-spawn Tier 1 escalation, 2026-04-29).**

Resolves the second open question in `docs/r3-structure.md` §"Design challenges to resolve up-front" ("Lens reflection completeness scope") and supersedes the placeholder language in `docs/r2-structure.md` §6 ("Lens application — extend `reflect_program_dag_nodes_in_file` from 'shallow/lossy' to complete reflection").

Authored as a standalone design doc per Director ↔ PM coordination 2026-04-29: PB-Runtime + bin-shim items (4+5) get a separate co-authored doc; reflection completeness is its own surface because the dissolution path runs through the Evaluator + lens-author seam, not the PB-Runtime seam.

---

## 1. Scope

`reflect_program_dag_nodes_in_file(program: &Dag, source_file: &str, id_space: &Dag) → FieldValue` (currently `src/v3/compiler/src/lens_apply.rs:314`) is the structural projection consumed by lens-instance bodies. It takes a substrate-shaped `Dag` and produces a lens-input shaped `FieldValue` that carries program-DAG content for static lens analysis. **Lens analysis is static** — reflection extracts structural facts; it does not execute the reflected program. Execution is the Evaluator's job (PR-B).

This document specifies what fields are reflected for each substrate carrier, settles the three sub-questions surfaced in `docs/r3-structure.md:189-192`, and names the anti-bridge invariants T-LensProducer-Retirement (R3) workers must hold while retiring the Rust-side mirror in `lens_apply.rs`.

## 2. Current state (lossy reflection)

`src/v3/compiler/src/lens_apply.rs:943-1001` (`reflect_behavior`) emits the following per-variant payloads today:

| Substrate carrier | Substrate fields | Currently reflected | Currently dropped |
|---|---|---|---|
| `Behavior::Value(ValueNode)` | `id`, `payload`, `result_port`, `span`, `lane2_workflow?` | `payload`, `result_port` | `id`, `span`, `lane2_workflow` |
| `Behavior::Transform(TransformNode)` | `id`, `target`, `inputs`, `result_port`, `span` | `result_port` | `id`, `target`, `inputs`, `span` |
| `Behavior::Branch(BranchNode)` | `id`, `input`, `paths`, `result_port`, `span`, `emit_participation?` | `result_port` | `id`, `input`, `paths`, `span`, `emit_participation` |
| `Behavior::Loop(LoopNode)` | `id`, `source`, `init`, `body`, `bound`, `result_port`, `span` | `result_port` | `id`, `source`, `init`, `body`, `bound`, `span` |
| `Behavior::Bind(BindNode)` | `id`, `name`, `result_port`, `params`, `span`, `lane2_workflow?`, `emit_participation?` | `name` | `id`, `result_port`, `params`, `span`, `lane2_workflow`, `emit_participation` |

The Rust-side comment at `lens_apply.rs:925, 942` ("lossy `Behavior` spine") names this directly. Workers writing lenses today route around the gap by re-reading the substrate via Rust-side accessors instead of through the reflected `FieldValue` — that is the parallel-representation debt the Q6.5 / Q1 design locks already identified at the wider scope (per `feedback_parallel_representation_debt.md`). Reflection completeness retires that workaround at the source.

## 3. Closure principle (the structural-not-runtime axis)

**Lens analysis is static.** A lens consumes the program DAG without executing it. Therefore:

- "Loop iteration counts" are **structural** (a `PortId` reference into the same `Dag`), not runtime values produced by executing the loop. The lens reads `LoopBound::Cardinality.count: PortId` as the structural fact that this loop is bounded by a port-typed cardinality witness; algebraic analysis (e.g., `Interval<D>` per `design-emission-model.md` Q1 refinement) is a separate fold over that fact.
- "Branch arm coverage" is **total** (every arm), not "the arm that executes." Static analysis cannot determine which arm executes without running the program. Every `BranchPath` (its `body: NodeId`, `pattern: BranchPattern`, optional `binding: PayloadBinding?`, `result_port: PortId`) is reflected.
- "Witness structure" (`Witness<Carrier>` per `src/v3/std/dimensions.dag:35-37`) is a substrate-declared coproduct (`Inhabits` | `Violates`); it reflects like any other substrate value via its declared shape. No special construction step.

This is the load-bearing distinction. Reflection ≠ evaluation. Reflection presents structural facts in lens-input shape; evaluation (PR-B / Evaluator Manager) executes the reflected program against runtime values.

## 4. Decision

**Complete reflection means: every substrate-declared field on every `Behavior` variant — and every variant of every reachable substrate type those fields carry — is projected into the resulting `FieldValue` via the substrate-declared shape, with no per-consumer narrowing and no Rust-side hand-rolled cases.**

Concretely, for each carrier:

### 4.1 `Behavior::Value(ValueNode)`
Reflect: `id` (NodeId), `payload` (LiteralBits — already structural), `result_port` (PortId), `span` (SourceSpan), `lane2_workflow` (`WorkflowEffect?` — present-or-absent reflected as the optional's two-variant shape, NOT collapsed to a flag).

### 4.2 `Behavior::Transform(TransformNode)`
Reflect: `id`, `target` (the full `TransformTarget` coproduct: `Callable(DeclarationId)` | `FieldProject { field_label, field_child }` | `Operator(OperatorKind)`), `inputs: List<PortId>`, `result_port`, `span`. **`target` reflection is recursive on its declared shape** — every variant payload is projected, not stringified.

### 4.3 `Behavior::Branch(BranchNode)`
Reflect: `id`, `input` (PortId), `paths: List<BranchPath>` — for each path: `body: NodeId`, `result_port: PortId`, `pattern: BranchPattern` (the full coproduct: `UnresolvedVariant { name, span }` | `ResolvedVariant(DeclarationId)`), `binding: PayloadBinding?` (the present/absent shape, with payload reflected when present), `result_port`, `span`, `emit_participation: BranchEmitParticipation?`.

`paths` carries every arm, full stop. There is no "executed arm" notion at reflection time.

### 4.4 `Behavior::Loop(LoopNode)`
Reflect: `id`, `source: PortId`, `init: PortId`, `body: NodeId`, `bound: LoopBound` (the full coproduct: `Cardinality { count: PortId }` | `Descent { cluster: ClusterId }`), `result_port`, `span`.

`bound` is reflected structurally regardless of which variant — `Cardinality` carries a port reference, `Descent` carries a cluster reference. Both are structural facts. Lenses analyzing termination read `bound` as the carrier; algebraic analysis (`BoundedLattice<DescentEvidence>` per `design-emission-model.md` Q1 refinement) is a downstream fold, not a reflection-time decision.

### 4.5 `Behavior::Bind(BindNode)`
Reflect: `id`, `name`, `result_port`, `params: List<PortId>`, `span`, `lane2_workflow: WorkflowEffect?`, `emit_participation: BindEmitParticipation?`.

### 4.6 Body resolution

Behaviors carry `NodeId` references (e.g., `LoopNode.body`, `BranchPath.body`) into other behaviors in the same `Dag`. Reflection presents these as structural references (the `NodeId` value), not as inlined sub-trees. Lens-instance bodies that need to follow a reference fold over `Dag::nodes()` via the keyed accessors (`v3.std.substrate` per DB-14); no inlining at reflection time. This preserves DAG identity (same body referenced from two contexts is the same `NodeId`, not two copies) and keeps reflection a pure structural projection.

### 4.7 Witness structure (`Witness<Carrier>`)

Reflection of `Witness<Carrier>` follows the same rule: the declared coproduct shape projects to a `FieldValue::Variant` whose payload reflects each variant's fields via the substrate-declared shape. `Inhabits(c)` reflects with its `Carrier` payload (which itself reflects per its own substrate declaration); `Violates { reason: String, at: Behavior }` reflects with both fields, and `at` reflects via the `Behavior` rule above. No special-casing.

This subsumes the question raised in `r3-structure.md:179` ("are witnesses first-class runtime values, or constructed by a separate proof-mode evaluation pass?"): for **reflection** purposes, witnesses are structural values that reflect like any substrate-declared coproduct. For **evaluation** purposes, witness *construction* is the Evaluator's runtime concern (PR-B).

## 5. Sub-questions resolved (from `r3-structure.md:190-192`)

### 5.1 "Every Node reflected as a structural value, or every Node reflected via its substrate-declared accessor?"

**Both, because they are the same thing.** Reflection produces a `FieldValue` mirroring the substrate-declared shape. The DB-14 keyed accessors (`v3.std.substrate.dag:391-405`) are the runtime authority for lookups *over* `Dag.nodes` / `Dag.ports`; reflection is a fold *across* `Dag.nodes()` that produces the equivalent shape in `FieldValue` carrier. Same content, different carrier — runtime substrate values vs. lens-input `FieldValue` tree.

The framing distinction the r3-structure question raises does not survive scrutiny: there is no path where reflection would emit something *other than* the substrate-declared shape. The carriers' shapes are the authority; reflection projects them faithfully.

### 5.2 "Loop iteration counts: structural facts or runtime facts?"

**Structural.** `LoopBound::Cardinality.count: PortId` is a *port reference*, not a count integer. The port whose cardinality bounds the loop has whatever algebra the type system attaches to it (currently `Interval<D>` per `design-emission-model.md` Q1 refinement, with cost-typed and descent-typed bounds following their own asymmetric algebras per Q1 lock). Reflection presents the port reference; algebraic analysis is a downstream fold.

Runtime iteration counts (i.e., "how many times *did* this loop iterate when the program ran") live in the Evaluator (PR-B), not in reflection.

### 5.3 "Branch arm coverage: every arm, or only the executed arm?"

**Every arm.** No arm is "executed" at reflection time — reflection runs at lens-analysis time, before evaluation. `BranchNode.paths: List<BranchPath>` is reflected in full; each path's `body: NodeId`, `pattern: BranchPattern`, optional `binding: PayloadBinding?`, and `result_port: PortId` are all reflected.

This is the only consistent reading: dropping arms at reflection time would require reflection to know runtime control flow, which it cannot (lens analysis is static).

## 6. Anti-bridge invariants

**T-LensProducer-Retirement (R3) workers MUST hold the following while landing complete reflection:**

1. **No per-consumer projection.** Reflection is a single function over `Dag` → `FieldValue`. There is no `reflect_for_lens_X` / `reflect_for_lens_Y` family. Each lens consumes the same complete reflection and folds it according to its own algebra.

2. **No lossy elision at the reflection boundary.** A field declared on a substrate carrier MUST reflect into the corresponding `FieldValue`. Workers MUST NOT decide "lens A doesn't need this field, so skip it" at reflection time — that decision belongs in the lens body's fold, not in reflection.

3. **No execution semantics in reflection.** Reflection MUST NOT run the reflected program: it MUST NOT execute `Loop` bodies (no iteration), pick a `Branch` arm (every arm is reflected; no runtime control flow), or evaluate sub-DAGs referenced by `LoopNode.body` / `BranchPath.body` (the `NodeId` references are reflected as structural pointers per §4.6, not inlined trees). Those are evaluation concerns (PR-B / Evaluator).

   This is a constraint on **what reflection produces**, not on **how it is implemented**. The Evaluator-backed reflection-projection authority described in §7.2 (Rust-side `reflect_behavior` retirement *through* Evaluator's substrate-fact projection) is the intended implementation seam — that is structurally fine because the Evaluator there is running the *reflection* (a static structural projection), NOT running the reflected program. A reviewer who sees reflection executing the reflected program at lens-analysis time should treat that as a structural error; a reviewer who sees the reflection projection itself implemented via the Evaluator's substrate-fact authority is seeing the dissolution path land correctly, not a violation.

4. **No Rust-side hand-rolled `reflect_behavior` tail.** The dissolution target is `src/v3/compiler/src/lens_apply.rs:925-1001`. After T-LensProducer-Retirement closes, no Rust function should pattern-match on `Behavior` variants to construct a `FieldValue` — the substrate-declared shape drives reflection through the Evaluator's substrate-fact projection (parallel to how `analyze_symbolic_cost_dimension` is gated on Evaluator landing per `design-dimension-abstraction.md`'s execution-authority note).

5. **No structural carrier widening for reflection.** This spec consumes existing substrate carriers — `Behavior`, `LoopBound`, `BranchPath`, `BranchPattern`, `TransformTarget`, `Witness<Carrier>` — exactly as declared. T-Substrate-Lens-Primitive (R2) is the seam where new carriers land if the reflection consumer (lens body) discovers a missing structural fact; reflection itself does not introduce new carriers.

## 7. Cascade and gates

### 7.1 Substrate (R2)

**No substrate carrier change.** All five `Behavior` variants and all reachable types are already declared in the relevant `src/v3/std/` modules:

- `src/v3/std/substrate.dag`: `Behavior` (line 377), `ValueNode` (320), `TransformNode` (328), `BranchNode` (341), `LoopNode` (350), `BindNode` (366), `LoopBound` (316), `BranchPath` (251), `BranchPattern` (242), `TransformTarget` (232), `BranchEmitParticipation` (338), `BindEmitParticipation` (363).
- `src/v3/std/dimensions.dag`: `Witness<Carrier>` (line 35).
- `src/v3/std/effects.dag`: `WorkflowEffect` (line 549) — referenced via `ValueNode.lane2_workflow` and `BindNode.lane2_workflow`.

Reflection is a fold over already-existing structure; the **carriers stay untouched**.

This is the same shape as the Q6.5 disposition — structural completion at a non-substrate-shape seam (the lens-input projection in this case; the diagnostic-carrier widening in Q6.5).

### 7.2 R2-Evaluator (PR-B)

Evaluator landing is the **path** through which Rust-side `reflect_behavior` retires: once the Evaluator can execute `.dag` body authority for the reflection projection, the Rust mirror is the dissolution target. PR-B (Evaluator runtime-value model) is the prerequisite; this spec is the definition of "complete" the Evaluator-driven reflection MUST satisfy.

### 7.3 R3-T-LensProducer-Retirement

T-LensProducer-Retirement consumes this spec to retire `lens_apply.rs` + `lens_testgen.rs`. The three R3 internal sub-gates (per `r3-structure.md`):

1. `lens_apply.rs` — reflection + lens application moves to `.dag` body authority via Evaluator. Complete reflection is the structural target.
2. `lens_testgen.rs` — testgen consumes reflected programs through the Evaluator, not through the Rust-side mirror.
3. `regen_lens.rs` — bin-shim retirement, gated separately on PB-Runtime + bin-shim spec (Items 4+5, separate doc).

This spec gates sub-gates 1 + 2; sub-gate 3 is gated on the PB-Runtime doc.

### 7.4 Q6.5 (two-layer diagnostic-kind authority)

Lens-instance `validate` functions producing Layer-2 `Diagnostic` values fold over **complete** reflections. If reflection is lossy, lens-instance validates either (a) cannot identify the structural fact they want to diagnose, or (b) bridge around the gap via Rust-side mirror — both anti-bridge violations. Complete reflection closes that gap by construction: every structural fact a lens-instance validate could want to cite is present in the reflected `FieldValue`.

### 7.5 Q1 refinement (asymmetric bound algebra)

Reflection presents `LoopBound` structurally (`Cardinality.count: PortId` | `Descent.cluster: ClusterId`); algebra (`Interval<D>` for cardinality, `BoundedLattice<DescentEvidence>` for descent, `BoundedLattice<BigOClass>` for cost-bound carriers per Q3) is the downstream fold lens-instances apply to the reflected facts. This spec is the structural prerequisite; the algebra lock is the analysis layer.

## 8. Verification (TestClaim shapes)

Three `TestClaim` shapes exercise the spec at landing time. These are bootstrap hooks per `src/v3/std/verification.dag:108-217` `TestPredicate`'s `LensOutputEquals` / `DifferentialEquals` variants — they consume completeness as observable lens-output equality, not as a separately encoded "completeness" predicate.

### 8.1 Reflection-completeness fixture: every-field round-trip

For each `Behavior` variant, a fixture program containing one occurrence of that variant; a lens that reads each substrate-declared field through the reflected `FieldValue` and emits a structural witness; expected witness equals the substrate-shape ground truth.

Fails if any field is dropped at reflection. Verifies §4.1-4.5 mechanically.

### 8.2 Branch-arm-totality fixture

A `Branch` with three arms whose `body`/`result_port`/`pattern`/`binding` shapes differ; a lens that folds `paths.length` and per-path field reads into a structural list; expected list equals the substrate-shape ground truth (every arm present).

Fails if reflection emits fewer than the declared arm count.

### 8.3 Loop-bound-coproduct fixture

Two fixture programs: one with `LoopBound::Cardinality { count: PortId }`, one with `LoopBound::Descent { cluster: ClusterId }`; a lens that reflects `bound` and emits the variant tag + payload; expected emission equals the substrate-shape ground truth for each fixture.

Fails if reflection narrows the coproduct (e.g., picks one variant or stringifies the bound).

These are positive fixtures — they verify the reflection contract at a lens-input shape. The implementation seam (Rust-side mirror retirement) is verified by the existing T-LensProducer-Retirement scope: when `reflect_behavior` is gone from `lens_apply.rs` and the same fixtures still pass via the Evaluator path, the dissolution is complete.

## 9. Cross-references

- `docs/r3-structure.md` §"Design challenges to resolve up-front" #2 — this doc is its disposition.
- `docs/r2-structure.md` §6 (Evaluator Manager) — "complete reflection" placeholder; consumers replace with cite to this doc.
- `docs/design-lens-framework.md` §Q6.5 — two-layer diagnostic-kind authority depends on complete reflection at the lens-input boundary.
- `docs/design-emission-model.md` Q1 refinement — algebra over `LoopBound` carriers is asymmetric (cardinality / descent / cost); reflection presents structural; algebra is downstream.
- `docs/design-dimension-abstraction.md` — `analyze_symbolic_cost_dimension` execution authority lives in the Evaluator; reflection is the shape it consumes.
- `src/v3/std/substrate.dag:232-389` — substrate carriers reflection consumes (`TransformTarget`, `BranchPattern`, `BranchPath`, `LoopBound`, `BranchEmitParticipation`, `BindEmitParticipation`, `Behavior`, `Dag`).
- `src/v3/std/dimensions.dag:35-78` — `Witness<Carrier>` and `AnalysisDimension<Carrier>` reflection consumes.
- `src/v3/std/effects.dag:549` — `WorkflowEffect` consumed via `ValueNode.lane2_workflow?` and `BindNode.lane2_workflow?`.
- `src/v3/compiler/src/lens_apply.rs:314-1001` — current lossy implementation; dissolution target.
- `feedback_parallel_representation_debt.md` — anti-bridge invariant shape; reflection lossiness is parallel-representation debt at the lens-input boundary.
- `feedback_groundedness_gates_lenses.md` — lens analysis is static; reflection presents structural facts; runtime is the Evaluator's concern.
- `INVARIANTS.md` §P1 — substrate-fact-introduction procedure (no new carriers required for this spec; passes Step 1 / 2 / 3 trivially).
- `INVARIANTS.md` §P2 — single-authority for `Behavior` reflection: the substrate-declared shape is the authority; no per-consumer fork.

## 10. Status

**LOCKED 2026-04-29 (Director-authored, PM-confirmed sub-question dispositions per inbox #828 2026-04-29T00:36:47Z).**

Consumed by:
- R2-Evaluator brief (PR-B + PR-C cadence) — Evaluator Manager.
- R3-T-LensProducer-Retirement brief — PB Manager (sub-gates 1 + 2 only; sub-gate 3 gated on the PB-Runtime + bin-shim doc).

PM updates the affected briefs to flip "PR-C complete reflection — gated on Director authoring" → "PR-C complete reflection — LOCKED via design-reflection-completeness.md §4 + §5 + §6" once this lands.
