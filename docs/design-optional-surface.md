# Tracking memo: Optional Match-Surface Consistency (T3 root-cause; lane adhoc-ce8d7ae6)

> **Status: TRACKING MEMO — measured root-cause + a conditional escalation.** Records why the
> T3 keystone stalled (measured: a substrate-consistency defect, **not** the
> positional-vs-labeled model question) and registers a conditional **#9** on the hard-problems
> list *if* the lane measures the defect as deep. The flavor is **unmeasured**; this memo states
> the branch criterion so the lane's measurement maps mechanically to a verdict. Map, not
> territory — no `.dag` lands from it; the lane (adhoc-ce8d7ae6) owns the fix.

## 1. The measured outcome (measure-don't-assume held)

The ~17h T3 blocker is root-caused, and the root cause is **upstream of the model question**:
`std` `Optional`'s *match surface* is inconsistent across modules — the canonical type is
`Absent | Present { value: T }`, but match arms in some modules spell it `Some`/`None`. That
inconsistency blocked the T3 witness from ever **compiling**, so the
positional-vs-labeled-Conj fold question (the thing we were about to design/escalate) was never
actually exercised. The honest consequence: **positional-vs-labeled is still unmeasured.** We
nearly escalated a guess; Mgr-SPINE caught that the witness wasn't even compiling. Good
field-guide outcome — a confident model read, falsified the moment something tried to run it.

Routing: a **separate substrate lane** (`adhoc-ce8d7ae6`), not folded into the T3 work, because
the defect is broader than the fold — *any* `.dag` matching `Optional` hits it. Lane flavor:
**measure → fix-if-bounded → escalate-if-deep.**

## 2. What's groundable in this clone (the defect class)

(Cited worktree sites — `node_query.dag` and a `predicate.dag` match arm in still-raven-546 —
are ahead of this container's clone; `predicate.dag` doesn't exist here yet. Same class is
groundable below without stale line numbers.)

- Canonical: `type Optional<T> = Absent | Present { value: T }` (`src/v2/std/collection.dag:28`),
  with `optional_absent` / `optional_present` constructors and `optional_present_witness`
  projecting `Present → Holds`, `Absent → Violates`.
- Drift, at the sharpest site: `map_get` (`src/v2/std/collection.dag:112`) matches
  `m.lookup(key)` — typed **`Witness<V>`** (`Map.lookup: fn(K) -> Witness<V>`,
  `collection.dag:70`) — with **`Some { value: v }` / `None`** arms, then re-wraps into
  `Present`/`Absent`. That is a **three-way tangle** at one call: `Witness` values, `Some/None`
  arms, `Present/Absent` results — compiling only because of the runtime `match_pattern` bridge
  #4564 added (present-coproduct-value → `Some { value }`). The B-LOOKUP-1 marker
  (`collection.dag:107`) already names this site's bridge.
- Spread (this clone): `Some`/`None` match arms appear across ~14 `std` modules alongside the
  canonical `Present`/`Absent` — so the surface drift is corpus-wide, not a one-file typo.

This is the P1 **nickname** shape lifted to the *constructor* level: two spellings for one
projection, with a permissive runtime bridge hiding the divergence — the exact pattern that
"passes whether or not the code runs correctly" until a witness finally tries to typecheck
against it.

## 3. The branch criterion (this is the memo's contribution)

The lane's one measurement — *is the inconsistency wrong-spellings or a genuine representation
split?* — maps to a verdict by this test:

- **Surface flavor → bounded fix, no #9.** There is exactly **one** representation
  (`Present | Absent`); `Some`/`None` are stray spellings (and/or the `Witness`↔`Optional`
  conflation at `map_get` is purely the bridge, dissolvable with the Map work). Fix = normalize
  every match arm to the canonical constructors; delete the runtime spelling bridge; no type
  changes. Bounded, mechanical, lands as a sweep with one discriminating claim (a module that
  previously compiled only via the bridge now compiles against canonical arms; a `Some`-arm
  reintroduced goes red).
- **Deep flavor → #9: Optional-representation unification** (sibling of **#5 Map-representation
  unification**, `design-map-unification.md`). There genuinely exist **two representations** —
  e.g. a distinct `Some/None`-shaped option type coexisting with `Optional`, or `Optional` and
  `Witness` are being conflated as one carrier through the bridge rather than the bridge being
  incidental. Then it is the same shape as #5: *a std container whose representation is
  incoherent across modules*, requiring pick-one-representation + migrate-all-consumers +
  delete-the-bridge, atomically (P5, no second bridge). At that point it earns a full design
  doc, not this memo.

**The entanglement with #5 is real, not analogical:** the deep-flavor evidence and the Map
`map_get` Option-bridge are the *same site*. If the lane reports deep, #5 and #9 likely share a
landing — one `Witness`/`Optional`/`Map`-lookup coherence fix — and the Map design's §4.1
(`map_get` honest body) and this memo's §3 collapse into one change. The lane should weigh that
site explicitly before declaring the flavor.

**Deletion ownership (explicit, 2026-06-10):** the `map_get` site and the runtime
`match_pattern` bridge belong to the **Map landing** (`design-map-unification.md` §5) under
either flavor. The surface sweep here covers every *other* `Some`/`None` arm and does not
touch that site; deep flavor ⇒ one co-landed PR. This prevents the two lanes racing for the
same deletion.

## 4. The clean sequence (unchanged by which flavor wins)

1. Optional-surface fix lands (bounded sweep, or #9 if deep) →
2. the T3 witness finally **compiles** →
3. **one bounded run** →
4. the **real** positional-vs-labeled model datapoint (the thing we refused to guess).

Meanwhile the **honest partial** lands: #4561's green pieces ship with arms **NOT** dissolved —
the partial is labeled as partial (the `project_*` arms that do compile, no claim that the fold
question is answered). That keeps progress without booking the unmeasured datapoint as decided.

## 5. Relationship to the existing designs

- **#5 Map-representation unification** — sibling shape; shares the `map_get`/`match_pattern`
  bridge site. If #9 fires, co-land.
- **#1 Termination checker** and **#2 value-set lattice** — both recurse on `Optional`-returning
  queries (`find_named_child`, list lookups); they are *downstream consumers* of a coherent
  Optional surface. Neither is blocked by this (they use `map_get`/`Outcome` surfaces, not the
  raw arms), but both get cleaner the moment the surface is canonical.
- **Bidirectional coercion (#3) T3 constraint** — `design-bidirectional-coercion.md` §6 says T3's
  fold carrier must be production-row references, not render closures. That constraint is
  **independent of and now unblocked-pending** this fix: the model question it gates
  (positional-vs-labeled discipline-as-data) is exactly the datapoint step 4 above produces. So
  the bidirectional design's "decide T3 shape now" item should read: *decide after the one
  bounded run, not before* — the measure-first correction applied to that doc's §6/Q-B-adjacent
  scope.

## 6. What stays unmeasured / non-goals

- **The flavor.** Not declared here. The lane reports it; this memo pre-commits only the
  criterion and the #9 registration.
- **The model question.** Positional-vs-labeled remains unmeasured until step 4. No design for
  it (writing one now is the guess we just avoided).
- No `.dag` change from this memo; the sweep/#9 is the lane's, with its own claims.
- Not a new container type, not a `Witness`/`Optional` merge proposal — that would be #9's
  design to make, only if measured deep.
