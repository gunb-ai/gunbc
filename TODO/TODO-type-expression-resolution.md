# TODO: Type Expression Resolution Scope

**Status**: Draft
**Date**: 2026-02-07

## Goal

Decide whether we need full generic type expression parsing (e.g., `Map<K,V>`, nested wrappers) or if wrapper-only parsing (`Optional<T>`, `List<T>`, etc.) is sufficient.

## Tasks

- [ ] Inventory current/near-term use cases that would require `Map<K,V>` or other nested generics.
- [ ] Evaluate complexity and risks of full parser + Map semantics vs. wrapper-only parsing.
- [ ] Decide scope and record recommendation; implement only if needed.

## Notes

- Wrapper-only parsing already enables `Optional<Map>` / `Optional<TransportResponse>` via identity DAGs.
- Full generic support would require Map key/value validation DAGs and deeper type-expression parsing.
