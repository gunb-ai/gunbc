# R3 Novel-Finding Worker Brief — F3 `Map<String, Bool>` used as set across graph/syntax/node files

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope.
**Authority parent**: gpt-5-5-pro reflective analysis Finding 3; PM dispatch at gunbc#846 c#4413701937 (operator authorized 2026-05-09).
**Priority**: MEDIUM — Class F missed algebraic structure; `Set<A>` already declared in std but bypassed.

---

## §0. Problem statement

`Map<String, Bool>` appears repeatedly as a presence-set across graph/syntax/node files — e.g., `src/v2/04_infer.dag:400` `fn set_has(m: Map<String, Bool>, key: String) -> Bool`. The stored `Bool` value is ignored; `set_has` exists solely because the substrate doesn't reach for `Set<A>` declared elsewhere in `dsl/std/`.

P1 Modeling Faithfulness: a present algebraic structure (`Set<A>`) is bypassed in favor of a structurally-weaker `Map<K,Bool>` shape. Field labels lie about what's stored.

## §1. Required outcome

`Map<String, Bool>` presence-set consumers migrate to `Set<String>`; `set_has` deletes (or routes to `Set::contains`).

## §2. Fix options

**Option A**: Find all `Map<String, Bool>` declarations + uses; verify each is a presence-set (Bool-value-ignored); migrate to `Set<String>`. Delete `set_has` helper after migration. Cementing test scans for any new `Map<X, Bool>` with `Bool`-ignored consumer.

**Option B**: Add ratchet test that flags new `Map<X, Bool>` declarations matching the bypassed-Set shape (regex over `.dag` files); allow existing instances to ratchet down via per-PR migration.

PM-recommended: Option A for the migration cycle; Option B as a closing ratchet if some instances can't migrate (cross-cutting constraint emerges).

## §3. Files

**Option A**:
- `src/v2/04_infer.dag` + similar across graph/syntax/node files (audit then migrate)
- `dsl/std/set.dag` (verify `Set::contains` shape; add if missing)
- consumers (typecheck migration)
- new `.dag` `TestClaim` for ratchet

## §4. Cross-cutting constraints

- Audit pass before migration: enumerate `Map<.*Bool>` instances; classify presence-set vs genuine bool-value.
- v2-side migrations (e.g., `src/v2/04_infer.dag`) are Class E v2-retirement-eligible; coordinate with PB Mgr (warm-dove-618 / gunbc#2074) on whether to migrate-then-retire or skip.
- Cross-references Class F row 4 in sweep doc.

## §5. Receipt

- All `Map<String, Bool>` presence-set consumers migrated to `Set<String>`.
- `set_has` helper deleted or aliased to `Set::contains`.
- Ratchet `TestClaim` blocks regressions.
- Sweep-doc Class F row 4 updated.

---

**End of brief.**
