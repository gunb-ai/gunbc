# v4 Python RCA Manager Worksheets - 2026-06-01

Scope: release-minimum lane for #4137 section 11.8. These worksheets separate Python L1 static structural evidence from L2 cross-target behavioral parity.

## Worksheet A - pyright L1 Static Structural

Authority rows:
- Profile: `src/v4/extdeps/typecheckers/pyright.dag` (`pyright_profile_l1`).
- Fixture: `src/v4/lens/leaf_model_verification.dag` (`python_l1_static_fixture`).
- Claim wiring: `src/v4/test/claim/language_model/python_l1_static.dag`.
- Runner: `scripts/v4-leaf-model-python-l1-static-verify.sh` (removed from tree in #4252 script hygiene; pyright `.dag` rows remain authoritative until the host runner is restored).

Fixture: a function annotated `-> int` returns `str`. CPython compile and runtime both miss this when the return value is not consumed. pyright must reject it with `reportReturnType` under the modeled profile. This proves a third static authority; it is not CPython compile/runtime and not L2 behavioral parity.

## Worksheet B - mypy L1 Static Structural

Authority rows:
- Profile: `src/v4/extdeps/typecheckers/mypy.dag` (`mypy_profile_l1`).
- Fixture: `src/v4/lens/leaf_model_verification.dag` (`python_l1_static_mypy_fixture`).
- Runner: `scripts/v4-leaf-model-python-l1-mypy-static-verify.sh`.

The mypy lane reuses the same fixture and claim id as Worksheet A, but with a distinct tool/profile namespace. Expected rejection is the mypy `return-value` code under `--strict --show-error-codes`. This is intentionally weaker than a full Python semantic verifier: it only proves structural return-type evidence for the L1 fixture.

## Worksheet C - L2 Cross-Target Behavioral Parity

Authority rows (implemented):
- Carrier: `LeafModelCrossTargetParityProbe` + `ValueDiff<String>` in `src/v4/std/leaf_model_verification.dag` (positive complement of `LeafModelCrossRuntimeDriftProbe`).
- Probes: `python_l2_parity_r2a_probe` / `python_l2_parity_r3_probe` (+ `python_l2_parity_probe_roster`) with `python_l2_parity_{r2a,r3}_{python,rust,go}_source` and `_value` in `src/v4/lens/leaf_model_verification.dag`.
- Claim wiring: `src/v4/test/claim/language_model/python_l2_cross_target_parity.dag` (wiring + positive-parity `TestClaim`s).
- Boundary host exercise: `src/v3/compiler/tests/boundary/v4_leaf_model_python_l2_cross_target_parity_test.rs` (Python + Rust mandatory, Go corroborating when present), until the modeled `TestClaim` runner executes the target sources directly (same gate as the drift lane).

The L2 receipt compares stdout across Python, Rust, and Go for the common-domain subset:
- R2a integer add on small values (`2 + 3 = 5`, inside the subdomain where arbitrary-precision and two's-complement coincide). Order's boolean surface formatting differs per language and is excluded from the numeric parity payload.
- R3-external Symbol nominal/value projection to the same displayed payload (`x`).

R2b arbitrary precision is not asserted as cross-target equality against Rust `i32`/`i64` or Go `int64`. That is the modeled divergence receipt (`python_cross_runtime_drift`), the exact complement of this parity lane, not an L2 parity claim.
