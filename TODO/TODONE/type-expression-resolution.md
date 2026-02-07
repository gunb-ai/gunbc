# TODO: Type Expression Resolution Scope

**Status**: Done
**Date**: 2026-02-07

## Goal

Decide whether we need full generic type expression parsing (e.g., `Map<K,V>`, nested wrappers) or if wrapper-only parsing (`Optional<T>`, `List<T>`, etc.) is sufficient.

## Tasks

- [x] Inventory current/near-term use cases that would require `Map<K,V>` or other nested generics.
- [x] Evaluate complexity and risks of full parser + Map semantics vs. wrapper-only parsing.
- [x] Decide scope and record recommendation; implement only if needed.

## Notes

- Wrapper-only parsing enables `Optional<Map>` / `Optional<TransportResponse>` via identity DAGs, but Map parsing was required for nested generics.
- Implemented full Map parsing and diagnostics; Map types resolve to identity DAGs with canonical `Map<K,V>` names.
- Future improvement: promote type expressions to first-class constructor templates (arity + DAG builder) so all consumers share a single parser/AST and Map key/value semantics can be enforced uniformly.
