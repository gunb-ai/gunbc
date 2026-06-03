# v4 TypeScript Leaf-Model Algebra-Inhabitance Widening Worksheet

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — TypeScript RCA
> Manager **ALPHA/PREVIEW** lane (#4137 §11.8). **Scope:** L0 complete for
> `typescript.dag` leaf-model **R2a/R2b/R3-external**, mirroring #4117 (Python
> MW-D3 cross-target widening) and W1.7 `rust.dag` widening (#4000).
> **Date:** 2026-06-03
> **Dispatch anchor:** MW-D3 alpha lane; manager sequencing in
> `docs/planning/v4-ts-rca-manager-worksheets-2026-06-03.md`.
> **Prerequisites:** `typescript.dag` Phase 1 fact exposure rows; PR #4157 bridge
> carriers in `leaf_model_verification.dag`.

> **Non-goals:** L1 `tsc --noEmit` static structural (shared
> `TargetStaticAnalysis*` — separate future worksheet); L2 cross-target stdout
> parity; L3/L4 self-host claims.

---

## Mechanical dispatch rule

L0 proves **leaf-model claim wiring and target-toolchain fixture behavior** for
bounded TypeScript rows. It does not claim cross-target execution equivalence.

Acceptance is:

1. Facts are named from `typescript.dag`, not from local strings in claims.
2. Each claim has a happy fixture and a falsification fixture (R2b: runtime pair).
3. The fixture pair is represented in `.dag` claim authority (`EqualsClaim` wiring).
4. The temporary host bridge exercises fixtures while T-22 modeled
   `run_target_verification` is not live.
5. The bridge has a dissolve path to `.dag` `TestClaim` execution and retires from
   SG-0 census when modeled execution takes over.

---

## Section 10.0-adapted worksheet

```text
Migration class:        TS-LEAF-MODEL-R2-R3-EXTERNAL
Representative failure: typescript.dag algebra rows exist but claims could still
                        be string-adjacent without tsc/node fixture proof.
Immediate local patch:  Manually run tsc/node snippets in a PR note.
Why forbidden:          Not structural, no falsification pair, no claim receipt.
DFS path:
  model facts:
    - ts_number_algebra_inhabitance_ts_facts_number
    - ts_bigint_algebra_inhabitance_ts_facts_bigint
    - ts_atom_realization_symbol (+ ts_target_atom_realization_symbol)
  fixture authority:
    - src/v4/lens/leaf_model_verification.dag typescript_r2a/r2b/r3_external pairs
  claim rows:
    - typescript_r2a.dag, typescript_r2b.dag, typescript_r3_external.dag
  temporary verifier:
    - v4_leaf_model_typescript_r2_r3_external_test.rs (integration module)
Deepest unsound boundary:
  Host Rust invokes npx tsc and node. Bridge only.
Systemic fix:
  (1) Keep fact anchors in .dag claim rows.
  (2) Keep happy and falsification rows paired.
  (3) Restore host shell scripts for operator-local repro (#4117 test-plan parity).
  (4) Retire boundary + scripts when T-22 executes TestClaims.
Non-goals:
  - L1 static structural tsc profile on shared carriers.
  - L2 cross-target behavioral parity.
  - R1 int surface spelling (Go/Python carry R1; TS alpha lane starts at R2a).
Falsification probes:
  R2a: call log2_exact on number; tsc TS2339.
  R2b: demonstrate bigint exactness vs number IEEE754 false positive at runtime.
  R3: new Symbol(); tsc TS7009 / not constructable.
```

---

## Section 4 Claim Rows

| Row | Claim id | Fact anchor | Happy fixture | Falsification |
|---|---|---|---|---|
| R2a | `leaf_model_claim_ts_r2a_number_algebra_operations` | `ts_number_algebra_inhabitance_ts_facts_number` | `[a+b, a<b]` over `number` accepted by `tsc --strict --noEmit` | `a.log2_exact()` rejected TS2339 |
| R2b | `leaf_model_claim_ts_r2b_bigint_beyond_safe_integer` | `ts_bigint_algebra_inhabitance_ts_facts_bigint` | `(2n**63n-1n)+1n === 2n**63n` under `node` | bigint inequality + number-lane IEEE754 divergence |
| R3-external | `leaf_model_claim_ts_r3_external_symbol_projection` | `ts_atom_realization_symbol` | `const s: symbol = Symbol("x")` accepted by `tsc` | `new Symbol("x")` rejected |

R2b is a **runtime** exercise (`LeafModelTypeScriptRuntimeFixturePair`) because
ECMAScript `bigint` exactness and `number` IEEE754 behavior diverge at execution
time. It remains L0 leaf-model verification, not L2 cross-target parity.

---

## Section 8 Modeling DFS checklist

- [x] Facts named from `typescript.dag` (number/bigint inhabitance + symbol atom)
- [x] R2a/R2b/R3 each have happy + falsification fixture rows in lens
- [x] Claim rows under `src/v4/test/claim/language_model/`
- [x] Temporary host bridge in SG-0 census with dissolve path to T-22
- [ ] Host shell scripts `scripts/v4-leaf-model-typescript-r2*-verify.sh` (debt)
- [ ] Modeled `LeafModelVerificationRunReceipt` populated (still `pending_verification`)

---

## Test plan (operator / worker)

- [x] `cargo test -p v3-compiler --test integration v4_leaf_model_typescript_r2_r3_external` — 9 passed (2026-06-03)
- [ ] `./scripts/v4-leaf-model-typescript-r2a-verify.sh` — script not landed
- [ ] `./scripts/v4-leaf-model-typescript-r2b-verify.sh` — script not landed
- [ ] `./scripts/v4-leaf-model-typescript-r3-external-verify.sh` — script not landed

---

## Related Artifacts

- `src/v4/extdeps/languages/typescript.dag` — fact authority (§2214 comment)
- `src/v4/lens/leaf_model_verification.dag` — fixture constructors
- `src/v4/test/claim/language_model/typescript_r2a.dag`
- `src/v4/test/claim/language_model/typescript_r2b.dag`
- `src/v4/test/claim/language_model/typescript_r3_external.dag`
- `src/v3/compiler/tests/boundary/v4_leaf_model_typescript_r2_r3_external_test.rs`
- `docs/planning/v4-ts-rca-manager-worksheets-2026-06-03.md`
