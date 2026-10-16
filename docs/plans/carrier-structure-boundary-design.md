# Carriers and structures: the std / external-mathematics boundary

**Status:** DRAFT for operator review and iteration, 2026-10-16. Not for merge. This document
proposes design and DESIGN.md changes; roadmap and code changes come after it is agreed.
**Supersedes, when agreed:** the open N3 question "List versus FreeMonoid" in roadmap node
`v2-namespace-cutover`.

Sections 1–3 are the agreed problem and principle. Sections 4–6 are the three design areas still
open: the module structure (§4, to be decided here), inhabitance (§5, problem statement and open
questions), and where operations come from (§6, open). §9 drafts the DESIGN.md changes and §12 collects
every open question with a recommendation.

---

## 1. Problem

Ordinary data types are declared under mathematical names, and the everyday names are aliases.

| Everyday name (`std.types`) | Actual declaration (`std.algebra`) | Uses of everyday name | Uses of math name |
|---|---|---|---|
| `List<T>` | `FreeMonoid<T> = Empty \| Cons { head, tail }` | ~28,900 `List<` | ~1,560 (392 files; 353 in `src/v2/compiler`) |
| `Map<K,V>` | `FinitelySupportedFunction<K,V>` (record of 9 fn fields + `empty`) | ~1,620 | ~50 |
| `Set<T>` | `FinitePowerSet<T>` (record of 7 fn fields) | ~46 | ~60 |
| `String` (v2) | `v2.std.text`: `type String = FreeMonoid<Char>`, **and** opaque host `String` in `v2.std.node` | | |

Consequences, all measured on main (`98e4b52575`):

1. **Import cycle.** `std.types` imports `std.algebra`, so `std.algebra` cannot import `List`/`Map`/`Bool`. It
   writes `FreeMonoid`, or a bare `List`/`Map`/`Bool` that is unbound under the no-built-ins ruling
   (2026-10-15). 77 resolve refusals in `std.algebra` and native-seven's resolve wall come from this.
2. **The compiler is keyed to the math names.** List literals introduce `std.algebra.FreeMonoid`
   (`v2.std.list_introduction` `list_introduction_head_path`); map literals introduce through
   `FinitelySupportedFunction`; the seed emits `pub type List<T> = FreeMonoid<T>`.
3. **Alias-following machinery exists for these aliases.** Resolve's renaming-alias chain, infer's
   transparent-alias peel, the v1 kernel-container identity check, and three recurring failure modes
   (`carrier_alias_refuses_its_own_carriers_constructor`, `transparent_alias_relation_read_the_brand_keeping_peel`,
   `cyclic_alias_head_is_accepted`).
4. **A second naming authority.** `std.algebra` `algebra_carriers` / `CarrierSpelling` rows record each
   type's snake/pascal/alias/roster/arity spellings; `std.types` reads them (`container_type_arity`,
   `container_template_alias_rows`, …) and the v1 seed binds methods through the string-keyed
   `kernel_algebra_profile: Map<String, AlgebraProfile>`.
5. **Map and Set are objects-with-methods, not data,** and their operations are stated twice (record fields
   and `AlgebraFieldTemplate` rows). The self-host build fails on it: `E0560 struct Rc<FinitelySupportedFunction>
   has no field named lookup` (`gunbc.self_host_compile_phase_frontier`).
6. **One operation, several names:** keyed read `lookup`/`get`/`map_get`/`map_lookup`/`map_get_checked`; key
   membership `map_has`/`contains_key`/`map_contains_key`; set membership `member`/`contains`; size
   `count`/`length`/`size`/`list_length` with `Int` vs `Nat` results.
7. **Category errors in the "free monoid" profile.** `free_monoid_collection_templates` lists `last`, `get`,
   `skip`, `take`, `enumerate`, `reverse` — sequence capabilities, not consequences of being a free monoid —
   and lists `get`/`map_get`/`lookup` twice.
