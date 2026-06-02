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

Authority rows:
- Fixture sources: `python_l2_parity_{python,rust,go}_source` in `src/v4/lens/leaf_model_verification.dag`.
- Runner: `scripts/v4-leaf-model-python-l2-cross-target-parity-verify.sh`.

The L2 receipt compares stdout across Python, Rust, and Go for the common-domain subset:
- R2a integer add/order on small values.
- R3-external Symbol nominal/value projection to the same displayed payload.

R2b arbitrary precision is not asserted as cross-target equality against Rust `i32` or Go `int`. That is a modeled divergence receipt from L0/L1, not an L2 parity claim.
