# Design memo: Carrier as Its Own Core Axis? (model_core, #4585 follow-up)

> **Status: DESIGN MEMO — decision criterion + measure-first front door.** Small model
> question from the COERCE-lane follow-ups: should "carrier" become a sixth
> `ModelCorePrimitiveFactAxis`, or fold into `Encoding`? Answer shape: **neither, until a
> census says otherwise** — the criterion below decides it mechanically.

## 1. The question, grounded

`ModelCorePrimitiveFactAxis` (`src/v2/std/model_core.dag:38`) is the closed core-fact
vocabulary: `Width | Signedness | Range | Encoding | OverflowDisposition` (core) +
`SurfaceSpelling` (surface). R3 (`project_to_core`, `src/v2/std/project_to_core_predicate.dag`)
compares fact-bundles **on core axes only** — surface facts drop out of the projection.

The *carrier* — which substrate `Node` a fact-bundle grounds to — is today a **field**, not
an axis: `PrimitiveFactBundle.substrate_carrier: Node` (`model_core.dag:62`). So
`project_to_core` drops it: two bundles with identical five-axis facts but different
`substrate_carrier`s compare core-equal. The #4585 R3 work (cross-surface sibling derive,
"compare on the semantic core") surfaced the question of whether that is a feature or a hole.

## 2. The decision criterion (this is the design)

Apply Step 2 of the substrate-fact introduction procedure (coordinates are facts a single
inhabitant carries *independently*) plus P2 single-authority:

- **If the five core axes functionally determine the carrier** (in our closed system: width
  + signedness + range + encoding + overflow ⇒ exactly one canonical substrate carrier),
  then carrier-as-axis would be a **duplicate authority** — the same fact stated twice, with
  drift available (a bundle whose width says 64 but whose carrier says `Word32` — which
  wins?). The faithful move is the opposite of adding an axis: a **coincidence obligation**
  (the Practice 8 *coincide* bar, `docs/modeling/grounding-worked-examples.md`): a
  structural check that `substrate_carrier` reduces to the canonical Node the axis facts
  determine. Carrier stays a field; the check makes it *provably redundant* rather than
  silently trusted.
- **If a genuine pair exists** — two bundles, identical core-axis facts, **non-coincident**
  carriers (the live candidate class: composite/collection carriers, where scalar axes
  underdetermine shape — e.g. element facts identical but `List`-backed vs `Set`-backed) —
  then carrier is an **independent coordinate** and earns the sixth axis:
  `ModelCoreFactAxisCarrier {}`, added to the closed coproduct with the role predicates
  (`model_core_primitive_fact_axis_is_core`) updated in the same change (one file — the
  closed coproduct is why this is cheap).

**Folding carrier into `Encoding` is rejected on either branch.** `Encoding` is a specific
spec fact (two's-complement, UTF-8, IEEE-754 layout). Overloading it with carrier identity
compresses two facts into one coordinate — the exact compression Step 2 exists to catch, and
it would make both facts unreadable individually downstream (which is how the next heuristic
gets born, P1).

Note what carrier-as-axis is **not** for: nominal distinctness. `UserId ≠ AccountId` over the
same carrier is the brand channel's job (`binding_id`, A3) — brands participate in identity
through their own side channel, never through core facts. Any proposal reaching for a
carrier axis to fix a brand-shaped problem is mis-routed.

## 3. Front door: measure first

One census, cheap, decisive: sweep `extdeps/` + `std/` fact-bundles for a pair with
**identical core-axis facts and non-coincident carriers**.

- Pair found ⇒ sixth axis (the pair is the discriminating fixture for its claim).
- No pair ⇒ keep five axes; land the coincidence obligation instead, with one claim:
  **green** on a real bundle (carrier reduces to the axis-determined canonical Node), and
  the discriminating **red** — a perturbed bundle (width 64, carrier `Word32`) is rejected,
  proving the drift the obligation closes is detectable.

Prior expectation, stated for honesty: in a closed system where we authored both sides, the
five-axes-determine-carrier branch is the likely one for scalars (P1's "heuristics are never
structurally necessary" cuts the same way for redundant coordinates) — and the collection
question, if real, may be better answered by collection fact-axes (element + cardinality
shape) than by a raw carrier axis. Let the census say.

## 4. Consumers (E-10)

Either branch lands with executing consumers that exist today: the R3 `project_to_core`
claims and the #4585 sibling-derive corpus (axis branch: the new axis must change a
previously-core-equal verdict, shown red→green; obligation branch: the perturbed-bundle red
above). No new machinery without one of those claims in the same PR.

## 5. Out of scope

- Brand/nominal identity (A3/A4 channel — see note in §2).
- Catalog-axis widening for `target_model` (G.1 note at `model_core.dag:37`) — separate
  vocabulary, same procedure when it arrives.
- Any change to `project_to_core`'s fold mechanics — both branches only touch the axis
  vocabulary or add a predicate; the fold is untouched.
