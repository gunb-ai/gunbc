# Nested optionality on the v1 type carrier

**Status:** design + census only. No implementation until gentle-dove-36 rules.
**Subject:** successor to parked gunbc#13467. C5 on gunbc#13280 (`match_arm_body_is_present_payload_binding`) is a claimed-scope exclusion while infer collapses `T??` and emission carries `Option<Option<T>>`.
**Counts:** taken on `origin/main` at this worktree (`session/quiet-eagle-533`), authored `src/v1/*.dag` as the authority and `src/v1/stage0/**/*.rs` as the generated mirror. Stage0 counts are expected to track `.dag` plus interpreter/witness extras.

---

## 0. What is wrong

`type Cardinality = Required | CardOptional` is a boolean. `join_optional_cardinality` is boolean OR. `with_optional_cardinality` writes the flag whether or not it is already set. So `OptionalOf(ReceiverElement)` over `List<T?>` types as `T?`, while `rust_carrier_optional_wrap` still emits `Option<…>` around a payload that is already `Option<T>`. Infer and emission answer different questions with the same name. That is DESIGN §3 meaning fork and §5 silent wrongness.

v2 already treats cardinality as a *wrapper connective with a child* (`v2.std.node` `TypeNode { connective: Cardinality }`). Nested wrappers are representable there (`src/v2/test/claim/impossible_bug/nested_optional_flatten.dag`). v1 cannot represent them on the flag. This note is the v1 carrier repair, not a v2 change.

---

## 1. Census of optional-layer readers and writers

Token `CardOptional` (includes the enum arm, constructions, and `==` / `!=`):

| corpus | count |
| --- | ---: |
| `src/v1/*.dag` | 116 |
| `src/v1/stage0/**/*.rs` | 127 |
| `src/v1/tests` | 3 |

Boolean tests `== CardOptional` / `!= CardOptional`:

| corpus | `==` | `!=` |
| --- | ---: | ---: |
| dag | 80 | 7 |
| stage0 | 86 | 7 |

`return_cardinality` field mentions: dag 324, stage0 360.

### 1.1 Writers (the constructors)

| helper / site | dag | stage0 | what it does today |
| --- | ---: | ---: | --- |
| `with_optional_cardinality` | 32 | 38 | sets flag; second wrap is identity |
| `with_required_cardinality` | 49 | 52 | clears flag; peels *all* layers (only one exists) |
| `join_optional_cardinality` | 4 | 4 | OR of two flags |
| `preserve_outer_optional_cardinality` | 19 | 19 | restore flag onto subst result if outer had it and inner does not |
| `maybe_optional` (`02_parse.dag`) | 1 def + 3 calls | same | one `?` sets the flag; a second `?` is `EatUnchanged` |
| `instantiate_algebra_type` `OptionalOf` | 1 | 1 | `with_optional_cardinality` on the inner instantiation |
| Node literals `return_cardinality: CardOptional` | in the 116 | — | parse, builtins, tests |
| `04_method.dag` builtin returns (`parse_int`, `lookup`, `get`, `Some`, …) | 8 wraps | — | one-layer optional results |

Root of the weed: `src/v1/00_core.dag` `type Cardinality = Required | CardOptional` plus the four helpers in the same file.

### 1.2 Readers by class (authored dag)

**Parse of `?`.** `maybe_optional`: 3 call sites after type expressions. One token, one flag. Authored `T??` is one layer. Field parse uses the same helper.

**Join / preserve / generic substitution.** `join_optional_cardinality` lives in `00_core.dag` and is called from `04_resolve.dag` (2). `preserve_outer_optional_cardinality`: `04_resolve.dag` 4, `04_lookup.dag` 5, `04_infer.dag` 9. `substitute_generics_apply` preserves the *outer* flag onto a substitution; if the replacement is already optional, preserve is a no-op and a layer is dropped. Structural nodes copy `return_cardinality: n.return_cardinality` after substituting children (container `List<T?>` keeps the list's own flag, not the element's).

**Implicit peel (treat optional as inner + lift).** These peel by `with_required_cardinality` then often re-wrap once:

| file | `== CardOptional` | typical function |
| --- | ---: | --- |
| `04_lookup.dag` | 4 | `lookup_field_type_node` (peel, lookup, `with_optional_cardinality` on result; `.value` returns inner); `field_summary_for_type`; `map_lookup_result_type` (skip wrap if already flagged) |
| `04_types.dag` | 9 | type equality / optional coproduct / `normalize` peel |
| `04_patterns.dag` | 5 | scrutinee Present/Absent expansion; optional vs coproduct |
| `04_resolve.dag` | 3 | peel before resolve |
| `04_infer.dag` | 39 | inhabitance, arm join, optional-at-required, casts, match payload, equality admission |
| `04_emit_info.dag` | 2 | |
| `05_emit.dag` | 6 | shared emit |
| `05_emit_rust.dag` | 35 | `rust_carrier_optional_wrap` once; call-arg Option; variant Present; optional receiver at emit |
| `05_emit_python.dag` | 2 | one wrap |
| `05_emit_go.dag` | 1 | one wrap |
| `02_parse.dag` | 3 | `maybe_optional` + constructions |
| `00_core.dag` | 5 | type + helpers |
| `compile.dag` / `compiler_tests_rust.dag` | 1+1 | |

