# Lower Pipeline Stage — L2.5 Domain Model (PB-4)

**Status:** DRAFT — Director-tier authoring per operator ratification 2026-05-14 ("can we author L2.5 models up front / agree on them") + Decision 1.A scoping = Option A (pipeline-stage L2.5s = Director scope).

**Authoring date:** 2026-05-14.
**Authoring tier:** Director (zesty-bear-812).
**Lane:** PB-4 (lower) per `docs/design-pure-bootstrap-zero.md` + `src/v3/SELF_HOSTING.md` §2 4-step migration discipline.
**Migration order rank:** 2nd (per `docs/substrate-reflection-design.md` §12.6 — emit → lower → infer → parse → tokenize bottom-up; PB-6 emit landed at PR #3066).
**Routing authority chain:** operator-ratification (via PM-delegated decision list 2026-05-14) → PM amends close plan + §1.8 PB-4 gate row → Director authors per-step worker briefs → R3 Substrate Mgr (warm-wolf-698) dispatches workers against Director-authored briefs.

---

## §1 Purpose + scope

This document is the **Step 1 model review** per `src/v3/SELF_HOSTING.md` §2.2 4-step discipline applied to PB-4 lower-stage migration. It declares the lower stage's input/output types in `.dag` substrate, the structural elaboration rules that compose them, the substrate prereqs the stage requires from sibling lanes, and the open design questions requiring operator ratification before Step 2 (pipeline-slot declaration) dispatches.

**This doc does NOT**:
- Author the `.dag` implementation (Step 3 work)
- Author the pipeline-slot declaration (Step 2 work)
- Design the parity test corpus (Step 4 work)
- Own implementation of bootstrap-runtime-loop or PB-Substrate / PB-Bootstrap-Process / PB-Runtime (those are separate lanes; this doc references them as dependencies / coordination points only)

**Authority chain**: Director-tier ratification grounds the model; subsequent worker briefs (Steps 2–4) cite this doc as the substrate; §1.8 PB-4 gate row close-criterion predicate cites this doc as the L2.5 model authority.

---

## §2 What lower IS structurally

Per `docs/design-emission-model.md` lens framing + `THESIS.md` §"Tier 1" (substrate ownership of meaning) + the live top-comment at `src/v3/compiler/src/lower.rs:1-23`:

**Lower is a 2-pass structural elaboration from (SurfaceModule, ElaborationSpec) → PreInferDag — first pass allocates Declaration placeholders + builds symbol table; second pass fills connectives + lowers function/let bodies to L1 behaviors. NOT a type-checking stage (infer's responsibility) and NOT a decision engine.**

Per `feedback_lenses_not_passes`: lower is a structural mapping from surface forms to substrate behaviors, not a decision process. Anything lower has to "decide" is an ElaborationSpec rule that should be declared in `.dag`, not encoded in lowering logic.

Per Decision 2.A operator-ratified shape (4-param `compile`): lower's signature accepts ElaborationSpec as a declared authority input rather than implicit compiler behavior. This is the substantive thesis demonstration — every previously-implicit compiler axis becomes substrate fact.

**Failure shape**: fail-closed (per `feedback_fail_closed_discipline` + INVARIANTS C-8). Unresolved identifiers produce placeholder ports + ResolveError diagnostics; unsupported surface forms fail-closed with typed `LowerDiagnostic`, never silent or fabricated.

**Typed-state carrier output**: per Decision 3.A operator-ratified shape (sum-variant `Dag = PreInferDag | InferredDag` per `feedback_coproduct_dissolution` Practice 4), lower produces `PreInferDag` variant explicitly — infer's job is to transform `PreInferDag` into `InferredDag` by resolving every port.

---

## §3 Input types (declared in `.dag` substrate)

Two input types feed lower (`SurfaceModule` from parse + `ElaborationSpec` per Decision 3.C):

### §3.1 `SurfaceModule` (post-parse)

The raw surface-form representation produced by parse stage. Tree of `SurfaceItem` (top-level declarations: fn / let / type / data) + `SurfaceExpr` (expression forms: literals / identifiers / calls / operators / if / match / block) + `SurfacePattern` (pattern forms: literal / binding / variant / record / wildcard).

**Substrate authority — LIVE V3 AUTHORITY**: `src/v3/std/parse_surface.dag:29` declares `SurfaceModule`; `:149` declares `SurfaceExpr` (closed-axis sum); `:123` declares `SurfacePattern` (closed-axis sum); `:257` declares `SurfaceItem` (closed-axis sum). Verified via `grep -n "^type Surface" src/v3/std/parse_surface.dag`.

**Lane dependency**: PB-3 parse (generates parse output consistent with this carrier).

### §3.2 `ElaborationSpec` (surface-form → substrate-behavior rules per Decision 3.C)

Per Decision 3.C operator-ratified: `.dag` rules consumed by lower, NOT Rust code reading `.dag`. ElaborationSpec is the declared authority that maps surface forms to substrate behaviors. Each rule is a structural fact mapping a `SurfaceExpr` / `SurfaceItem` / `SurfacePattern` variant to a `Behavior` construction recipe.

Per the live top-comment at `lower.rs:8-21`, the canonical lowering rules currently in hand-Rust:
- `SurfaceLiteral::{Int, Bool, String}` → `Value(LiteralBits::*)`
- `Var (local)` → scope lookup via symbol-table
- `Var (unresolved)` → placeholder port + ResolveError diagnostic
- `Call` → `Transform { target: TransformTarget::Callable(DeclarationId), inputs }`
- `Operator` → `Transform { target: TransformTarget::Operator(OperatorKind), inputs }`
- `If` → `Branch` with 2 Paths
- `Fn item` → `Bind` with non-empty params + optional `Loop` wrapper
- `Let item` → `Bind` with empty params
- `Match` → `Branch` with N Paths per arm (existing handling per `lower.rs` body)
- `Block` → composed `Bind` chain via sequencing
- (additional rules per `lower.rs` — Step 2 brief authors against full rule set; this enumeration is the seed from the top-comment)

**Substrate authority**: `src/v3/std/elaboration_spec.dag` is a NEW substrate file PB-4 must author. Verified at this doc's HEAD via `grep -rn "type ElaborationSpec" src/v3/std/ dsl/std/` (no existing declaration). Per Decision 3.C operator-ratified shape, `ElaborationSpec` carries the rules as declared substrate, not as Rust closures.

**Lane dependency**: PB-Substrate (generates parse_surface.dag mirror + behavior carriers from substrate.dag); Director-tier substrate-fact-introduction for `ElaborationSpec` carrier itself.

### §3.3 Target identity NOT an input to lower

Lower is target-agnostic. Per `feedback_target_agnostic_ir`, the output `PreInferDag` carries no target-specific facts; target selection happens at emit time via LanguageSpec choice. Lower's job is the structural mapping; emit's job is the per-target inhabitance lookup.

---

## §4 Output types (declared in `.dag` substrate)

One primary output (typed-state `PreInferDag` carrier) + a diagnostic stream (closed-axis sum-variant for fail-closed surfaces).

### §4.1 `PreInferDag` (typed-state output carrier per Decision 3.A)

Per Decision 3.A operator-ratified (sum-variant `Dag = PreInferDag | InferredDag` per `feedback_coproduct_dissolution` Practice 4), lower produces `PreInferDag` variant explicitly. Construction-time invariant: `PreInferDag` carries declarations + L1 behaviors + ports where every `Port.state` is either `Uninferred` (pre-infer transient — admitted in `PreInferDag` by construction) or `Unresolved` (lower could not resolve identifier, diagnostic emitted in parallel stream).

**The variant boundary IS the structural invariant**: pre-infer state cannot exist outside `PreInferDag` (Modeling Practice 2 illegal states unrepresentable). emit cannot accept `PreInferDag` (its signature requires `InferredDag` per PR #3066 §3.1 / §9 Step 2).

**Substrate authority**: extension of `src/v3/std/substrate.dag` `Dag` declaration to a sum-variant per Decision 3.A. The current `type Dag` single-variant carrier becomes `type Dag = PreInferDag | InferredDag` — refactoring routes via PB-Substrate (cross-stage carrier change affects parse / lower / infer / emit signatures + every Dag-consumer).

### §4.2 `LowerDiagnostic` (substrate extension per Decision 2.B)

**LIVE substrate state at HEAD** (verified via `grep -n "^type Diagnostic" src/v3/std/diagnostics.dag`):
- Line 150: `type Diagnostic { kind: AnyDiagnosticKind, span: SourceSpan, message: String, correction: Correction }` — runtime diagnostic carrier
- Line 139: `type AnyDiagnosticKind = CompilerKind(CompilerDiagnosticKind) | LensInstanceKind(LensInstanceKindWitness)` — discriminates by KIND-LAYER (compiler-primitive vs lens-instance), NOT by stage source
- Line 201: `type EmissionDiagnostic = | UnderRefined | NoInhabitant | ...` — SEPARATE carrier for emission-stage fold failures (cited by PR #3066 §4.2)

**Decision 2.B operator-ratified shape requires SUBSTRATE EXTENSION** — the per-stage `source: DiagnosticSource` axis is NOT currently a live substrate fact. The existing discrimination axis is kind-layer (CompilerKind vs LensInstanceKind), per Q6.5 anti-bridge invariant. Decision 2.B adds an orthogonal source axis (Parse / Lower / Infer / Emit) that requires extending the live carrier:

```
// Per Decision 2.B substrate extension (NEW; NOT currently live):
type DiagnosticSource = Parse | Lower | Infer | Emit
type Diagnostic {
  kind: AnyDiagnosticKind     // existing — kind-layer discrimination
  source: DiagnosticSource    // NEW — per Decision 2.B per-stage axis
  span: SourceSpan
  message: String
  correction: Correction
}

// Per-stage shape variants attach via CompilerDiagnosticKind extension OR
// separate lane-local sums per Step 2 worker brief authoring decision:
type LowerDiagnostic
  = ResolveError { identifier: String, scope_chain: List<DeclarationId> }
  | UnsupportedSurfaceForm { form: String, reason: String }
  | DuplicateDeclaration { name: String, prior_span: SourceSpan }
  | DuplicateRecordFieldLabel { label: String, prior_span: SourceSpan }  // per PR #3075 ratchet
  | (additional variants per Step 2 worker brief authoring against lower.rs)
```

**Substrate authority — SUBSTRATE EXTENSION REQUIRED**: PB-4 lower's diagnostic substrate requires either (i) extending live `Diagnostic` carrier at `src/v3/std/diagnostics.dag:150` with `source: DiagnosticSource` field per Decision 2.B, or (ii) lane-local `LowerDiagnostic` sum that maps into existing `AnyDiagnosticKind::CompilerKind` (downstream consumer concern per the line 163-168 anti-bridge note). The current `EmissionDiagnostic` at line 201 is a SEPARATE carrier (per Q6.5 anti-bridge); PB-4 does NOT extend that.

Operator/PM ratification needed on extension shape at §12 Q-new (added per codex BLOCKING #3077): which extension path for Decision 2.B per-stage source axis — carrier-field-extension or lane-local-sum-mapping.

**Lane dependency**: PB-Substrate (Decision 2.B substrate extension authoring); Director-tier per-stage diagnostic variant authoring.

### §4.3 `LowerResult` (disjoint sum)

```
type LowerResult = Either<PreInferDag, List<LowerDiagnostic>>
```

Per `feedback_fail_closed_discipline` C-8 + `feedback_state_space_vs_behavioral_invariants`: the sum variant makes illegal states unrepresentable (partial-lower-with-no-diagnostic OR diagnostic-without-failure are both unrepresentable by construction). lower returns `LowerResult`, never raw `PreInferDag` without typed-state context if errors occurred.

Note: parallel to PB-6 emit's `EmissionResult = Either<TargetSource, EmissionDiagnostic>` — consistent typed-state output discipline across pipeline stages.

---

## §5 Structural elaboration (the core)

lower composes from these substrate facts via 2-pass walk:

### §5.1 Pass 1 — Declaration allocation + symbol table

Per `lower.rs:3-7` top-comment: "Pass 1 walks all top-level items and allocates placeholder Declarations for each named type/fn, populating a symbol table (name → DeclarationId)."

Pass 1 is purely structural: SurfaceItem variants map 1:1 to Declaration placeholders. No resolution happens; identifiers are not yet bound. Output: symbol table (name → DeclarationId) + `PreInferDag` with placeholder declarations.

Per Modeling Practice 4 (Coproduct dissolution): each `SurfaceItem` variant has exactly one Declaration shape it maps to (per `lower.rs` top-comment + the 4-pattern dissolution receipt at the Declaration sum-variant level). The mapping is mechanical.

### §5.2 Pass 2 — Connective + behavior body lowering

Per `lower.rs:5-7` top-comment: "Pass 2 fills in each declaration's connective and lowers function/let bodies to L1 behaviors, using the symbol table to resolve identifier references."

Pass 2 walks the body of each declaration + applies the `ElaborationSpec` rules (per §3.2). Each `SurfaceExpr` / `SurfacePattern` variant has exactly one Behavior mapping; if elaboration requires "deciding" between multiple behavior shapes, that's a missing `ElaborationSpec` fact (per `feedback_lenses_not_passes`).

The 2-pass split is structurally necessary because pass 2 needs the full symbol table from pass 1 to resolve identifier references; mutual recursion + forward references require the placeholder/fill structure. This is NOT a passes-over-IR pattern (per `feedback_lenses_not_passes`); it's a 2-step structural construction.

### §5.3 Mechanical walker dispatch via ElaborationSpec

Walker dispatch is **mechanical**: match on SurfaceExpr/SurfaceItem/SurfacePattern variant, look up the ElaborationSpec rule, apply structurally. **No conditional logic encoded in lower body** — per `feedback_no_textual_enforcement_bridges`: every "decision" is an ElaborationSpec fact.

When an identifier cannot be resolved (Var refers to a name not in symbol table): emit `ResolveError` diagnostic + create placeholder port with state `PortState::Unresolved`. The Unresolved state propagates structurally; downstream stages (infer / emit) handle the unresolved port per their own fail-closed discipline.

---

## §6 Substrate prereqs (per-Gap-tier anchored)

Per `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`: anchor prereqs on Gap-tier identifiers, not session IDs.

**Note on PR citations in "Status at HEAD" column**: PR numbers below freeze a snapshot AS OF AUTHORING DATE 2026-05-14 and will rot. Operators reviewing post-2026-05-14 should anchor on Gap-tier lane column.

| Prereq | Substrate authority | Gap-tier lane | Status at HEAD (as of 2026-05-14) |
|---|---|---|---|
| PB-Substrate | `src/v3/std/substrate.dag` extension for `Dag = PreInferDag \| InferredDag` sum-variant per Decision 3.A | Gap 13 R3 Grounding Mgr lane + R3 Substrate Mgr (warm-wolf-698) | DEPENDS on Decision 3.A operator-ratified shape; refactoring cross-stage |
| PB-3 Parse | `src/v3/std/parse_surface.dag` (live; `SurfaceModule` / `SurfaceItem` / `SurfaceExpr` / `SurfacePattern`) | PB-3 lane (R3 Substrate Mgr post-PB-4) | LIVE at HEAD per `grep -n "^type Surface" src/v3/std/parse_surface.dag` |
| ElaborationSpec carrier | `src/v3/std/elaboration_spec.dag` (NEW substrate to author per Decision 3.C) | Director-tier substrate-fact-introduction | NEEDS AUTHORING; Step 2 brief scope |
| LowerDiagnostic substrate extension | `src/v3/std/diagnostics.dag:150` `Diagnostic` carrier exists with `kind: AnyDiagnosticKind` (CompilerKind|LensInstanceKind discrimination per Q6.5 anti-bridge); Decision 2.B per-stage `source` axis is a SUBSTRATE EXTENSION (NOT currently live). Path: either (i) extend `Diagnostic` with `source: DiagnosticSource` field, or (ii) lane-local LowerDiagnostic sum mapping into CompilerKind | PB-Substrate + Director-tier per-stage diagnostic authoring + operator/PM ratification on extension path per §12 Q-new | Live `Diagnostic` + `AnyDiagnosticKind` + `EmissionDiagnostic` (line 201, separate per Q6.5) NO per-stage source axis live |
| Symbol-table substrate | `src/v3/std/symbol_table.dag` (proposed; verify existence at Step 2) | Director-tier substrate-fact-introduction | NEEDS GREP VERIFICATION at Step 2 authoring; current `lower.rs` uses `HashMap<String, DeclarationId>` (hand-Rust; not `.dag` substrate) |

**Critical observation**: PB-4 lower's substrate prereqs are LIGHTER than PB-6 emit's. PB-4 mostly extends existing live carriers (substrate.dag / diagnostics.dag) + authors 1 NEW carrier (ElaborationSpec) + verifies symbol-table substrate. PB-6 emit had 8 prereqs routing through R3 Grounding Mgr; PB-4 lower has 5 with most LIVE or DEPENDS-on-Decision-3.A.

The DOMINANT dependency is **Decision 3.A operator-ratified sum-variant shape** — the `Dag = PreInferDag | InferredDag` carrier change ripples across all 4 pipeline stages. PB-4 lower's signature + output type depend on this landing first via PB-Substrate.

---

## §7 Cross-stage coordination

### §7.1 Upstream dependencies

lower depends on `SurfaceModule` output from parse (PB-3) + ElaborationSpec rules from substrate. Per `src/v3/SELF_HOSTING.md` §2 migration order, lower migrates SECOND (after PB-6 emit) despite consuming parse's output, because the bottom-up principle is about which substrate authority needs to exist. PB-4 lower's substrate authority (ElaborationSpec + the elaboration rules) is smaller surface than parse's (full grammar + tokenization).

### §7.2 Downstream consumers

PB-5 infer consumes `PreInferDag` output from lower; infer's signature is `fn infer(d: PreInferDag, ...) -> InferredDag` per Decision 3.A. emit consumes `InferredDag` (post-infer); per PR #3066 §3.1 + Decision 3.A, emit cannot accept `PreInferDag` by construction.

The cross-stage chain is: parse → SurfaceModule → lower → PreInferDag → infer → InferredDag → emit → EmissionResult.

Per Decision 2.B discriminated-union diagnostics, each stage emits diagnostics tagged with its `DiagnosticSource`; downstream stages can read prior-stage diagnostics for context but do not need to re-process them.

### §7.3 Sibling-stage coordination

When PB-3 parse / PB-5 infer / PB-6 emit subsequently migrate (or have migrated), their `.dag` implementations must produce typed-state outputs consistent with downstream stage expectations. The typed-state carrier (`PreInferDag` / `InferredDag` per Decision 3.A) enforces this at the type system; mis-shaped output fails type-check at compose time.

Per `feedback_target_agnostic_ir`: lower's `PreInferDag` output carries NO target-specific facts. Target identity enters the pipeline at emit stage via `LanguageSpec` choice (per PR #3066 §3.2).

---

## §8 Two shapes of omni-emission — N/A for lower

lower is structural-only; it produces target-agnostic `PreInferDag`. Shape A vs Shape B disambiguation lives at emit stage (per PR #3066 §3.3 / §8), not at lower. PB-4 lower has no Shape A/B concerns.

---

## §9 SELF_HOSTING.md §2.2 4-step applied to PB-4 lower

| Step | Deliverable | Owner | Substrate |
|---|---|---|---|
| **Step 1: Model review** | THIS DOC | Director (zesty-bear-812) | docs/design-lower-stage-l25-model.md (this doc) |
| **Step 2: Pipeline slot** | `fn lower(surface: SurfaceModule, elaboration: ElaborationSpec) -> LowerResult` declared in compiler.dag with `ExternalRealization` body (Rust-backed placeholder pointing to current `lower.rs`). The signature uses `LowerResult = Either<PreInferDag, List<LowerDiagnostic>>` sum-variant per Modeling Practice 6 API-level enforcement + Practice 4 Coproduct dissolution. Step 2 worker brief authoring routes through Director after Decision 3.A operator ratification + PB-Substrate carrier extension. | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 2 brief | compiler.dag refinement |
| **Step 3: Implementation** | `src/v3/std/lower.dag` (the .dag implementation of lower; fill the function body using ElaborationSpec rules) + `src/v3/std/elaboration_spec.dag` (NEW substrate carrier for the rules) | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 3 brief | src/v3/std/lower.dag + src/v3/std/elaboration_spec.dag (NEW substrate authorities) |
| **Step 4: Parity test + simultaneous Rust deletion** | Parity verification authored as `.dag` TestClaim — generated test fixture set + `.dag` TestClaim asserting `lower_via_rust(surface) == lower_via_dag(surface, elaboration_spec)` structural-equality across canonical corpus (Dag comparison up to NodeId renaming). **P5 dissolution receipt**: this TestClaim is transient-by-construction; dissolves when lower.rs deletes in same PR. Any hand-Rust scaffolding for stage0 lower invocation routing bears P5 receipt `parity_lower_dag_vs_rust_scaffolding — transient; dissolves with lower.rs deletion in same PR per Step 4 atomic discipline`. `lower.rs` + (any per-pass sibling files like `lower_pass1.rs` if extant) DELETED in same PR. EXPECTED_HAND_AUTHORED_NON_TEST shrinks by N entries at PR-merge. | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 4 brief | tests/parity_lower_dag_vs_rust (TestClaim shape, not hand-Rust .rs file) + `lower.rs` deletion |

**Critical: parity test is against LOWER.RS OUTPUT, not against lower.dag-template-of-lower.rs** (the discriminator that fails for cycle-4 PR #3048 / cycle-5 PR #3057 template-relocation paper-shrink class per `feedback_paper_shrink_variants`).

---

## §10 Determinism invariant preservation

Per `feedback_no_textual_enforcement_bridges` adjacent (structural enforcement of determinism): `lower.dag` implementation must use structural iteration with sorted keys; no HashMap iteration without canonical ordering; no non-deterministic primitives. The 2-pass walker iterates SurfaceItem list in source order + builds symbol table by insertion order; Pass 2 walks Declarations in same order.

The current `lower.rs` uses `HashMap<String, DeclarationId>` for the symbol table. Per Step 3 brief authoring, the `.dag` implementation must use a deterministic alternative (sorted-keyed map or association list with insertion-order discipline).

---

## §11 Cross-cutting consistency (NOT cost lens — that's emit's concern)

Lower has no cost-lens responsibilities; cost emerges at emit stage when per-target realization costs combine with per-op algebra costs. Lower's only cross-cutting consistency concern is the Dag construction invariant: every Port has either a producer Behavior (Resolved at construction OR Unresolved+diagnostic) OR is a placeholder allocated by Pass 1 awaiting Pass 2 fill. No port can exist without provenance.

This invariant is enforceable structurally at construction time via the `PreInferDag` carrier's constructor + port allocation discipline. Per Modeling Practice 2 illegal states unrepresentable.

---

## §12 Open design questions (operator ratification)

These surface to operator (or PM-delegate per 2026-05-14 operator directive) before Step 2 (pipeline-slot) dispatch:

### Q1: ElaborationSpec rule shape

Per Decision 3.C operator-ratified (`.dag` rules consumed by lower), the rules ARE `.dag` substrate. Two possible rule-shape conventions:

**Option (a) — Closed-axis per-variant rule sum**: one `ElaborationRule` variant per `SurfaceExpr` / `SurfaceItem` / `SurfacePattern` variant, each carrying the construction recipe as a typed function.
```
type ElaborationRule
  = LiteralRule { surface: SurfaceLiteral, behavior: ValueBehavior }
  | VarRule { ... }
  | CallRule { ... }
  | ...
```

**Option (b) — Pattern-table indexed**: ElaborationSpec is a typed map from `(SurfaceVariantTag) → ElaborationRule`. Less explicit per-variant but easier to compose across L1 surface extensions.

**Director-recommend: (a) closed-axis sum** per `feedback_coproduct_dissolution` Practice 4. Per-variant explicit-ness is the canonical thesis demonstration. Operator/PM ratification before Step 2 dispatch.

### Q2: Symbol-table substrate carrier

Current `lower.rs` uses `HashMap<String, DeclarationId>`. Per Step 3 `.dag` migration, symbol table needs `.dag` substrate carrier. Verified via `grep -rn "type SymbolTable\b" src/v3/std/ dsl/std/` — no existing carrier.

**Director-recommend**: NEW `src/v3/std/symbol_table.dag` carrier with sorted-keyed deterministic structure (association list with insertion-order + lookup-via-fold OR refined-Map-via-where-clause if refinement substrate at HEAD). Specific carrier shape gated on Decision 3.A precedent (newtype / refinement / sum-variant) for similar typed-state contexts.

### Q3: 2-pass split — keep or single-pass?

Current `lower.rs` is 2-pass (Pass 1 allocates + symbol-table; Pass 2 fills). Could a single-pass fixed-point approach achieve same result? The 2-pass split is structurally necessary for forward references + mutual recursion in the symbol table; single-pass would require either fixed-point iteration or out-of-order traversal with deferred resolution.

**Director-recommend: keep 2-pass** — the split IS the structural justification for the symbol table; merging would require fixed-point iteration which violates `feedback_decidability_invariant` (all `.dag` code must be decidable; fixed-point convergence is harder to prove decidable). 2-pass is the structural-honest pattern.

### Q4: PreInferDag construction-time invariants — runtime gate vs constructor

Per Decision 3.A operator-ratified shape (sum-variant), `PreInferDag` is a variant of `Dag`. Construction-time invariants:
- Every Port has provenance (producer Behavior OR placeholder OR Unresolved+diagnostic)
- Every named Declaration has a placeholder allocated in Pass 1

Two paths for invariant enforcement:
- **(a) Constructor-only**: `PreInferDag::construct(declarations, ports) -> Result<PreInferDag, ConstructionError>` — type-enforced
- **(b) Builder + verify**: `PreInferDag::Builder` accumulates; `.finalize()` validates invariants

**Director-recommend: (a) constructor-only** per `feedback_state_space_vs_behavioral_invariants` (type enforcement > API enforcement). Builder pattern admits intermediate illegal states; constructor enforces at the boundary.

### Q5: Migration scope — full lower.rs OR phased?

`lower.rs` is 11895 lines (per `wc -l`). PB-6 emit was substantially smaller. Migration scope question:

**Option (a) — Full single-PR migration**: Step 3 emits complete `lower.dag` + `elaboration_spec.dag`; Step 4 deletes `lower.rs` atomically. High-risk single-PR shape.

**Option (b) — Phased migration**: per `feedback_paper_shrink_variants` discipline (NO template-relocation / NO module-relocation), phased means breaking lower.rs into multiple semantically-meaningful sub-deletions, each its own PR. e.g.:
- Phase 4a: Pass 1 (declaration allocation) → `.dag`; corresponding `lower.rs` block deleted
- Phase 4b: Pass 2 expression lowering → `.dag`; corresponding block deleted
- Phase 4c: Pass 2 pattern lowering → `.dag`; corresponding block deleted
- Phase 4d: Pass 2 item lowering → `.dag`; final `lower.rs` deletion

**Director-recommend: (b) phased** for risk management on 11895-line migration. Each phase is its own PR with its own parity test + dissolution receipt. Operator/PM ratification on phasing acceptable per `feedback_paper_shrink_variants` (genuine deletion, not relocation).

### Q6: PB-3 parse landing dependency

PB-4 lower's input is `SurfaceModule` from parse (PB-3). PB-3 parse migration is downstream in the bottom-up order. **Does PB-4 lower migration block on PB-3 parse migration completing?**

**Director-recommend: NO — PB-4 lower migrates independently of PB-3 parse status**. lower consumes `SurfaceModule` which is already LIVE substrate at `src/v3/std/parse_surface.dag`; the carrier shape is stable regardless of whether parse-emitter is hand-Rust or `.dag`. Per the SELF_HOSTING.md bottom-up principle, lower migrates when its substrate (input + output + elaboration rules) is ratified; parse migration is independent.

---

## §13 Non-goals (out of scope for this L2.5)

- **`.dag` implementation of lower elaboration** — Step 3 work, separate brief
- **Per-rule tactical decisions** — Step 3 work
- **Test corpus design + parity-test harness implementation** — Step 4 work
- **Bootstrap-runtime-loop concerns** — PB-Substrate / PB-Bootstrap-Process / PB-Runtime / PB-Lib+PB-Build separate lanes
- **PB-3 parse migration** — separate L2.5 doc + lane
- **PB-5 infer migration** — separate L2.5 doc + lane
- **PB-6 emit migration** — already landed at PR #3066
- **Shape A/B emission** — emit's concern, not lower

---

## §14 Acceptance criteria for this L2.5 model

This doc lands on main when:

1. ✅ All input types declared structurally with `.dag` substrate paths (§3)
2. ✅ All output types declared structurally with closed-axis sum-variant discipline + typed-state carrier per Decision 3.A (§4)
3. ✅ Structural elaboration composed without decision logic (§5 — per `feedback_lenses_not_passes`)
4. ✅ All substrate prereqs named with Gap-tier / Mgr-lane anchors (§6)
5. ✅ Cross-stage dependencies explicit (§7)
6. ✅ N/A — Shape A/B framing irrelevant for lower (§8)
7. ✅ SELF_HOSTING.md §2.2 4-step concretely applied (§9)
8. ✅ Determinism preservation discipline (§10)
9. ✅ Construction-time invariants identified (§11)
10. ✅ Open design questions enumerated for operator/PM-delegate ratification (§12)
11. ⏳ Operator/PM ratification on §12 Q1-Q6

Post-ratification: this doc becomes the substrate authority for Step 2 worker brief authoring + §1.8 PB-4 gate row close-criterion predicate.

---

## §15 Authoring sequence post-ratification

1. **Operator / PM-delegate ratifies §12 Q1–Q6** (per operator 2026-05-14 directive "have pm sign off on everything")
2. **PM amends close plan** to route through PB-X lanes + cite this doc as PB-4 L2.5 substrate
3. **PM amends §1.8** with PB-4 gate row citing this doc as close-criterion authority
4. **PB-Substrate Decision 3.A landing first** — `Dag = PreInferDag | InferredDag` carrier extension lands via warm-wolf-698 dispatch (cross-stage dependency)
5. **Director authors PB-4 Step 2 worker brief** (pipeline-slot ExternalRealization PR scope) — for R3 Substrate Mgr (warm-wolf-698)
6. **R3 Substrate Mgr (warm-wolf-698)** dispatches Step 2 worker against Director-authored brief
7. **Director ratifies Step 2 PR substance + admin-merges** when CI clears
8. **Director authors PB-4 Step 3 worker brief** (`.dag` implementation scope — phased per Q5) — for R3 Substrate Mgr
9. **R3 Substrate Mgr** dispatches Step 3 worker; iterate substantive review per `feedback_paper_shrink_variants` 5th axis (authority-direction)
10. **Director ratifies Step 3 PR(s)**, admin-merges (phased = multiple PRs per Q5)
11. **Director authors PB-4 Step 4 worker brief(s)** (parity tests + simultaneous Rust deletion per phase)
12. **R3 Substrate Mgr** dispatches Step 4 worker(s); parity must be against `lower.rs` OUTPUT (not against template per `feedback_paper_shrink_variants`)
13. **Director ratifies Step 4 PR(s)**, admin-merges → `lower.rs` DELETED → PB-4 gate row CLOSES per §1.8

Subsequent L2.5 models (PB-5 infer / PB-3 parse / PB-2 tokenize) follow same Director-tier authoring → operator/PM ratification → worker brief dispatch sequence per migration order. Director-tier authoring continues per Option A scoping ratification.

---

## §16 Cross-references

**Primary authority**:
- `src/v3/SELF_HOSTING.md` §2.2 (4-step migration discipline)
- `docs/design-pure-bootstrap-zero.md` (PB-X lane framing)
- `docs/substrate-reflection-design.md` §12.6 (migration order)
- `docs/design-emission-model.md` (sibling lens framing for emit; PB-4 lower applies analogous structural elaboration)
- `docs/design-emit-stage-l25-model.md` (sibling PB-6 emit L2.5 — sets the L2.5 template; this doc mirrors structure)

**Live substrate referenced**:
- `src/v3/std/parse_surface.dag:29` (SurfaceModule), `:149` (SurfaceExpr), `:123` (SurfacePattern), `:257` (SurfaceItem)
- `src/v3/std/substrate.dag` (Dag, Declaration, Port — extends to PreInferDag|InferredDag per Decision 3.A)
- `src/v3/std/diagnostics.dag:150` (Diagnostic carrier — extends with LowerDiagnostic variants per Decision 2.B)
- `src/v3/compiler/src/lower.rs:1-23` (current hand-Rust top-comment with canonical lowering rules)

**Memory disciplines applied**:
- `feedback_lenses_not_passes` (lower = structural elaboration, not decision engine)
- `feedback_fail_closed_discipline` C-8 (LowerResult sum-variant)
- `feedback_state_space_vs_behavioral_invariants` (typed-state PreInferDag at output)
- `feedback_coproduct_dissolution` Practice 4 (sum-variant Dag per Decision 3.A)
- `feedback_no_textual_enforcement_bridges` (no decision-logic in lower; ElaborationSpec facts)
- `feedback_target_agnostic_ir` (lower output carries no target-specific facts)
- `feedback_paper_shrink_variants` (Step 4 parity = genuine deletion, not relocation)
- `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id` (Gap-tier anchors)
- `feedback_typed_state_carriers_arent_metadata_markers` (PreInferDag is structural typed-state, not metadata marker)
- `feedback_grep_carrier_semantic_before_ratification` (4-axis grep applied at authoring time)
- `feedback_discipline_change_audit_all_contract_mentions` (signature consistency across §2 / §3 / §7 / §9)

**Surfaces awaiting**:
- Operator/PM ratification on §12 Q1–Q6 (per operator 2026-05-14 directive)
- Decision 3.A operator-ratified shape lands via PB-Substrate first (sum-variant Dag extension)
- PM Phase 2 close plan + §1.8 amendments citing this doc
- R3 Grounding Mgr respawn (per Decision 5.A) if any substrate prereqs route through that lane
