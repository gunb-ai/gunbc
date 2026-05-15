# Infer Pipeline Stage — L2.5 Domain Model (PB-5)

**Status:** DRAFT — Director-tier authoring per operator ratification 2026-05-14 (Decision 1.A scoping = Option A; "harder/more correct thing up front" for tradeoffs).

**Authoring date:** 2026-05-14.
**Authoring tier:** Director (zesty-bear-812).
**Lane:** PB-5 (infer) per `docs/design-pure-bootstrap-zero.md` + `src/v3/SELF_HOSTING.md` §2 4-step migration discipline.
**Migration order rank:** 3rd (per `docs/substrate-reflection-design.md` §12.6 — emit → lower → infer → parse for the 4 pipeline-stage migrations explicitly tabled there; `docs/design-pure-bootstrap.md` §"PB-2 — tokenize retire" (line ~134) extends the chain with tokenize. PB-6 landed at PR #3066, PB-4 in flight at PR #3077).
**Routing authority chain:** operator-ratification + PM-delegate (per 2026-05-14 directive) → PM amends close plan + §1.8 PB-5 gate row → Director authors per-step worker briefs → R3 Substrate Mgr (warm-wolf-698) dispatches workers.

---

## §1 Purpose + scope

This document is the **Step 1 model review** per `src/v3/SELF_HOSTING.md` §2.2 4-step discipline applied to PB-5 infer-stage migration. It declares the infer stage's input/output types in `.dag` substrate, the substrate-driven inference structure that composes them, the substrate prereqs the stage requires, and the open design questions requiring operator/PM ratification before Step 2 (pipeline-slot declaration) dispatches.

**This doc does NOT**:
- Author the `.dag` implementation (Step 3 work)
- Author the pipeline-slot declaration (Step 2 work)
- Design the parity test corpus (Step 4 work)
- Own implementation of bootstrap-runtime-loop or PB-Substrate / PB-Bootstrap-Process / PB-Runtime (separate lanes; referenced as dependencies)

**Authority chain**: Director-tier ratification grounds the model; subsequent worker briefs cite this doc; §1.8 PB-5 gate row close-criterion predicate cites this doc as L2.5 authority.

---

## §2 What infer IS structurally

Per `feedback_lenses_not_passes` + the live top-comment at `src/v3/compiler/src/infer.rs:1-30`:

**Infer is a substrate-driven type-propagation function: PreInferDag → InferredDag, dispatching on TypeConnective variants per the substrate's algebraic structure. NOT a decision engine — the "inference rule book" is EMBEDDED IN THE SUBSTRATE (TypeConnective + Declaration + AtomPayload variants implicitly carry the inference behavior via algebraic role).**

This is structurally distinct from PB-4 lower:
- **Lower** has EXPLICIT rules (ElaborationSpec) mapping surface forms → substrate behaviors per Decision 3.C
- **Infer** has IMPLICIT rules: dispatching on TypeConnective IS substrate-driven; no separate "InferenceSpec" carrier is needed beyond the live `src/v3/std/substrate.dag` algebraic structure

Per the live `infer.rs:5-15` rules:
- `Arrow { inputs, output, body }` → direct signature inference from input/output declarations
- `Atom(Identifier { name, resolved })` → follow resolved link OR look up name in declaration table (§8.9 inhabitance walk)
- Other TypeConnective variants → not callable; produce Unresolved + diagnostic

**Fail-closed (INVARIANTS C-8)**: every detectable problem routes through `Dag::mark_unresolved` (current Rust API) or equivalent `.dag` substrate operation. Post-infer invariant: `state != Uninferred for all ports` AND `state == Unresolved iff diagnostics.contains(port_id)`.

**Typed-state carrier per Decision 3.A operator-ratified shape** (sum-variant `Dag = PreInferDag | InferredDag` per `feedback_coproduct_dissolution` Practice 4): infer's signature is `fn infer(d: PreInferDag) -> InferredDag` — the variant transition IS the structural invariant of inference completion. Pre-infer state cannot exist in `InferredDag` by construction.

---

## §3 Input types (declared in `.dag` substrate)

### §3.1 `PreInferDag` (output of lower stage)

Per Decision 3.A operator-ratified, infer's input is `PreInferDag` variant of `Dag`. Construction invariant of PreInferDag: ports may be `Uninferred` (pre-completion transient) or `Unresolved` (lower's resolve failures); after infer, no `Uninferred` remains.

