# T3 — Set/Map carrier realization fork, re-measured on current main (2026-08-22)

**Session:** `bright-newt-108`. **Work item:** `node://adhoc-ce1e99b8-e49`.
**Assignment (smart-ram-730):** re-derive the T3 population on current main; do **not** inherit the
number 21. This document is a measurement. It proposes no repair and lands no implementation.

## Subject, ref, producer

| | |
|---|---|
| subject | `src/v2/compiler/03_ingest.dag` closure, emitted → cssl-assembled → `cargo build --release --lib` |
| ref | `origin/main@753c7d1def0c82a1ae3da81f7e6386b3e028e8a7` |
| producer | `docs/probes/curated_cargo_probe_one.sh`, `PRODUCER_PATH=curated_cargo_probe_one+emit+seedlink+cargo`, `EMIT_COUNT_SRC=gunbc_compiled_line` |
| contract | `CSSL_STD_SEED_LINK=1`, `shim_lib_rel=""` (no lane shim for this entry) |
| same-base pin | `PROBE_EXPECT_BASE_SHA=753c7d1def…` armed; the run did not refuse, so tree == pin |
| emitted | 177 files, 503 diagnostics |
| M | 1. Every figure here is "at M=1, 03_ingest" and may not be summed with another module's board |

Board this run reproduces, so the E0308 share is readable beside its denominator:

```
E0308:128 E0425:24 E0277:23 E0599:22 E0004:21 E0609:18 E0061:17 E0560:17
E0631:9   E0433:8  E0369:7  E0282:6  E0614:6  E0071:3  E0728:2  E0310:2
E0223:1   E0533:1
CARGO_ERROR_TOTAL=329   HISTOGRAM_SUM=330
```

E0308 blocks = **128**; distinct `(file, line, col, expected, found)` sites = **123**.

## Column 1 — current T3 population

**T3 on current main is 14 sites**, not 21 and not the 25 the 2026-08-21 partition reported under
that label. It did **not** go to zero, so the lane is not released.

The three numbers are three different instruments and must not be differenced casually:

| figure | what it counted |
|---:|---|
| 25 | the `T3` label in [`e0308_partition_2026-08-21.md`](e0308_partition_2026-08-21.md), which also absorbed `Vector`/arity pairs (`Vector<()>` vs `Vector<Rc<ComplexityLowering>>`, `StateDurabilityBoundary` vs `Vector<Rc<…>>`) that are not a Set/Map carrier fork at all |
| 23 | that same TSV re-counted with **this** document's definition — a pair naming `PointwisePower` or `PartialFunction` on either side — at `2a2bd0ad59` |
| **14** | the same definition at `753c7d1def0`, measured here |

The instrument-consistent movement is therefore **23 → 14 at the same subject and definition**,
across `2a2bd0ad59…753c7d1def0`. No attribution is offered for the nine that left (see Controls).
`21` is not reproduced by any of the three readings and is not carried forward.

## Column 2 — semantic identity of the 14 sites

Joined at `.dag` module + declared symbol, not `file|line|col`. The correspondence is exactly
1:1 — 14 emitted sites, 14 authored record literals, no site without a literal and no literal in
this closure without a site.

| `.dag` module | symbol | literal | emitted site |
|---|---|---|---|
| `v2.compiler.02_parse` | `parse_table_empty` | `Set { member: fn(_) { false } }` | `v2_compiler_parse.rs` |
| `v2.compiler.02_parse` | `set_symbol_insert` | `Set { member: … }` | `v2_compiler_parse.rs` |
| `v2.compiler.02_parse` | `compute_nullable_set` | `Set { member: fn(_) { false } }` | `v2_compiler_parse.rs` |
| `v2.compiler.03_resolve` | `empty_canonical_symbol_set` | `Set { member: … }` | `v2_compiler_resolve.rs` |
| `v2.std.language_model` | `language_model_empty_canonical_symbols` | `Set { member: … }` | `v2_std_language_model.rs` |
| `v2.compiler.04_infer` | `infer_emit_shape_frontier_inferred_tree` | `Map { lookup: … }` | `v2_compiler_infer.rs` |
| `v2.compiler.04_infer` | `facts_map_from_entries` | `Map { lookup: … }` | `v2_compiler_infer.rs` |
| `v2.compiler.05_eval` | `empty_runtime_bindings` | `Map { lookup: … }` | `v2_compiler_eval.rs` |
| `v2.extdeps.languages.llvm_ir` | `llvm_integer_spec_facts` | `Map { lookup: … }` | `v2_extdeps_languages_llvm_ir.rs` |
| `v2.extdeps.languages.llvm_ir` | `llvm_float_spec_facts` | `Map { lookup: … }` | `v2_extdeps_languages_llvm_ir.rs` |
| `v2.extdeps.runtimes.v2_effect_io_pure` | `effect_io_pure_empty_environment` | `Map { lookup: … }` | `v2_extdeps_runtimes_v2_effect_io_pure.rs` |
| `v2.extdeps.runtimes.v2_evaluator` | `v2_eval_empty_bindings` | `Map { lookup: … }` | `v2_extdeps_runtimes_v2_evaluator.rs` |
| `v2.std.collection` | `map_insert` | `Map { lookup: … }` | `v2_std_collection.rs` |
| `v2.std.model_core` | `model_core_bool_spec_facts` | `Map { lookup: … }` | `v2_std_model_core.rs` |

