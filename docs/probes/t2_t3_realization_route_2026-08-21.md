# T2+T3 measured: 52 of 59 E0308 sites are ONE root — one carrier, two realization authorities, arbitrated by reference position (2026-08-21)

**Session:** `royal-dove-436`. **Work item:** `node://adhoc-c735d227-60b`.
**Assignment (smart-ram-730):** T2 (34 sites, "structural text carrier vs host `String`") and T3
(25 sites, "collection carrier fork") are the one plausibly-shared realization cluster on the
E0308 board — **and the first deliverable is a measurement, not a repair.**

This document answers the question that was asked and stops there. It attributes every one of the
59 sites to the realization authority that produced *each side* of its `expected`/`found` pair,
reports how many reduce to one mechanism, and reports the ones that do not. No emitter, no
authority table, and no `.dag` declaration is changed by this PR.

## Verdict, up front

| | sites | share of 59 |
|---|---:|---:|
| **shared root** — one modeled carrier, two realization authorities, selected by the *syntactic position* of the reference | **52** | 88.1% |
| other mechanisms already coded elsewhere on the same board | 7 | 11.9% |

T2 is **34/34 shared root — no residue at all.** T3 is **18/25**; its remaining 7 sites are three
different mechanisms that the 2026-08-21 partition codes separately, and are reported as such below
rather than absorbed into the cluster.

The root, stated once:

> The emitter answers *"what host type realizes this modeled carrier"* from **more than one
> authority**, and selects between them by **where the reference appears** — the type renderer
> reads the realization tables (`extdeps.languages.rust.types` `rust_type_checkpoints` /
> `rust_algebra_inhabitants`), the expression renderer emits the carrier's **own declared
> structure**. Nothing reconciles the two. Every site where one carrier is reached from both
> positions is an E0308.

This is not a spelling problem. At the T3 head the two sides are the **same spelling, in the same
module, on the same field** — `Set<Symbol>` in `v2.compiler.02_parse` `ParseTableRealization`
(type position) against `Set { member: … }` in that module's `parse_table_empty` (value position).
A fix keyed on names would not see it.

## Subject, ref, producer

| | |
|---|---|
| subject | `src/v2/compiler/03_ingest.dag` closure, emitted to Rust |
| pairs measured | the 59 T2+T3 rows of [`e0308_partition_2026-08-21/sites_classified.tsv`](e0308_partition_2026-08-21/sites_classified.tsv), taken at `origin/main@2a2bd0ad59…` |
| corroborating emit | `gunbc compile --source-root dag --source-root src/v2 --entry src/v2/compiler/03_ingest.dag --target rust --dependency-pool-index primary-precedence`, run at `d72ffe8708…`, `compiled: 177 files emitted, 504 diagnostics` |
| route attribution | [`t2_t3_realization_route_2026-08-21/routes.tsv`](t2_t3_realization_route_2026-08-21/routes.tsv), 59 rows |
| M | **1**. Every figure here is "at M=1, 03_ingest", inherited from the partition it reads |