**Substrate authority — DEPENDS on Decision 3.A landing**: `src/v3/std/substrate.dag` `Dag = PreInferDag | InferredDag` sum-variant extension per PB-Substrate work. Currently `type Dag` is single-variant; the carrier change ripples across lower (PB-4) / infer (PB-5) / emit (PB-6).

**Lane dependency**: PB-Substrate (Decision 3.A sum-variant carrier extension); PB-4 lower (produces PreInferDag).

### §3.2 No separate InferenceSpec carrier (substrate IS the spec)

Per `feedback_lenses_not_passes` + observation in §2: infer's "rule book" is embedded in the substrate's algebraic structure (`TypeConnective` + `AtomPayload` + `Declaration` variants per `src/v3/std/substrate.dag`). No analogous "InferenceSpec" carrier (cf. ElaborationSpec for lower per Decision 3.C) is needed.

**Why infer differs from lower in this axis**:
- Lower maps SURFACE FORMS (which are user-authored grammar) → substrate behaviors. The mapping is a design choice (multiple valid mappings possible per grammar/elaboration separation per Decision 2.A 4-param compile).
- Infer propagates TYPES through substrate-declared algebraic structure. The propagation rule per TypeConnective variant is determined by the variant's algebraic role (Arrow's direct-signature, Atom's lookup, etc.). The rule is forced by the substrate's structure; no design freedom for re-mapping.

**If a future substrate refactor adds new TypeConnective variants**, the inference rules do NOT extend automatically. Per cursor INLINE BLOCKING #3085 + thesis stop-signal discipline: a 7th TypeConnective variant requires (a) explicit C1 substrate-extension audit + (b) named infer-rule receipt for the new variant's structural inference behavior. The "per-variant structural facts" framing means new variants need new per-variant facts, NOT silent inheritance. Earlier draft "rules extend automatically" weakened the substrate-extension stop signal; corrected here.

---

## §4 Output types (declared in `.dag` substrate)

### §4.1 `InferredDag` (typed-state output carrier per Decision 3.A)

Per Decision 3.A operator-ratified, infer produces `InferredDag` variant explicitly. Construction-time invariant: every `Port.state` is either `Resolved(TypeShape)` or `Unresolved` (with diagnostic in the diagnostic table per the `state == Unresolved iff diagnostics.contains(port_id)` biconditional). `Uninferred` cannot exist in `InferredDag` by construction.

**Substrate authority — DEPENDS on Decision 3.A landing**: extension of `src/v3/std/substrate.dag` `Dag` declaration to sum-variant; `InferredDag` variant constructor enforces port-state invariant. PB-Substrate work.

