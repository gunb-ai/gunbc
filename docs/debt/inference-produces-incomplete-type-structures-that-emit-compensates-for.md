### Inference produces incomplete type structures that emit compensates for

**Invariant violated:** Correctness by construction, not by validation.

**Observation (2026-03-25):** `bare_map_node()` and `bare_list_node()` in
`04_types.dag` create container type nodes with zero children. These are
structurally incomplete — a `Map` without key/value children is not a fully
resolved type. Inference hands them to emit unchanged (via `empty_map()`,
`map_insert()`, `map_merge()` in `04_infer.dag` and `04_method.dag`).

The old per-backend emitters compensated with hardcoded fallbacks:
`"Map"` → `"BTreeMap<_, _>"`, `"List"` → `"Vec<_>"`. When the shared emitter
was extracted (P4.2), these compensations were initially lost. The shared
emitter now restores them (`emit_node_type_leaf_rc` bare container branch),
but the fix is in the wrong layer — emit shouldn't need to know that
inference might produce incomplete containers.

| # | Severity | Where | What |
|---|----------|-------|------|
| IV-1 | MED | `04_types.dag:76-78` | `bare_map_node()` creates Map with 0 children — structurally incomplete |
| IV-2 | MED | `04_infer.dag:1912-1921` | `empty_map()` returns bare container without resolving type params |
| IV-3 | MED | `04_method.dag:153,176` | `map_insert()`/`map_merge()` return bare_map_node |

**Direction:** Either inference resolves container type parameters from context
(bidirectional inference), or bare containers carry an explicit "unresolved
parameters" marker that emit can handle uniformly rather than per-backend.

---

