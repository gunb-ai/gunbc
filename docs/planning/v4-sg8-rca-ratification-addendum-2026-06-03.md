# v4 SG-8 RCA Ratification Addendum — 2026-06-03

> **Status:** ADDENDUM — implementation dispatch **authorized** (Modeling DFS Arbiter §8 sign-off #4143).
> **Authority:** PR #4140 Jun1 M1 rustc catalog; SG-8 worksheet #4127 on main.
> **Does not supersede:** `v4-sg8-module-graph-carrier-reexports-worksheet-2026-05-31.md` (git: #4127).

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
| Acceptance | F1–F4 in the SG-8 worksheet, not SG-8 count reduction |

---

## Implementation Dispatch Packet

```text
Implement SG-8 per v4-sg8-module-graph-carrier-reexports-worksheet-2026-05-31.md (#4127).

MUST:
  - Fix v2 emit_imports so graph type names do not trigger enum-variant parent expansion.
  - Resolve pub-use and variant paths from defining ItemInfo.module_name, not import-site module.
  - Emit parametric type aliases (pub type Foo<T> = ...) through the generic alias path.
  - Prove F1–F4 with forbidden-pattern greps and a fresh M1 probe receipt.

MUST NOT:
  - Add name-keyed import patch tables.
  - Re-export promoted carriers from old compiler-local modules.
  - Claim acceptance from E0425/E0432/E0433 count reduction.
```

Representative failures from #4140 §4:

- `CarrierKind` imported from `v4_compiler_target_carriers` while authority moved to `std/pipeline` — module-authority projection gap.
- `NodeRef` imported from `v4_std_node` with no such item — concept-home decision required before broad consumer imports.
- `EdgeLabel` undeclared in generated claim modules — dependency-edge emission from type references in generated bodies.