8. **`std.algebra` holds about eight unrelated concerns** in 1,695 lines: abstract structures, carrier
   declarations, map-literal assembly, the realization template metamodel, the spelling roster, numeric
   admissibility, `Ordering`, a general list library, and string functions (`trim`, `string_contains`).

### 1.1 Root cause

Each era added a representation and left the previous one in place: March's algebra records; the
`src/v2/std` tree beside `dag/std` (mid-June); August's Map/Set method records and template tables;
October's map-literal assembly placed in `std.algebra` to avoid the cycle. DESIGN.md has the rule
("Replacement migrations cut over at the root", §3) but two gaps let it lapse:

- **Layers are unenforced.** §3 says folders are "browsing conventions, not a direction rule"; acyclicity is
  the only enforced law, so a fact can be filed anywhere.
- **Forks are recorded rather than cut over.** The file's own comments say "held twice" and "a nicknaming
  fork being RECORDED"; only one such fork has a named dissolve trigger.

### 1.2 What the earlier ruling did and did not decide

Operator ruling 2026-09-29 made `List` and `FreeMonoid` "one type": an `Empty {}` may not be refused at a
`List<Int>` formal. That is value compatibility under the alias. It did not decide which name is the
declaration. Under this design it is restated as: **`List<T>` is the carrier; it has a free-monoid
witness; no conversion exists because there is one carrier declaration.**

---

## 2. The principle (agreed)

> **External mathematics names structures; Dag names data.** `extdeps` owns accepted mathematical
> definitions, structures and laws. `std` owns the concrete data types exposed to ordinary programs and
> their ordinary operations. A concrete type may inhabit an external mathematical structure, but that
> fact never gives it a second source-language name.

- **Externally governed meaning** — abstract algebra, order theory, classical logic, set theory, standards,
  protocols, vendor APIs → `extdeps`.
- **Dag's programming interface** — `List`, `NonEmptyList`, `Map`, `Set`, `String`, `Bool`, `Int`, `Nat`,
  their constructors and ordinary operations → `std`.
- **The relation between them** — an inhabitance fact keyed by declaration identity (§5), never an alias or
  a spelling table.

Consequence: the end state **deletes `std.algebra`**. It is not relocated; each concern goes to its own home
(§4.3).

---

## 3. Settled points

1. **One `String`** (agreed 2026-10-16). One declaration, realized as host text (DESIGN §4's text-crossing
   rule stands), with a **declared** unfold to `List<Char>`. `v2.std.text`'s `type String = FreeMonoid<Char>` is
   deleted. This also removes a silent-correctness defect: today the host `String` binds "only where no other
   declaration of the spelling is visible" (`v2.std.node`), so which `String` a file gets depends on its
   imports. `NonEmptyString` is its own declaration in the same home.
2. **Map and Set carry the mathematics that is actually true:**
   - `List<T>` is the free monoid on `T` — the witness must include the generator (singleton) and the
     universal fold, not only append.
   - `NonEmptyList<T>` is the free semigroup on `T`.
   - `Map<K,V>` is a **finite partial function** `K ⇀ V`. It is not a finitely supported total function:
     that needs a distinguished default in `V`, which a generic `Map` has not got. The sound equivalent is
     a finitely supported `K → Optional<V>` with `Absent` as the default.
   - `Set<T>` is a **finite subset** of `T` (a join-semilattice under union). It is **not** a Boolean
     algebra unless a universe and complement are modelled.
3. **`FinitelySupportedFunction` and `FinitePowerSet` are not carried forward as copied labels.** The
   Map/Set redesign records whichever external structure really holds (item 2).

---

## 4. Module structure (to decide in this document)

### 4.1 The constraint

The structures need a few base types. `OrderedRing` declares `eq`/`ne: fn(T, T) -> Bool`, the ordered
structures use `Ordering`, and some structures use `Optional`. Today `Bool` lives in `std.types` and
`Ordering` in `std.algebra`, so `std.algebra` uses `Bool` ambiently — it cannot import it, because
`std.types` imports `std.algebra`.

