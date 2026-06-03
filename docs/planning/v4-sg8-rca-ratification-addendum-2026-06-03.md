# v4 SG-8 RCA Ratification Addendum — 2026-06-03

> **Status:** ADDENDUM — implementation dispatch **authorized** (Modeling DFS Arbiter §8 sign-off #4143).
> **Authority:** PR #4140 Jun1 M1 rustc catalog (recover:
> `git show 65e8db2ac0:docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.md`).
> **SG-8 worksheet:** landed #4127; planning path removed in #4192 — recover F1–F4 and full
> packet via
> `git show 8c26800586:docs/planning/v4-sg8-module-graph-carrier-reexports-worksheet-2026-05-31.md`.
> **Does not supersede** that recovered worksheet; this addendum only raises priority post-#4140.

---

## What Changed Since The Worksheet

PR #4140 remeasured the full M1 Rust emitted tree after the Jun1 cascade. The
SG-8 code family is the largest delta driver:

| Code | #4140 count | #4122 baseline | Delta |
| --- | ---:| ---:| ---:|
| `E0425` | 538 | 485 | +53 |
| `E0432` | 415 | 238 | +177 |
| `E0433` | 271 | 81 | +190 |
| **SG-8 total** | **1,224** | **804** | **+420** |

This growth does not create a new worksheet class. It confirms the existing SG-8
readout: imported names, variant-parent expansion, generic aliases, and promoted
carrier homes are emitted without a single export/import authority.

---

## Ratification Status

| Check | Answer |
| --- | --- |
| Single authority | Defining module/export surface, not import-site spelling or shim re-export tables |
| Existing worksheet still valid | Yes: #4127 — `emit_imports` graph-type isolation + defining-module `pub use`; parametric alias emission |
| §8 ratification | **Closed** 2026-06-01 via #4143 (`proud-fox-405`) |
| New #4140 evidence changes dispatch? | No; raises priority only |
| Spot-fix forbidden | Hand-added `pub use` rows, CarrierKind/List/Char shim duplication, per-error unresolved import patches |
| Acceptance | F1–F4 in recovered SG-8 worksheet (`git show 8c26800586:…` above), not SG-8 count reduction |

---

## Implementation Dispatch Packet

```text
Implement SG-8 per git show 8c26800586:docs/planning/v4-sg8-module-graph-carrier-reexports-worksheet-2026-05-31.md (#4127).

MUST:
  - Fix v2 emit_imports so graph type names do not trigger enum-variant parent expansion.
  - Resolve pub-use and variant paths from defining ItemInfo.module_name, not import-site module.
  - Emit parametric type aliases (pub type Foo<T> = ...) through the generic alias path.
  - Prove F1–F4 with forbidden-pattern greps; rustc residual remeasure per #4140 §5 repro
    (extended probe), not the live ci-floor M1 gate alone.

MUST NOT:
  - Add name-keyed import patch tables.
  - Re-export promoted carriers from old compiler-local modules.
  - Claim acceptance from E0425/E0432/E0433 count reduction.
```

Representative failures from #4140 §4:

- `CarrierKind` imported from `v4_compiler_target_carriers` while authority moved to `std/pipeline` — module-authority projection gap.
- `NodeRef` imported from `v4_std_node` with no such item — concept-home decision required before broad consumer imports.
- `EdgeLabel` undeclared in generated claim modules — dependency-edge emission from type references in generated bodies.