**Two refs, deliberately, and what that costs.** The pairs come from the partition's pinned
`2a2bd0ad59…`; the emitted text quoted as corroboration comes from a fresh emit at `d72ffe8708…`
(504 diagnostics against that run's 502). These are **not** differenced anywhere — no count in this
document is computed from the second run. The second run is used only to read what the emitter
actually wrote at a construction the first run reported, and a quoted specimen is admitted **only
where the emitted text independently exhibits the recorded pair's shape**, so a shifted line
disqualifies itself rather than being attributed. Three of the 51 distinct `(file, line)` pairs did
not so exhibit (`v2_std_compilers_target_model.rs` 500 and 6056, `v2_std_integer.rs` 1151); they are
**not quoted below** and carry no emitted-text evidence. Their route attribution stands on the
recorded pair alone, exactly like the other 56.

## The three carriers, and the two authorities each

Each carrier below is **one type** in the `.dag` substrate — the alias and its definiens are the
same type, by declaration, not by convention.

| carrier | declared as | authority A | authority B |
|---|---|---|---|
| `String` | `std.string_type` `String = FreeMonoid<Char>` | `rust_type_checkpoints` row `String → String` | `rust_algebra_inhabitants` row `FreeMonoid → Vec<{0}>` |
| `Set<T>` | `std.types` `Set<element> = PointwisePower<element>` | `rust_algebra_inhabitants` row `PointwisePower → BTreeSet<{0}>` (`im::OrdSet`, aliased `as BTreeSet` in the seed prelude) | `std.algebra` `PointwisePower<T> { member: fn(T) -> Bool }` — the declared record |
| `Map<K,V>` | `std.types` `Map<key,value> = PartialFunction<key,value>` | `rust_algebra_inhabitants` row `PartialFunction → HashMap<{0}, {1}>` | `std.algebra` `PartialFunction<K,V>` — the declared record |

Authority A is consulted from **type** position. Authority B is what the **expression** renderer
produces. Both are reachable for every one of the three carriers, and the corpus reaches both —
because in the `.dag` layer there is nothing to notice: one type, written two ways, in a substrate
that resolves the alias.

## Route attribution — the measurement

Each side of each pair is labelled by the authority whose output it is:

- `CP:<name>` — a `rust_type_checkpoints` row (spelling-keyed, arity 0).
- `AI:<algebra>` — a `rust_algebra_inhabitants` row.
- `SR:<type>` — the carrier's own declared structure, rendered as a record.
- `UNROUTED` — a spelling no realization authority produces; the pair is about something else.

```
34  T2   CP:String            <->  AI:FreeMonoid          carrier: String = FreeMonoid<Char>
10  T3   AI:PartialFunction   <->  SR:PartialFunction     carrier: Map<K,V> = PartialFunction<K,V>
 8  T3   AI:PointwisePower    <->  SR:PointwisePower      carrier: Set<T> = PointwisePower<T>
--- 52 shared root -----------------------------------------------------------------
 5  T3   UNROUTED             <->  AI:FreeMonoid          RT-builtin (see below)
 1  T3   AI:FreeMonoid        <->  AI:FreeMonoid          C — differs only at the element
 1  T3   UNROUTED             <->  UNROUTED               nesting depth — differs only at the element
```

T2's 34 sites carry **one** route pair and nothing else. Their bidirectionality — 19 sites
`expected Rc<Vector<_>>, found String` against 7 in the exact reverse — is not two mechanisms; it
is one, seen from whichever end the failing reference sat at. The four literal spellings collapse
to two route directions:

```
19  expected Rc<Vector<_>>     found String           |  23 expected AI:FreeMonoid
 4  expected Rc<Vector<i64>>   found String           |
 7  expected String            found Rc<Vector<_>>    |  11 expected CP:String
 4  expected String            found Rc<Vector<i64>>  |
```

`Rc<Vector<i64>>` and `Rc<Vector<_>>` are one route with the element resolved (`std.types`
`Char = Int`, checkpoint row `Int -> i64`), not a third category. An identity is symmetric, so the
`.dag` layer admits the flow in both directions and the realization refuses in both.

## Emitted specimens

**T2 — `String` field, `FreeMonoid` value, one declaration.** `v2.compiler.01_tokenize` declares
`LexRuleApply.LexRuleToken.lexeme: String` and `LexMatchResult.LexMatchAccepted.lexeme: String`,
and constructs those same fields with `FreeMonoid` constructors and operations:
`lex_match_char_pred` writes `lexeme: Cons { head: c, tail: std.algebra.Empty }`, and
`lex_repeat_step` writes `lexeme: list_append(left: state.lexeme, right: consumed)`. Emitted:

```rust
pub lexeme: String,                                                   // type position   -> CP:String
lexeme: Rc::new({ let mut __cons_v = (*Rc::new(vec![])).clone();
                  __cons_v.insert(0, c.clone()); __cons_v }),         // value position  -> AI:FreeMonoid
lexeme: list_append(state.lexeme.clone(), consumed.clone()),          // consumer takes Rc<FreeMonoid<T>>
```

**T3 — same spelling, same module, same field.** `v2.compiler.02_parse` declares
`ParseTableRealization.nullable_set: Set<Symbol>` and `parse_table_empty` constructs
`nullable_set: Set { member: fn(_) { false } }`. Emitted:

```rust
pub nullable_set: Rc<BTreeSet<String>>,     // type position   -> AI:PointwisePower
nullable_set: Rc::new(Set { … }),           // value position  -> SR:PointwisePower
pub type Set<Element> = Rc<crate::std_algebra::PointwisePower<Element>>;
```

The alias emitted for `Set` resolves to the **declared record**, while the field emitted for the
same alias resolves to the **inhabitant row**. One `.dag` declaration, two host types, in one
emitted file.

**The prior assumption this refutes, named.** `dag/test/claim/map_key_alias_hop_witness_test.dag`
states, as the premise of its negative arm, that a realized alias renders as its host type and that
therefore *"the emitter never renders that structure"* — a realized alias's declared-structure
fields never reach the fixpoint. That holds for `Bytes`, the checkpoint carrier it tests. It does
**not** hold for `Set`/`Map`: the emitter renders the declared structure at every value position,
and the 18 T3 shared-root sites are what that costs. The witness is not wrong about its own
subject; the generalization it reads as is.

## The 7 sites that are not this root

Reported rather than absorbed, because a cluster that swallows its residue is not a measurement.

- **5 × RT-builtin** (`std_state_durability.rs`). Pairs read element-vs-carrier
  (`StateDurabilityBoundary` against `Vector<Rc<StateDurabilityBoundary>>`), and the emitted text
  names the cause directly: `v1_rt::append(acc.ordered.clone(), Rc::new(vec![boundary.clone()]))`
  against `v1_rt` `append<T>(list: Rc<Vec<T>>, item: T)`. A host builtin whose signature is
  *push* has intercepted the corpus's *concat*. This is the mechanism the 2026-08-21 partition
  already codes as `RT-builtin` (20 sites); these 5 belong with it, not with T3.
- **1 × C** (`v2_lens_complexity_lowering.rs`). Both sides are `AI:FreeMonoid`; they differ only at
  the element, `Vector<()>` against `Vector<Rc<ComplexityLowering>>`. The carrier agrees — the
  element collapsed to `()`, which is the partition's `C` root.
- **1 × nesting depth** (`v2_std_fold_assembly.rs`). `Rc<Outcome<Rc<Vector<Rc<Vector<…>>>>>>`
  against `Rc<Outcome<Rc<Vector<Rc<Node>>>>>` — the carrier agrees at every level; the nesting
  depth of the element does not.

None of the three is a carrier fork, and none would be closed by anything that closes the 52.

## Controls, and what this does not establish

- **The classifier is fail-closed.** `UNROUTED` is a live arm: 7 of the 118 sides carry it, and the
  attribution refuses to call a pair shared-root unless *both* sides route and the two routes name
  the same carrier. A run with no `UNROUTED` would be the finding.
- **Position, not spelling, is the discriminator — and it is checkable.** The T3 head is one
  spelling (`Set`) reaching two host types. Any account of this root that is keyed on the alias
  vs. its definiens is refuted by that site alone.
- **Not a corpus board.** M=1, 03_ingest's closure. Per-module boards are overlapping closures;
  these 59 may not be summed with another module's.
- **No before/after is claimed.** Nothing here is differenced against the 204/275 boards or against
  the 502-diagnostic run; see the two-refs note above.
- **This is a route attribution, not a root-cause trace into the emitter.** The two authorities are
  named from the corpus declarations that carry them and from the emitted text they produce. The
  emitter code path that chooses between them by position was **not** traced in this run, and no
  authored-decision count is offered.
- **No repair is proposed as decided.** The shape of one is visible — `std.coercion`
  `TypeRealizationDecision` (landed #8739) is exactly the *"which of a named declaration's possible
  renderings actually applies"* query this root needs, and it currently has **zero consumers** in
  the corpus. Whether the expression renderer should consult it, and what that does to the
  carriers' construction walls, is the next question, not this document's answer.

Per-site attribution: [`t2_t3_realization_route_2026-08-21/routes.tsv`](t2_t3_realization_route_2026-08-21/routes.tsv)
(columns: file, line, col, expected, found, coded_root, expected_route, found_route, carrier,
verdict, reason).

To repeat: take the T2+T3 rows of the partition TSV; label each side by the realization authority
that produces that spelling; a pair is shared-root when both sides route and the routes name one
carrier.