5 Set, 9 Map. Twelve of the fourteen construct a **constant** collection (an empty set, an empty
binding environment, a fixed fact table); the other two — `set_symbol_insert` and
`v2.std.collection` `map_insert` — are functional updates that chain a new closure over the old
one.

## Column 3 — the exact Set / Map declarations involved

- `dag/std/types.dag` — `type Set<element> = PointwisePower<element>` and the `Map` alias, plus
  `container_template_alias_rows` (`Set`/`set` → `PointwisePower`, `Map`/`map` → `PartialFunction`)
  and `container_template_algebra_rows` (the same mapping, keyed for the algebra lookup).
- `dag/std/algebra.dag` — `type PointwisePower<T> { member: fn(T) -> Bool }` and
  `type PartialFunction<K, V> { lookup: fn(K) -> V?, insert: …, merge: …, size: … }`.
- `dag/extdeps/languages/rust/types.dag` — the Rust inhabitant rows: `PointwisePower` →
  `BTreeSet<{0}>` (identity `BTreeSet::new()`), `PartialFunction` → `HashMap<{0}, {1}>`
  (identity `HashMap::new()`). `BTreeSet` is the seed's `im::OrdSet` alias, which is why rustc
  prints `OrdSet<String>` on the expected side.

## Column 4 — the TYPE-position producer

`v1.compiler.emit_rust` `render_rust_type` / `render_rust_decl_type` classify the node with
`v1.compiler.types` `node_is_keyed_collection` / `node_is_element_collection` — both of which
require `is_declared_container_alias_spelling`, i.e. the authored spelling is one of
`container_template_alias_rows` — and then render through `v1.compiler.emit` `emit_map_type` /
`emit_container` → `v1.compiler.coercion` `coerce_container_template` → the
`extdeps.languages.rust.types` inhabitant row. So `Set<Symbol>` in any declaration position
becomes `OrdSet<Symbol>` and `Map<K, V>` becomes `HashMap<K, V>`.

## Column 5 — the VALUE-position producer

The record-literal path in `v1.compiler.emit_rust` (`record_lit_resolved_ctor_import_names`,
`rust_struct_field_lookup_candidates`) consults `std.types` `container_template_algebra` on the
**same** spelling, resolves `Set` → `PointwisePower` / `Map` → `PartialFunction`, and constructs
**that struct**, shared-wrapped: `Rc<PointwisePower<_>>`, `Rc<PartialFunction<_, _>>`.

**The two positions consult one authority row and then take different hops off it.** The type
position continues to the target inhabitant template; the value position stops at the modeled
algebra struct. This is one spelling with two lowerings, not two independent tables that happen
to disagree — which is what makes it a §3 single-authority defect rather than a missing
conversion.

A third position exists and sides with the value position: the alias RHS. `render_rust_alias_rhs_type`
routes host container bases through `rust_seed_host_container_base`, which handles only
`List`/`FreeMonoid` → `Vec`; `Set`/`Map` fall through to the struct path, which is why the seed
carries `pub type Set<Element> = Rc<crate::std_algebra::PointwisePower<Element>>` in
`std_types.rs` while every *use* of `Set<T>` renders `OrdSet<T>`.

## Column 6 — expected / found at each site

Four distinct pair signatures cover all 14:

| expected | found | sites |
|---|---|---:|
| `OrdSet<String>` | `Rc<PointwisePower<_>>` | 5 |
| `HashMap<String, Rc<Node>>` | `Rc<PartialFunction<_, _>>` | 3 |
| `HashMap<Rc<EnvironmentBindingKey>, …>` | `Rc<PartialFunction<_, _>>` | 3 |
| `HashMap<Rc<Node>, Rc<InferredFacts>>` | `Rc<PartialFunction<_, _>>` | 2 |
| `HashMap<K, V>` | `Rc<PartialFunction<_, _>>` | 1 |

