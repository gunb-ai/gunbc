### src/v2/ (compiler)

**00_core.dag** — 8.5/10
- M4: `AuthConfig.scheme` as String — should be enum
- Good: TypeExpr is exemplary, predicates compositional

**01_tokenize.dag** — 8.5/10
- M1: `Unknown` conflates invalid chars and unterminated strings
- Good: explicit state threading, keywords as data

**02_parse.dag** — 6/10 (CRITICAL)
- M6: **42 result types** — needs generic `Result<T>`
- M8: `kind_tag(token)` string comparison — fragile
- M7: `keyword_to_name` — local hardcoded strings removed; reads
  `dag_non_name_keywords` (partial: subset table, not derived from
  single keyword record)

**03_resolve.dag** — 8/10
- M5: Wildcard import `"*"` sentinel — should be `Optional<List<String>>`
- Good: Kahn's algorithm, diagnostic aggregation

**04_typecheck.dag** — 5.5/10 (CRITICAL)
- M5: **`lookup_in_scope` silently returns `unit_type()` on miss** — fabrication
- M5: **`lookup_field_type` also silently returns `unit_type()`**
- M8: `infer_method_call_type` dispatches on string method names

**05_emit.dag** — 6.5/10
- M5: **Anonymous products → `serde_json::Value`** — silent data loss
- M8: `needs_reference` hardcodes type names as strings

**06_pipeline.dag** — 8/10
- Good: clean linear pipeline, explicit error gating

---

