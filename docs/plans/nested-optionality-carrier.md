# Nested optionality on the v1 type carrier

**Status:** design + census only. Side-chat NO-LAND on 8246299; this revision answers the four corrections. Still no compiler code.
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

**Parse of `?`.** `maybe_optional`: 3 call sites after type expressions. One `ExpectQuestion`, one flag. Authored `T??` is tokenized as `ShNullCoalesce`, so it is not a second question token (see §3.3).

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

## 3. Recommended construction: (a) structurally positive layer tower

**Confirmed direction:** count carrier on `Cardinality`; delete the boolean/nominal dual; keep #13467 closed; C5 Present-payload exclusion retires only when the nested control is green by execution.

### 3.1 Zero is unconstructible

```
type OptionalLayers
  = OneLayer
  | MoreLayers { inner: OptionalLayers }

type Cardinality
  = Required
  | CardOptional { layers: OptionalLayers }
```

Zero optional layers has only the `Required` constructor. One layer is `OneLayer`. Two is `MoreLayers { inner: OneLayer }`. There is no `CardOptional` without an `OptionalLayers`, and `OptionalLayers` has no zero arm.

**Judgement of `CardOptional { pred: Nat }`:** it is isomorphic (`pred: Zero` ≅ `OneLayer`, `Succ` ≅ `MoreLayers`), so it does not inhabit count-zero on the optional arm. It is still the wrong spelling. A `Nat` field is the *predecessor of the count*, which is a prose reading of a representable `Zero`. Callers can write `pred` as if it were the layer count (`pred: Zero` as “no layers”). `OptionalLayers` is the same successor tower with constructors named as layers, not as a natural that happens to start at one. `std` has no constructor-confined `PositiveNat`; `v2.std.refinement` `PositiveInt` is `Outcome` validation. Do not use `CardOptional { layers: Nat }`.

**(ii) still rejected:** a single `Nat` with `0 = Required` makes `Required` a nickname for `Zero` and makes zero layers a `Nat` you can construct on the optional path.

Do not keep a parallel count field beside `Cardinality`.

### 3.2 Named operations (no generic join = max)

Delete `join_optional_cardinality` and `preserve_outer_optional_cardinality` as boolean OR/max. Replace with four named operations:

| op | relation | law |
| --- | --- | --- |
| `compose_optional_layers` (addition) | wrapper composition / generic substitution | layers of the outer wrapper plus layers of the substitute. `T?` with `T := U?` is `U??`. `preserve_outer` becomes this sum, not max. `OptionalOf` is compose with `OneLayer`. |
| `peel_optional_layer` | one eliminator step | `OneLayer` → `Required`; `MoreLayers { inner }` → `CardOptional { layers: inner }`. `Required` refuses at typed sites. Present-binding and `.value` are this op and **do not rewrap**. |
| `exact_optional_layers` | joins and conformance | `T`, `T?`, `T??` are distinct. Mismatch **refuses** unless an explicit conversion plan is selected and executed at the value (wrap `Present` *n* times, or peel). No undeclared `T → T?` or `T? → T??` (C5’s rule at every adjacent depth). Match arms that differ in layer count refuse, or the arm that needs wrapping must carry the plan; max does not mint missing `Present`s. |
| `reconcile_optional_layers` | two observations of one type | authored identity vs structural resolution must **agree**, or a named authority wins. Max would hide disagreement. |

A future LUB is not this cut. If one is added later it must return the conversion plan with the type, and interpreter and emitter must apply that plan.

`with_optional_cardinality` becomes compose-with-`OneLayer`. Former `with_required_cardinality` becomes peel-one except at sites that walk to the required base (equality’s *after* the depth check).

### 3.3 Parse (tokenizer)

The lexer emits adjacent `??` as **one** `ShNullCoalesce` before `ShQuestion`. A loop that only `eat`s `ExpectQuestion` never sees authored `T??`.

**Syntax (minimal contextual rule):** in a **type suffix**, `maybe_optional` loops: `ShQuestion` composes one layer, `ShNullCoalesce` composes two, so `T?` / `T??` / `T???` are 1 / 2 / 3. Expression `a ?? b` is unchanged null-coalescing (`ShNullCoalesce` in expr position). No new authored spelling.

### 3.4 Receiver lift (one recursive rule)

Lookup peels **one** layer only when the member is not on the current type. Each such peel **adds one** layer to the result, composed with the member’s own result layers.

- `String?.len()` → `Int?` (one peel, `len: Int`)
- `String??.len()` → `Int??` (two peels)
- `String??.parse_int()` → `Int???` (`parse_int: Int?` plus two peels)
- field `U?` through receiver `S??` → `U???`

Runtime must step through each `Present` and **preserve which layer was Absent** (outer None vs `Some(None)`), not flatten to one Option. Explicit eliminators peel without rewrapping.

`map_lookup_result_type` must not skip wrapping because the raw type is already optional; wrapping follows the algebra row via compose.

### 3.5 Equality

Compare **layer towers first**. `T? == T??` refuses (depth mismatch) rather than peeling both to `T` and succeeding. Then peel to the required base for the rest of equality admission.

Delete `name == "Optional"` as a layer peel. Keep a **non-kernel type named `Optional`** control: it is not a cardinality layer (the deleted arm must not return).

### 3.6 Why not (b)

Unchanged: v1 home is `return_cardinality`; algebra `OptionalOf` compiles to that field; v2 uses a Cardinality connective; a user type named `Optional` is not inhabitance optionality.

### 3.7 Replacement migration

Root: boolean `Cardinality` and the four OR/max helpers. Delete-first in one implementation PR after go. Mechanical `return_cardinality: CardOptional` → `CardOptional { layers: OneLayer }`. Regen stage0 in a fresh standalone clone; floor green. No dual-encoding adapter. No code until the next go on this revision.

---

## 4. Controls (must go red if collapse returns)

1. **`first()` over `List<T?>`:** two layers. Outer `Present` payload is `T?`. Nested `Present` / `Absent` admitted. **Red if** collapsed to bare `T`.
2. **`first()` over `List<T>`:** one layer. Nested `Present` on the payload is **red**.
3. **Parse type suffix:** `T?` is `OneLayer`; `T??` is two (`ShNullCoalesce`); `T???` is three. **Expression** `a ?? b` still null-coalesces (unchanged).
4. **Receiver:** `String?.len()` is `Int?`; `String??.len()` is `Int??`. **Optional-returning member:** `String??.parse_int()` is `Int???`. **Outer vs inner absence:** runtime distinguishes Absent at the outer layer from Absent at an inner layer (not a single flattened None).
5. **Equality depth:** `T? == T??` **refuses**. **Non-kernel type named `Optional`:** not treated as a cardinality layer (positive control that the deleted name-peel does not return).
6. **Substitution:** `T?` with `T := U?` is `U??` (compose), **red** if collapsed to `U?`.
7. **Conformance:** using `T` where `T?` (or `T?` where `T??`) is declared **refuses** without an explicit conversion plan.
8. **Emission:** `List<T?>.first()` renders `Option<Option<T>>` matching two layers.
9. C5 exclusion `match_arm_body_is_present_payload_binding` on gunbc#13280 **retires in the same change that makes control (1) green**.

---

## 5. What this is not

- Not a v2 Node/Cardinality connective migration.
- Not new `== CardOptional` sites; after the cut that token should not exist as a unit variant.
- Not landing #13467's helpers.

Implementation starts only after gentle-dove-36 relays go on this revision.
