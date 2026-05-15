# Tokenize Pipeline Stage — L2.5 Domain Model (PB-2)

**Status:** DRAFT — Director-tier authoring per operator ratification 2026-05-14 (Decision 1.A scoping = Option A).

**Authoring date:** 2026-05-14.
**Authoring tier:** Director (zesty-bear-812).
**Lane:** PB-2 (tokenize) per `docs/design-pure-bootstrap-zero.md` + `src/v3/SELF_HOSTING.md` §2 4-step migration discipline.
**Migration order rank:** 5th + LAST in pipeline-stage sequence (per `docs/substrate-reflection-design.md` §12.6 — emit → lower → infer → parse for the 4 pipeline-stage migrations explicitly tabled there; `docs/design-pure-bootstrap.md` §"PB-2 — tokenize retire" (line ~134) extends with tokenize).
**Routing authority chain:** operator-ratification + PM-delegate (per 2026-05-14 directive) → PM amends close plan + §1.8 PB-2 gate row → Director authors per-step worker briefs → R3 Substrate Mgr (warm-wolf-698) dispatches workers.

---

## §1 Purpose + scope

This document is the **Step 1 model review** per `src/v3/SELF_HOSTING.md` §2.2 4-step discipline applied to PB-2 tokenize-stage migration.

**Distinct from sibling pipeline-stage L2.5s**: PB-2 tokenize is the FURTHEST-ALONG pipeline stage. Substrate authority already lives in `.dag`:
- `src/v3/std/tokenize.dag` (143 lines; Token / TokenKind taxonomy as shared std/ vocabulary)
- `src/v3/compiler/tokenize.dag` (154 lines; tokenizer implementation in `.dag`)
- `src/v3/compiler/src/tokenize_generated.rs` (362 lines; AUTO-GENERATED via `regen_tokenize` codegen-driver from `tokenize.dag`)

