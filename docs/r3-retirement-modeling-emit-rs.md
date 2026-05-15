# `emit.rs` — Substrate-Driven Retirement Modeling

**Status**: DRAFT — first modeling doc per operator 2026-05-15 directive ("for each of these files - find a layer to move into .dag - and lets do a very good job on the modeling asap"). Format under operator review.

**Source file**: `src/v3/compiler/src/emit.rs` (3,992 lines, ~160 KB)

**Sibling files in same retirement bundle** (deferred to later docs):
- `src/v3/compiler/src/emit/rust_target.rs` (6,845 lines)
- `src/v3/compiler/src/emit/python_target.rs` (2,232 lines)
- `src/v3/compiler/src/emit/collection_ops_method_contract.rs` (183 lines)
- `src/v3/compiler/src/emit_rust.rs` (2 lines — re-export only)
- `src/v3/compiler/src/emit_rust_bin_shim.rs` (92 lines)
- `src/v3/compiler/src/emit_rust_roundtrip_fixtures.rs` (175 lines)

---

## §1 What `emit.rs` does

Owns the cross-target dispatch layer for code emission. Routes `(dag, target, mode)` → per-target `emit_*_with_mode` call. Also contains:
1. **Dispatcher** (~80 lines: `emit_with_mode`, `emit`, `emit_module`, per-target convenience wrappers `emit_rust_text`, `emit_python_text`, `emit_go_text`)
2. **Inline Go emitter** (~1,300 lines: `emit_go_with_mode` + helpers; not yet extracted to a `go_target.rs` sibling)
3. **Cross-target shared scaffolding** (~150 lines: `VariantPayloadBinding<T>`, `VariantPayloadFieldAccessRuleBinding`, and the `parse_*_syntax` family that reads spec-`.dag` field values into typed-in-Rust forms)
4. **Per-construct syntax-spec readers** (~1,400 lines: `parse_pattern_binding_rule`, `parse_variant_payload_field_access_rule`, `parse_statement_syntax`, `parse_expression_syntax`, ... — read `src/v3/spec/{rust,python,go}.dag` declaration bodies via typed `DeclarationRef` field access)

The file is **target-monolithic** today: per-target emission for Rust/Python lives in sibling modules but the Go body sits inline. Per the file's own §1 doc, the Go body extraction is named as planned ("α §10 Stages 1e.2–1e.4 will move logic out of those files into generic walker helpers here").

---

## §2 Reference pattern: `tokenize.dag` (the proven `.dag`-substrate-drives-retirement example)

`src/v3/compiler/tokenize.dag` (154 lines) is the PROOF OF CONCEPT that hand-Rust pipeline-stage retirement works end-to-end. What makes it work:

**The compositional pattern**:
- **Substrate carriers in `src/v3/std/tokenize.dag` + `src/v3/std/unicode.dag`** (`Token`, `TokenKind`, `KeywordTokenKind`, `PunctTokenKind`, `LocalPunctSpec`, `StringEscapeSpec`, `CharClass`) — the SHARED vocabulary
- **Data values for every parameterizable fact** in `tokenize.dag`:
  - `data ascii_scan_order: List<CharClass> = [Whitespace, Digit, IdentStart, IdentContinue]` — the scan order
  - `data minus_infix_only_after_token_kinds: List<String> = [...]` — context-sensitive rule
  - `data string_escape_*: StringEscapeSpec = {...}` — per-escape rows
  - `data local_punct_*: LocalPunctSpec = {...}` — per-punctuation rows
  - `data diagnostic_*: String = "..."` — fixed-text diagnostic bodies
  - `data line_comment_prefix / string_literal_delimiter: String` — single-string scanner facts
- **NO imperative logic in `tokenize.dag`**. Behavior lives in `regen_tokenize.rs` (1,186 lines, hand-Rust codegen driver) which READS the `.dag` data via structural traversal + emits the corresponding match arms into `tokenize_generated.rs` (362 lines, generated Rust).
- **`tokenize.rs` (hand-Rust) does not exist** — fully retired. The runtime uses `tokenize_generated.rs` directly.

