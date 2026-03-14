# v2 Compiler Postmortem — Bootstrap to cargo check (2026-03-14)

Operational entrypoint: `src/v2/WORKBOARD.md`

This file is the exhaustive audit ledger. Keep active priorities, task slicing,
and parallel work coordination in `src/v2/WORKBOARD.md`.

This document records the full state of the v2 self-hosted compiler at
the point where the generated Rust crate passes `cargo check`, `cargo
build`, and runtime smoke tests. It catalogs every workaround, hack,
and shortcut taken during bootstrap, and maps out all remaining work.

## What exists

**v2 .dag source:** 7 modules, 7,292 lines.

| Module | Lines | Purpose |
|--------|-------|---------|
| 00_core.dag | 341 | Shared types (Token, TokenKind, AST nodes, TypeExpr, IR) |
| 01_tokenize.dag | 471 | Tokenizer (string scanning, keyword recognition) |
| 02_parse.dag | 3,313 | Recursive descent parser with Pratt precedence |
| 03_resolve.dag | 465 | Module resolution (Kahn's topological sort) |
| 04_typecheck.dag | 1,041 | Type checking (structural, recursive type detection) |
| 05_emit.dag | 1,528 | Rust code emission from typed IR |
| 06_pipeline.dag | 133 | Pipeline orchestration (tokenize → parse → resolve → typecheck → emit) |

**Generated Rust crate:** ~10,500 lines, 9 source files (7 modules + lib.rs + v2_rt.rs).

**No `extern func` declarations.** The v2 compiler is pure .dag — no
Rust-backed extern functions in the .dag source.

## What's proven

| Layer | Evidence | Status |
|-------|----------|--------|
| Syntactic correctness | 94 parser tests (daglang-syntax), 315 codegen unit tests (daglang-emit) | CI gate |
| Type correctness | `cargo check` passes on generated crate | CI gate (v2_crate_cargo_check) |
| Link correctness | `cargo build` succeeds | Test (--ignored) |
| Semantic correctness | Phase 3 interpreter tests: tokenize → parse → resolve → typecheck → emit on real input | CI gate |
| Runtime correctness | 3 smoke tests: tokenizer produces tokens, ends with Eof, recognizes `fn` as KwFn | Test (--ignored) |

**Test counts:** 60 pass, 0 fail, 3 ignored (OOM on large files × 2, cargo build + smoke).

---

## Debt catalog

### Pass ledger setup

This section is the running scan log for the v2 pipeline. Each pass
records the exact file inventory that was in scope and then notes, per
file, whether a confirmed invariant violation was found.

### Pass 1: Pipeline file inventory

Requested directory command:

```sh
tree -d src/v2 dsl/std
```

Local fallback used during this pass (`tree` was not installed in the
workspace shell):

```sh
find src/v2 dsl/std -type d | sort
```

Pipeline file inventory for pass 1:

```text
src/v2
├── 00_core.dag
├── 01_tokenize.dag
├── 02_parse.dag
├── 03_resolve.dag
├── 04_typecheck.dag
├── 05_emit.dag
├── 06_pipeline.dag
└── tests
    └── src

dsl/std
├── behavioral.dag
├── errors.dag
├── resources.dag
└── types.dag
```

`src/v2/tests/**` is part of the directory scope above, but not part of
the compiler pipeline proper. Pass 1 scanned the 7 compiler modules plus
the 4 `dsl/std` files they depend on.

Canonical numbered file order for repeated end-to-end scans:

| ID | File | Role |
|----|------|------|
| 01 | `src/v2/00_core.dag` | Shared compiler contracts and AST/type surface |
| 02 | `dsl/std/types.dag` | Standard type vocabulary imported by the pipeline |
| 03 | `dsl/std/resources.dag` | Standard resource vocabulary imported by the pipeline |
| 04 | `dsl/std/behavioral.dag` | Behavioral stdlib shapes referenced by the pipeline |
| 05 | `dsl/std/errors.dag` | Canonical error-shape vocabulary |
| 06 | `src/v2/01_tokenize.dag` | Tokenizer |
| 07 | `src/v2/02_parse.dag` | Parser |
| 08 | `src/v2/03_resolve.dag` | Resolver |
| 09 | `src/v2/04_typecheck.dag` | Typechecker |
| 10 | `src/v2/05_emit.dag` | Rust emitter |
| 11 | `src/v2/06_pipeline.dag` | End-to-end pipeline wiring |

All later passes should use these IDs so repeated scans stay aligned to
the same end-to-end order.

### Pass 1: Per-file invariant ledger

**`[01] src/v2/00_core.dag`**
- No confirmed file-local invariant violation on pass 1. This file is
  mostly contract surface; several downstream findings are breaches of
  guarantees documented here rather than local implementation bugs.

**`[06] src/v2/01_tokenize.dag`**
- `scan_number()` launders integer parse failure or overflow into a valid
  `LitInt(0)` token via `parse_int(s: text) ?? 0`
  (`src/v2/01_tokenize.dag:221-239`). Invalid numeric source should stay
  invalid until it becomes a diagnostic.

**`[07] src/v2/02_parse.dag`**
- `parse_config_fields()`, `parse_rest_fields()`, and
  `parse_file_fields()` fabricate empty-string expressions for required
  fields (`endpoint`, `base_url`, `base_path`) instead of rejecting the
  missing input (`src/v2/02_parse.dag:1290-1322`,
  `src/v2/02_parse.dag:1376-1399`,
  `src/v2/02_parse.dag:1443-1450`).
- Inline `operation` and inline `capability` lowering only preserve
  `Product` and plain `Named` return types; any other return shape
  collapses to `outputs: []`, erasing source information before
  typecheck sees it (`src/v2/02_parse.dag:1504-1510`,
  `src/v2/02_parse.dag:1974-1980`).
- `parse_status_pattern()` converts an invalid response or exit status
  token into a synthetic `"_"` literal with `err: none`
  (`src/v2/02_parse.dag:1702-1746`).
- `parse_pattern()` and `parse_field_init()` invent placeholders instead
  of failing: malformed match arms can become `Wildcard`, and malformed
  record literal entries can become a synthetic `_` field
  (`src/v2/02_parse.dag:2792-2827`,
  `src/v2/02_parse.dag:3023-3055`).

**`[08] src/v2/03_resolve.dag`**
- `resolve_modules()` assigns `dep_order = -1` to modules never placed by
  topo sort and then sorts them into the normal resolver output anyway
  (`src/v2/03_resolve.dag:90-104`,
  `src/v2/03_resolve.dag:439-444`).

**`[09] src/v2/04_typecheck.dag`**
- `merge_envs()` documents last-writer-wins shadowing but implements
  first-writer-wins deduplication (`src/v2/04_typecheck.dag:174-197`).
- `build_type_env()` loses import provenance by resolving every imported
  name against one merged parent environment instead of the specific
  module named by each `ResolvedImport`
  (`src/v2/04_typecheck.dag:209-225`).
- `typecheck_and_validate()` exists, but the main pipeline does not use
  it, so the post-typecheck unresolved-type audit is not part of the
  normal compile path (`src/v2/04_typecheck.dag:992-999` together with
  `src/v2/06_pipeline.dag:124-125`).

**`[10] src/v2/05_emit.dag`**
- Service method signatures promise `Result<ret, Box<dyn Error>>`, but
  several transport lowerings do not preserve that contract:
  `emit_shell_call()` and `emit_file_call()` always return `String`,
  while `emit_local_call()` returns a bare function call rather than a
  `Result` (`src/v2/05_emit.dag:917-931`,
  `src/v2/05_emit.dag:986-1009`).
- The emitter's own file header says it receives a fully resolved,
  unambiguous typed graph (`src/v2/05_emit.dag:1-11`), but the pipeline
  currently violates that precondition.

**`[11] src/v2/06_pipeline.dag`**
- `compile_sources()` gates only on parse failure, then always resolves,
  typechecks, and emits even when later stages produced error
  diagnostics (`src/v2/06_pipeline.dag:99-136`).
- The main path calls `typecheck()` directly rather than
  `typecheck_and_validate()`, so the compiler bypasses its own
  post-typecheck invariant audit (`src/v2/06_pipeline.dag:124-125` with
  `src/v2/04_typecheck.dag:992-999`).

**`[02] dsl/std/types.dag`**
- No confirmed pipeline invariant violation on pass 1.

**`[03] dsl/std/resources.dag`**
- No confirmed pipeline invariant violation on pass 1.

**`[04] dsl/std/behavioral.dag`**
- No confirmed pipeline invariant violation on pass 1.

**`[05] dsl/std/errors.dag`**
- No confirmed pipeline invariant violation on pass 1.

### Pass 2: Silent drop / placeholder laundering

This pass focused on places that still return success while skipping
input, fabricating placeholders, or discarding malformed substructure.

**`[07] src/v2/02_parse.dag`**
- `parse_op_body_entries()` silently skips unknown `name: value` entries
  inside v1 operation bodies instead of producing a diagnostic. A
  misspelled key can disappear without leaving evidence in the parsed
  `OperationDef` (`src/v2/02_parse.dag:1600-1634`).
- `parse_capability()` accepts a bare `capability Name` with no block or
  inline signature and manufactures an empty capability instead of
  failing (`src/v2/02_parse.dag:1944-1985`).
- `make_call_expr()` only preserves calls whose callee parsed as `Var` or
  `FieldAccess`. Any other callee expression is rewritten to
  `Call { func: "<expr>", ... }`, which launders the original source
  shape into a synthetic placeholder (`src/v2/02_parse.dag:2559-2564`).

No new confirmed findings on this pass in `src/v2/00_core.dag`,
`src/v2/01_tokenize.dag`, `src/v2/03_resolve.dag`,
`src/v2/04_typecheck.dag`, `src/v2/05_emit.dag`,
`src/v2/06_pipeline.dag`, or the scanned `dsl/std` files.

### Pass 3: Semantic erasure

This pass focused on constructs that parse or emit successfully while
collapsing meaning that earlier stages should preserve.

**`[07] src/v2/02_parse.dag`**
- `parse_return()` discards the `return` control-flow marker and returns
  only the inner expression. After parse, `return x` and bare `x` become
  indistinguishable (`src/v2/02_parse.dag:2940-2948`).
- `parse_paren_expr()` maps `()` to `RecordLit {}` rather than a distinct
  unit literal, so source unit and anonymous empty record collapse to the
  same AST shape (`src/v2/02_parse.dag:3106-3112`).

**`[10] src/v2/05_emit.dag`**
- `emit_product_type_expr()` and `emit_coproduct_type_expr()` erase
  anonymous structural types during Rust emission: single-field products
  collapse to the field type, multi-field products become
  `serde_json::Value`, and anonymous coproducts also become
  `serde_json::Value`. The typed graph contains more structure than the
  emitted type layer preserves (`src/v2/05_emit.dag:304-330`).

No new confirmed findings on this pass in `src/v2/00_core.dag`,
`src/v2/01_tokenize.dag`, `src/v2/03_resolve.dag`,
`src/v2/04_typecheck.dag`, `src/v2/06_pipeline.dag`, or the scanned
`dsl/std` files.

### Pass 4: Call / async / output contract mismatches

This pass focused on places where the emitter's generated Rust no longer
matches the call, async, or return-shape contracts implied by the typed
graph.

**`[10] src/v2/05_emit.dag`**
- `build_item_registry()` indexes only items from the current module
  (`src/v2/05_emit.dag:95-103`, wired in
  `src/v2/05_emit.dag:127-129`), but `emit_call()` uses that registry to
  decide whether a callee is a `func` and which implicit service
  arguments to thread (`src/v2/05_emit.dag:506-526`). Imported workflow
  functions therefore lose `.await?` and service-parameter threading
  during call emission.
- `emit_func_def()` declares `Result<T, Box<dyn std::error::Error>>` for
  every `func`, but emits the raw body expression as the function tail
  instead of wrapping success in `Ok(...)` or otherwise producing a
  `Result` value (`src/v2/05_emit.dag:350-364`,
  `src/v2/05_emit.dag:379-383`). This is a broader signature/body
  mismatch than the service-method issue already noted in pass 1.

No new confirmed findings on this pass in `src/v2/00_core.dag`,
`src/v2/01_tokenize.dag`, `src/v2/02_parse.dag`,
`src/v2/03_resolve.dag`, `src/v2/04_typecheck.dag`,
`src/v2/06_pipeline.dag`, or the scanned `dsl/std` files.

### Pass 5: Module-boundary fidelity

This pass focused on whether source-module boundaries and import intent
survive emission.

**`[10] src/v2/05_emit.dag`**
- `emit_imports()` ignores the explicit imported-name list and emits
  `use crate::<module>::*;` for every source import. That widens source
  visibility, launders name-boundary intent, and can introduce collisions
  that the original `import foo { Bar }` syntax did not permit
  (`src/v2/05_emit.dag:148-156`).

No new confirmed findings on this pass in `src/v2/00_core.dag`,
`src/v2/01_tokenize.dag`, `src/v2/02_parse.dag`,
`src/v2/03_resolve.dag`, `src/v2/04_typecheck.dag`,
`src/v2/06_pipeline.dag`, or the scanned `dsl/std` files.

### Pass 6: Full numbered reread (`01 -> 11`)

This pass re-read every numbered file in the canonical order above. The
lens for this pass was source-fidelity and metadata preservation:
whether later stages can still tell what the source said, and whether
auxiliary data like field/import metadata survives the transition into
the typed graph and emitted Rust.

**`[01] src/v2/00_core.dag`**
- No new confirmed findings on pass 6.

**`[02] dsl/std/types.dag`**
- No new confirmed findings on pass 6.

**`[03] dsl/std/resources.dag`**
- No new confirmed findings on pass 6.

**`[04] dsl/std/behavioral.dag`**
- No new confirmed findings on pass 6.

**`[05] dsl/std/errors.dag`**
- No new confirmed findings on pass 6.

**`[06] src/v2/01_tokenize.dag`**
- Unterminated strings are laundered into ordinary string tokens instead
  of surfacing as lexical errors. `scan_string()` turns
  `UnterminatedString` into `LitStr`, and `scan_str_cont()` turns it
  into `StrEnd`, both with `err`-free success paths
  (`src/v2/01_tokenize.dag:258-297`,
  `src/v2/01_tokenize.dag:303-339`).

**`[07] src/v2/02_parse.dag`**
- `parse_import()` collapses `import foo.bar` and `import foo.bar {}` to
  the same AST shape (`Import { names: [] }`), so the parser no longer
  preserves whether the binding list was omitted or explicitly empty
  (`src/v2/02_parse.dag:547-562`).

**`[08] src/v2/03_resolve.dag`**
- No new confirmed findings on pass 6 beyond earlier resolver issues.

**`[09] src/v2/04_typecheck.dag`**
- `resolve_field()` drops `from_key` when rebuilding `Field`, so JSON-key
  provenance present in the parsed AST can disappear during type
  resolution (`src/v2/04_typecheck.dag:388-398`).
- `resolve_operation()` resolves only input/output field types and leaves
  `response` and `exit_mappings` untouched, even though both carry
  `TypeExpr` payloads. The post-typecheck invariant audit also skips
  those operation substructures, checking only inputs and outputs in
  service operations (`src/v2/04_typecheck.dag:440-459`,
  `src/v2/04_typecheck.dag:887-896`). That means unresolved response or
  exit types can survive typecheck without being reported.

**`[10] src/v2/05_emit.dag`**
- `emit_prelude()` emits `use serde::\{Serialize, Deserialize\};`, which
  is not valid Rust syntax. This is an output-shape violation in the
  emitted crate surface itself (`src/v2/05_emit.dag:164-166`).

**`[11] src/v2/06_pipeline.dag`**
- No new confirmed findings on pass 6 beyond earlier pipeline gating and
  validation-path issues.

### Pass 7: Parser structural violations

This pass audited `02_parse.dag` for structural violations not covered
in passes 1-6: wildcard fabrication, systematic dummy-node patterns,
and duplicated logic.

**`[07] src/v2/02_parse.dag`**
- `token_to_binop()` wildcard `_ => Add` silently substitutes addition
  for any unrecognized token kind. An internal caller passing a
  non-operator token gets a valid `BinOpKind` back instead of a failure
  (`src/v2/02_parse.dag:2383`).
- ~28 error-path functions construct dummy AST nodes with `name: ""`,
  `span: SourceSpan { start: 0, end: 0 }`, `fields: []`. The error IS
  set alongside the dummy node, but the node is also returned — callers
  can use it without checking the error. Pass 1 documented specific
  instances (`parse_config_fields`, `parse_status_pattern`, etc.); this
  finding confirms the pattern is systematic across the entire parser.
- `expect_ident` vs `expect_name` (lines 316-361): near-duplicate logic
  diverging only on keyword fallback. `expect_name` adds a
  `keyword_to_name()` branch; `expect_ident` lacks it. Both share
  identical error construction and `NameResult` return structure
  (`src/v2/02_parse.dag:316-361`).
- `parse_io_blocks_acc` (lines 1994-2027): `KwInput` and `KwOutput`
  branches are copy-pasted, differing only in which accumulator
  (`inputs` vs `outputs`) receives `r2.fields` in the tail call
  (`src/v2/02_parse.dag:1994-2027`).

### Pass 8: v1 codegen fabrication depth

This pass audited the v1 codegen pipeline (`fn_codegen.rs`,
`type_codegen.rs`, `render_rust.rs`) for fabrication, duplication, and
invariant violations beyond what Categories 0b, 3, and 5 already
document. All findings die with self-hosting.

**`fn_codegen.rs`**
- `infer_struct_name()` returns an empty string when no struct
  candidates match the field names. Downstream code receives `""` as a
  valid struct name, generating `struct` definitions with an empty
  identifier (`fn_codegen.rs:451-453`). Category 3 S77 documents wrong
  matching; this is the degenerate case where matching fails entirely.
- `infer_field_type_from_expr()` defaults all unknown expressions to
  `"String"`. Synthesized structs get wrong field types for any
  non-literal expression (`fn_codegen.rs:2824`).
- `with` intrinsic silently falls back to `v2_rt::with()` runtime call
  when the pattern doesn't match known cases. No error emitted; the
  codegen silently delegates to the runtime shim
  (`fn_codegen.rs:1034-1043`).
- `is_already_optional_expr()` falls back to searching ALL structs in
  the global context when receiver type is unknown. Can misidentify a
  field as optional based on an unrelated struct that happens to have
  the same field name (`fn_codegen.rs:2437-2443`).
- `resolve_named_or_positional()` uses unchecked array index
  `args[pos]` with no bounds validation. Panics if `pos >= args.len()`
  (`fn_codegen.rs:1392`).

**`type_codegen.rs`**
- Duplicate container kind matching: `List`/`Map`/`Set` enumerated at
  lines 60-65 (returning `ContainerKind`) and again at lines 238-241
  (returning Rust type strings). Two representations of the same
  dispatch that must stay synchronized (`type_codegen.rs:60-65`,
  `type_codegen.rs:238-241`).
- Rust-specific constructs (`&'static str`, `&[...]`, `LazyLock`)
  generated in type codegen rather than the Rust renderer. Invariant 6
  violation: type codegen should produce target-agnostic IR; rendering
  decisions belong in `render_rust.rs` (`type_codegen.rs:227-233`,
  `type_codegen.rs:288-315`).
- String-to-static-str rule hardcoded in 3 separate places: line 228
  (`Named("String")` → `"&'static str"`), line 836 (return type
  mapping), and line 293 (`is_string_list` check for `&[&str]`). No
  single authority (`type_codegen.rs:228,293,836`).

**`render_rust.rs`**
- `IrType::Unknown` renders as `"_"`, which is not valid Rust type
  syntax in most positions (`render_rust.rs:253`).
- Container name mapping (`List→Vec`, `Map→HashMap`) duplicates
  `type_codegen.rs` lines 238-241, and the `render_rust` version is
  missing `Set→HashSet`, falling through to `other → other`
  (`render_rust.rs:229-232`).

### Pass 9: v2 crate assembly

This pass audited `v2_crate_emit.rs` and `v2_runtime_shim.rs` for
violations beyond what Category 1 already documents.

**`v2_crate_emit.rs`**
- `module_prelude()` hardcoded imports diverge from actual .dag imports:
  `03_resolve` gets `use crate::parse::*` but `resolve.dag` imports
  only from `v2.compiler.core`, not from parse. Category 1 S79
  documents the hardcoding; this notes the specific divergence
  (`v2_crate_emit.rs:336-359`).
- Manual `struct_field_types` AND `struct_field_ir_types` entries for
  `BindingPower` and `SourceSpan` create two parallel sources of truth
  alongside the computed `build_struct_field_types()` output. Category 1
  documents the manual entries; this notes the dual-map duplication:
  same fields hand-inserted into both `HashMap<String, HashMap<String,
  String>>` and `HashMap<String, Vec<(String, IrType)>>`
  (`v2_crate_emit.rs:93-107`, `v2_crate_emit.rs:113-126`).

**`v2_runtime_shim.rs`**
- `scan_string_end()` does `pos += 2` for backslash escape without
  bounds check. If `\` appears at the last position in the string,
  `pos` jumps past `chars.len()` — the loop exits cleanly but silently
  succeeds on malformed input with a trailing backslash
  (`v2_runtime_shim.rs:135`).

### Pass 10: Cross-file boundary contracts

This pass audited boundary contracts across pipeline stages.

**`[09] src/v2/04_typecheck.dag`**
- `build_type_env()` silently drops imports not found in parent envs:
  when `lookup_type()` returns `None`, `inner_acc` is returned unchanged
  with a comment that it "will be caught as diagnostic later," but no
  mechanism exists to produce that diagnostic. The missing import is
  simply absent from the type environment. Extends the Pass 1 finding
  about lost import provenance with a concrete dropped-name consequence
  (`src/v2/04_typecheck.dag:216-221`).

**Pipeline boundary: `CompileResult` coherence.**
`CompileResult` (defined in `00_core.dag`) carries both `files` and
`diagnostics` with no type-level enforcement that error diagnostics
imply empty files. Combined with the pipeline's failure to gate on
resolve/typecheck errors (documented in Pass 1 and Category 0), the
emitter can produce output files from a graph that contains unresolved
references. The type makes this invalid state representable — violates
correctness by construction (Invariant 9).

### Pass 11: Tokenize-stage diagnostic contract

This pass re-read the numbered pipeline with a narrow lens on lexical
failure representation: whether the tokenize stage can actually surface
errors as diagnostics, or only as malformed-but-successful token streams.

**`[06] src/v2/01_tokenize.dag`**
- `tokenize()` returns only `List<Token>` with no diagnostic channel
  (`src/v2/01_tokenize.dag:84-88`). Combined with the malformed-number
  and unterminated-string cases already logged in Passes 1 and 6,
  lexical failure is not representable as a stage result; the tokenizer
  can only launder bad input into tokens.

**`[11] src/v2/06_pipeline.dag`**
- `TokenizeResult { tokens, diagnostics }` exists in the pipeline file
  but nothing produces or consumes it (`src/v2/06_pipeline.dag:39-42`,
  `src/v2/01_tokenize.dag:84-88`). The pipeline therefore advertises a
  diagnostic-carrying tokenize stage that the actual tokenizer API
  cannot implement.

No new confirmed findings on this pass in `src/v2/00_core.dag`,
`src/v2/02_parse.dag`, `src/v2/03_resolve.dag`,
`src/v2/04_typecheck.dag`, `src/v2/05_emit.dag`, or the scanned
`dsl/std` files.

### Pass 12: Import intent collapse

This pass followed import syntax end-to-end to check whether the parser,
resolver, and type environment preserve what the source import meant.

**`[07] src/v2/02_parse.dag`**
- `parse_import()` still lowers both bare `import foo.bar` and explicit
  `import foo.bar {}` to `Import { names: [] }`
  (`src/v2/02_parse.dag:547-562`). Pass 6 logged the AST collapse; this
  pass traced its downstream semantic consequence.

**`[09] src/v2/04_typecheck.dag`**
- `build_type_env()` imports only the names listed in `imp.names`
  (`src/v2/04_typecheck.dag:213-224`). Combined with the parser
  collapse above, a bare `import foo.bar` becomes a semantic no-op after
  type environment construction: the module target is resolved, but no
  bindings are introduced.

No new confirmed findings on this pass in `src/v2/00_core.dag`,
`src/v2/01_tokenize.dag`, `src/v2/03_resolve.dag`,
`src/v2/05_emit.dag`, `src/v2/06_pipeline.dag`, or the scanned
`dsl/std` files.

### Pass 13: Expression-tree type holes

This pass checked whether the so-called typecheck stage actually covers
all typed substructure in the AST, including types nested inside
expressions and service/data values.

**`[09] src/v2/04_typecheck.dag`**
- `resolve_item_types()` resolves declared type surfaces only and
  threads executable/value expressions through unchanged: `FnDef.body`,
  `FuncDef.body`, `ServiceDef.transport`, `ServiceDef.config`, and
  `DataDef.value` are all preserved without any expression walk
  (`src/v2/04_typecheck.dag:521-589`,
  `src/v2/04_typecheck.dag:605-614`).
- `validate_no_unresolved()` says it walks every `TypeExpr` in every
  item, but `collect_unresolved_in_item()` also skips expression trees
  entirely (`src/v2/04_typecheck.dag:846-918`). Since the core AST
  permits `Cast { target: TypeExpr }` inside expressions and carries
  `Expr` payloads inside `TransportBinding` and `DataDef`
  (`src/v2/00_core.dag:108-113`, `src/v2/00_core.dag:200-216`,
  `src/v2/00_core.dag:257-263`), unresolved types embedded in
  expressions can survive both typecheck and its post-pass validator.

No new confirmed findings on this pass in `src/v2/01_tokenize.dag`,
`src/v2/02_parse.dag`, `src/v2/03_resolve.dag`,
`src/v2/05_emit.dag`, `src/v2/06_pipeline.dag`, or the scanned
`dsl/std` files.

### Pass 14: Call and record emission shape loss

This pass re-read the emitter with a focus on whether expression shapes
from the AST survive Rust lowering without positional/structural loss.

**`[10] src/v2/05_emit.dag`**
- Generic call emission drops `NamedArg.name` entirely. Both
  `emit_call()` and `emit_method_call()` map through
  `emit_call_arg()`, which serializes only `arg.value`
  (`src/v2/05_emit.dag:506-526`, `src/v2/05_emit.dag:569-570`,
  `src/v2/05_emit.dag:573-604`; AST surface at
  `src/v2/00_core.dag:204-218`). Any non-positional argument semantics
  are erased during Rust lowering.
- `emit_record_lit()` emits anonymous record literals as bare
  `{\n field: expr,\n}` when `type_name == none`
  (`src/v2/05_emit.dag:698-710`). That is not valid Rust struct-literal
  syntax, so a valid AST node can become invalid emitted Rust.

No new confirmed findings on this pass in `src/v2/00_core.dag`,
`src/v2/01_tokenize.dag`, `src/v2/02_parse.dag`,
`src/v2/03_resolve.dag`, `src/v2/04_typecheck.dag`,
`src/v2/06_pipeline.dag`, or the scanned `dsl/std` files.

### Pass 15: Option/null and cast assumptions in emit

This pass focused on emitter helpers that assume Rust `Option` or Rust
primitive-cast semantics without typed evidence from the input graph.

**`[10] src/v2/05_emit.dag`**
- `emit_literal(LitNull)` always lowers to `None`, while
  `emit_data_value_json(LitNull)` lowers to `null`
  (`src/v2/05_emit.dag:482-490`,
  `src/v2/05_emit.dag:1407-1415`). The same DSL literal changes meaning
  depending only on which emission helper touches it.
- `emit_bin_op()` always lowers `??` to `.unwrap_or_else(...)`
  (`src/v2/05_emit.dag:732-744`), assuming the left operand is an
  `Option` even though the v2 typecheck stage does not annotate
  expression trees.
- `emit_cast()` always lowers `Cast` to Rust `as`
  (`src/v2/05_emit.dag:856-859`), but the AST permits arbitrary
  `TypeExpr` targets, not just primitive numeric/pointer casts
  (`src/v2/00_core.dag:200-216`). Record, container, and optional
  targets therefore become invalid or nonsensical Rust casts.

No new confirmed findings on this pass in `src/v2/01_tokenize.dag`,
`src/v2/02_parse.dag`, `src/v2/03_resolve.dag`,
`src/v2/04_typecheck.dag`, `src/v2/06_pipeline.dag`, or the scanned
`dsl/std` files.

### Pass 16: Data emission laundering and helper contract drift

This pass followed value-bearing helper APIs to see whether they still
mean what their signatures and comments say after the latest bootstrap
changes.

**`[10] src/v2/05_emit.dag`**
- For nested-record `data` emission, `emit_data_value_json()` serializes
  `Var` as the quoted variable name and all other non-literal,
  non-list, non-record expressions as `"null"`
  (`src/v2/05_emit.dag:1063-1075`,
  `src/v2/05_emit.dag:1407-1429`). Since `DataDef.value` is any `Expr`
  and typecheck leaves that expression untouched
  (`src/v2/00_core.dag:113`, `src/v2/04_typecheck.dag:605-614`),
  complex data definitions can silently change meaning during the JSON
  fallback path.

**`[11] src/v2/06_pipeline.dag`**
- `compile_file()` is documented as compiling a single source file
  “through all stages,” but the implementation only tokenizes and parses
  before returning the parsed module plus diagnostics
  (`src/v2/06_pipeline.dag:84-90`). The helper's public contract
  overclaims what callers actually receive.

### Pass 17: Resource model erasure

This pass followed `resource` declarations from stdlib source through
the parser, typechecker, and emitter to see which parts of the resource
contract survive.

**`[01] src/v2/00_core.dag`**
- `ResourceDef` carries only `properties` and `capabilities`; there is
  no structural slot for `acquire` or `release`
  (`src/v2/00_core.dag:111-112`). The core AST cannot faithfully
  represent the full source shape used by `dsl/std/resources.dag`.

**`[07] src/v2/02_parse.dag`**
- `parse_resource_entries()` handles `acquire { ... }` and
  `release { ... }` by calling `skip_until_rbrace()` and discarding the
  entire block contents (`src/v2/02_parse.dag:1883-1899`,
  `src/v2/02_parse.dag:1921-1941`). Resource lifecycle semantics are
  erased during parse rather than preserved for later checking.

**`[09] src/v2/04_typecheck.dag`**
- `typecheck` preserves `ResourceDef.properties` untouched and resolves
  only capability signatures (`src/v2/04_typecheck.dag:591-603`), so
  the resource metadata that *does* survive parse never gets validated
  as part of the resource model.

**`[10] src/v2/05_emit.dag`**
- `emit_item()` drops `ResourceDef.properties`, and `emit_resource_def()`
  lowers every resource to a trait containing only capability methods
  (`src/v2/05_emit.dag:182-183`,
  `src/v2/05_emit.dag:1018-1037`). Fields like `kind`, `mode`,
  `expires`, and any lifecycle structure are absent from the emitted
  Rust, even though they are central in the stdlib resource definitions
  (`dsl/std/resources.dag:28-110`).

### Pass 18: Item-kind coercion

This pass checked whether item kinds tokenized by the front end remain
distinguishable after parse and emission.

**`[07] src/v2/02_parse.dag`**
- `parse_item()` dispatches both `KwPattern` and `KwInterface` to
  `parse_func_def()` (`src/v2/02_parse.dag:587-600`), and
  `parse_func_def()` explicitly accepts `func`, `pattern`, and
  `interface` as interchangeable leading keywords
  (`src/v2/02_parse.dag:1104-1113`). Distinct surface constructs are
  therefore coerced to the same AST item.

**`[01] src/v2/00_core.dag`**
- The core `Item` union has `FuncDef` and `FnDef`, but no dedicated
  `PatternDef` or `InterfaceDef`
  (`src/v2/00_core.dag:107-114`). Once the parser performs the coercion,
  no later stage can recover the original declaration kind.

**`[10] src/v2/05_emit.dag`**
- The emitter only knows how to lower `FuncDef`/`FnDef`
  (`src/v2/05_emit.dag:176-183`), so parsed `pattern` and `interface`
  declarations are emitted as ordinary Rust functions/traits for those
  existing item kinds rather than as their own semantic category.

### Pass 19: Evaluator + interpreter fabrication patterns

Category 6 documents evaluator resource limits. This pass documents
value-level fabrication in the same codebase: places where the
evaluator or DAG interpreter invents values instead of failing.

**`eval_stack.rs`**
- `map`, `filter`, `filter_map`, and `flat_map` intrinsics all share
  the same fallback: when the lambda argument is missing or
  unparseable, they return the input list unchanged
  (`eval_stack.rs:1473`, `eval_stack.rs:1512`,
  `eval_stack.rs:1532`, `eval_stack.rs:1552`). A silently dropped
  transformation is indistinguishable from identity.
- `eval_ident()` converts unbound uppercase identifiers to
  `Value::Str(name)` instead of failing (`eval_stack.rs:1211-1212`).
  An undefined constructor like `Foo` silently becomes the string
  `"Foo"`, masking typos and missing imports.

**`interp/src/lib.rs`**
- `execute_primitive()` defaults every missing input port to
  `Value::Skipped` via `unwrap_or(Value::Skipped)` across 10+ call
  sites (`interp/src/lib.rs:72-139`). Wiring errors in the DAG
  propagate as Skipped instead of failing at the execution boundary.
- `execute_callable()` swallows evaluation errors when all inputs are
  Skipped, returning an empty output map (`interp/src/lib.rs:236-247`).
  Legitimate errors in function bodies are hidden if the node happened
  to receive no real inputs.
- `execute_callable()` with no `fn_body` performs a passthrough using
  `__out:` prefix conventions on input keys
  (`interp/src/lib.rs:213-225`). The IR does not distinguish
  intentional passthrough from missing body, so the interpreter guesses.

### Pass 20: Runtime shim per-function fabrication

Category 2 lists each runtime shim function. This pass adds the
specific fabrication behavior per function: what each returns on
malformed input instead of failing.

**`v2_runtime_shim.rs`**
- `char_at()`: out-of-bounds or negative index → empty string via
  `unwrap_or_default()` (`v2_runtime_shim.rs:53`).
- `code_point()`: empty string → `0` (null code point) via
  `unwrap_or(0)` (`v2_runtime_shim.rs:147`). Indistinguishable from
  a legitimate `\0` input.
- `from_code_point()`: invalid or negative code point → empty string
  via `unwrap_or_default()` (`v2_runtime_shim.rs:154`).
- `substring()`: inverted range (`end < start`) silently clamped to
  empty string via `.max(0)` (`v2_runtime_shim.rs:66`).
- `scan_while()`, `skip_horizontal_ws()`, `scan_to_eol()`: negative
  `start` silently clamped to 0 via `.max(0)` — caller cannot
  distinguish "started at 0" from "gave invalid position"
  (`v2_runtime_shim.rs:99`, `v2_runtime_shim.rs:109`,
  `v2_runtime_shim.rs:119`).

### Pass 21: Typechecker + resolver structural gaps

Extends passes 1, 6, 10, and 13 with function-level findings in
`04_typecheck.dag` and `03_resolve.dag` not previously documented.

**`[09] src/v2/04_typecheck.dag`**
- `resolve_type_expr_with_resolving()` cycle detection returns success
  with empty diagnostics when a name is already on the resolving stack
  (`src/v2/04_typecheck.dag:287-288`). If the name is also unresolved,
  the unresolved-name diagnostic is suppressed by the early return.
- `lookup_type()` takes the first binding when multiple bindings share
  the same name (`src/v2/04_typecheck.dag:158-169`). Combined with
  `merge_envs()` first-writer-wins (Category 0), name collisions from
  different env layers are silently resolved to an arbitrary winner.
- `find_resolved_module()` and `find_typed_module()` return the first
  match without checking uniqueness
  (`src/v2/04_typecheck.dag:751-773`). Duplicate module names in the
  graph are silently collapsed.
- `validate_no_unresolved()` collects defined names from
  `tm.type_env.bindings` instead of from `tm.module.items`
  (`src/v2/04_typecheck.dag:851-861`). If a type definition failed to
  resolve and wasn't added to the environment, later references to that
  name are reported as "unresolved" instead of "failed to resolve."
- `collect_parent_envs()` silently returns `acc` unchanged when
  `find_typed_module()` returns `None` for a dependency
  (`src/v2/04_typecheck.dag:782-789`). An untyped dependency's
  bindings are simply absent, with no diagnostic to distinguish
  "dependency not typed" from "dependency has no exports."

**`[08] src/v2/03_resolve.dag`**
- `resolve_import()` returns `ResolvedImport { target_module: None }`
  with a diagnostic when the target module is not found
  (`src/v2/03_resolve.dag:162-176`), but downstream consumers do not
  filter partial imports. The typechecker receives the full list
  including failed imports, then silently drops them when lookup fails
  (covered above in `collect_parent_envs`). No stage enforces that
  only fully-resolved imports flow past the resolve/typecheck boundary.

### Pass 22: Emitter fabrication residuals

This pass caught remaining fabrication in `05_emit.dag` not covered by
passes 1, 4, 5, 14-18.

**`[10] src/v2/05_emit.dag`**
- `emit_first_arg()` returns an empty string literal `""` when the
  argument list is empty (`src/v2/05_emit.dag:615`). Callers that
  expect a serialized first argument get a valid-looking but fabricated
  Rust expression.
- `extract_service_name()` returns `"Unknown"` when the receiver
  expression does not match expected patterns
  (`src/v2/05_emit.dag:1298`, `src/v2/05_emit.dag:1300`). The string
  becomes a variable name in emitted Rust (`unknown.method()`), which
  compiles or fails unpredictably depending on scope.
- `emit_operation_test()` emits test scaffolding (mock status + body
  setup) with no operation invocation and no assertions
  (`src/v2/05_emit.dag:1143-1153`). The generated test always passes.

### Pass 23: Parser expression-level erasure

Extends Pass 3 (semantic erasure) with expression-level findings in
`02_parse.dag`.

**`[07] src/v2/02_parse.dag`**
- `parse_interp_parts()` silently completes a `StringInterp` node when
  the next token after an interpolated expression is neither `StrMid`
  nor `StrEnd` (`src/v2/02_parse.dag:3224`). Malformed interpolation
  produces a valid result with `err: none` instead of failing.
- `parse_brace_expr()` unwraps single-statement blocks to bare
  expressions (`src/v2/02_parse.dag:3252-3256`). After parse,
  `{ expr }` and `expr` are indistinguishable — later stages cannot
  recover that the source used an explicit block scope.

### Pass 24: Crate assembly + test harness gaps

This pass extends Pass 9 (crate assembly) and audits the test
infrastructure for contract violations.

**`v2_crate_emit.rs`**
- `assemble_v2_crate()` silently drops modules whose `dag_stem` is not
  in `V2_MODULE_MAP` via a bare `continue` (`v2_crate_emit.rs:172`).
  The resulting crate is incomplete with no diagnostic.

**`src/v2/tests/src/lib.rs`**
- `compile_tokenizer_module()` and `compile_all_modules()` silently
  discard data value evaluation failures with an empty `Err(_) => {}`
  arm (`tests/src/lib.rs:152-158`, `tests/src/lib.rs:532-534`). Tests
  pass with incomplete data state.
- `phase3_kind_tag_matches_ident` and `phase3_expect_ident_on_ident_token`
  hand-construct token `Value::Map`s with hardcoded `_variant` keys
  instead of running the tokenizer (`tests/src/lib.rs:862-968`).
  Bugs in the real token representation would not be caught.
- Runtime smoke tests are embedded as string literals injected into the
  generated crate (`tests/src/lib.rs:2353-2387`). They are not
  first-class test files and cannot be maintained independently.

### Pass 25: CodeIR definitions + lower_to_ir bridge

This pass audited the IR type definitions (`code_ir/mod.rs`) and the
AST-to-IR bridge (`lower_to_ir.rs`) for structural issues not covered
by Category 5 or Pass 8.

**`code_ir/mod.rs`**
- `FnDef.params` stores types as `Vec<(String, String)>` and
  `FnDef.return_type` as `Option<String>` instead of `IrType`
  (`code_ir/mod.rs:409-410`). Same for `StructDef.fields` as
  `Vec<(String, String, bool)>` (`code_ir/mod.rs:447`). Type
  information enters the IR as pre-rendered strings, preventing
  backends from re-interpreting the structure. Category 0b notes the
  IrType layer is incomplete; this is where the incompleteness is
  structural — the definitions themselves use strings where IrType
  should go.
- `Stmt::Let.ir_type`, `Expr::Call.obligation`, and
  `Expr::Struct.field_types` are all `Option` with comments saying
  "populated during compilation" (`code_ir/mod.rs:143,219,244`), but
  `lower_to_ir.rs` never populates them — it always passes `None`.
  Renderers must handle the missing case with fallbacks.

**`lower_to_ir.rs`**
- `map_abstract_type()` hardcodes `Backend::Rust` when resolving
  entrypoint parameter types (`lower_to_ir.rs:524-526`). IR produced
  by the bridge is already target-locked before any backend runs.
  Category 5 documents fn_codegen injecting Rust constructs into IR
  expressions; this shows the bridge itself bakes Rust into IR type
  annotations.
- `PureBody::ServiceCall` lowering silently drops unrecognized `phase`
  metadata values via wildcard: anything other than `"prepare"` or
  `"parse"` falls through to `None`, and the obligation is omitted
  from the emitted call with no diagnostic
  (`lower_to_ir.rs:254-270`).

### Pass 26: Valid-program miscompilation (2026-03-14)

This pass focused on the retrospective's most under-sampled theme:
well-formed .dag programs that compile to semantically wrong Rust.

**`[07] src/v2/02_parse.dag` + `[10] src/v2/05_emit.dag`**
- `??` (NullCoalesce) has `BindingPower { left: 1, right: 0 }` — right-
  associative (`src/v2/02_parse.dag:2348`). Every mainstream language
  with `??` (C#, JS, Kotlin) evaluates left-to-right. Right-associative
  `??` inverts short-circuit order: `a ?? b ?? c` evaluates the inner
  `b ?? c` before checking `a`, changing observable behavior when
  operands have side effects.
- `for` loops are desugared to `Call { func: "for", args: [...] }` by
  the parser (`src/v2/02_parse.dag:2976-2984`), then emitted as
  `.into_iter().map(|item| { ... }).collect::<Vec<_>>()`
  (`src/v2/05_emit.dag:565-566`). This changes the return type from
  `()` to `Vec<T>`, prevents `?`/`.await` propagation from inside the
  closure, and allocates a `Vec` for side-effectful loops.

**`[10] src/v2/05_emit.dag`**
- `NonEmptyList` and `NonEmptySet` container kinds are emitted as `Vec`
  and `BTreeSet` (`src/v2/05_emit.dag:297-300`), silently dropping the
  non-emptiness invariant. Downstream code relying on guaranteed
  non-emptiness can panic at runtime.
- Uppercase bind patterns in match arms are parsed as
  `VariantPattern { name, field_bindings: [] }` instead of `Bind`
  (`src/v2/02_parse.dag:2800-2801`). The emitter produces a bare
  identifier in pattern position (`src/v2/05_emit.dag:648-649`). In
  Rust, this is a constant/variant pattern, not a binding. Combined with
  no match exhaustiveness checking, valid but non-exhaustive .dag
  matches produce non-compiling Rust.

**`[09] src/v2/04_typecheck.dag` + `[10] src/v2/05_emit.dag`**
- Type aliases are transparently resolved by `type_body_to_expr`
  (`src/v2/04_typecheck.dag:257-258`): `type UserId = String` becomes
  `Primitive { name: "String" }` in the type environment. Function
  parameters typed as `UserId` are emitted as `String` in Rust
  signatures, losing nominal alias identity.
- `resolve_field` drops `from_key` (documented in Pass 6). The concrete
  miscompilation: `field: String from "fieldName"` emits without
  `#[serde(rename = "fieldName")]`, causing JSON deserialization to look
  for the wrong key at runtime.

### Pass 27: Negative-space audit (2026-03-14)

This pass identified constructs that are declared but never consumed, or
consumed but never produced.

**Dead types in `[11] src/v2/06_pipeline.dag`:**
- `StageResult` (line 34): never constructed, never returned.
- `ParseStageResult` (line 44): never constructed, never returned.
- `CompileFileResult` (line 51): produced by `compile_file()`, but
  `compile_file()` itself is never called.

**Dead functions:**
- `compile_file` (`src/v2/06_pipeline.dag:86`): never called.
- `read_source` (`src/v2/06_pipeline.dag:60`): never called.
- `make_error` (`src/v2/04_typecheck.dag:825`): never called; diagnostics
  constructed inline.
- `make_warning` (`src/v2/04_typecheck.dag:834`): never called.
- `detect_cycle` (`src/v2/03_resolve.dag:417`): never called;
  `topological_sort` handles cycles internally.
- `typecheck_and_validate` (`src/v2/04_typecheck.dag:992`): never called;
  pipeline calls `typecheck()` directly.
- `find_resolved_module` (`src/v2/04_typecheck.dag:751`): never called.
- `error_count` (`src/v2/04_typecheck.dag:1006`): only called by
  `typecheck_ok`, which is itself never called.
- `typecheck_ok` (`src/v2/04_typecheck.dag:1016`): never called.
- `module_type_bindings` (`src/v2/04_typecheck.dag:1021`): never called.
- `lookup_type_in_graph` (`src/v2/04_typecheck.dag:1029`): never called.

**Never-constructed enum variants:**
- `Severity::Info` (`src/v2/00_core.dag:341`): defined, never emitted.
- `BinOpKind::Concat` (`src/v2/00_core.dag:243`): handled in emit
  (line 762) but never produced by the parser.
- `TypeExpr::TypeApp` (`src/v2/00_core.dag:146`): handled in typecheck
  (line 372) and emit (line 276) but never produced by the parser. The
  parser maps `Name<T>` to `Container` or `Named`, never `TypeApp`.
- `Backend::Python` (`src/v2/06_pipeline.dag:32`): match arm emits `[]`
  (empty file list) — fabrication, not failure.

**Parsed-but-never-consumed fields (corroborates Pass 30):**
`Field.default_value`, `Param.default_value`, `OperationDef.modifiers`,
`OperationDef.response`, `OperationDef.exit_mappings`,
`OperationDef.transport` (per-op override), `AuthConfig.scheme`,
`AuthConfig.token_expr`, `ServiceConfig.*` (all fields),
`ResourceDef.properties`. All populated at parse, preserved through
typecheck, never read at emit.

**Duplicate type definitions (sustainability invariant violation):**
`TypedGraph`, `TypedModule`, `TypeEnv`, `TypeBinding` defined in both
`04_typecheck.dag` and `05_emit.dag`. `ResolvedImport`, `ResolvedModule`,
`ModuleGraph` defined in both `03_resolve.dag` and `04_typecheck.dag`.
Acknowledged as temporary (C4/C5) but still a parallel representation.

### Pass 28: Determinism and output stability (2026-03-14)

This pass audited ordering-dependent code paths in the v1 codegen and
v2 .dag pipeline.

**High risk — non-deterministic output:**
- `infer_struct_name()` tiebreaker uses `HashMap` iteration order
  (`fn_codegen.rs:434-453`). When multiple known structs have the same
  field-count proximity, which struct wins depends on hash seed. The
  wrong struct can be inferred, silently generating incorrect code.
- `compute_recursive_fields()` DFS roots from `HashSet<String>`
  iteration (`type_codegen.rs:1214-1225`). Which field gets `Box<>`
  varies between runs — both placements may compile, but the generated
  code differs non-deterministically.

**Medium risk — fragile or semantically wrong:**
- `build_variant_to_enum()` first-definition-wins depends on
  `V2_MODULE_MAP` order (`v2_crate_emit.rs:469-481`). Accidentally
  deterministic; no structural guarantee.
- `TypeDefSignature::Record` field-order-sensitive comparison
  (`v2_crate_emit.rs:563-588`). Structurally identical types with
  reordered fields are treated as distinct, violating structural type
  semantics. Deterministic but wrong.

**Low risk — cosmetic or latent:**
- `fill_missing_fields()` iterates `HashMap.keys()`
  (`fn_codegen.rs:341-369`). Emitted optional-field order is non-
  deterministic; Rust struct construction is order-independent.
- `global_fn_return_types` silently drops duplicate fn names
  (`v2_crate_emit.rs:132-144`). Accidentally deterministic.
- `struct_field_types` (HashMap) vs `struct_field_ir_types` (Vec)
  divergent ordering guarantees (`v2_crate_emit.rs:484-543`).
- `lib.rs` module order depends on `dep_order` stability; mitigated by
  alphabetical tiebreaker in resolver (`05_emit.dag:114-121`,
  `03_resolve.dag:365-373`).

### Pass 29: Boundary contract audit (2026-03-14)

This pass compared each stage's public contract with what the
implementation actually guarantees.

**typecheck → emit (highest severity):**
- **Gap 10: Emitter ignores `TypeEnv`.** `TypedModule` carries
  `type_env: TypeEnv` — the resolved type environment from the
  typechecker. `emit_module()` (`src/v2/05_emit.dag:127-142`)
  extracts `typed_module.module` and operates entirely on the raw
  pre-resolution AST. The typechecker's output is structurally present
  but functionally dead. The emit stage re-derives type structure from
  AST items rather than consuming the resolved types.
- **Gap 11: Emitter has no diagnostic channel.** Returns
  `List<TextFile>` with no error representation. Every other stage
  returns `{ value, diagnostics }`. Emit-stage problems have no
  representation.
- **Gap 12: Emitter locally redefines typechecker output types.**
  Independent `TypedGraph`, `TypedModule`, `TypeEnv`, `TypeBinding`
  definitions in `05_emit.dag:36-53` vs `04_typecheck.dag:65-79`.

**resolve → typecheck:**
- **Gap 7: Typechecker trusts but does not verify topological order.**
  `fold` over `graph.modules` in list order (`04_typecheck.dag:806`).
  If the list is not topologically sorted (e.g., `dep_order = -1`
  cycles), `collect_parent_envs()` silently returns empty, making
  imports no-ops.
- **Gap 8: `ModuleGraph.diagnostics` silently dropped.** Typechecker
  starts fresh `TypedGraph { diagnostics: [] }` — resolve-stage
  diagnostics are not propagated through the typecheck boundary.

**parse → resolve:**
- **Gap 4:** `Module?` → `Module` unwrap depends on behavioral
  invariant of two independent `ParseResult` fields (`module` and
  `error`).
- **Gap 5:** `Module.span` is a single-token span, not file-covering.
- **Gap 6:** `Module.name` is unvalidated `String`; empty strings from
  error paths are representable.

**tokenize → parse:**
- **Gap 2:** `Unknown` tokens conflate lexical and parse errors into a
  single halt point.
- **Gap 3:** Pipeline sidesteps `TokenizeResult` entirely; actual
  boundary is raw `List<Token>`.

**Pipeline:**
- **Gap 13:** Diagnostic concatenation order
  (`concat(graph.diagnostics, typed.diagnostics, parse_diagnostics)`)
  does not follow pipeline stage order.

**v1 evaluator:**
- **Gap 15:** S57 type-boundary checks are soft warnings collected in
  `EvalOutcome.warnings`, not enforcement. Wrong-typed values flow
  through function boundaries.
- **Gap 16:** `output_value()` "value" key fallback is undocumented in
  .dag contracts.

**v1 crate assembly:**
- **Gap 17:** `module_prelude()` widens module visibility beyond what
  .dag import declarations authorize.
- **Gap 18:** Type deduplication semantics ("same shape = same type")
  disagree with typechecker's type identity model ("each module defines
  its own types").

### Pass 30: Provenance loss audit (2026-03-14)

This pass traced provenance-carrying fields through parse → resolve →
typecheck → emit.

**Pattern:** The parser populates faithfully. The typechecker preserves.
The emitter ignores.

| Field | Populated | Preserved | Read at emit |
|-------|-----------|-----------|--------------|
| `Field.default_value` | parse:1024 | typecheck:388-398 | NEVER |
| `Param.default_value` | parse:2181 | typecheck:423-434 | NEVER |
| `OperationDef.modifiers` | parse:1588-1598 | typecheck:455 | NEVER |
| `OperationDef.response` | parse:1514-1523 | typecheck:452 (unresolved) | NEVER |
| `OperationDef.exit_mappings` | parse:1606-1614 | typecheck:454 (unresolved) | NEVER |
| `OperationDef.transport` | parse:1549 | typecheck:456 | NEVER (service-level used) |
| `ServiceConfig.*` | parse (config_fields) | typecheck:583 | NEVER |
| `AuthConfig.scheme` | parse | typecheck | NEVER (hardcoded `self.auth_token`) |
| `AuthConfig.token_expr` | parse | typecheck | NEVER (hardcoded) |

**ResolvedImport drops span:** `Import { span }` exists
(`src/v2/00_core.dag:99-103`), used for diagnostics at resolve time
(`src/v2/03_resolve.dag:167,188`), but `ResolvedImport` type
(`src/v2/03_resolve.dag:33-37`) omits `span`. Downstream stages cannot
produce span-accurate diagnostics for import-related issues.

**Alias name lost in `type_body_to_expr`:** For `Alias { base }`,
`type_body_to_expr` (`src/v2/04_typecheck.dag:257-258`) returns `base`
directly, discarding the alias name. `type UserId = String` resolves to
`Primitive { name: "String" }` — field types that referenced `UserId`
are emitted as `String`, not `UserId`.

## Pass retrospective

This section synthesizes the outcomes of the passes themselves rather
than adding more file-local findings. The aim is to understand what the
scan log is converging on, what kinds of violations it is best at
surfacing, and which themes are still underrepresented.

### Dominant themes in the pass log

**Fail-open behavior is the main recurring pathology.**
Across tokenize, parse, resolve, typecheck, emit, the runtime shim, and
the evaluator/interpreter, malformed or unsupported input usually
continues as a placeholder, wildcard, empty list, dummy node, identity
transform, or heuristic fallback instead of becoming a hard failure.
The passes are repeatedly finding success-shaped failure.

**Semantic distinctions are being collapsed too early.**
Many findings have the same shape: two distinct source constructs become
indistinguishable before later stages can reason about them. Examples:
bare vs empty imports, `return x` vs `x`, `()` vs empty record,
`pattern`/`interface`/`func`, named vs positional arguments, resource
lifecycle blocks, and anonymous structural types. This is now one of
the clearest cross-pass themes.

**The intermediate models are not authoritative enough.**
Several findings are not just "bad lowering" bugs. They show the AST,
typed graph, or helper result types are too weak to faithfully carry the
language semantics: tokenizer diagnostics have no representation,
resource lifecycle has no core slot, expression-embedded `TypeExpr`s are
not covered, and invalid pipeline states are representable in
`CompileResult`. Downstream heuristics are compensating for missing
structure upstream.

**The typechecker is still mostly a type-reference resolver.**
The pass log now makes this hard to ignore. Top-level declared types are
resolved, but expression trees, many value payloads, transport/config
expressions, and cast targets are largely outside the checking surface.
That pushes semantic debt into emit, where Rust-specific assumptions are
currently standing in for language semantics.

**Comments, helper types, and implementations often disagree.**
`TokenizeResult` exists but is unused, `compile_file()` overclaims its
scope, `merge_envs()` comments do not match implementation, the
post-typecheck validator exists but is bypassed, and emitter comments
claim stronger preconditions than the pipeline enforces. Contract drift
is not incidental; it is one of the recurring sources of invariant
violations.

**Provenance loss deserves to be treated as its own theme.**
The passes keep finding places where the compiler loses "where this came
from": imported module identity, `from_key`, response/exit typing,
named-argument names, resource metadata, and original item kind. This is
related to semantic collapse, but it is specific enough to track on its
own.

**Bootstrap fixes often repair the visible artifact but not the helper state.**
The branch review reinforced a pattern the pass log was already hinting
at: a fix at the emitted-definition layer is not necessarily a fix in
the helper registries that drive codegen. `TypeDefSignature` repaired
the worst same-name type collapse in emitted Rust, but bare-name helper
maps (`struct_field_types`, `struct_field_ir_types`, `optional_fields`,
`variant_to_enum`) can still launder nominal identity or ambiguity
before the final file is written.

**Partially-started IR layers become dangerous once they are reachable.**
`IrType` is valuable progress toward typed `code_ir`, but the branch
review showed the usual compiler failure mode: a representation that is
"not yet authoritative" is relatively safe while dormant and becomes an
actual invariant hazard once new call sites rely on it. The empty-`Vec`
annotation path made that concrete by turning invalid record-type
rendering from a theoretical issue into a reachable compile failure.

### What passes 26-30 addressed (2026-03-14)

Passes 26-30 were run in parallel, targeting the five themes the
retrospective identified as under-sampled. Results:

**Valid-program miscompilation (Pass 26).** Now covered. Key findings:
`for` loops emit as `.map().collect()` (wrong return type, no `?`
propagation), `??` associativity disagrees with all mainstream languages,
`NonEmptyList` downgrades to `Vec`, type aliases lose nominal identity
in signatures, `from_key` loss causes serde miscompilation.

**Negative-space auditing (Pass 27).** Now covered. 38 findings: 3 dead
types, 11 dead functions, 4 never-constructed enum variants, 12
parsed-but-never-consumed fields, 7 duplicate type definitions. The
typecheck module alone has 8 dead functions — a full query API that
nothing calls.

**Determinism and output stability (Pass 28).** Now covered. Two
high-risk items: `infer_struct_name` tiebreaker and `compute_recursive_
fields` DFS roots both depend on `HashMap`/`HashSet` iteration order,
producing non-deterministic output that can change semantics.

**Boundary contract auditing (Pass 29).** Now covered. The headline
finding: the emitter declares `TypeEnv` in its input type but never
reads it, emitting from the raw pre-resolution AST instead. The
typechecker's output is dead data at its most important consumer. Also:
emitter has no diagnostic channel (only stage without one),
`ModuleGraph.diagnostics` silently dropped by typechecker, and the
evaluator's S57 type checks are soft warnings, not enforcement.

**Provenance loss (Pass 30).** Now covered. Systematic trace: 9 field
families are populated at parse, preserved through typecheck, and never
read at emit. The emitter is structurally disconnected from the rich
data earlier stages produce.

### What remains under-sampled

**Diagnostic quality, not just diagnostic existence.**
We have found many places where errors are suppressed or bypassed, but
we have not yet done a systematic audit of whether surviving diagnostics
carry the correct span, module name, blame site, and deduplication
behavior end-to-end.

**Effect/resource enforcement.**
We now know the resource model is being erased (Pass 17, Pass 30), but
we still have not done a systematic pass over whether claims like
`readonly`, `idempotent`, `hermetic`, `kind`, `mode`, and `expires` are
enforced by any downstream stage in a meaningful way.

**Construct coverage as a matrix.**
The passes are still partly opportunistic. A more exhaustive next step
would be a feature matrix:
tokenized -> parsed -> representable in core -> resolved -> validated ->
emitted -> tested. That would expose unsupported language surface more
systematically than ad hoc rereads.

**Test coverage correlation.**
The scan log is now large enough that it should start recording which
findings are protected by regression tests, which are documented only,
and which remain vulnerable because there is no test pressure on that
construct family.

**Target-language leakage.**
The emitter is repeatedly compensating for missing upstream semantics by
making Rust-specific decisions (`Option`, `as`, JSON fallback, trait-only
resource lowering, import widening). That deserves to be tracked as its
own category because it explains a large fraction of later-stage bugs.

### Implication for future passes

If the rereads continue, the highest-yield next themes are probably:

- source-construct coverage matrix across all stages
- provenance retention: names, spans, module identity, field origin,
  item kind
- valid-program semantic miscompilation, not just malformed-input paths
- stage contract audits: comment/type/API vs actual guarantees
- test-pressure mapping: documented debt vs protected behavior

### Category 0: Confirmed invariant violations from follow-up scan

These are not generic bootstrap rough edges; they are places where the
current .dag pipeline claims one invariant and implements another.

**Invariant violation: `merge_envs()` is first-writer-wins, not last-writer-wins.**
`04_typecheck.dag` says later bindings shadow earlier ones, and the
comments explicitly describe a right-to-left dedupe. The implementation
at lines 186-197 does not reverse the bindings before folding, so it
keeps the first occurrence of each name. That means kernel or imported
bindings can incorrectly shadow later local bindings, violating the
documented environment layering (`kernel < imports < local`).

**Invariant violation: `build_type_env()` loses import provenance.**
`ResolvedImport` carries the target module for each import, but
`build_type_env()` ignores it. At lines 213-224, each imported name is
looked up in one merged parent environment built from *all* parent
modules, not in the specific module named by `imp.module_path`. If two
dependencies export the same type name, `import a { Foo }` can bind
`Foo` from `b` instead of `a`. This launders module identity across the
resolve/typecheck boundary.

**Invariant violation: the pipeline emits after resolve/typecheck failure.**
`06_pipeline.dag` only gates on parse errors. Once parsing succeeds,
`compile_sources()` always calls `resolve_modules()`, `typecheck()`, and
`emit_rust()` (lines 99-136), even if resolver or typechecker
diagnostics contain errors. That violates the emitter precondition in
`05_emit.dag` lines 1-11, which says emit receives a fully resolved,
fully typed graph with no remaining ambiguity.

**Concrete bad case: cycle modules get `dep_order = -1` and still flow downstream.**
When topological sort cannot place a module, `find_index_in_list()`
returns `-1` (03_resolve.dag lines 439-444). `resolve_modules()` still
attaches that `dep_order` and sorts the full module list by it (lines
90-104). Combined with the pipeline behavior above, cyclic or otherwise
unsorted modules are pushed to the front of the resolver output and then
typechecked/emitted anyway instead of being quarantined behind the
diagnostic.

**Invariant checker exists but the pipeline does not use it.**
`04_typecheck.dag` defines `typecheck_and_validate()` plus
`validate_no_unresolved()` / `typecheck_ok()` (lines 992-1017), but
`06_pipeline.dag` calls plain `typecheck()`. So even the compiler's own
post-typecheck invariant audit is bypassed on the main pipeline path.

### Category 0b: Semantic deviations in v1 codegen (bootstrap-only)

These are codegen behaviors that produce working Rust but encode
incorrect or non-obvious semantics. They exist because the v1 codegen
lacks type information and compensates with heuristics. All die with
self-hosting.

**`.value` accessor → `.unwrap()` insertion (fn_codegen.rs:466-471).**
When DSL code accesses `.value` on an expression the codegen infers as
optional, it emits `.unwrap()` in Rust. This is a hidden runtime panic
path. `compile_struct_field_value` correctly fails closed with
`compile_error!` for optional-to-required field assignments (line 402),
but the `.value` accessor bypasses that gate. These two paths are
inconsistent: one fails at compile time, the other panics at runtime.
Should be unified — either both fail closed, or both are explicit
about the coercion.

**`map` cardinality preservation (FIXED — regression test locks it).**
`compile_map_intrinsic` previously rewrote `map` to de facto
`filter_map` when the mapper body looked optional, silently dropping
elements and breaking positional/cardinality alignment. This was fixed;
the regression test `map_intrinsic_preserves_optional_mapper_results`
(fn_codegen.rs:3561) now asserts that map is a 1:1 transform. The test
should remain until self-hosting deletes fn_codegen entirely.

**IrType layer is started but not yet authoritative.**
`type_expr_to_ir_type()` (fn_codegen.rs:66-86) only special-cases
`Bool`, `Int`, and `String`; everything else (`Float`, `Bytes`, `Json`,
user-defined types) falls through as `IrType::Named(...)` — a string,
not a structural type. `render_ir_type()` (render_rust.rs:225-254)
renders `IrType::Record` as `{ a: T }`, which is not valid Rust type
syntax. This is now reachable, not hypothetical: the new empty-`Vec`
accumulator annotations in `compile_expr()` can infer element types from
field accesses or lambda bodies, so collections of anonymous records can
render as `Vec<{ ... }>` and fail at compile time. The right framing: we
have *started* a target-agnostic IR layer, and it is not authoritative
yet. It provides correct type annotations for the three primitives and
for Generic/Optional, but is pass-through for everything else.

**Duplicate type suppression is bootstrap-only, even after the
`TypeDefSignature` fix.** The improved structural signature comparison
(v2_crate_emit.rs:553-585) correctly distinguishes same-name types with
different field types. But the mechanism still says "same name + same
structural shape across modules = collapse to one emitted definition,"
which is not a sound nominal type model. Two distinct types can have
identical structure but different semantics. The test
`assemble_v2_crate_keeps_same_name_records_with_different_field_types`
proves the worst case is fixed, but the whole mechanism should stay
labeled as temporary bootstrap scaffolding that dies with self-hosting.

**Cross-module helper maps still key on bare names.**
Even after the emitted-definition fix above, the helper registries that
feed fn_codegen (`struct_field_types`, `struct_field_ir_types`,
`optional_fields`) are still built globally and keyed only by bare type
or variant name. Later modules overwrite earlier entries before the
per-module visibility filter runs, so fn_codegen can still infer fields,
optionality, or return-shape facts against the wrong nominal type. This
means S81 is only partially repaired: the final `struct` definition can
be right while the code that constructs it is still reasoning from the
wrong helper state.

**`v2_runtime_shim::filesystem_read()` panics on failure.**
This performs real I/O and `panic!`s on read errors. It exists to get
the generated compiler working, but it is exactly the kind of
fail-open runtime behavior the project invariants prohibit. It should
be replaced by the I/O transport mechanism (Invariant 2: World I/O is
structural) or at minimum return a Result.

### Category 1: Hardcoded bootstrap scaffolding in v2_crate_emit.rs

These exist because the v1 emitter pipeline works on individual parsed
modules without cross-module knowledge. The v2 compiler's resolve phase
handles this properly — all of these die with self-hosting.

**S78: Materialized types in `std_types_prelude()`.**
Types imported from `std.types` (`SourceSpan`, `FilePath`, `NonEmptyStr`,
`BindingPower`) are hand-written as Rust struct definitions. In the v2
compiler, these come from the .dag type definitions via resolved imports.

**S79: Hardcoded cross-module imports in `module_prelude()`.**
A match statement maps each .dag stem to its `use crate::` statements
(e.g., `02_parse` → `use crate::tokenize::*`). Should be derived from
`import` declarations in each .dag file.

**`V2_MODULE_MAP` constant.**
Hardcoded mapping from .dag file stems to Rust module names (7 entries).
Should be derived from module declarations.

**`struct_field_types` manual entries.**
`BindingPower` and `SourceSpan` field maps are manually inserted into
the struct field type registry. Should come from the type definitions.

**S81: Duplicate type suppression.**
Downstream modules that re-declare structurally identical types get their
definitions suppressed so cross-module references use the upstream type
via `use crate::upstream::*`. Correct but positional — depends on module
processing order matching the dependency graph.

**Helper registries are still same-name / first-definition-wins.**
The v2 crate emitter now preserves same-name types with different
signatures at the final emitted-definition layer, but its supporting
registries still collapse by bare name. `struct_field_types`,
`struct_field_ir_types`, and `optional_fields` are global maps keyed by
type or variant name, so later modules can overwrite earlier helper
entries. This leaves nominal identity partially laundered inside the
bootstrap codegen path even when the emitted Rust type declarations are
no longer wrong.

### Category 2: Runtime shim functions (v2_rt.rs)

Every function in the runtime shim represents something the generated v2
crate can't express in pure generated Rust. Each must either become a
proper stdlib module or be eliminated by better codegen.

**String operations** (needed because .dag treats strings as opaque values):
- `char_at(s, pos)` — character at position
- `string_length(s)` — character count
- `substring(s, start, end)` — character range extraction
- `str_eq(a, b)` — string equality
- `code_point(c)` / `from_code_point(cp)` — Unicode conversion
- `process_escapes(raw)` — escape sequence handling (currently in .dag, shim for compiled path)

**Scanner operations** (tokenizer-specific character scanning):
- `scan_while(s, start, pred)` — scan while predicate holds
- `skip_horizontal_ws(s, start)` — skip spaces/tabs
- `scan_to_eol(s, start)` — scan to end of line
- `scan_string_end(s, start)` — scan string literal with escape handling

**Collection operations:**
- `concat<T: Concat>(a, b)` — polymorphic string/list concatenation
- `lookup<V>(table, key)` — HashMap lookup with clone
- `list_concat<T>(a, b)` — deprecated list concatenation

**Filesystem:**
- `filesystem_read(path)` — file read (panics on error)

### Category 3: Type-unaware codegen heuristics in fn_codegen.rs (S81)

The v1 fn_codegen pipeline compiles .dag function bodies to Rust without
type information. Every decision requiring types is heuristic. All of
these are eliminated by the v2 compiler's typed emitter.

**S76: `clone_if_needed()` — blind ownership.**
Adds `.clone()` to all variable/field expressions passed as arguments or
struct fields. ~300 unnecessary clones in the generated crate. Correct
but inefficient. v2 fix: ownership/liveness tracking.

**S77: `infer_struct_name()` — field-name guessing.**
Anonymous records `{ field: value }` mapped to named Rust structs by
matching field names against known definitions. Wrong when multiple
structs share field names. v2 fix: typechecker resolves anonymous
records to their structural type.

**`infer_scrutinee_type()` / `infer_type_from_arms()`.**
Infers match scrutinee enum type from parameter types, ir_scope, and arm
variant names. Picks enum with best variant overlap; returns None on ties.
v2 fix: typechecker knows the scrutinee type.

**`is_likely_option_receiver()` / `is_likely_option_receiver_ctx()`.**
Detects method chains returning Option (`.last()`, `.first()`, `.get()`,
`.find()`) to convert `.value` field access to `.unwrap()`. v2 fix:
type system tracks optionality.

**`is_already_optional_expr()` / `is_null_ast_expr()`.**
Prevents double-wrapping in `Some()` by detecting expressions that
already produce optional values. v2 fix: type annotations on IR nodes.

**`needs_box_wrapping()`.**
Checks if a field needs `Box<>` wrapping across three naming patterns
(direct, variant-qualified, enum-qualified). v2 fix: recursive type
detection in typechecker.

**`compile_expr_in_field_context()`.**
Compiles expressions with expected field type for variant qualification.
Uses `enum_variants` map to resolve ambiguous variant names. v2 fix:
typechecker resolves variants from context type.

**Ambiguous bare variants still resolve by file order in v2 crate emit.**
The older `type_codegen` path excluded ambiguous entries from
`variant_to_enum`; the newer v2 crate path's `build_variant_to_enum()`
keeps the first enum that declares a variant name. That means adding a
second enum with `Info`, `Error`, etc. can silently retarget existing
bare constructors outside field-context disambiguation. v2 fix: either
typed resolution from context or a fail-closed ambiguity error.

**`infer_collection_element_struct()` / `infer_ast_expr_type()`.**
Core workaround: infers types from ir_scope, struct_field_types, and
function return types without actual type information. v2 fix: typed IR.

**`escape_rust_keyword()`.**
Hardcoded list of ~40 Rust keywords, prefixed with `r#`. v2 fix:
the v2 emitter already has this list in 05_emit.dag line 59-68, but
derived from .dag source.

### Category 4: .dag source workarounds

**C4: Forward declarations in 04_typecheck.dag.**
Lines 33-41 forward-declare types (`ResolvedImport`, `ResolvedModule`,
`ModuleGraph`) that should come from `import v2.compiler.resolve`.
Status: awaiting proper cross-module import in compiled path.

**C5: Forward declarations in 05_emit.dag.**
Lines 29-33 forward-declare types (`TypedGraph`, `TypedModule`,
`TypeEnv`, `TypeBinding`) that should come from
`import v2.compiler.typecheck`. Same root cause as C4.

**S56: Parse error check before module extraction in 06_pipeline.dag.**
Lines 101-106 check for parse errors before extracting modules because
`parse()` returns `{ module: none }` on failure. Extracting before
checking would produce `List<Module?>` — a type violation the evaluator
can't catch. v2 fix: typed Result returns.

**Optional type handling limitation in 04_typecheck.dag.**
Lines 505-518: Uses field access `.value` instead of pattern matching on
`TypeExpr?` because the evaluator represents Optional as
`Map({"value": ...})` without a `_variant` field. Workaround: checks
`expr == none` instead of matching `Some { value: te }`. v2 fix:
compiled code has proper Option types.

**S54: Service parameter forwarding in 05_emit.dag.**
Line 86: builds an item registry mapping item names to their kind and
service dependencies. Used by `emit_call` to forward service params to
callees. This is correct design but could be cleaner with a proper
module-level analysis pass.

### Category 5: code_ir target leakage (S81 — CRITICAL)

The code_ir layer was designed as target-agnostic IR that all backends
(Rust, Go, C, Verilog) can render. During v2 bootstrap, fn_codegen has
injected ~15 Rust-specific constructs directly into the IR:

- `clone_if_needed()` → `.clone()` method calls
- `Box::new()` wrapping → Rust heap boxing
- `Some()`/`None` injection → Rust `Option<T>`
- `.as_str()` insertion → Rust `String` vs `&str`
- `..Default::default()` → Rust struct update syntax
- `LazyLock` for Map data → Rust static initialization
- `Deref`/`*` for Box unwrapping → Rust-specific dereference

**Why this matters:** If the IR contains `"clone"`, `"Box::new"`,
`"Some"`, then every non-Rust backend must enumerate and strip Rust
idioms. The IR has become a Rust AST with extra steps.

**Fix:** The v2 emitter reads computation facts (types, cardinality,
recursion, optionality) and applies rendering facts per-backend. The
code_ir should represent "this value is used here and here" — the
backend decides what that means (Rust: clone, C: nothing, Go: nothing,
Verilog: fan-out wire).

### Category 6: Evaluator limitations (Branch 7)

The v1 evaluator was built for simple fn bodies. Using it for the v2
compiler's 80+ mutually-recursive functions, deep self-recursion, and
multi-stage pipeline contracts exposed:

- **Stack amplification:** Each DSL call pushes ~1.6KB of native Rust
  frames vs ~100-200 bytes compiled. Tests need 16-32MB stacks.
- **No type safety:** `Value::Unit` flows where `Value::Map` (Module)
  is expected with no error until downstream fails (S57, now mitigated).
- **Performance:** `Env::from_inputs` clones on every non-self call.
  Map field flattening clones every field.

**Status:** Mitigated (S55-S61 fixes), not solved. Self-hosting
eliminates this entire category.

## Retro retrospective

This section collapses the scan log into the hottest / largest invariant
families rather than the chronological order of passes.

### 1. Invalid states keep moving forward

This is still the highest-blast-radius cluster. Once a bad state enters
the pipeline, later stages usually keep going with placeholders,
heuristics, or diagnostics that do not actually gate progress.
Representative findings: parse-only gating in `06_pipeline.dag`,
resolver/typechecker diagnostics that do not block emit, cycles getting
`dep_order = -1` and still flowing downstream, the skipped
post-typecheck invariant audit, evaluator soft type-boundary warnings,
and runtime-shim panics instead of typed failure.

### 2. Provenance and nominal identity are laundered too early

This is the hottest semantic cluster after fail-open behavior because it
turns correct later-stage reasoning into an impossible problem. Once the
compiler forgets which module, alias, enum, or field a thing came from,
every later stage is forced into heuristics. Representative findings:
`build_type_env()` ignoring import provenance, `ResolvedImport` dropping
span, alias names disappearing in `type_body_to_expr`, duplicate type
suppression, helper registries keyed by bare names, and ambiguous
variant resolution falling back to first-definition-wins.

### 3. Intermediate representations exist but are not authoritative

The project now has several half-authoritative models: `TokenizeResult`,
`TypedGraph`, `TypeEnv`, `IrType`, helper registries for fn_codegen, and
the typed output of the typechecker. The recurrent problem is not "no
model exists" but "the model exists and downstream stages do not trust
or consume it as source of truth." Representative findings: emit
re-deriving types from raw AST instead of `TypeEnv`, local redefinition
of typechecker output types in `05_emit.dag`, the partly-started
`IrType` layer, and helper maps compensating for missing nominal data.

### 4. Rust-specific codegen heuristics are standing in for language semantics

This is the largest branch-local source of bootstrap debt. It explains a
large share of the remaining silent-wrong-behavior risk even when the
pipeline "works." Representative findings: `.value` becoming
`.unwrap()`, `clone_if_needed()`, `infer_struct_name()`,
`compile_expr_in_field_context()`, `needs_box_wrapping()`, `Some`/`None`
injection, and Rust-specific constructs leaking directly into `code_ir`.
The new helper-map and ambiguous-variant findings both strengthen this
theme: the codegen is still resolving language meaning through Rust-side
guesswork rather than typed source semantics.

### 5. Diagnostic channels are weaker than the invariants they claim to defend

This cluster is slightly cooler than the four above, but it is the one
that most directly blocks ratcheting. The compiler often has enough
information to know something went wrong but either does not propagate
it, does not attach the right provenance, or has no representation for
the failure at all. Representative findings: emit has no diagnostic
channel, resolve diagnostics are dropped by typecheck, import span is
lost before downstream use, diagnostic ordering does not match stage
order, and diagnostic quality remains under-audited relative to raw
existence checks.

---

## Design decisions (locked in 2026-03-14)

### D1: TypedGraph is the compiler boundary — emit is per-backend

`04_typecheck.dag` outputs `TypedGraph` (fully resolved, no unresolved
Named references). `05_emit.dag` = `emit_rust()` — explicitly a Rust
backend. `06_pipeline.dag` has `type Backend = Rust | Python` with a
match dispatch.

Adding a backend means adding one `.dag` module + one match arm. The
typecheck output is the contract. Code upstream of emit is
target-agnostic; code in emit is target-specific. No exceptions.

**Implication for S81:** The code_ir target leakage in v1 is moot — it
dies with fn_codegen. The v2 emitter is *correctly* Rust-specific
because it IS the Rust backend. We don't need to clean up code_ir; we
need to delete it.

### D2: v1 is frozen — no parallel implementations

v1's fn_codegen is a parallel implementation of v2's 05_emit.dag. The
only justified v1 changes are those that unblock the self-hosting
fixed point. No heuristic improvements, no new workarounds. Once
self-hosting lands, delete entirely.

### D3: Self-hosting equivalence is behavioral, not textual

The v2 emitter may produce intentionally different (better) Rust than
v1. The fixed-point comparison happens at the *pipeline output* level:

1. Both compile (`cargo check` passes)
2. Same observable behavior (same tokens/AST/types on same .dag input)
3. NOT character-identical source

This frees the v2 emitter to improve codegen quality without
artificial constraints.

### D4: Runtime shims → first-class language features, not annotations

**Principle:** Solve through first-class features before reaching for
annotations or `extern func` declarations. Each annotation pattern must
be justified by a concrete failure of first-class features.

String operations map to language features with existing analogs:

| Current shim call | First-class feature | Rationale |
|---|---|---|
| `char_at(s, pos)` | `s[pos]` — indexing syntax | Universal; renders per-backend |
| `string_length(s)` | `count(s)` — extend existing intrinsic | `count` already works on lists |
| `substring(s, start, end)` | `s[start..end]` — slice syntax | Natural extension of indexing |
| `lookup(table, key)` | `table[key]` — indexing syntax | Same syntax as strings |
| `code_point(ch)` / `from_code_point(cp)` | Cast syntax or known intrinsic | Type conversion |
| `concat` | Already first-class | No change |
| `scan_while`, `skip_horizontal_ws`, etc. | Already .dag functions | Shim exists only because v1 can't compile the .dag versions |

Indexing and slicing are computation facts ("access element at
position"), not rendering decisions. Each backend renders in its idiom.

### D5: Forward declarations are temporary — imports are the only mechanism

C4 (typecheck.dag) and C5 (emit.dag) have forward-declared types that
are exact copies of types in resolve.dag and typecheck.dag. This is a
duplicate representation. Once the compiled v2 crate has working
cross-module imports, the .dag source switches to `import` and the
duplicated types are deleted. No formalization of forward declarations
as a language feature.

### D6: The compiler tests itself through generated tests

**Principle:** The v2 compiler emits tests alongside code. When it
compiles itself, it must also emit tests that verify the compiled
compiler works correctly. The compiler is not special — it is a .dag
program like any other, and the same testing contract applies.

**Current gap:** The generated v2 crate has 3 hand-written smoke tests
injected as string literals from the v1 test harness
(`tests/src/lib.rs:2353-2387`). These test only trivial tokenizer
behavior. The semantic issues found in passes 26-30 (for-loop
miscompilation, `??` associativity, NonEmptyList erasure, provenance
loss, 12 dead fields) are invisible because no test exercises them.

**What "generated testing" means for the compiler:**

1. **Stage-level round-trip tests.** The emitter knows the v2 pipeline's
   stage contracts. For each stage function it emits, it should also emit
   a test that feeds known input and checks known output. Example: emit
   a test that tokenizes `"fn foo() -> Int"` and asserts the token kinds
   match `[KwFn, Ident, LParen, RParen, Arrow, Ident, Eof]`. The test
   input and expected output come from the .dag source's own test data
   or from `mock_response`-style annotations.

2. **Pipeline integration tests.** Feed a small .dag file through the
   full compiled pipeline (tokenize → parse → resolve → typecheck →
   emit) and assert the output files are non-empty and valid. This is
   the behavioral equivalence test from D3, emitted as a Rust `#[test]`.

3. **Self-compilation smoke test.** The ultimate generated test: the
   compiled v2 compiler tokenizes/parses one of its own .dag source
   files and asserts the output matches what the v1 interpreter produced.
   This is the fixed-point seed.

**Why this matters now:** The scan passes found ~30 semantic issues that
`cargo check` cannot catch. The purpose of this project is to eliminate
glue bugs through structural correctness, and the compiler's own test
gap is itself a glue bug — the emitter produces code but no evidence
that the code is correct. Closing this gap is prerequisite to trusting
self-hosting output.

### D7: `data` definitions → backend renders per-idiom

`data` in the typed IR is a constant definition. The Rust emitter
renders as `lazy_static!` / `const` / `static`. A Go emitter renders
as package-level `var`. 05_emit.dag already handles this correctly via
`emit_data_def()`. No backport to v1 — let it die.

---

## Follow-on work (ordered by dependency)

### Phase 1: Generated tests for the compiled v2 crate (D6)

**Goal:** The compiled v2 compiler verifies its own correctness through
emitted tests, not hand-written string-literal smoke tests.

1. **Emit per-stage unit tests.** The v1 crate assembly emits a test
   module alongside each compiled .dag module. Each test calls the
   module's entry function with known input and asserts expected output.
   Start with tokenize (known input→expected token list) since the
   interpreter already validates this path.

2. **Emit pipeline integration test.** A test that feeds a small .dag
   snippet through the full compiled pipeline (tokenize → parse →
   resolve → typecheck → emit) and asserts the output is non-empty
   valid Rust.

3. **Emit self-parse test.** The compiled v2 compiler tokenizes and
   parses one of its own .dag source files. Assert the token count and
   module name match what the v1 interpreter produces. This is the
   behavioral equivalence seed from D3.

4. **Replace injected smoke tests.** Delete the 3 hand-written smoke
   tests in `tests/src/lib.rs` and replace with the emitted tests above.
   The test harness runs `cargo test` on the generated crate — same
   mechanism, but the tests come from the compiler, not from the host.

**Exit criterion:** `cargo test` on the generated v2 crate runs ≥10
emitted tests covering tokenize, parse, and at least one pipeline
round-trip. All pass.

### Phase 2: Self-hosting fixed point

**Goal:** The v2 compiler compiles itself. Behavioral equivalence with
v1 output (D3).

1. **Run v2 pipeline end-to-end on a trivial .dag file.** The Phase 1
   emitted tests already prove tokenize works. Extend to prove `parse`,
   `resolve`, `typecheck`, and `emit` work by feeding the tokenizer
   output through each stage and checking the output.

2. **Run v2 pipeline on v2 source.** Feed the 7 .dag files through the
   compiled v2 pipeline. Compare pipeline output (tokens, AST, types)
   with v1's output. Both should produce working code.

3. **Fixed-point test.** Compile v2 with v1 → get binary B1. Compile v2
   with B1 → get binary B2. B1 and B2 should produce identical output
   on the same input.

### Phase 3: Eliminate runtime shims via language features (D4)

Add first-class language features to replace v2_rt.rs:

1. **Add Index expression** to AST/parser/typechecker/emitter —
   `s[pos]` for strings, `table[key]` for maps.
2. **Add Slice expression** — `s[start..end]` for substring.
3. **Extend `count()` intrinsic** to strings (already works on lists).
4. **Handle `code_point`/`from_code_point`** — cast syntax or known
   intrinsic (decide when the need arises).
5. **Migrate tokenizer** — `char_at(s, pos)` → `s[pos]`,
   `string_length(s)` → `count(s)`, `substring(s, start, end)` →
   `s[start..end]`, `lookup(table, key)` → `table[key]`.
6. **Scanner functions** (`scan_while`, `skip_horizontal_ws`, etc.)
   are already .dag — they'll just compile correctly once self-hosting
   works. No language feature needed.
7. **`filesystem_read`** → I/O transport (existing mechanism).
8. **Delete v2_rt.rs.**

### Phase 4: Eliminate bootstrap scaffolding (S78, S79)

1. **Derive `module_prelude()` from .dag imports.** Read `import`
   declarations from each module's AST, map to `use crate::` statements.
   Delete the hardcoded match statement.

2. **Derive `std_types_prelude()` from std .dag files.** Parse the std
   type definitions and emit them. Delete hand-written struct defs.

3. **Derive `V2_MODULE_MAP` from module declarations.** Read `module`
   declarations and derive stem→name mappings.

4. **Delete manual `struct_field_types` entries.** All field type maps
   should come from `build_struct_field_types()` over parsed type defs.

### Phase 5: Resolve .dag source forward declarations (C4, C5)

1. **Wire cross-module imports in the compiled v2 crate.** When the
   compiled crate's modules can import from each other, replace forward
   declarations in 04_typecheck.dag and 05_emit.dag with actual imports.

2. **Delete the forward-declared types.** Once imports work, the
   duplicated type definitions in typecheck and emit modules can be
   removed.

### Phase 6: Clean up code_ir target leakage (S81)

This is the architectural cleanup. Every Rust-specific construct
currently injected into code_ir by fn_codegen must be moved to
`render_rust.rs` (or deleted when v2 emitter replaces fn_codegen).

1. **Audit all `code_ir::Expr` and `code_ir::Stmt` variants** for
   target-specific content.
2. **Add target-agnostic IR nodes** for ownership, optionality,
   recursion where needed.
3. **Move rendering decisions to backends.** Each backend interprets
   the target-agnostic nodes in its own idiom.

**Note:** This is largely moot if the v2 emitter replaces fn_codegen
entirely. The v2 emitter in 05_emit.dag already reads typed IR and
makes per-target decisions. The cleanup is only needed if fn_codegen
persists beyond bootstrap.

### Phase 7: Delete v1 bootstrap code

Once self-hosting is proven and tested:

- `fn_codegen.rs` — entire file
- `v2_crate_emit.rs` — entire file
- `v2_runtime_shim.rs` — entire file
- All heuristic functions: `infer_struct_name`, `clone_if_needed`,
  `is_option_expr`, `is_none_expr`, `is_likely_option_receiver`,
  `needs_box_wrapping`, `is_already_optional_expr`,
  `compile_expr_in_field_context`, `infer_ast_expr_type`,
  `escape_rust_keyword`, `infer_scrutinee_type`, `infer_type_from_arms`,
  `synthesize_anonymous_structs`
- `std_types_prelude()`, `module_prelude()`, `V2_MODULE_MAP`
- Phase 3 interpreter tests (replaced by compiled tests)
- `with_parser_stack(16MB)` scaffolding

### Phase 8: Capabilities that would further reduce debt

From SUSTAINABILITY.md — not blockers, but would accelerate:

- **Language model serialization to JSON IR** — backend type mappings
  as data, not code
- **`behavior` DSL construct** — algebraic property test enumeration
- **Structural coercion paths** — eliminate `is_compatible` case
  enumeration

---

## Sustainability ledger cross-reference

| S-ID | Status | Description |
|------|--------|-------------|
| S76 | OPEN | `clone_if_needed()` blind ownership — dies with self-hosting |
| S77 | OPEN | `infer_struct_name()` field-name guessing — dies with self-hosting |
| S78 | OPEN | Materialized types in `std_types_prelude()` — Phase 3 above |
| S79 | OPEN | Hardcoded `module_prelude()` imports — Phase 3 above |
| S81 | OPEN | fn_codegen emits Rust, not code_ir — Phase 5 above / dies with self-hosting |
| S80 | DONE | Untyped `PR.val: Map` → 45 typed result types |
| S75 | DONE | `+` operator overloaded → `concat()` intrinsic |
| S54 | DONE | Service param forwarding → item registry |
| S55 | DONE | TCO for self-recursive tail calls |
| S56 | DONE | Parse error laundering → explicit error gates |
| S57 | DONE | No runtime type enforcement → `check_call_inputs()` |
| S58-S61 | DONE | Evaluator performance fixes |
| C4 | OPEN | Forward declarations in 04_typecheck.dag — Phase 4 above |
| C5 | OPEN | Forward declarations in 05_emit.dag — Phase 4 above |

## Error count progression

```
2204 → 829 → 279 → 258 → 231 → 223 → 115 → 36 → 32 → 10 → 7 → 0
```

Key inflection points:
- 2204 → 829: Optionality tracking (check `T?` annotations, prevent double-wrapping)
- 829 → 115: Box wrapping, runtime shim imports, string match, variant disambiguation
- 115 → 36: S76 clone_if_needed, S80 typed parse results, anonymous struct naming
- 36 → 0: Optionality mismatches, ServiceCall lowering, parser bug (`{ "string" }`
  parsed as empty record), fold Vec inference, .dag source fixes
