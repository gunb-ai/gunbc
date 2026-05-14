# Emit Pipeline Stage — L2.5 Domain Model (PB-6)

**Status:** DRAFT — Director-tier authoring per operator ratification 2026-05-14 ("can we author L2.5 models up front / agree on them"). Surfaces for operator review before PB-6 lane execution authority dispatches.

**Authoring date:** 2026-05-14.
**Authoring tier:** Director (zesty-bear-812).
**Lane:** PB-6 (emit) per `docs/design-pure-bootstrap-zero.md:104` + `src/v3/SELF_HOSTING.md` §2 4-step migration discipline.
**Migration order rank:** 1st (per `docs/substrate-reflection-design.md` §12.6 — emit→lower→infer→parse bottom-up).
**Routing authority chain:** operator-ratification → PM amends close plan + §1.8 PB-6 gate row → Director authors per-step worker briefs → R3 Substrate Mgr dispatches workers against Director-authored briefs.

---

## §1 Purpose + scope

This document is the **Step 1 model review** per `src/v3/SELF_HOSTING.md` §2.2 4-step discipline applied to PB-6 emit-stage migration. It declares the emit stage's input/output types in `.dag` substrate, the structural projection that composes them, the substrate prereqs the stage requires from sibling lanes, and the open design questions requiring operator ratification before Step 2 (pipeline-slot declaration) dispatches.

**This doc does NOT**:
- Author the `.dag` implementation (Step 3 work)
- Author the pipeline-slot declaration (Step 2 work)
- Design the parity test corpus (Step 4 work)
- Touch bootstrap-runtime-loop or PB-Substrate / PB-Bootstrap-Process / PB-Runtime concerns (separate lanes)

**Authority chain**: Director-tier ratification grounds the model; subsequent worker briefs (Steps 2–4) cite this doc as the substrate; §1.8 PB-6 gate row close-criterion predicate cites this doc as the L2.5 model authority.

---

## §2 What emit IS structurally

Per `docs/design-emission-model.md` ("Design — Emission Model (no separate coercion engine)") + `THESIS.md` §"Tier 1" ("Coercion = emission: the compiler reads a target spec and translates. No separate coercion engine"):

**Emit is a structural projection from (Dag, LanguageSpec) → (TargetSource ⊕ EmissionDiagnostic), composed via fold over per-target inhabitance facts. NOT a decision engine.**

Per `feedback_lenses_not_passes`: emit is a lens over substrate, not a decision process. Anything emit has to "decide" is a substrate fact the LanguageSpec should declare. The fold is small and mechanical because all the real work is in the substrate facts.

Per `docs/design-emission-model.md:44-48` (verbatim):
> "The fold is small and mechanical because all the *real work* is in the substrate facts. Anything the fold has to 'decide' is a fact the substrate should have declared."

**Failure shape**: fail-closed (per `feedback_fail_closed_discipline` + INVARIANTS C-8). When projection cannot resolve to a unique target primitive, emit returns a typed `EmissionDiagnostic`, never a fabricated default or fallback.

---

## §3 Input types (declared in `.dag` substrate)

Three input types feed emit:

### §3.1 `Dag` (post-infer)

The fully-resolved `.dag` program after parse → lower → infer. After infer completes, every `Port.state` is either `Resolved(TypeShape)` or `Unresolved` (with `state != Uninferred` invariant per the infer.rs header comment: `state == Unresolved iff diagnostics.contains(port_id)` — the diagnostic payload lives in the diagnostic table indexed by port_id, NOT in the `PortState` payload). The `Uninferred` variant is a pre-completion transient that infer guarantees is absent in the post-infer `Dag` emit consumes.

**Important — post-infer readiness must be modeled in the type, NOT enforced at runtime**: per `docs/modeling-discipline.md` Practice 2 (illegal states unrepresentable) + Practice 6 (API-level enforcement over convention) + `feedback_state_space_vs_behavioral_invariants` (type enforcement > API enforcement). The plain `Dag` type admits both pre-infer (`Uninferred` ports present) and post-infer states; emit consuming plain `Dag` would push enforcement to a behavioral runtime check, leaving the stage boundary convention-level. The correct shape is an `InferredDag` carrier (or equivalent typed-state newtype) that emit's signature requires by construction — `fn emit(d: InferredDag, s: LanguageSpec) -> EmissionResult`. §12 Q7 surfaces the resolution for operator ratification before Step 2 dispatch (Step 2 worker brief authoring depends on whether the carrier is `InferredDag` newtype, refined-`Dag` predicate-via-where-clause, or sum-variant `Dag = PreInferDag | InferredDag`).

**Substrate authority**: `src/v3/std/substrate.dag` (defines `Dag`, `Declaration`, `Port`, `TypeShape`).
**Lane dependency**: PB-Substrate (generates dag.rs from substrate.dag).

