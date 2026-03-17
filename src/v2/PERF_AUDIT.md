# v2 Compiler Performance Audit

Five passes completed across the runtime compiler path:
`00_core.dag`, `01_tokenize.dag`, `02_parse.dag`, `03_resolve.dag`,
`04_infer.dag`, `05_emit.dag`, `05_emit_rust.dag`,
`05_emit_python.dag`, `05_emit_go.dag`, and `06_pipeline.dag`.

`DESIGN.md` was read as the intended contract reference. `tests/` was not
ranked because it does not execute on the self-compile hot path.

## Symbols

- `C`: total source characters across all input files
- `T`: total tokens
- `M`: modules
- `I`: import edges plus imported names
- `N`: AST / typed-expression / item nodes
- `K`: type bindings
- `B`: emitted output bytes / lines
- `P`: params or args at one call site
- `S`: statements in one block

## Executive Summary

The self-host bottlenecks are not one bug. They cluster into five repeatable
patterns:

1. Quadratic builders on lists and strings
2. Linear scans where the code wants set / map lookup
3. Re-inference or re-walk of the same subtree
4. Typed-to-untyped full-tree cloning
5. Backend duplication of the same bad block-emission pattern

The highest leverage fixes are still:

1. `04_infer.dag`
2. `01_tokenize.dag`
3. `03_resolve.dag`
4. `05_emit*.dag`
5. `02_parse.dag` and `06_pipeline.dag` only after the above

## Pass 1: Stage I/O Contracts

- `00_core.dag`
  Main I/O: support conversions, especially `TypedExpr -> Expr`.
  Contract: `typed_expr_to_expr` (`361-381`) is `O(N)` time and `O(N)`
  space per call because it clones the whole typed tree.

- `01_tokenize.dag`
  Main I/O: `String -> List<Token>`.
  Intended: `O(C)` time, `O(T)` space.
  Actual today: `O(C^2)` in the main token list builder plus `O(L^2)` for
  long string literal bodies, where `L` is literal length.

- `02_parse.dag`
  Main I/O: `List<Token> -> Module`.
  Intended: `O(T)` time, `O(T)` output, bounded recursion per grammar shape.
  Actual today: mostly holds. The parser is the cleanest stage. Exceptions are
  localized helper costs, not stage-wide collapse.

- `03_resolve.dag`
  Main I/O: `List<Module> -> ModuleGraph`.
  Intended: `O(M + I)`.
  Actual today: list-backed Kahn bookkeeping pushes the sort toward `O(M^4)`
  in the worst case, with extra `O(M^2)` duplicate and import-name checks.

- `04_infer.dag`
  Main I/O: `ModuleGraph -> TypedGraph`.
  Intended: `O(N + K + type_refs)`.
  Actual today: repeatedly superlinear. Match arms, string interpolation,
  block bodies, and record literals all do extra work. Several hot lookups are
  linear scans or `skip(... ) |> first` indexing.

- `05_emit.dag`
  Main I/O: shared emitter helpers on typed graphs / exprs.
  Intended: `O(N + B)`.
  Actual today: shared helpers add `O(P^2)` arg ordering, `O(U^2)` string
  dedupe, and `O(D^2)` integer formatting.

- `05_emit_rust.dag`
  Main I/O: `TypedGraph -> List<TextFile>`.
  Intended: `O(N + B)`.
  Actual today: many function bodies emit in `O(B^2)` because block text is
  assembled by repeated string concat.

- `05_emit_python.dag`
  Main I/O: `TypedGraph -> List<TextFile>`.
  Intended: `O(N + B)`.
  Actual today: same `O(B^2)` block builder pattern as Rust.

- `05_emit_go.dag`
  Main I/O: `TypedGraph -> List<TextFile>`.
  Intended: `O(N + B)`.
  Actual today: same `O(B^2)` block builder pattern as Rust/Python.

- `06_pipeline.dag`
  Main I/O: `List<SourceFile> x RenderTarget -> CompileResult`.
  Intended: sum of stage costs.
  Actual today: orchestration is clean. Only small local list-copy cost in
  diagnostic accumulation.

## Pass 2: Builder / Accumulator Growth

### Critical

- `01_tokenize.dag`
  `tokenize_loop`, `emit`, `scan_ident`, `scan_number`, `scan_string`,
  `scan_str_cont` all prepend tokens with `concat([tok], state.tokens)` and
  then reverse once at the end (`88`, `105`, `126`, `177`, `186`, `207`,
  `220`, `233`, `245`, `276`, `288`, `299`, `319`, `331`, `342`).
  Because `concat` copies both lists, every prepend copies the full token
  accumulator. This is quadratic in token count.

- `01_tokenize.dag`
  `scan_string_body` builds literal content one character at a time with
  string concat (`355-382`). `process_escapes_loop` repeats the same pattern
  (`409-423`). Both are quadratic in literal length.

- `05_emit_rust.dag`
  `emit_block` (`1045-1052`) and `emit_typed_block` (`1366-1373`) build
  output via `concat(acc.text, "\n", line)`. The same pattern appears inside
  function-body and TCO helpers (`550`, `1049`, `1370`, `1444`).