If the structures move to `extdeps.math` and a carrier's witness is recorded in `std.types` beside the
declaration, the cycle returns: `std.types → extdeps.math → std.types`.

Facts that make this solvable:

- `std.optional`, `std.unicode.types` and `std.error_primitives` already import nothing.
- `std.types` imports `std.algebra` **only** for the three carriers and the spelling tables
  (`kernel_algebra_profile`, `carrier_container_*_rows`, `algebra_type_param_names`), all of which this
  design deletes. Once they are gone, `std.types` imports nothing from the algebra layer.

### 4.2 Proposed layering

Three layers, with an import rule a check can enforce:

```
L0  std base carriers      Bool, Ordering, Optional, Unit, Int, Nat, Char, List/NonEmptyList (declarations only)
     imports: nothing outside L0

L1  extdeps.math.*         structures and laws; may import L0 only
     extdeps.math.abstract_algebra   Magma … Field
     extdeps.math.order_theory       TotalOrder, Lattice, BoundedLattice, BooleanAlgebra
     extdeps.math.free_structures    FreeMonoid, FreeSemigroup as STRUCTURES over a generator
     extdeps.math.set_theory         finite subset, finite partial function

L2  std carrier modules    operations and witnesses: std.list, std.map, std.set, std.text, std.nat, …
     may import L0 and L1
```

**Rule:** `extdeps.math` imports only L0. `std` modules that declare witnesses import `extdeps.math`.
Nothing in L0 imports L1 or L2. The witness for a carrier lives in its L2 module (`std.list` records
"List inhabits free monoid"), **not** beside the L0 declaration, which is what breaks the cycle.

### 4.3 Where each current `std.algebra` concern goes

| Concern | Home |
|---|---|
| `Magma` … `Field`, `CommutativeMonoid`, `AbelianGroup`, `Semiring` … | `extdeps.math.abstract_algebra` (composed, not restated — §4.5) |
| `TotalOrder`, `Lattice`, `BoundedLattice`, `BooleanAlgebra` | `extdeps.math.order_theory` |
| `Ordering` | L0 (a data type, not a structure) |
| `FreeMonoid`/`FreeSemigroup` **carriers** | become `List`/`NonEmptyList` in L0 |
| `FreeMonoid`/`FreeSemigroup` **structures** | `extdeps.math.free_structures` |
| `FinitelySupportedFunction`, `FinitePowerSet`, `PartialFunction`, `TotalMap`, `PointwisePower` | deleted; `Map`/`Set` redesign (§7) records the true structure in `extdeps.math.set_theory` |
| list operations (`fold_list`, append, reverse, sort, …) | `std.list` |
| `trim`, `string_contains`, `string_concat_join` | `std.text` |
| `MapIntroductionEntry`, `map_from_entries` | the map home (§7) |
| `AlgebraProfile`, `AlgebraFieldTemplate`, `CostShape`, template scanners | realization layer — see §5 Q5; not mathematics |
| `AlgebraCarrier`, `CarrierSpelling`, `kernel_algebra_profile`, roster/arity rows | deleted (replaced by identity-keyed inhabitance, §5) |
| numeric admissibility / support axis | open — likely `std.nat`/`std.integer` |

### 4.4 Open structural questions

- **S1. `std.types` itself.** It is 496 lines and mixes L0 carriers with workflow ids, `MimeType`, branded
  strings and more. Recommended: keep `List`, `Bool`, `Int`, `Char`, `Unit` in `std.types` for the first cut
  (3,590 files import from it), make `std.types` L0 by deleting its algebra imports, and split the rest out
  later. The alternative — moving `List` to its own module now — costs a 3,590-file import sweep up front.
