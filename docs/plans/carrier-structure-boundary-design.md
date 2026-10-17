# Carriers and structures: the std / external-mathematics boundary

**Status:** DRAFT r2 for operator review and iteration, 2026-10-16. Not for merge. Roadmap and code changes
come after it is agreed. Before merge, the agreed authority moves out of this markdown into the `.dag` design
carrier (`gunbc.design_document`) and a dated ruling; this file is the drafting surface.
**Answers, when agreed:** the open N3 question "List versus FreeMonoid" in roadmap node `v2-namespace-cutover`.

**r2 changes** (side-chat review of r1): the design is organised around **distinct typed relations** instead
of one overloaded "inhabitance"; the linear L0/L1/L2 layering is replaced by a **per-carrier diamond** (a convention);
`std.types` is migration-only, not a target home; the existing v2 `AlgebraInhabitanceDecl` is prior art, not
the canonical form; Map/Set **denotation** is separated from algebraic inhabitance; the operations question has
a recommended rule; the second-form check is strengthened from name detection to typed authority uniqueness.

**Decisions still needed from the operator** are D1–D3 at the top of §14.

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

## 2. The principle

> **External concepts name interfaces and structures; Dag names carriers.** `extdeps` owns externally
> grounded abstract-data-type interfaces (thin: characteristic operations and laws over an unknown carrier),
> mathematical structures, laws, standards and upstream specifications. `std` owns Dag's concrete carrier
> declarations and ordinary operations. The realization layer owns host/target storage. A std carrier may
> implement an external interface, inhabit a structure, denote a mathematical object, convert to another
> carrier and realize as a host representation. These are distinct relations, and none gives the carrier a
> second source-level name.

**What each level means (operator, 2026-10-17):**

- **`extdeps` is acknowledgement, not design.** It is our most faithful, honest transcription of an outside
  concept: "this is literally that thing, 1:1, as far as we need to know about it." It records what the
  external source says (a standard, a textbook definition, an upstream spec). It need not be executable; you
  cannot run a Wikipedia page. It carries none of our own choices, so it does not grow when our needs grow.
- **`std` is our interpretation.** "We respect the outside thing, but it does not do what we need, so we fleshed
  it out and gave our own take on it." `std` may add operations, pick representations-in-meaning (e.g. String
  observed as Unicode scalar values), and choose names. It must not contradict the external concept it
  claims; where it deliberately departs, the departure is stated on the relation, not hidden.
- **`Implements` is the claim between them:** "our take honours that external concept." The binding names
  which std declarations realize which acknowledged operations, and records any stated departure. What `std`
  adds beyond the acknowledged surface is ordinary std, owing nothing to `extdeps`.

**Mostly convention, grounded on real sources (operator, 2026-10-17).** Most of this design is a naming and
placement convention, not new machinery. Its anchor is that every `extdeps` declaration cites the real source
it transcribes (standard, spec, reference text), so "is this faithful?" can be checked against something outside
the repo. A declaration with no source to cite is not an acknowledgement and belongs in `std`. Mechanical gates
(§8) are kept to what convention alone cannot hold; they are not the point of the design.

Consequence for disposition (§5.4): any `extdeps` declaration that encodes *our* design choices rather than the
external source is misplaced and moves to `std`. Any `std` declaration that merely restates an external
definition with no interpretation added is misplaced and moves to `extdeps`.

Three levels, not two:

| Level | Owns | List example |
|---|---|---|
| `extdeps.adt` | abstract interface and laws | finite ordered sequence: `empty`, `cons`, `head`, `tail` |
| `std` | Dag carrier and semantic implementation | `std.list.List<T> = Empty \| Cons`, `list_fold`, `list_skip` |
| realization | host storage | Rust `im::Vector` / `Vec` |

An `extdeps.adt` interface is **not a carrier**. There is no `extdeps.adt.list.List<T>` type; ordinary code
never names the interface as a type. It stays thin: `sort`, indexed access, slicing and performance guarantees
are not essential List semantics just because `std.list` offers them.

The end state **deletes `std.algebra`**. It is not relocated: each concern goes to its own home (§5.3).

---

## 3. The authority model: one question, one relation

