### Silent type fabrication in emit

**Invariant violated:** No fallbacks that fabricate.

**Observation (2026-03-25):** Several emit code paths produce valid-looking
but wrong output instead of failing. The `"String"` fallback was the
canonical case — a multi-field anonymous product with a missing `return_type`
emitted `(String, SomeType)` as valid Rust that compiles but has the wrong
type. Single-field products correctly used `compile_error!`.

Fixed: multi-field anonymous product now uses `compile_error!` (2026-03-25).
CLI param type mapping (`05_emit_rust.dag:3584-3591`) still fabricates
`"String"` for structured/unknown types — left as-is because CLI surface
is P4.5 scope, but tracked here.

| # | Severity | Where | What |
|---|----------|-------|------|
| IV-4 | FIXED | `05_emit.dag:952` | Multi-field anonymous product: `"String"` → `compile_error!` |
| IV-5 | LOW | `05_emit_rust.dag:3584-3591` | CLI param type mapping fabricates `"String"` for unknown types |

---