The expected side is always the declared return/field type (type position, target realization);
the found side is always the record literal (value position, modeled realization). **No site has
a conversion in either direction** — there is no `From`/`Into`, no adapter, nothing that rustc
proposes. The conversion cannot exist as a matter of semantics, not of missing code: a
`PointwisePower` is a characteristic *function* and an `OrdSet` is a finite enumeration, and
nothing can turn an arbitrary predicate into a finite set.

## Column 7 — reference-layer contribution, kept separate

**1 site**, and it is not part of the 14:

```
src/v2_std_compilers_target_model.rs:2647:62   expected HashMap<_, _>, found Rc<HashMap<_, _>>
```

Both sides name the same carrier; only the `Rc` depth differs. At `2a2bd0ad59` this file carried
six pairs in which the carrier fork and the `Rc` depth were entangled
(`Rc<Rc<PartialFunction<String, …>>>` against `Rc<HashMap<…>>`); on current main only the
reference-layer half survives there. Repairing the realization fork would not have closed this
site, and closing this site would not have touched the 14.

## The class beneath the sites, stated at model grain

The corpus-wide fork is larger than the 14 rustc sites, because a fork only becomes an error
where both positions meet inside one compiled closure. Counted over `src/v2` + `dag`:

| surface | count |
|---|---:|
| `Set<…>` type positions | 100 |
| `Map<…>` type positions | 668 |
| `Set { … }` record literals | 38 |
| `Map { … }` record literals | 145 |
| finite-set builtin calls (`set_contains`, `set_insert`, `empty_set`) | 34 |
| predicate calls `.member(…)` | 28 |
| finite-map builtin calls (`map_insert`, `map_get`, `empty_map`, `map_keys`, `map_values`, `map_lookup`, `map_contains_key`) | 1070 |
| partial-function calls `.lookup(…)` | 35 |

One declared type is being used as two concepts: a **finite collection** (the builtin surface,
realized `OrdSet`/`HashMap`) and a **characteristic function / partial function** (the record
surface, realized `PointwisePower`/`PartialFunction`). `dag/std/types.dag`'s
`type Set<element> = PointwisePower<element>` is where the two are made one name.

`v2.std.collection` shows both inside a single module: `empty_map` delegates to the primitive
(realized `v1_rt::empty_map()` → `HashMap`), while `map_insert` two lines later constructs
`Map { lookup: … }` (realized `PartialFunction`). That is the whole class in eight lines.

## Controls, and what this run does not establish

- **Ref.** `git rev-parse HEAD` echoed from inside the remote dispatch = `753c7d1def0…`, and the
  probe's own column 9 carries the same sha independently. `PROBE_EXPECT_BASE_SHA` was armed.
- **Not a zero from an instrument that can fail toward zero.** The run produced 123 E0308 sites
  against 128 blocks and an 18-code histogram summing to 330 against a `CARGO_ERROR_TOTAL` of 329;
  the T3 count of 14 sits inside a populated board, not beside an empty one.
- **Row count against expectation.** 14 emitted sites were expected to correspond to 14 authored
  literals and do. The two instruments are independent (rustc spans vs a `.dag` grep), so the
  agreement is a check, not a restatement.
- **No attribution for 23 → 14.** The nine sites that left are 3 in `v2_extdeps_languages_dag.rs`
  and 6 in `v2_std_compilers_target_model.rs`. `src/v2/extdeps/languages/dag.dag` still carries
  its three `Set { … }` literals today, so those three sites left the *board* without the
  authored construction changing — masking by an earlier refusal, or a closure change, are both
  live explanations and neither is established here. Do not read the delta as nine repairs.
- **M = 1.** These are 03_ingest's closure. The corpus-grain table above is a separate instrument
  (a grep over authored source) and is not a superset claim about any other module's board.
- **No repair is proposed.** The producers in columns 4 and 5 are named from the emitter source
  and from pair evidence; neither was traced by executing the emitter on a single site in this
  run.

Per-site TSV for the full 123-site E0308 board this partition was derived from:
[`t3_collection_realization_2026-08-22/e0308_sites_753c7d1def0.tsv`](t3_collection_realization_2026-08-22/e0308_sites_753c7d1def0.tsv)
(columns: file, line, col, expected, found — the discriminating pair is carried raw, not folded
into a root label, so a later lane can re-partition it without re-running the probe).

To repeat: same probe, same contract, same single entry, at the ref above; deduplicate to distinct
`(file, line, col, expected, found)`; T3 is the subset whose pair names `PointwisePower` or
`PartialFunction` on either side.
