# v2 Compiler Postmortem — Bootstrap to cargo check (2026-03-14)

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

### What the current passes are under-sampling

**Valid-program miscompilation.**
The current manual reread method is strongest at finding silent drops,
placeholder fabrication, dead helper types, and stage-contract
mismatches visible in control flow. It is weaker at catching programs
that are fully well-formed and still compile to the wrong meaning.

**Diagnostic quality, not just diagnostic existence.**
We have found many places where errors are suppressed or bypassed, but
we have not yet done a systematic audit of whether surviving diagnostics
carry the correct span, module name, blame site, and deduplication
behavior end-to-end.

**Determinism and output stability.**
A few ordering issues surfaced, but there has not yet been a full
determinism pass over import order, sort keys, field ordering, registry
buildup, and emitted file stability for semantically identical inputs.

**Effect/resource enforcement.**
We now know the resource model is being erased, but we still have not
done a systematic pass over whether claims like `readonly`,
`idempotent`, `hermetic`, `kind`, `mode`, and `expires` are enforced by
any downstream stage in a meaningful way.

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

### Missing themes to name explicitly

**Boundary contract auditing.**
Some of the highest-value findings came from comparing a stage's public
contract with what the implementation actually guarantees. That should
be an explicit scan theme, not just an emergent one.

**Negative-space auditing.**
We should explicitly look for fields, variants, tokens, item kinds, and
helper result types that are declared but never consumed, or consumed
but never produced. `TokenizeResult` is the clearest example so far, but
it is unlikely to be the only one.

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
syntax if it ever becomes a let-binding annotation. The right framing:
we have *started* a target-agnostic IR layer, and it is not
authoritative yet. It provides correct type annotations for the three
primitives and for Generic/Optional, but is pass-through for everything
else.

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

### D6: `data` definitions → backend renders per-idiom

`data` in the typed IR is a constant definition. The Rust emitter
renders as `lazy_static!` / `const` / `static`. A Go emitter renders
as package-level `var`. 05_emit.dag already handles this correctly via
`emit_data_def()`. No backport to v1 — let it die.

---

## Follow-on work (ordered by dependency)

### Phase 1: Self-hosting fixed point

**Goal:** The v2 compiler compiles itself. Behavioral equivalence with
v1 output (D3).

1. **Run v2 pipeline end-to-end on a trivial .dag file.** The smoke
   test proves `tokenize` works. Next: prove `parse`, `resolve`,
   `typecheck`, and `emit` work by feeding the tokenizer output through
   each stage and checking the output.

2. **Run v2 pipeline on v2 source.** Feed the 7 .dag files through the
   compiled v2 pipeline. Compare pipeline output (tokens, AST, types)
   with v1's output. Both should produce working code.

3. **Fixed-point test.** Compile v2 with v1 → get binary B1. Compile v2
   with B1 → get binary B2. B1 and B2 should produce identical output
   on the same input.

### Phase 2: Eliminate runtime shims via language features (D4)

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

### Phase 3: Eliminate bootstrap scaffolding (S78, S79)

1. **Derive `module_prelude()` from .dag imports.** Read `import`
   declarations from each module's AST, map to `use crate::` statements.
   Delete the hardcoded match statement.

2. **Derive `std_types_prelude()` from std .dag files.** Parse the std
   type definitions and emit them. Delete hand-written struct defs.

3. **Derive `V2_MODULE_MAP` from module declarations.** Read `module`
   declarations and derive stem→name mappings.

4. **Delete manual `struct_field_types` entries.** All field type maps
   should come from `build_struct_field_types()` over parsed type defs.

### Phase 4: Resolve .dag source forward declarations (C4, C5)

1. **Wire cross-module imports in the compiled v2 crate.** When the
   compiled crate's modules can import from each other, replace forward
   declarations in 04_typecheck.dag and 05_emit.dag with actual imports.

2. **Delete the forward-declared types.** Once imports work, the
   duplicated type definitions in typecheck and emit modules can be
   removed.

### Phase 5: Clean up code_ir target leakage (S81)

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

### Phase 6: Delete v1 bootstrap code

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

### Phase 7: Capabilities that would further reduce debt

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
