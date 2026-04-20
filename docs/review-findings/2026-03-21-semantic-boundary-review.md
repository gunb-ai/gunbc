### 2026-03-21 — semantic-boundary review

Classified as invariant violations:

- Rust emission still repairs semantics downstream instead of consuming a
  fully classified boundary: `emit_typed_field_access` branches on
  `.typed`, `.value`, `is_likely_optional_receiver(...)`, and
  `emit_typed_expr` conditionally appends `.map(Rc::new)` via
  `lookup_on_data_needs_rc_wrap(...)`. This violates "Heuristics
  indicate lost structure" / "Explicit boundary contracts."
- `lookup_in_scope` falls back to `lookup_func_sig(...).return_type` for
  function-as-value references. That fabricates a non-callable value from
  a callable binding and violates "Explicit boundary contracts" / "No
  fallbacks that fabricate."
- `node_type_equals` still contains permissive compatibility rules
  (`Dynamic` matches anything, plus same-name/same-connective/same-child-count
  fallback) that hide missing earlier normalization. This violates "No
  fallbacks that fabricate" / "Explicit boundary contracts."
- ~~Reconcile downgrades semantic gaps to `Warning`~~
  **FIXED (2026-04-01).** `OwnershipWarning` renamed to `OwnershipViolation`,
  `VariantCollisionWarning` renamed to `VariantCollision`, both promoted to
  errors. `is_error_diagnostic` now always returns `true`. No warning
  severity remains in the compiler.

Not invariant violations by themselves:

- Roadmap/docs drift (`A7 full retirement`, `P1b done`, acceptance text
  that still names future work).
- Loose ratchets and unlanded StageMetrics/performance-contract work.
  Current checked-in values: `SELF_COMPILE_ERROR_RATCHET = 2700`,
  `CLONE_RATCHET = 21000` (pipeline.rs:7845). These are backlog/test
  debt, not direct invariant violations until a concrete boundary or
  algorithm violates a stated rule.

---

