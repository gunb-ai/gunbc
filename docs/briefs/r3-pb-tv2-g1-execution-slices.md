# R3 PB — T-V2-Retirement G-1 execution slices

**Status:** EXECUTION BRIEF for issue #1975. Authored after the PM S-1 worker brief landed, using the later Population A/B audit deltas as the current substrate-grep receipt.

**Scope:** Population B G-1 only: matrix §3.1, §3.2, and §3.3. This brief does not authorize `src/v2/` deletion, workspace-member removal, Pop A property-test migration, or `verification.dag` convergence.

## Current gates

S-1 is met by [`r3-tv2-retirement-s1-worker-brief.md`](r3-tv2-retirement-s1-worker-brief.md), so the original S-1 STOP no longer blocks planning. The remaining gates are per-slice:

| Slice | Current gate | Execution posture |
|---|---|---|
| §3.1 `p0_std_render_repeat_string_test.rs` | Needs either evaluator-hosted replacement or a structural-guarantee receipt for deletion. | HELD until one branch is proven. |
| §3.2 `m2_substrate_inhabitance_test.rs::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` | Substrate authority migration is already landed; the v2 mirror test is now redundant, but Cargo deps cannot drop until §3.1 is green. | READY as an isolated parity-test retirement, or bundle with §3.3 when §3.1 lands. |
| §3.3 `src/v3/compiler/Cargo.toml` v2 path deps | Both §3.1 and §3.2 consumers must be gone first. | ATOMIC with the second consumer-removal PR. |

Load-bearing substrate-grep receipt: [`r3-pb-tv2-population-coverage-audit.md`](r3-pb-tv2-population-coverage-audit.md) §"B.2 reclassification" and the 2026-05-06 delta. It verifies:

- `Dag::kernel_algebra_profile` reads lowered `data kernel_algebra_profile` from `dsl/std/algebra.dag`.
- `v3_kernel_algebra_profile_reads_lowered_dag_map_authority` already ratchets the v3-side authority read.
- The surviving v2 oracle line and `v2_profile_to_v3` shim are drift-ratchet residue, not semantic authority.
- The `v2-compiler` / `v2-compiler-tests` Cargo edges are still present solely because §3.1 and §3.2 consumers still compile.

## Slice §3.1 — replace vs delete branch

**Target:** `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs`.

**Property:** `repeat_string` / `indent_text` lower-time fold yields the expected `String` result. The current test proves this by compiling through v2 and running the v2 interpreter.

Workers must choose exactly one branch before editing the test:

| Branch | Required receipt before edit | Allowed edit |
|---|---|---|
| **Replace** | Evaluator surface can execute the fixture and compare the result without v2. | Replace the v2 oracle with a v3 evaluator equivalence-corpus row or v3-side test fixture preserving the same `repeat_string` / `indent_text` assertions. |
| **Delete** | Substrate/evaluator owner signs a structural-guarantee receipt that the fold is guaranteed by v3 typed primitive composition and does not need an executable ratchet. | Delete the v2 oracle test and cite the guarantee in the PR body or adjacent test comment. |

No third branch is allowed. In particular, do not keep a shape-only fixture that cannot observe the string result; that would not preserve the v2 oracle's behavioral coverage.

## Slice §3.2 — parity-test retirement

**Target:** `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`.

**Ready edit:**

- Delete `v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`.
- Delete the local `v2_profile_to_v3` shim if no remaining code uses it.
- Keep `v3_kernel_algebra_profile_reads_lowered_dag_map_authority` and the rest of the v3 substrate-inhabitance tests intact.

**Why this is no longer cross-program:** the cross-program authority migration already landed. The v3 map authority is `dsl/std/algebra.dag`; the v2 stage0 map is only a mirror generated from that authority. Retiring the parity test no longer loses drift detection because the v3 authority-read test remains.

This slice may land before §3.1 as a small PB-internal PR, but it must not delete Cargo deps unless §3.1 is also green in the same PR.

## Slice §3.3 — Cargo edge deletion

**Target:** `src/v3/compiler/Cargo.toml`.

Delete these path deps only after both §3.1 and §3.2 no longer reference v2 crates:

```toml
v2-compiler = { path = "../../v2/stage0" }
v2-compiler-tests = { path = "../../v2/tests" }
```

This deletion should be atomic with the second consumer-removal PR. If §3.2 lands first and §3.1 remains held, keep the deps. If §3.1 lands first and §3.2 remains live, keep the deps.

## Verification

Before opening a PR that claims G-1 green:

```sh
rg -n '\bv2_compiler(_tests)?\b|v2-compiler' src/v3/compiler
cargo test -p v3-compiler
```

Expected `rg` result after full G-1: no substantive matches in `src/v3/compiler` test code or `Cargo.toml`. Cosmetic references outside the G-1 surface remain G-2 cleanup unless the deletion PR touches them for a direct reason.
