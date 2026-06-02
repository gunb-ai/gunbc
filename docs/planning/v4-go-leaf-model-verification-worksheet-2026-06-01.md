# v4 Go Leaf-Model Verification Worksheet - R1/R2 Release-Minimum Lane

> **Status:** **WORKSHEET APPROVED - READY-FOR-WORKER-DISPATCH** - Go RCA Manager release-minimum lane for #4137 Section 11.8. **Scope:** L0 complete for `go.dag` leaf-model **R1/R2a/R2b**, with L1 fixture-scale unblocked by `v4-go-l1-compiler-slice-compile-worksheet-2026-06-01.md`.
> **Date:** 2026-06-01
> **Dispatch anchor:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` Section 11.8.1 Go row target "L0 complete + L1 fixture-scale".
> **Prerequisites:** #4076 landed; `src/v4/extdeps/languages/go.dag` carries Go integer fact-bundles and atom realization rows touched by this worksheet.

---

## Mechanical dispatch rule

L0 proves **leaf-model claim wiring and target-toolchain fixture behavior** for bounded Go rows. It does not claim L2 cross-target execution parity, L3 fixed point, or full compiler self-compile.

Acceptance is:

1. Go facts are named from `go.dag`, not from local strings.
2. Each claim has a happy fixture and a falsification fixture.
3. The fixture pair is represented in `.dag` claim authority.
4. The temporary host bridge exercises the fixture while T-22 modeled `run_target_verification` is not yet live.
5. The bridge has a dissolve path to `.dag` `TestClaim` execution.

---

## Section 10.0-adapted worksheet

```text
Migration class:        GO-LEAF-MODEL-R1-R2
Representative failure: Go emit layout fixed in #4076, but go.dag leaf facts could still be
                        string-adjacent rather than proven by target fixture rows.
Immediate local patch:  Manually run go build/go run snippets in a PR note.
Why forbidden:          Not structural, no falsification pair, no claim receipt, no dissolve path.
DFS path:
  model facts:
    - src/v4/extdeps/languages/go.dag integer primitive fact-bundles
    - src/v4/extdeps/languages/go.dag atom realization rows
  fixture authority:
    - src/v4/lens/leaf_model_verification.dag Go R1/R2a/R2b fixture pair functions
  claim rows:
    - src/v4/test/claim/language_model/go_r1.dag
    - src/v4/test/claim/language_model/go_r2a.dag
    - src/v4/test/claim/language_model/go_r2b.dag
  temporary verifier:
    - src/v3/compiler/tests/boundary/v4_leaf_model_go_r1_r2_r3_external_test.rs
Deepest unsound boundary:
  Host Rust still invokes go build/go run. This is a bridge, not the authority.
Systemic fix:
  (1) Keep fact anchors in .dag claim rows.
  (2) Keep happy and falsification rows paired.
  (3) Retire the host Rust bridge when T-22 run_target_verification executes these TestClaims.
Non-goals:
  - L1 compiler-slice go build receipt (separate worksheet).
  - L2 cross-target evaluation equivalence.
  - L3 self-output fixed point or L4 compiler self-compile.
Falsification probes:
  R1: use i32 in Go source; go build must reject undefined type.
  R2a: call missing int method log2_exact; go build must reject undefined method.
  R2b: assert the wrong int64 overflow result; go run must panic.
Metric allowed only as secondary:
  Number of Go files in a downstream emitted package.
```

---

## Section 4 Claim Rows

| Row | Claim id | Fact anchor | Happy fixture | Falsification |
|---|---|---|---|---|
| R1 | `leaf_model_claim_go_r1_int_surface_spelling` | `go_surface_spelling_int` projected as a Node atom | `func r1() int { return 0 }` accepted by `go build` | `func r1() i32` rejected as undefined type |
| R2a | `leaf_model_claim_go_r2a_int_algebra_operations` | `algebra_inhabitance_node(go_integer_algebra_inhabitance(go_facts_int))` | `a + b` and `a < b` accepted by `go build` | `a.log2_exact()` rejected as undefined method |
| R2b | `leaf_model_claim_go_r2b_int64_silent_overflow_truncates` | `go_integer_facts_node(go_facts_int64)` | `math.MaxInt64 + 1` observed as `math.MinInt64` under `go run` | wrong expected wrap result panics |

R2b is a runtime exercise because Go typed integer overflow behavior is observed at execution time for the fixture. It remains L0 leaf-model verification, not L2 cross-target parity.

---

## Section 8 Modeling DFS checklist

- [x] Facts named from `go.dag` (`go_surface_spelling_int`, `go_facts_int`, `go_facts_int64`)
- [x] R1/R2a/R2b each have happy + falsification fixture rows
- [x] Claim rows are `.dag` data under `src/v4/test/claim/language_model/`
- [x] Temporary host bridge is same-path SG-0 residual with dissolve path to T-22
- [x] L1 compiler-slice worksheet can depend on these L0 rows without claiming L2/L3/L4

---

## Related Artifacts

- `src/v4/extdeps/languages/go.dag` - Go language model fact authority
- `src/v4/lens/leaf_model_verification.dag` - Go fixture-pair constructors
- `src/v4/test/claim/language_model/go_r1.dag` - R1 claim row
- `src/v4/test/claim/language_model/go_r2a.dag` - R2a claim row
- `src/v4/test/claim/language_model/go_r2b.dag` - R2b claim row
- `src/v3/compiler/tests/boundary/v4_leaf_model_go_r1_r2_r3_external_test.rs` - temporary host bridge
- `docs/planning/v4-go-l1-compiler-slice-compile-worksheet-2026-06-01.md` - downstream L1 fixture-scale worksheet
