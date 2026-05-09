# R3 Novel-Finding Worker Brief — F6 `derive_op_effect(method_str, path_str)` string parser at structural boundary

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope.
**Authority parent**: gpt-5-5-pro reflective analysis Finding 6; PM dispatch at gunbc#846 c#4413701937.
**Priority**: HIGH — Class C string-keyed dispatch over typed carriers; structural-boundary violation.

---

## §0. Problem statement

`src/v2/effect_derivation.dag:20-41` exposes `derive_op_effect(method_str: String, path_str: String) -> OperationEffect`. The function parses unstructured strings (HTTP method as String, path as String) at what should be a structural boundary — typed carriers `HttpMethod` (sum-type-shaped) and `Path` (`http_path.dag`) exist but are bypassed.

P2 Boundary Discipline: structural facts encoded as strings at internal boundaries. Reverse-direction problem from F1 — here the consumer is well-typed but the producer-input boundary stringifies.

## §1. Required outcome

`derive_op_effect` consumes typed `HttpMethod` + `Path` carriers; string parsing dissolves to surface-only (parse once at HTTP boundary, structural everywhere downstream).

## §2. Fix options

**Option A (proper dissolution)**: Change signature to `derive_op_effect(method: HttpMethod, path: Path) -> OperationEffect`. Parsing happens once at the HTTP-surface boundary; `derive_op_effect` operates structurally. Cementing test pins typed-input invariant.

**Option B (hybrid)**: Keep string signature for v2 compat; add typed wrapper `derive_op_effect_typed(method: HttpMethod, path: Path)` that string-converts internally; migrate callers; delete string variant after migration.

PM-recommended: Option A if v2-retirement timeline allows direct migration. Option B if Class E v2-bridge constraint forces gradual migration.

## §3. Files

**Option A**:
- `src/v2/effect_derivation.dag` (signature change + remove string parsing)
- `src/v2/stage0/src/v2_compiler_effect_derivation.rs` (Rust consumer; v2-Class-E)
- callers (typecheck migration)
- new `.dag` `TestClaim` for typed-input invariant

## §4. Cross-cutting constraints

- v2-side surface; coordinate with PB Mgr on v2-retirement timing (Class E).
- Cross-references Class C row 6 in sweep doc.
- STOP-and-PING via Mgr inbox if `HttpMethod` typed-carrier doesn't yet exist (substrate prereq).

## §5. Receipt

- `derive_op_effect` consumes typed carriers; string parsing dissolves.
- Callers migrated.
- Cementing `TestClaim` pins typed-input invariant.
- Sweep-doc Class C row 6 updated.

---

**End of brief.**
