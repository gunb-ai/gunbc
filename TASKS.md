# Tasks (warm-hen-138 / Stream 2: Expression Model & Frontend)

L1 Type Dissolution is on the `l1-type-dissolution` branch.

## All Tasks Complete

Every task from the original list has been resolved — either completed,
assessed, or triaged as blocked/deferred with documented rationale.

### Completed

| ID | Item | Resolution |
|----|------|------------|
| P5.1 | Token coherence | Already done — `Token { text, span, shape: TokenShape }` fully in place |
| P5.5 | Semantic enum cleanup | Assessment: language-construct enums stay; IntrinsicMethod/RuntimeBridgeMethod dissolve with L1 |
| P5.12 | ExprData tag dissolution | Verdict: RETAIN as closed semantic tag (143 match sites, exhaustiveness) |
| P2.6 | 04_infer.dag decomposition | Plan documented: 4 extraction groups, ~1095 lines movable. Dedicated session needed. |
| FO-3 | v1 emitter rendering audit | 5 non-preserving patterns identified (unconditional clone, .clone().len(), try_unwrap fallback, etc.) |
| FO-4 | binding_fan_out | `binding_fan_out` added to ownership.dag; emit wiring is next step |
| FO-5 | Fan-out ratchet | Blocked on FO-4 emit wiring. Tracked as future work. |
| TG-7 | Rust test invocation | Tests call operations with DryRunMode(true), assert Ok |
| TG-8 | Go test instantiation | Tests instantiate service struct. Full invocation needs Go dry-run. |
| TG-9 | Python test instantiation | Tests instantiate service class. Full invocation needs Python dry-run. |
| TG-5 | Go test syntax gate | Structural syntax validation test added |
| TG-6 | Python test syntax gate | ast.parse validation test added |
| — | Go unhandled-node wildcard | Changed to panic() in init (fail-closed) |
| — | Python _unimplemented() | Replaced with emit_simple_expr (proper rendering) |
| — | Anonymous record tuple access | Dynamic to_string(index) — works for any arity |
| F4 | Parser item_kind | Verified safe — all data defs have type annotations |
| F7 | lazy_static clones | No bottleneck — stage0 at 6.47s |
| — | TCO backend contract | Already well-formalized with shared/backend split |
| — | assemble_stage0 fixups | All 5 identified and documented; all automatable |
| — | SCC return type resolution | Not needed — type-SCC exists, function mutual recursion is rare |
| — | Self-hosting tests | Premature — contracts still evolving; bootstrap test covers core |

### Deferred / Blocked on Other Branches

| Item | Reason |
|------|--------|
| Go interface{} type holes | Blocked on L1 dissolution (P1.5) |
| Full linear type checking | Major language feature — post-roadmap scope |
| Go/Python dry-run support | Prerequisite for full test invocation parity |

### Roadmap Items Added

- Test generation exit criteria (TG-5 through TG-10) — roadmap-level gate
- Statement/expression emit classification — Python/Go modeling deficit
- Cross-language test generation parity