- `05_emit_python.dag`
  `emit_py_block` (`842-849`) and `emit_py_typed_block` (`1117-1124`) use the
  same repeated string append pattern. The same issue appears in async body /
  TCO helpers (`454`, `846`, `1121`, `1158`, `1231`).

- `05_emit_go.dag`
  `emit_go_block` (`852-859`) and `emit_go_typed_block` (`1136-1143`) use the
  same repeated string append pattern. The same issue appears in func body /
  TCO helpers (`458`, `856`, `1140`, `1177`, `1250`).

### High

- `04_infer.dag`
  Block inference appends chunk lists on every statement:
  `concat(acc.diag_chunks, [stmt_result.diagnostics])` and
  `concat(acc.entry_chunks, [stmt_result.type_entries])` (`1498-1500`).
  Even before the second block pass, this makes bookkeeping inside one block
  `O(S^2)` in statement count.

- `05_emit.dag`
  `to_string_helper` (`553-564`) is quadratic in digit count because it does
  `concat(ch, acc)` per digit. The parser has the same helper shape in
  `02_parse.dag` (`1799-1812`).

### Minor

- `06_pipeline.dag`
  `collect_diagnostics` appends with `concat(acc, [diag])` (`42-47`). Since
  parse is first-error-halt, this is bounded by file count, but it is still
  the same right-append copy pattern.

- `02_parse.dag`
  Most accumulators are correct functional list builders:
  `concat([x], acc)` plus a final `reverse()`. This stage is mostly clean.

## Pass 3: Lookup / Indexing Pathologies

### Critical

- `03_resolve.dag`
  Kahn's algorithm is fully list-backed (`335-455`).
  `all_edges |> filter(... ) |> count` for initial in-degree (`346`),
  `final_state.sorted |> any(...)` for cycle-membership (`366`),
  `get_at_index_int` plus `zero_nodes |> any(...)` inside each step
  (`403-433`), and list-filter-based index helpers (`462-495`) compound into
  worst-case `O(M^4)`.

### High

- `03_resolve.dag`
  Import-name validation scans `exported` for every imported name
  (`196-204`): `specific_names |> filter(name => exported |> any(...))`.
  This is `O(imported_names * exported_names)` per import.

- `03_resolve.dag`
  Duplicate module detection uses a `List<String>` as a set
  (`291-313`). Membership is linear, and `seen_names` is right-appended.

- `04_infer.dag`
  Field / variant lookup is scan-heavy:
  `find_field_in_list` (`468-480`),
  `find_variant_in_type` (`1102-1116`),
  `find_field_type` (`1119-1127`).
  These all use `filter(... ) |> first` when the code wants first-match or a
  map lookup.

- `04_infer.dag`
  Result indexing by `skip(..., pair.first) |> first` appears in four hot
  inference paths:
  call args (`1248`),
  method call args (`1282`),
  record literal typed fields (`1585`),
  record literal field types (`1597`).
  That turns a linear result list into repeated positional scans.

- `04_infer.dag`
  Recursive-type membership and cycle detection are list-based:
  `reaches_self` visited scan (`1803-1815`),
  cycle detection root walk (`1832`),
  `env.recursive_types |> any(...)` in resolution (`2035`) and later variant
  registration (`2742`, `2761`).

- `05_emit.dag`
  `order_call_args` and `order_typed_call_args` (`172-224`) repeatedly
  `filter(... ) |> first` for every param and then re-scan params to compute
  leftovers. This is `O(P * args)` before counting list-copy overhead.

- `05_emit.dag`
  `unique_strings` (`364-372`) uses `any` over the growing accumulator, giving
  `O(U^2)` dedupe.

### Medium

- `05_emit_rust.dag`
  `starts_with_prefix` (`1600-1612`) compares characters by nested
  `enumerate |> filter(... ) |> first`. This is avoidable `O(|s| * |prefix|)`.

- `02_parse.dag`
  `looks_like_arm_start` and `scan_braces_depth` (`3166-3247`) can rescan long
  braced spans while collecting implicit match-arm blocks. This is not a top
  self-host cost today, but the worst case is superlinear on large match arms.

## Pass 4: Redundant Traversals and Full-Tree Cloning

### Critical

- `04_infer.dag`
  Match arms are inferred twice. One pass builds `typed_arms`; another pass
  re-infers the arm bodies to collect diagnostics / result type
  (`1298-1335`, especially `1304-1328`).

- `04_infer.dag`
  String interpolation is inferred twice. `typed_parts` re-infers each
  interpolation (`1462-1467`), then `interp_results` does it again
  (`1468-1474`).

- `04_infer.dag`
  Blocks are inferred twice. The fold threads scope and diagnostics
  (`1484-1500`), then `block_stmts_typed = stmts |> map(...)` re-infers every
  statement from the original scope (`1503`). This is both a perf issue and a
  scope-threading correctness risk.

- `04_infer.dag`
  Record literal inference uses the same `fi_results` twice, but each use
  re-finds the matching element by positional skip (`1583-1609`).