- **S2. `Ordering`.** L0 in `std.types`, or its own module `std.ordering`?
- **S3. Does `extdeps.math` need `Optional`?** Only if a structure's signature returns it (e.g. a partial
  inverse). If yes, `std.optional` is already import-free and qualifies as L0.
- **S4. `src/v2/std` duplicates.** Ten module basenames exist in both `dag/std` and `src/v2/std` (logic, nat,
  integer, …). This layering applies to both; the de-fork decisions (defork audit) should be taken in the
  same terms rather than separately.
- **S5. Enforcement.** The L0/L1/L2 rule is a declared import model, checked the way
  `v2.std.cross_tree.import_model` checks the dag/v2 direction today. That replaces "folders are browsing
  conventions" for these layers (§8).

### 4.5 Structures are composed, not restated

Today `CommutativeMonoid` repeats `Monoid`, `Semiring` repeats `CommutativeSemiring`, `AbelianGroup` repeats
`Group`, and the three lattices each restate `meet`/`join`. The law that distinguishes them (commutativity,
absorption, …) has no representation, so they are nominal copies. `TotalOrder` is already composed
(`OrderedRing.order`), so the pattern exists. In `extdeps.math`, each structure is its parent plus its added
operation or law, and `AlgebraLawObligation` (`v2.std.model_core`) is the existing shape for a law.

---

## 5. Inhabitance: problem statement and open questions

### 5.1 What has to be recorded

"Carrier C inhabits structure S, with these operations as S's operations." For example: `List<T>` inhabits
the free monoid over `T` with `Empty` as unit, `list_append` as concat and `[x]` as generator; `Nat` inhabits
the ordered commutative semiring; `Bool` inhabits the Boolean algebra; `Int` inhabits the additive group
**and** the multiplicative monoid.

### 5.2 Today there are three mechanisms

| Mechanism | Where | Keyed by | Used for |
|---|---|---|---|
| `kernel_algebra_profile: Map<String, AlgebraProfile>` + `AlgebraFieldTemplate` rows | `std.algebra` | **spelling** (`"FreeMonoid"`, `"Map"`) | v1 method binding (`v1.compiler.infer_method`), cost shapes, emission |
| `AlgebraCarrier` / `CarrierSpelling` | `std.algebra` | **spelling**, per case style | containerhood, arity, canonical emitted spelling |
| `AlgebraInhabitanceDecl { algebra, inhabitant, witness }` rows + structure nodes | `v2.std.model_core`, rows in `v2.extdeps.languages.dag` `dag_algebra_inhabitance_decls`, structures in `v2.std.algebra_structure_signature` | **Node / declaration** | v2 infer: operator fields (`+`, `||`, `eq`) derive from the operand type's inhabitance row (`infer_algebra_field_result_type`) |

The v2 mechanism is the closest to this design: it is identity-keyed, it already carries a free-monoid
**structure** distinct from the `Empty|Cons` carrier (Ruling B, calm-boar-904: the generator is declared per
row), and `AlgebraLawObligation` exists beside it. The two spelling-keyed mechanisms are what this design
deletes.

### 5.3 Requirements

- R1. Keyed by the carrier's and the structure's **declaration identities**; no spelling anywhere.
- R2. One form for every carrier (List, NonEmptyList, String, Nat, Int, Bool, Map, Set, and user types).
- R3. A carrier may inhabit one structure more than once (Int: additive and multiplicative monoid), so a
  witness is named or otherwise distinguishable.
- R4. The witness names the actual operations (declaration references), so "the monoid's concat for List"
  resolves to `list_append`'s identity.
- R5. Laws are obligations attached to the witness (`AlgebraLawObligation`), checkable or explicitly
  unchecked.
- R6. Respects the layering of §4.2 (no cycle).

### 5.4 Open questions

- **I1. Home of the witness.** Beside the carrier's operations (`std.list`, per §4.2), or in the language
  model (`v2.extdeps.languages.dag`, where v2 rows live today)? Argument for the language model: "Dag's
  `Int` is a ring" is a fact about the Dag language. Argument for the carrier module: it is the one place a
  reader of `std.list` looks, and user-declared types need the same form, with no language-model access.
