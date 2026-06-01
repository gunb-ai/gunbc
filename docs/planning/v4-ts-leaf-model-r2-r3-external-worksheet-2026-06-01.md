# v4 TS Leaf-Model R2a / R2b / R3-external Worksheet (MW-D3 alpha lane)

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). Alpha/preview lane only.  
> **Lane:** ALPHA/PREVIEW — NOT v4 release-minimum (Wave F F3). Mirror landed pattern: Python #4117, Rust #4000.  
> **Authority:** `docs/planning/v4-leaf-model-verification-2026-05-30.md` §5–§7; `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.8.2.  
> **Prerequisite worksheets:** `v4-ts-target-atom-realization-worksheet-2026-06-01.md` (R3-external row must exist before R3 worker); `v4-ts-algebra-inhabitance-widening-worksheet-2026-06-01.md` (stable fact IDs for R2a/R2b subjects).

---

## Mechanical dispatch rule

> **No TS R2a/R2b/R3-external implementation worker may land until this worksheet is Modeling DFS Arbiter–approved.**

Acceptance is dual-path verification (happy + falsification) against **tsc** and, where noted, **Node** runtime — not TestClaim compile-only wiring alone.

---

## §10.0-adapted worksheet

```text
Migration class:        TS-LEAF-MODEL-R2-R3-EXTERNAL (MW-D3 cross-target widening, alpha lane)
Representative failure:  typescript.dag declares ApproximateField<number> and OrderedRing<bigint>
                         algebra inhabitance, but no LeafModelClaim + host runner exercises ECMA
                         semantics; emit/translate bugs masquerade as "TS backend gaps."
Immediate local patch:
  Hand-author emitted .ts snippets in integration tests without modeled claim IDs or
  falsification probes — parallel authority on TS numeric/atom facts.
Why that patch is forbidden:
  INVARIANTS P2 — claim corpus must reference stable fact IDs on typescript.dag;
  unfalsifiable assertions (happy path only) violate v4-leaf-model-verification §5.