**Why this works**: the tokenizer's behavior is **tabular**. Every "rule" (keyword X tokenizes as TokenKind Y, character class Z is scanned before W, escape sequence `\n` produces codepoint 10) is a per-row fact. `.dag` is naturally good at expressing tabular facts as `data` values. The codegen driver reads the table + emits match arms — straightforward.

**The translation Rust → `.dag` was clear for tokenize because tokenize is data-driven**. For emit, we need to identify which LAYERS are similarly tabular vs which layers are logic-driven.

---

## §3 Layer identification for `emit.rs`

Emit's behavior is a MIX of tabular and logic-driven. The natural decomposition into layers, ordered by tractability (most tabular first):

### Layer 1 — Cross-target dispatcher (TABULAR — proposed for first migration)

The dispatcher itself: `emit_with_mode(dag, target, mode)` matches on `target: EmitTarget` and calls one of `emit_go_with_mode` / `rust_target::emit_rust_with_mode` / `python_target::emit_python_with_mode`. Plus 6 convenience wrappers (`emit_rust_text`, `emit_rust_module_text`, etc.) that unwrap target-specific error variants.

**Why this is tractable**: pure tabular — `EmitTarget` variant → `(per_target_fn, per_target_error_unwrap)`. Same pattern as tokenize's `local_punct_*: LocalPunctSpec` rows. Each row is data.

**Scope**: ~80 lines of Rust → 1 substrate carrier + 3 data rows + 1 codegen driver extension.

### Layer 2 — `VariantPayloadBinding` shared scaffolding (SUBSTRATE-MIRROR — Phase 4 of overall plan)

The `VariantPayloadBinding<T>` Rust struct (50 lines) is a parallel-representation of `std.clean_emission.VariantPayloadFieldAccessRule` (already in `src/v3/std/clean_emission.dag`). The file's own line 14 confirms: `/// Shared emitter-side mirror of std.clean_emission.VariantPayloadFieldAccessRule`.

**Why this is tractable**: it's substrate-mirror duplication. The `.dag` carrier already exists. The Rust struct should be GENERATED from it, not hand-authored. Per `feedback_isomorphism_or_generation_for_mirrors`.

**Scope**: 50 lines of hand-Rust → codegen driver extension to mirror `clean_emission.dag` types into Rust.

### Layer 3 — Per-construct syntax-spec readers (TABULAR — reads `.dag`, returns typed Rust forms)

The `parse_*_syntax` family (~1,400 lines): `parse_statement_syntax`, `parse_expression_syntax`, `parse_function_syntax`, `parse_type_application_syntax`, `parse_type_definition_syntax`, `parse_value_construction_syntax`, `parse_literal_syntax`. Each function reads a declaration body in `src/v3/spec/{rust,python,go}.dag` via typed `DeclarationRef` field access + returns a structured Rust value (typically a struct with String / typed-enum fields).

**Why this is tractable**: pure structural reads. The behavior is "extract field X from declaration Y of type Z." `.dag` already supports field access on data values. Could be modeled as pure `.dag` functions returning typed records.

**Scope**: ~1,400 lines → set of `.dag` `fn` declarations + codegen extension.

### Layer 4 — Inline Go emitter (`emit_go_with_mode` + helpers; ~1,300 lines)

The Go-specific emission body. Currently inline in `emit.rs`; per the file's own §1, planned extraction to a sibling `go_target.rs` is named. Pre-modeling step is the extraction (refactor, not retirement). After extraction, Go gets the same layered treatment as Rust/Python.

**Why this is harder**: per-construct emit logic (BindNode → Go let-equivalent, BranchNode → Go switch, etc.) is the structural-fold-over-Behavior layer that ALL three target modules implement. This is Layer 5 territory.

### Layer 5 — Per-construct emit functions in target modules

The bulk: `rust_target.rs` (6,845 lines), `python_target.rs` (2,232 lines), and post-extraction `go_target.rs` (~1,300 lines). These contain the per-Behavior-variant emission logic (how to render a `BindNode`, `BranchNode`, `LoopNode` as target source). This is **logic-driven**: each construct's emission is a structural fold over its sub-DAG with target-specific syntactic glue.

