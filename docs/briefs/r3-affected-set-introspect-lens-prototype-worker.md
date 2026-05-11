# R3 affected-set Introspect-lens prototype — substrate-composition implementation

**Status:** PRE-AUTH DISPATCH-READY. Canvas-driven rebuild for gunbc#2699; supersedes silent-bat-152 PR #2701 once the replacement implementation PR opens.

**Owner:** worker TBD on dispatch. Coordinator: R3 Verification Mgr (`clever-tern-670`). PM/Director escalation path: deep-wolf-155 / zesty-bear-812.

**Design authority:** [`docs/design-affected-set-lens.md`](../design-affected-set-lens.md), especially §0 locked substrate-vs-adapter terminology, §2 dimension-parameterized affected-set definition, §3 substrate-composition algorithm sketch, and §4 worked examples.

**Context:** PR #2701 is being closed as superseded because it introduced a large hand-Rust affected-set implementation and increased the SG-0 hand surface. This rebuild must prove the same prototype shape through `.dag` substrate composition, not a new Rust analysis engine.

---

## §0. Locked framing

The affected-set surface is an **IntrospectApplication-carrier lens**, not a new query substrate.

- The lens lives at `src/v3/lenses/affected_set_lens.dag`.
- The substrate output is `Set<NodeRef>` or a `NodeRef`-keyed record carrying dimension and provenance.
- User-facing CLI / IDE / agent rendering (`{file, span}`, symbols, line ranges) is an adapter boundary after substrate output exists.
- No `Query` type, no parallel affected-set Rust engine, and no hand-maintained downstream graph.

The worker composes the locked design. Do not redesign the carrier shape in this slice.

## §1. Deliverable

Implement the pre-R3-close prototype from `docs/design-affected-set-lens.md` §7:

1. Add `src/v3/lenses/affected_set_lens.dag` as the substrate authority for the affected-set Introspect lens.
2. Keep the lens compact and compositional. Expected size is roughly 20-50 lines unless a clearly cited substrate shape forces slightly more.
3. Add tests-as-data fixtures only under `src/v3/compiler/tests/dag/`.
4. Demonstrate three real PR outputs from the gunbc#2699 charter:
   - PR #2693: v2 directory delete
   - PR #2679: gate #4 `workflow_idempotency`
   - PR #2647: quantifier substrate
5. Each real-PR output must include per-dimension affected-set data plus commentary on why the set is narrower than naive transitive-downstream.

## §2. Required substrate composition

The implementation must cite and consume these substrate pieces:

| Piece | Authority | Required role |
| --- | --- | --- |
| `IntrospectApplication<Set<NodeRef>>` | `src/v3/std/lens_application.dag` | User-surface introspect mode; no enforcement budget axis. |
| `Dag` graph edges | v3 substrate / `Dag` shape | Reverse-edge traversal over the compiled graph. |
| `DescentEvidence` | `src/v3/std/termination.dag` | Call-site descent lattice; narrows propagation when evidence is known. |
| `SubValueRelation` | `src/v3/std/induction.dag` | Tracks which sub-piece flows through a call/refinement edge. |
| Cardinality lens | `src/v3/lenses/` | Distinguishes opaque-carrier changes from structural cardinality/data-shape changes. |
| `cross_target_coverage` | `src/v3/std/cross_target_coverage.dag` | Narrows target-specific propagation rather than marking every target consumer. |
| `TestClaim` | `src/v3/std/verification.dag` | Tests are data; affected-set can intersect with TestClaim references. |

If any required substrate piece is missing or cannot be consumed from `.dag`, STOP and escalate. Do not fill the gap with Rust.

## §3. Dimension semantics

The prototype must preserve the dimension-parameterized shape from the design:

- dimensions: `value`, `cost`, `complexity`, `effect`, `refinement`
- seed only nodes with a **PROVEN** delta in that dimension
- include unknown/unprovable deltas by default (fail-closed)
- propagate only through typed edges where that dimension is read by the consumer
- emit or fixture a per-dimension affected-set output, not just one undifferentiated set

Unknowns must not silently disappear. If the lens cannot prove `delta(dim) == empty`, the affected consumer stays in the affected-set and the fixture commentary must say why.

## §4. Hard constraints

1. **No hand-Rust analysis engine.** Do not add `src/v3/compiler/src/affected_set*.rs` or comparable host-side graph logic.
2. **Near-zero Rust budget.** A user-surface adapter is legitimate only at a CLI/rendering boundary and only with explicit canvas-tier justification citing the substrate gap that prevents `.dag` rendering. This prototype should not need one.
3. **Tests-as-data only.** Add `TestClaim` fixtures under `src/v3/compiler/tests/dag/`. Do not add Rust integration tests, unit tests, or SG-0 entries.
4. **SG-0 delta must be zero or negative.** Positive SG-0 delta is a stop signal. Escalate to PM/Director before pushing.
5. **No template-only fake.** The lens body must compose substrate facts. A table of hand-authored PR-name-to-output templates does not satisfy the brief.
6. **No new substrate carriers unless escalated.** The prototype consumes the carriers named above. New substrate requires canvas-tier approval before implementation.

## §5. Acceptance

The implementation PR is acceptable when all are true:

- `src/v3/lenses/affected_set_lens.dag` exists and is the only affected-set lens authority.
- The lens is modeled as an `IntrospectApplication<Set<NodeRef>>`-compatible substrate lens per `docs/design-affected-set-lens.md` §0.
- The `.dag` tests-as-data fixtures cover:
  - dimension-specific outputs for value, cost, complexity, effect, refinement
  - fail-closed unknown-delta behavior
  - the three real PRs named in §1
- Fixture commentary explains narrowing vs naive transitive-downstream for each real PR.
- SG-0 hand-authored Rust count does not increase.
- PR body includes: `Supersedes PR #2701` and a closure note that #2701 should close as superseded once the replacement PR is open.

## §6. Stop and escalate

Escalate with `dashboard-message send --to parent --body "..."` if:

- implementing the lens appears to require Rust graph traversal outside an adapter boundary
- a required substrate fact is absent or only available through a host mirror
- a TestClaim cannot express the prototype output without a new predicate/carrier
- the SG-0 delta would be positive
- the worked real-PR outputs cannot be represented as `Set<NodeRef>` / dimension-keyed records

Do not push a workaround PR for any of these; the point of this rebuild is to avoid repeating PR #2701's hand-Rust path.

---

**End of canvas.**