The r1 draft used "inhabitance" for several materially different facts. Each is its own relation:

| Question | Authority or relation | Example |
|---|---|---|
| What values can exist? | **Carrier declaration** | `List<T> = Empty \| Cons { head, tail }` |
| What computation exists? | **Operation declaration** | `list_append`, `map_get` |
| What behaviour defines an abstract data concept? | **External interface** (`extdeps.adt`) | `extdeps.adt.list`, `extdeps.adt.map` |
| What operations and laws define a structure? | **External structure** (`extdeps.math`) | `Monoid`, `FreeMonoid`, `Ring` |
| How does a carrier satisfy a structure? | **`Inhabits`** | `List<T>` inhabits the free monoid on `T` |
| How does a concrete carrier satisfy an external interface (an ADT contract, not a mathematical structure)? | **`Implements`** | `std.list.List` implements `extdeps.adt.list`, binding `empty`/`cons`/`head`/`tail` to `Empty`/`Cons`/`list_head`/`list_tail` |
| What mathematical object does a value represent? | **`Denotes`** | `Map<K,V>` denotes a finite partial function `K ⇀ V` |
| How do two distinct carriers cross? | **`ConvertsTo`** | `String ↔ List<Char>` |
| How is a carrier stored or emitted? | **`RealizesAs`** | `String` realized as host text |
| How does a whole's operation act on a part? | **Component action** | `Car.roll` acts through `wheel_rotate` |
| What view is computed from an authority? | **`ProjectsFrom`** | v2's structure `TypeNode`s, v1's profile tables (while they live) |
| What did a migration replace? | **`Replaces`** | atomic; the replaced declaration is absent at the resulting head |

Two live declarations may coexist only through one of these relations. A plain second carrier, a copied
signature, an untyped re-export, or a carrier alias has no relation and refuses (§8).

---

## 4. Settled points

1. **One `String`** (agreed 2026-10-16):
   - `std.text.String` is the one carrier, `RealizesAs` host text. It moves out of `v2.std.node`: the Node model
     references the language's `String`, it does not own it.
   - `List<Char>` is a **distinct** carrier, not another representation of `String`.
   - `string_chars: String -> List<Char>` and `string_from_chars: List<Char> -> String` are the `ConvertsTo`
     pair. `string_from_chars` is total only if every `Char` is structurally a Unicode scalar. `Char` is
     `Int where unicode_scalar`; until that construction wall is shown complete, the reverse conversion
     carries a typed refusal (or a proof obligation). Once both directions are total and inverse, the pair is
     recorded as an isomorphism.
   - `String` and `List<Char>` may each inhabit the free monoid on `Char` with their own operations; the
     conversion preserves that structure.
   - This **amends** DESIGN §4's text paragraph and ruling `text_crossings_go_through_the_unfold`, which
     sanction two `String` representations (§9 item 7). The rule that every crossing goes through the declared
     conversion stays. `v2.std.text`'s `type String = FreeMonoid<Char>` is deleted. This also removes a
     silent-correctness defect: today the host `String` binds "only where no other declaration of the spelling
     is visible", so which `String` a file gets depends on its imports.
2. **The mathematics recorded is the mathematics that holds.** Historical labels are not copied forward:
   - `List<T>` **inhabits** the free monoid on `T` (§6.3 states what that instance must bind).
   - `NonEmptyList<T>` inhabits the free semigroup on `T`.
   - `Map<K,V>` **denotes** a finite partial function `K ⇀ V`. It is not a finitely supported *total* function,
     which needs a default in `V`; the sound equivalent is finitely supported `K → Optional<V>`.
   - `Set<T>` **denotes** a finite subset of `T`, and may separately **inhabit** a join-semilattice under
     union. It is not a Boolean algebra unless a universe and complement are modelled.

---

## 5. Module structure

### 5.1 Why the r1 layering was wrong

r1 proposed L0 base types / L1 `extdeps.math` / L2 `std` carrier modules, with `std.types` kept as L0
because 3,590 files import it. That is a migration argument, not a design argument, and it would canonize a
mixed 496-line module (carriers beside workflow ids, `MimeType`, branded strings). The real shape is a
**diamond per carrier**, not a global stack.

