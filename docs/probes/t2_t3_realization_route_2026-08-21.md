# T2+T3 measured: 52 of 59 E0308 sites are ONE root — one carrier, two realization authorities, arbitrated by reference position (2026-08-21)

**Session:** `royal-dove-436`. **Work item:** `node://adhoc-c735d227-60b`.
**Assignment (smart-ram-730):** T2 (34 sites, "structural text carrier vs host `String`") and T3
(25 sites, "collection carrier fork") are the one plausibly-shared realization cluster on the
E0308 board — **and the first deliverable is a measurement, not a repair.**

This document attributes every one of the 59 sites to the realization authority that produced
*each side* of its `expected`/`found` pair, reports how many reduce to one mechanism, and reports
the ones that do not. It then traces the arbiter that selects between those authorities — an
authorized follow-up from `smart-ram-730`, still measurement-only. **No emitter, no authority
table, and no `.dag` declaration is changed by this PR**, and no repair is proposed as decided.

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

The arbiter is traced below, and the trace sharpens this: the two renderers do not merely sit in
different positions, they **key on different facts** — the type renderer on the authored *element*
spelling, the value renderer on the constructor's resolved *carrier name*, element-blind.

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

**The prior assumption this narrows, named — and it is the RATIONALE, not the test.**
`dag/test/claim/map_key_alias_hop_witness_test.dag` states, as the stated rationale of its negative
arm, that a realized alias renders as its host type and that therefore *"the emitter never renders
that structure"* — a realized alias's declared-structure fields never reach the fixpoint. **That
witness is not failing and must not be made to fail.** Its fixture is `Bytes`, `Bytes` renders as
`Vec<u8>`, and for `Bytes` the claim holds exactly as written; the assertion is correct.

What the evidence above falsifies is the **universal form of the rationale**, which is stated
without qualification while the fixture exercises exactly one checkpoint row. For `Set`/`Map` the
emitter *does* render the declared structure — at every value position — and the 18 T3 shared-root
sites are what that costs. The remedy is therefore to narrow the stated premise to what the fixture
actually exercises and to name `Set`/`Map` as the known counter-case, **not** to change the
assertion.

**Where else that premise is load-bearing, so the two artifacts find each other.**
`v1.trait_derive_emit` `map_key_alias_hop_reconciliation_note` (authored 2026-08-21) leans on the
same "the emitter never renders that structure" reasoning. It is **owned by `smart-ram-730`, who
authored it and is correcting it**; it is deliberately **not edited by this PR**, which changes no
declaration anywhere. This paragraph exists so a reader arriving at either artifact finds the other.

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

## The arbiter, traced (authorized follow-up, still measurement-only)

`smart-ram-730` authorized tracing the position-dependent choice. It is traced here, in the same
PR, because the trace **refines the root statement above** and leaving the two apart would leave a
reader holding the coarser account. Nothing is repaired.

### Both halves are named, and they key on different facts

**Type position** — `v1.compiler.05_emit_rust` `render_rust_decl_type` (and its `render_rust_fn_sig_type`
sibling). Its **first line**, ahead of every other arm, is the text-carrier bypass:
`is_host_text_carrier_type` → render `"String"`. That predicate accepts the authored spelling
`String`, **or** `FreeMonoid` / `List` when `rust_host_text_carrier_elem_name` returns exactly
`"Char"`. Anything else falls through to `rust_applied_type_base` → `v1.compiler.coercion`
`coerce_primitive_type` (the checkpoint table) and the container templates
(`coerce_container_template` → `lookup_inhabitant` → `rust_algebra_inhabitants`).

**Value position** — `v1.compiler.05_emit_rust` `emit_typed_record_lit`. Its `Cons`-under-`FreeMonoid`
arm emits a vec splice (`{ let mut __cons_v = (*tail).clone(); __cons_v.insert(0, head); __cons_v }`)
on the condition `tn == "Cons" && effective_parent == "FreeMonoid"`, and its zero-field arm routes
through `rust_seed_host_freemonoid_empty` → `rust_seed_host_container_base`, which is, in full:
`List | FreeMonoid → "Vec"`, otherwise none. Every other record literal — `PointwisePower`,
`PartialFunction` — falls to the ordinary record path and emits the **declared struct**.

So the asymmetry is not merely that two authorities exist. It is that **the two renderers key on
different facts**:

| | keys on | consequence |
|---|---|---|
| type renderer | the authored **element** spelling (`== "Char"`) | `FreeMonoid<Char>` → `String`; `FreeMonoid<T>` → the inhabitant |
| value renderer | the constructor's resolved **carrier name**, element-blind | every `Cons` / `Empty` → `Vec`, whatever the field is declared as |

**This refines "position, not spelling" rather than reversing it.** Position is *where* the two
authorities are selected; the facts they key on are what makes them disagree. The T3 head remains
the clean proof that spelling alone cannot explain it — one spelling, `Set`, two host types in one
emitted file — and the T2 head now has a sharper statement than "two spellings": a generic
`FreeMonoid<T>` **cannot** satisfy the type renderer's test, because `T` is not spelled `Char`. The
decision is syntactic and pre-instantiation, so no monomorphization repairs it.

### The emitted aliases carry the fork, and they are the control

