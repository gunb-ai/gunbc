# v4 SG-8 RCA Ratification Addendum — 2026-06-01

> **Status:** ADDENDUM ONLY — does not supersede `docs/planning/v4-sg8-module-graph-carrier-reexports-worksheet-2026-05-31.md`.
> **Authority:** PR #4140 Jun1 M1 rustc catalog; existing SG-8 worksheet #4127 on main.
> **Gate:** proud-fox-405 §8 ratification remains required before implementation dispatch.

---

## What Changed Since The Worksheet

#4140 remeasured the full M1 Rust emitted tree after the Jun1 cascade. The SG-8 code family is now the largest delta driver:

| Code | #4140 count | #4122 baseline | Delta |
| --- | ---:| ---:| ---:|
| `E0425` | 538 | 485 | +53 |
| `E0432` | 415 | 238 | +177 |
| `E0433` | 271 | 81 | +190 |
| **SG-8 total** | **1,224** | **804** | **+420** |

This growth does not create a new worksheet class. It confirms the existing SG-8 readout: imported names, variant-parent expansion, generic aliases, and promoted carrier homes are being emitted without a single export/import authority.

---

## Ratification Checklist For proud-fox-405

| Check | Required answer |
| --- | --- |
| Single authority | Defining module/export surface, not import-site spelling or shim re-export tables |
| Existing worksheet still valid | Yes: `emit_imports` graph-type isolation + defining-module `pub use`; `emit_typed_item` parametric alias emission |
| New #4140 evidence changes dispatch? | No; it raises priority only |
| Spot-fix forbidden | Hand-added `pub use` rows, CarrierKind/List/Char shim duplication, per-error unresolved import patches |
| Acceptance | F1-F4 in the SG-8 worksheet, not SG-8 count reduction |

---

## Implementation Dispatch Packet After Ratification

```text
Implement SG-8 per docs/planning/v4-sg8-module-graph-carrier-reexports-worksheet-2026-05-31.md.

MUST:
  - Fix v2 `emit_imports` so graph type names do not trigger enum-variant parent expansion.
  - Resolve pub-use and variant paths from the defining ItemInfo.module_name, not the import-site module.
  - Emit parametric type aliases (`pub type Foo<T> = ...`) through the generic alias path.
  - Prove F1-F4 with forbidden-pattern greps and a fresh M1 probe receipt.

MUST NOT:
  - Add name-keyed import patch tables.
  - Re-export promoted carriers from old compiler-local modules.
  - Claim acceptance from E0425/E0432/E0433 count reduction.
```

---

## Local Probe Note

The manager session regenerated the M1 probe to recover E0308 stratification. The local SG-8 counts were lower than #4140 while stable families matched exactly, so this addendum keeps #4140 as the only SG-8 population authority and uses the local run only as supporting evidence that the representative failure shapes remain present.