### 5.2 The diamond

```
   extdeps.math.<area>                 std.<carrier>
   structures + laws                   carrier declaration + ordinary operations
   (imports only the few base          (imports no structure)
    carriers needed to spell
    signatures: Bool, Ordering)
              \                           /
               \                         /
                v                       v
              std.<carrier>.inhabitance        std.<carrier>.denotation
              the join: binds structure         Denotes facts
              operations to carrier operations
```

- A **carrier module** never imports the structures it inhabits.
- A **structure module** may import only the base carriers its signatures mention (`Bool`, `Ordering`,
  perhaps `Optional`), never a carrier's operations or witnesses.
- The **inhabitance / denotation module** is the intentional join, and the only place both meet.

Each carrier can take part in several independent joins, each its own module:

| Carrier | `Implements` (`.conformance`) | `Inhabits` (`.inhabitance`) | `Denotes` (`.denotation`) | Other |
|---|---|---|---|---|
| `std.list.List` | `extdeps.adt.list` | `extdeps.math.free` (free monoid on `T`) | none | `RealizesAs` target sequence |
| `std.text.String` | `extdeps.adt.string`: observed as a finite sequence of Unicode scalar values, storage hidden | free monoid on `Char` | none | `ConvertsTo` `List<Char>`; `RealizesAs` host text |
| `std.map.Map` | `extdeps.adt.map` | only structures it really carries | finite partial function | none |
| `std.set.Set` | `extdeps.adt.set` | join semilattice | finite subset | none |

This breaks the current cycle (`std.types → std.algebra → Bool in std.types`) without a global direction rule
across all of `std` and `extdeps`.

Module names of the form `std.list` plus `std.list.inhabitance` are legal under the name-to-path rule: a module
file and a same-named directory already coexist (`dag/std/filesystem.dag` + `dag/std/filesystem/`).

### 5.3 Target homes

| Concept | Target module(s) |
|---|---|
| `List`, `NonEmptyList`, `Empty`, `Cons`, list operations | `std.list`; `std.list.inhabitance` |
| Map-backed list algorithms now in `std.list` (duplicate detection, …) | `std.list.keyed` (so `std.list` does not import `Map`) |
| `String`, `NonEmptyString`, text operations (`trim`, `string_contains`, `string_concat_join` from `std.algebra`) | `std.text`; `std.text.inhabitance`; `std.text.scalar_sequence` (the conversion) |
| `Map`, map operations, map-literal assembly (`MapIntroductionEntry`, `map_from_entries`) | `std.map`; `std.map.denotation`; `std.map.inhabitance` only for structures Map really carries |
| `Set`, set operations | `std.set`; `std.set.denotation`; `std.set.inhabitance` |
| `Ordering` | `std.ordering` |
| `Optional` | `std.optional` (exists, imports nothing) |
| `Nat` | `std.nat`; `std.nat.inhabitance` |
| `Magma` … `Field`, groups, rings | `extdeps.math.algebra` |
| `TotalOrder`, `Lattice`, `BoundedLattice`, `BooleanAlgebra`, `JoinSemilattice` | `extdeps.math.order` |
| `FreeMonoid`, `FreeSemigroup` as structures | `extdeps.math.free` |
| `FinitePartialFunction`, `FiniteSubset` | `extdeps.math.set` |
| `AlgebraCarrier`, `CarrierSpelling`, `kernel_algebra_profile`, roster/arity rows | deleted |
| `AlgebraProfile`, `AlgebraFieldTemplate`, `CostShape` | realization layer, identity-keyed (open, §12 I5) |
| `std.types` | **migration source only**; emptied as each carrier moves, then deleted or reduced to what genuinely belongs together |

### 5.4 Disposition of the other mathematical declarations

r1 deleted `PartialFunction`, `TotalMap`, `PointwisePower`, `FinitePowerSet` and `FinitelySupportedFunction`
outright. Instead, one rule per declaration:

- a consumed, standard mathematical concept → its `extdeps.math` module;
- only an alternate name for an ordinary Dag carrier → delete that use;
- unconsumed scaffold → delete;
- a real open-function or predicate carrier used by ordinary code → give it an ordinary programming name in
  `std` and record its `Denotes` separately.

