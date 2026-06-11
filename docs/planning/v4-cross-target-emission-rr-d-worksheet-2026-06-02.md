# v4 Cross-Target Emission Round-Robin Worksheet (RR-D)

> **Status:** RATIFIED FOR W2 DISPATCH — Branch D contract closure (ctrl#1425 §6.8, 2026-06-02).
> **Work item:** `node://adhoc-b4e8b554-bae` — Cross-target Emission Mgr (`silent-bear-54`).
> **Gate:** D.1–D.2 land in W4.6 after Branch C.1 + Branch G.1; Shape B **GUARDED** in compiler.
> **Contract (W4.6 design):** `v4-branch-d-contract-d1-d2-2026-06-02.md` (PM dispatch 2026-06-02).

## §10.0-adapted worksheet

```text
Migration class:        D-SHAPE-B-CONTRACT (format/framework data models; user .dag emit only)
Representative failure:  Compiler 05_emit/06_translate grows OpenAPI/SQL/React string emitters;
                         Shape B artifacts conflated with Shape A TargetSource; framework
                         substrate mistaken for compiler emission authority.
Immediate local patch:   Hand-authored OpenAPI YAML emit in compiler; SQL DDL strings in
                         translate; React JSX templates beside target_model.
Why forbidden:           §2.3.1 GUARDED — Shape B = user .dag programs over typed values;
                         compiler emits Shape A only (LanguageSpec / TargetModel homomorphism);
                         INVARIANTS P1 — extdeps hold external facts, not gunbc emit policy.
DFS path:
  D.1 format contract:
    - src/v4/extdeps/formats/openapi.dag — document/schema facts (existing)
    - src/v4/extdeps/formats/sql.dag — DDL/relational facts (existing)
    - Contract: parse/emit axes stay format-local; NO imports from v4.compiler.emit/translate
  D.2 framework contract:
    - src/v4/extdeps/frameworks/react.dag — framework carriers (existing)
    - User programs project Shape B; compiler supplies types + verification hooks only
  W4.6 implementation (blocked on C.1 + G.1):
    - Typed boundary records linking format rows → TestClaim / RoundTrip where PROVEN
    - Dissolve openapi-json-yaml selector coproduct when media-type projection exists
Deepest unsound boundary:
  Treating OpenAPI `OpenApiDocument` or SQL migration carriers as targets of `emit()` —
  blurs source text, IR tree, and receipt JSON (§2.7.4).
Systemic fix:
  D.1–D.2 document the allowed consumer graph: formats/frameworks → user .dag → (optional)
  external toolchain; Shape A pipeline unchanged; cross-target routing uses RCA mgr charters
  for rust/python/go/ts only.
Non-goals:
  - T-24 CI YAML emission (Compiler Spine / Phase 2).
  - New compiler stages for Markdown/HTML/OpenAPI render.
  - Populating full format parse/emit until C.5 + ingestion carrier convergence (Branch A).
Falsification probe:
  SG-0 / grep gate: `v4/compiler/05_emit.dag` and `06_translate.dag` do not import
  `v4.extdeps.formats.*` or `frameworks.react` after D contract PR.
Metric allowed only as secondary:
  Count of compiler→format imports (target: 0); secondary to structural grep in CI.
```

## §1 Branch D row map (ctrl#1425 §3)

| Row | Deliverable | When |
|-----|-------------|------|
| D.1 | OpenAPI + SQL Shape B contract (consumer graph + anti-import receipt) | W4.6 |
| D.2 | React/framework Shape B contract (user-program projection only) | W4.6 |

## §2 Shape B GUARDED checklist (§2.3.1)

- [ ] No new `emit_*` paths for OpenAPI, SQL, React, or Markdown in compiler stages.
- [ ] Format modules remain extdeps fact-bundles; render/projection is user `.dag` or downstream tool.
- [ ] Receipts name `TargetSource` vs format artifact paths separately (§2.7.4).
- [ ] Stress probes (e.g. `spice.dag` Shape B falsification) stay in test/claim, not compiler.

## §3 Per-target routing (§6.7)

| Target | RCA mgr session | Emission mgr role |
|--------|-----------------|-------------------|
| Rust | vivid-lynx | Hand off C.6 + PR @-message |
| Python | vivid-eagle | Hand off C.7 |
| Go | gentle-lynx | Hand off C.8 |
| TypeScript | swift-fox | Hand off C.9–C.10 slice |

Cross-target Emission Mgr owns substrate contract (C.1–C.5, D.1–D.2 docs) — not duplicate emitters per language.

## §4 Test plan

- Contract PR: grep/static receipt only (no substrate breakage)
- Post-W4.6: format `TestClaim` rows cite D.1 carriers without compiler emit imports
