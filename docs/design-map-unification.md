# Design: Map Representation Unification (extensional data, one equality authority)

> **Status: DESIGN — map, not territory** (INVARIANTS.md "Map vs territory"). No code lands
> from this doc without the consumers named in §6 (E-10). This is the *real dissolution*
> behind the recurring Map gap: #4564 (`43b7516`) fixed the runtime symptom (structural-key
> `Map==` false even vs itself); the model-side incoherence that produced it is still in the
> tree, and three live interim markers are waiting on exactly this design:
> `feature:B-MAP-LOOKUP-OPTION-C-1` (`src/v2/std/collection.dag:65` — "dissolve-on-arrival:
> **unified runtime representation** so lookup ops need no per-site bridge"),
> `feature:B-LOOKUP-1` (`collection.dag:107`, `lens/affected_set.dag:627`), and
> `feature:finite-set-uniqueness-witness` (`collection.dag:157`).

## 1. Problem

`v2.std.collection` declares:

```dag
type Map<K, V> { lookup: fn(K) -> Witness<V> }     // collection.dag:69
```

That is a **partial function by intension** — a stored closure. But every consumer treats
`Map` as a **finite extensional structure**: they insert pairs, expect decidable whole-map
equality (`affected_set` frontier receipts, `02_parse` `parse_table_lookup` tables,
`PrimitiveFactBundle.spec_facts`), and expect lookup-absent to be a *normal outcome*, not a
defect. The name says "Map"; the structure says "PartialFunction" — the P1 **nickname**
problem shape, and this one is load-bearing:

- **Equality:** intensional values have no decidable equality. Pre-#4564 the closure form
  reached the runtime with no `PartialEq` arm, so structural-key `Map ==` was false even
  reflexively — the root cause of the false affected_set receipts. #4564 fixed the *runtime*
  by giving `Value::Map` a native finite-pairs representation with extensional equality
  delegating to `Value::eq` (single authority, P2) and fail-closed inserts for un-keyable
  values (P3). The model still says closure.
- **The bridge:** because model and runtime disagree, v2 carries the `raw_map_lookup`
  dual-dispatch chokepoint (Option-C, ctrl#1476 B6) bridging closure-form and native-form
  per operation site — a P5 bridge whose own marker names unification as its dissolution.
- **The lookup contract:** `Map.lookup` returns `Witness<V>`, but bootstrap projects it as
  `Option`, and `map_get` (`collection.dag:112`) matches `Some/None` against a
  `Witness`-typed call — held together by a runtime match-pattern bridge. The substrate
  intent (B-LOOKUP-1: `^map_key_absent` `Violates` ⇒ `Absent`, any *other* `Violates` ⇒
  `Rejected`) is unimplemented. "None vs Violates" is incoherent because the type is wrong:
  a *finite data* map's lookup has exactly one failure mode (absent); only a genuine partial
  *function* can "violate."

Each symptom got a patch (#4560 per-key workaround, then #4564 native repr, the `map_get`
Option bridge); the gap resurfaces because the model type misstates what a Map is.

## 2. What already exists (M9 DFS)

| Piece | Where | Role |
|---|---|---|
| `Map<K,V>` closure form + `empty_map`/`map_insert` closure-chain constructors | `collection.dag:69-104` | the type to re-shape |
| `map_get -> Outcome<Optional<V>>` (the public read surface) | `collection.dag:112` | **signature is right; body/bridge dissolve** — most consumers keep compiling unchanged |
| `FiniteSet<T>` + pending uniqueness witness (`feature:finite-set-uniqueness-witness`) | `collection.dag:157-161` | the key-set concept: a map's domain *is* a finite set; the uniqueness witness this marker wants is the same one map keys need |
| `List<T> = FreeMonoid<T>`, `Set<T> = PointwisePower<T>` | `collection.dag:63-64` | the extensional family Map joins |
| `TotalMap<K,V>` / `TotalPolicy<…>` (stored-closure carriers) | `collection.dag:162-167` | genuinely intensional — they stay function-shaped but stop sharing the "Map" family name (§4.4) |
| Runtime precedent: native `Value::Map`, `CanonKey` delegating `Eq` to `Value::eq`, fail-closed un-keyable insert, fail-closed non-String JSON keys, extensional `==` | #4564 (`src/v1/stage0/src/v1_interpreter.rs`) + claim `test/claim/manual/map_structural_key_equality.dag` | **the semantics already proven by execution** — the model catches up to it, not vice versa |
| Consumers at the gap | `affected_set.dag:599-643` (`mark_rerun`/`mark_excluded`, B-LOOKUP-1 site), `02_parse` `parse_table_lookup`, `model_core.PrimitiveFactBundle.spec_facts` | the executing consumers for the slice |

**Substrate target named (P1):** `v2.std.collection` — `Map<K,V>` re-shaped in place; one new
honestly-named carrier `PartialFunction<K,V>` for what the old shape actually was; the v2
`raw_map_lookup` chokepoint and the `map_get` Option-bridge delete. No connective/behavior
change.

## 3. Substrate-fact introduction procedure (MODELING.md, cited)

- **Step 1 (DAG-ancestor):** ran. A finite map is *a finite set of pairs with unique keys* —
  the ancestors are `FreeMonoid` (entry list), `FiniteSet` (the key domain + its pending
  uniqueness witness), and pair `Conj`. `PartialFunction` is the separate ancestor the old
  shape belongs to. Map *inhabits* partial-function (projection, §4.3); it is not one.
- **Step 2 (coproduct-vs-coordinate):** ran. `MapEntry { key, value }` — coordinates, record
  correct. Lookup result `Present | Absent` — alternatives, sum correct. The old
  `Witness<V>` result conflated a third thing (defect) that the data form cannot produce —
  the coproduct shrinks because the representation got honest.
- **Step 3 (primitive-vs-lens-extensible):** ran. Substrate-declared: collections are
  kernel-adjacent vocabulary every layer consumes.

## 4. Design

### 4.1 The shape: Map is extensional data

```dag
type MapEntry<K, V> { key: K  value: V }
type Map<K, V> {
  entries: List<MapEntry<K, V>>        // FreeMonoid — the finite extension
  // key-uniqueness witness: same structural witness the FiniteSet marker wants;
  // shared, not duplicated (its dissolution arrival)
}
```

- **Entries are maintained in canonical key order — the canonical form IS the
  representation** (ruling, operator 2026-06-09: simplest correct deterministic form). There
  is no separate "insertion order" concept and no order-independence machinery: `map_insert`
  is replace-or-ordered-insert at the key's canonical position; two equal maps are
  *structurally identical* entry lists, so whole-map `==` is plain structural equality —
  exactly what #4564's extensional semantics mean, with the order question dissolved rather
  than parameterized. Iteration and serialization inherit the same single order, so DB-8
  (deterministic emission) and the fixed-point artifact comparison are satisfied by
  construction.
- `empty_map` = empty entries. `map_get` keeps its exact public signature
  (`Outcome<Optional<V>>`); its body becomes a fold over entries; `Absent` means absent,
  and the `Rejected` arm is *unreachable by construction* for the data form — B-LOOKUP-1's
  "None vs Violates" question dissolves rather than being answered.
- **Key validity:** a key must be decidably equatable (the runtime already rejects
  closure/fn/NaN keys via reflexivity-under-`Value::eq`, fail-closed). The model states the
  obligation on `K` (equality-bearing carrier); until the language has first-class
  constraint bounds on type parameters, the runtime check remains the enforcement and the
  obligation is documented at the type — honestly marked, with constraint-substrate arrival
  as its dissolution trigger. (Construction-over-convention is the destination; the interim
  is the same fail-closed check #4564 landed.)

### 4.2 What dissolves in the runtime

With no closure form left in the model, the runtime's dual representation ends:

- `raw_map_lookup` per-site dual dispatch (Option-C) deletes — its marker's named arrival
  verbatim ("unified runtime representation so lookup ops need no per-site bridge").
- The `match_pattern` value→`Some` bridge that made `map_get`'s match work over native maps
  deletes with `map_get`'s honest body.
- `Value::Map` (CanonKey-keyed pairs) becomes *the* representation rather than the fast
  path — #4564's semantics (Eq delegation, fail-closed inserts, fail-closed JSON keys)
  carry over unchanged; that PR's claim corpus is the regression floor.

### 4.3 Map inhabits PartialFunction (the coercion tie-in)

The old shape doesn't vanish — it gets its true name:

```dag
type PartialFunction<K, V> { lookup: fn(K) -> Witness<V> }   // what Map used to claim to be
```

Every finite map *projects* to a partial function (`fold` over entries; absent ⇒
`Violates(^map_key_absent)`) — a derivable, one-direction widening: extensional → intensional
derives with a witness; intensional → extensional **refuses** (a closure's extension is not
enumerable — `WouldLoseInformation`'s sibling, surfaced not guessed). This is the same
derive-the-determined / refuse-the-rest boundary as the coercion thesis, applied to the
collection family, and it is why the unification matters beyond Map: extensional data is
what makes whole-value comparison decidable everywhere Maps appear — affected_set receipts,
fact-bundle coincidence proofs (`spec_facts: Map<Symbol, Node>`), and the self-host
fixed-point artifact comparisons ([`design-self-host-fixed-point.md`](design-self-host-fixed-point.md)
§3 — closures inside compared values would make bit-identical meaningless).

### 4.4 The honest renames around it

- `TotalMap<K,V> { lookup: fn(K) -> V }` is a **total function carrier**, not a map — rename
  (`TotalFunction` or fold into the function-carrier family) per the P1 nickname rule;
  `TotalPolicy` likewise stays intensional and correctly so (templates are functions).
- `FiniteSet<T>`'s pending uniqueness witness lands once, shared: a map's key domain is a
  `FiniteSet<K>`; the same witness discharges both markers.

## 5. Migration (behavioral, atomic where it touches the contract — P5, no second bridge)

- **Census first (measure-first):** count direct `.lookup(` call sites on `Map`-typed values
  across `src/v2` (distinct from `map_get` callers, which migrate for free since the
  signature holds). Direct intension-callers either switch to `map_get` or actually want
  `PartialFunction` — each is a one-line, mechanical decision, but the *count* decides
  whether the swap is one PR or a short sequence with the old constructor briefly
  forbidden-to-new-callers (no long-lived dual form — the bridge-as-steady-state shape is
  what this design exists to end).
- The `affected_set` B-LOOKUP-1 site (`mark_excluded`) and `parse_table_lookup` migrate in
  the landing PR — they are the consumers whose receipts prove the change.
- v2 interpreter chokepoint deletion rides the same change as the model swap (the runtime
  already prefers the native form; deleting the closure arm is removal, not addition).
- **Deletion ownership (deconflicted with the Optional lane, 2026-06-10):** the `map_get`
  site and the runtime `match_pattern` Witness→`Some` bridge are owned by **this landing**,
  regardless of the Optional lane's flavor verdict. The Optional surface sweep
  (`design-optional-surface.md`) normalizes match arms everywhere *else* and explicitly
  excludes this site; if the Optional lane measures **deep**, the two land as one PR (#9
  co-landing per that memo §3) with this design's §4.1 as the `map_get` authority. Two PRs
  must not both reach for the same deletion.

## 6. Consumers and minimal slice (E-10 / seesaw)

- **Consumers (exist, executing):** the #4564 claim
  (`manual/map_structural_key_equality.dag`, wired into the roster) — stays green over the
  re-shaped model; the four affected_set Excluded/rerun receipts (greened by #4564, now
  model-honest); `parse_table_lookup` claims.
- **Minimal slice:**
  1. re-shape `Map` + derived `map_get` body + `PartialFunction` carrier + `TotalMap` rename
     in `v2.std.collection`; migrate the censused direct-lookup sites;
  2. delete the v2 `raw_map_lookup` dual dispatch + `match_pattern` map bridge;
  3. claims under `src/v2/test/claim/std_collection/` (plus the existing manual claim):
     **green** — whole-map structural-key `==` reflexive and order-independent, *in `.dag`*,
     over a map built by `map_insert`; **green** — map→PartialFunction projection derives
     and looks up; **red (discriminating)** — duplicate-key construction is unrepresentable
     /fails closed; un-keyable key insert rejects (the #4564 case, now model-level); a
     closure-carrier (`PartialFunction`) compared for equality is rejected, not false —
     proving the incoherence is gone, not relocated.
- The B-MAP-LOOKUP-OPTION-C-1, B-LOOKUP-1, and finite-set-uniqueness markers close in the
  landing PR — three named dissolutions, one change (the receipt that this was the real fix,
  not the next symptom patch).

## 7. Open questions — escalate, don't improvise

- **Q-M1 — entry order semantics. RESOLVED (operator 2026-06-09; reconciled 2026-06-10):**
  canonical key order **is the representation** (§4.1) — entries are maintained sorted, there
  is no insertion-order concept, iteration and serialization inherit the one order, and
  whole-map `==` is plain structural equality. (An earlier draft of this question floated
  "insertion order preserved in `entries`" — withdrawn: it would reopen the order-dependence
  §4.1 dissolves.) One requirement this introduces, stated honestly: keys need a canonical
  **total order** (not just decidable equality) — the same fail-closed key-validity site
  enforces it, and the runtime's `CanonKey` handling is the existing precedent.
- **Q-M2 — uniqueness as witness vs by-construction.** Wave 1: `map_insert`
  replace-or-append keeps uniqueness by construction and the witness is a structural check;
  a richer `UniqueKeys` witness carrier (shared with `FiniteSet`) can follow when a consumer
  needs to *transport* the proof. Don't build the carrier ahead of that consumer (E-10).
- **Q-M3 — `Map` performance shape.** Entry-list folds are O(n) lookup; the runtime's native
  form is already keyed. If/when a `.dag`-level performance fact is needed, it is a cost-
  algebra fact on the same single representation — **not** a second representation. Flagging
  so nobody reintroduces the dual form as an "optimization."

## 8. Non-goals

- No new collection kinds, no ordered-map/multimap family (no consumer).
- No change to `Witness` itself — it stops being misused at one site, that's all.
- No JSON/serialization policy changes beyond inheriting #4564's fail-closed non-String key
  stance.
- No generic constraint-bounds language feature — named as the eventual dissolution for the
  key-validity obligation, designed elsewhere if/when pulled by more than this consumer.
