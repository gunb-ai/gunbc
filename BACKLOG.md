# gunbc Backlog

Items deferred from ROADMAP.md. Not blocking current work. Revisit when
relevant or when profiling/usage shows need.

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

## Invariant Violations

| Item | Violation | What needs to happen |
|------|-----------|----------------------|
| Emission/complexity dual classification | **No duplicate representations.** Method cost shapes are classified independently in `07_complexity.dag` (`classify_method_cost`) and `05_emit.dag` (`classify_intrinsic_method`). The test `test_emission_cost_contract_coverage` hardcodes a third copy of the method list. Three representations of the same fact. | The emitter should be the single authority. `classify_intrinsic_method` should return both the `IntrinsicMethod` and its `CostShape`. The complexity analyzer imports and consumes the cost shape from the emitter — not its own parallel string-matching classifier. Adding a new intrinsic without a cost shape becomes a compile error, not a test failure. |

## Root Cause B: Closed Sets as Strings

Mechanical enum conversions — no design ambiguity for B-1/2/3/5/6.
B-4 (method intrinsics) intersects with the IntrinsicMethod enum
already defined in `05_emit.dag`.

See INVARIANTS.md "Root Cause B" for the full table.

## Root Cause C: Errors as Fabrications

37+ sites where errors propagate as valid-looking values. See
INVARIANTS.md "Root Cause C" for the full table. Design decision
(structural error variant) documented but not yet implemented.

## Multi-Walk P4-P5

- **P4:** Registry rebuild in emit (subsumed by EmitContext migration)
- **P5:** Kahn single-pass (landed in `03_resolve.dag`)
