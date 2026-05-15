# M2 / Class-5 Deferral Debt Survey

**Status:** survey artifact — operator-requested 2026-05-15 ("Yeah this is always what happens in these projects - basically we defer something to M2 because its hard/delays progress - but then we keep working and accumulate debt. Can we please find all instances of this happening in this project - i want to attack one of these problems specifically so we can gather some confidence").

**Authority:** none — this is investigation/inventory, not ratification.

## The pattern

`docs/design-pure-bootstrap-zero.md` is LIVE since 2026-04-25. Earlier scaffolds were named with explicit dissolution triggers pointing at "M2+" or "class-5 grammar lands." **M2/class-5 didn't land.** Workers built around the missing substrate-language features via hand-Rust trampolines + ArrowBody::Unparsed std fns + parallel-representation mirrors, and the deferred debt accumulated.

Quantitative scope:
- **119 references to "class-5"** across `docs/` + `src/v3/` + `dsl/`
- **61 citations** naming class-5 as a blocker / dissolution trigger
- **161 ArrowBody::Unparsed std fn bodies** (58% of 276 total) — direct consequence
- **1 dedicated design doc** (`docs/design-m2-feature-parity.md`, status "Design ready for implementer review") that has been sitting unimplemented

## M2 Feature Parity items (per docs/design-m2-feature-parity.md)

| DB | Title | Stage | Size | Design status |
|---|---|---|---|---|
| DB-9 | Mutual recursion lowering | 3a.1 | L | Design ready (separate doc `design-mutual-recursion-lowering.md`) |
| **DB-10** | **`data` value semantics** | **3a.2** | **S** | **Design ready** (substrate carrier already exists; downstream consumers missing) |
| DB-11 | `where` refinement predicates | 3a.3 | M | Design ready |
| DB-12 | Surface generics | 3a.4 | S | Design ready |
| DB-13 | `Disj` dotted-path | 3a.5 | S | Design ready |

## Class-5 grammar gaps (the substrate-language sub-program)

Class-5 is a category of substrate-language gaps that prevent `.dag` source forms from lowering / executing structurally. Each gap has been named with a dissolution trigger; none have closed.