DFS path:
  std/ authority:
    - LeafModelClaimId + LeafModelVerificationRunReceipt at src/v4/std/leaf_model_verification.dag
    - **Shared verdict carriers (ratified):** R2a/R3-external on `TargetCompileVerdict` +
      `target_diagnostic_ts_*` + `leaf_model_toolchain_tsc_strict`; R2b on shared
      `TargetRuntimeExerciseVerdict` — **no** `TargetTypeScriptCompileVerdict` /
      `TargetTypeScriptRuntimeVerdict` (per authority map)
  extdeps/language authority:
    - src/v4/extdeps/languages/typescript.dag — ts_number_algebra_inhabitance,
      ts_bigint_algebra_inhabitance (widened fact IDs per algebra-inhabitance worksheet)
    - R3-external: ts_target_atom_realization_symbol (per atom-realization worksheet)
  claim corpus (sibling files, Layer A):
    - src/v4/test/claim/language_model/typescript_r2a.dag
    - src/v4/test/claim/language_model/typescript_r2b.dag
    - src/v4/test/claim/language_model/typescript_r3_external.dag
  fixture authority:
    - src/v4/lens/leaf_model_verification.dag (TS fixture pairs + source strings)
  host runners (interim, dissolve-on-arrival per #4117 pattern):
    - scripts/v4-leaf-model-typescript-r2a-verify.sh
    - scripts/v4-leaf-model-typescript-r2b-verify.sh
    - scripts/v4-leaf-model-typescript-r3-external-verify.sh
  compiler stage:
    - None for Phase 1 L0 (external toolchain only). R3-internal remains out of scope (Rust-only SG-1 receipt).
Deepest unsound boundary:
  Modeled algebra + atom facts on typescript.dag are not exercised against tsc/Node.
Systemic fix:
  Phase 1 claim set (3 claim IDs) + fixture pairs + host runners (tsc/Node). Hand-Rust
  boundary test is NOT optional — see §P5(b) implementation worker receipt below (same
  gate as Python #4117; alpha lane does NOT waive Mechanism (b)).
Non-goals:
  - R1 primitive surface-spelling claims (defer unless Arbiter batches with Go L0).
  - R3-internal emit projection mutation receipt (Rust SG-1 only at L0).
  - Full ~50–100 fixture inventory (Phase 2 after framework proves useful on 3 claims).
  - Module packaging / project-reference modeling (L1 concern; see grammar-inverse worksheet).
  - tsc replacing Node for all runtime semantics (R2b may require Node where tsc is silent).
Falsification probe (per claim):
  R2a: happy exercises + and < on declared primitive; falsification calls non-existent method
        (TS2339 Property 'log2_exact' does not exist — analog E0599 / Python AttributeError).
  R2b: happy bigint add beyond Number.MAX_SAFE_INTEGER; falsification uses number lane for same
        magnitude and expects IEEE754 precision loss or tsc rejection per declared expectation.
  R3-external: happy ECMA Symbol() factory call `Symbol("x")` typechecks; falsification uses
        illegal `new Symbol("x")` (Symbol is not a constructor — ECMA-262 / MDN) or arity
        `Symbol(1, 2)` → TS2554.
Metric allowed only as secondary:
  TestClaim wiring count; NOT acceptance.
```

---

## Claim inventory (Phase 1 L0)

| Claim ID | Subject fact (typescript.dag) | Happy exercise | Falsification |
|----------|------------------------------|----------------|---------------|
| `leaf_model_claim_ts_r2a_number_algebra_operations` | `ts_number_algebra_inhabitance_ts_facts_number` | `function r2a(a: number, b: number): [number, boolean] { return [a + b, a < b]; }` | Call `a.log2_exact()` → TS2339 |
| `leaf_model_claim_ts_r2b_bigint_beyond_safe_integer` | `ts_bigint_algebra_inhabitance_ts_facts_bigint` | Runtime: `(2n ** 63n - 1n) + 1n === 2n ** 63n` via Node | Same expr in `number` lane → diverges from model |
| `leaf_model_claim_ts_r3_external_symbol_projection` | `ts_target_atom_realization_symbol` | `const s: symbol = Symbol("x");` typechecks (`Symbol()` factory; not constructable) | `new Symbol("x")` → not constructable (tsc) **or** `Symbol(1, 2)` → TS2554 |

**R3-external ECMA grounding (P1):** TypeScript `Symbol` is a callable factory, **not** a constructor — `new Symbol()` is invalid ([MDN Symbol()](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/Symbol)). Do **not** mirror Python R3’s nominal `class Symbol` + `__init__` happy path; that pattern is CPython-faithful, not TS-faithful.

## §P5(b) implementation worker receipt (hand-Rust — separate PR from worksheets)

> **This worksheet PR (#4147) lands markdown only — zero paths under `src/v3/compiler/tests/**`.**
> Hand-Rust is **forbidden** here. The first implementation worker that adds the boundary
> harness MUST satisfy **exactly one** checkable P5 Mechanism (b) receipt per
> `INVARIANTS.md` Dispatch-Discipline (b) — mirror PR #4117 / `sg0-pr-body-append.4117.txt`.

| Receipt artifact (same PR as harness) | Required content |
|--------------------------------------|------------------|
| `src/v3/compiler/tests/boundary/v4_leaf_model_typescript_r2_r3_external_test.rs` | Boundary tsc (+ Node for R2b bigint) exercise; fixture bytes pinned to `src/v4/lens/leaf_model_verification.dag` authority strings |
| `_internal/INVARIANTS_OPS.md` | New table row: path, ROADMAP **T-PB-B** / `pb_rust_tests_outside_residual_zero`, dissolution (T-22 `run_target_verification` on `typescript_r{2a,2b,3_external}.dag`), interim ratchet test names |
| `src/v3/compiler/tests/integration/sg0_census_test.rs` | **Net +1** on `EXPECTED_HAND_AUTHORED_TEST` for the boundary path (before/after counts in PR body) |
| `scripts/ci-merge/sg0-pr-body-append.*.txt` (or PR body) | SG-0 pairing `(c)` line naming path + INVARIANTS §P5(b) + paired verify shells |
| Host runners (same PR) | `scripts/v4-leaf-model-typescript-r2a-verify.sh`, `r2b-verify.sh`, `r3-external-verify.sh` |
| Claim wiring (same PR or prior) | `typescript_r2a.dag`, `typescript_r2b.dag`, `typescript_r3_external.dag` + lens fixtures |

**Forbidden:** landing boundary Rust without the INVARIANTS row + census line in the **same** PR; vague deferrals; “alpha lane exempt” from census.

---

**Numeric lane split (operator-visible):** TypeScript has no single `Int` primitive. R2a anchors on ECMA `number` (IEEE-754 binary64 / `ApproximateField`). R2b anchors on `bigint` (exact ℤ / `OrderedRing`). This mirrors Python (#4117) splitting unbounded int behavior (R2b) from algebra ops (R2a), adapted to TS’s dual numeric types.

---

## Verification toolchain

| Step | Tool | Notes |
|------|------|-------|
| Typecheck | `tsc --strict` (or `npx tsc` with repo-pinned TS version) | Primary authority for R2a/R3-external |
| Runtime | `node` on emitted `.mjs` or `.ts` via `tsx` only if tsc insufficient | R2b bigint equality |
| Packaging | Single-file fixtures in scratch dir | **No** `package.json` / path-mapping in L0 scope |

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Single-authority fact: claim IDs reference `typescript.dag` data lines only
- [x] **Shared verdict carriers:** R2a/R3-external → `TargetCompileVerdict` + `target_diagnostic_ts_*` + `leaf_model_toolchain_tsc_strict`; R2b → shared `TargetRuntimeExerciseVerdict` — **no** `TargetTypeScript*Verdict`
- [x] Prerequisites: type-expr + atom + algebra worksheets §8 APPROVED (strict order)
- [x] R3-external blocked until `ts_target_atom_realization_symbol` row lands
- [x] Alpha/preview only — NOT v4-done gate
- [x] §P5(b) receipt table accepted for implementation PR (not worksheet PR)
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

## Related artifacts

- `docs/planning/v4-leaf-model-verification-2026-05-30.md` §7 (R2a/R2b/R3-external definitions)
- Python mirror: `src/v4/test/claim/language_model/python_r{2a,2b,3_external}.dag`, `scripts/v4-leaf-model-python-*.sh`
- Rust mirror: `src/v4/test/claim/language_model/rust_r{2a,2b,3_external}.dag`
- `docs/planning/v4-ts-target-atom-realization-worksheet-2026-06-01.md`
- `docs/planning/v4-ts-algebra-inhabitance-widening-worksheet-2026-06-01.md`
