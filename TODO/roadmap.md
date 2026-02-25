# Roadmap: SDLC in Pure DSL

**Target**: SDLC pipeline running holistically in `.dag` — issue intake through
close, claim-based workers, multi-provider adapters. Zero extern bridges. All
domain logic in DSL.

**Timeframe**: ~10 weeks (March–May 2026)

**Principle**: Model first. Delete Rust along the way. Every phase ships
something that works.

---

## Design Docs

| Doc | Scope | Tasks |
|-----|-------|-------|
| [`foundation-cleanup.md`](../docs/design/v4/foundation-cleanup.md) | Dead code, NF-7, extern bridge elimination, compiler features | FC-CL, FC-NF7, FC-P6, FC-P7, FC-CF, FC-P8 |
| [`implementation-roadmap.md`](../docs/design/sdlc/implementation-roadmap.md) | SDLC activation, local e2e, cloud deployment | SDLC-1:8, SDLC-CD1:6 |

## Combined Graph

```
                Foundation                           SDLC
          ┌─────────────────┐               ┌──────────────────┐
Wk 1-2    │ FC-CL (cleanup) │               │ SDLC-1 (catalog) │
          │ FC-NF7 (lowerer)│──────────────▶│ SDLC-5 (signal)  │
          │        │        │               │ SDLC-6 (artifact)│
Wk 2-4    │ FC-P6 (policy)  │               │        │         │
          │ FC-P7 (registry)│               │ SDLC-2 (dispatch)│
          │        │        │               │ SDLC-3 (validate)│
Wk 4-7    │ FC-CF (compiler)│               │ SDLC-4 (testing) │
          │        │        │               │        │         │
Wk 7-9    │ FC-P8 (anemic)  │               │ SDLC-7 (verify)  │
          │        │        │               │ SDLC-8 (local e2e)│
          └─────────────────┘               │        │         │
                                            │ SDLC-CD (cloud)  │
                                            └──────────────────┘
```

SDLC-1:6 can start immediately (no foundation dependency).
FC-P6 and FC-P7 run in parallel after FC-NF7.
FC-CF runs in parallel with P6/P7.
FC-P8 requires FC-P6 + FC-P7 + FC-CF.

## Endstate

Zero extern bridges. All domain logic in DSL. SDLC runs e2e on local and
cloud profiles. The Rust substrate compiles and executes — it has no
knowledge of SDLC stages, policy rules, tree rendering, or makefile assembly.
