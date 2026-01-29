# Repository Reconciliation

**Status**: Living document  
**Last updated**: January 2026

---

## Overview

Three repositories form the gunb ecosystem:

| Repo | Role | Language | Status |
|------|------|----------|--------|
| **gunb.ai** | Production runtime | Go | Active, proven patterns |
| **the-gunbai** | Theory & V2/V3 spec | Docs + Rust | Design source of truth |
| **gunbc** | Compiler & IR | Rust | Active development |

This document captures how they relate and what gunbc should adopt from each.

---

## Evolution Story

1. **gunb.ai**: Started as imperative Go code. Added DAG "overlay" when things
   went wrong — the DAGs caught wiring bugs. Realized much of the wiring could
   be *derived* from the DAG structure.

2. **the-gunbai**: Flipped the model — generate code FROM the DAG. Introduced
   "understandings" to model external tools. Discovered that understandings
   themselves kept needing refactoring to use more fundamental constructs.

3. **gunbc**: The search for fundamental primitives. If we get the primitives
   right, understandings (and everything above) become stable compositions.

**Key insight**: Each layer exists because the layer above was "wobbly" —
needed something more fundamental underneath.

---

## Target Architecture

gunbc should eventually subsume the runtime capabilities of gunb.ai:

```
┌─────────────────────────────────────────────────────────┐
│                      gunbc                               │
│                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌────────────┐ │
│  │  Primitives  │───▶│  Validation  │───▶│  Executor  │ │
│  │  Patterns    │    │  Contracts   │    │  (JIT?)    │ │
│  │  Types       │    │  Lowering    │    │            │ │
│  └──────────────┘    └──────────────┘    └────────────┘ │
│        ▲                                       │        │
│        │                                       ▼        │
│   (the-gunbai                            (gunb.ai      │
│    theory)                               capabilities) │
└─────────────────────────────────────────────────────────┘
```

We don't literally compile to gunb.ai — we evolve gunbc to have:
- **Compile-time**: Strong wiring/correctness (the-gunbai theory)
- **Runtime**: Execution capabilities (gunb.ai patterns)

---

## What gunbc Adopts from the-gunbai

### Core Model (Already Aligned)

| Concept | the-gunbai | gunbc |
|---------|------------|-------|
| Fractal structure | `Node<T>` + `SubDag` | ✓ Same |
| Typed ports | `TypedPort` | ✓ `Port { type_id }` |
| Patterns as sub-DAGs | Upsert = 3-node chain | ✓ Pattern builders |
| Transport abstraction | Request/Response | ✓ Transport layer |

### Needed Adoptions

| Feature | the-gunbai | gunbc Status | Priority |
|---------|------------|--------------|----------|
| Provides/Requires | `NodeContract` | Not implemented | High |
| Effect bits | `PURE/READ/WRITE` | Structural only | High |
| Idempotency | `Idempotent/WithKey/Not` | Implicit in Upsert | High |
| Lane discipline | A/B/C type lanes | TypeId exists, no lanes | Medium |
| Property claims | `{ property, verified_by }` | Not implemented | Medium |

### Theory to Preserve

From the Abstraction Calculus (ac.pdf):
- **Kernel quotient**: Two impls equivalent if same typed output on same typed input
- **Idempotence**: Once opaque, further compression is trivial
- **Re-priming**: Opening a node — sub-DAG becomes new universe
- **Stratified reflection**: Compiler validates; executor runs opaque leaves

---

## What gunbc Adopts from gunb.ai

### Runtime Patterns (Future)

| Feature | gunb.ai | gunbc Status | Priority |
|---------|---------|--------------|----------|
| Parallel execution | Wave-based (batched) | Sequential | High |
| Progress events | JSONL stream | Not implemented | Medium |
| Approval gates | `REQUIRE_APPROVAL` | Not implemented | Future |
| State persistence | Checkpoint/resume | Not implemented | Future |
| Cursor integration | `cursor_apply_change` | Not implemented | Future |
| GitHub integration | `github_open_pr` | Not implemented | Future |

### Patterns NOT to Adopt

| Pattern | gunb.ai | Why Not |
|---------|---------|---------|
| Wave batching | Discrete wave groups | Work-queue is simpler/faster |
| Go implementation | pkg/dag in Go | Rust is the target |
| Template factories | `TemplateBuilder` | gunbc patterns are more fundamental |

---

## Execution Model

### gunb.ai Model (Wave-Based)

```
Wave 1: [A, B, C]  →  wait all  →  Wave 2: [D, E]  →  wait all  →  Wave 3: [F]
```

Problem: Artificial barriers. A finishes early, but waits for B and C.

### gunbc Model (Work-Queue)

```
Ready: {A, B, C}
A done → D now ready
B done → nothing new  
C done → E now ready
...
```

No artificial barriers. Maximum throughput. Parallelization falls out of the
DAG structure — no need to precompute waves.

---

## Key Design Decisions

### 1. Contracts Are Explicit

Every node declares what it `requires` and what it `provides`. The compiler
verifies all requirements are satisfied by prior nodes. This catches wiring
errors at compile time.

### 2. Effects Are Explicit

Nodes declare `Pure/Read/Write`. The executor uses this for parallel safety.
Structural boundary detection (current) is supplementary, not primary.

### 3. Types Have Lanes

Lane A (core), Lane B (tool-specific), Lane C (doc-only). Promotion happens
at 2+ consumers. This prevents type proliferation while allowing extension.

### 4. Patterns Are Sub-DAGs

Patterns aren't tags or annotations — they're actual DAG structures with
typed data flow. Upsert IS a three-node sub-DAG: Check → Create → Resolve.

### 5. JIT is Future

Current focus: compile-time correctness. JIT execution (for dynamic node
insertion) comes later. The same DAG structure supports both.

---

## Migration Path

1. **Phase 1**: Contracts & Validation (compile-time correctness)
2. **Phase 2**: Effect & Property Classification (executor information)
3. **Phase 3**: Work-Queue Executor (parallel runtime)
4. **Phase 4**: Type Registry with Lanes (extension discipline)
5. **Phase 5**: JIT Execution (dynamic capabilities)
6. **Phase 6**: Integrations from gunb.ai (Cursor, GitHub, etc.)

gunb.ai remains the production system while gunbc matures. Eventually,
gunbc-based tools replace gunb.ai workflows.
