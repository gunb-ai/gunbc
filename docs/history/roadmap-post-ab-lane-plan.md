## Post-A/B Lane Plan

Four major lanes derived backward from the thesis, sixteen stages
total. Per-stage sizes use S/M/L/XL t-shirts; lane totals are
aggregate sizes, not calendar weeks. Full plan with sequencing and
dependency graph:
[docs/post-l15-phase-plan.md](docs/post-l15-phase-plan.md).

| Lane | Size | Closes | Design doc |
|---|---|---|---|
| **Lane 1 — Emission unification** | XL (six stages) | "Adding a new target = one spec file, zero new Rust" | 4 stage docs, master embedded in phase plan |
| **Lane 2 — Compile-time proofs** | XL (six stages) | "Structural properties are inescapable" (idempotency, symbolic cost, parallelism, user dims) | [lane2-compile-time-proofs.md](docs/lane2-compile-time-proofs.md) |
| **Lane 3 — Self-hosting cycle** | XL (three stages, one with five sub-stages) | "Causal engine: compiler is its own first consumer" | [lane3-self-hosting-cycle.md](docs/lane3-self-hosting-cycle.md) |
| **Lane 4 — Completion layer** | L (four stages) | Transport declarations, `dag run`, side effects, space bounds, async emission | [lane4-completion.md](docs/lane4-completion.md) |

**Hard sequencing:** Lane 1 Stage 1b gates Lane 2 start. Lane 1 Stage
1e gates Lane 3 Stage 3c and Lane 4 Stage 4d. Lane 2 Stage 2f gates
Lane 4 Stages 4b/4c. Lane 3 Stage 3a gates Lane 4 Stage 4a. Critical
path is six stages: `1a → 1b → 1c → 1d → 1e → 3c` (five M, one L).

**Nothing is backlog.** Every item previously marked "deferred M3/M4"
or "what NOT to build yet" is now a stage in a lane with acceptance
gates. Including async emission.

Lane 1 stages and their design docs:
- 1a: [phase1-lane1-l15-tail.md](docs/phase1-lane1-l15-tail.md)
- 1b: [lane1-stage-b-substrate-keyed-lookup.md](docs/lane1-stage-b-substrate-keyed-lookup.md)
- 1c: [phase1-lane2-clean-emission-invariant.md](docs/phase1-lane2-clean-emission-invariant.md)
- 1d: [phase1-lane3-consolidation-build-plan.md](docs/phase1-lane3-consolidation-build-plan.md)
- 1e, 1f: written just before each stage starts, informed by what 1a–1d learn

Each stage carries scope, direction, escalation criteria, and
acceptance gates. See the master plan for sequencing details and the
full acceptance checklist for "plan complete."

