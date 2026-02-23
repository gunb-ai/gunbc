# M15-D: Typed Package Manager Modeling

## Status

- Decision: **Approved for implementation**
- Scope: `lib/tools/deps` typed install planning and adapter boundaries

## Problem

Legacy install flows can drift when package manager identity remains stringly or when install-option choice is implicit. We need strict typed identity, deterministic policy, and fail-closed unknown handling.

## Required contract

1. Package manager identity is typed (`PackageManagerId`), not free-form string.
2. Unknown IDs fail closed in strict paths.
3. Install-option selection policy is explicit/deterministic.
4. Adapter boundaries preserve required fields (`script`, `url`, `packages`) without silent loss.

## Type model

- `PackageManagerId`: closed enum (Apt, Apk, Brew, Cargo, Script, GithubRelease).
- `InstallPlan`: typed install intent:
  - `package_manager: PackageManagerId`
  - `packages: Vec<String>`
  - `script: Option<String>`
  - `url: Option<String>`

## Selection policy model

- `InstallSelectionPolicy` defines total, deterministic ranking.
- Ties are broken deterministically (manager rank, stable enum ordering, declaration index).
- No declaration-order-only implicit fallback.

## Strict vs compatibility boundaries

### Strict path

- `parse_strict` must reject unknown manager IDs.
- Invalid/underspecified plans fail fast (e.g. script manager without script, github_release without url).

### Compatibility path

- Allowed only at explicit legacy boundaries.
- May accept older surface shapes but must still validate required fields before execution.

## DAG/resource/admission implications

- Install operations remain explicit effectful steps; typed manager IDs do not bypass admission/resource controls.
- Manager identity is data, not execution escape hatch.

## Migration strategy

1. Keep compatibility parse helper only at legacy ingress points.
2. Keep strict parse as authoritative internal path.
3. Consolidate all option-selection through explicit policy.
4. Add/keep exhaustive tests over manager parsing and selection determinism.

## Acceptance criteria (M15)

- Unknown manager IDs fail closed on strict path.
- Deterministic policy is used for option selection.
- Required fields are preserved/validated across adapter conversion.
- Tests cover all supported manager IDs and policy ordering behavior.