**Why this is hardest**: requires `.dag` to express:
- Pure functions pattern-matching `Behavior` variants
- Mutual recursion (e.g., rendering a `BindNode` triggers rendering its body, which may contain another `BindNode`)
- Possibly string-building monoid (concatenating rendered fragments)

The recent gate work on `T-E-P-Producer-Broadening` (e_p_call_pattern_lookup, e_p_sub_value_relation_per_call) + `T-Lens-Self-Application` suggests these features are landing, but I haven't verified that the full set needed for emit-as-`.dag` is structurally complete.

**Sibling files in this layer**: `rust_target.rs`, `python_target.rs`, `collection_ops_method_contract.rs`. Each gets its own modeling doc (this doc is `emit.rs`-only).

### Layer 6 — Whole-stage retirement

After Layers 1-5 are migrated, `emit.rs` is empty (or just module re-exports). Retirement is mechanical deletion. This is the meta-circular bootstrap convergence point per SELF_HOSTING.md §7.

---

## §4 Compositional `.dag` model for Layer 1 (dispatcher)

### §4.1 Substrate carriers needed

**Two new carriers in `src/v3/std/emit_model.dag`** (extending the existing 534-line file; no new file needed):

```dag
// New 🟢 TERMINAL carrier — closed sum of supported emission targets.
// Replaces the hand-Rust `pub enum EmitTarget { Go, Rust, Python }` at
// src/v3/compiler/src/emit.rs:1060. Discriminator authority for the
// emit dispatcher.
//
// Coproduct classification per `feedback_coproduct_dissolution`:
//   Pattern = STRUCTURE (each variant identifies a distinct target
//   runtime). Dissolution attempts walked: (1) String tag REJECTED
//   (opaque-strings-attract-heuristics); (2) Refinement<Target>
//   REJECTED (no refinement relation between Go/Rust/Python); (3)
//   Single Target type with field discriminator REJECTED (admits
//   illegal target/spec mismatches). Variant per supported runtime
//   is structurally appropriate.
// TERMINAL: new targets land as new variants; vocabulary stable.
type EmitTarget = Go | Rust | Python

// New 🟢 TERMINAL carrier — emit mode (full program vs library module).
// Replaces hand-Rust `pub enum EmitMode { Program, Module }` at
// src/v3/compiler/src/emit.rs:1067. Distinguishes whether the emitted
// source is a top-level executable program or a library module.
type EmitMode = Program | Module

// New 🟢 TERMINAL carrier — successful emission output. Replaces
// hand-Rust `pub struct EmittedSource { text, target, mode }` at
// src/v3/compiler/src/emit.rs:1073.
type EmittedSource {
  text: String
  target: EmitTarget
  mode: EmitMode
}

// New 🟡 SCAFFOLD carrier — dispatch row mapping a target to its
// per-target emit declaration. The dispatch table is a List of these
// rows. SCAFFOLD until the per-target emit `.dag` declarations exist
// (currently the per-target emit logic is still hand-Rust at
// rust_target.rs / python_target.rs / inline Go in emit.rs; Layers 4-5
// retire those, at which point this carrier becomes 🟢 TERMINAL).
//
// Naming: `emit_decl` rather than `emit_fn` because the value-position
// is a DeclarationRef pointing at a `data` row (or a future `fn`
// declaration) in src/v3/spec/{rust,python,go}.dag. Resolved at lower
// time per the typed-DeclarationRef discipline established in
// `src/v3/spec/rust.dag` (cf. the M1(2.7) name-bridge unwind cited at
// `emit.rs:1-30`).
type EmitTargetDispatchRow {
  target: EmitTarget
  emit_decl: DeclarationRef    // points at per-target emit declaration
}
```

### §4.2 Data values (dispatch table)

**In `src/v3/std/emit_model.dag`** (or, if we want to keep target-specific bindings in the per-target spec files: split into `src/v3/spec/{rust,python,go}.dag` with one row each):

