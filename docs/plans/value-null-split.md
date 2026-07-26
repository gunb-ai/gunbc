# Plan — Value::Null split (Optional / Witness / lookup-miss carriers)

**Status:** lane opened (keen-ferret-250) · **Parent:** model↔realization fork §3.2 · **DESIGN.md open thread** · Linked from [model-realization-fork.md](model-realization-fork.md) and [fail-closed-lockdown.md](fail-closed-lockdown.md) §4 tier-1.

**Verified against the live tree 2026-07-26.** Census counts are receipts; re-check before acting.

## 0. Problem — one native carrier, four meanings

`Value::Null` overloads four distinct semantics today: (1) the `None`/`none` literal and `LitNull`, (2) `Optional::Absent` (bridged in `match_pattern` at `v1_interpreter.rs:2830`), (3) `Witness::Violates` on map miss (bridged at `:2808` with a fabricated diagnostic), (4) untyped lookup miss (`raw_map_lookup`, `list_get_at_or_null`, `get` on map/list). A blanket `CrossRepresentationEquality` guard cannot close the Optional/Witness straddle — `present == None → false` is *legitimate* at ~218 corpus sites — so the fix is **splitting**, not grounding onto one sentinel.

## 1. Target carriers (construction authority)

| meaning | modeled form | target native | construction site | status |
| --- | --- | --- | --- | --- |
| Optional absent | `Absent` (`collection.dag`) | `Value::Variant { Optional, Absent }` | `optional_absent()` `:420` | LANDED at construction; still bridged from `Null` in `match_pattern` |
| Optional present | `Present { value }` | `Value::Variant { Optional, Present, .. }` or raw payload + Present bridge | `optional_present()` `:412`; `map_lookup_as_optional` `:428` | partial — `map_get` uses bridge; raw `get` on map still returns `Null` on miss |
| Witness holds | `Holds { value }` | `Value::Variant { Witness, Holds, .. }` | `witness_holds()` `:3333` | LANDED |
| Witness violates | `Violates { diagnostic }` | `Value::Variant { Witness, Violates, .. }` | `witness_violates()` `:3341`; `raw_map_lookup_witness` `:9903` | LANDED at construction; `Null` still bridged to `Violates` in `match_pattern` |
| Lookup miss (untyped) | (no coproduct — host sentinel) | `Value::Null` narrowed to this role only | `raw_map_lookup` `:9927`; `list_get_at_or_null` `:9877` | OPEN — still shared with `None` literal |
| `None` literal / `none` var | context-dependent (Optional sugar vs host null) | dissolve into `Absent` variant OR keep as alias of lookup-miss per type context | `eval_literal LitNull` `:2041`; `eval_var none` `:2067` | OPEN — needs type-directed resolution |

## 2. Emitter rows (v2 self-host lane — write down before emitting)

Grounded Rust realization targets (cited in `extdeps.languages.rust.types` + `witness_option_bridge_test.rs`; v2 `target_model` rows must match):

| modeled | Rust emission | authority |
| --- | --- | --- |
| `Optional<T>::Absent` | `None` | `rust_none_expr` / `emit_variant_pattern` when parent is `Optional` |
| `Optional<T>::Present { value }` | `Some({value})` | `rust_some_template` |
| `Witness<C>::Holds { value }` | `v1_rt::Witness::Holds(value)` | `rust_type_checkpoints` Witness row |
| `Witness<C>::Violates { diagnostic }` | `v1_rt::Witness::Violates(diagnostic)` | same |
| lookup miss (untyped `get`) | `Option::None` only when return type is `Optional<T>`; else refuse until typed | dissolve with Phase 3 API split |

## 3. Census (2026-07-26)

1. `Value::Null` in `v1_interpreter.rs`: **53** textual sites (hash/eq/display/match bridges/production — receipt: `rg -c Value::Null src/v1/stage0/src/v1_interpreter.rs`).
2. `== None` / `!= None` in `dag/` + `src/v2/`: **218** comparison sites across **66** files.
3. `optional_absent()` call sites in v2: construction already uses modeled `Absent` variant (not `Null`).
4. Existing witnesses: `witness_option_bridge_test.dag` (map_get → Present/Absent at model layer); `cross_representation_equality.dag` roster includes `Optional` × `native_value_null` straddle (target: remove when grounded).

## 4. Phased landing (construction-first)

- [ ] **Phase A (this lane):** plan + discriminating witnesses pinning carrier invariants (`value_null_split_witness_test.dag`); emitter row table above is authority for S2 emit.
- [ ] **Phase B:** stop *producing* `Null` where the return type is `Optional<T>` — route `map_get`/`get`+Optional context through `map_lookup_as_optional` (keep raw `get` returning bare value + `Null` miss for `map_lookup_dual_dispatch` until typed overload lands).
- [ ] **Phase C:** delete `match_pattern` `Null` bridges (`:2808–2834`) once no producer emits `Null` for Optional/Witness arms.
- [ ] **Phase D:** type-directed `None` literal → `optional_absent()` where inhabiting `Optional<_>`; migrate `== None` sites (~218) to `match Absent` or `optional_is_absent` helper.
- [ ] **Phase E:** cross-representation equality — ground `Optional` so `Absent` variant reconciles with narrowed `Null` *only* at the lookup-miss boundary; remove `Optional`×`native_value_null` from testgen roster; bundle `CrossRepresentationEquality` guard removal with this phase (fenced in model-realization-fork §3.1).

## 5. Review bar

No absorbing fallback: a miss must not widen to `Null` when the type is `Optional` or `Witness`. No new per-site `if matches!(v, Value::Null)` bridges without a counted dissolution trigger. Every phase ships with a discriminating witness that goes RED when the carrier regresses.

## Dissolution trigger (DESIGN §6)

Delete when `Value::Null` means only untyped lookup-miss, `Optional::Absent` and `Witness::Violates` are construction-reached only as their `Value::Variant` forms (no `match_pattern` Null bridges), the testgen `Optional`×`native_value_null` straddle row is removed, and `CrossRepresentationEquality` backstop for numerics is the only remaining guard.