### High

- `00_core.dag`
  `typed_expr_to_expr` (`361-381`) is a full typed-tree clone. It is not bad
  by itself; the problem is how often other stages still call it.

- `04_infer.dag`
  Parent-module state is traversed repeatedly:
  `service_registry` build (`2680-2691`),
  `service_locals` build (`2706-2718`),
  `variant_locals` over resolved bindings (`2722-2733`),
  recursive variant recovery from parents (`2757-2768`).
  These should be fused or cached once per module.

- `04_infer.dag`
  Validation walks typed expressions, then converts each typed expression back
  to plain `Expr` and walks it again:
  `collect_unresolved_in_typed_expr` (`3202-3206`).
  That is an avoidable full extra clone + traversal after typecheck.

- `05_emit_rust.dag`
  Typed-to-untyped fallback cloning still exists in hot or semi-hot paths:
  TCO fallback (`483`),
  workflow service discovery fallback (`515`),
  nested-record data JSON emission (`1783`),
  workflow collection (`1920-1921`).

- `05_emit_python.dag`
  Same fallback pattern:
  TCO fallback (`392`) and workflow service discovery fallback (`423`).

- `05_emit_go.dag`
  Same fallback pattern:
  TCO fallback (`393`) and workflow service discovery fallback (`427`).

### Medium

- `04_infer.dag`
  `resolve_expr_types` is intentionally a full recursive walk (`2367-2538`),
  but it means the compiler currently pays for:
  `resolve_item_types` tree walk,
  then `infer_items`,
  then `validate_no_unresolved`.
  The stage is architecturally multi-pass even before the duplicate local work
  above is counted.

## Pass 5: Backend-Wide Pattern Synthesis

- The Rust, Python, and Go emitters all have the same core disease:
  `Expr` and `TypedExpr` rendering paths coexist, and both still keep their
  own block builder. The per-backend syntax is different, but the performance
  shape is identical.

- Shared helper debt lives in `05_emit.dag`, not in any one backend:
  arg ordering, string dedupe, integer formatting, service-call collection,
  and some typed/untyped fallback decisions are centralized. Fixing those once
  removes cost from all three targets.

- Backend-specific ranking:
  Rust has one extra medium hotspot in `starts_with_prefix` (`1600-1612`).
  Python and Go do not have unique algorithmic issues as severe as the shared
  block-builder pattern.

- Good news:
  service-call walkers (`collect_service_calls`, `collect_typed_service_calls`)
  are single-pass recursive traversals and are not the main problem.
  The big backend costs come from builders and helper lookups, not from the
  recursive walkers themselves.

## File-by-File Verdict

- `00_core.dag`
  Mostly support code. The important perf fact is that `typed_expr_to_expr`
  is expensive enough that callers should treat it as a last resort.

- `01_tokenize.dag`
  Architecturally supposed to be linear. Currently not linear.

- `02_parse.dag`
  Mostly good. Keep its prepend-and-reverse style. Only fix the helper hot
  spots, not the whole parser.

- `03_resolve.dag`
  Needs a data-structure rewrite, not micro-tuning.

- `04_infer.dag`
  Worst file by far. This is where self-host hours disappear.

- `05_emit.dag`
  Shared helpers are amplifiers. Small fixes here pay three times.

- `05_emit_rust.dag`
  Major cost is block text assembly; remaining typed/untyped fallback cleanup
  is secondary.

- `05_emit_python.dag`
  Same as Rust: block emission first, fallback cleanup second.

- `05_emit_go.dag`
  Same as Rust/Python: block emission first.

- `06_pipeline.dag`
  Fine. Do not spend time here until the stage bodies are fixed.

## Priority Order

1. `04_infer.dag`
   Remove duplicate match / string-interp / block inference.
   Replace `skip(... ) |> first` result indexing with single-pass zips or
   indexed maps.
   Convert recursive-type membership and visited sets to maps.
   Fuse parent traversals and delete `typed_expr_to_expr` from validation.

2. `01_tokenize.dag`
   Stop copying the token list on every token.
   Stop building strings with repeated `concat`.

3. `03_resolve.dag`
   Rewrite Kahn using maps / sets and adjacency indexes.
   Stop using list scans for duplicates and imported-name validation.

4. `05_emit.dag` plus all `05_emit_*`
   Replace block string accumulation with `List<String>` + `join`.
   Fix arg ordering and `unique_strings` in the shared helper layer.
   Reduce remaining typed-to-untyped fallback clones.

5. `02_parse.dag` and `06_pipeline.dag`
   Fix only the localized helper issues after the major stages are stable.

## Suggested Complexity Targets

- Tokenize: `O(C)` time, `O(T)` space
- Parse: `O(T)` time, `O(T)` space
- Resolve: `O(M + I)` time, `O(M + I)` space
- Infer: `O(N + K + type_refs)` time, `O(N + K)` space
- Emit: `O(N + B)` time, `O(B)` incremental builder space
- Pipeline: sum of stage costs, no extra superlinear work