### 5.5 Structures are composed, not restated

`CommutativeMonoid` repeats `Monoid`, `Semiring` repeats `CommutativeSemiring`, `AbelianGroup` repeats `Group`,
the three lattices restate `meet`/`join`, and the distinguishing laws have no representation. In
`extdeps.math`, each structure is its parent plus its added operation or law, with laws as declared
obligations. `TotalOrder` (already composed into `OrderedRing`) is the existing pattern.

---

## 6. Inhabitance

### 6.1 Today: three mechanisms, none canonical

| Mechanism | Where | Keyed by |
|---|---|---|
| `kernel_algebra_profile: Map<String, AlgebraProfile>` + `AlgebraFieldTemplate` rows | `std.algebra`; read by v1 method binding | spelling |
| `AlgebraCarrier` / `CarrierSpelling` | `std.algebra`; read by `std.types` and the seed | spelling |
| `AlgebraInhabitanceDecl { algebra: Node, inhabitant: Node, witness: Node }` | `v2.std.model_core`; rows in `v2.extdeps.languages.dag` `dag_algebra_inhabitance_decls`; structures hand-built as `TypeNode` trees in `v2.std.algebra_structure_signature` | synthetic Nodes |

The v2 form is the closest **prior art**, but it is not the canonical form. Its carrier and structure are
synthetic Nodes, not references to source declarations. Its structures are a second, hand-built
representation of the `std.algebra` records, which `gunbc.recurring_failure_mode`
`algebra_structure_has_two_representations_with_no_declared_mapping` already records as a fork. Moving one side
to `extdeps.math` would rename the fork, not remove it.

### 6.2 The canonical form: source-level and identity-keyed

```
type StructureInhabitance {
  carrier: AppliedTypeRef               // e.g. List<T>, generic over T
  structure: AppliedStructureRef        // e.g. FreeMonoid over T
  instance_identity: DeclarationRef     // the instance itself is a declaration
  operation_bindings: List<StructureOperationBinding>
  law_standings: List<StructureLawStanding>
}
type AppliedTypeRef { declaration: DeclarationRef, type_arguments: List<TypeArgumentBinding> }
type StructureOperationBinding { structure_operation: DeclarationRef, carrier_operation: DeclarationRef }
```

Requirements:

- **R1 Identity.** Carrier, structure and every bound operation are declaration references; no spelling.
- **R2 One form** for every carrier, including user types.
- **R3 Generic schemas.** One instance states the fact for `List<T>`, not one per element type.
- **R4 Laws** carry evidence or an explicit unproved standing.
- **R5 Composition is derived.** A ring instance yields its additive group and multiplicative monoid; they are
  not stated again.
