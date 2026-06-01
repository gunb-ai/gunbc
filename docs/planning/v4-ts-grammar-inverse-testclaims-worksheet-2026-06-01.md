# v4 TS Grammar-Inverse TestClaims Worksheet (#3850 line, alpha lane)

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). Orthogonal to leaf-model L0 track.  
> **Lane:** ALPHA/PREVIEW — NOT v4-done / release-minimum gate.  
> **Authority:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §8.7 (#3850); T-11 MVP-1 receipts on main.  
> **Existing substrate:** `mvp1_typescript_add_translate.dag`, `mvp1_typescript_record_task_translate.dag`, `typescript_wave2a*.dag`.

---

## Mechanical dispatch rule

> **No TS grammar-inverse expansion worker may land until this worksheet is Modeling DFS Arbiter–approved.**

Acceptance is **CompilesClaim / EqualsClaim** receipts on named productions — not "translate compiles" without grammar-inverse path exercised.

---

## §10.0-adapted worksheet

```text
Migration class:        TS-GRAMMAR-INVERSE-TESTCLAIMS (Shape-A target, alpha/preview)
Representative failure:  T-11 typescript slice proves MVP-1 add-fn compile-inferred path, but
                         grammar-inverse coverage is not enumerated per production; packaging/module
                         layout unmodeled — L1 blocked silently.
Immediate local patch:
  Expand hand-authored expected source strings in manual tests without production→claim map.
Why that patch is forbidden:
  Claims must name grammar productions / TargetModel edges as authority (mvp1_rust pattern).
DFS path:
  std/ authority:
    - grammar.dag relation rows, derive_grammar_relation_row
  extdeps/language authority:
    - typescript.dag — ts_production_mvp1_fn_add, wave2a type/record productions
  compiler:
    - 06_translate.dag grammar-inverse serialize (translate_grammar_inverse_* )
  test/claim:
    - src/v4/test/claim/manual/mvp1_typescript_*.dag (extend, do not fork parallel vocab)
Deepest unsound boundary:
  Grammar-inverse path is exercised for one MVP-1 fixture but not receipt-closed per production
  class; module resolution not in model.
Systemic fix:
  Phase 1 L0 scope (ratified minimal set):
    G1. mvp1 add-fn — production ts_production_mvp1_fn_add (existing anchor; refresh labels)
    G2. wave2a type_alias_decl — production ts_production_wave2a_type_annotation family
         (record_task_translate.dag authority)
  Each: TestClaim asserts compile_inferred → emit → translate → grammar-inverse → canonical
  source_text equals ts_mvp1_source_text / wave2a expected authority.
  Explicit NOT_IN_L0: import/export paths, package.json, project references, nodenext resolution.
Non-goals:
  - Full Rust Reference-scale grammar closure.
  - Source ingestion from .ts files on disk (§14 GAP — separate T-4 tranche).
  - v4-done L5 cross-target equivalence.
  - Replacing tsc with grammar-inverse alone (inverse is structural receipt; tsc remains L0 leaf-model).
Falsification probe:
  Mutate single token in grammar_relation_row emitted field; grammar-inverse claim MUST fail
  (EqualsClaim mismatch). Attempt inverse on production without serialize_source edge →
  translate_grammar_inverse_not_realized diagnostic (fail-closed).
Metric allowed only as secondary:
  Count of productions with Tier1 Boundary claims.
```

---

## #3850 relevance (2026-06-01 disposition)

Post-#4091 elastic CI ratification, the #3850 "grammar-inverse TestClaims for python/go/cpp/typescript Shape-A targets" line remains **relevant for TS alpha lane** but **scoped down**:

| Target | L0 action |
|--------|-----------|
| TypeScript | This worksheet — G1 + G2 only |
| python/go/cpp | Owned by respective RCA managers (release-minimum family) |

---

## Packaging / module-layout blocker (documented, not solved in L0)

TypeScript L1 (compiler-subset emit typechecks) requires modeling:

- `import` / `export` / `export type`
- `moduleResolution` / `module` kind (at least Node16/NodeNext stand-in)
- Single-file vs project emit

**Escalation path:** if worker hits packaging before L0 leaf-model closes, TS RCA Manager escalates via `dashboard-ops escalate` — does not smuggle packaging into grammar-inverse L0 claims.

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] L0 scope = G1 + G2 productions only
- [x] Packaging explicitly deferred to L1
- [x] Reuse manual TestClaim patterns from `mvp1_typescript_*.dag`
- [x] Alpha/preview — NOT v4-done gate
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

## Related artifacts

- `src/v4/test/claim/manual/mvp1_typescript_add_translate.dag`
- `src/v4/test/claim/manual/mvp1_typescript_record_task_translate.dag`
- `src/v4/test/claim/parse/typescript_wave2a.dag`
- `src/v4/std/grammar.dag` — `grammar-inverse-lhs-exact-selection` gate
- `docs/planning/v4-ts-leaf-model-r2-r3-external-worksheet-2026-06-01.md` (orthogonal L0 track)