**Already a second encoding in production.** `equality_operand_admission` (`04_infer.dag`):

`peeled.return_cardinality == CardOptional || (peeled.name == "Optional" && (peeled.children |> count) == 1)`

The flag and a nominal `Optional` application are already both peeled. That is the #13467 shape, pre-existing, for equality only.

**Algebra instantiation.** `OptionalOf` → `with_optional_cardinality`. `lookup_structural_method` does **not** peel an optional receiver before `enrich_kernel_type` / `instantiate_algebra_field`. Methods are found on the required kernel name. An optional receiver that is still flagged therefore either fails to enrich or is handled by the field-lookup peel in `lookup_field_type_node`.

**Field and structural-method lookup.** `lookup_field_type_node`: one peel. `lookup_structural_method`: no peel (product vs enrich by name). `map_lookup_result_type`: flatten-if-already-optional (OR-shaped wrap).

**Pattern expansion.** `04_patterns.dag`: Present payload is `with_required_cardinality` of the scrutinee (one peel). Nested Present on a collapsed `first()` payload is why C5 is excluded.

**Emission.** `rust_carrier_optional_wrap`: if flag then `Option<rendered>` once. Nested runtime appears when the *inner rendered type* is already `Option<_>` (generic `T` instantiated at a flagged type, or a child node's own wrap). Infer's collapsed type still type-checks `Present` as if there were one layer.

**Interpreter (stage0 only, not a `.dag` authority).** 3 `CardOptional` uses in `v1_interpreter.rs` (optional field / cardinality). Witnesses: `infer_semantics_witness.rs` 6, `diagnostics_witness.rs` 2.

**04_method.dag:** 8 `with_optional_cardinality` writes, 0 `CardOptional` token reads (builtins go through the helper).

### 1.3 Mirror vs authority

Stage0 `CardOptional` is 127 vs dag 116: the extra is interpreter + witness bins + compile/tests mirrors. Helper counts match within a few (`with_optional` 32 vs 38) from stage0 tests (`namespace_occurrence_serde.rs`) and interpreter. Treat `.dag` as the rewrite surface; stage0 regenerates.

---

## 2. What #13467 proved (option (c) evaluated)

gunbc#13467 (head `c9451441`, canonicalizing commit `a4221ecc`) kept the flag for layer 1 and used a nominal `Optional` application for layer 2, plus `canonicalize_optional_spelling` / `wrap_optional_layer` / `node_is_optional_layer` / `node_is_optional_application`.

Commit sequence is four symptom gates on the same link:

1. `44b22f35` second layer as `Optional<T?>`
2. `bf7a70d2` receiver lift: CardOptional join, not a second wrap
3. `764228ce` nest `OptionalOf` only when the element is already optional
4. `3e306166` peel optional receivers before algebra instantiate
5. then canonicalize, then three failure-mode rows for remaining flatten

On that “predicates unified” head, `04_lookup.dag` still had `normed.return_cardinality == CardOptional` and `raw.return_cardinality == CardOptional` (lines 952 and 1012 at that ref). The one-predicate claim was already false in the same files that grew `node_is_optional_layer`.

`equality_operand_admission` on main already special-cases both encodings. Adding a third helper set does not remove that fork; it staffs it.

**Ruling on (c): do not keep #13467 and do not land a dual encoding.** Two spellings of one layer-count, a canonicalize pass as a second constructor, and a reader that still tests the flag, are the attractor DESIGN §3 replacement-migration forbids. Four successive gates are the receipt that the next optional-layer reader will trip again. Close or leave parked; do not resume that branch.

---

## 3. Recommended construction: (a) layer count on the carrier

**One encoding:** replace the boolean with a count on the existing home.

```
type Cardinality = Required | CardOptional { layers: Nat }
```

`layers >= 1`. `Required` is zero layers. Do not keep a parallel `optional_layers` field beside `Cardinality` (that would be two homes).

**Operations (the only writers):**

| op | meaning |
| --- | --- |
| `optional_layer_count(n)` | 0 or `layers` |
| `is_optional(n)` | count > 0 |
| `wrap_optional_layer(n)` | 0 → `CardOptional { layers: 1 }`; `k` → `k+1` |
| `peel_optional_layer(n)` | `1` → `Required`; `k>1` → `k-1`; `0` → unchanged (or refuse at typed sites) |
| `join` for inhabitance | `max` of counts (OR was max on `{0,1}` only — that is the collapse) |

Delete `with_optional_cardinality` as a boolean set, or make it a synonym of `wrap_optional_layer` and migrate call sites that meant “exactly one layer from required” vs “add a layer”. Default of a former `with_optional_cardinality` call is **wrap**, not **set**. Default of former `with_required_cardinality` is **peel one**, not **clear all**, except at sites whose documented meaning is “the required payload after all wrappers” (Present binding peels one; equality admission peels until required).

`OptionalOf` instantiates as `wrap_optional_layer`. Emission wraps `Option<…>` `count` times (or recurse peel+wrap). Parse: `maybe_optional` loops: each `?` wraps one layer.

**Why not (b) nominal `Optional` for every layer, no flag.** That deletes `Cardinality` and stores optionality only as a type application. Three reasons not to do that as this C5 cut:

1. The home of inhabitance optionality in v1 is already `Node.return_cardinality`. Algebra `OptionalOf` already *compiles to that field*, not to a named type. Emission already reads that field. Moving the fact into `name == "Optional"` makes the equality-admission dual encoding the *only* encoding, and collides with a user type named `Optional` (the `|| peeled.name == "Optional"` arm exists because that collision is real).
2. v2's wrapper is a **Cardinality connective**, not a nominal Optional type. (b) would fork v1 toward a third spelling.
3. (b) is a larger replacement (324 `return_cardinality` mentions become type-tree walks). It is a legitimate later generation-cut once v1's flag is a count; it is not required to make `first()` over `List<T?>` agree with `Option<Option<T>>`.

**Replacement migration.** Root is the `Cardinality` type and the four helpers. Delete the boolean arms in one motion; fix forward every red. Do not land canonicalize-plus-flag. Do not keep `CardOptional` as “at least one” while a second structure holds the rest.

**Staged cut.** Not as dual encodings. The seed must keep compiling, which forces a **single** commit (or a tightly stacked pair: type change + mechanical `CardOptional` → `CardOptional { layers: 1 }` on every construction, then semantic wrap/peel at `OptionalOf` / `maybe_optional` / Present). That is gap-intolerant on parse of the compiler's own `Cardinality` literals, not a licence for a shadow Y. No adapter that understands both boolean and count.

**What “flatten” is allowed.**

- **Optional-receiver method/field lift** peels **exactly one** outer layer, looks up on the inner type, then wraps the result **once**. Remaining inner layers stay on that inner type. `String?.len()` is `Int?`. `String??.len()` peels to `String?`, looks up `len` (which itself peels once to `String`), result `Int` wrapped once for the inner lookup and once for the outer receiver → `Int??` if both lifts wrap — **or** the inner lookup sees `String?` and lifts once to `Int?` without a second wrap if we define lift as “wrap iff the receiver layer we peeled was the one that made the method available.” The law: each peel that was required to reach the method contributes one wrap on the result. Flattening *all* layers so `T??.len()` and `T?.len()` are both `Int?` would erase nested optionality on the chaining axis and is not the C5 repair; it is a different product rule. **Why flatten-one is not a floor-green hack:** Absent at the *outer* layer means the method does not run; the result is absent at that layer only. An inner Absent is a value of the inner optional type, still Present at the outer layer. Collapsing those is the same fork as `first()`.
- **`map_lookup_result_type`** today skips wrapping if the raw result is already optional. That is OR. After the cut it must wrap (or not) from the Map algebra row, not from “already flagged.”
- **`join` / unify** uses max, not “set flag.” Unifying `T?` with `T??` is `T??` (information-preserving); unifying `T` with `T?` is `T?`.

---

## 4. Controls (must go red if collapse returns)

1. **`first()` over `List<T?>`:** inferred type has `optional_layer_count == 2`. Outer `Present` payload is `T?`. Nested `Present` / `Absent` on that payload is admitted. **Red if** the inner Present is refused as `VariantNotFound` or the payload types as bare `T`.
2. **`first()` over `List<T>` (required element):** count 1. Nested `Present` on the payload is **red**.
3. **Authored `T??`:** parse yields count 2, equal to wrap(wrap(T)).
4. **Optional-receiver:** `String?.len()` is `Int?` (count 1), not `Int`. `String??.len()` is not collapsed to `Int?` (see §3).
5. **Emission:** `List<T?>.first()` renders `Option<Option<T>>` matching infer count 2.
6. C5 exclusion `match_arm_body_is_present_payload_binding` on gunbc#13280 **retires in the same change that makes control (1) green**, not in a later “stage 2.” There is no second encoding to wait on.

---

## 5. What this is not

- Not a v2 Node/Cardinality connective migration.
- Not new `== CardOptional` sites; after the cut that token should not exist as a unit variant.
- Not landing #13467's helpers.

Implementation starts only after gentle-dove-36 chooses (a), or explicitly overrules toward (b).