- **R6 Coherence.** Implicit instance selection only when exactly one candidate applies. Where a carrier has
  several instances of one structure (Int's additive and multiplicative monoids), a generic algorithm receives
  the witness explicitly; "the monoid on Int" is ambiguous and refuses.
- **R7 Projections.** v2's synthetic structure Nodes and v1's spelling profiles become `ProjectsFrom` views
  generated from this form while they live, never independently authored.

### 6.3 What a faithful free-monoid instance binds

Free monoid on `T`: a monoid `M`, an injection `η: T → M`, and for every monoid `N` and `f: T → N` a unique
monoid homomorphism extending `f`. The current v2 signature carries the generator as a Node but not the
injection or the universal property. The `List<T>` instance binds:

```
carrier    List<T>
identity   Empty
combine    list_append
generator  list_singleton
extension  list_fold_map
laws       associativity, left_identity, right_identity, generator_extension, universal_uniqueness
```

The first implementation may carry some laws as obligations, but it names the actual property, not
"appendable sequence".

---

## 7. Where operations come from

### 7.1 The tension

DESIGN §4 says "operations come from *inhabitance* (no per-type ops)". Read literally, it is why sequence
operations (`skip`, `take`, `get`, `sort_by`, `enumerate`) sit under a "free monoid" profile today, though
none follows from being a free monoid. Meanwhile real usage is per-type functions (`fold_list(` ~1,830,
`list_length(` ~830, pipes ~4,930). Operations-from-inhabitance is real today only for v2 operators (`+`,
`||`, `eq` through `AlgebraInhabitanceDecl`).

### 7.2 The operator's intent (2026-10-16)

> When we have a list type in compositional modeling, treating any higher-order composition as a list
> probably has some effect on the list part — like if you roll a car type, the wheels must spin — though
> other things might change as well. But if you really want to do a list operation, you should probably be
> doing a list operation. I don't think I have a very consistent answer on it.

### 7.3 The rule (adopted, D1, 2026-10-16)

> A carrier's constructors and eliminators come from its declaration. Operations guaranteed by an external
> structure come from a declaration-identity inhabitance witness. Concrete algorithms over one carrier are
> ordinary declarations in that carrier's module. A composite acquires no operation merely because a component
> has it; a whole-to-part action must be declared.

Consequences:

- `list_append` is an ordinary List operation; the free-monoid instance binds its `combine` to that same
  declaration. Code meaning "append lists" calls `list_append`; a generic monoid algorithm calls the witness's
  `combine`.
- Receiver syntax (`xs.fold(...)`), where kept, resolves to the same declaration by receiver identity — not a
  method table.
- **The car and its wheels:** containment alone lifts nothing, because there is no unique lifting — wheels turn
  under rolling, braking, slipping or a dynamometer. `Car.roll` may be **declared** in terms of `wheel_rotate`
  (component action). That honours both halves of the operator's intent: the part's behaviour is explicit, and
  a list operation is a list operation. A component-action model is designed when a real composite consumer
  needs it.

**Operator ruling on D1 (2026-10-16):** adopted. The aim is *sustainable abstractions that keep opacity
safely*: rolling a car affects every component, and that effect is described at different layers (mechanism of
action), but each part sees only the narrow interface it needs. The wheels do not know about the other
components; nearly all of their behaviour is "roll forwards or backwards", so they need only to be told which way.
A component action is therefore a declared, narrow message from the whole to the part, not the part seeing the
whole and not the whole inheriting the part's operations.

The r1 call-site classification is no longer a prerequisite for the rule. It stays useful to validate the rule
against real code and to drive the rewrite (§12 O1).

### 7.4 Naming

The operator asked that naming follow one consistent convention (D1 ruling). The convention below is part of
the adopted rule and is checked mechanically (a lint over declaration names), not left to review.

`<carrier>_<operation>` for ordinary operations; structure fields keep mathematical vocabulary
(`Monoid.combine`, `Ring.add`, `FreeMonoid.generator`), and the witness maps them to carrier declarations.

| Current | Target |
|---|---|
| `fold_list`, `fold_list_right`, `list_snoc_item` | `list_fold`, `list_fold_right`, `list_snoc` |
| bare `skip`, `take_prefix`, `is_empty`, `length`, `contains`, `unique` | `list_skip`, `list_take`, `list_is_empty`, `list_length`, `list_contains`, `list_is_unique` |
| `for_all`, `count_where`, `count_equal` | `list_all`, `list_count_where`, `list_count_equal` |
| `freemonoid_empty()` | deleted; use `Empty` |
| `lookup` / `get` / `map_get` / `map_lookup` | `map_get` |
| `map_has` / `contains_key` / `map_contains_key` | `map_contains_key` |
| set `member` / `contains` | `set_contains` |
| `count` / `length` / `size` on Map, Set | `map_size`, `set_size` |
| text extent | `string_length` |

Cardinality words: `length` for sequences and text, `size` for maps and sets, `count` for counting selected
elements. A wrapper collapses into the operation it wraps only when its extra outcome is proven uninhabited
(e.g. `map_get_checked`, which appears to wrap a total `map_get` in an `Accepted` that cannot reject).

---

## 8. Enforcement

Most of this design is convention grounded on real sources (§2). Checks exist only for what convention alone
cannot hold. Module placement and the §5.2 diamond are conventions, reviewed like any other; the only hard
dependency law stays DESIGN's existing acyclicity rule. (r2's role-by-role dependency matrix is dropped,
operator 2026-10-17. Imports are retiring anyway, so any future dependency check would be stated over resolved
declaration references, not `import` statements.)

