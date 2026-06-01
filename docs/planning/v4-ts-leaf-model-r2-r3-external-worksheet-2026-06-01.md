# v4 TS Leaf-Model R2a / R2b / R3-external Worksheet

> **Status:** L0 complete on `main` (implementation PR #4157).  
> **Lane:** ALPHA/PREVIEW, not release-minimum per Wave F F3.  
> **Pattern:** mirrors Python #4117: modeled claim rows plus interim host runners and an SG-0 hand-test receipt.

## Receipt

```text
Migration class:
  TS-LEAF-MODEL-R2-R3-EXTERNAL

Representative failure:
  TypeScript numeric and Symbol facts were modeled, but no leaf-model
  claim plus falsification runner exercised them against tsc/Node.

Forbidden local patch:
  Hand-author TypeScript snippets in Rust tests without modeled claim IDs,
  stable TypeScript fact IDs, and paired falsification probes.

DFS path:
  std authority:
    src/v4/std/leaf_model_verification.dag
  extdeps authority:
    src/v4/extdeps/languages/typescript.dag
  fixture authority:
    src/v4/lens/leaf_model_verification.dag
  claim corpus:
    src/v4/test/claim/language_model/typescript_r2a.dag
    src/v4/test/claim/language_model/typescript_r2b.dag
    src/v4/test/claim/language_model/typescript_r3_external.dag
  interim host runners:
    scripts/v4-leaf-model-typescript-r2a-verify.sh
    scripts/v4-leaf-model-typescript-r2b-verify.sh
    scripts/v4-leaf-model-typescript-r3-external-verify.sh

Systemic fix:
  Three L0 TypeScript leaf-model claims with happy and falsification fixtures.
  The hand-Rust boundary harness is temporary and paired with the SG-0 census
  receipt, exactly like #4117.
```

## Claim Inventory

| Claim ID | Subject fact | Happy exercise | Falsification |
| --- | --- | --- | --- |
| `leaf_model_claim_ts_r2a_number_algebra_operations` | `ts_number_algebra_inhabitance_ts_facts_number` | `number` addition and comparison accepted by `tsc --strict` | `a.log2_exact()` rejected with TS2339 |
| `leaf_model_claim_ts_r2b_bigint_beyond_safe_integer` | `ts_bigint_algebra_inhabitance_ts_facts_bigint` | Node proves bigint add beyond `2**63` is exact | `number` lane demonstrates IEEE-754 precision divergence |
| `leaf_model_claim_ts_r3_external_symbol_projection` | `ts_atom_realization_symbol` | `const s: symbol = Symbol("x");` accepted | `new Symbol("x")` rejected as non-constructable |

## L0 Acceptance

- `src/v4/lens/leaf_model_verification.dag` owns the fixture source strings.
- The three `typescript_r*.dag` claim files reference TypeScript fact IDs by Symbol equality.
- `src/v3/compiler/tests/boundary/v4_leaf_model_typescript_r2_r3_external_test.rs` pins fixture bytes to the `.dag` authority and executes `tsc`/Node.
- `src/v3/compiler/tests/integration/sg0_census_test.rs` carries the net +1 hand-authored test receipt.
- `scripts/ci-merge/sg0-pr-body-append.4157.txt` records the P5(b) pairing and T-PB-B dissolution path.

## Non-Goals

- R1 primitive surface-spelling inventory.
- R3-internal row mutation receipts.
- TypeScript package/module layout modeling.
- Replacing the interim host runners before T-22 modeled `run_target_verification` owns target verdicts.