```rust
pub type FreeMonoid<T> = Vec<T>;                                    // inhabitant
pub type List<Element> = Vec<Element>;                              // inhabitant  — agrees with FreeMonoid
pub type Set<Element>  = Rc<crate::std_algebra::PointwisePower<Element>>;   // declared record
pub type Map<Key, Value> = Rc<crate::std_algebra::PartialFunction<Key, Value>>;
pub type String = std::string::String;                              // host string
```

`List` and `FreeMonoid` resolve to the **same** host type and contribute **zero** sites between
themselves — the negative control sits in the same emitted crate as the positives. `Set` and `Map`
resolve to the declared record while the type renderer reaches them through the inhabitant rows;
`String` resolves to the host string while both of its other spellings reach `Vec`.

### Which arm each of the 52 sites reaches

| arm | T2 | T3 | what the emitted text shows |
|---|---:|---:|---|
| **A** — generic carrier signature: the callee is a `FreeMonoid<T>`-typed fn, so the type renderer's `"Char"` test cannot fire | 25 | 0 | `list_append(state.lexeme.clone(), …)`, `length(lexeme.clone())`, `is_empty(text.clone())` |
| **B** — declared-structure constructor: the value renderer emits the carrier's own structure | 4 | 17 | `Rc::new({ … __cons_v.insert(0, c.clone()) … })`, `Rc::new(vec![])`, `Rc::new(Set { … })`, `Rc::new(Map { … })` |
| **C** — direct carrier-to-carrier assignment, no call or constructor on the line | 3 | 0 | `tail: source.clone()`, `remaining: rem.clone()` |
| **UNALIGNED** — line did not exhibit the recorded pair; not attributed | 2 | 1 | — |

Arm A is **T2-only and is the single largest arm on the board**, and it is the one a repair aimed
at constructors would leave entirely standing. Arm B is **T3's whole shared-root population** and
four of T2's. Per-site: [`t2_t3_realization_route_2026-08-21/arbiter_arms.tsv`](t2_t3_realization_route_2026-08-21/arbiter_arms.tsv).

### One half of this was already predicted in the corpus

`v1.compiler.05_emit_rust` `checkpoint_table_bypasses_identity_note` records, as a stated
next-rung trigger, that `emit_typed_item`'s type-alias arm special-cases `item_text == "String"`
unconditionally, so *"the declaration and its own references can now disagree in the SAME emitted
file."* The emitted `v2_std_text.rs` `pub type String = std::string::String;` above is that
prediction standing in the artifact. What that note did not have — and states it did not have — is
a live site count. **This is it: 34, all of T2.** The note's own framing is unchanged by that; only
its "NO live site is claimed" clause is now answerable.

### Two cautions carried forward to whoever repairs this

Both were raised by `smart-ram-730` and are recorded here rather than in a message, because they
constrain the *next* measurement and a message is not where the next author will look.

1. **Report site conversion, not category totals.** With two authorities live, a change can move a
   site from one family to another rather than close it. A falling E0308 count is consistent with
   both, so the instrument must be a per-site join against this TSV, not a histogram difference.
2. **The discriminating control must show one authority consulted in BOTH positions.** A control
   that only shows the count fall is equally consistent with one authority simply not firing —
   which is the absorbing-fallback shape (DESIGN §5) wearing a green build.

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
- **The trace names branches; it does not price a repair.** The arbiter section names both renderer
  arms, the predicates they key on, and which arm each of the 52 sites reaches. It does **not**
  establish what any of those arms should do instead, what a unified consultation costs elsewhere in
  the emitter, or that the three arms have one fix — nothing was changed and nothing was re-emitted
  under a change, so no such claim could be evidenced here.
- **No repair is proposed as decided.** The design that follows from this trace is
  [`docs/plans/carrier-realization-arbiter-repair-design.md`](../plans/carrier-realization-arbiter-repair-design.md);
  it leaves the one policy question it turns on to that question's owner and proposes a measurement,
  not a merge, as the next executable step.
- **CORRECTION, 2026-08-21, same day: an earlier revision of this document and of this lane's PR body
  claimed `TypeRealizationDecision` has "zero consumers" in the corpus. That is FALSE.**
  `v1.compiler.coercion` `lookup_checkpoint` is a thin derivation of `type_realization_decision` for
  every `decl_file != ""` caller, and `v1.compiler.trait_derive_emit`'s alias-hop arm reaches it that
  way. The error came from grepping the *type* name rather than the *function*. It is corrected here
  rather than quietly dropped because it was relayed upward and planned against, and because the
  corrected fact is the stronger one: the authority is present, correct, **and unreachable** for this
  carrier — five type renderers (`render_rust_type`, `render_rust_type_without_applied_binding`,
  `render_rust_applied_type`, `render_rust_decl_type`, `render_rust_fn_sig_type`) each return on
  their first line via `is_host_text_carrier_type` → `"String"`, unconditional on `decl_file`, while
  `structural_declaration_modules_for("String")` lists both declaring modules. An unreachable wall,
  not a missing one (DESIGN §6 coverage-by-illusion). Established by reading those five renderers'
  control flow; **not** established by a discriminating execution, and it says nothing about a sixth
  renderer that may handle some type position without that preamble.

Per-site attribution: [`t2_t3_realization_route_2026-08-21/routes.tsv`](t2_t3_realization_route_2026-08-21/routes.tsv)
(columns: file, line, col, expected, found, coded_root, expected_route, found_route, carrier,
verdict, reason).

To repeat: take the T2+T3 rows of the partition TSV; label each side by the realization authority
that produces that spelling; a pair is shared-root when both sides route and the routes name one
carrier.