### §3.2 `LanguageSpec` (per-target spec authority)

Per-target structural facts: primitive set with refinement bounds, algebra inhabitance, structural axes distinguishing candidates, diagnostic enumeration order, construction patterns, operator dispatch, external-realization shape (per `docs/design-emission-model.md:193-209`).

**Substrate authority — LIVE V3 AUTHORITY** (verified via `grep -rn "type LanguageSpec" src/v3/`): `LanguageSpec` carrier declared at `src/v3/std/emit_model.dag:430` (DeclarationRef-shaped: statements/expressions/control_flow/literals/modules/functions/type_applications/type_definitions/record_derive_templates/patterns/collection_ops/values; also documented at `src/v3/SELF_HOSTING.md:592`). Per-target instance authority is the live v3 spec family at `src/v3/spec/rust.dag` / `src/v3/spec/python.dag` / `src/v3/spec/go.dag` (each carries 4 Realization meta-types: type realizations + operator realizations + behavior template realizations + per-target dispatch tables) + cross-target L1 substrate markers at `src/v3/spec/v3_l1.dag`. Shape A targets per `docs/thesis/what-else-falls-out.md` §"Two shapes of omni-emission".

**Legacy bootstrap layer to dissolve (NOT consumed as PB-6 authority)**: a DIFFERENT `LanguageSpec` schema is declared at `dsl/std/languages.dag:438` (older full-schema: language identity + syntax + runtime + value semantics + serialization + scaffold + service calls) consumed by the legacy decomposed per-target authority at `dsl/extdeps/languages/<target>/{syntax,runtime,errors,primitives,async,emit,imports,naming,lint,types}.dag` (multi-file decomposition). This is bootstrap scaffolding that PB-6 emission migration should **NOT** depend on; carrying it forward would constitute a P2 parallel-authority path (per codex BLOCKING #3066). Legacy-layer dissolution lives in a separate Director-tier lane (not PB-6 scope).

**§12 Q1 raises operator ratification BEFORE PB-6 Step 2 dispatch**: confirm v3 live authority (`src/v3/std/emit_model.dag` + `src/v3/spec/<target>.dag`) is the canonical PB-6 substrate; legacy layer treated as separate dissolution lane.
**Lane dependency**: T-Ground-LanguageSpec (R3 Grounding Mgr lane / Gap 13) — schema authoring; T-Ground-CrossTarget-Meta (R3 Grounding Mgr lane / Gap 13) — portability requirements.

### §3.3 `EmissionConfig` (target selection + shape disambiguation)

Selects which Shape A target to emit to (Rust / Python / Go for R3). **EmissionConfig is Shape-A-only by construction** — Shape B is user-space artifact emission (SPICE / English / YAML / Verilog / Terraform), authored by users as standalone `.dag` programs walking typed values via `concat`/`fold`/`match`. Shape B is NOT a compiler-emit dispatch axis and does NOT belong in PB-6 EmissionConfig surface (per `docs/thesis/what-else-falls-out.md` §"Two shapes of omni-emission" + `r3-structure.md` framing). PB-6 emit substrate is compiler-emit only.

**Substrate authority**: `src/v3/std/emit_config.dag` (proposed; may already exist as part of emit_model.dag — needs grep verification at Step 2 authoring).

---

## §4 Output types (declared in `.dag` substrate)

Two output types from emit; their disjoint sum is the emit return value:

### §4.1 `TargetSource` (the emitted code)

Structured per target: typically a typed string-tree composed via `std/string.dag` operations OR a typed AST representation of the target language. Per `docs/design-clean-emission-contract.md`, target source must satisfy every CleanEmissionContract rule **by construction** — violations are emission bugs, not warnings.

**Substrate authority**: `src/v3/std/target_source.dag` (proposed) OR refinement of existing `EmissionResult` carriers in `src/v3/std/emit_model.dag`.

### §4.2 `EmissionDiagnostic` (closed-axis sum-variant)

Fail-closed surface (per `docs/design-emission-model.md:166-190`):

```
type EmissionDiagnostic =
  | UnderRefined { program_intent: ProgramIntent, candidates: List<Candidate>, unspecified_axis: Axis, resolution_hints: List<Hint> }
  | NoInhabitant { program_intent: ProgramIntent, target: TargetRef, blocker: BlockerCause }
  | (additional variants surfaced during Step 2 authoring)
```

**Substrate authority**: T-Ground-Diagnostic (R3 Grounding Mgr lane / Gap 13) authors `EmissionDiagnostic` carrier; emit consumes it.
**Lane dependency**: T-Ground-Diagnostic.

### §4.3 `EmissionResult` (disjoint sum)

```
type EmissionResult = Either<TargetSource, EmissionDiagnostic>
```

Per `feedback_fail_closed_discipline` C-8 + `feedback_state_space_vs_behavioral_invariants`: the sum variant makes illegal states (e.g., partial-emit-with-no-diagnostic) unrepresentable. emit returns EmissionResult, never raw text without typed-state context.

---

## §5 Structural projection (the core)

emit composes from these substrate facts via fold:

### §5.1 Per-Behavior projection rule

For each `Behavior` variant in `Dag` — substrate-level `Value | Transform | Branch | Loop | Bind` per `src/v3/std/substrate.dag` (verified `src/v3/compiler/src/dag.rs:2600-2606`) — emit declares **one structural rule per behavior**, NOT decision logic. Note this `Behavior` axis is distinct from the type-level DAG primitive vocabulary (`Node/Conj/Disj/Cardinality/Bit` per `feedback_compiler_is_dag_processor`); the two operate at different layers (L1 substrate vs type-level primitives) and should not be conflated in worker briefs. Per `feedback_lenses_not_passes`: each `Behavior` variant has exactly one structural projection to the target language; if projection requires "deciding" between multiple targets, that's a missing `LanguageSpec` fact (axis distinguishing candidates).

### §5.2 Per-target inhabitance lookup

For each (Behavior, target) pair, `LanguageSpec` declares which primitive realizes the behavior in that target. emit reads the lookup; no decision-logic between candidates.

If multiple candidates satisfy a Behavior at a target, `LanguageSpec` must declare a **structural axis distinguishing them** (per `docs/design-emission-model.md:217-223` T-Ground-CrossTarget-Meta). Without the axis, emit fail-closes with `UnderRefined` diagnostic.

### §5.3 Mechanical walker dispatch via CleanEmissionContract

Per live substrate authority `src/v3/std/clean_emission.dag:13` (the canonical type declaration; supplements `docs/design-clean-emission-contract.md` DB-4 framing):

`CleanEmissionContract` declares **9 typed rule fields** per target covering constructive rendering + verifier-gate concerns (verified via `grep -n "type CleanEmissionContract" src/v3/std/clean_emission.dag`):

1. `expression_wrapping: ExpressionWrappingRule` (precedence-paren handling; rustc's `unused_parens`)
2. `pattern_bindings: PatternBindingRule` (match-arm payload binding emission; rustc's `unused_variables`)
3. `variant_payload_field_access: VariantPayloadFieldAccessRule` (variant payload access in pattern + body contexts)
4. `imports: ImportRule` (import emission policy; `unused_imports`)
5. `block_return: BlockReturnRule` (terminal-expression block-return handling)
6. `variable_bindings: VariableBindingRule` (variable binding rule; `_` underscore-when-unused vs explicit-name)
7. `match_arm_body: MatchArmBodyRule` (match-arm body formation)
8. `correction_style: CorrectionStyle` (formatting/lint correction emission strategy)
9. `post_emit_verifier: PostEmitVerifier` (embedded verifier-gate field — the contract itself carries the per-target verifier reference)

Walker dispatch is **mechanical**: match on rule variant, emit accordingly. **No `#[allow(...)]` / `# noqa` / pragma escape hatches** — violation = emission bug per `feedback_no_textual_enforcement_bridges`.

### §5.4 PostEmitVerifier discipline

Per `docs/design-clean-emission-contract.md:160-182` + live carrier at `src/v3/std/clean_emission.dag:22` (field of `CleanEmissionContract`): emit output gates through `PostEmitVerifier` (command + args + syntax_only + expected_exit_code + output_policy). For Rust: `rustc --edition=2021 -D warnings`. CI enforces no-suppression; verifier failure = emit failure.

**Substrate authority**: `src/v3/std/clean_emission.dag` declares `PostEmitVerifier` as the 9th field of `CleanEmissionContract` (NOT a separate top-level substrate file); PB-Runtime substrate work at `post_emit_verifier.rs` migrates the Rust-side runtime that consumes this field per cycle-5 PR #3057 paper-shrink revert pending.

---

## §6 Substrate prereqs (per-Gap-tier anchored)

Per `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`: anchor prereqs on Gap-tier identifiers, not session IDs.

| Prereq | Substrate authority | Gap-tier lane | Status at HEAD (2026-05-14) |
|---|---|---|---|
| PB-Substrate | `src/v3/std/substrate.dag` → dag.rs/ports.rs/effects.rs | Gap 13 R3 Grounding Mgr lane + R3 Substrate Mgr (warm-wolf-698) | In-flight (PR #3040 sum-variant landed; broader PB-Substrate work continues) |
| T-Ground-LanguageSpec | LIVE v3 authority: `LanguageSpec` carrier at `src/v3/std/emit_model.dag:430` + per-target instances at `src/v3/spec/{rust,python,go}.dag` (4 Realization meta-types each) + L1 markers at `src/v3/spec/v3_l1.dag`. Legacy bootstrap (NOT to consume as PB-6 authority): `dsl/std/languages.dag:438` + `dsl/extdeps/languages/<target>/*.dag` decomposition — separate dissolution lane | Gap 13 R3 Grounding Mgr lane | **§12 Q1 raises operator ratification BEFORE PB-6 Step 2 dispatch**: confirm v3 live authority canonical |
| T-Ground-Coercion-Fold | mechanical fold implementation | Gap 13 R3 Grounding Mgr lane | In-flight (PR #1980 ScratchIntExamples retirement; broader work continues) |
| T-Ground-Diagnostic | `EmissionDiagnostic` carrier | Gap 13 R3 Grounding Mgr lane | NOT-STARTED per closure-ledger; brief authored at PR #1216 |
| T-Ground-Lifetime-Analyzer | structural intent derivation | Gap 13 R3 Grounding Mgr lane | In-flight (R2-scope a/b/c impl landed at PR #1206) |
| T-Ground-CrossTarget-Meta | portability requirements meta | Gap 13 R3 Grounding Mgr lane | In-flight (PR #2103 L6 EmissionPathProjection CLOSED) |
| `target_source.dag` (if new) | TargetSource carrier | Director-tier substrate-fact-introduction | NEEDS canvas-ratification |
| `emit_config.dag` (if new) | EmissionConfig carrier | Director-tier substrate-fact-introduction | NEEDS canvas-ratification OR refinement of existing emit_model.dag |

**Critical observation**: 6 of 8 prereq rows above route through **R3 Grounding Mgr lane (Gap 13)** — PB-Substrate (Gap 13 + Substrate Mgr co-ownership) + T-Ground-LanguageSpec + T-Ground-Coercion-Fold + T-Ground-Diagnostic + T-Ground-Lifetime-Analyzer + T-Ground-CrossTarget-Meta. The remaining 2 (`target_source.dag` + `emit_config.dag`) are Director-tier substrate-fact-introduction scope. Gap 13 R3 Grounding Mgr lane is currently between-sessions (per still-dove-462 archive pattern). PB-6 emit Step 3 implementation cannot proceed substantively until R3 Grounding Mgr lane re-spawns AND closes the 3 key sub-lanes (LanguageSpec / Coercion-Fold / Diagnostic).

This is the **structural blocker** for PB-6 emit dispatch — substrate prereqs are upstream of pipeline-stage migration. Step 2 (pipeline-slot ExternalRealization) can proceed in parallel with Grounding Mgr work; Step 3 (implementation) must wait.

---

## §7 Cross-stage coordination

### §7.1 Upstream dependencies

emit depends on `Dag` whose `PortState` is post-infer-resolved → output of infer stage (PB-5). Per §3.1, post-infer readiness is modeled in the type via an `InferredDag` carrier (or equivalent typed-state newtype) per Modeling Practices 2 + 6 — illegal pre-infer state is unrepresentable at emit's signature boundary, NOT enforced at runtime. The signature is `fn emit(d: InferredDag, s: LanguageSpec) -> EmissionResult`. Per `src/v3/SELF_HOSTING.md` §2 migration order, emit migrates FIRST despite consuming infer's output, because the bottom-up principle is about which substrate authority needs to exist; emit substrate authority + LanguageSpec spec authorities are smaller surface than infer's.

emit does NOT depend on parse (PB-3) or lower (PB-4) directly — those produce `Dag` which infer (PB-5) refines. emit consumes the refined `Dag` only.

### §7.2 Downstream consumers

PB-Runtime (test_runner.rs / lens_apply.rs / post_emit_verifier.rs) consumes emit output for verification + lens execution. Parity test (Step 4) requires PostEmitVerifier substrate at HEAD.

R3 Verification Mgr's L4 emit/eval match gates + L5 cross-target consistency gates consume emit output for **typed semantic verification** (compile the emitted code, evaluate against canonical inputs, compare evaluation results) — NOT byte-equality of source text. Cross-target source bytes differ by construction (Rust vs Python vs Go syntax); the verification is at the semantic/behavioral layer. Byte equality is reserved for PB-6 Step 4 parity verification only — see §9 Step 4: emit.rs hand-Rust output vs emit.dag substrate output for the SAME target. Same-target byte identity is the discriminator that fails for paper-shrink class.

### §7.3 Sibling-stage coordination

When PB-4 lower / PB-5 infer subsequently migrate, their `.dag` implementations must produce typed-state outputs consistent with downstream stage expectations. Per §3.1 / §7.1, infer's output type is `InferredDag` — a carrier whose construction is gated on every port being `Resolved` or `Unresolved` (no `Uninferred` admitted). emit's signature `fn emit(d: InferredDag, s: LanguageSpec) -> EmissionResult` accepts only that typed-state; pre-infer `Dag` cannot be passed by construction. The stage boundary enforcement is API-level (Modeling Practice 6), NOT convention-level. §12 Q7 surfaces the exact carrier-shape (newtype vs refined-Dag-via-where-clause vs sum-variant) for operator ratification.

---

## §8 Two shapes of omni-emission

Per `docs/thesis/what-else-falls-out.md` §"Two shapes of omni-emission" + `r3-structure.md` framing:

### §8.1 Shape A — compiler targets (R3 scope)

Rust, Python, Go (primary for R3); Swift, Kotlin (post-R3). Each target grounds to a `LanguageSpec` declaring algebra inhabitance + structural axes + realization costs. **All Shape A targets emit from the same `Dag` value** — compositional layering ensures coherence by construction (same Node tree; no separate per-target compilation).

### §8.2 Shape B — user-space artifacts (DEFERRED post-R3)

SPICE netlists, English documentation, YAML configs, Verilog, Terraform — outputs of `.dag` programs written by users. NOT compiler targets. Shape B composition with omni-emission is deferred post-R3 per `r3-structure.md`. **Out of scope for PB-6 emit migration.**

---

## §9 SELF_HOSTING.md §2.2 4-step applied to PB-6 emit

| Step | Deliverable | Owner | Substrate |
|---|---|---|---|
| **Step 1: Model review** | THIS DOC | Director (zesty-bear-812) | docs/design-emit-stage-l25-model.md (this doc) |
| **Step 2: Pipeline slot** | `fn emit(d: InferredDag, spec: LanguageSpec) -> EmissionResult` declared in compiler.dag with `ExternalRealization` body (Rust-backed placeholder pointing to current emit.rs). **Exact `InferredDag` carrier shape gated on §12 Q7 operator ratification** — newtype / refined-`Dag`-via-where-clause / sum-variant `Dag = PreInferDag \| InferredDag`. Whichever shape lands, the signature accepts only post-infer typed-state by construction (Modeling Practice 6 API-level enforcement). Step 2 worker brief authoring routes through Director once Q7 ratified. | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 2 brief | compiler.dag refinement |
| **Step 3: Implementation** | `src/v3/std/emit.dag` (the .dag implementation of emit; fill the function body) | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 3 brief | src/v3/std/emit.dag (NEW substrate authority) |
| **Step 4: Parity test + simultaneous Rust deletion** | Byte-identical output vs emit.rs across full test matrix + `emit.rs` + sibling `*_target.rs` files DELETED in same PR | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 4 brief | Parity verification authored as `.dag` TestClaim — generated test fixture set + `.dag` TestClaim asserting `emit_via_rust(dag, spec) == emit_via_dag(dag, spec)` byte-equality across canonical corpus. **P5 dissolution receipt**: this TestClaim is transient-by-construction; it dissolves when emit.rs deletes in the same PR (Step 4 = parity + simultaneous deletion). Any hand-Rust scaffolding required for invocation routing between stage0 emit (reading new emit.dag via Evaluator) and emit.rs (current hand-Rust) bears P5 receipt: `parity_emit_dag_vs_rust_scaffolding — transient; dissolves with emit.rs deletion in same PR per Step 4 atomic discipline`. EXPECTED_HAND_AUTHORED_NON_TEST shrinks by N entries at PR-merge. |

**Critical: parity test is against EMIT.RS OUTPUT, not against emit.dag-template-of-emit.rs** (the discriminator that fails for cycle-4 PR #3048 / cycle-5 PR #3057 template-relocation paper-shrink class).

---

## §10 D-1 determinism invariant preservation

Per `src/v3/compiler/src/emit.rs:1-8` (current emit.rs top comment): D-1 invariant requires byte-identical output for fixed `(dag, target)` across successive calls. emit.dag implementation must preserve this; no Map iteration without sorted-key discipline; no HashMap (use sorted keys per `feedback_no_textual_enforcement_bridges` adjacent: structural enforcement of determinism).

Per Step 3 brief authoring: the `.dag` implementation must explicitly use structural iteration (sorted-key fold; no non-deterministic ordering primitives).

---

## §11 Cost lens cross-cutting consistency

Per `docs/design-emission-model.md:224-249` ("Worked example A" / "Worked example B" / "Worked example C"):

"Coercion cost = complexity" thesis claim must hold by construction:
- Cost lens reads LanguageSpec realization-cost declarations (per-primitive realization cost)
- Combined with per-op algebra cost (from substrate.dag) via structural fold
- No parallel per-target cost table (would violate P2 single-authority)

PB-6 emit migration must preserve this discipline: emit output cost = sum of per-op realization cost reads, NOT a separate cost calculation embedded in emit logic.

---

## §12 Open design questions (operator ratification)

These surface to operator before Step 2 (pipeline-slot) dispatch:

### Q1: LanguageSpec canonical-authority confirmation (REFRAMED per codex BLOCKING #3066)

**v3 live authority** (verified via grep): `LanguageSpec` carrier at `src/v3/std/emit_model.dag:430` (DeclarationRef-shaped) + per-target instances at `src/v3/spec/{rust,python,go}.dag` (each carrying 4 Realization meta-types: type realizations + operator realizations + behavior template realizations + dispatch tables) + cross-target L1 markers at `src/v3/spec/v3_l1.dag`.

**Legacy bootstrap layer** (verified via grep): different `LanguageSpec` schema at `dsl/std/languages.dag:438` consumed by older decomposed per-target authority at `dsl/extdeps/languages/<target>/{syntax,runtime,errors,primitives,async,emit,imports,naming,lint,types}.dag` (multi-file decomposition).

**Operator ratification question**: confirm v3 live authority (`src/v3/std/emit_model.dag` + `src/v3/spec/<target>.dag`) is the canonical PB-6 substrate; legacy `dsl/std/languages.dag` + `dsl/extdeps/languages/` decomposition gets a separate dissolution lane (NOT carried forward into PB-6 substrate). Carrying both forward would constitute a P2 parallel-authority path that codex BLOCKING #3066 explicitly flagged.

Director-recommend: **v3 live authority canonical**; legacy layer treated as separate dissolution lane (post-PB-6 scope); PB-6 Step 2 worker brief cites `src/v3/std/emit_model.dag` LanguageSpec carrier + `src/v3/spec/<target>.dag` per-target instances exclusively. This is consistent with `feedback_lenses_not_passes` (single substrate authority) + INVARIANTS P2 (no parallel-authority paths).

### Q2: emit.rs current `*_target.rs` siblings

emit.rs is single dispatch surface; each `emit_*_target.rs` sibling has one target-monolithic implementation body (per emit.rs:10-15). Do these get separate sub-models (per-target L2.5) OR fold into single `emit.dag` authority with per-target `LanguageSpec` driving projection?

Director-recommend: **single `emit.dag` authority with LanguageSpec-driven projection** — this is the structural-projection framing per §5; per-target siblings collapse into "one structural rule per behavior, target-distinguished via LanguageSpec inhabitance lookup". The current target-monolithic implementation is the artifact of pre-LanguageSpec architecture.

### Q3: Self-hosting bootstrap meta-circular constraint

emit.dag reads realization specs (which are themselves data in `src/v3/spec/*.dag` per `design-pure-bootstrap-zero.md:62-79`). At first compile, stage0 Rust emit reads the new emit.dag authority. **Meta-circular**: once emit.dag lands, hand-Rust emit must read it; before it lands, stage0 is sole authority.

How does Step 4 parity-and-delete handle this? **Director-recommend resolution**: Step 4 parity test uses stage0 Rust emit (final pre-deletion form) to emit code from emit.dag; compare byte-equality against current emit.rs output on canonical corpus. After parity passes, emit.rs deletes + bootstrap rebuild uses emit.dag (which stage0 Rust loaded from disk + executed via Evaluator). Bootstrap consistency preserved.

This depends on R3 Evaluator Mgr lane sub-lanes (body_evaluator + witness_construction) being GREEN at HEAD (per jolly-ram-652 PR #3053 audit: 3 GREEN + 2 in-flight) — both currently GREEN per jolly-ram-652 audit. **Evaluator ready for emit.dag execution.**

### Q4: Shape B omni-emission deferral boundary (RESOLVED — removed from PB-6 substrate)

emit.dag scope = Shape A only (Rust / Python / Go for R3). **Shape B does NOT belong in PB-6 EmissionConfig surface** — Shape B (SPICE / English / YAML / Verilog / Terraform) is user-space artifact emission authored as standalone `.dag` programs that walk typed values via `concat`/`fold`/`match`; it is NOT a compiler-emit substrate dispatch axis. PB-6 compiler-emit substrate has no Shape A/B disambiguation flag — `EmissionConfig` is Shape-A-only by construction (per §3.3 update + §13 Non-goals).

The earlier draft proposed `EmissionConfig.target_kind: ShapeAVariant | ShapeBVariant` (closed-axis sum) — that was a substrate-modeling error: putting Shape B inside compiler-emit substrate conflates user-program-artifact-emission with compiler-target-emission. Shape B emission is its own lane (post-R3 user-emission work), separate from PB-6.

This question is RETIRED from open-ratification status; resolution captured here for traceability.

### Q5: CleanEmissionContract enumeration — RESOLVED (per codex BLOCKING #3066)

Earlier draft framed this as "8 rules with 3 missing per design-clean-emission-contract.md" enumeration drift. The live substrate authority `src/v3/std/clean_emission.dag:13` carries **9 typed fields** (verified + enumerated in §5.3 update): `expression_wrapping` / `pattern_bindings` / `variant_payload_field_access` / `imports` / `block_return` / `variable_bindings` / `match_arm_body` / `correction_style` / `post_emit_verifier`.

Note in particular `variant_payload_field_access: VariantPayloadFieldAccessRule` which the earlier draft dropped — codex BLOCKING #3066 flagged this as INVARIANTS P2 (parallel-authority risk) + Modeling Practice 3 (facts-carry-forward) violation. The fix carries the live `clean_emission.dag` field set forward as substrate authority.

This question is RETIRED from open-ratification status; enumeration is live in §5.3 + carrier-of-truth is `src/v3/std/clean_emission.dag`.

### Q6: PB-Runtime PostEmitVerifier substrate dependency

PostEmitVerifier substrate (currently hand-Rust in post_emit_verifier.rs per cycle-5 PR #3057 paper-shrink revert pending) is the gate that ensures emit output passes verifier discipline. PB-6 Step 4 parity test depends on PostEmitVerifier being substrate-true (.dag) authority, not hand-Rust.

**Director-recommend**: surface PB-6 emit migration as DEPENDENT ON PB-Runtime post_emit_verifier.dag migration completing first. PB-Runtime is on R3 Evaluator Mgr lane scope per msg_e66f4326 (β) option OR warm-wolf-698 R3 Substrate Mgr scope per (α) option. Decision is operator-pending.

### Q7: Post-infer readiness — typed-state carrier shape (NEW — codex BLOCKING #3066; codex REQUEST_CHANGES #3066-post-fix)

Per §3.1 / §7.1 / §7.3 update, post-infer readiness MUST be modeled in the type at emit's signature boundary per `docs/modeling-discipline.md` Practices 2 (illegal states unrepresentable) + 6 (API-level enforcement over convention) + `feedback_state_space_vs_behavioral_invariants` (type enforcement > API enforcement). Plain `Dag` admits both pre-infer and post-infer states; emit consuming plain `Dag` would leave the stage boundary convention-level — codex REQUEST_CHANGES (PR #3066) flagged this as locking in a weaker boundary that modeling discipline asks to enforce structurally.

The director-recommend in the earlier draft (keep plain `Dag` + runtime `UninferredPortPresent` gate) was wrong on this axis. Acknowledged + reversed.

**Open question for operator ratification: which typed-state carrier shape?** Three valid options:

**Option (a) — `InferredDag` newtype**: simple wrapper around `Dag` whose constructor is the only path; constructor checks ports + returns `Result<InferredDag, InferDiagnostic>`. Pros: simple, idiomatic. Cons: extra wrapper-type.

**Option (b) — refined-`Dag` via where-clause/predicate**: per the language's refinement type system (per `feedback_groundedness_gates_lenses` + the refinement work in dsl/std/), `Dag where all_ports_inferred(d)` as the input type. Pros: structural refinement; reuses substrate refinement machinery. Cons: depends on refinement predicate substrate being ready at HEAD.

**Option (c) — sum-variant `Dag = PreInferDag | InferredDag`**: model the pipeline state as a closed-axis sum-variant on `Dag` itself; infer produces `InferredDag`, emit accepts only that variant. Pros: explicit state machine; aligns with `feedback_coproduct_dissolution` + closed-axis modeling discipline. Cons: requires refactoring all `Dag`-consuming code to handle both variants.

**Director-recommend now: option (b) refinement-via-where-clause IF refinement substrate at HEAD; otherwise option (a) newtype as transition shape**. Reasoning: option (b) is most aligned with the language's structural-refinement framing (refinements are first-class facts about typed values, not marker types); option (a) is the pragmatic fallback when refinement substrate isn't ready. Option (c) is most structurally explicit but has higher refactoring cost across all `Dag`-consuming sites; operator can pick (c) if the explicit-state-machine framing is preferred over wrapper-types.

Whatever shape is picked, the constraint is: **post-infer readiness IS modeled in the type at emit's signature; the runtime `UninferredPortPresent` framing is retired.**

Operator ratification needed before Step 2 dispatch (Step 2 worker brief authoring depends on which carrier shape — newtype vs refinement vs sum-variant).

---

## §13 Non-goals (out of scope for this L2.5)

- **`.dag` implementation of emit projection** — Step 3 work, separate brief
- **Per-pass tactical decisions** — Step 3 work
- **Test corpus design + parity-test harness implementation** — Step 4 work
- **Bootstrap-runtime-loop concerns** — PB-Substrate / PB-Bootstrap-Process / PB-Runtime / PB-Lib+PB-Build separate lanes
- **emit.dag self-application demonstration** — T-Lens-Self-Application separate scope (Gap 8)
- **Shape B omni-emission migration** — post-R3 scope per `r3-structure.md`

---

## §14 Acceptance criteria for this L2.5 model

This doc lands on main when:

1. ✅ All input types declared structurally with `.dag` substrate paths (§3)
2. ✅ All output types declared structurally with closed-axis sum-variant discipline (§4)
3. ✅ Structural projection composed without decision logic (§5 — per `feedback_lenses_not_passes`)
4. ✅ All substrate prereqs named with Gap-tier / Mgr-lane anchors (§6)
5. ✅ Cross-stage dependencies explicit (§7)
6. ✅ Two-Shape framing scope-limited to R3 (§8)
7. ✅ SELF_HOSTING.md §2.2 4-step concretely applied (§9)
8. ✅ D-1 determinism preservation discipline (§10)
9. ✅ Cost lens cross-cutting consistency (§11)
10. ✅ Open design questions enumerated for operator ratification (§12)
11. ⏳ Operator ratification on §12 Q1, Q2, Q3, Q6, Q7 (Q4 + Q5 both RESOLVED inline per codex BLOCKING #3066 (Q4 Shape B removed from PB-6 substrate; Q5 CleanEmissionContract 9-field enumeration adopted from src/v3/std/clean_emission.dag:13); ratification of remaining 5 questions lands as Director-tier follow-on or inline updates)

Post-ratification: this doc becomes the substrate authority for Step 2 worker brief authoring (pipeline-slot ExternalRealization PR) + §1.8 PB-6 gate row close-criterion predicate.

---

## §15 Authoring sequence post-ratification

1. **Operator ratifies §12 Q1, Q2, Q3, Q6, Q7** (or surfaces revisions; Q4 + Q5 both RESOLVED inline per codex BLOCKING #3066)
2. **PM amends close plan Gap 1** to route through PB-X lanes + cite this doc as PB-6 L2.5 substrate
3. **PM amends §1.8** with PB-6 gate row citing this doc as close-criterion authority
4. **Director authors PB-6 Step 2 worker brief** (pipeline-slot ExternalRealization PR scope) — for R3 Substrate Mgr (warm-wolf-698)
5. **R3 Substrate Mgr (warm-wolf-698)** dispatches Step 2 worker against Director-authored brief
6. **Director ratifies Step 2 PR substance + admin-merges** when CI clears
7. **Director authors PB-6 Step 3 worker brief** (`.dag` implementation scope) — conditional on T-Ground substrate prereqs (Gap 13 R3 Grounding Mgr lane) being green at HEAD
8. **R3 Substrate Mgr** dispatches Step 3 worker; iterate substantive review per §5.1 PR-template enforcement (5th axis: authority-direction)
9. **Director ratifies Step 3 PR**, admin-merges
10. **Director authors PB-6 Step 4 worker brief** (parity test + simultaneous Rust deletion scope)
11. **R3 Substrate Mgr** dispatches Step 4 worker; parity must be against `emit.rs` OUTPUT (not against template)
12. **Director ratifies Step 4 PR**, admin-merges → emit.rs DELETED → PB-6 gate row CLOSES per §1.8

Subsequent L2.5 models (PB-4 lower / PB-5 infer / PB-3 parse / PB-2 tokenize) follow same Director-tier authoring → operator ratification → worker brief dispatch sequence per migration order.

---

## §16 Cross-references

**Primary authority**:
- `src/v3/SELF_HOSTING.md` §2.2 (4-step migration discipline)
- `docs/design-pure-bootstrap-zero.md` (PB-X lane framing)
- `docs/substrate-reflection-design.md` §12.6 (migration order)
- `docs/design-emission-model.md` (engine-reframe; no-separate-coercion-engine)
- `docs/design-clean-emission-contract.md` (DB-4 CleanEmissionContract)

**Secondary authority / context**:
- `THESIS.md` §"Tier 1" (coercion = emission claim)
- `docs/thesis/what-else-falls-out.md` §"Two shapes of omni-emission" (Shape A vs Shape B framing; see also THESIS.md Shape A/B references at the omni-emission claims)
- `src/v3/compiler/src/emit.rs:1-30` (current emit.rs structure + D-1 invariant)
- `docs/r3-structure.md` (R3 scope; Shape A/B deferral)
- `docs/r3-actual-close-plan.md` Gap 1 (PB-0 close plan; pending Phase 2 amendment per msg_b9f9c36b)

**Memory disciplines applied**:
- `feedback_lenses_not_passes` (emit = projection, not decision engine)
- `feedback_fail_closed_discipline` C-8 (EmissionResult sum-variant)
- `feedback_state_space_vs_behavioral_invariants` (typed state, illegal states unrepresentable)
- `feedback_no_textual_enforcement_bridges` (no #[allow], structural enforcement)
- `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id` (Gap-tier anchors)
- `feedback_construction_over_ratchets` (model first, refinement of existing where possible)
- `feedback_substrate_principle_audit` (4-axis grep + invariant-conformance)

**Surfaces awaiting**:
- Operator ratification on §12 Q1, Q2, Q3, Q6, Q7 (Q4 + Q5 both RESOLVED inline per codex BLOCKING #3066)
- PM Phase 2 close plan + §1.8 amendments citing this doc
- R3 Grounding Mgr lane re-spawn (post-deployment-trigger) for substrate-prereq closure cascade