### 8.1 Source citation

Every `extdeps` declaration cites the real source it transcribes (standard, spec, reference text, upstream
version). A declaration with nothing to cite is not an acknowledgement and is refused in `extdeps`; it belongs
in `std`. This is the check that keeps `extdeps` honest.

### 8.2 One live authority per concept

The name-based "second form" check would miss every fork in this area, because each pair has **different**
names (`List`/`FreeMonoid`, `Map`/`FinitelySupportedFunction`, host/structural `String`). The rule instead:

> Two live declarations may not independently claim one concept. A declaration derived from another names its
> §3 relation to it.

Checked over the **whole** source population, not only what required CI compiles. Concretely it refuses: a plain
carrier alias (`type List<T> = FreeMonoid<T>`); a compatibility re-export left after a migration; two std
carriers for one concept; a std carrier claiming an external concept without an `Implements` edge. An
interface and its implementation linked by `Implements` are not duplicates.

Arbitrary semantic equivalence is not mechanically discoverable. What is enforceable is that every authored
shared-concept claim carries a relation, and uniqueness over those claims. The existing `no_dual_representation`
lens has partial coverage and is not this check.

### 8.3 Recorded forks fire

A comment or recurring-failure entry recording a fork ("held twice", "kept in sync") carries a measurable
trigger, or it is not admitted.

---

## 9. Map and Set

Default: **opaque nominal carriers with controlled operations** (`type Map<K, V>`, `empty_map`, `map_insert`,
`map_get -> Optional<V>`; `type Set<T>`, `empty_set`, `set_insert`, `set_contains`). Opaque keeps a public
`List<MapEntry>` from making duplicate keys authorable and keeps the host realization from becoming the
interface.

Decisions the Map/Set pass takes explicitly:

- **Key equality.** Either `Map<K,V>` requires an equality witness on `K`, or the carrier exists for every `K`
  and `map_get`/`map_insert` require it. The second composes better unless the language has constrained type
  constructors. Hashing or ordering is a realization choice; source semantics need equality.
- **Iteration.** An unordered map cannot honestly answer `map_keys: Map<K,V> -> List<K>` without an order. The
  semantic answer is `Set<K>`; a list projection needs an ordering witness or a declared canonical order.
  `map_values` is semantically a bag of values.
- **Merge.** Not one operation until key conflicts have a disposition: left wins, right wins, combine, or
  refuse on overlap. Each contract gets its own name or takes a conflict function.
- Also: extensional equality; literal duplicate refusal (exists: `map_introduction_duplicate_key`);
  persistence and removal semantics; cost contracts.

---

## 10. Proposed DESIGN.md changes

DESIGN.md is generated from `gunbc.design_document`; these become edits there plus a dated ruling.

1. **§3, new paragraph "External semantic authority":**
   > `extdeps/` owns models whose meaning is governed outside gunbc, including independently versioned
   > upstreams and accepted formal mathematics. A versioned authority cites its version; a timeless formal
   > framework cites a stable source without inventing a version. `std/` owns the concrete programming
   > carriers exposed by Dag. A carrier may inhabit an external structure, but that inhabitance never renames
   > the carrier, creates an alternate source spelling, or makes ordinary code name the external structure.
2. **§3, replace** "model the accepted universal frameworks (classical logic, set theory, algebra)
   faithfully, and in `extdeps/` the real upstream spec" with a reference to (1).
3. **§3, add the relation taxonomy** (§3 of this doc) and the authority-uniqueness rule (§8.2).
4. **§3, add** that `extdeps` declarations cite their external source (§8.1); folders otherwise stay
   browsing conventions.
5. **§3b External facts, Owns:** add "accepted formal mathematics".
6. **§4, replace** "operations come from inhabitance (no per-type ops)" with the rule in §7.3 (pending
   operator decision).
7. **§4 text paragraph (line 329)** is corrected: one `String` carrier realized as host text, a distinct
   `List<Char>`, and a declared conversion between them. Ruling `text_crossings_go_through_the_unfold` is
   amended, not deleted.
