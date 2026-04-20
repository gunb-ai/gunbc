### CollectionKind — DISSOLVED (2026-03-28)

**Previously:** `CollectionKind` enum (6 variants) on `Node`, 184 sites
across 17 files. Compiler branched on enum to distinguish collection types.
Every `Node { ... }` literal had to set `collection_kind` correctly or
`node_is_map`/`node_is_container` silently returned false.

**Resolution:** Enum deleted, field removed from Node. Collection-ness is
now derived structurally after resolution: containers are the only type
nodes with `children > 0 && connective == NoConnective`. Three structural
predicates (`node_is_collection`, `node_is_keyed_collection`,
`node_is_element_collection`) replace all enum matching. A `container_types`
data list in `00_core.dag` controls which types stay unexpanded during
resolve. Emit uses `to_snake(n.name)` as LanguageSpec template key.

| # | Status | What |
|---|--------|------|
| IV-11 | **FIXED** | `CollectionKind` enum deleted |
| IV-12 | **FIXED** | `collection_kind_for_name` deleted |
| IV-13 | **FIXED** | Normalization block deleted (no field to normalize) |

---

