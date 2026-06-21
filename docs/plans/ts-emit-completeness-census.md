# TypeScript emit-target completeness gap census (Lane C)

> **Status:** ANALYSIS ONLY (no emit-machinery edits, no PR). Authority: `docs/plans/v2-self-hosting.md` Track T.
> **Baseline:** TS is proven on exactly one end-to-end slice — the `add` slice, Python→core→TS
> (`cross_language_add_python_to_typescript_test.dag`), via `neutralize_core_for_target` rewriting
> Python `int` → TS `number` over `std_projection_additive_numeric`. Everything past `add` is the gap.
> **Reference:** the shared value/type projection contract in `src/v2/std/compilers/target_model.dag`,
> NOT the Rust target — see §0.

---

## 0. Framing correction: Rust is the type-level reference, NOT the value-level one

The brief says "compare TS rows to Rust (the completeness reference)." That holds for **type emit** but
**inverts for value emit**:

- **Type-expression projection** (`TargetTypeExpressionProjection`, 6 forms: atom / conj / disj / arrow /
  cardinality / instantiation): Rust wires all 6 (`rust_type_expression_projection`,
  `rust.dag:3199`). TS also wires all 6 (`ts_type_expression_projection`, `typescript.dag:1618`). **Parity.**
- **Value-expression projection** (`TargetValueExpressionProjection`, 11 forms): **Rust does not wire it
  at all** — `rust_mvp1_target_model_core_edges` (`rust.dag:1474`) has no
  `target_model_edge_value_expression_projection` edge, and Rust's `add` body is a **baked fixture node**
  (`rust_mvp1_fixture_emitted_add_fn`, `rust.dag:993`), not a compositional projection. **TS is the
  pioneer here** — it is the only target that wires `target_model_edge_value_expression_projection`
  (`typescript.dag:1473` → `ts_value_expression_projection`, line 557).

**Consequence for the census:** the completeness target is the **contract** in
`src/v2/std/compilers/target_model.dag` (the 11 value-projection forms + 18 `TargetValueExpressionKind`
variants), not Rust's row set. TS is simultaneously *ahead* of Rust (compositional value path exists) and
*incomplete* (most forms are fail-closed sentinels). The gap list below is the work to take TS's
already-wired projection from `add`-only to corpus-complete.

---

## 1. What TS currently HAS (wired, real tokens, test-backed)

| Construct | Form / row | Token authority | Proven by |
|---|---|---|---|
| Numeric primitive bundles | number / bigint / boolean / string fact-bundles inhabiting ApproximateField / OrderedRing / BooleanAlgebra / FreeMonoid | `ts_facts_*` | wave-2b inhabitance + laws |
| Function decl (typed, fixed shape) | `mvp1 fn add`, `pr3 typed identity` grammar relation rows | full token rows | `mvp1_typescript_add_translate_test`, `mvp1_typescript_pr3_typed_fn_translate` |
| Type alias + record TYPE | `wave2a type alias decl`, record-type, union-type (string-literal) | `ts_wave2a_*` productions | `typescript_wave2a.dag` (3 parse claims) |
| Binding ref (value) | `binding_ref_form` → `^ts_token_ident` | real | `comprep_value_expression_fold_typescript_test` |
| Primitive apply (value) | `primitive_apply_form` → `( , )` | real | `comprep_value_expression_fold_typescript_test` |
| Callable apply (value) | `callable_apply_form` → `callee( , )` | real | cross-language tests |
| Effect apply (value) | `effect_apply_form` → `^ReadResource`/`^WriteResource` host-shim callees | real, descriptor-driven | `typescript_effect_io_emit_test` |
| Infix `+` operator | `ts_add_operator_realization_row` → `InfixToken ^ts_token_plus` | real | `comprep_add_body_emit_typescript_test` |
| Type-expr projection (6/6) | atom / conj / disj / arrow / cardinality / instantiation | real | SG-2 projection tests |

**Net:** TS can emit a *typed top-level function whose body is a tree of primitive/callable/effect
applications over identifier and literal leaves with `+`*. That is exactly the `add` slice plus its
near neighborhood. It cannot yet emit any control flow, binding, closure, match, record/variant
*value*, or field access.

---