8. **§3b Cardinality / absence existing authorities (line 285):** `std.algebra::FreeMonoid` → `std.list::List`.
9. **§4 example** pointing at `dag/std/algebra.dag` moves to the new homes.
10. **Ruling** in `gunbc.design_ruling`: the N3 text in §11.

---

## 11. Proposed N3 ruling text

> **N3 — External interfaces, concrete carriers and mathematical structures.** Externally grounded abstract
> data concepts (List, String, Map, Set) have thin interface-and-law authorities under `extdeps.adt`.
> Accepted mathematical structures such as FreeMonoid have their own authorities under `extdeps.math`.
> `std.list.List`, `std.list.NonEmptyList`, `std.text.String`, `std.map.Map` and `std.set.Set` are the sole
> source-level Dag carrier declarations for those concepts. A carrier implements an interface only through a
> declaration-identity implementation instance binding interface operations to existing carrier operations.
> It inhabits a structure only through a separate declaration-identity inhabitance instance carrying the
> operation bindings and law standing. Implementation, inhabitance, denotation, conversion, target
> realization and whole-to-component action are distinct typed relations. Ordinary code names std carriers
> and their operations, not extdeps interfaces or structures. Containment alone grants no lifted operation.
> `String` is one host-text-realized carrier; `List<Char>` is a distinct scalar-sequence carrier, and every
> crossing uses its declared conversion. No plain carrier alias, compatibility re-export, spelling table or
> independently authored projection may create a second live form.

---

## 12. Interlocks with work in flight

- **#14017** (generated import sweep; the namespace lane's one generator, "never a second", N4). It adds
  `std.algebra` collection-shape imports and sweeps `map_get` → `map_lookup`, deepening the boundary this design
  removes. Proposal: freeze and retarget it to this design, so the List rewrite runs **through** it. Needs the
  namespace lane (wise-moth-707).
- **#14056** (`node_type_equals` by declaration identity). Keep the identity work; replace its permanent
  `List<Int> == FreeMonoid<Int>` control with a neutral generic-alias control.
- **`std.coercion` / `std.syntax` `List` imports** (tidy-ant-251, in flight) import `std.types { List }`; the
  List cut's generated sweep rewrites them with everything else.
