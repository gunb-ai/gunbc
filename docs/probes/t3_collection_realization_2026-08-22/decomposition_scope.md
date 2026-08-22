# T3 decomposition — scope (2026-08-22)

Scoping only, per the ruling "decompose, do not choose": `Set`/`Map` name the **finite collection**;
the predicate / partial-function algebra keeps its own authority under its own name. No repair is
implemented here and no PR carries one. Measurement this rests on:
[`../t3_collection_realization_2026-08-22.md`](../t3_collection_realization_2026-08-22.md).

## 1 — the 14 literals, and the population behind them

**All 14 in-closure literals are finite constructions.** None of them needs the predicate shape:

| shape | count | sites |
|---|---:|---|
| empty collection | 8 | `parse_table_empty`, `compute_nullable_set`, `empty_canonical_symbol_set`, `language_model_empty_canonical_symbols`, `infer_emit_shape_frontier_inferred_tree`, `empty_runtime_bindings`, `v2_eval_empty_bindings`, `effect_io_pure_empty_environment` |
| finite table over a closed key set | 4 | `facts_map_from_entries`, `llvm_integer_spec_facts`, `llvm_float_spec_facts`, `model_core_bool_spec_facts` |
| functional update | 2 | `set_symbol_insert`, `v2.std.collection` `map_insert` |

**But the edit is field-driven, not literal-driven, and that is the finding that changes the size
of the work.** Five of the 14 feed `LanguageModel.canonical_symbols`, and that same field is fed
**outside this closure by genuine open predicates** — `rust_canonical_symbols`,
`ts_canonical_symbols`, `dag_canonical_symbols` and their siblings delegate to
`*_catalog_surface_symbol`, `grammar_carries_symbol`, `*_lex_token_symbol`. Re-authoring the five
empty ones as finite sets without re-typing the field would put a finite value and a predicate
value on one declaration — the fork moved one layer down, not removed.

Corpus-wide population, so the denominator is visible (`src/v2` + `dag`):

| | count | note |
|---|---:|---|
| `Set { … }` literals | 37 | a 38th grep hit is `operation Set` in `extdeps.tools.hostname` — a service operation, not a record literal |
| — empty | 6 | |
| — finite enumeration over literal symbols | 11 | `v2.lens.subsumption` ×2, grammar/parse test fixtures |
| — functional update | 1 | `set_symbol_insert` |
| — **genuine open predicate** | **19** | every `*_canonical_symbols` in 13 language modules, plus `dag_canonical_symbols`, `ts_canonical_symbols`, `rust_hash_set_eq_hash_supplemental_generic_bound_requirements` |
| `Map { … }` literals | 145 | |
| — empty | 8 | |
| — inline finite table | 8 | |
| — axis → node table (`*_spec_facts`) | 32 | keys are a closed fact-axis coproduct |
| — `lookup: <named fn>` delegate | 97 | sampled: the delegates are `if key == a { … } else if key == b { … } else { Absent }` if-chains over a fixed key list — e.g. `v2.program` `program_facts_lookup`, six keys |
| — **genuine open predicate** | **0 on the evidence taken** | |

Read individually: all 14 in-closure literals and all 37 `Set` literals. **Sampled, not exhaustive:**
the 97 `Map` delegate bodies — I read a sample and classified the rest by the shape of the
delegate; that row is the one number here that a full read could move.

So the two edits and their sizes: **~19 declarations move to the predicate algebra's own name**
(concentrated in one field across 13 language modules), and **~163 literals are finite
constructions currently mis-spelled** (18 Set + 145 Map).

## 2 — the alias RHS position

**Answered by reading the row set** — `dag/extdeps/languages/rust/types.dag` (all six inhabitant
rows), `dag/std/algebra.dag` `kernel_algebra_profile_value` and the profile template functions,
and `dag/std/types.dag` `container_template_algebra_rows` / `container_template_alias_rows`.

`rust_seed_host_container_base` maps only `List`/`FreeMonoid` → `"Vec"`. **Set/Map should not join
that list**, because joining it would hard-code a host base name for a carrier the model already
has:

