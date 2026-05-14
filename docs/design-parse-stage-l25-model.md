# Parse Pipeline Stage — L2.5 Domain Model (PB-3)

**Status:** DRAFT — Director-tier authoring per operator ratification 2026-05-14 (Decision 1.A scoping = Option A; Decision 3.B operator-overrode my rec to (b) compile-time parser tables — the "harder/more correct" path).

**Authoring date:** 2026-05-14.
**Authoring tier:** Director (zesty-bear-812).
**Lane:** PB-3 (parse) per `docs/design-pure-bootstrap-zero.md` + `src/v3/SELF_HOSTING.md` §2 4-step migration discipline.
**Migration order rank:** 4th (per `docs/substrate-reflection-design.md` §12.6 — emit → lower → infer → parse → tokenize bottom-up; PB-6 emit landed PR #3066, PB-4 lower at PR #3077, PB-5 infer at PR #3085).
**Routing authority chain:** operator-ratification + PM-delegate (per 2026-05-14 directive) → PM amends close plan + §1.8 PB-3 gate row → Director authors per-step worker briefs → R3 Substrate Mgr (warm-wolf-698) dispatches workers.

---

## §1 Purpose + scope

This document is the **Step 1 model review** per `src/v3/SELF_HOSTING.md` §2.2 4-step discipline applied to PB-3 parse-stage migration. It declares the parse stage's input/output types in `.dag` substrate, the grammar-table-driven structural parsing approach (per Decision 3.B (b) compile-time tables), the substrate prereqs the stage requires, and the open design questions requiring operator/PM ratification before Step 2 (pipeline-slot declaration) dispatches.

**This doc does NOT**:
- Author the `.dag` implementation (Step 3 work)
- Author the pipeline-slot declaration (Step 2 work)
- Design the parity test corpus (Step 4 work)
- Own implementation of bootstrap-runtime-loop or PB-Substrate / PB-Bootstrap-Process / PB-Runtime (separate lanes; referenced as dependencies)

**Authority chain**: Director-tier ratification grounds the model; subsequent worker briefs cite this doc; §1.8 PB-3 gate row close-criterion predicate cites this doc as L2.5 authority.

---

## §2 What parse IS structurally

Per the live `src/v3/compiler/src/parse_generated.rs:1-3` top-comment + `parse_tables.dag:1-30` substrate precedent:

**Parse is a substrate-driven recursive-descent function: `List<Token> → SurfaceModule`, dispatching on token kind per compile-time-generated grammar tables. Surface carriers (`SurfaceModule`, `SurfaceItem`, `SurfaceExpr`, `SurfacePattern`, `SurfaceType`) are declared in `.dag` substrate; parser dispatch tables are declared in `.dag` substrate; parser BODY is the residual hand-Rust SG-2c surface still requiring substrate capability to retire.**

Per Decision 3.B operator-ratified (b) compile-time parser tables: GrammarSpec is `.dag` substrate that compiles to parser dispatch tables at build time. The parser engine reads compile-time-generated tables; no runtime grammar interpretation. This is more thesis-accurate than runtime data-driven (option a) — substrate authority all the way down.

Per `feedback_lenses_not_passes`: parse dispatch decisions are substrate facts (token-to-operator mapping; bracket-role membership; type-rhs-boundary; primary-prefix dispatch). Anything parse "decides" is a grammar table that should be declared in `.dag`, not encoded in parser body logic.

**Failure shape**: fail-closed (per `feedback_fail_closed_discipline` + INVARIANTS C-8). Parse failures produce typed `ParseDiagnostic` variants with structural source-span attribution.

---

## §3 Input types (declared in `.dag` substrate)

Two input types feed parse (`List<Token>` from tokenize + `GrammarSpec` per Decision 3.B compile-time tables):

### §3.1 `List<Token>` (output of tokenize stage)

The token stream produced by PB-2 tokenize. Each `Token` carries a `TokenKind` discriminant + lexeme + source-span.

**Substrate authority**: `Token` type at `src/v3/compiler/src/tokenize.rs` (current hand-Rust); will migrate to `src/v3/std/tokenize.dag` per PB-2 tokenize L2.5 (sibling doc in flight). The carrier shape is stable across the Rust/`.dag` boundary; PB-3 parse migration is independent of PB-2 tokenize migration status.

**Lane dependency**: PB-2 tokenize (provides Token carrier + List<Token> output).

### §3.2 `GrammarSpec` (compile-time parser tables per Decision 3.B (b))

Per Decision 3.B operator-overrode my rec to (b) compile-time parser tables: GrammarSpec is `.dag` substrate that the build system compiles to parser-dispatch tables. NOT a runtime-interpreted spec.

**Live substrate precedent** at `src/v3/compiler/parse_tables.dag:1-30` (verified):
- Binary operator token-to-semantics mapping (SG-2c-1)
- Top-level item keyword → parse_item dispatch class (SG-2c-2)
- Type-RHS boundary keyword membership (SG-2c-3)
- Bracket opener/closer role membership (SG-2c-4)
- Primary-expression prefix openers (SG-2c-6)
- Primary-expression atomic tail tokens (SG-2c-7)

These 6 table-families ARE the GrammarSpec content. The build system reads `parse_tables.dag` + emits `parse_tables_generated.rs` (compile-time codegen); the parser body consumes the generated tables via `binary_op_at_level`, `top_level_item_dispatch`, `is_type_rhs_boundary_keyword`, `bracket_role`, `primary_prefix_dispatch`, `primary_atom_class`.

**Per Decision 3.B (b) "harder/more correct"**: compile-time table generation IS substrate authority all-the-way-down. Runtime grammar interpretation (option a my-original-rec) would require a generic parser engine reading GrammarSpec at runtime — more flexible but introduces runtime authority that compile-time tables don't have. (b) preserves the property that GrammarSpec is data-known-at-compile-time, not data-fetched-at-runtime.

**Substrate authority**: `src/v3/compiler/parse_tables.dag` is LIVE at HEAD (517 lines; 6 table-families). PB-3 migration extends this with the full SG-2c parser body authority via the substrate capability dependency in §6 (recursive list-body emission per SELF_HOSTING.md §6 Phase 4a).

**Lane dependency**: PB-Substrate (substrate-capability for recursive list-body emission — REQUIRED for SG-2c full parser body migration per `parse_tables.dag:18-22` STOP-AND-ESCALATE bullet).

---

## §4 Output types (declared in `.dag` substrate)

### §4.1 `SurfaceModule` (typed-state output)

Per `src/v3/std/parse_surface.dag:29` (verified live): `SurfaceModule` is the top-level parse output carrying `List<SurfaceItem>` + module-level metadata.

**Construction-time invariant**: every Token consumed produces either a SurfaceItem/SurfaceExpr/SurfacePattern/SurfaceType/SurfaceLiteral variant in the output tree (parser advanced) OR a ParseDiagnostic in the diagnostic stream (parser failed-closed at that position). No tokens silently dropped; no surface forms fabricated without source-span provenance.

**Substrate authority**: `src/v3/std/parse_surface.dag:29` (live; closed-axis SurfaceModule + SurfaceItem at :257 + SurfaceExpr at :149 + SurfacePattern at :123 + SurfaceType at :67 + SurfaceLiteral at :143). PB-3 parse output type is stable across the migration.

### §4.2 `ParseDiagnostic` (substrate extension per Decision 2.B)

**LIVE substrate state at HEAD**: `src/v3/std/diagnostics.dag` Diagnostic + AnyDiagnosticKind (per PR #3077 §4.2 audit). Per PR #3077 §12 Q7, Decision 2.B per-stage `source` axis is a SUBSTRATE EXTENSION requiring operator/PM ratification on path: (a) carrier-field extension OR (b) lane-local sum + mapping into CompilerKind.

**Per-stage discipline applies uniformly** across PB-2/3/4/5/6 — PR #3077 §12 Q7 is the cross-stage authority for Decision 2.B extension path. PB-3 ParseDiagnostic variants attach via whichever path Q7 ratifies.

**Typed-carrier discipline** (per openai-pro PR #3077 BLOCKING + INVARIANTS P2/P3): diagnostic boundaries carry typed facts, NOT String. Classification fields use typed closed-axis references; human display details may be String.

ParseDiagnostic variant shape (Step 2 brief authors against full set):

```
// Typed reference carriers (same pattern as PB-4 lower / PB-5 infer):
type TokenKindRef = TokenKind   // closed-axis sum from tokenize.dag
type SyntaxFormRef
  = ItemForm       // expected at top-level
  | ExprForm       // expected in expression position
  | PatternForm    // expected in pattern position
  | TypeForm       // expected in type position
  | LiteralForm    // expected at literal position

type ParseDiagnostic
  = UnexpectedToken { found: TokenKindRef, expected: List<TokenKindRef>, context: SyntaxFormRef }
  | UnterminatedConstruct { construct: SyntaxFormRef, opener_span: SourceSpan }
  | InvalidLiteral { kind: TokenKindRef, reason: String }   // reason is human display
  | DuplicateRecordFieldLabel { label: NonEmptyStr, prior_span: SourceSpan }   // per PR #3075 ratchet
  | (additional variants per Step 2 worker brief authoring against parse_generated.rs)
```

**Lane dependency**: PR #3077 §12 Q7 ratification (cross-stage); Director-tier per-stage variant authoring.

### §4.3 No separate `ParseResult` sum-variant — diagnostics coupled INTO `SurfaceModule`

Same pattern as PB-4 lower's PreInferDag + PB-5 infer's InferredDag (per PR #3077 §4.3 + PR #3085 §4.3): output IS the typed-state carrier; diagnostics couple INTO the SurfaceModule via the diagnostic table indexed by source-span (analogous to lower's port-keyed coupling but with span as the natural key for parse-stage failures since pre-substrate positions don't yet have ports).

**Cross-stage consistency**:
- PB-3 parse + PB-4 lower + PB-5 infer: output IS typed-state carrier with diagnostics coupled structurally
- PB-6 emit: uses EmissionResult sum because emit produces target-language bytes (different output domain)
- Discriminator: when output is a STRUCTURAL value (Dag-shape or Surface-tree), partial-failure couples structurally; when output is FINAL ARTIFACT (target source bytes), partial-failure couples via Result sum.

Signature: `fn parse(tokens: List<Token>, grammar: GrammarSpec) -> SurfaceModule` (NOT ParseResult). Diagnostics coupled via the SurfaceModule's diagnostic-table field.

---

## §5 Substrate-driven parsing (the core)

parse's structure is **recursive-descent dispatching on compile-time-generated grammar tables**, NOT decision-engine logic. Per `feedback_lenses_not_passes`:

### §5.1 Compile-time table generation

Per Decision 3.B (b) operator-ratified: GrammarSpec is `.dag` substrate compiled to parser dispatch tables at build time. Live precedent at `parse_tables.dag` + `parse_tables_generated.rs`. The 6 table-families:

1. Binary-operator precedence table (SG-2c-1)
2. Top-level item keyword dispatch (SG-2c-2)
3. Type-RHS boundary keyword membership (SG-2c-3)
4. Bracket opener/closer role (SG-2c-4)
5. Primary-prefix dispatch (SG-2c-6)
6. Primary-atom class (SG-2c-7)

Step 3 of PB-3 migration extends this with the full parser-body authority once the substrate capability dependency lands (§6).

### §5.2 Recursive-descent parser body

Per `parse_generated.rs:138`: `pub fn parse(tokens: &[Token], file: &str) -> Result<SurfaceModule, Diagnostic>` — current hand-Rust recursive-descent body. Reads tokens + dispatches on TokenKind per generated tables + recursively descends through grammar productions.

The parser body is the RESIDUAL SG-2c surface. Per `parse_tables.dag:13-22`: SG-2c proper migration is blocked on substrate capability (recursive list-body emission per SELF_HOSTING.md §6 Phase 4a + `src/v3/std/list.dag:13-15`). Until that lands, the parser body cannot move to `.dag` without a hidden Rust host layer (forbidden per STOP-AND-ESCALATE).

### §5.3 Dependency on substrate capability

Per `parse_tables.dag:13-22` STOP-AND-ESCALATE bullet:

> "SG-2c proper (parser authority proper — retiring `parse_parser_body.txt` as parse logic) is blocked on a named substrate capability: recursive list-body emission over `List<Token>` with cursor threading. See `src/v3/std/list.dag:13-15` (...) and SELF_HOSTING.md §6 Phase 4a. Until that lands, any full `.dag` parser port routes through a hidden Rust host layer, which SG-2c's STOP-AND-ESCALATE bullet forbids."

**Implication for PB-3 migration**: Step 3 (`.dag` implementation of parser body) BLOCKED on substrate-capability landing. Step 2 (pipeline-slot declaration) + Step 3a (extend grammar tables in `parse_tables.dag`) are unblocked.

---

## §6 Substrate prereqs (per-Gap-tier anchored)

Per `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`.

| Prereq | Substrate authority | Gap-tier lane | Status at HEAD (as of 2026-05-14) |
|---|---|---|---|
| PB-2 Tokenize | `src/v3/std/tokenize.dag` (NEW per PB-2 L2.5) + Token carrier | PB-2 lane (R3 Substrate Mgr post-PB-3) | NOT-STARTED; Director PB-2 L2.5 in flight (sibling doc) |
| Live SurfaceModule + Surface* carriers | `src/v3/std/parse_surface.dag` (live; closed-axis sums) | PB-Substrate | LIVE at HEAD per PR #3077 §3.1 audit |
| Live grammar tables substrate | `src/v3/compiler/parse_tables.dag` (517 lines; SG-2c-1/2/3/4/6/7 tables) | PB-Substrate | LIVE at HEAD; Step 3 extends |
| Substrate-capability: recursive list-body emission | `src/v3/std/list.dag` capability + SELF_HOSTING.md §6 Phase 4a | PB-Substrate + R3 Grounding Mgr lane | BLOCKER for full Step 3; per parse_tables.dag:13-22 STOP-AND-ESCALATE |
| ParseDiagnostic substrate extension | extension of `src/v3/std/diagnostics.dag:150` per Decision 2.B per PR #3077 §12 Q7 | PB-Substrate + Director-tier per-stage authoring | Carrier LIVE; per-stage variant authoring NEW per Q7 ratification |

**Critical observation**: PB-3 parse has a HARD DEPENDENCY on substrate-capability landing (recursive list-body emission) for Step 3 full implementation. Step 2 + grammar-table extensions are unblocked. This makes PB-3 migration HARDER than PB-4/PB-5 — the substrate-capability gap is real, not just an L2.5 ratification question.

---

## §7 Cross-stage coordination

### §7.1 Upstream dependencies

parse depends on `List<Token>` from tokenize (PB-2). PB-2 tokenize migration is downstream in the bottom-up order. Per `src/v3/SELF_HOSTING.md` §2 migration order, parse migrates AFTER tokenize substrate-side stable, but Token carrier shape is stable regardless of whether tokenize-emitter is hand-Rust or `.dag` — PB-3 parse migration is independent of PB-2 tokenize migration status (same independence pattern as PB-4 lower vs PB-3 parse per PR #3077 §12 Q6).

### §7.2 Downstream consumers

PB-4 lower consumes `SurfaceModule` from parse (per PR #3077 §3.1). The carrier is LIVE at parse_surface.dag; downstream stages don't depend on parse's migration timing.

Per Decision 2.B discriminated-union diagnostics: parse's diagnostics are discriminable by source via whichever substrate-extension path PR #3077 §12 Q7 ratifies. **PR #3077 §12 Q7 must ratify before any Step 2 worker brief authoring** (cross-stage authority for Decision 2.B extension path).

### §7.3 Sibling-stage coordination

Cross-stage discipline: tokenize → List<Token> → parse → SurfaceModule → lower → PreInferDag → infer → InferredDag → emit → EmissionResult.

Per `feedback_target_agnostic_ir`: parse's output carries no target-specific facts (Surface carriers are target-agnostic by construction).

---

## §8 Two shapes of omni-emission — N/A for parse

parse is target-agnostic structural parsing; Shape A/B disambiguation lives at emit stage. PB-3 parse has no Shape A/B framing.

---

## §9 SELF_HOSTING.md §2.2 4-step applied to PB-3 parse

| Step | Deliverable | Owner | Substrate |
|---|---|---|---|
| **Step 1: Model review** | THIS DOC | Director (zesty-bear-812) | docs/design-parse-stage-l25-model.md (this doc) |
| **Step 2: Pipeline slot** | `fn parse(tokens: List<Token>, grammar: GrammarSpec) -> SurfaceModule` declared in compiler.dag with `ExternalRealization` body (Rust-backed placeholder pointing to current `parse_generated.rs:138`). Signature consistent with PB-4/PB-5 pattern (output IS typed-state carrier; diagnostics coupled INTO SurfaceModule). | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 2 brief | compiler.dag refinement |
| **Step 3a: Grammar table extension** | Extend `src/v3/compiler/parse_tables.dag` with remaining grammar productions (unary operators / let-bindings / fn-signatures / pattern-syntax / module-imports / etc.); regenerate `parse_tables_generated.rs` | R3 Substrate Mgr — worker dispatched against Director-authored Step 3a brief; **unblocked NOW** (doesn't depend on substrate capability) | parse_tables.dag extension |
| **Step 3b: Full parser-body `.dag` migration** | `src/v3/std/parse.dag` — full recursive-descent parser body in `.dag`; consumes parse_tables.dag-generated dispatch tables | R3 Substrate Mgr — **BLOCKED on substrate-capability landing** (recursive list-body emission per `list.dag:13-15` + SELF_HOSTING.md §6 Phase 4a); cannot proceed without it | src/v3/std/parse.dag (NEW substrate authority) — DEPENDS on substrate-capability |
| **Step 4: Parity test + simultaneous Rust deletion** | Parity verification authored as `.dag` TestClaim — assert `parse_via_rust(tokens) == parse_via_dag(tokens, grammar_spec)` structural-equality across canonical corpus. **P5 dissolution receipt**: TestClaim is transient-by-construction; dissolves when `parse_generated.rs:138` parse() body deletes in same PR. Any hand-Rust scaffolding bears P5 receipt: `parity_parse_dag_vs_rust_scaffolding — transient; dissolves with parse_generated.rs parse() body deletion in same PR per Step 4 atomic discipline`. `parse_parser_body.txt` + `parse_generated.rs` parse() body DELETED in same PR. EXPECTED_HAND_AUTHORED_NON_TEST shrinks by N entries at PR-merge. | R3 Substrate Mgr — worker dispatched against Director-authored Step 4 brief | tests/parity_parse_dag_vs_rust (TestClaim shape) + parse body deletion |

**Critical: parity test is against PARSE.RS OUTPUT, not against parse.dag-template-of-parse.rs** per `feedback_paper_shrink_variants`. Same discipline as PB-6/PB-5/PB-4 Step 4.

---

## §10 Determinism invariant preservation

Parsing is inherently deterministic (token stream + grammar → unique surface tree per LL(k) discipline); the `.dag` implementation preserves this via structural iteration order matching the token stream order. No HashMap iteration without sorted-key discipline.

Per Step 3b brief authoring (post-substrate-capability landing): use structural recursive-descent matching the current `parse_generated.rs` body shape.

---

## §11 Parsing invariants (cross-cutting)

Per `feedback_fail_closed_discipline` C-8 + parser hygiene:

- **No silent token-skipping**: every consumed token contributes to a Surface variant OR triggers a ParseDiagnostic.
- **Source-span provenance**: every Surface variant + every ParseDiagnostic carries `SourceSpan`. No fabricated spans.
- **Recursive depth bounded**: parser body uses bounded recursion per grammar rules; no fixed-point iteration.
- **Anti-bridge invariant** (per `feedback_no_textual_enforcement_bridges`): grammar tables are the SINGLE authority on token-to-syntactic-form mapping; no fallback hand-Rust heuristics.

---

## §12 Open design questions (operator/PM ratification)

These surface to operator/PM before Step 2 dispatch:

### Q1: Substrate-capability dependency — when does Step 3b unblock?

Per `parse_tables.dag:13-22` + SELF_HOSTING.md §6 Phase 4a: SG-2c full parser-body migration blocked on substrate capability (recursive list-body emission over `List<Token>` with cursor threading).

**Path options**:
- **(a) Wait for substrate-capability landing in R3**: PB-Substrate Mgr authors substrate-capability before PB-3 Step 3b dispatches
- **(b) Operator-ratified scope extension**: substrate-capability is part of PB-3 Step 3b scope; warm-wolf-698 authors both in same PR
- **(c) Defer PB-3 Step 3b to post-R3**: only PB-3 Step 2 + Step 3a (grammar table extension) land in R3; full parser body migration is R4 scope

**Director-recommend: (b) bundled** — PB-3 Step 3b explicitly depends on substrate-capability; bundling keeps the authority chain clean (one worker, one PR, one P5 receipt). Operator/PM ratification.

### Q2: Compile-time table generation discipline (per Decision 3.B (b))

Decision 3.B operator-overrode my rec to (b) compile-time parser tables. Existing precedent at `parse_tables.dag` + `parse_tables_generated.rs`. Step 3a extends this.

**Open question**: does Step 3a extension introduce any axis NOT currently in the 6 table-families? (e.g., custom-operator definitions, module-imports syntax, where-clause refinement syntax). Director-recommend: Step 3a worker brief enumerates the FULL grammar table set by grepping `parse_generated.rs` for remaining open-coded TokenKind dispatch sites; each is a candidate new table-family.

### Q3: ParseDiagnostic variant exhaustiveness

`parse_generated.rs` body emits multiple Diagnostic variants today. Step 2 worker brief must enumerate the full set to inform PR #3077 §12 Q7 ratification (which extension path).

**Director-recommend**: Step 2 worker grep-enumerates all `Diagnostic::*` construction sites in `parse_generated.rs`; reports the full variant set as input to Q7 ratification. The lane-local-sum-mapping option (b) per Q7 lets PB-3 author the per-stage variant set cleanly.

### Q4: Pre-substrate-capability scope vs post-capability scope

If Director-recommend (b) in Q1 holds: PB-3 Step 3b waits for substrate-capability. What CAN PB-3 do BEFORE substrate-capability lands?

**Director-recommend**:
- Step 2 (pipeline-slot in compiler.dag) — unblocked
- Step 3a (grammar table extension in parse_tables.dag) — unblocked
- Step 3b (full parser body `.dag` migration) — blocked on substrate-capability

This phases naturally with the substrate-capability landing as the trigger for Step 3b.

### Q5: Migration scope phasing

`parse_generated.rs` is 2018 lines. Per `feedback_paper_shrink_variants` discipline, phased migration with per-phase P5 receipts is acceptable.

**Director-recommend: Step 3a + Step 3b PHASING**:
- 3a: grammar table extension (parse_tables.dag growth + parse_tables_generated.rs regen) — landed as own PR
- 3b: full parser body `.dag` migration + parser body deletion in same PR

Each phase = own PR + own parity test + own P5 receipt. Step 4 (parity-and-delete) is ONLY the 3b phase since 3a doesn't yet delete the parser body.

### Q6: PB-2 tokenize landing dependency

PB-3 parse's input is `List<Token>` from tokenize (PB-2). **Does PB-3 parse migration block on PB-2 tokenize migration completing?**

**Director-recommend: NO — PB-3 parse migrates independently of PB-2 tokenize status**, same shape as PB-4 lower per PR #3077 §12 Q6 + PB-5 infer per PR #3085 §12 Q6. Token carrier shape is stable; PB-3 parse migrates when its substrate (GrammarSpec + parse_tables.dag + substrate-capability for Step 3b) is at HEAD.

---

## §13 Non-goals

- **`.dag` implementation of parser body** — Step 3b work, separate brief (BLOCKED on substrate-capability)
- **Per-rule tactical decisions** — Step 3 work
- **Test corpus design + parity-test harness implementation** — Step 4 work
- **Substrate-capability authoring (recursive list-body emission)** — PB-Substrate / R3 Grounding Mgr lane scope
- **Bootstrap-runtime-loop concerns** — separate lanes
- **PB-2 tokenize migration** — separate L2.5 doc + lane
- **PB-4 lower migration** — separate L2.5 doc (PR #3077)
- **PB-5 infer migration** — separate L2.5 doc (PR #3085)
- **PB-6 emit migration** — already landed at PR #3066
- **Shape A/B emission** — emit's concern, not parse

---

## §14 Acceptance criteria for this L2.5 model

This doc lands on main when:

1. ✅ All input types declared structurally with `.dag` substrate paths (§3)
2. ✅ All output types declared with typed-state carrier per cross-stage discipline (§4)
3. ✅ Substrate-driven parsing structure composed without decision logic (§5 — per `feedback_lenses_not_passes`)
4. ✅ All substrate prereqs named with Gap-tier / Mgr-lane anchors (§6) + substrate-capability dependency explicit
5. ✅ Cross-stage dependencies explicit (§7)
6. ✅ N/A — Shape A/B framing irrelevant for parse (§8)
7. ✅ SELF_HOSTING.md §2.2 4-step concretely applied (§9) with Step 3 phased into 3a/3b
8. ✅ Determinism preservation discipline (§10)
9. ✅ Parsing invariants explicit (§11)
10. ✅ Open design questions enumerated for operator/PM ratification (§12)
11. ⏳ Operator/PM ratification on §12 Q1-Q6

Post-ratification: this doc becomes substrate authority for Step 2 + Step 3a worker brief authoring + §1.8 PB-3 gate row close-criterion predicate.

---

## §15 Authoring sequence post-ratification

1. **Operator / PM-delegate ratifies §12 Q1–Q6** (per 2026-05-14 directive)
2. **PM amends close plan** to route through PB-X lanes + cite this doc as PB-3 L2.5 substrate
3. **PM amends §1.8** with PB-3 gate row citing this doc
4. **PR #3077 §12 Q7 ratifies** (cross-stage Decision 2.B extension path) — affects PB-3 ParseDiagnostic shape
5. **Director authors PB-3 Step 2 worker brief** (pipeline-slot ExternalRealization PR scope)
6. **R3 Substrate Mgr (warm-wolf-698)** dispatches Step 2 worker
7. **Director ratifies Step 2 PR + admin-merges** when CI clears
8. **Director authors PB-3 Step 3a worker brief** (grammar table extension — unblocked NOW)
9. **R3 Substrate Mgr** dispatches Step 3a worker
10. **Director ratifies Step 3a PR**, admin-merges
11. **WAIT for substrate-capability landing** (recursive list-body emission per Q1)
12. **Director authors PB-3 Step 3b worker brief** (full parser body `.dag` migration)
13. **R3 Substrate Mgr** dispatches Step 3b worker
14. **Director ratifies Step 3b PR**, admin-merges
15. **Director authors PB-3 Step 4 worker brief** (parity test + simultaneous parser body deletion)
16. **R3 Substrate Mgr** dispatches Step 4 worker; parity against `parse_generated.rs` parse() body OUTPUT
17. **Director ratifies Step 4 PR**, admin-merges → parser body DELETED → PB-3 gate row CLOSES

Subsequent L2.5 model (PB-2 tokenize) follows same sequence with its own substrate-capability dependencies.

---

## §16 Cross-references

**Primary authority**:
- `src/v3/SELF_HOSTING.md` §2.2 (4-step migration discipline)
- `src/v3/SELF_HOSTING.md` §6 Phase 4a (substrate-capability dependency)
- `docs/design-pure-bootstrap-zero.md` (PB-X lane framing)
- `docs/substrate-reflection-design.md` §12.6 (migration order)
- `docs/design-emit-stage-l25-model.md` (PB-6; sets L2.5 template)
- `docs/design-lower-stage-l25-model.md` (PB-4; cross-stage diagnostic / typed-state pattern)
- `docs/design-infer-stage-l25-model.md` (PB-5; substrate-driven dispatch precedent)

**Live substrate referenced**:
- `src/v3/std/parse_surface.dag:29` (SurfaceModule) + closed-axis sibling carriers
- `src/v3/compiler/parse_tables.dag` (517 lines; SG-2c-1/2/3/4/6/7 grammar tables; live precedent for Decision 3.B (b) compile-time tables)
- `src/v3/compiler/src/parse_generated.rs:138` (current `fn parse` signature; recursive-descent residual)
- `src/v3/std/diagnostics.dag:150` (Diagnostic carrier — extends with ParseDiagnostic per PR #3077 §12 Q7 ratification)
- `src/v3/std/list.dag:13-15` (substrate-capability dependency: recursive list-body emission)

**Memory disciplines applied**:
- `feedback_lenses_not_passes` (parse = substrate-table-driven dispatch, NOT decision engine)
- `feedback_fail_closed_discipline` C-8 (ParseDiagnostic coupled INTO SurfaceModule)
- `feedback_state_space_vs_behavioral_invariants` (typed-state SurfaceModule at output)
- `feedback_target_agnostic_ir` (parse output carries no target-specific facts)
- `feedback_paper_shrink_variants` (Step 4 parity = genuine deletion, not relocation)
- `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id` (Gap-tier anchors)
- `feedback_no_textual_enforcement_bridges` (anti-bridge: grammar tables are single authority)
- `feedback_grep_carrier_semantic_before_ratification` (4-axis grep applied at authoring time)
- `feedback_discipline_change_audit_all_contract_mentions` (signature consistency across sections)

**Surfaces awaiting**:
- Operator/PM ratification on §12 Q1–Q6
- PR #3077 §12 Q7 ratification (cross-stage Decision 2.B extension path)
- Substrate-capability landing (recursive list-body emission per Q1 + SELF_HOSTING.md §6 Phase 4a)
- PM Phase 2 close plan + §1.8 amendments citing this doc
