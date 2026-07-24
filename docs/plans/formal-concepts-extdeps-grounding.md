# Formal concepts derive from extdeps — std is a bootstrap seed + ergonomics

**Operator directive, 2026-07-24** (recorded from the namespace-flip conversation that surfaced it).
The concrete trigger is the `cardinality` ↔ `termination` fork the §13 flip exposed (below); the
directive generalizes past it.

## The correction

The current layering places the *universal formal frameworks* in `std/` (DESIGN §3: "std keeps only
the frameworks they build on — `measure`, `types`"). The operator's refinement — *"I think I made a
mistake early on"* — splits that:

- **`std/` bootstrap seed** — the irreducible core needed to *express* a formal definition at all:
  primitives, `Int`, `List`, and their kin. Small on purpose.
- **`extdeps/` — the formal definitions** of concepts, grounded in and **cited to** their real
  upstream sources (the mathematics: well-founded relations and ranking functions for termination,
  ordinals/counting for cardinality, the induction principle, graph theory). This is the *formal*
  layer — model what the concept **is**, faithfully, with a citation, exactly as `extdeps/` already
  does for service APIs and hardware.
- **`std/` ergonomic definitions** — re-exports/wrappers over the extdeps formal concept, shaped for
  *actual programming* rather than for formal fidelity.

So the dependency reads **std-seed ← extdeps-formal ← std-ergonomic ← compiler**. This refines the
strict layer DAG (`std ← extdeps ← …`) by recognizing that `std` is really two sub-layers with
`extdeps` between them — the seed everything needs, and the ergonomic surface built *on top of* the
cited formal layer.

This is DESIGN §1 taken one turn further — *reduce convention to necessity until nothing arbitrary
survives*. A concept coined std-native is a convention standing where a citation was available; a
concept **derived from a cited formal source** is necessity. It is the same move §3 already makes for
domain models ("a CPU/DRAM model … lives downstream of std in `extdeps/`") and the same move the
[axiom-syllogism thread](../../DESIGN.md) makes for A1–A3 (*model what an axiom **is**, in extdeps,
first*) — now applied to the formal frameworks themselves.

## Worked example — `cardinality` ↔ `termination` (the flip's trigger)

The namespace §13 flip (NamespaceOnlyY) refuses a bare name with two reachable definitions. Beyond
the four two-std forks it was scoped for (`Set`/`Map`/`Byte`/`Char`), the flip's **witness corpus**
oracle surfaced a fifth: `RankingDimension` and `TerminationProof` are each defined **twice**, with
**different models**:

| | `dag/std/termination.dag` (foundational) | `src/v2/std/cardinality.dag` (downstream) |
| --- | --- | --- |
| `RankingDimension` | closed enum of measure **kinds**: `TreeSize \| ListLength \| ArithmeticValue \| TokenPosition \| SetCardinality` | record `{ measured: Symbol }` — a **specific** measured quantity |
| `TerminationProof` | `{ dimensions: List<RankingDimension> }` | `{ non_increasing: List<RankingDimension>, strict: RankingDimension }` |
| used by | `std.graph`, `std.induction`, `std.computation`, **and `v2.std.cardinality` itself** | the v2 compiler (`04_infer`, `05_eval`) + ~60 modules |

The first reflex — "consolidate: 60 consumers beat 4, so fold `termination` into `cardinality`" — is
**backwards**, and the operator caught it. `termination` is the *foundational* module (it owns
`DescentEvidence`, the descent lattice, `PositiveDescentAmount`, `ProportionalDivisor`) and
`cardinality` **imports it**. A foundational module cannot fold into its own downstream consumer.

Nor is it a mechanical redirect: the two `RankingDimension`s may not even be the *same concept* — one
is *what kind* of thing descends (a closed taxonomy), the other is *which* thing descends (a symbol).
Reconciling them is a real modeling decision (merge onto one model, or decide they are distinct and
**rename** apart), on load-bearing foundational modules.

The right resolution per this directive: **neither std-native model wins.** Both re-ground on a cited
`extdeps` formal source — termination as **well-founded descent / ranking functions** — from which a
`std` ergonomic surface derives. A ranking *dimension* (the axis) and a ranking *measure* (the value
along it) then have a single grounded home, and the fork dissolves because there is one authority to
point at, not two conventions.

## The family this covers

The formal-concept `std` modules that should re-ground on cited `extdeps` foundations — the ones the
example above touches or sits beside:

- `std.termination` — well-founded relations / ranking-function termination analysis
- `v2.std.cardinality` — counting / ordinals (and its `TerminationProof` folds into the above)
- `std.induction` — the induction principle / structural recursion
- `std.graph` — graph theory
- `std.computation` — reduction / evaluation order

Each is currently a std-native coinage; each has a real, citable formal source. *"All of this is up
for grabs"* (operator) — the placement is not sacred.

## Status & sequencing

- **The flip does not wait on this.** Namespace-flip Dispatch 1 (#7178) **qualifies** every ambiguous
  witness ref inline (`v2.std.cardinality.TerminationProof`, etc.) — the operator's inline ruling —
  which greens the flip without prejudging the modeling decision. No `dag/std/*` is touched.
- **This re-grounding is a deferred de-fork/modeling lane**, sequenced after the flip. It most
  naturally couples to the body-lowering lane, which already owns `dag/std/termination.dag` as
  `DescentEvidence`'s single authority, and to the [axiom-in-extdeps thread](../../DESIGN.md).
- **Dissolution trigger:** each formal concept lands a cited `extdeps` definition with a `std`
  ergonomic re-export; the duplicate std-native `type` is deleted and the qualified refs re-point to
  the single authority. When the family is grounded, this doc dissolves into the carriers (§6 — the
  mark on the carrier is the authority, not a parallel-ledger doc).

## Cross-references

- [namespace-only resolution design](namespace-resolution-design.md) — the flip lane
- [namespace flip · last 28 → 0 + Post-0 roadmap](namespace-flip-last-28-root-a-two-std-defork.md) —
  where the fork was found by execution
- [body-lowering design](body-lowering-design.md) — owns `dag/std/termination` (`DescentEvidence`)
- DESIGN §1 (reduce convention to necessity), §3 (single authority; extdeps models the real upstream)
