# v2 Pipeline Performance Audit

Audit date: 2026-03-14. Applies to: all 10 v2 compiler modules.

## Bottom line

The pipeline is acceptable for small bootstrap corpus. Bottlenecks are
structural, not micro. One sentence: **stop recomputing meaning in later
stages, and stop rebuilding whole lists/strings one element at a time.**

Performance and correctness are aligned — hotspots are also where the
model is weakest.

## Hotspot ranking (CPU)

1. **Typecheck** — most passes, linear lookups everywhere
2. **Emit** — re-runs inference, string-concat heavy
3. **Resolve** — quadratic by module count

## Priority order for fixes

### P1: Stop re-inferring inside emit

Emitter imports `infer_expr` from typecheck and re-runs inference on raw
`Expr` subtrees during emission. Root cause: `TypedExpr` is
`{expr, resolved_type}` at one node, not a recursively typed tree.

**This is the single most important structural finding.** The emitter
walks the raw expression tree, calling `infer_expr` at every node to
recover type information that the typechecker already computed. Fix:
make `TypedExpr` recursive (every subexpression carries its type), then
emission is a single pass over a fully-typed tree.

### P2: Ban append-by-concat in hot paths

Token accumulation, string building, parser accumulators, resolver list
assembly all do `concat(acc, [x])`. This is O(n) per append → O(n²)
for building a list of n items.

Fix: build in reverse and reverse once, or lower to mutable builders.
Applies to:
- `tokenize_loop` token accumulation
- `parse_*` parser state accumulators
- `resolve_modules` import list assembly
- `emit_*` string building

### P3: Give resolve real indexed maps

`check_duplicate_modules` is quadratic. `find_module` filters full module
list per import. Topo sort uses `get_at_index` scans. Currently
O(M²)–O(M³) where M = module count.

Fix: module-name map, exported-names-per-module map, Kahn on indexed
vectors.

### P4: Collapse typecheck passes or memoize

6+ passes over same data: `build_type_env`, `resolve_item_types`,
`resolve_env_bindings`, `build_func_env`, `infer_items`,
`validate_no_unresolved`. Lookups are linear (filter over
bindings/locals/signatures). Even a small memo layer helps.

### P5: Stream the early pipeline

`parse(tokens: tokenize(source: s.content))` directly instead of
materializing a separate `tokenized` list first.

## Three recurring themes

1. **Performance and correctness are aligned** — hotspots are also where
   the model is weakest.
2. **Current compiler is "bootstrap-correct," not "logically minimal"** —
   large simplification pass available later.
3. **Biggest improvement is architectural, not local** — recursively typed
   expressions + indexed identities + fewer passes.