- **I2. Is the witness a `data` row, a declaration form, or a derived reading?** Today it is a `data` row of
  Nodes. A dedicated declaration form (`inhabits FreeMonoid<T> by { … }`) is clearer but is grammar work.
- **I3. Which structures are worth witnessing at all?** Only those whose derived operations are used
  (operators, generic folds), or all that hold?
- **I4. Does the v1 seed need this?** Under ruling 2 (v2 is the authority), v1 mirrors or is bridged; the
  spelling-keyed profile in v1 needs a named bridge with a deletion trigger until v1 is deleted.
- **I5. What happens to the realization metamodel** (`AlgebraProfile`, `AlgebraFieldTemplate`, `CostShape`)?
  It describes how a carrier is realized and what operations cost — realization facts, not mathematics.
  Likely home: the realization/emission layer, keyed by declaration identity. Needs its own decision.

---

## 6. Where operations come from (open)

### 6.1 The tension

DESIGN §4 says "operations come from *inhabitance* (no per-type ops)". The naming proposal in §6.4 —
`list_fold`, `map_get`, `set_contains` — is per-type operations. One of them has to change.

### 6.2 The operator's intent, as stated 2026-10-16

> When we have a list type in compositional modeling, treating any higher-order composition as a list
> probably has some effect on the list part — like if you roll a car type, the wheels must spin — though
> other things might change as well (from the car's perspective). But if you really want to do a list
> operation, you should probably be doing a list operation. We need to revisit how usage came about across
> the codebase; there isn't a consistent answer yet.

Read as two distinct claims:

- **(Lifting)** An operation on a composite has a defined effect on its components: rolling the car spins
  the wheels. The composite's operation is derived from, or constrained by, its parts' structure — closer to
  functorial lifting or lenses than to inheritance.
- **(Directness)** Code that means "do a list operation" should call a list operation on the list, not
  reach it through a composite's surface.

### 6.3 How usage looks today (main, `dag/` + `src/v2/`)

| Form | Occurrences |
|---|---|
| Pipe `x \|> f(...)` | ~4,930 |
| `fold_list(` free call | ~1,830 |
| `.length()` receiver | ~1,510 |
| `contains(` free call | ~1,220 |
| `list_length(` | ~830 |
| `.contains(` receiver | ~630 |
| `map_lookup(` / `map_get(` | ~350 / ~160 |
| `list_map(` | ~265 |
| `.count()` / `.size()` | ~220 / ~11 |
| `.map(` / `.filter(` / `.fold(` / `.get(` receivers | ~50 each |

How they bind:

- **Free calls and pipes** bind by import, to a declaration.
- **Receiver calls in v1** bind through the spelling-keyed `kernel_algebra_profile` templates.
- **Operators in v2** (`+`, `||`, `eq`) bind through `AlgebraInhabitanceDecl` rows: this is the one place
  "operations from inhabitance" is real today.

So §4's rule is implemented for operators (v2) and approximated for receivers (v1, by spelling). Ordinary
collection operations are per-type functions in practice.

### 6.4 Options

- **(A) Concrete types own their ordinary operations; inhabitance derives only generic ones.** §4 is
  amended: "a carrier's ordinary operations are its own declarations; inhabitance derives the operations a
  structure guarantees (operators, generic folds over any monoid)". Receiver syntax is a projection of the
  carrier's own declarations by receiver identity. Naming convention `<type>_<operation>`:

  | Current | Target |
  |---|---|
  | `fold_list`, `fold_list_right` | `list_fold`, `list_fold_right` |
  | `list_snoc_item` | `list_snoc` |
  | bare `skip`, `take_prefix`, `is_empty`, `length`, `contains`, `unique` | `list_skip`, `list_take`, `list_is_empty`, `list_length`, `list_contains`, `list_is_unique` |
  | `for_all`, `count_where`, `count_equal` | `list_all`, `list_count_where`, `list_count_equal` |
  | `freemonoid_empty()` | delete; use `Empty` |
  | `lookup` / `get` / `map_get` / `map_lookup` / `map_get_checked` | `map_get` |
  | set `member` / `contains` | `set_contains` |
  | `count` / `length` / `size` on Map, Set | `map_size`, `set_size` |

  Cardinality words: `length` for sequences and text, `size` for maps and sets, `count` for counting
  selected elements.

- **(B) Keep §4 literally.** `contains`, `size`, `fold` are generic operations derived from structures
  (e.g. a `Foldable`/`Container` structure); `List`, `Set`, `Map` inhabit it. Fewer names; needs a structure
  for "container", which is not classical algebra and so does not belong in `extdeps.math`.

- **(C) A and lifting.** Option A for ordinary operations, plus a declared lifting mechanism for composites
  (the car/wheels case): a composite declares how its operation acts on a component, by component
  identity. This is the operator's lifting claim made explicit; it is a separate design.

### 6.5 Proposed next step

Before choosing: classify a sample of real call sites (receiver calls, free calls, and composite types that
wrap a `List`) into "direct list operation", "operation the composite owns", and "operation that should lift
to a component". The split decides between A and C and whether B's generic operations are used at all.

---

## 7. Map and Set (follows the List cut; not in its PRs)

Decisions the Map/Set pass must take explicitly: opaque carrier vs. structural constructor;
duplicate-key/member impossibility; key equality requirements; extensional equality; deterministic iteration
order; literal duplicate refusal (already refused by `map_introduction_duplicate_key`); insertion/removal and
persistence semantics; cost contracts.

**Default proposal:** opaque nominal carriers with controlled constructors (`type Map<K, V>`,
`fn empty_map`, `fn map_insert`, `fn map_get -> Optional<V>`; `type Set<T>`, `fn empty_set`,
`fn set_insert`, `fn set_contains`). Opaque prevents a public `List<MapEntry>` representation from making
duplicate keys authorable, and prevents the host realization from becoming the interface. Values hold data;
operations are separate declarations.

---

## 8. Process: keep the next era from forking again

- **P1. Layer rule is checked** (§4.4 S5), and DESIGN §3's "folders are browsing conventions" is narrowed to
  folders outside declared layers.
- **P2. Second-form check.** A new declaration that duplicates an existing declaration's name in the same
  layer, or adds an alias to a carrier, is refused unless the change removes the old form or names a ruled
  exception.
- **P3. Recorded forks fire.** A comment or recurring-failure entry recording a fork ("held twice") carries
  a measurable trigger, or it is not admitted.

---

## 9. Proposed DESIGN.md changes

DESIGN.md is generated from `gunbc.design_document`; these become edits there plus a dated ruling.

1. **New paragraph, §3, "External semantic authority":**
   > `extdeps/` owns models whose meaning is governed outside gunbc, including independently versioned
   > upstreams and accepted formal mathematics. A versioned authority cites its version; a timeless formal
   > framework cites a stable source without inventing a version. `std/` owns the concrete programming
   > carriers exposed by Dag. A carrier may inhabit an external structure, but that inhabitance never renames
   > the carrier, creates an alternate source spelling, or makes ordinary code name the external structure.
2. **§3, replace** "model the accepted universal frameworks (classical logic, set theory, algebra)
   faithfully, and in `extdeps/` the real upstream spec" with a reference to (1).
3. **§3, narrow** "the `std`/`extdeps`/`compiler`/`workflow` folders are browsing conventions" so the
   declared layers of §4.2 are an enforced import rule.
4. **§3b External facts, Owns:** add "accepted formal mathematics".
5. **§4, operations clause:** pending §6 (A/B/C).
6. **§4 example:** the reference to `dag/std/algebra.dag` as the substrate exemplar moves to the new homes.
7. **§4 text paragraph (DESIGN.md line 329) is corrected.** It currently reads "`v2.std.text` `String` is
   `FreeMonoid<Char>`; the kernel `String` is host text; neither is a nickname for the other", from ruling
   `gunbc.design_ruling text_crossings_go_through_the_unfold`. Under §3.1 of this design there is one `String`
   declaration, realized as host text, whose declared unfold produces `List<Char>`; it is not itself
   `FreeMonoid<Char>`. The ruling is amended, not deleted: "every crossing goes through the declared unfold or
   refuses" still holds, with the crossing now between `String` and `List<Char>` rather than between two
   `String`s.
8. **§3b Cardinality / absence, existing authorities (DESIGN.md line 285):** `std.algebra::FreeMonoid` is
   replaced by the `List` declaration.
9. **Ruling record** in `gunbc.design_ruling`: this design as the N3 answer for `List`, `NonEmptyList`,
   `String`, and the direction for `Map`/`Set`.

---

## 10. Interlocks with work in flight

- **#14017** (generated import sweep, the namespace lane's one generator — "never a second", N4). It adds
  `std.algebra` collection-shape imports and sweeps `map_get` → `map_lookup`, deepening the boundary this
  design removes. Proposal: hold and retarget it to this design's naming table, so the List rewrite runs
  **through** it. Needs the namespace lane (wise-moth-707) to agree.
- **#14056** (`node_type_equals` by declaration identity). Keep the identity work; replace its permanent
  `List<Int> == FreeMonoid<Int>` control with a neutral generic-alias control.
- **`std.coercion` / `std.syntax` `List` imports** (tidy-ant-251, in flight) are compatible: they import
  `std.types { List }`, which remains the home in the first cut (S1).
- **Roadmap `core-type-merges`** (stale since 2026-09-30) records List as merged (#12526); the merge landed as
  the alias this design removes. String's merge stalled (#12809 closed).

---

## 11. Migration order (after agreement)

1. DESIGN amendment and ruling (§9).
2. `extdeps.math` created; structures moved and composed; consumers swept; no re-export from `std.algebra`.
3. **List root cut:** `List` declared in `std.types` (L0); `Empty`/`Cons` move with it; list literals
   introduce `List`; `FreeMonoid<T>` type positions rewritten; Rust emits `List<T>`; List-specific alias peels
   and container exceptions deleted (generic alias support stays); free-monoid witness recorded by identity.
4. List operations move to `std.list` with the §6 naming (if A/C), via the one generator.
5. `NonEmptyList` and `String` cuts.
6. Map, then Set.
7. `std.algebra` deleted, with its remaining tables.

Steps 2 and 3 are independent. Step 3 is the one that unblocks native-seven, so it may go first once step 1
lands.

---

## 12. Open questions, collected

| # | Question | Recommendation |
|---|---|---|
| S1 | `List` stays in `std.types` for the first cut? | Yes; split `std.types` later |
| S2 | `Ordering` home | `std.types` (L0) |
| S3 | `extdeps.math` may import `std.optional`? | Yes, if a signature needs it |
| S4 | De-fork `src/v2/std` duplicates in the same terms | Yes |
| S5 | Layer rule enforced via the import model | Yes |
| I1 | Witness home: carrier module or language model | Carrier module (user types need the same form) |
| I2 | Witness: `data` row vs declaration form | `data` row first; form later if warranted |
| I3 | Which structures to witness | Those whose derived operations are used |
| I4 | v1 bridge for spelling-keyed profiles | Named bridge with deletion trigger |
| I5 | Realization metamodel home | Realization layer, identity-keyed; own decision |
| O1 | §4 operations: A, B, or C | Decide after the §6.5 usage classification |
| M1 | Map/Set opaque by default | Yes |
| P1–P3 | Process checks | Adopt all three |