| # | Gap | Concrete blockage | Workaround in tree |
|---|---|---|---|
| **1** | Brace-bodied fn declarations on `.dag` files | `parse_generated.rs:727-758` skip-balance-parses `.dag` fn bodies → `ArrowBody::Unparsed`. Affects 161 std fns. (Only `.v3` files brace-parse.) | Expression-bodied `fn ... = expr` form OR external Rust authority |
| **2** | Record literals inside `data` bodies | `data foo: T = { field: ... }` — record literals don't lower for `data` value carriers | Hand-Rust trampolines (e.g., `analyze_symbolic_cost_dimension`) |
| **3** | Lens-fold prereq #3 (per `design-lens-fold-prerequisites.md:71`) | Lens-fold-related substrate not authored due to grammar gap | Rust-side fold execution |
| **4** | Variant-constructor expressions | Per `design-lens-fold-prerequisites.md:155, 552, 556` — variant-constructor lowering NYI | Hand-Rust resolution paths |
| **5** | `Dimension<C>` data declarations | Per `design-cost-lens-sizevar-dimension-wiring.md:18, 175-176, 425-427` — `data symbolic_cost_dimension: AnalysisDimension<SymbolicCost> = { ... }` blocked on (#2) | `v3_compiler::analyze_symbolic_cost_dimension` hand-Rust trampoline |
| **6** | Name rosters (per `v3-validation-experiments.md:369`) | "class-5 gap #6" | (unclear from grep) |

## Hand-Rust trampolines that exist BECAUSE class-5 isn't closed

Each of these is documented with an explicit "when class-5 lands, this trampoline retires" dissolution trigger:

| Trampoline | Lives In | Retires When |
|---|---|---|
| `v3_compiler::analyze_symbolic_cost_dimension` | `src/v3/compiler/src/` | class-5 record-body grammar lands (per design-cost-lens-sizevar-dimension-wiring.md:185) |
| Multiple complexity-lens data decls (`data work_dimension` / `data span_dimension` / `data asymptotic_class_lattice`) | TBD | class-5 record bodies (per design-complexity-lens-behavioral-completeness.md:676) |
| `patterns.dag` substrate authoring | `dsl/std/` | class-5 parser gaps (per substrate-reflection-design.md:1654) |
| All 161 Unparsed std fns | various `dsl/std/*.dag` | class-5 brace-body parse (per dag.rs:1145-1152 + Brief 3 finding) |

## Downstream effects (the visible symptoms)

- **R3 retirement program stalls** because 58% of std fn bodies are Unparsed → not structurally executable → codegen drivers can't read them → can't author `.dag`-driven retirement
- **Lens framework substrate** is partially landed but trampolined through Rust execution at multiple sites (cost lens, complexity lens, dimension framework)
- **patterns.dag substrate** exists in design but can't author cleanly
- **Hand-Rust grew 1.21x v2 size** (per the v2 vs v3 ratio earlier; substantial cause is class-5 trampolines and Unparsed-body Rust mirrors)

## Why M2 didn't land

Grep doesn't surface a single "decided not to do M2" point. The pattern appears to be:
1. M1(2.7) ships with named scaffold (Unparsed, trampolines) + dissolution trigger pointing at M2
2. Pressure to land features → workers build AROUND the M2 hole rather than closing it
3. New features add new trampolines + new Unparsed bodies, ALL pointing at the same M2 dissolution
4. The dissolution trigger becomes "everyone is waiting for class-5" → nobody is actively closing it
5. Class-5 itself becomes a structural bottleneck that no single feature owns

This is the same dynamic operator named: "we defer something to M2 because its hard/delays progress - but then we keep working and accumulate debt."

## Recommendation for confidence-building attack

Of the 5 M2 DB items + 6 class-5 sub-gaps, the smallest tractable closure with the highest leverage:

### Recommendation: **DB-10 — `data` value semantics**

**Why this one**:
- **Smallest**: doc explicitly tagged "size S"
- **Carrier already exists**: per `design-m2-feature-parity.md:22`: `Declaration.value_body: Option<ValueBody>` (`src/v3/compiler/src/dag.rs:122`) already carries the lowered record-literal fields and scalar literals. `ValueBody::Structural { fields }` and `ValueBody::List/Map/Record/Variant` variants exist.
- **Gap is downstream consumer**: "But nothing downstream reads `value_body` back — emission treats every identifier reference as an opaque pointer to a `Declaration`, never inlines the carried value."
- **Closes class-5 sub-gap #2** (record literals inside `data` bodies usable at compile time)
- **Unblocks several trampolines** (`analyze_symbolic_cost_dimension` + complexity-lens dimension decls + dimension framework writ large)
- **Demonstrates the pattern works**: if we can close DB-10, the path to closing DB-11/12/13 + class-5 brace-body becomes concrete

**What it doesn't unblock directly**:
- The 161 Unparsed std fns (those need class-5 brace-body parse, which is sub-gap #1 — separate work but parallel to Brief 3)
- Full M2 feature parity (still need DB-11/12/13)

**Estimated effort**: 1-2 weeks if the design doc is accurate; longer if implementation surfaces additional gaps.

### Alternative attack: class-5 brace-body parse for `.dag` files

**Why this might be the right one instead**:
- **Biggest single lever**: closes 161 Unparsed std fn bodies at once
- **Already partially investigated**: bright-swift-668 (Brief 3) found the gate at `parse_generated.rs:727-758`
- **Coordinates with ongoing Brief 3**: subsumes the narrow char_in_class rewrite into a general fix

**Why I lean DB-10 instead for confidence-building**:
- DB-10 has cleaner scope (substrate carrier exists; just need consumers)
- Brace-body parse is a more invasive parser change with broader ripple
- Confidence-building is better served by a TRACTABLE close than the biggest lever

## What I'd want from operator

Pick one:
- **(A) DB-10 data value semantics** — recommended; clean confidence-builder; ~1-2 weeks; unblocks dimension framework + cost-lens trampoline retirement
- **(B) class-5 brace-body parse on `.dag` files** — bigger lever but invasive; coordinates with Brief 3 worker bright-swift-668 (we'd merge their work into this); ~1-3 months; unblocks 161 Unparsed std fns
- **(C) something else** — name which deferral pit you want to attack first

Either choice: I'd then dispatch a worker brief directly under `deep-wolf-155` (similar to Briefs 1-3) so we surface implementation findings as they arrive.
