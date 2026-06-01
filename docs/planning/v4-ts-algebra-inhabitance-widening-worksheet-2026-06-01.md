# v4 TS Algebra Inhabitance Widening Worksheet (stable fact IDs)

> **Status:** DRAFT — `ready-for-review` (TypeScript RCA Manager `fierce-fox-719`; 2026-06-01).  
> **Lane:** ALPHA/PREVIEW — NOT release-minimum.  
> **Analog:** W1.7 / Rust #4000 widening (`rust_integer_algebra_inhabitance_rust_facts_i32`); Python #4117 (`python_integer_algebra_inhabitance_python_facts_int`).  
> **Authority:** `docs/planning/v4-leaf-model-verification-2026-05-30.md` §5 Layer A (model owns facts; claims reference fact IDs).

---

## Mechanical dispatch rule

> **No TS algebra widening worker may land until this worksheet is Modeling DFS Arbiter–approved.**

This worksheet is **substrate on typescript.dag** (fact ID exposure). Leaf-model host runners are **downstream** in the R2/R3-external worksheet.

---

## §10.0-adapted worksheet

```text
Migration class:        TS-ALGEBRA-INHABITANCE-WIDENING (W1.7 / #4000 analog for typescript.dag)
Representative failure:  ts_number_algebra_inhabitance(facts) and ts_bigint_algebra_inhabitance(facts)
                         exist as fn bodies but lack stable top-level data fact IDs — claim corpus
                         cannot reference subjects without duplicating AlgebraInhabitanceDecl shapes.
Immediate local patch:
  LeafModelClaim Subject coproduct arms per primitive on typescript.dag (forbidden by W2.6b deferral).
Why that patch is forbidden:
  Phase 1 discriminator is LeafModelClaimId + fact Symbol references (rust/python pattern).
DFS path:
  std/ authority:
    - AlgebraInhabitanceDecl at src/v4/std/model_core.dag (consume only)
  extdeps/language authority:
    - src/v4/extdeps/languages/typescript.dag
  claim corpus:
    - References widened data lines (typescript_r2a.dag, typescript_r2b.dag)
Deepest unsound boundary:
  Algebra inhabitance facts are not addressable as stable claim subjects.
Systemic fix:
  Add top-level data bindings (names ratified here; worker may adjust spelling with Arbiter):
    data ts_number_algebra_inhabitance_ts_facts_number: AlgebraInhabitanceDecl =
      ts_number_algebra_inhabitance(facts: ts_facts_number)
    data ts_bigint_algebra_inhabitance_ts_facts_bigint: AlgebraInhabitanceDecl =
      ts_bigint_algebra_inhabitance(facts: ts_facts_bigint)
    data ts_bool_algebra_inhabitance_ts_facts_boolean: AlgebraInhabitanceDecl =
      ts_bool_algebra_inhabitance()
  Optional Phase 1.5:
    data ts_string_algebra_inhabitance_ts_facts_string: AlgebraInhabitanceDecl =
      ts_string_algebra_inhabitance(facts: ts_facts_string)
Non-goals:
  - New Subject/Expectation coproduct types on typescript.dag (W2.6b posture).
  - Changing claimed algebra (ApproximateField vs OrderedRing) — verification may FALSIFY model;
    that routes to Modeling DFS per v4-leaf-model-verification §9, not silent emit patch.
  - Wave-2b law obligation expansion (separate T-4 tranche).
Falsification probe:
  After widening: claim fixture pair in typescript_r2a.dag references
  ts_number_algebra_inhabitance_ts_facts_number by Symbol equality (compile-time TestClaim).
  Mutate witness node in typescript.dag; claim wiring test MUST fail without claim file edit.
Metric allowed only as secondary:
  Count of AlgebraInhabitanceDecl rows with exported fact IDs.
```

---

## Claim ↔ fact wiring table

| Stable fact ID | Algebra (modeled) | Leaf-model consumer |
|----------------|-------------------|---------------------|
| `ts_number_algebra_inhabitance_ts_facts_number` | `ApproximateField` on `number` / IEEE-754 | R2a |
| `ts_bigint_algebra_inhabitance_ts_facts_bigint` | `OrderedRing` on `bigint` | R2b |
| `ts_bool_algebra_inhabitance_ts_facts_boolean` | `BooleanAlgebra` on `boolean` | Phase 2 unless batched |
| `ts_string_algebra_inhabitance_ts_facts_string` | `FreeMonoid` on string code units | Phase 2 unless batched |

---

## §8 Manager approval checklist — OPEN

- [ ] No new Subject/Expectation coproduct at Phase 1
- [ ] `LeafModelClaimId` + fact Symbol discriminator only
- [ ] Number vs bigint split explicit (no faux unified `Int`)
- [ ] Worker may land **before** R2 host runners but **after** Arbiter approval
- [ ] Worker dispatch — **forbidden** until Arbiter sign-off

## Related artifacts

- `docs/planning/v4-w2.6b-python-leaf-model-claim-worksheet-2026-05-30.md` (deferral precedent)
- `src/v4/extdeps/languages/python.dag` — `python_integer_algebra_inhabitance_python_facts_int`
- `docs/planning/v4-ts-leaf-model-r2-r3-external-worksheet-2026-06-01.md`