- **Roadmap `core-type-merges`** (stale since 2026-09-30) records List as merged (#12526); that merge landed as
  the alias this design removes. String's merge stalled (#12809 closed).
- **The v1 seed** keys container identity to a written carrier module. Under ruling 2 it follows the v2 design
  through a named bridge with a deletion trigger, and never constrains it.

---

## 13. Migration order

1. Freeze and retarget #14017.
2. Land the N3 ruling, the DESIGN.md amendments, and the relation declarations.
3. Establish the canonical structure and inhabitance forms (§6.2); make v2's synthetic signatures and the v1
   bridge projections of them.
4. Move to `extdeps.math` only the structures the first cut needs, deleting their old source authority in the
   same change. No third representation.
5. **List cut:** `List` declared in its final home; list literals introduce it; type and import rewrite
   through the one generator; Rust emits `List<T>`; List-specific alias peels, container exceptions and
   spelling rows deleted (generic alias support stays); the free-monoid instance recorded.
6. List operations move and are renamed (§7.4).
7. `String` canonicalized, then its scalar-sequence conversions.
8. `Map`, then `Set`: denotation, key equality, iteration, conflict semantics.
9. Remaining mathematical declarations dispositioned (§5.4).
10. `std.algebra` deleted.

---

## 14. Open questions

**Operator decisions:**

| # | Question | Recommendation |
|---|---|---|
| D1 | **Adopted 2026-10-16.** Adopt the operations rule in §7.3 (ordinary per-carrier operations; structure operations via witness; no automatic lifting; declared component action)? | Yes |
| D2 | **Adopted 2026-10-16 (straight to `std.list`).** Does the List cut go straight to its final home `std.list`, or through `std.types` first? | Straight to `std.list`. Costs: the Map-backed algorithms leave `std.list` first (`std.list.keyed`), and the generated sweep rewrites ~3,590 importers once instead of twice |
| D3 | **Adopted in principle 2026-10-16; DESIGN amendment text needs careful separate review before landing.** Adopt the relation taxonomy (§3) and the checks (§8: source citation, one live authority, forks fire) as DESIGN amendments? | Yes |

**Design questions to settle in this doc:**

| # | Question | Recommendation |
|---|---|---|
| I1 | Instance declaration: a `data` row of the §6.2 shape, or a dedicated declaration form (grammar work)? | `data` row first |
| I2 | Which structures get instances? | Those whose operations a generic algorithm or operator consumes |
| I3 | v1 spelling profiles during migration | `ProjectsFrom` bridge with a deletion trigger |
| I4 | Home of the realization metamodel (`AlgebraProfile`, `AlgebraFieldTemplate`, `CostShape`) | Realization layer, identity-keyed; its own decision |
| I5 | `Map` key equality: on the carrier or on the operations | On the operations |
| I6 | `map_keys` result | `Set<K>`; ordered list only with an ordering witness |
| O1 | Run the call-site classification to validate D1 before the rewrite? | Yes, as validation, not as a prerequisite |
| D4 | **Settled 2026-10-16 (operator, via side chat).** List/String/Map/Set keep a thin externally grounded interface in `extdeps.adt`; `std` owns the carrier and its operations; realization owns storage | Recorded |
| I7 | `Implements` vs `Inhabits`: external ADT interfaces (thin contracts in `extdeps`: operations plus behavioural laws, no carrier) are not mathematical structures, and must not be modelled as either a structure or a second carrier. Which current `extdeps` declarations are interfaces rather than structures? | Keep `Implements` distinct; classify `extdeps` declarations as part of the §5.4 disposition pass |
| B1 | **Scope beyond math and lists (operator concern).** How far did these patterns (second carriers, structure names standing in for data, per-layer synonyms for one operation, string-keyed identity) percolate into higher-level code: business logic, product types, workflows? | Survey before sizing the roadmap; the cleanup is likely broader than `std` (see §15) |
| S1 | `src/v2/std` duplicates (logic, nat, integer, …) de-forked in these terms | Yes |

---

## 15. Scope beyond std (survey, 2026-10-16, HEAD 6eab655b13)

**Verdict:** std-first, repo-wide in effect. Categories 1–3 are std's own aliases and synonyms copied outward,
so they are fixed by canonicalizing std and then running mechanical sweeps. Category 4 is a separate,
non-mechanical cleanup. Counts are matching `.dag` lines outside `dag/std` and `dag/extdeps`, given as
`dag/gunbc` / `src/v2` / `dag/product`. They are rough (they include comments and strings).

| # | Pattern | Size | Risk | Fix |
|---|---|---|---|---|
| 4 | String-keyed identity (`Map<String, …>`; `decl_name`/`module_name: String`) | ~331/232/4 maps; ~2,000 `*name/_ref/_id: String` lines | **Correctness.** At least one documented collision (`dag/gunbc/instruments/prose_citation_census.dag:366`); nothing checks that a named reference exists | Designed separately: replace spellings with declaration references (the v1 spelling-identity root cause) |
| 5 | Forks recorded but not fixed ("kept in sync", "held twice", "must match") | a few dozen real, inside ~280 noisy hits | Drift | Triage by reading; each gets a trigger or a fix (§8.3) |
| 3 | Operation synonyms (`count`/`length`/`list_length`; `map_lookup`/`get`/`lookup`/`map_get`; `fold_list`) | ~8k call sites | Cosmetic | Codemod after §7.4 is fixed in std. It must rewrite call shapes, because argument labels differ by spelling |
| 1 | `FreeMonoid<…>` written for List | 178/1147/1 | Cosmetic while the alias exists | Swept by the List cut (D2) |
| 2 | Domain clones of std concepts (`OptionalHashMember`, `ControlOptionalParam`) | a handful | Minor | Fold into std types |

Other findings:
- `src/v2/extdeps` is a second extdeps tree, a layering fork in its own right.
- The fork-census instruments record forks without removing them. They become gate inputs (§8.2) or are deleted.
- `dag/product` is lightly affected throughout: tens of sites, not thousands.