**Cross-stage role**: `InferredDag` is the input type to emit (per PR #3066 §3.1 / §9 Step 2). The typed-state carrier discipline is the foundation of post-infer-readiness contract per Decision 3.A operator-ratified shape (NOT lens-checked enforcement per existing l1.5-clean-bootstrap-design.md §2.2 + SELF_HOSTING.md §2.1 — those design docs require amendment per PM-routed disposition β; PM authors concurrent amendment PR).

### §4.2 `InferDiagnostic` (substrate extension per Decision 2.B)

**LIVE substrate state at HEAD** (verified via `grep -n "^type Diagnostic" src/v3/std/diagnostics.dag`):
- Line 150: `type Diagnostic { kind: AnyDiagnosticKind, span: SourceSpan, message: String, correction: Correction }` — runtime carrier
- Line 139: `type AnyDiagnosticKind = CompilerKind(CompilerDiagnosticKind) | LensInstanceKind(LensInstanceKindWitness)` — discriminates by KIND-LAYER per Q6.5 anti-bridge (NOT by stage source)
- Line 201: `type EmissionDiagnostic` — SEPARATE carrier per Q6.5 anti-bridge (cited by PR #3066 §4.2 correctly)

**Decision 2.B operator-ratified per-stage `source` axis is a SUBSTRATE EXTENSION** (not live). Per PR #3077 codex BLOCKING audit + analogous fix here. Earlier draft framing as "live cross-stage carrier" was wrong.

Two extension paths apply equally to PB-5 infer (same as PB-4 lower per PR #3077 §12 Q7):

**Option (a) — Carrier field extension**: add `source: DiagnosticSource` to `Diagnostic`. Cross-stage refactor.
**Option (b) — Lane-local sum + mapping**: `InferDiagnostic` sum (variants below) maps into `AnyDiagnosticKind::CompilerKind(CompilerDiagnosticKind)` per the anti-bridge pattern. No change to existing `Diagnostic` carrier shape.

Per PR #3077 §12 Q7 director-recommend: **(b) lane-local sum + mapping** per Q6.5 anti-bridge preservation. Operator/PM ratification needed; same decision applies to all 4 pipeline stages (Parse/Lower/Infer/Emit).

InferDiagnostic variant shape (Step 2 brief authors against full set).

**Typed-carrier discipline** (per openai-pro PR #3077 BLOCKING + INVARIANTS P2/P3): diagnostic boundaries carry typed facts, NOT String. Closed-axis variants (TypeConnective / AlgebraAxis / etc.) must be typed-referenced. Human display details may be String but classification fields must be typed.

```
// Typed reference carriers (same discipline as PB-4 lower's
// SurfaceFormRef/IdentifierRef per PR #3077 commit 9feecacec):
type IdentifierRef
  = SurfaceVarRef { name: NonEmptyStr, span: SourceSpan }
  // Future: TypePathRef, ModulePathRef per Step 2 brief

// Proper closed-axis enum (per codex BLOCKING PR #3085 /api/reviews/12088 —
// earlier draft had `AlgebraAxisRef = AlgebraAxis(NonEmptyStr)` which was a
// String-wrapper masquerading as typed; that contradicts the typed-carrier
// discipline + creates a second authority on algebra identity. Closed-axis
// enum is the structurally honest form):
// 🟡 SCAFFOLD coproduct at PROPOSED stage. Per cursor PR #3085 INLINE
// BLOCKING + modeling-discipline Practice 4: substrate coproducts require
// 🟢/🟡/🔴 classification + named ledger/trigger for dissolution.
//
// **Dissolution trigger**: when Step 2 worker brief enumerates the full
// algebra-axiom set against infer.rs algebra-inhabitance check sites at
// `src/v3/compiler/src/infer.rs` AND verifies coverage parity with the
// 3-variant subset already live at verification.dag:146 AlgebraicLawKind,
// promote to 🟢 TERMINAL at the per-stage algebra-axis scope.
//
// **Adjacent live**: verification.dag:146 AlgebraicLawKind already declares
// 3-variant subset (Associativity / Commutativity / Identity) at the
// algebraic-law-kind scope. AlgebraAxis is a broader closed-axis enumeration
// covering the algebra-inhabitance failure axes specifically (Closure /
// Inverse / Distributivity / OrderingTotality / etc.).
type AlgebraAxis
  = Closure
  | Associativity
  | Commutativity
  | Identity
  | Inverse
  | Distributivity
  | OrderingTotality
  | OrderingTransitivity
  | OrderingAntisymmetry
  // Variants enumerated per Step 2 brief grep of infer.rs algebra-inhabitance
  // check sites; promote to 🟢 TERMINAL once full set covers actual check axes.

// 🟡 SCAFFOLD coproduct at PROPOSED stage. Per cursor PR #3085 INLINE
// BLOCKING + modeling-discipline Practice 4 (Coproduct dissolution):
// substrate coproducts require 🟢/🟡/🔴 classification + dissolution trigger.
//
// **Dissolution trigger**: when Step 2 worker brief enumerates the full
// variant set against `parse_generated.rs` Diagnostic::ParseError, lower.rs
// Diagnostic construction sites, and infer.rs Dag::mark_unresolved emission
// sites — promote to 🟢 TERMINAL at the per-stage diagnostic-variant scope.
// PR #3077 §12 Q7 ratification on Decision 2.B extension path determines
// whether this stays per-stage sum (option b) or extends shared Diagnostic
// (option a).
//
// **Anti-bridge**: per Q6.5 anti-bridge invariant at diagnostics.dag:135-141,
// InferDiagnostic does NOT collapse into CompilerDiagnosticKind without
// substrate-extension ratification; the relationship between this proposed
// per-stage sum + the shared Diagnostic carrier is itself the ratification
// scope of PR #3077 §12 Q7.
type InferDiagnostic
  = UnresolvedIdentifier { identifier: IdentifierRef, scope: SectionRef }
  | NotCallable { type_connective: TypeConnective }
  | ArgumentArityMismatch { expected: Nat, actual: Nat }
  | TypeMismatch { expected: TypeShape, actual: TypeShape }
  | AlgebraInhabitanceFail { connective: TypeConnective, axis: AlgebraAxis }
  | PostSweepUninferred { port_id: PortId, fallback_reason: String }   // per §12 Q3; fallback_reason is human display detail
  | (additional variants per Step 2 worker brief authoring against infer.rs; ALL classification fields are typed closed-axis carriers, not String)
```

**Lane dependency**: PB-Substrate Decision 2.B extension path landing; Director-tier authoring for `InferDiagnostic` variant shape.

### §4.3 `InferResult` — NOT a separate sum-variant

Unlike lower's `LowerResult = Either<PreInferDag, List<LowerDiagnostic>>`, infer's output is plain `InferredDag` — diagnostics are coupled INTO the InferredDag via the `state == Unresolved iff diagnostics.contains(port_id)` biconditional. Per `feedback_state_space_vs_behavioral_invariants`: the structural coupling makes the diagnostic-port relationship a type invariant, not a sum-variant.

`InferredDag` is honest about partial inference success — ports that failed inference are `Unresolved` with explanation in the diagnostic table; ports that succeeded are `Resolved(TypeShape)`. The InferredDag is well-formed even with partial-failure; downstream consumers (emit) handle `Unresolved` ports per their own fail-closed discipline.

---

## §5 Substrate-driven inference (the core)

Infer's structure is NOT "decide what to do" but "look up what the substrate says". Per `feedback_lenses_not_passes`:

### §5.1 TypeConnective-dispatch walker

For each Behavior in PreInferDag, infer:
1. Inspects the TypeConnective of relevant ports/declarations
2. Looks up the structural inference rule per TypeConnective variant
3. Applies the rule mechanically (no decision logic)

Per the live `infer.rs:5-15` rules + `M1_DESIGN.md §4`:
- `Arrow` → input/output signature direct
- `Atom(Identifier)` → resolution-link OR declaration-table lookup
- `Atom(ResolvedByStructure|ResolvedByName)` → follow chain
- Other variants → not callable → mark_unresolved + diagnostic

### §5.2 Forward propagation via fold

Inference walks the DAG topologically (per port dependency order) and propagates types forward. The fold is structural — read upstream port types + apply per-Behavior inference rule + write downstream port types.

Per `feedback_decidability_invariant`: forward propagation is decidable (no fixed-point iteration needed for structural inference; type propagation terminates per port count). Backward inference (constraint solving) is OUT OF SCOPE for R3 infer — forward-only.

### §5.3 Anonymous-instantiation declaration push

Per `infer.rs:18-22` M1 extension: inference mutates `Dag.declarations` by pushing anonymous Instantiation shapes (e.g., `Result<T, DivError>` for totalizing `div`). `find_equivalent_anonymous_instantiation` deduplicates so fixpoint growth stays bounded.

In the `.dag` substrate model, this mutation happens via a `Builder` pattern on the InferredDag carrier — or equivalent typed-state construction. The Builder accumulates anonymous Instantiations during fold; `.finalize()` constructs the `InferredDag` with the canonical declaration set. This preserves `feedback_state_space_vs_behavioral_invariants` (typed-state at completion).

### §5.4 Post-sweep fail-closed

Per `infer.rs:21-25`: after forward propagation, a post-sweep drives any remaining `Uninferred` ports to `Unresolved` with a generic diagnostic. This ensures the post-infer invariant `state != Uninferred for all ports` holds by construction at `InferredDag` finalization.

Per Modeling Practice 2 illegal states unrepresentable: the `InferredDag` constructor REJECTS any port with `Uninferred` state; the post-sweep is the mechanism that makes construction succeed by lifting all remaining Uninferred to Unresolved+diagnostic.

---

## §6 Substrate prereqs (per-Gap-tier anchored)

Per `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`.

**Note on PR citations**: "Status at HEAD" column freezes a snapshot AS OF 2026-05-14; lane-anchor is primary.

| Prereq | Substrate authority | Gap-tier lane | Status at HEAD (as of 2026-05-14) |
|---|---|---|---|
| PB-Substrate Decision 3.A | `Dag = PreInferDag \| InferredDag` sum-variant extension at `src/v3/std/substrate.dag` | Gap 13 R3 Grounding Mgr lane + R3 Substrate Mgr (warm-wolf-698) | DEPENDS on Decision 3.A operator-ratified; cross-stage refactoring |
| TypeConnective + algebraic structure | `src/v3/std/substrate.dag` (live; TypeConnective + AtomPayload + Declaration variants) | PB-Substrate | LIVE at HEAD; inference rules are per-variant structural facts |
| InferDiagnostic substrate extension | `src/v3/std/diagnostics.dag:150` `Diagnostic { kind: AnyDiagnosticKind, ... }` exists (kind-layer discrimination per Q6.5); Decision 2.B per-stage `source` axis is SUBSTRATE EXTENSION (NOT live). Path: (a) extend Diagnostic carrier OR (b) lane-local InferDiagnostic sum + mapping into CompilerKind (Director-recommend (b) per PR #3077 §12 Q7) | PB-Substrate + Director-tier per-stage authoring + operator/PM ratification per PR #3077 §12 Q7 (cross-stage Decision 2.B path) | Live `Diagnostic` + `AnyDiagnosticKind` + `EmissionDiagnostic` (separate per Q6.5); NO per-stage source axis live |
| Algebra inhabitance walk | `src/v3/std/algebra.dag` + per-target inhabitance facts | T-Ground-Coercion-Fold (Gap 13 R3 Grounding Mgr) | In-flight per PR #1980 ScratchIntExamples retirement; broader algebra inhabitance fold pending |
| AmendmentPR for 3.A typed-state enforcement | `docs/l1.5-clean-bootstrap-design.md:86-88` + `src/v3/SELF_HOSTING.md:248-258` flip enforcement model from lens-checked to typed-state | PM-authored amendment PR per disposition β | DEPENDS on operator ratification (3.A = (c) per 2026-05-14 directive); PM authors |

**Critical observation**: PB-5 infer's substrate prereqs are LIGHTER than PB-4 lower's (no ElaborationSpec equivalent — substrate IS the inference rule book per §3.2). The DOMINANT dependencies are: (1) Decision 3.A sum-variant carrier extension landing first via PB-Substrate; (2) PM-authored amendment PR for thesis-doc consistency.

---

## §7 Cross-stage coordination

### §7.1 Upstream dependencies

infer depends on `PreInferDag` from lower (PB-4). The carrier is Decision 3.A sum-variant — PB-Substrate landing IS the load-bearing prereq. PB-4 lower migration is upstream in lane terms but independent in carrier shape (both depend on the same PB-Substrate extension).

Per `src/v3/SELF_HOSTING.md` §2 migration order: infer migrates THIRD (after emit, after lower) per bottom-up. Despite infer being upstream of emit in pipeline execution, emit migrates first because emit's substrate authority (LanguageSpec + per-target specs) is smaller than infer's substrate authority (algebraic structure + TypeConnective dispatch rules).

### §7.2 Downstream consumers

emit (PB-6 — already landed at PR #3066) consumes `InferredDag`. Per PR #3066 §3.1 + Decision 3.A, emit's signature accepts only `InferredDag` by construction.

PB-Runtime test_runner / lens_apply consume InferredDag for lens execution. The typed-state carrier preserves the post-infer-completion contract across runtime consumers.

### §7.3 Sibling-stage coordination

Cross-stage discipline: parse → SurfaceModule → lower → PreInferDag → infer → InferredDag → emit → EmissionResult. The chain is enforced by Decision 3.A sum-variant typed-state at each boundary.

Per Decision 2.B discriminated-union diagnostics: each stage's diagnostics are discriminable by source (Parse / Lower / Infer / Emit) via whichever substrate-extension path PR #3077 §12 Q7 ratifies — (a) carrier-field OR (b) lane-local sum mapping into CompilerKind. Whichever path lands, downstream stages can discriminate by source but don't reprocess prior-stage diagnostics. **PR #3077 §12 Q7 must ratify before any Step 2 worker brief authoring** (cross-stage authority for Decision 2.B extension path).

---

## §8 Two shapes of omni-emission — N/A for infer

infer is target-agnostic structural inference; Shape A/B disambiguation is emit's concern (per PR #3066 §3.3). PB-5 infer has no Shape A/B framing.

---

## §9 SELF_HOSTING.md §2.2 4-step applied to PB-5 infer

| Step | Deliverable | Owner | Substrate |
|---|---|---|---|
| **Step 1: Model review** | THIS DOC | Director (zesty-bear-812) | docs/design-infer-stage-l25-model.md (this doc) |
| **Step 2: Pipeline slot** | `fn infer(d: PreInferDag) -> InferredDag` declared in `src/v3/compiler/pipeline.dag` (per dsl/gunbc/compiler.dag:24 — internal pipeline lives in pipeline.dag, NOT generic compiler.dag) with `ExternalRealization` body (Rust-backed placeholder pointing to current `infer.rs`). Signature uses Decision 3.A sum-variant typed-state at both boundaries (input = PreInferDag, output = InferredDag); Modeling Practice 6 API-level enforcement. Step 2 worker brief authoring routes through Director after Decision 3.A operator ratification + PB-Substrate carrier extension. | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 2 brief | pipeline.dag refinement |
| **Step 3: Implementation** | `src/v3/std/infer.dag` (the .dag implementation; structural TypeConnective dispatch + forward propagation). NO separate "InferenceSpec" file — substrate IS the rule book per §3.2. | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 3 brief | src/v3/std/infer.dag (NEW substrate authority) |
| **Step 4: Parity test + simultaneous Rust deletion** | Parity verification authored as `.dag` TestClaim — generated test fixture set + `.dag` TestClaim asserting `infer_via_rust(pre_infer_dag) == infer_via_dag(pre_infer_dag)` structural-equality across canonical corpus (Dag comparison up to NodeId renaming + diagnostic table equality). **P5 dissolution receipt**: this TestClaim is transient-by-construction; dissolves when infer.rs deletes in same PR. Any hand-Rust scaffolding bears P5 receipt `parity_infer_dag_vs_rust_scaffolding — transient; dissolves with infer.rs deletion in same PR per Step 4 atomic discipline`. `infer.rs` DELETED in same PR. EXPECTED_HAND_AUTHORED_NON_TEST shrinks by N entries at PR-merge. | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 4 brief | tests/parity_infer_dag_vs_rust (TestClaim shape) + `infer.rs` deletion |

**Critical: parity test is against INFER.RS OUTPUT, not against infer.dag-template-of-infer.rs** per `feedback_paper_shrink_variants`. Same discipline as PB-6 emit Step 4 + PB-4 lower Step 4.

---

## §10 Determinism invariant preservation

Per `feedback_no_textual_enforcement_bridges` adjacent (structural enforcement of determinism): `infer.dag` implementation must use structural iteration with deterministic ordering. Current `infer.rs` uses `HashSet` (per the import); the `.dag` migration needs deterministic-set alternative (sorted-set or BTreeSet equivalent).

The post-sweep ordering matters for diagnostic ordering reproducibility. Per Step 3 brief authoring, the `.dag` implementation uses structural iteration over the port set in canonical order.

---

## §11 Inference invariants (cross-cutting)

Per `infer.rs:23-28` + Decision 3.A typed-state enforcement:

- **Post-infer invariant**: `state != Uninferred for all ports` (lifted via post-sweep)
- **Diagnostic coupling**: `state == Unresolved iff diagnostics.contains(port_id)` (biconditional)
- **Anonymous-instantiation deduplication**: `find_equivalent_anonymous_instantiation` ensures fixpoint growth bounded
- **InferredDag constructor**: rejects construction with any `Uninferred` port; the post-sweep is the mechanism that satisfies the constructor invariant

Per Modeling Practice 2 + Practice 6: these invariants are enforced AT THE CARRIER (InferredDag constructor) — runtime fail-close at construction time, not post-hoc lens check. Per Decision 3.A disposition β (RATIFY + amend), the existing lens-checked framing in l1.5-clean-bootstrap-design.md §2.2 + SELF_HOSTING.md §2.1 needs amendment to align with typed-state carrier enforcement.

---

## §12 Open design questions (operator/PM ratification)

These surface to operator/PM (per 2026-05-14 directive) before Step 2 dispatch:

### Q1: Forward-only vs constraint-solving inference

Current `infer.rs` is forward-only (propagation per port dependency order). Constraint-solving (Hindley-Milner-style unification or bidirectional inference) is more powerful but adds complexity.

**Director-recommend: forward-only for R3** per `feedback_decidability_invariant` (decidability via topological propagation is structurally honest; constraint-solving introduces fixpoint convergence concerns). Future constraint-solving extensions are post-R3 scope. Operator/PM ratification.

### Q2: Anonymous-instantiation Builder pattern

`infer.rs` mutates `Dag.declarations` during inference via `push_declaration`. In `.dag` substrate, the typed-state carrier (InferredDag) cannot be mutated — construction is once. Options:

**(a) Builder pattern**: `InferredDag::Builder` accumulates anonymous Instantiations; `.finalize()` constructs.
**(b) Fixed-point fold**: external loop that re-runs inference until no new Instantiations are pushed; each iteration produces a new `InferredDag` (with growing declaration set); fixed-point when stable.

**Director-recommend: (a) Builder pattern** per `feedback_state_space_vs_behavioral_invariants` (constructor enforces invariants; Builder is implementation detail). (b) introduces fixed-point complexity that violates decidability discipline. Operator/PM ratification.

**CODING.md fluent-builder note** (per cursor PR #3085 exploratory observation): the Builder here is a substrate-owned typed-state accumulator with an explicit non-fluent API (e.g., `push_anonymous_instantiation(d, decl)` + `finalize(b) -> InferredDag`), NOT a Rust-side fluent chained builder. The .dag substrate idiom is data + free functions per CODING.md; the "Builder" terminology is conceptual (accumulator pattern), not the Rust BuilderPattern anti-discipline. Step 3 brief authoring against this constraint.

### Q3: InferDiagnostic post-sweep generic diagnostic

Per `infer.rs:21-25`, post-sweep lifts remaining `Uninferred` ports to `Unresolved` with a "generic diagnostic". This generic-ness loses specificity — the user gets "type not inferred" without explanation of why.

**Director-recommend**: extend InferDiagnostic with a `PostSweepUninferred { port_id, fallback_reason: String }` variant. The fallback_reason is best-effort context (e.g., "port not reachable by forward propagation from any Bind"). Better than fully generic.

### Q4: Diagnostic ordering reproducibility

Multi-port inference can emit multiple diagnostics. Order matters for golden-fixture tests + user-facing diagnostic display. Current `infer.rs` uses HashSet (non-deterministic).

**Director-recommend**: structural ordering by `(port_id_canonical, diagnostic_variant_index)`. The `.dag` implementation uses sorted-set; bootstrapped Rust uses BTreeSet equivalent. Step 3 worker brief authors against this discipline.

### Q5: Migration scope — full single-PR OR phased?

`infer.rs` is ~7300 lines (per `wc -l src/v3/compiler/src/infer.rs`; specific count drifts with each main merge — verify at Step 2 brief time rather than treating any specific integer as authoritative). Per `feedback_paper_shrink_variants` discipline, phased migration with per-phase P5 receipts is acceptable. Possible phasing:

- **5a**: TypeConnective dispatch core (Arrow + Atom + lookup walk) → `.dag`; corresponding `infer.rs` block deleted
- **5b**: Algebra inhabitance fold → `.dag`; corresponding block deleted
- **5c**: Anonymous-instantiation Builder → `.dag`; corresponding block deleted
- **5d**: Post-sweep + final infer.rs deletion

**Director-recommend: (b) phased**. Same discipline as PB-4 lower per PR #3077 §12 Q5. Each phase = own PR + own parity test + own P5 receipt.

### Q6: PB-4 lower landing dependency

PB-5 infer's input is `PreInferDag` from lower (PB-4). **Does PB-5 infer migration block on PB-4 lower migration completing?**

**Director-recommend: NO — PB-5 infer migrates independently of PB-4 lower status**, similar to PB-4's stance on PB-3 parse (per PR #3077 §12 Q6). `PreInferDag` carrier is stable regardless of whether lower-emitter is hand-Rust or `.dag`. PB-5 infer migrates when its substrate (Decision 3.A sum-variant landing + algebra-inhabitance fold) is at HEAD.

---

## §13 Non-goals

- **`.dag` implementation of infer** — Step 3 work, separate brief
- **Per-rule tactical decisions** — Step 3 work
- **Test corpus design + parity-test harness implementation** — Step 4 work
- **Bootstrap-runtime-loop concerns** — separate lanes
- **PB-4 lower migration** — separate L2.5 doc + lane (PR #3077)
- **PB-6 emit migration** — already landed at PR #3066
- **Constraint-solving extensions** — post-R3 scope (per Q1)
- **Shape A/B emission** — emit's concern, not infer
- **Backward inference / type-checking** — forward-propagation-only per Q1

---

## §14 Acceptance criteria for this L2.5 model

This doc lands on main when:

1. ✅ All input types declared structurally with `.dag` substrate paths (§3)
2. ✅ All output types declared with typed-state carrier per Decision 3.A (§4)
3. ✅ Substrate-driven inference structure composed without decision logic (§5 — per `feedback_lenses_not_passes`)
4. ✅ All substrate prereqs named with Gap-tier / Mgr-lane anchors (§6)
5. ✅ Cross-stage dependencies explicit (§7)
6. ✅ N/A — Shape A/B framing irrelevant for infer (§8)
7. ✅ SELF_HOSTING.md §2.2 4-step concretely applied (§9)
8. ✅ Determinism preservation discipline (§10)
9. ✅ Inference invariants explicit (§11)
10. ✅ Open design questions enumerated for operator/PM ratification (§12)
11. ⏳ Operator/PM ratification on §12 Q1-Q6

Post-ratification: this doc becomes substrate authority for Step 2 worker brief authoring + §1.8 PB-5 gate row close-criterion predicate.

---

## §15 Authoring sequence post-ratification

1. **Operator / PM-delegate ratifies §12 Q1–Q6** (per 2026-05-14 directive)
2. **PM amends close plan** to route through PB-X lanes + cite this doc as PB-5 L2.5 substrate
3. **PM amends §1.8** with PB-5 gate row citing this doc
4. **PM authors disposition β amendment PR** (concurrent: SELF_HOSTING.md + l1.5-clean-bootstrap-design.md from lens-checked → typed-state enforcement model per Decision 3.A)
5. **PB-Substrate Decision 3.A landing first** — `Dag = PreInferDag | InferredDag` carrier extension lands via warm-wolf-698 dispatch
6. **Director authors PB-5 Step 2 worker brief** (pipeline-slot ExternalRealization PR scope)
7. **R3 Substrate Mgr (warm-wolf-698)** dispatches Step 2 worker against Director-authored brief
8. **Director ratifies Step 2 PR + admin-merges** when CI clears
9. **Director authors PB-5 Step 3 worker brief(s)** (phased per Q5) — for R3 Substrate Mgr
10. **R3 Substrate Mgr** dispatches Step 3 worker(s); substantive review per `feedback_paper_shrink_variants` 5th axis
11. **Director ratifies Step 3 PR(s)**, admin-merges
12. **Director authors PB-5 Step 4 worker brief(s)** (parity + simultaneous deletion per phase)
13. **R3 Substrate Mgr** dispatches Step 4 worker(s); parity against `infer.rs` OUTPUT
14. **Director ratifies Step 4 PR(s)**, admin-merges → `infer.rs` DELETED → PB-5 gate row CLOSES

Subsequent L2.5 models (PB-3 parse / PB-2 tokenize) follow same sequence.

---

## §16 Cross-references

**Primary authority**:
- `src/v3/SELF_HOSTING.md` §2.2 (4-step migration discipline) — pending amendment per Decision 3.A disposition β
- `docs/l1.5-clean-bootstrap-design.md` §2.2 (lens-checked enforcement model — pending amendment per Decision 3.A disposition β)
- `docs/design-pure-bootstrap-zero.md` (PB-X lane framing)
- `docs/substrate-reflection-design.md` §12.6 (migration order)
- `docs/design-emit-stage-l25-model.md` (sibling PB-6 emit L2.5 — set the L2.5 template via PR #3066)
- `docs/design-lower-stage-l25-model.md` (sibling PB-4 lower L2.5 via PR #3077; close template parity)
- `src/v3/M1_DESIGN.md` §4 (TypeConnective dispatch design)

**Live substrate referenced**:
- `src/v3/std/substrate.dag` (Dag, Declaration, Port, TypeConnective, AtomPayload — extends to PreInferDag|InferredDag per Decision 3.A)
- `src/v3/std/diagnostics.dag:150` (`Diagnostic { kind: AnyDiagnosticKind, ... }` carrier — kind-layer discrimination per Q6.5 anti-bridge); `:139` (AnyDiagnosticKind); `:201` (EmissionDiagnostic — separate per Q6.5). InferDiagnostic extension path per PR #3077 §12 Q7 ratification (Director-recommend (b) lane-local sum + mapping into CompilerKind)
- `src/v3/std/algebra.dag` (algebra inhabitance + AsymptoticClass + algebraic structure for inference rule book)
- `src/v3/compiler/src/infer.rs:1-30` (current hand-Rust top-comment with canonical inference rules + invariants)

**Memory disciplines applied**:
- `feedback_lenses_not_passes` (infer is substrate-driven dispatch, NOT decision engine; rule book IS substrate)
- `feedback_fail_closed_discipline` C-8 (Dag::mark_unresolved + post-sweep)
- `feedback_state_space_vs_behavioral_invariants` (typed-state InferredDag constructor)
- `feedback_coproduct_dissolution` Practice 4 (sum-variant Dag per Decision 3.A)
- `feedback_decidability_invariant` (forward propagation, no fixed-point iteration)
- `feedback_target_agnostic_ir` (infer output carries no target-specific facts)
- `feedback_paper_shrink_variants` (Step 4 parity = genuine deletion, not relocation)
- `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id` (Gap-tier anchors)
- `feedback_typed_state_carriers_arent_metadata_markers` (InferredDag is structural typed-state)
- `feedback_grep_carrier_semantic_before_ratification` (4-axis grep applied at authoring time)
- `feedback_discipline_change_audit_all_contract_mentions` (signature consistency across sections)

**Surfaces awaiting**:
- Operator/PM ratification on §12 Q1–Q6
- Decision 3.A sum-variant carrier extension lands via PB-Substrate
- PM-authored amendment PR for thesis docs per disposition β
- PM Phase 2 close plan + §1.8 amendments citing this doc