PB-2 migration is FURTHER ALONG than other pipeline stages BUT NOT complete (per codex BLOCKING #3127 — corrected understanding). Substrate authority exists in `.dag` AND has documented open scaffolds requiring substantive retirement work:

1. **SG-1a tracked scaffold** (`src/v3/compiler/tokenize.dag:16-22`): `regen_tokenize` currently parses raw source text for `dag_keyword_set` / `dag_operators` because those shared-syntax bodies lower as `ValueBody::Unparsed`. Named dissolution trigger: once those bodies lower structurally under `compile_to_dag`, delete the raw-text extractor + derive directly from lowered Dag in same PR. PB-2 Step 4 carries this scaffold-retirement scope.

2. **Character-level under-consumption scaffold** (`tokenize.dag:23-30+`): scan phases slice ASCII/Unicode codepoint space in parallel forms NOT consuming `dsl/std/` character authorities. `StringEscapeSpec` / `LocalPunctSpec.pattern` / `string_literal_delimiter` treated as opaque Strings; hidden Rust character predicates (`byte.is_ascii_digit()` / `byte.is_ascii_lowercase()` etc. at `tokenize_generated.rs:15-22`) emitted into codegen rather than driven by `.dag`. **Named dissolution trigger** (mirror of the SG-1a scaffold's shape, per openai-pro PR #3127 BLOCKING sha d15e1f29 fixed in PR #3138): when (a) scanner classes / string escape / local punctuation consume `dsl/std/unicode.dag` `CharClass` + `char_in_class` authorities structurally — i.e. when `StringEscapeSpec.suffix` retypes to `Char`, `LocalPunctSpec.pattern` to `List<Char>`, `string_literal_delimiter` to `Char`, etc. — AND (b) `tokenize_generated.rs` no longer emits hidden `byte.is_ascii_*` predicates because the codegen-driver reads class-predicate facts structurally from lowered `tokenize.dag`, delete the parallel character-predicate scaffold in the same PR (no Rust-and-`.dag` coexistence per `feedback_paper_shrink_variants`). Same-PR receipt: the deletion lands in the same PR that flips the substrate consumption. Gating substrate prereqs are tracked at §9 Step 4: ROADMAP.md:467 (Character-level under-consumption row) + ROADMAP.md:416 (Class 5 Gap 3 top-level `ValueBody` boundary) + `std.unicode` bootstrap/load-set decision. PB-2 Step 4 carries this scaffold-retirement scope.

3. **Residual hand-Rust**: NOT just the codegen artifact `tokenize_generated.rs`. The actual residual includes (a) `regen_tokenize` codegen-driver logic (per Q1 PB-Bootstrap-Process lane), (b) SG-1a raw-text extractor scaffold, (c) character-predicate scaffold leaking through codegen.

So PB-2 is "substrate-driven by `.dag` AT scanner-state-machine level but NOT at character-predicate level"; the doc's earlier "mostly verification" framing UNDERSTATED the substantive scaffold-retirement work.

**This doc does NOT**:
- Re-author the tokenize substrate (already live in `tokenize.dag`)
- Own retirement of the `regen_tokenize` codegen-driver itself (that's PB-Bootstrap-Process lane scope — codegen-driver retirement is cross-cutting)
- Own implementation of bootstrap-runtime-loop concerns (separate lanes)

**Authority chain**: Director-tier ratification grounds the model; subsequent worker briefs cite this doc; §1.8 PB-2 gate row close-criterion predicate cites this doc as L2.5 authority.

---

## §2 What tokenize IS structurally

Per the live substrate at `src/v3/std/tokenize.dag:1-15`:

**Tokenize is a substrate-driven scanner: `String → List<Token>`, dispatching on byte character class (Whitespace / Digit / IdentStart / IdentContinue) per `tokenize.dag` declarations. Token taxonomy + scanner logic both live in `.dag` substrate; `tokenize_generated.rs` is the codegen artifact from `regen_tokenize`.**

The substrate authority is structured as:
1. **Token taxonomy** in `src/v3/std/tokenize.dag` (shared vocabulary for compiler + future user-space tooling)
2. **Scanner implementation** in `src/v3/compiler/tokenize.dag` (declarative scanner-class + token-kind-recognition tables)
3. **Codegen artifact** in `src/v3/compiler/src/tokenize_generated.rs` (auto-regenerated; do NOT hand-edit)

Per `feedback_lenses_not_passes`: tokenize is a fold over bytes with scanner-class dispatch — substrate-driven, NOT decision engine. The class definitions IN tokenize.dag ARE the rule book; no separate "TokenizationSpec" carrier needed (substrate IS the spec, same pattern as PB-5 infer per PR #3085 §3.2).

**Failure shape**: fail-closed (per `feedback_fail_closed_discipline` + INVARIANTS C-8). Live state at `tokenize_generated.rs:96`: `fn tokenize(...) -> Result<Vec<Token>, Diagnostic>` returns generic `Diagnostic::TokenizerError { message, span, correction }` (per `src/v3/std/diagnostics.dag` Diagnostic carrier). PROPOSED extension (NOT currently live): typed `TokenizeDiagnostic` variants (unterminated string literal / invalid character / numeric literal overflow / etc.) per §4.2 — same shape as PB-3/4/5 per-stage diagnostic extension per PR #3077 §12 Q7 ratification path.

---

## §3 Input types (declared in `.dag` substrate)

### §3.1 Source text + source identity (TWO inputs, not one)

> **Codex PR #3127 BLOCKING (sha 8e682eef) Finding 1 (fixed in PR #3140)**: an earlier draft modeled tokenize as `String → List<Token>` — a single bare-string input. That contradicts the live signature at `src/v3/compiler/src/tokenize_generated.rs:96`:
>
> ```
> pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, Diagnostic>
> ```
>
> The `file` parameter carries the source identity (file path or equivalent identifier). It is load-bearing because `SourceSpan` requires a file/source-id field for byte ranges to be attributable; without it the Token + Diagnostic span fields would have to fabricate a source-id at the pipeline boundary (P3 fail-closed violation). The L2.5 ratifies the live two-input shape rather than the single-string framing.

Tokenize consumes:

- **`source: String`** — UTF-8 source text. Primitive per `src/v3/std/substrate.dag`. No new carrier needed for this input.
- **`file: SourceFileId`** — source identity carrier. The live Rust uses `&str` for the file path; the Step 2 brief should ratify the appropriate `.dag` carrier shape (a `NonEmptyStr` newtype or a richer `SourceFileId` sum if multiple source-class kinds are expected). Either way, the file identity is a structural input — not implicit context — because every `SourceSpan` carries it.

**Lane dependency**: none upstream of tokenize — it's the pipeline entry point. (Source-identity originates from the compile-driver / build system, not from a prior pipeline stage.)

### §3.2 No GrammarSpec / TokenizationSpec — substrate IS the spec

Per `feedback_lenses_not_passes`: tokenize.dag declarations (scanner classes + token-kind tables) ARE the tokenization rule-book. No separate "TokenizationSpec" carrier needed (same pattern as PB-5 infer's no-InferenceSpec per PR #3085 §3.2).

Distinct from PB-3 parse which has explicit GrammarSpec per Decision 3.B (b) compile-time tables: parse's grammar tables are richer (6 SG-2c table-families covering precedence / dispatch / bracket-roles / etc.); tokenize's scanner is simpler (byte → character class → token kind) and the rule book is implicit in the substrate file structure.

---

## §4 Output types (declared in `.dag` substrate)

### §4.1 `List<Token>` (typed-state output)

Per `src/v3/std/tokenize.dag` (live; Token + TokenKind closed-axis sum):
- Token carries `kind: TokenKind` + `span: SourceSpan` (verified at `src/v3/std/tokenize.dag:65-67`; the live carrier is 2 fields only, no optional lexeme on Token itself)
- TokenKind variants carry their own payloads: `Ident(String)`, `IntLit(String)`, `StringLit(String)`, etc. — lexeme-content lives ON the variant, NOT on Token
- TokenKind closed-axis: keywords (KwLet/KwIf/KwThen/...), identifiers (Ident), literals (IntLit/StringLit), operators (Eq/EqEq/Lt/Gt/...), punctuation (LParen/RParen/...), etc.

**Construction-time invariant**: every consumed byte advances the scanner position; every emitted Token carries source-span provenance. No silent byte-skipping; no fabricated tokens.

**Substrate authority**: `src/v3/std/tokenize.dag` (LIVE at HEAD). PB-2 output type is stable.

### §4.2 `TokenizeDiagnostic` (PROPOSED — substrate extension per Decision 2.B / PR #3077 §12 Q7 — NOT yet live)

> **Codex PR #3127 BLOCKING (sha 8e682eef) Finding 2 (fixed in PR #3140)**: the prior §4.2 framing blurred the live carrier with the proposed per-stage carrier. Clarified here:
>
> - **Live carrier at HEAD**: `type Diagnostic { kind: AnyDiagnosticKind, span: SourceSpan, message: String, correction: Correction }` at `src/v3/std/diagnostics.dag:150`, with `type AnyDiagnosticKind = CompilerKind(CompilerDiagnosticKind) | LensInstanceKind(LensInstanceKindWitness)` at `diagnostics.dag:139-142`. Tokenize emits today via `Diagnostic::TokenizerError`-shaped construction sites in `tokenize_generated.rs:96`, all carrying `kind: CompilerKind(CompilerDiagnosticKind::*)`.
> - **`TokenizeDiagnostic`**: PROPOSED per-stage refinement carrier. NOT a live `.dag` declaration. Its substrate-extension path (option (a) carrier-field on `Diagnostic` vs option (b) lane-local sum + mapping into `CompilerDiagnosticKind`) was ratified by **PR #3077 §12 Q7, merged 2026-05-15T00:21:19Z** (DONE — see §15 step 3 + §14 "Surfaces awaiting"). Director-tier per-stage variant authoring is the remaining work; the variants below are a starting set, not the live carrier.

Same cross-stage Decision 2.B framing as PB-3/4/5: per-stage diagnostic variants attach via the path PR #3077 §12 Q7 ratified (carrier-field vs lane-local-sum; live status DONE).

```
// Typed reference carrier (cross-stage discipline per openai-pro
// PR #3077 BLOCKING + INVARIANTS P2/P3) — bound to the live .dag
// character-class authority `dsl/std/unicode.dag:62`:
//     type CharClass = Whitespace | Digit | IdentStart | IdentContinue
// consumed at `src/v3/compiler/tokenize.dag:103`:
//     data ascii_scan_order: List<CharClass> = [Whitespace, Digit, IdentStart, IdentContinue]
// There is NO `ScannerCharClass` declaration in any .dag file; an
// earlier draft of this section invented that name from the generated
// Rust enum spelling (codex PR #3138 BLOCKING sha f08b9525 Finding 1
// fixed in PR #3138). TokenizeDiagnostic references the live `CharClass`
// directly.

// 🟡 SCAFFOLD coproduct at PROPOSED stage. Per modeling-discipline Practice 4
// (Coproduct dissolution) + `feedback_coproduct_dissolution` + codex PR #3138
// BLOCKING sha 887c69671 Finding 2 (fixed in PR #3138):
//
// **Dissolution trigger**: when Step 2 worker brief enumerates the full
// variant set against `src/v3/compiler/src/tokenize_generated.rs:96`
// `Diagnostic::*` construction sites (tokenize-stage only — not cross-stage),
// promote to 🟢 TERMINAL at the per-stage diagnostic-variant scope. The
// listed variants below are a starting set; Step 2 brief authoring closes
// the enumeration. PR #3077 §12 Q7 ratification (DONE 2026-05-15T00:21:19Z —
// see §15 step 3 + §14 "Surfaces awaiting") determines whether this stays
// per-stage sum (option b) or extends shared Diagnostic (option a).
//
// **Anti-bridge**: per Q6.5 anti-bridge invariant at diagnostics.dag:135-141,
// TokenizeDiagnostic does NOT collapse into CompilerDiagnosticKind without
// substrate-extension ratification.
type TokenizeDiagnostic
  = UnterminatedStringLiteral { opener_span: SourceSpan }
  | InvalidCharacter { byte: Nat, span: SourceSpan, expected_class: CharClass }
  | NumericLiteralOverflow { lexeme: NonEmptyStr, span: SourceSpan }
  | (additional variants per Step 2 worker brief authoring against tokenize_generated.rs)
```

**Lane dependency**: PR #3077 §12 Q7 ratification — DONE 2026-05-15T00:21:19Z when PR #3077 (PB-4 lower L2.5) merged carrying the cross-stage Decision 2.B Q7 ratification (see §15 step 3 below); Director-tier per-stage variant authoring is the remaining lane work.

### §4.3 Tokenize output is `Result<List<Token>, TokenizeDiagnostic>` — NO `TokenizedSource` extension

> **Codex PR #3127 BLOCKING (sha d15e1f29) revision**: an earlier draft of this section proposed a `TokenizedSource { tokens, diagnostics }` extension parallel to PR #3126's earlier-draft `SurfaceModule { items, diagnostics }`. Both proposals are REJECTED for the same reason: tokenize and parse sit in the **fail-fast output domain**, not the structural-output domain. The cross-stage discriminator (re-stated in PR #3138's §4.3 reframe of the parse doc) places tokenize alongside PB-3 parse + PB-6 emit: a partial token list with a corrupt token in the middle is not a valid downstream input for parse, so the failure couples via `Result`, not into the structural carrier. No `TokenizedSource` carrier is being authored; §12 Q6 (TokenizedSource carrier shape) is resolved as "rejected — Result-sum per cross-stage discriminator".

**Live state matches**: `tokenize_generated.rs:96` already returns `pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, Diagnostic>`. The L2.5 ratifies this shape rather than replacing it.

Step 2 signature: `fn tokenize(source: String) -> Result<List<Token>, TokenizeDiagnostic>` — `List<Token>` is the Ok-branch payload, `TokenizeDiagnostic` (§4.2 substrate extension) is the Err-branch variant. This is the **single canonical boundary carrier** between PB-2 tokenize and PB-3 parse: `List<Token>` on the Ok branch, no parallel `TokenizedSource`.

**Cross-stage discriminator** (consistent with PR #3138 parse L2.5 §4.3):
- **Result-sum** (PB-2 tokenize + PB-3 parse + PB-6 emit): fail-fast output domain — partial token/parse/emit output is not a valid intermediate.
- **Typed-state-with-coupled-diagnostics** (PB-4 lower + PB-5 infer): structural output domain — Unresolved ports / pre-inferred Dag are valid intermediates consumed downstream.

---

## §5 Substrate-driven tokenization (the core)

Per `tokenize_generated.rs:6-25` + `tokenize.dag` declarations:

### §5.1 Byte → `CharClass` dispatch

Each byte maps to one of the four `CharClass` variants declared at `dsl/std/unicode.dag:62` (`type CharClass = Whitespace | Digit | IdentStart | IdentContinue`) and consumed at `src/v3/compiler/tokenize.dag:103` (`data ascii_scan_order: List<CharClass>`):
- `Whitespace` (tab/newline/form-feed/carriage-return/space)
- `Digit` (ascii digit)
- `IdentStart` (ascii letter or underscore)
- `IdentContinue` (alphanumeric or underscore)

This dispatch is purely structural — a byte-class lookup function. Live at `tokenize_generated.rs:13-25` (regenerated from tokenize.dag). The substrate authority is the `CharClass` declaration in `unicode.dag`, NOT a `ScannerCharClass` (no such .dag type exists; that name appears only in the generated Rust enum spelling).

### §5.2 `CharClass` → token-recognition state machine

Per `tokenize.dag` declarations: byte sequences matching specific patterns produce specific TokenKind variants. The state machine is small + declarative:
- Whitespace sequences: skipped (no token emitted)
- Digit sequences: collected → `IntLit(decimal_string)`
- IdentStart + IdentContinue sequences: collected → either keyword (per closed-axis keyword table) or `Ident(string)`
- String delimiters: collected with escape handling → `StringLit(string)`
- Operator symbols: matched against closed-axis operator-symbol table → operator TokenKind variants
- Punctuation symbols: matched → punctuation TokenKind variants

### §5.3 Mechanical dispatch via closed-axis enums

Walker dispatch is **mechanical**: match on byte class → look up the recognition rule → emit Token or continue scanning. **No conditional logic encoded in tokenize body** beyond the state-machine structural transitions; the recognition tables (keyword set / operator-symbol set / etc.) are closed-axis enums in tokenize.dag.

---

## §6 Substrate prereqs (per-Gap-tier anchored)

| Prereq | Substrate authority | Gap-tier lane | Status at HEAD (as of 2026-05-14) |
|---|---|---|---|
| Token + TokenKind taxonomy | `src/v3/std/tokenize.dag` (Token + TokenKind closed-axis sum) | PB-Substrate | LIVE at HEAD; complete |
| Tokenizer implementation | `src/v3/compiler/tokenize.dag` (scanner-class + recognition tables) | PB-Substrate | LIVE at HEAD; 154 lines |
| Codegen pipeline | `regen_tokenize` codegen-driver → `tokenize_generated.rs` | PB-Bootstrap-Process lane | LIVE at HEAD; codegen-driver retirement is PB-Bootstrap-Process scope (NOT PB-2 scope) |
| TokenizeDiagnostic substrate extension | extension of `src/v3/std/diagnostics.dag:150` per PR #3077 §12 Q7 | PB-Substrate + Director-tier per-stage authoring | Carrier LIVE; per-stage variant authoring NEW per Q7 ratification |

**Critical observation**: PB-2 tokenize has LIGHTER prereq surface than other pipeline stages — substrate-driven at scanner-state-machine level. BUT substantive residual work remains (per codex BLOCKING #3127 corrected): SG-1a raw-text-extractor scaffold + character-level under-consumption scaffold + regen_tokenize codegen-driver retirement (per Q1 PB-Bootstrap-Process). Earlier "mostly verification" framing was understated.

---

## §7 Cross-stage coordination

### §7.1 Upstream dependencies

None. Tokenize is the pipeline entry point (consumes raw source text).

### §7.2 Downstream consumers

PB-3 parse consumes `List<Token>` from tokenize (per PR #3126 §3.1). The Token carrier shape is stable (live at tokenize.dag); PB-3 parse migration is independent of PB-2 tokenize migration status.

Per Decision 2.B / PR #3077 §12 Q7: tokenize diagnostics are discriminable by source per the ratified extension path.

### §7.3 Sibling-stage coordination

Cross-stage discipline (Ok-branch propagation; Err branches are stage-terminal fail-fast per §4.3 Result-sum discriminator): **tokenize** → `List<Token>` (Ok of `Result<List<Token>, TokenizeDiagnostic>`) → parse → `SurfaceModule` (Ok of `Result<SurfaceModule, ParseDiagnostic>`) → lower → `PreInferDag` (typed-state with coupled diagnostics; structural output domain) → infer → `InferredDag` (typed-state) → emit → `Result<EmittedArtifact, EmissionDiagnostic>` (fail-fast like tokenize/parse). The chain shows the success-path data flow; on any stage's Err branch the pipeline aborts at that stage (no partial-output propagation across boundaries).

Tokenize is the FOUNDATION; its output type stability affects every downstream stage. Since `tokenize.dag` is already the live substrate authority, this stability is preserved across PB-2 migration.

---

## §8 Two shapes of omni-emission — N/A for tokenize

Tokenize is target-agnostic byte-scanning; Shape A/B disambiguation lives at emit stage. PB-2 has no Shape A/B framing.

---

## §9 SELF_HOSTING.md §2.2 4-step applied to PB-2 tokenize

**The 4-step discipline applies UNUSUALLY for PB-2 because substrate is already live**:

| Step | Deliverable | Owner | Substrate |
|---|---|---|---|
| **Step 1: Model review** | THIS DOC | Director (zesty-bear-812) | docs/design-tokenize-stage-l25-model.md (this doc) |
| **Step 2: Pipeline slot** | `fn tokenize(source: String) -> Result<List<Token>, TokenizeDiagnostic>` declared in `src/v3/compiler/pipeline.dag` (per dsl/gunbc/compiler.dag:24 — internal pipeline lives in pipeline.dag, NOT generic compiler.dag) with `ExternalRealization` body. **Single canonical boundary carrier**: `List<Token>` on the Ok branch — no parallel `TokenizedSource` wrapper, no `diagnostics` field added to tokenize output (§4.3 Result-sum disposition; §12 Q6 resolved REJECTED). Matches live `tokenize_generated.rs:96` `Result<Vec<Token>, Diagnostic>` shape; only refinement is per-stage `TokenizeDiagnostic` variant typing (§4.2). Note: `tokenize.dag` exists as scanner-state-machine substrate, BUT residual hand-Rust includes (a) `regen_tokenize` codegen-driver logic (P5 deferral receipt: `docs/design-pure-bootstrap-zero.md:116` PB-Bootstrap-Process lane + ROADMAP.md:467 character-level row), (b) SG-1a raw-text-extractor scaffold for `dag_keyword_set`/`dag_operators` per `tokenize.dag:16-22`, (c) character-predicate scaffold (`byte.is_ascii_digit()` etc. at `tokenize_generated.rs:15-22`) leaking through codegen (P5 receipt: ROADMAP.md:467 phase-2 retype, gated on Class 5 Gap 3 + std.unicode bootstrap/load-set decision). Step 4 carries scaffold-retirement scope, NOT just codegen-artifact retirement. | R3 Substrate Mgr (warm-wolf-698) — worker dispatched against Director-authored Step 2 brief | pipeline.dag refinement |
| **Step 3: Verify substrate completeness** | Audit `src/v3/compiler/tokenize.dag` vs `tokenize_generated.rs` — confirm `.dag` is the complete authority (no hand-Rust logic in `tokenize_generated.rs` beyond mechanical codegen artifacts); identify any residual hand-Rust scaffolding that needs retirement. Per `feedback_paper_shrink_variants` discipline: verify tokenize.dag is NOT V1 template-relocation (hand-Rust scanner logic relocated to `.dag` text without substrate-substance growth). | R3 Substrate Mgr — worker dispatched against Director-authored Step 3 brief | tokenize.dag audit |
| **Step 4: Retire residual hand-Rust + codegen-driver decoupling** | If Step 3 reveals residual hand-Rust scaffolding: retire it. `regen_tokenize` codegen-driver retirement is **NOT** in PB-2 scope — it routes to the PB-Bootstrap-Process lane at `docs/design-pure-bootstrap-zero.md:116` (author `bootstrap.dag` + generated trampoline; sized M; verification gates per `docs/design-pure-bootstrap-zero.md:118-123`). Phase-2 char-class retype (`Char` / `List<Char>` / `CharClass`) routes to ROADMAP.md:467 ("Character-level under-consumption in tokenize + syntax authorities"), gated on **Class 5 Gap 3** (top-level `ValueBody` boundary at ROADMAP.md:416) plus the std.unicode bootstrap/load-set decision. Overall non-test census reaches 0 per ROADMAP.md:53 T-PB-A. PB-2 Step 4 retires only what closes in-lane; the named cross-lane receipts above are the P5 deferral. EXPECTED_HAND_AUTHORED_NON_TEST shrinks by whatever residual lands. | R3 Substrate Mgr + coordination with PB-Bootstrap-Process lane | residual hand-Rust deletion |

**Critical**: PB-2's Step 3 is VERIFY not AUTHOR (substrate already exists). PB-2's Step 4 is HANDOFF/RETIRE not PORT (no porting needed beyond what's already in tokenize.dag).

Per `feedback_paper_shrink_variants` discipline applied at Step 3 audit: verify tokenize.dag is NOT V1 template-relocation. The discriminator: does tokenize.dag declare scanner-class + recognition tables as SUBSTRATE DATA (declarative tables), OR does it carry hand-Rust scanner code in text form? The former is substantive; the latter is paper-shrink. Per the live tokenize.dag content I grep'd (substrate-style declarations), it appears substantive — but Step 3 audit confirms formally.

---

## §10 Determinism invariant preservation

Tokenization is inherently deterministic (byte stream → unique token stream per scanner state machine). Already structurally enforced via the `.dag` substrate's declarative scanner-class definitions. No HashMap iteration concerns.

---

## §11 Tokenization invariants (cross-cutting)

Per `feedback_fail_closed_discipline` C-8 + scanner hygiene:

- **Every byte consumed**: no silent byte-skipping; whitespace explicitly skipped per scanner state
- **Source-span provenance**: every Token + every TokenizeDiagnostic carries `SourceSpan`
- **Lookahead bounded**: scanner uses constant-bounded lookahead (no unbounded backtracking)
- **Anti-bridge invariant** (per `feedback_no_textual_enforcement_bridges`): tokenize.dag declarations are SINGLE authority; no fallback hand-Rust heuristics in tokenize_generated.rs

---

## §12 Open design questions (operator/PM ratification)

### Q1: codegen-driver retirement scope

`regen_tokenize` codegen-driver lives at the build-system layer. Per `docs/design-pure-bootstrap-zero.md` PB-Bootstrap-Process lane: codegen-driver retirement is cross-cutting across all pipeline stages (tokenize / parse / etc.). Question: does PB-2 retire `regen_tokenize` specifically, OR does PB-Bootstrap-Process retire all codegen-drivers in one cross-stage PR?

**Director-recommend: PB-Bootstrap-Process retires all codegen-drivers** as one coherent lane — keeping codegen-driver retirement to a single substrate-cross-cutting authority avoids per-stage paper-shrink risk (`feedback_paper_shrink_variants`). PB-2 confirms tokenize.dag is the substantive authority + flags `regen_tokenize` as retire-target, but the retirement PR lives in PB-Bootstrap-Process. Operator/PM ratification.

### Q2: Step 3 audit scope — paper-shrink check

Per `feedback_paper_shrink_variants` discipline + my Director-side history of missing template-relocation in cycles 3/4/5/6: Step 3 audit must explicitly check whether `tokenize.dag` is substantive substrate (declarative tables + scanner-class definitions) or paper-shrink (hand-Rust scanner code in `.dag` text).

**Director-recommend**: Step 3 brief explicitly enumerates the audit dimensions:
1. Scanner-class definitions are declarative byte-pattern membership (not hand-coded match-arms in text form)
2. Recognition tables are closed-axis enums (not String-keyed maps)
3. State machine transitions are structural (not embedded code)
4. No `pub mod tokenize { ... }` absorption into adjacent files (V2 module-relocation check)

If audit fails: tokenize.dag may itself need refactoring before retirement-of-residual-Rust is meaningful. Operator/PM ratification on audit criteria.

### Q3: TokenizeDiagnostic variant exhaustiveness

Step 2 worker brief enumerates the full variant set by grepping `tokenize_generated.rs` for Diagnostic construction sites. Same discipline as PB-3 §12 Q3.

**Director-recommend**: defer to Step 2 worker brief authoring (consistent with PB-3 / PB-4 / PB-5 approach).

### Q4: Substrate completeness criterion

What's the formal predicate for "tokenize substrate is complete + residual hand-Rust can retire"?

**Director-recommend**: predicate spans BOTH the codegen artifact AND the codegen-driver boundary (per codex BLOCKING PR #3127 — earlier scoping to tokenize_generated.rs alone missed the regen_tokenize logic + .dag authority boundary).

Predicate = `cargo test --release -p v3-compiler --test integration tokenize_substrate_authority` shows:
- (a) `tokenize.dag` content unchanged but codegen regenerates `tokenize_generated.rs` byte-identically (idempotent codegen)
- (b) `tokenize_generated.rs` contains NO hand-edit zones (all body is codegen-driver-emitted)
- (c) No fallback Rust scanner logic outside the codegen artifact
- (d) **`regen_tokenize` codegen-driver itself contains NO scanner-logic decisions** — it reads `tokenize.dag` declaratively and emits Rust mechanically. If `regen_tokenize` carries scanner logic (rather than just template-rendering substrate facts), the substrate isn't actually complete: the driver IS hand-Rust scanner logic in disguise.
- (e) **Or explicit ROADMAP.md deferral row** naming `regen_tokenize` codegen-driver retirement scope as PB-Bootstrap-Process lane (per Q1); deferral receipt is PB-2 → PB-Bootstrap-Process handoff at the codegen-driver authority boundary.

If (a)(b)(c)(d) hold, substrate is complete + tokenize_generated.rs can retire (replaced by Evaluator-loaded `.dag` at runtime per PB-Runtime). If (d) is violated but (e) is named, partial-completion with explicit deferral is acceptable (per `feedback_paper_shrink_variants` P5 receipt discipline).

### Q5: PB-3 parse landing dependency

PB-3 parse's input is List<Token> from tokenize. **Does PB-2 tokenize migration block on PB-3 parse migration completing? Reverse?**

**Director-recommend: NO bidirectional blocking** — Token carrier shape is stable across both migrations. Same independence pattern as PB-4 lower vs PB-3 parse per PR #3077 §12 Q6 + PB-5 infer vs PB-4 lower per PR #3085 §12 Q6.

### Q6: TokenizedSource carrier shape — RESOLVED REJECTED (codex PR #3127 BLOCKING)

**Disposition**: REJECTED. No `TokenizedSource` carrier is being authored.

**Why**: this Q was opened on the (incorrect) premise that tokenize, like PB-4 lower / PB-5 infer, should couple diagnostics structurally into its output carrier. The cross-stage discriminator (§4.3, mirrored in PR #3138 parse-L2.5 §4.3) classifies tokenize alongside PB-3 parse + PB-6 emit in the **fail-fast output domain**: a partial token list with a corrupt token in the middle is not a valid intermediate for parse, so the failure couples via `Result`, not into the structural carrier. Live shape at `tokenize_generated.rs:96` already matches this disposition (`Result<Vec<Token>, Diagnostic>`); the L2.5 ratifies the live shape rather than proposing an extension.

**Step 2 boundary carrier** is therefore unique: `Result<List<Token>, TokenizeDiagnostic>`, with `List<Token>` on the Ok branch and no parallel `TokenizedSource`. No PM/operator ratification is needed beyond confirming the discriminator, which is also load-bearing in PR #3138.

---

## §13 Non-goals

- **`.dag` implementation of tokenizer** — already exists at `src/v3/compiler/tokenize.dag` (LIVE)
- **`regen_tokenize` codegen-driver retirement** — PB-Bootstrap-Process lane scope
- **Test corpus design + parity-test harness implementation** — Step 4 work
- **Bootstrap-runtime-loop concerns** — separate lanes
- **PB-3 parse migration** — separate L2.5 doc (PR #3126)
- **Shape A/B emission** — emit's concern, not tokenize

---

## §14 Acceptance criteria for this L2.5 model

This doc lands on main when:

1. ✅ Input types declared structurally with substrate paths (§3)
2. ✅ Output types declared with live substrate citations (§4)
3. ✅ Substrate-driven tokenization composed without decision logic (§5 — per `feedback_lenses_not_passes`)
4. ✅ All substrate prereqs named with Gap-tier / Mgr-lane anchors (§6)
5. ✅ Cross-stage dependencies explicit (§7)
6. ✅ N/A — Shape A/B framing irrelevant for tokenize (§8)
7. ✅ SELF_HOSTING.md §2.2 4-step applied (§9) — with PB-2-specific adjustments (Step 3 = verify, Step 4 = handoff/retire)
8. ✅ Determinism preservation discipline (§10)
9. ✅ Tokenization invariants explicit (§11)
10. ✅ Open design questions enumerated for operator/PM ratification (§12)
11. ⏳ Operator/PM ratification on §12 Q1–Q5 only (Q6 resolved REJECTED in §12 per codex PR #3127 BLOCKING + cross-stage Result-sum discriminator — no ratification needed; see §14 "Surfaces awaiting" + §15 step 1 for the same scoping)

Post-ratification: this doc becomes substrate authority for Step 2/3/4 worker brief authoring + §1.8 PB-2 gate row close-criterion predicate.

---

## §15 Authoring sequence post-ratification

1. **Operator / PM-delegate ratifies §12 Q1–Q5** only (per 2026-05-14 directive; §12 Q6 was resolved REJECTED in-doc per codex PR #3127 BLOCKING + cross-stage Result-sum discriminator and needs no separate ratification cycle)
2. **PM amends close plan + §1.8** to route through PB-X lanes + cite this doc as PB-2 L2.5 substrate
3. **PR #3077 §12 Q7 ratifies** (cross-stage Decision 2.B extension path; affects TokenizeDiagnostic shape). **DONE 2026-05-15T00:21:19Z** when PR #3077 (PB-4 lower L2.5) merged; Q7 ratification carried in that merge. The §4.2 + §7.2 "TokenizeDiagnostic extension waits on Q7" gate is therefore now satisfied — TokenizeDiagnostic per-stage variant authoring is genuinely unblocked, not just procedurally listed as the next step. Companion to PR #3138 design-parse-stage-l25-model.md §15 step 4 Q7-DONE annotation.
4. **Director authors PB-2 Step 2 worker brief** (pipeline-slot ExternalRealization PR scope; trivial since substrate already lives in tokenize.dag)
5. **R3 Substrate Mgr (warm-wolf-698)** dispatches Step 2 worker
6. **Director ratifies Step 2 PR + admin-merges**
7. **Director authors PB-2 Step 3 worker brief** (substrate-completeness audit per §12 Q2)
8. **R3 Substrate Mgr** dispatches Step 3 worker
9. **Director ratifies Step 3 PR**, admin-merges → audit findings inform Step 4 scope
10. **Director authors PB-2 Step 4 worker brief** (residual hand-Rust retirement; coordinates with PB-Bootstrap-Process lane per Q1 if codegen-driver retirement is bundled)
11. **R3 Substrate Mgr** dispatches Step 4 worker
12. **Director ratifies Step 4 PR**, admin-merges → PB-2 gate row CLOSES per §1.8

---

## §16 Cross-references

**Primary authority**:
- `src/v3/SELF_HOSTING.md` §2.2 (4-step migration discipline)
- `docs/design-pure-bootstrap-zero.md` (PB-X lane framing; PB-Bootstrap-Process lane for codegen-driver retirement)
- `docs/substrate-reflection-design.md` §12.6 (migration order)
- `docs/design-emit-stage-l25-model.md` (PB-6 PR #3066 — sets L2.5 template)
- `docs/design-lower-stage-l25-model.md` (PB-4 PR #3077 — cross-stage diagnostic pattern)
- `docs/design-infer-stage-l25-model.md` (PB-5 PR #3085 — substrate-driven dispatch precedent)
- `docs/design-parse-stage-l25-model.md` (PB-3 PR #3126 — sibling pipeline-stage L2.5)

**Live substrate referenced**:
- `src/v3/std/tokenize.dag` (Token + TokenKind taxonomy; LIVE; 143 lines)
- `src/v3/compiler/tokenize.dag` (tokenizer implementation; LIVE; 154 lines)
- `src/v3/compiler/src/tokenize_generated.rs` (AUTO-GENERATED codegen artifact; 362 lines)
- `src/v3/std/diagnostics.dag:150` (Diagnostic carrier — extends with TokenizeDiagnostic per PR #3077 §12 Q7 ratification)

**Memory disciplines applied**:
- `feedback_lenses_not_passes` (tokenize = substrate-driven byte fold, NOT decision engine)
- `feedback_fail_closed_discipline` C-8 (tokenize returns `Result<List<Token>, TokenizeDiagnostic>` — fail-closed Result-sum like parse + emit; the Err branch is the only carrier for diagnostics, no structural coupling into `List<Token>`)
- `feedback_state_space_vs_behavioral_invariants` (tokenize output is `Result<List<Token>, TokenizeDiagnostic>` — the type rules out partial-tokenize states by construction; `List<Token>` itself carries no diagnostic field per §4.3)
- `feedback_target_agnostic_ir` (tokenize output carries no target-specific facts)
- `feedback_paper_shrink_variants` (Step 3 audit explicitly checks substrate vs paper-shrink)
- `feedback_anchor_mgr_lane_synthesis_on_gap_tier_not_session_id` (Gap-tier anchors)
- `feedback_no_textual_enforcement_bridges` (anti-bridge: tokenize.dag is single authority)
- `feedback_grep_carrier_semantic_before_ratification` (4-axis grep applied at authoring time)

**Surfaces awaiting**:
- Operator/PM ratification on §12 Q1–Q5 (Q6 resolved REJECTED in-doc per codex PR #3127 BLOCKING — no ratification needed)
- ~~PR #3077 §12 Q7 ratification (cross-stage Decision 2.B extension path)~~ — DONE 2026-05-15T00:21:19Z (PR #3077 merged carrying Q7 ratification; see §15 step 3)
- PM Phase 2 close plan + §1.8 amendments citing this doc
- Coordination with PB-Bootstrap-Process lane for codegen-driver retirement per Q1
