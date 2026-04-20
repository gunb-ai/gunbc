### Root Cause C: Errors Propagate as Valid-Looking Fabrications

**Invariants violated:** No fallbacks that fabricate, Explicit boundary
contracts, Correctness by construction.

**The problem:** When the compiler encounters an error (missing argument,
unresolved type, unknown function), it fabricates a valid-looking node
(LitNull, Dynamic, `<error:*>` string) and continues. This lets broken
programs reach emit, which generates invalid target code containing
sentinels like `<error:unknown_with_type>` or empty strings.

**Design decision required:** Structural error representation. Currently
error state is encoded as:
- `LitNull` with `return_type: none` (37 sites across parser/reconcile/emit)
- `Dynamic` type name (universal compat in `node_type_equals`)
- `<error:*>` strings detected by `string_contains` (2 check sites, 4 production sites)
- ~~`Warning` severity for semantic errors (`access_error`, `inference_error`)~~ **FIXED (2026-04-01).** All diagnostics are now errors; `is_error_diagnostic` always returns `true`.

The fix: make error a structural variant — either an `ExprError` in ExprData
or a flag on Node — so downstream phases can test `is_error(node)` without
string parsing. Emit skips error nodes (or emits `compile_error!()`) instead
of translating fabricated values.

Parser LitNull recovery (23 sites in `02_parse.dag`) is a separate concern —
parser error recovery that produces dummy nodes with attached error
diagnostics is standard practice. The issue is that reconcile and emit don't
recognize these as error nodes and try to process them normally.

| # | Pattern | Sites | Where |
|---|---------|-------|-------|
| C-1 | LitNull sentinel for missing arguments | 5 | `05_emit_rust.dag:1751,1752,1760,1761,1786` |
| C-2 | LitNull sentinel for missing defaults/config | 9 | `04_reconcile.dag:3025,3053,3114,3158,3165,3172,3272,3293,3510` |
| C-3 | LitNull dummy for parser error recovery | 23 | `02_parse.dag` (throughout) |
| C-4 | `<error:*>` placeholder types | 4 production | `04_reconcile.dag:1531,1698,1861,2255` |
| C-5 | `<error:*>` detection via string_contains | 2 check | `04_reconcile.dag:900`, `05_emit_rust.dag:1473` |
| C-6 | `<error:unknown_*>` sentinels in emit | 2 | `05_emit_rust.dag:1766,2117` |
| C-7 | Dynamic as universal compatibility | multiple | `node_type_equals` in `04_reconcile.dag:901+`; `extend_scope_for_lambda` in `05_emit_rust.dag:1959` |
| C-8 | ~~Warning severity for semantic errors~~ | **FIXED** | `OwnershipWarning` → `OwnershipViolation`, `VariantCollisionWarning` → `VariantCollision`; `is_error_diagnostic` always returns `true` (2026-04-01) |
| C-9 | Empty node / empty string fabrication | 2 | `05_emit_rust.dag:819` (empty Node for missing field), `05_emit_rust.dag:3368` (LitNull → "") |
| C-10 | `Rc::try_unwrap` clone fallback (v1) | 1 | `fn_codegen.rs:3783` — blocked on Track D ownership proof |

**Previously tracked as:** TD-7, SB-2, SB-3

---

