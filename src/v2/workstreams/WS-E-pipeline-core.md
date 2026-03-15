# WS-E: Pipeline & Core Contracts

## Scope

Fix fail-open gating in the pipeline, diagnostic threading, and core type model
gaps.

## Files

- `src/v2/06_pipeline.dag`
- `src/v2/00_core.dag`

## Key invariant

> **No fallbacks that fabricate.** Every code path succeeds fully or fails with a
> clear error.

## Status note

`06_pipeline.dag` has been significantly cleaned up (108 lines). Several items
from the original plan may already be resolved. Verify each item against the
current code before changing it.

## Pipeline items (06_pipeline.dag)

1. **Pipeline error gating completeness**
   - Verify all stages have proper error gates (parse, resolve, typecheck)
   - Current code gates on parse errors (line 63-69) and resolve errors (77-83)
     and typecheck errors (91-97) — this may already be complete
   - Verify: does the pipeline proceed to emit if any prior stage produced errors?

2. **`typecheck_and_validate` call verification**
   - Pipeline should call `typecheck_and_validate`, not bare `typecheck()`
   - Current code at line 88: `let typed = typecheck_and_validate(graph: graph)` — appears correct
   - Verify this is the validated version that runs `validate_no_unresolved`

3. **Diagnostic concatenation order** (line 105)
   - `concat(parse_diagnostics, graph.diagnostics, typed.diagnostics, emit_result.diagnostics)`
   - Verify diagnostics from all stages are preserved and ordered correctly

4. **`TokenizeResult` consumption**
   - The tokenizer returns raw token lists; verify the pipeline handles tokenizer
     errors (not just parse errors)
   - Currently `tokenize()` returns tokens directly — if it can produce diagnostics,
     those need to be collected

## Core type model items (00_core.dag)

1. **`CompileResult` allows invalid state**
   - `CompileResult { files, diagnostics }` permits files + errors simultaneously
   - The pipeline code prevents this (gates on errors), but the type doesn't
   - Fix: consider a sum type: `Success { files, warnings }` | `Failure { diagnostics }`

2. **`Module.span` is single-token, not file-covering**
   - `SourceSpan` for a module should cover the entire file
   - Fix: set span to cover from first to last token

3. **`Module.name` is unvalidated String**
   - Any string can be a module name, including empty strings
   - Fix: consider a `ModuleName` newtype with validation, or validate at parse time

4. **`ResourceDef` missing `acquire`/`release` slots** (`00_core.dag` ~line 112)
   - `ResourceDef` has `name`, `properties`, `capabilities`, `span`
   - But no slots for `acquire`/`release` lifecycle operations
   - Fix: add lifecycle operation fields if the language supports them

5. **Missing `PatternDef`/`InterfaceDef` in Item union** (`00_core.dag`)
   - The `Item` type may not have distinct variants for patterns and interfaces
   - Fix: verify the `Item` enum covers all declaration kinds from the parser

## Verification

```bash
# Pipeline tests (phase 3+6)
cargo test -p v2-compiler-tests

# Full pipeline round-trip
cargo test -p v2-compiler-tests v2_crate_emit_to_target -- --ignored
```

## Working notes

- `06_pipeline.dag` is only 108 lines — read the entire file before making changes
- Several items may already be fixed based on the current clean state of the pipeline
- Changes to `00_core.dag` types affect all downstream modules — verify imports
  in `01_tokenize.dag` through `06_pipeline.dag` after any type changes