- **The finite set algebra already exists.** `BooleanAlgebraCollectionProfile` carries
  `union`, `intersect`, `diff`, `member`, `contains`, `filter`, `map`, `flat_map`, `fold`, `any`,
  `all`, `count`, `length` — a finite collection surface — while `PointwisePowerCollectionProfile`
  carries exactly one row, `member`. And it already has its own Rust inhabitant row: `BooleanAlgebra`
  → `BTreeSet<{0}>`, identity `BTreeSet::new()`, **byte-identical to the `PointwisePower` row**.
- So the decomposition on the Set side is a **retarget of the alias, not a new host row**:
  `std.types` `type Set<element> = PointwisePower<element>` and `kernel_algebra_profile`'s
  `"Set" → PointwisePowerCollectionProfile` both point at the finite carrier instead. The alias RHS
  then renders the finite carrier and `pub type Set<Element> = Rc<PointwisePower<Element>>`
  **dissolves without `rust_seed_host_container_base` being touched at all**. The type-position
  lowering does not move, because both rows lower to the same template.
- **Map needs no retarget.** `PartialFunction` *is* the map profile, and its template roster is
  already the finite map surface — `map_insert`, `map_keys`, `map_values`, `map_contains_key`,
  `map_get`, `length`, `count` — beside `lookup`. What is missing is the construction route:
  `std.primitives` declares `map_insert_contract` and `v2.std.collection` binds
  `map_insert_host_binding`, but `map_insert` is written as a closure chain while `empty_map`
  delegates through `empty_map_primitive_delegate`. **The primitive route is declared and unused.**
  Map's fix is that one missing delegate, plus the 145 re-authorings.

## 3 — blast radius on the algebra side

If `PointwisePower` stops being reachable as `Set<T>`, what still names it (`.dag`, whole corpus):

| | count |
|---|---:|
| files naming `PointwisePower` | 14 |
| — target inhabitant row files (`rust`, `go`, `python`, `typescript`) | 4 |
| — v1 seed emitter / coercion (`05_emit_rust`, `coercion`) | 2 |
| — std authorities (`std.types` alias, `std.algebra` decl + profile, `std.computation` profile arm) | 3 |
| — v2 re-export (`v2.std.algebra`) | 1 |
| — prose only, no program dependency (`v2.std.grammar`, `v2.compiler.02_parse`, `std.state_durability`, `gunbc.doc_graph_roots`) | 4 |
| `.member(` call sites | 28 |
| declarations that would move to the `PointwisePower` spelling | ~19 (§1) |
| files naming `PartialFunction` | 12 — rows, emitter, and two witness tests; **no corpus consumer names it directly** |

## 4 — the reference-layer site

**Untouched, and it would still stand after a complete T3 repair.** The one site is
`v2_std_compilers_target_model.rs:2647` `expected HashMap<_, _>, found Rc<HashMap<_, _>>` — the
same carrier on both sides, differing only in `Rc` depth. Nothing in §1–§3 changes a sharing-wrap
decision; that is the R1 axis (`rust_shared_wrap_ctor`), and it is measured, reported and left
alone here.

## Three prior-art constraints the ruling should see before anything is built

None of these refutes the ruling. Each is an authored decision premised on today's grounding that
the ruling would change, and each must be updated *by* the change rather than silently contradicted.

1. **`v2.std.grammar` moved a field the other way, with a measured receipt.** `GrammarRoot.sync_tokens`
   was re-typed `Set<Symbol>` → `List<Symbol>` precisely because "a characteristic function cannot
   be enumerated, so the whole grammar value became unhashable through this one field" — the
   `prepare_grammar` memo could never form its key, costing a full ~12s grammar preparation on
   every parse, for every language. Under the ruling that motivation dissolves (a finite set
   hashes), so the note becomes stale in the direction of *understating* what is now possible.
2. **`std.state_durability` names the grounding explicitly** — "Set is the non-enumerable
   PointwisePower while durability qualification must enumerate every boundary" — and refuses "a
   Map-as-set or bitset nickname" as the workaround. That refusal stays correct; its premise moves.
3. **`v2.compiler.02_parse`'s nullable fixpoint exists because sets cannot be compared.**
   `nullable_member_count` plus the `fuel` bound is a monotone-measure convergence test standing in
   for `next == previous`, which the comment states outright. A finite `Set` makes the direct test
   available and that machinery dissolvable — listed as a consequence the ruling buys, not as a
   claim that it should be spent in the same change.