## 2. What TS has as a STUB (sentinel present, fail-closed — decodes but emits LOUD on use)

These are the riskiest rows: the projection bundle structurally decodes, so all *other* TS emit is
unaffected, but routing one of these constructs through TS today fails closed at serialize (sentinel
token-class has no lex spelling) rather than emitting plausible-but-wrong TS. Each carries a 🟡
dissolve-on marker. Listed with the real TS lowering each needs.

| Form | Sentinel | `typescript.dag` | Real TS lowering required (dissolve-on) |
|---|---|---|---|
| `closure_form` (7 fields) | `^ts_closure_emit_unsupported` | 571–592 | fat-arrow `(p, …) => expr` — expression body, no `fn`/brace. (`feature:ts-closure-fat-arrow-projection`, Lane 2c) |
| `conditional_form` | `if_token`/`then_token`/`else_token` (TS has no `then`) | 615–625 | ternary `cond ? T : E` (§12.9(B) optional-token refinement) |
| `let_form` | `in_token: ^ts_token_unwired_bind_in` | 626–634 | TS let-in lowering (`let x = …; …` sequence; `in_token` intentionally unlexed) |
| `loop_form` | `loop_token: ^ts_token_unwired_loop` | 635–637 | TS loop lowering (`for`/`while` or recursion) |
| `field_access_form` | `dot: ^ts_token_unwired_dot` | 645–647 | `obj.field` — needs a real `.` token wired |
| `match_form` | `match_token: ^ts_token_unwired_match` (open/close/`=>`/sep are real) | 648–654 | match→`switch` on `_variant` discriminant, or if-else chain over tagged union |
| `record_construct_form` | tokens REAL but **no TS test** | 639–644 | author a TS record-construct emit test (tokens already `{ k: v, … }`) |

---

## 3. What TS is MISSING entirely (no row at all)

- **Coproduct VALUE construction** — tagged-variant constructor in value position (e.g. `Present{value:…}`
  → `{ _variant: "Present", value: … }`). Type-level disj is wired (`disj_form`, line 1631) but there is
  no value-construct row for variants.
- **Pattern match deconstruction** — bound to `match_form` stub above; even once `match_token` lands, arm
  *payload binding* (destructuring the variant) has no carrier.
- **Generic type-parameter DECLARATION** — `<T>` on a `fn`/`type` definition. Only *instantiation*
  (`Foo<T>` use-site) is wired (`instantiation_form`). Definition-site type params: absent.
- **`Optional<T>` surface** — no TS optional spelling (`T | undefined` exists in std metadata but no
  emit row / no `Present`/`Absent` value construction).
- **`Map<K,V>` / `Set<T>` value surfaces** — type-level cardinality form exists; value construction
  (`empty_map`, `map_insert`, `set_insert`) realizations: absent.
- **`List<T>` value construction** — array TYPE via `pr3_array_type` (`T[]`) exists; list-literal /
  push / concat value realizations: absent.
- **Operator catalog beyond `+`** — only `op_add` (`+`) is realized; `-` exists only as a comprep test
  override. Missing: `* / %`, comparisons (`== != < > <= >=`), logical (`&& || !`), `div`.
- **Fold / catamorphism lowering** — no direct row; folds decompose into closure + match + recursion,
  so they are blocked transitively on those three.

---

## 4. Ranked gap list (each = one row/slice to author later)

Ranked by **corpus load-bearing weight** — how many of the 732 `src/v2/*.dag` files use the construct
(the compiler must emit *itself*, so corpus frequency = priority). Frequencies measured by grep over
`src/v2 --include='*.dag'`.

