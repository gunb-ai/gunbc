# Parse Pipeline Stage — L2.5 Domain Model (PB-3)

**Status:** DRAFT — Director-tier authoring per operator ratification 2026-05-14 (Decision 1.A scoping = Option A; Decision 3.B operator-overrode my rec to (b) compile-time parser tables — the "harder/more correct" path).

**Authoring date:** 2026-05-14.
**Authoring tier:** Director (zesty-bear-812).
**Lane:** PB-3 (parse) per `docs/design-pure-bootstrap-zero.md` + `src/v3/SELF_HOSTING.md` §2 4-step migration discipline.
**Migration order rank:** 4th (per `docs/substrate-reflection-design.md` §12.6 — emit → lower → infer → parse for the 4 pipeline-stage migrations explicitly tabled there; `docs/design-pure-bootstrap.md` §"PB-2 — tokenize retire" (line ~134) extends with tokenize; PB-6 emit landed PR #3066, PB-4 lower at PR #3077, PB-5 infer at PR #3085).
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

Per Decision 3.B operator-ratified (b) compile-time parser tables: the 6 SG-2c table-families in `parse_tables.dag` (conceptually called "GrammarSpec", though there is no `type GrammarSpec` carrier — see §3.2) are codegenned to parser dispatch tables at build time. The parser engine reads compile-time-generated tables; no runtime grammar interpretation, no runtime grammar input. This is more thesis-accurate than runtime data-driven (option a) — substrate authority all the way down.

Per `feedback_lenses_not_passes`: parse dispatch decisions are substrate facts (token-to-operator mapping; bracket-role membership; type-rhs-boundary; primary-prefix dispatch). Anything parse "decides" is a grammar table that should be declared in `.dag`, not encoded in parser body logic.

**Failure shape**: fail-closed (per `feedback_fail_closed_discipline` + INVARIANTS C-8). Parse failures produce typed `ParseDiagnostic` variants with structural source-span attribution.

---

## §3 Input types (declared in `.dag` substrate)

**One** input type feeds parse at the API boundary: `List<Token>` from tokenize. Parser-dispatch tables (the conceptual "GrammarSpec" per Decision 3.B (b)) are NOT a runtime input — they are compile-time substrate at `parse_tables.dag`, codegenned into the parser body, and have no `type GrammarSpec` carrier.

> **Cursor PR #3126 INLINE BLOCKING line:32 correction (2026-05-14T23:30:04Z, fixed in PR #3138)**: earlier draft framed `GrammarSpec` as a second declared `.dag` input type. Verified via `grep -rn "^type GrammarSpec\b" src/v3/ dsl/` — empty. No `type GrammarSpec` carrier exists in the repo. "GrammarSpec" in this doc is a concept-level grouping for the 6 SG-2c table-families in `parse_tables.dag` (§3.2 below); it is not a substrate type and never appears in the Step-2 `fn parse` signature.

### §3.1 `List<Token>` (output of tokenize stage)

The token stream produced by PB-2 tokenize. Each `Token` carries a `TokenKind` discriminant + lexeme + source-span.

**Substrate authority** (corrected per cursor PR #3126 BLOCKING line:29 — earlier draft cited non-existent `tokenize.rs`): `Token` type lives in LIVE `.dag` substrate at `src/v3/std/tokenize.dag:65-67` (the shared taxonomy authority). Tokenizer implementation also lives in `.dag` at `src/v3/compiler/tokenize.dag` (154 lines). Codegen artifact at `src/v3/compiler/src/tokenize_generated.rs` (362 lines, auto-generated from tokenize.dag via `regen_tokenize`). The hand-Rust file `src/v3/compiler/src/tokenize.rs` does NOT exist — that was an earlier-draft phantom reference. Per `design-pure-bootstrap.md` §"PB-2 — tokenize retire" (line ~134), PB-2 has already substantially landed at substrate level (residual scaffold-retirement scope per PB-2 L2.5 PR #3127). The Token carrier shape is stable; PB-3 parse migration is independent of PB-2's residual-retirement timing per §7.1 axis split.

**Lane dependency**: PB-2 tokenize (provides Token carrier + List<Token> output).

### §3.2 Compile-time parser tables per Decision 3.B (b) — concept "GrammarSpec", NOT a carrier

Per Decision 3.B operator-overrode my rec to (b) compile-time parser tables: "GrammarSpec" names the **conceptual** grouping of 6 SG-2c table-families authored in `parse_tables.dag` and codegenned into the parser body. There is no `type GrammarSpec` declaration; the name is shorthand for those 6 table-families taken collectively, not a substrate carrier or runtime input.

**Live substrate precedent** at `src/v3/compiler/parse_tables.dag:1-30` (verified):
- Binary operator token-to-semantics mapping (SG-2c-1)
- Top-level item keyword → parse_item dispatch class (SG-2c-2)
- Type-RHS boundary keyword membership (SG-2c-3)
- Bracket opener/closer role membership (SG-2c-4)
- Primary-expression prefix openers (SG-2c-6)
- Primary-expression atomic tail tokens (SG-2c-7)

These 6 table-families ARE what "GrammarSpec" names. The build system reads `parse_tables.dag` + emits `parse_tables_generated.rs` (compile-time codegen); the parser body consumes the generated tables via `binary_op_at_level`, `top_level_item_dispatch`, `is_type_rhs_boundary_keyword`, `bracket_role`, `primary_prefix_dispatch`, `primary_atom_class`. None of these is reached through a `GrammarSpec` carrier; they are direct table lookups inside the parser body.

**Per Decision 3.B (b) "harder/more correct"**: compile-time table generation IS substrate authority all-the-way-down. Runtime grammar interpretation (option a my-original-rec) would require a generic parser engine reading a runtime spec — more flexible but introduces runtime authority that compile-time tables don't have. (b) preserves the property that the grammar is data-known-at-compile-time, not data-fetched-at-runtime, and therefore needs no runtime carrier.

**Substrate authority**: `src/v3/compiler/parse_tables.dag` is LIVE at HEAD (517 lines; 6 table-families). PB-3 migration extends this with the full SG-2c parser body authority via the substrate capability dependency in §6 (recursive list-body emission per SELF_HOSTING.md §6 Phase 4a).

**Lane dependency**: PB-Substrate (substrate-capability for recursive list-body emission — REQUIRED for SG-2c full parser body migration per `parse_tables.dag:18-22` STOP-AND-ESCALATE bullet).

---

## §4 Output types (declared in `.dag` substrate)

### §4.1 `SurfaceModule` (typed-state output)

Per `src/v3/std/parse_surface.dag:29` (verified live): `SurfaceModule { items: List<SurfaceItem> }` — single field only. Per cursor PR #3126 APPROVE_WITH_COMMENTS line:79 + INVARIANTS P1 live-state honesty: earlier draft "+ module-level metadata" was a phantom addition; live carrier has ONLY the `items` field.

**Live failure boundary** (per `parse_generated.rs:138`): parse returns `Result<SurfaceModule, Diagnostic>` (fail-closed; aborts on first parse error). NO diagnostic-stream coupling in live state.

**Construction-time invariant** (live + ratified shape per §4.3 Result-sum framing): every Token consumed produces either a SurfaceItem/SurfaceExpr/SurfacePattern/SurfaceType/SurfaceLiteral variant in the Ok-arm output tree (parser advanced) OR triggers Err-arm with a typed `ParseDiagnostic` (parser failed-closed at first error position). No tokens silently dropped; no surface forms fabricated without source-span provenance; no partial-parse SurfaceModule with embedded diagnostics (per codex BLOCKING #3126 fail-closed correction).

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

// Diagnostic = record-wraps-kind pattern per codex BLOCKING PR #3126 finding 1:
// matches live Diagnostic shape at diagnostics.dag:150 (record { kind, span, ... }
// where kind is the closed-axis sum, span on the record). Earlier draft mixed
// kind + span into variant fields directly; corrected to:

type ParseDiagnostic {
  kind: ParseDiagnosticKind
  span: SourceSpan
  // Optional: additional context fields per Step 2 brief enumeration
}

type ParseDiagnosticKind
  = UnexpectedToken { found: TokenKindRef, expected: List<TokenKindRef>, context: SyntaxFormRef }
  | UnterminatedConstruct { construct: SyntaxFormRef, opener_span: SourceSpan }   // additional opener_span here; ParseDiagnostic.span is the unterminated-end position
  | InvalidLiteral { kind: TokenKindRef, reason: String }   // reason is human display
  | DuplicateRecordFieldLabel { label: NonEmptyStr, prior_span: SourceSpan }   // per PR #3075 ratchet; ParseDiagnostic.span is current-site
  // (additional variants per Step 2 worker brief authoring against parse_generated.rs)

// Per cursor PR #3126 BLOCKING line:65: span lives on ParseDiagnostic record,
// not on each kind variant. Per codex finding 1: matches live Diagnostic
// shape (kind + span on record carrier). Single source of truth for span;
// variant-specific spans (opener_span / prior_span) live on kind variants
// where they're meaningful.
```

**Lane dependency**: PR #3077 §12 Q7 ratification (cross-stage); Director-tier per-stage variant authoring.

### §4.3 Parse output is `Result<SurfaceModule, ParseDiagnostic>` — NO SurfaceModule diagnostics-extension

> **Codex PR #3138 BLOCKING (sha cd6e8d15) revision (fixed at HEAD of PR #3138)**: this section previously proposed an `items + diagnostics` extension to `SurfaceModule` modeled on PB-4 lower's PreInferDag / PB-5 infer's InferredDag. That model is REJECTED for parse. The corrected Decision (per codex BLOCKING PR #3126 ratified into §6 Step 2 + §4.2) is that parse uses **Result-sum** (fail-fast first-error abort), like PB-6 emit and unlike PB-4 lower / PB-5 infer. The earlier-draft SurfaceModule extension is superseded; no `diagnostics` field is being added to `parse_surface.dag`.

**Live substrate state at HEAD** (verified via `grep -n "^type SurfaceModule" src/v3/std/parse_surface.dag`): `type SurfaceModule { items: List<SurfaceItem> }` — single field, NO diagnostic field, and no extension is now proposed. Parse-stage diagnostics live OUTSIDE SurfaceModule, on the Err branch of `Result<SurfaceModule, ParseDiagnostic>`.

**Cross-stage discriminator** (the rule that places parse with emit, not with lower/infer):
- **Result-sum** (PB-3 parse + PB-6 emit): fail-fast output domain — a partial parse tree or a partial target-byte buffer is not a valid intermediate; the stage either produces the complete artifact or fails with the first diagnostic.
- **Typed-state-with-coupled-diagnostics** (PB-4 lower + PB-5 infer): structural output domain — Unresolved ports / pre-inferred Dag are valid intermediates consumed downstream, so the diagnostics couple structurally into the carrier.

Live parser at `src/v3/compiler/src/parse_generated.rs:138` matches this: `pub fn parse(tokens: &[Token], file: &str) -> Result<SurfaceModule, Diagnostic>`. The Step 2 contract refines that to `fn parse(tokens: List<Token>) -> Result<SurfaceModule, ParseDiagnostic>` per §6.

Signature: `fn parse(tokens: List<Token>) -> Result<SurfaceModule, ParseDiagnostic>` per live `parse_generated.rs:138` shape (Result<SurfaceModule, Diagnostic>; ParseDiagnostic is per-stage refinement per §4.2).

**Per codex BLOCKING PR #3126**: earlier draft of this section proposed `fn parse(tokens, grammar: GrammarSpec) -> SurfaceModule` with embedded diagnostics field. Both load-bearing problems:

1. **GrammarSpec parallel-authority** (P2 violation): §3.2 says the parser tables are compile-time-only, NOT runtime-interpreted, and have no `type GrammarSpec` carrier in the repo. The earlier-draft signature accepted a `GrammarSpec` value as runtime input — that would have invented a parallel authority alongside the compile-time tables AND a substrate type that doesn't exist. The corrected signature consumes the compiled tables via internal parser-table-driven dispatch — no runtime grammar input, no `GrammarSpec` parameter, no carrier needed.

2. **Fail-closed weakening** (P3 + Practice 1+2 violation): live parser at `parse_generated.rs:138` returns `Result<SurfaceModule, Diagnostic>` (fail-closed, aborts on first error). Earlier draft proposed `SurfaceModule` with embedded diagnostics — would let partial-parse states be constructible + let downstream observe "success" output after parse failure. The corrected signature preserves fail-closed Result: parse either succeeds with complete SurfaceModule OR fails with first-encountered ParseDiagnostic (no partial states).

Cross-stage consistency NOTE: parse uses Result-sum (like emit) NOT typed-state-with-coupled-diagnostics (like lower/infer) because parse is fail-fast (single first-error abort). Different discriminator from `feedback_fail_closed_discipline`-applied-to-Dag-output stages:
- **Result-sum** (parse + emit): fail-fast output domain; partial output isn't a valid intermediate
- **Typed-state-with-coupled-diagnostics** (lower + infer): structural output domain where partial-failure (Unresolved ports) IS valid intermediate state consumed by downstream

---

## §5 Substrate-driven parsing (the core)

parse's structure is **recursive-descent dispatching on compile-time-generated grammar tables**, NOT decision-engine logic. Per `feedback_lenses_not_passes`:

### §5.1 Compile-time table generation

Per Decision 3.B (b) operator-ratified: the parser-dispatch tables (concept "GrammarSpec"; no `type GrammarSpec` carrier — see §3.2) are `.dag` substrate compiled at build time. Live precedent at `parse_tables.dag` + `parse_tables_generated.rs`. The 6 table-families:

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

**Implication for PB-3 migration**: Step 3 (`.dag` implementation of parser body) BLOCKED on substrate-capability landing. Step 2 (pipeline-slot declaration) + Step 3a (extend grammar tables in `parse_tables.dag`) are unblocked at HEAD — **gate**: PR #3077 §12 Q7 ratification was a hard precondition on Step 2 brief authoring (§7.2) and merged 2026-05-15T00:21:19Z, so the gate is satisfied. Step 2 must NOT be brief-authored before that merge timestamp; at HEAD it has been.

---

## §6 Substrate prereqs (per-Gap-tier anchored)

Per `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id`.

| Prereq | Substrate authority | Gap-tier lane | Status at HEAD (as of 2026-05-14) |
|---|---|---|---|
| PB-2 Tokenize | `src/v3/std/tokenize.dag` (LIVE — 143 lines; Token + TokenKind taxonomy already substantially landed per `design-pure-bootstrap.md` §"PB-2 — tokenize retire") | PB-2 lane (R3 Substrate Mgr) | LIVE substrate at HEAD; PB-2's residual scope is scaffold-retirement (SG-1a + character-level + codegen-driver per PB-2 L2.5 §1), NOT carrier authoring. PB-3 consumes the live Token carrier; carrier shape stable across PB-2's residual-retirement timing. |
| Live SurfaceModule + Surface* carriers | `src/v3/std/parse_surface.dag` (live; closed-axis sums) | PB-Substrate | LIVE at HEAD per PR #3077 §3.1 audit |
| Live grammar tables substrate | `src/v3/compiler/parse_tables.dag` (517 lines; SG-2c-1/2/3/4/6/7 tables) | PB-Substrate | LIVE at HEAD; Step 3 extends |
| Substrate-capability: recursive list-body emission | `src/v3/std/list.dag` capability + SELF_HOSTING.md §6 Phase 4a | PB-Substrate + R3 Grounding Mgr lane | BLOCKER for full Step 3; per parse_tables.dag:13-22 STOP-AND-ESCALATE |
| ParseDiagnostic substrate extension | extension of `src/v3/std/diagnostics.dag:150` per Decision 2.B per PR #3077 §12 Q7 | PB-Substrate + Director-tier per-stage authoring | Carrier LIVE; per-stage variant authoring NEW per Q7 ratification |

**Critical observation**: PB-3 parse has a HARD DEPENDENCY on substrate-capability landing (recursive list-body emission) for Step 3 full implementation. Step 2 + grammar-table extensions are unblocked at HEAD — **the additional Step 2 gate is PR #3077 §12 Q7 ratification** (§7.2 / §15 step 4), which merged 2026-05-15T00:21:19Z; without that merge timestamp in repo history Step 2 brief authoring is structurally blocked (P3 failure shape would land unfixed). This makes PB-3 migration HARDER than PB-4/PB-5 — the substrate-capability gap is real, not just an L2.5 ratification question.

---

## §7 Cross-stage coordination

### §7.1 Upstream dependencies

parse depends on `List<Token>` from tokenize (PB-2). **Two distinct axes per cursor PR #3126 APPROVE_WITH_COMMENTS clarification**:

1. **Substrate-stability ordering** (SELF_HOSTING.md §2 migration order; bottom-up): tokenize's substrate (Token carrier shape) must be stable BEFORE parse migration proceeds. This is already true at HEAD — `src/v3/std/tokenize.dag:65-67` declares the live Token carrier shape; tokenize.dag is the substrate authority. Substrate-side stability ✓.

2. **Migration-timing independence** (parallel-dispatch axis): PB-3 parse migration can ship in parallel with PB-2 tokenize MIGRATION — i.e., parse migration doesn't WAIT for PB-2's residual hand-Rust retirement (per PB-2 L2.5 §1: SG-1a + character-level scaffold + codegen-driver retirement). What parse needs is the stable Token CARRIER, which already exists; PB-2's migration is about retiring the residual emitter-side hand-Rust, not about changing the carrier.

So: ordering (substrate-stability) IS satisfied (live); independence (migration-timing) means parse migration is parallel-dispatchable with respect to PB-2's residual-retirement work. Both claims coherent on the same axis split; not contradictory.

### §7.2 Downstream consumers

PB-4 lower consumes `SurfaceModule` from parse (per PR #3077 §3.1). The carrier is LIVE at parse_surface.dag; downstream stages don't depend on parse's migration timing.

Per Decision 2.B discriminated-union diagnostics: parse's diagnostics are discriminable by source via the substrate-extension path PR #3077 §12 Q7 ratified. **Gate satisfied 2026-05-15T00:21:19Z** when PR #3077 merged; Step 2 worker brief authoring is unblocked at HEAD per §15 step 4. The contradiction cursor PR #3126 BLOCKING line:119 flagged (§6 unblocking claims vs §7.2 gate claim) is now resolved by the Q7 ratification merging rather than by retracting either statement.

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
| **Step 2: Pipeline slot** | `fn parse(tokens: List<Token>) -> Result<SurfaceModule, ParseDiagnostic>` declared in `src/v3/compiler/pipeline.dag` (per dsl/gunbc/compiler.dag:24 — internal pipeline lives in pipeline.dag, NOT generic compiler.dag) with `ExternalRealization` body (Rust-backed placeholder pointing to current `parse_generated.rs:138`). Signature matches live parser shape (fail-closed Result; aborts on first parse error). NO runtime GrammarSpec input — compile-time generated parser tables consumed via internal dispatch per Decision 3.B (b). Per codex BLOCKING #3126: distinct from lower/infer typed-state-with-coupled-diagnostics pattern; parse uses Result-sum like emit (fail-fast output domain). | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 2 brief | pipeline.dag refinement |
| **Step 3a: Grammar table extension** | Extend `src/v3/compiler/parse_tables.dag` with remaining grammar productions (unary operators / let-bindings / fn-signatures / pattern-syntax / module-imports / etc.); regenerate `parse_tables_generated.rs` | R3 Substrate Mgr — worker dispatched against Director-authored Step 3a brief; **unblocked NOW** (doesn't depend on substrate capability) | parse_tables.dag extension |
| **Step 3b/4 COMBINED: Full parser body + parity + simultaneous deletion** | `src/v3/std/parse.dag` (full recursive-descent parser body in `.dag` consuming parse_tables.dag-generated dispatch tables) + parity TestClaim + `parse_generated.rs:138` parse() body deletion — ATOMIC in single PR. Per cursor PR #3126 BLOCKING line:181 + `feedback_paper_shrink_variants` discipline: cannot land `.dag` parser body BEFORE Rust deletion (would create temporary Rust+`.dag` coexistence = paper-shrink-relocation risk). | R3 Substrate Mgr — **BLOCKED on substrate-capability landing** (recursive list-body emission per `list.dag:13-15` + SELF_HOSTING.md §6 Phase 4a); cannot proceed without it | src/v3/std/parse.dag (NEW substrate authority) + parse_generated.rs deletion — DEPENDS on substrate-capability |
| **Step 4: Parity test + simultaneous Rust deletion** | Parity verification authored as `.dag` TestClaim — assert `parse_via_rust(tokens) == parse_via_dag(tokens)` Result-equality across canonical corpus (both return `Result<SurfaceModule, ParseDiagnostic>`; equality on Ok arm SurfaceModule structural-equality + on Err arm ParseDiagnostic content-equality). **P5 dissolution receipt**: TestClaim is transient-by-construction; dissolves when `parse_generated.rs:138` parse() body deletes in same PR. Any hand-Rust scaffolding bears P5 receipt: `parity_parse_dag_vs_rust_scaffolding — transient; dissolves with parse_generated.rs parse() body deletion in same PR per Step 4 atomic discipline`. `parse_parser_body.txt` + `parse_generated.rs` parse() body DELETED in same PR. EXPECTED_HAND_AUTHORED_NON_TEST shrinks by N entries at PR-merge. | R3 Substrate Mgr — worker dispatched against Director-authored Step 4 brief | tests/parity_parse_dag_vs_rust (TestClaim shape) + parse body deletion |

**Critical: parity test is against PARSE.RS OUTPUT, not against parse.dag-template-of-parse.rs** per `feedback_paper_shrink_variants`. Same discipline as PB-6/PB-5/PB-4 Step 4.

---

## §10 Determinism invariant preservation

Parsing is inherently deterministic (token stream + grammar → unique surface tree per LL(k) discipline); the `.dag` implementation preserves this via structural iteration order matching the token stream order. No HashMap iteration without sorted-key discipline.

Per Step 3b brief authoring (post-substrate-capability landing): use structural recursive-descent matching the current `parse_generated.rs` body shape.

---

## §11 Parsing invariants (cross-cutting)

Per `feedback_fail_closed_discipline` C-8 + parser hygiene:

- **No silent token-skipping**: every consumed token contributes to a Surface variant OR triggers a ParseDiagnostic.
- **Source-span provenance** (corrected per cursor PR #3126 BLOCKING line:151): **most** Surface variants carry `SourceSpan` directly, but per live `src/v3/std/parse_surface.dag`: SurfaceItem::Let (line ~33) has fields `{ name, type_ann, expr }` with NO direct span; SurfaceLiteral variants `Int(String) | Bool(Bool) | String(String)` are plain-tuple variants with NO direct span. These cases acquire span via their enclosing carrier (Let-item inherits container span; SurfaceLiteral always wraps within `Literal { value, span }` per parse_surface.dag:150). Step 2 PR scope SHOULD audit whether the Let + SurfaceLiteral exceptions are structural-honest (acceptable per enclosing-carrier-provides-span) OR substrate-extension-required (add span directly to those variants). Earlier "every Surface variant carries SourceSpan" claim was incorrect; corrected to "every parse output structurally has source-span provenance via direct field OR enclosing carrier". Every ParseDiagnostic carries `SourceSpan` directly per §4.2.
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

**Live state correction** (per codex INLINE BLOCKING #3126): `parse_generated.rs` body emits a SINGLE `Diagnostic::ParseError` variant today (verified via `grep "Diagnostic::" src/v3/compiler/src/parse_generated.rs` — only ParseError construction sites; ~30+ sites all using the same single variant with different message/span content). The proposed `ParseDiagnostic` taxonomy in §4.2 is a SUBSTRATE EXTENSION — Step 2 worker brief extends the live single-variant into the proposed structured sum, NOT "enumerates existing multiple variants" (which was a misframing).

**Director-recommend**: Step 2 worker grep-enumerates all `Diagnostic::*` construction sites in `parse_generated.rs`; reports the full variant set as input to Q7 ratification. The lane-local-sum-mapping option (b) per Q7 lets PB-3 author the per-stage variant set cleanly.

### Q4: Pre-substrate-capability scope vs post-capability scope

If Director-recommend (b) in Q1 holds: PB-3 Step 3b waits for substrate-capability. What CAN PB-3 do BEFORE substrate-capability lands?

**Director-recommend**:
- Step 2 (pipeline-slot in `src/v3/compiler/pipeline.dag`) — unblocked at HEAD; gate was PR #3077 §12 Q7 (per §7.2 + §15 step 4), satisfied 2026-05-15T00:21:19Z
- Step 3a (grammar table extension in parse_tables.dag) — unblocked
- Step 3b (full parser body `.dag` migration) — blocked on substrate-capability

This phases naturally with the substrate-capability landing as the trigger for Step 3b.

### Q5: Migration scope phasing

`parse_generated.rs` is 2018 lines. Per `feedback_paper_shrink_variants` discipline, phased migration with per-phase P5 receipts is acceptable.

**Director-recommend: Step 3a + COMBINED-Step-3b/4 PHASING** (per cursor PR #3126 BLOCKING line:181 — earlier framing conflated Step 3b "migration" vs Step 4 "parity-and-delete" creating P5 receipt ambiguity):

- **Phase 3a** (separate PR): grammar table extension (parse_tables.dag growth + parse_tables_generated.rs regen). P5 receipt: ROADMAP.md deferral row naming Step 3b/4 as future-receipt scope (no parser-body deletion in 3a; refactor-only phase per `feedback_paper_shrink_variants` deletion-or-deferral discipline).
- **Phase 3b/4 COMBINED** (single PR): full parser body `.dag` migration + parity test (`.dag` TestClaim) + `parse_generated.rs:138` parse() body deletion + EXPECTED_HAND_AUTHORED_NON_TEST shrink — ALL atomic in one PR. P5 receipt: deletion + census shrink.

**Why combined Step 3b/4** (not sequential phases): if Step 3b lands `.dag` parser body BEFORE Step 4 deletes Rust parse() body, the Rust + `.dag` parser bodies coexist temporarily — paper-shrink-relocation risk per `feedback_paper_shrink_variants`. Combining ensures atomic substrate substitution: Rust parse() body never coexists with `.dag` parser body on main.

**Update to §9 + §15**: Step 3b row reframed as "Step 3b/4 COMBINED"; §15 sequence collapses steps 12-17 into single dispatch+merge for combined phase.

### Q6: PB-2 tokenize landing dependency

PB-3 parse's input is `List<Token>` from tokenize (PB-2). **Does PB-3 parse migration block on PB-2 tokenize migration completing?**

**Director-recommend: NO — PB-3 parse migrates independently of PB-2 tokenize status**, same shape as PB-4 lower per PR #3077 §12 Q6 + PB-5 infer per PR #3085 §12 Q6. Token carrier shape is stable; PB-3 parse migrates when its substrate (`parse_tables.dag` 6 table-families + substrate-capability for Step 3b recursive list-body emission) is at HEAD.

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
4. **PR #3077 §12 Q7 ratifies** (cross-stage Decision 2.B extension path) — affects PB-3 ParseDiagnostic shape. **DONE 2026-05-15T00:21:19Z** when PR #3077 (PB-4 lower L2.5) merged; Q7 ratification carried in that merge. The §6 / §7.2 "Step 2 must wait on Q7" gate is therefore now satisfied — Step 2 brief authoring is genuinely unblocked, not just procedurally listed as the next step.
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
- `feedback_fail_closed_discipline` C-8 (parse returns `Result<SurfaceModule, ParseDiagnostic>` per §4.3 + live `parse_generated.rs:138` — fail-closed Result-sum, NOT typed-state-with-coupled-diagnostics; distinct from lower/infer pattern per cross-stage discriminator at §4.3)
- `feedback_state_space_vs_behavioral_invariants` (parse output is `Result<SurfaceModule, ParseDiagnostic>` — the type rules out partial-parse states by construction; SurfaceModule itself carries no diagnostic field per §4.3)
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
