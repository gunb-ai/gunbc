# PLAN — nominal types as the first consumer of the coercion frontier; delete refinements

**Status: PLAN, revision 4 — design only. Do not re-enqueue.** Revision 3 (one declaration modifier
per point in the visibility plane, plus a `derives` list) was approved and then withdrawn by the
operator on 2026-10-02: too many keywords, and the nominal question is really a coercion question.
This revision recasts it as the **first consumer of the coercion frontier** that
[v2 compiler architecture](v2-compiler-architecture.md) §1.4 and its frontier line ("Coercion and
named conversions") name. §7 places revision 3 (option A) side by side with this model.

Governing sections: DESIGN §4 (operations from inhabitance; the coercion ruling, `bool_indicator`),
§3 (replacement migrations), §4b, §5; architecture plan §1 (completion law, residual addressees),
§1.4.

Every count is **provisional** — line greps at main `dd0614551b5`, not the census — and is not a size
(DESIGN §6). M0, the resolved census (`gunbc.instruments.type_declaration_use_census`, being built by
tidy-koi-264), is the only admissible size.

## 1. What stays as ruled

- `type X = Y` is a transparent alias: X and Y are the same type.
- Refinements (`where <pred>`) are deleted. Every invariant becomes a checked construction one layer
  down.
- No typecase, and no conditional design for one. Matching is on constructors and closed sums.
- M0, the resolved semantic census, sizes the migration.
- The cutover is one atomic, generated PR that deletes `where`, `brand`, `nominal_opaque`, the v1
  spelling matches and hand tables, `gunbc.where_refinement_predicate_vocabulary` with
  `WherePredicateGrounding`, and #12506's parked row together (the full consumer list is revision 3 §6, commit 97dea7c55fd, and stands unchanged).
- Checked construction returns `Result` residue plus a typed cause (`std.error_primitives` `Result`).

## 2. The model: identity on the declaration, everything else is a route

**Declaration side: two forms and zero new words.**

| form | meaning |
|---|---|
| `type X = Y` | alias. Equality, no routes. |
| `type X { value: Y }` | a new type. Records are already nominal in v2 (`v2.compiler.infer` `infer_record_construct_type`), so a one-field record is the whole declaration of a nominal. |

The brand (`where brand("…")`), the refinement and `nominal_opaque` stop being declaration facts.
What distinguished them was never the type. It was **which conversions exist between X and Y, and who
may use them**, and those are conversion routes.

**Routes.** A conversion route is a declared function between X and its representation. Each of the
three old modes is a set of routes:

| old mode | Y → X (in) | X → Y (out) |
|---|---|---|
| brand | total route, public | exact route, public |
| refinement | **partial** route `Y -> Result<X, R>`, public; the total raw constructor confined | exact route, public |
| opaque | confined to the declaring module | **no exported route** |

The in-route is the constructor. Confining it uses the mechanism the corpus already has:
`sole_constructor` with `admit_callers`, which `std.bignat`, `std.allocation_roster`, `v2.compiler.normalized_tree`
and others already use. The confined raw constructor plus a public checked function **is** the
refinement. No new word is needed for it.

**Why no route is implicit.** Each route fails DESIGN §4's implicit-safety test, for a different
reason:

- `Y → X` adds meaning (it asserts that a String *is* a RoadmapNodeId). That is not
  semantics-preserving, so it is never silent. When it is partial (`Int → Port`), it also requires
  proof.
- `X → Y` (`RoadmapNodeId → String`) is exact and information-preserving, but it **changes
  interpretation**: the meaning is dropped. Under §4 that refuses implicit coercion, and under the
  architecture plan §1 it is a **residual addressed to the call site**. The developer answers it by
  naming the route, which is the operator's "might be valid, but the developer might need to
  coerce/acknowledge it". This is DESIGN §4's `bool_indicator` case exactly: `true as Int` refuses
  unless it names a declared route.

So the coercion fold's three outcomes (architecture §1.4) apply to nominals with no special case:

| crossing | outcome |
|---|---|
| alias `X = Y` | silent (`Identity`) |
| `X → Y` with an exported route | residual to the call site, answered by naming the route |
| `Y → X` | residual to the call site, answered by calling the constructor (or the checked route) |
| `X → Y` with no exported route (opaque) | residual to the **upstream module**: the module never offered one |
| two exported routes for one crossing | residual (`AmbiguousTargetCandidate`): never a pick |

**The one remaining word, if any.** `sole_constructor` confines *in*. Nothing in v2 confines *out*:
`x.value` is a field read, legal from any module, so a field read bypasses "no exported route". There are two
ways to close that, and choosing between them is decision O2:

- **(i) zero words.** Reading the representation field of a `sole_constructor` record outside its
  module is itself an `X → Y` crossing, so it must name an exported route. This changes every existing
  `sole_constructor` carrier that is read from outside, so M0 must count those sites first.
- **(ii) one word.** A modifier in the existing `nominal_opaque` slot (`type X opaque { value: Y }`)
  confines field reads to the declaring module. `nominal_opaque` is deleted and this word replaces it.

## 3. Operations from inhabitance, without a `derives` list

Equality, hash and ordering are not routes. The value never becomes a `Y`. They are operations X
**inhabits** (DESIGN §4: operations come from inhabitance), and inhabitance can be derived from the
routes rather than listed:

- An **exact, injective** exported route `X → Y` is an embedding. An embedding reflects equality
  and hash, so `X` inhabits `Y`'s equality (`std.algebra` `algebra_profile_equality_extensional`) and
  `Y`'s `std.content_hash` `HashFamily` by pullback. `RoadmapNodeId == RoadmapNodeId` works.
  `RoadmapNodeId` where `String` is declared still refuses, because the operation pulled back and
  the value did not cross.
- **Ordering** pulls back just as lawfully, but it may be meaningless (IDs ordered by spelling).
  Whether order pulls back by default is decision O3.
- **Display and encoding** are not pullbacks. They are crossings *out* (to text or bytes). They need
  the exported route, so an opaque `Secret`, which has no route out, has no display and no JSON by
  construction. No list and no special case are needed. That is the property revision 3 needed a
  `derives` opt-in to protect.
- **An opaque type that still needs equality** (constant-time `Secret` comparison) gets it from a
  function its declaring module writes. That is an ordinary operation, not a capability vocabulary.

So **no `derives` list is needed**. What an author controls is which routes the module exports, and
the operations follow from them. The single residual case is O3, and it is a default, not a list.

## 4. What `v2.std.coercion` must gain (model-first)

What exists today (`v2.std.coercion`, `v2.std.inhabitance`, `v2.std.find_witness`):

- the coercion result is `CoercionResult { target, quality: Identity | Exact | Widened, witness }` or a
  typed `CoercionMismatchKind = NoTargetCandidate | AmbiguousTargetCandidate | StructuralMismatch |
  WouldLoseInformation`;
- candidate selection is `find_witness` under `TargetSelectionPolicy = TargetDeclaredPriority |
  UserSelected`;
- position-aware inhabitance is `declared_type_inhabitance` over `DeclaredTypePosition`;
- value-set widening is `v2.std.refinement_widening_predicate`. That handles representation widening
  such as `rust i32 → python int`. It is **not** the `where` machinery and is kept.

In order, each step model-first (types and witnesses land before any stage consumes them):

1. **C1: the residual addressee.** The architecture plan §1.2 makes the addressee part of a
   residual's type: call site, root, upstream module or observer. No addressee type exists anywhere in
   `dag` or `src/v2` (DFS: none found). Completion and coercion are one procedure (§1.4), so the
   addressee is minted **once**, in a home both consume, never inside `v2.std.coercion` alone. The home
   is decision O4. `CoercionMismatchKind` gains its addressee by a total map from kind to addressee,
   not by a parallel field authored per site.
2. **C2: a mismatch kind for "exact, but changes interpretation".** Today an exact, lossless crossing
   that drops meaning has no kind. `WouldLoseInformation` is false of it and `StructuralMismatch` is
   wrong. Add `ChangesInterpretation`, whose addressee is the call site. This is the kind
   `RoadmapNodeId → String` and `true as Int` both produce.
3. **C3: the declared conversion route.** A route is a declared function, not a new declaration kind:
   any function in X's declaring module whose signature is `Y -> X`, `Y -> Result<X, R>` or `X -> Y`
   is a candidate route for that crossing, and the candidate set is closed by the module. That keeps the
   declaration count at zero, and it is decision O1. The route carrier records source, target,
   totality (`Total` or `Partial { refusal: R }`) and the quality the fold already computes. A
   `ChangesInterpretation` residual names its candidate routes in its diagnostic, so the developer is
   told the answer. With two candidates it is `AmbiguousTargetCandidate`.
4. **C4: route visibility.** For the in-route, v2 must **enforce** `sole_constructor` and
   `admit_callers`. Today v2 parses them, and the seed enforces them (`v1.compiler.infer`
   sole-constructor construction diagnostics), but on the native route they are dropped:
   `gunbc.rung_drop.admit_callers_discarded_on_the_native_route`. Closing that drop is a precondition,
   not new design. For the out-route, see O2.
5. **C5: operation pullback.** `v2.std.inhabitance` admits X into an algebra Y inhabits when exactly
   one exact injective route `X → Y` is exported (§3). The judgment cites the route, so the
   inhabitance witness carries its own provenance.
6. **C6: the first instrument**, as the architecture frontier names it: one core module emitted to the
   Rust and TypeScript targets, with the per-target mismatch list. For this program, the M0 census
   rows are what that list is checked against.

Then the migration (generated from M0, as before): brand → one-field record + public in-route + public
out-route; refinement → `sole_constructor` record + public checked route + public out-route;
`nominal_opaque` → per O2; aliases untouched; introduction casts → constructor calls; workaround casts
(strip to compare, hash or encode) → the pulled-back operation directly; true raw consumers → the named
out-route.

## 5. Controls (expecting red first)

The six arms of `test.claim.brand_nominal_identity_witness_test` are kept and their refusals
preserved, re-spelled over records and routes:

- **COLLISION / MISMATCH:** refuse, unchanged.
- **CONSTRUCT / STRIP:** accept, via the in-route and the out-route.
- **DUAL:** accept.
- **UNDECLARED_SITE:** refuses at parse.

These controls are added:

| control | expected after | today |
|---|---|---|
| `RoadmapNodeId` at a declared `String` with no route named | residual `ChangesInterpretation`, addressee call site, names the route | refuses with no addressee: **red** for the addressee |
| same, route named | accept | n/a |
| `Port { value: 0 }` outside the module (`sole_constructor`) | refuse on the **v2** route | **red** (rung drop `admit_callers_discarded_on_the_native_route`) |
| `port_of(n: 0)` | `Err` with residue `0` and a cause; `port_of(n: 80)` advances; by execution | positive control |
| `Secret` displayed or encoded, no exported route | refuse, addressee upstream module | **red** |
| `RoadmapNodeId == RoadmapNodeId` | accept by pullback (C5) | depends on today's leniency; M0 measures |
| two exported `X → Y` routes | `AmbiguousTargetCandidate` | **red** |
| `true as Int` without `bool_indicator` | `ChangesInterpretation`, call site | refuses with no addressee |

None of these retires (DESIGN §4b(4)).

## 6. Rungs (DESIGN §4b) and §4's coercion law

- **Identity (X vs Y, X vs sibling):** structurally guaranteed on both models. Records are nominal.
- **Invariant (`Port ∈ 1..65535`):** structurally guaranteed only when the raw constructor is confined
  **and** that confinement is enforced on the v2 route. Until `admit_callers_discarded_on_the_native_route`
  closes, the v2 route's rung for every invariant is mechanically preventable at best, and the plan
  must say so per type rather than claim it.
- **Opacity:** guaranteed only after O2 lands. Before that, a field read leaks.
- **Coercion law:** unchanged and strengthened. No nominal crossing is implicit. Every one is a residual
  with an addressee, answered by a named route. The only silent route is the alias's identity. "A brand
  is cosmetic until construction enforces it" (§4b) becomes "a nominal is cosmetic until its routes'
  visibility is enforced", and C4 plus O2 are exactly that enforcement.

## 7. Revision 3 (option A) against this model

Population figures are provisional. They are not a size, and M0 sizes both.

| | option A (rev 3) | conversions (rev 4) |
|---|---|---|
| **new words** | `nominal`, `sealed`, `opaque`, `derives(…)` = 4 | 0 (O2-i) or 1 (O2-ii `opaque`). `sole_constructor`/`admit_callers` already exist |
| **TrueAlias** | untouched | untouched |
| **UnconstrainedNominal** (old brand) | `type T nominal = Y` | `type T { value: Y }` + in/out routes |
| **CheckedConstructionStage** (old refinement) | `sealed` + checked ctor | `sole_constructor` record + checked route |
| **Opaque** | `opaque` | O2 |
| **introduction sites** (~16.6k `"…" as T`, provisional) | rewritten to a constructor call | same rewrite, generated; identical burden |
| **projection sites** (~9.3k `as String\|Int\|Nat`, provisional) | rewritten to `.value` | rewritten to the named out-route; per site, a function call rather than a field read. **Workaround sites** (compare, hash, encode) **vanish** under C5 pullback instead of being rewritten, so this model's burden is lower by exactly M0's `OperationWorkaround` count |
| **capabilities** | `derives(…)`: author-listed, a second vocabulary to keep in step with `std.algebra` | derived from exported routes; nothing listed; display/encode blocked on opaque by construction |
| **invariant rung** | structural once `sealed` is enforced | structural once `sole_constructor` is enforced on v2 (C4): **the same precondition**, already modeled and with a seed implementation |
| **`RoadmapNodeId` as `String`** | refuses; author projects `.value`, with no stated reason | residual to the call site naming the route: the acknowledgment the operator asked for |
| **needs that do not exist** | 3 modifiers + derives parser/infer, visibility semantics, capability derivation | C1 addressee, C2 kind, C3 route carrier, C4 enforcement on v2, C5 pullback, O2 (if ii) |
| **reusable beyond nominals** | no | yes: C1–C3 are the coercion frontier the architecture plan already owes (`bool_indicator`, target emission mismatches) |

The deciding row is the last one. Option A built a nominal-only mechanism next to an unfinished
coercion model. Revision 4 finishes the coercion model, and nominals fall out of it. That is §2's
test that net concepts must not grow by re-invention: `sealed` was a re-invention of
`sole_constructor`, which the revision 3 DFS missed.

## 8. Operator decisions

- **O1: routes are recognised by signature in the declaring module** (no route keyword, no route
  table). Recommend: yes. The candidate set is closed by the module, and two candidates refuse as
  ambiguous rather than being picked.
- **O2: out-route confinement.** (i) a field read of a `sole_constructor` record outside its module is
  a crossing that must name a route; or (ii) one modifier, `opaque`, in `nominal_opaque`'s slot.
  Recommend: **(ii)**. (i) silently retypes every existing `sole_constructor` carrier's public reads,
  which is a meaning fork over 1k+ existing uses. Choose (i) only if M0 shows the outside reads are
  few.
- **O3: ordering pulls back by default?** Recommend: **no**. Equality and hash pull back. Ordering pulls
  back only if the module exports an order-preserving route, which is spelled as the same signature
  rule over `Ordering`. A spelling order on IDs is not a fact anyone asserted.
- **O4: home of the residual addressee.** It is shared by completion and coercion (architecture §1.4).
  Recommend: a new `v2.std.residual` module consumed by `v2.std.coercion` first. Its second consumer
  is completion when §1.5's elaboration lands, which is a stated frontier.
- **O5: spelling of "answering" a route at a use site.** Recommend: **the function call**
  (`roadmap_node_id_text(id)`). That adds no syntax. `as <route>` would be a second spelling of the same
  call.
- **O6: sequencing.** C1–C5 land before the generated cutover, and C4 (closing the native
  `admit_callers` drop) is first because every invariant rung depends on it. M0 continues independently
  under either model. Recommend: as listed.