| # | Gap (TS row to author) | Corpus reach | Current TS state | Blocks |
|---|---|---|---|---|
| **1** | **Field access** `obj.field` | **~11.5k occ** (record-oriented corpus) | STUB (`^ts_token_unwired_dot`) | every record consumer |
| **2** | **Record VALUE construction** `{k:v,…}` | **561 files (76%), 12.1k occ** | tokens real, **no test** | every product-type value |
| **3** | **Closure / fat-arrow** `(p)=>e` | **410 files (56%), 7.8k occ** | STUB (`^ts_closure_emit_unsupported`) | fold/map/filter, every `|>` lambda |
| **4** | **Pattern match** → `switch`/if-chain + arm payload binding | **412 files (56%), 3.4k occ** | STUB (`^ts_token_unwired_match`) + payload carrier absent | all variant control flow |
| **5** | **Generic type-param DECL** `<T>` on fn/type | **423 files (58%), 4.1k occ** | absent (only instantiation) | every generic def (`fold_node`, `Witness<T>`, …) |
| **6** | **Coproduct VALUE construct** (tagged `{_variant,…}`) | sum decls 63 files; constructed pervasively | absent | #4 match, Optional, all sum values |
| **7** | **Conditional → ternary** `?:` | **312 files (43%), 1.46k `if`** | STUB (`then` keyword invalid) | every `if/else` expr |
| **8** | **let-binding** `let x = …;` sequencing | **223 files (30%), 1.16k occ** | STUB (`^ts_token_unwired_bind_in`) | every `let` body |
| **9** | **Operator catalog** (`- * / %`, cmp, logical) | comparisons/logical pervasive in guards | only `+` (`-` test-only) | every non-add arithmetic/guard |
| **10** | **`Present`/`Absent` + `Optional<T>`** value surface | 276 files (38%) Present/Absent | absent | Optional-returning fns |
| **11** | **`List<T>` value ops** (literal/push/concat) | 281 files (38%) | type only (`T[]`) | list-building fns |
| **12** | **Loop lowering** (`for`/recursion) | folds 142 files; explicit loop rarer | STUB (`^ts_token_unwired_loop`) | imperative loops |
| **13** | **`Map<K,V>` / `Set<T>` value realizations** | Map 55 files (8%), Set 23 (3%) | type-level only | map/set-using fns |

**Dependency note:** #3 (closure), #4 (match), #6 (variant construct) form the **critical cluster** —
folds/catamorphisms (`fold_node`, used by all 7 v2 stages) decompose into exactly these three, so the
single highest-leverage emit milestone after field-access/record-construct is *closure + match + variant
construction together*. #5 (generic decl) is orthogonal but equally pervasive and independently
authorable.

---

## 5. Suggested slicing for the implementer (deferred until Lane A is cargo-green)

1. **Slice T-A (records):** dissolve `field_access_form` stub (#1) + author the record-construct TS test
   (#2). Unblocks all product-type emit; both are nearly-wired (tokens exist).
2. **Slice T-B (sums + match):** variant value construction (#6) + `match_form` → `switch`/if-chain with
   arm payload carrier (#4). The hardest single slice; needs an arm-payload carrier in the shared
   `target_model`.
3. **Slice T-C (closures + control):** fat-arrow `closure_form` (#3), ternary `conditional_form` (#7),
   `let_form` sequencing (#8). Unblocks folds/HOFs once T-B lands.
4. **Slice T-D (generics):** definition-site `<T>` type params (#5) — orthogonal, parallelizable.
5. **Slice T-E (collections + operators):** operator catalog (#9), Optional/List/Map/Set value
   surfaces (#10–#13).

Each form's `target_model` carrier is shared, so several of these (notably the arm-payload carrier for
#4 and the labeled-param carrier flagged in TS's `arrow_form`) require a **std `target_model` carrier
edit first** — model-before-implement (DESIGN §6). Those carrier gaps are the true blockers, not the
per-language rows.

---

## 6. Receipts (file:line)

- TS value projection + every stub marker: `src/v2/extdeps/languages/typescript.dag:557–656`, edge
  wired at `:1473`.
- TS type-expr projection (6/6 wired): `typescript.dag:1618`.
- Rust has no value projection; `add` is a baked fixture: `rust.dag:1474` (core edges), `rust.dag:993`.
- Shared contract a complete target must fill: `src/v2/std/compilers/target_model.dag` —
  `TargetValueExpressionKind` (18 variants, `:579`), `TargetValueExpressionProjection` (11 forms, `:755`),
  `TargetTypeExpressionProjection` (6 forms, `:7299`).
- Shared emit fold dispatch universe: `src/v2/compiler/06_translate.dag:1757–1922` (type nodes),
  `:10760–10798` (value behaviors: Transform/Branch/Bind/Loop/Match).
- Proven TS baseline: `src/v2/compiler/manual/cross_language_add_python_to_typescript_test.dag`.