```dag
// Dispatch table: ordered list of (target → emit-declaration) rows.
// One row per supported EmitTarget variant. Order is structurally
// irrelevant (dispatcher matches on target variant) but conventionally
// matches declaration order in src/v3/spec/ for diff readability.
data emit_target_dispatch_table: List<EmitTargetDispatchRow> = [
  { target: Go,     emit_decl: rust_path_to_go_emit_decl },     // points at src/v3/spec/go.dag::emit_go_module
  { target: Rust,   emit_decl: rust_path_to_rust_emit_decl },   // points at src/v3/spec/rust.dag::emit_rust_module
  { target: Python, emit_decl: rust_path_to_python_emit_decl }  // points at src/v3/spec/python.dag::emit_python_module
]
```

(The exact DeclarationRef values resolve at lower time; the `rust_path_to_*_emit_decl` symbols are placeholders pending Layer 4-5 authoring of the per-target emit declarations themselves. For Layer 1 alone, the dispatch table can land with stub DeclarationRefs pointing at TODO-decls; the dispatcher's match-arm dispatch logic is independent of what the per-target emit declarations DO.)

### §4.3 Pure dispatch function

**Also in `src/v3/std/emit_model.dag`**:

```dag
// Dispatcher: look up the dispatch row for a target, invoke its
// emit_decl with (dag, mode), wrap result in EmittedSource.
//
// Match-arm enumeration follows the EmitTarget variants directly. The
// dispatch table at §4.2 is the SINGLE AUTHORITY for which targets are
// supported; this function reads it.
//
// Error handling: each per-target emit may fail with target-specific
// errors. The dispatcher returns a sum (EmitDispatchError) discriminating
// by which target produced the error.
fn emit_with_mode(dag: Dag, target: EmitTarget, mode: EmitMode)
  -> Result<EmittedSource, EmitDispatchError> =
  match target {
    Go     => invoke_target_emit(dag, mode, Go,     emit_target_dispatch_table)
    Rust   => invoke_target_emit(dag, mode, Rust,   emit_target_dispatch_table)
    Python => invoke_target_emit(dag, mode, Python, emit_target_dispatch_table)
  }

fn emit(dag: Dag, target: EmitTarget) -> Result<EmittedSource, EmitDispatchError> =
  emit_with_mode(dag, target, Program)

fn emit_module(dag: Dag, target: EmitTarget) -> Result<EmittedSource, EmitDispatchError> =
  emit_with_mode(dag, target, Module)
```

(The `invoke_target_emit` helper looks up the row matching `target` in the dispatch table, invokes the referenced declaration with `(dag, mode)`, and wraps. The exact shape of `invoke_target_emit` depends on whether `.dag` can express "invoke a DeclarationRef as a function" at compile time — this is the SAME pattern `regen_lens` uses to invoke generated lens declarations, so the capability is present in the codegen-driver context if not in pure `.dag`.)

**EmitDispatchError** is also a sum carrier — one variant per target's error type. Layers 4-5 will define the per-target error sum; Layer 1 can declare the dispatcher-side sum-variant shape:

```dag
// Sum discriminating which target's emit produced the error.
// Per-variant inner type lands when each per-target emit declares its
// own error carrier (Layers 4-5).
type EmitDispatchError
  = GoEmitFailed(GoEmitError)         // GoEmitError TBD post-Layer-4
  | RustEmitFailed(RustEmitError)     // RustEmitError TBD post-Layer-5
  | PythonEmitFailed(PythonEmitError) // PythonEmitError TBD post-Layer-5
```

### §4.4 Codegen driver outline

A new codegen driver `src/v3/compiler/src/regen_emit_dispatcher.rs` (or extension of an existing regen driver) reads:
- The new carriers in `emit_model.dag` (EmitTarget, EmitMode, EmittedSource, EmitTargetDispatchRow)
- The data value `emit_target_dispatch_table`
- The functions `emit_with_mode`, `emit`, `emit_module`

…and produces a generated file `src/v3/compiler/src/emit_dispatcher_generated.rs` containing:
- `pub enum EmitTarget { Go, Rust, Python }` (from the EmitTarget variant declaration)
- `pub enum EmitMode { Program, Module }`
- `pub struct EmittedSource { text, target, mode }`
- `pub fn emit_with_mode(dag, target, mode) -> Result<EmittedSource, EmitDispatchError>` with match arms generated from the dispatch table
- `pub fn emit(...)` and `pub fn emit_module(...)` thin wrappers

The driver follows the established codegen pattern (see `regen_tokenize.rs` for prior art at 1,186 lines for tokenize). For the dispatcher specifically, the driver is small — probably ~100-200 lines — because the substrate is also small.

### §4.5 Parity verification (semantic, not textual)

Per §"Anti-paper-shrink check" in the catalogue: the parity invariant must be SEMANTIC (behavioral equivalence) not TEXTUAL (byte-identical source-clone).

Specifically: the existing test suite already exercises `emit(dag, EmitTarget::Rust)` via `tests/integration/m1_3_emit_rust_test.rs`, `emit(dag, EmitTarget::Python)` via `m1_4_emit_python_test.rs`, `emit(dag, EmitTarget::Go)` via `m1_3_emit_go_test.rs`, etc. These tests assert specific OUTPUT TEXT for given input DAGs (the per-target emission rules). Semantic parity = these tests pass unchanged when emit.rs's dispatcher is replaced by the generated counterpart.

Determinism (D-1) is preserved automatically because the generated dispatcher is a pure match on `EmitTarget` — no map/set iteration, no timestamps, no `file!()`/`line!()`. Same `(dag, target, mode)` → same dispatch → same per-target call → same output.

**Anti-paper-shrink discriminator**: the retirement PR must include diff hunks ADDING the `EmitTarget`, `EmitMode`, `EmittedSource`, `EmitTargetDispatchRow` types to `emit_model.dag` AND the dispatch table data row + the three pure functions. The Rust counterparts in `emit.rs` (lines 1060-1163) are DELETED. The generated file appears at `src/v3/compiler/src/emit_dispatcher_generated.rs`. If the retirement PR instead moves `emit.rs` content to `tools/emit_dispatcher.rs.in` without growing `emit_model.dag`, the substrate-growth check fails (no new carriers added) — paper-shrink caught.

---

## §5 Sketches for Layers 2-N

### Layer 2 sketch (`VariantPayloadBinding` substrate-mirror)

Already-existing carrier at `src/v3/std/clean_emission.dag` (`VariantPayloadFieldAccessRule`). The hand-Rust `VariantPayloadBinding<T>` at `emit.rs:14-55` is a parallel-representation. The codegen driver reads `clean_emission.dag` + generates the Rust enum + methods. ~50 lines of hand-Rust → 0.

### Layer 3 sketch (per-construct syntax-spec readers)

`parse_*_syntax` family (~1,400 lines at `emit.rs:2513-3100`+) reads declaration field values via typed `DeclarationRef` access. Each reader function is a structural read returning a typed Rust record. Could be authored as `.dag` `fn` declarations directly (since they're pure data-extraction), or as a generic `fn extract_syntax_spec<S>(decl: DeclarationRef) -> S` walker. Estimated: ~1,400 lines → ~200 lines of `.dag` substrate.

### Layer 4 sketch (inline Go emitter extraction + retirement)

Step 4a (refactor): extract `emit_go_with_mode` + Go-specific helpers from `emit.rs` to a new `src/v3/compiler/src/emit/go_target.rs`. Pure code motion; no `.dag` work. Step 4b (retire as Layer 5 pattern): same as Rust/Python target retirement.

### Layer 5 sketch (per-construct emit fns in target modules)

Each per-construct emit fn is a structural fold over `Behavior` variants. Modeled as `.dag` `fn` declarations pattern-matching `Behavior`. Requires:
- `.dag` mutual recursion (currently being added per recent T-E-P-Producer-Broadening gates)
- String-building monoid (concat as List<String> fold then join)
- Per-target `LanguageSpec` field reads (already supported per the typed-DeclarationRef pattern)

If any of these `.dag` features is missing, Layer 5 stalls on substrate-language features, not pipeline-stage migration. Pre-Layer-5 verification: pick ONE small construct (e.g., "emit a literal int") and prove the whole pipeline (`.dag` substrate + codegen + parity) works for it. Per the framing in the catalogue: this is "the small proof-of-concept" question.

### Layer 6 sketch (whole-file retirement)

After Layers 1-5: `emit.rs` is empty except for `pub use` re-exports. Delete the file; consumers re-import from `emit_dispatcher_generated.rs` (or whatever the post-retirement module path is). Mechanical.

---

## §6 Open questions before authoring substrate

1. **DeclarationRef-as-function-invocation in `.dag`**: does `.dag` currently support `decl_ref(args)` syntax where `decl_ref` is a `DeclarationRef` value resolved at lower time? `regen_*` drivers do this in Rust, but if `.dag` itself can't express it, the dispatcher needs codegen-side glue. Spot-checking suggests this works per the pattern in `pipeline.dag::PipelineStageBinding`.
2. **Variant-arm match exhaustiveness check in `.dag`**: the dispatcher's `match target { Go => ..., Rust => ..., Python => ... }` MUST be exhaustive over `EmitTarget`'s variants. Does the `.dag` lower verify exhaustiveness, or is this a Layer-1 gap?
3. **Result<T, E> sum-carrier dispatch**: the dispatcher returns `Result<EmittedSource, EmitDispatchError>`. Both `Result` and the per-target wrapping (`GoEmitFailed(GoEmitError)`) need to be expressible. `Result` is standard library (`std.result` per other `.dag` files); per-target wrapping needs `EmitDispatchError` declared (this doc proposes it).
4. **Codegen driver placement**: extend an existing regen driver, or author a new `regen_emit_dispatcher.rs`? Recommendation: new driver, named for its single responsibility, matching `regen_tokenize.rs` pattern.

---

## §7 Substantive retirement risks

1. **Paper-shrink via dispatcher template-clone**: naive shape = `mv emit.rs::{EmitTarget, EmitMode, EmittedSource, emit_with_mode, ...} → tools/emit_dispatcher.rs.in`, codegen-driver copies through, `emit.rs` lines 1060-1163 deleted. Anti-discriminator: substrate-growth check on `emit_model.dag` (must add the 4 new carriers + dispatch table + 3 fns).
2. **EmitDispatchError carrier coupling to Layer 4-5**: the dispatcher's error sum references per-target error types that don't exist as `.dag` substrate yet. Until Layers 4-5 author them, the dispatcher's `EmitDispatchError` must either (a) carry a placeholder generic error, or (b) the dispatcher itself defers retirement until per-target error types exist. Recommendation: (b) — Layer 1 retires together with the per-target error sum-variants declared as SCAFFOLD pending Layer 4-5.
3. **Cross-target convenience wrappers** (`emit_rust_text`, `emit_go_module_text`, etc.) need their own dispatch entries or can be expressed as generic `extract_text(emit(dag, T)) for T in EmitTarget`. The 6 wrappers in `emit.rs:1105-1163` collapse to 1 generic function if `.dag` supports this; otherwise stay as 6 declarations.
4. **Tests import from `crate::emit::*`**: the retirement PR will break test imports unless the generated file is re-exported from `crate::emit` (i.e., a thin `mod emit { pub use crate::emit_dispatcher_generated::*; }` shim survives until Phase 2 test-harness dissolution).

---

## §8 What this layer's retirement demonstrates

If Layer 1 lands cleanly:
- The translation Rust → `.dag` is CLEAR for tabular-shaped dispatch logic in emit (not just for tokenize/parse). Pattern transfers.
- Substrate growth is the measurable receipt (not file-path shrinkage).
- The codegen driver follows the established `regen_tokenize.rs` shape; no new substrate-language features needed.
- Subsequent layers (2-5) can lean on Layer 1's pattern.

If Layer 1 stalls:
- The stall point names the specific substrate-language feature gap (DeclarationRef invocation, exhaustiveness check, etc.).
- Those gaps become explicit substrate-language work items, not "we don't know how to retire emit."

The proof-of-concept for emit retirement is Layer 1, not the full 13K lines.
