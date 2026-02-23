# M7-D: Secret Redaction by Default

## Status

- Decision: **Approved for implementation**
- Scope: `core/ir`, `core/exec`, render/codegen surfaces, lint guardrails

## Problem

Secret-bearing values are redacted in many display paths, but plaintext extraction still exists broadly enough to risk accidental leakage when new code paths are added. The contract must be capability-split and fail-closed.

## Required contract

1. `Display` / `Debug` / `.to_string()` for secret-bearing values are **always redacted**.
2. Plaintext extraction is explicit and boundary-scoped (`*_for_transport` API naming).
3. Non-boundary usage of plaintext extraction is lint-audited and denied by default.
4. Renderers and execution/status output never emit plaintext secrets.

## Capability split model

### Runtime secret carriers

- `Value::Secret(SecretString)`
- transport credential `Secret`

These may hold plaintext internally, but have no implicit formatting path that reveals content.

### Redacted render surface

- Any formatter/logging path sees only redacted forms (`***`).
- Secret-aware renderers must preserve redaction at code/text output boundaries.

### Plaintext boundary capability

- `expose_plaintext_for_transport()` remains the only plaintext extraction API.
- Semantics: allowed only at outbound transport adaptation boundaries and cryptographic serialization boundaries that must materialize bytes.

## DAG/resource/admission implications

- Secret flow remains typed (`Secret`, `Credential`) and explicit through ports/resources.
- No scheduler/admission bypass: secret extraction cannot be used as an alternate data path around transport/resource modeling.

## Invalidation and migration strategy

1. Keep existing `expose_plaintext_for_transport` API stable.
2. Continue deprecating/removing legacy aliases (`expose`).
3. Tighten lint policy for plaintext extraction usage scope.
4. Audit callsites:
   - keep boundary callsites,
   - replace non-boundary callsites with redacted-safe alternatives.
5. Add regressions proving no plaintext appears in display/render outputs.

## Enforcement plan

- Lint-level policy:
  - deny legacy alias use,
  - add targeted disallowed-method enforcement for plaintext extractors outside approved modules where feasible.
- Test-level policy:
  - secret formatting tests for `Value`, transport credentials, and renderer outputs.

## Acceptance criteria (M7)

- Redaction invariant holds for all standard formatting paths.
- Plaintext extraction is explicit and grep-auditable.
- New regression tests fail if plaintext appears in renderer/display outputs.
