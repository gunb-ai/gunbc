# gunbc Backlog

Items not on the critical path in ROADMAP.md. Revisit when relevant or
when profiling/usage shows need.

---

## Language Features

| Item | What | Status |
|------|------|--------|
| General generic syntax | `type Foo<T> = ...` parameterized types. Special-cased Result/Option sufficient for now. | Deferred |
| Full linear type checking | Prove ownership flow statically in v2 compiler. Use-count-based proof (D-ownership) is sufficient for now. | D-ownership landed |
| Widen V5 | Handle non-takeable modified fields in functional record update. Current conservative V5 covers hot paths. | Deferred |

## Compiler Improvements

| Item | What | Status |
|------|------|--------|
| Anonymous record target resolution | Ambiguous cases must fail closed. | Deferred |
| Collection intrinsic semantics in shared IR | | Deferred |
| Generated self-hosting tests and stage contracts | | Deferred |
| TCO backend contract | No silent partial fallback. | Deferred |
| B3 Ph2a Contract 2 | SCC-aware return type resolution (not yet blocking). | Deferred |

## Root Cause B: Closed Sets as Strings

Mechanical enum conversions — no design ambiguity for B-1/2/3/5/6.
B-4 (method intrinsics) intersects with the IntrinsicMethod enum
already defined in `05_emit.dag`.

See INVARIANTS.md "Root Cause B" for the full table.

## Multi-Walk P4-P5

- **P4:** Registry rebuild in emit (subsumed by EmitContext migration)
- **P5:** Kahn single-pass (landed in `03_resolve.dag`)

---

*Moved to ROADMAP.md Phase 1:*
- *Emission/complexity dual classification (invariant violation)*
- *Root Cause C: errors as fabrications (structural error variant)*
