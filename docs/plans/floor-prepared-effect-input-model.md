# Floor: a prepared effect input carried into a warm row (PR two of the #10994 chain) — Phase 1 model

Status: MODEL ONLY, stop-before-code (lane `warm-heron-775`, leaf of `eager-raven-113`, root
`fierce-lark-661`). Nothing below is realized; the types are the proposal and the reviewers'
subject. PR one is gunbc#11303 (`quiet-newt-362`); PR three is #10994's enrolment.

## 0. What the demand actually looks like after PR one

Read off `origin/session/quiet-newt-362` rather than from the brief. Every live consumer now
spells one shape, inlined at the call site (`gunbc.roadmap_authority`, `roadmap_page`,
`roadmap_serve`, `roadmap_spawner`, `roadmap_forecast`, `roadmap_belt_actuate`,
`roadmap_site_surface_observe`, `roadmap_served_observation`):

```
roadmap_authority_projection(history: roadmap_acceptance_event_history_load())
```

- `roadmap_acceptance_event_history_load()` is NULLARY and EFFECTFUL: one `Filesystem.Read` of
  `roadmap_acceptance_event_history_carrier_repo_path` (17 lines, ~11 KB), then a host-builtin
  JSONL parse, yielding the typed coproduct `RoadmapAcceptanceEventHistoryLoad`
  (`Loaded { events }` | `LoadRefused { detail }`). It declares no `uses`, so the seed routes it
  through `eval_pure_named_call`; the read bumps `effect_dispatch_count`, so the ordinary fold
  path never stores it (correct today).
- `roadmap_authority_projection(history)` is UNARY and PURE — the ~186 ms fold the brief names —
  demanded by seven required-lane claims plus #10994's witness, each in its own claim frame.

So the one shared identity across all claims is already `(roadmap_authority_projection,
args = {history: <content>})`, and the cross-claim tier's existing key `(fn_node, args_hash)`
ALREADY covers the content of the effect input: `args_hash` is a content hash of the parsed
history. What is missing is not a key — it is a FILL DISPOSITION and an ACQUISITION CARRY:

1. the unary fold is claim-shaped, so under today's two row kinds it can only be CLAIM-FORCED —
   filled inside the fold by whichever claim touches it first, the nondeterministic charge the
   roster exists to delete, and interrupt-prone against the 500 ms line;
2. even if the fold were served, every claim would still re-acquire the input (read + parse +
   hash) to FORM the key — the rostered class `shared_precondition_re_derived_once_per_claim_frame`
   at a smaller grain.

## 1. Where this sits in the materialization model (DESIGN §2, §3b; `std.materialization_ladder`)

- Demands: N `FrameDemand`s with identity `roadmap_authority_projection` whose sites are sibling
  claim frames (`IsolatedChildrenFrame` — fresh `InterpContext` per claim, by design) under one
  `SharedStateFrame` (the floor process, whose preparation is the shared ancestor).
- Ladder rule (2): the LCA is an isolation boundary, so the redundancy cannot be rewired inside a
  claim and is a MATERIALIZATION OBLIGATION at the LCA, discharged by a covering provider whose
  key represents every declared input. `std.realization.Materialization` verdict: `Share`
  (carry the first value at the ancestor) — not `Memoize`.
- Provider: the existing cross-claim tier (`v1_interpreter` cross-claim pure share) is a
  `MemoTier { keying: ContentKeyed }` with scope = the preparation frame, coverage =
  `CoversIdentities { roster }`, retention = entry cap + byte budget. No new provider.
- Nature of the effect input: NOT `WorldRead`. Inside the floor, a readonly `Filesystem.Read` of
  a path the single authority `v1_interpreter::hermetic_checkout_input_disposition` confirms is a
  committed checkout input is INPUT ACCESS, not a host effect ("the commit IS the run's injected
  input"). So the prepared subject is widened, honestly, to include the CHECKOUT INPUTS a rostered
  acquisition reads — content-identified exactly as `.dag` module sources are — and the fold is
  `PureComputation` over that widened subject. That is what makes "nullary from the row's point of
  view" true rather than asserted: "a rostered producer's value depends only on the prepared
  subject's content" (the roster header's own enrolment assertion) still holds verbatim.
- Identity shape: `std.materialization_provider.DeclaredInput { ref, digest: Fnv1a64Structural }`,
  one per checkout path read, derived by a fold over the bytes read — NEVER caller-asserted
  (`materialization_provider_derivation_invariant`: derive from the request variant, a fold over
  carried parts, or a read-back; nothing else).

## 2. The carrier

### 2a. Roster row (home: `v2.workflow.floor_pure_producer_share`)

The warm roster becomes a coproduct rather than a third `List<String>` beside two others — one
concept (a preparation-forced row) with one real distinguishing fact (does it carry a prepared
effect input), so the two existing collision walls keep covering it through
`floor_cross_claim_admitted_producers()`:

```
type WarmShareRow
  = NullaryWarm { producer: String }
  | PreparedEffectInputWarm {
      producer: String       // the PURE fold; must take exactly ONE parameter (derived, refused otherwise)
      effect_input: String   // a NULLARY acquisition whose only effects are confirmed checkout reads
    }

data floor_cross_claim_pure_producers_warm: List<WarmShareRow> = [ NullaryWarm { producer: "..." } x28, ... ]
```

The 28 existing string rows migrate mechanically to `NullaryWarm { producer }`; the decoder
`floor_decode_module_prefix_roster` for this roster is replaced by a coproduct decoder (root
cut-over, DESIGN §3 replacement migration). The roadmap row:

```
PreparedEffectInputWarm {
  producer:     "gunbc.roadmap_authority.roadmap_authority_projection",
  effect_input: "gunbc.roadmap_authority.roadmap_acceptance_event_history_load",
}
```

Both identities are admitted to the tier (the acquisition is served too — §2c). The parameter
binding is DERIVED from the producer's declaration (arity exactly one), never authored in the row.

### 2b. The prepared effect input (home: `v2.workflow.floor_preparation`)

```
type PreparedCheckoutInput
  = PreparedCheckoutInputPresent { input: DeclaredInput }      // ref = repo-relative path, digest = fnv1a64 over bytes read
  | PreparedCheckoutInputAbsent  { ref: NonEmptyStr }          // commit-deterministic absence, its own identity

type PreparedEffectInput {
  acquisition: String                    // the rostered nullary acquisition
  inputs: List<PreparedCheckoutInput>    // every checkout read the acquisition dispatched, in dispatch order
}
```

`floor_preparation` already imports `std.materialization_provider` and owns the prepared subject's
identity; this is the same module gaining the non-module checkout inputs the subject depends on.
Whether `PreparedClaimSubjectIdentity` folds these digests in is deliberately NOT changed in PR two
(the tier key carries them — §2c); it is named here as the obvious next fold.

Seed side: the preparation holds one table `path -> PreparedCheckoutInput` bound while the
acquisitions warm (a thread-local beside `PURE_PRODUCER_SHARE_ROSTER`, for the same reason).

### 2c. How preparation binds it, and how a claim is served

At `install_pure_producer_share`, for each `PreparedEffectInputWarm` row, in order:

1. Resolve both spellings by declaration identity in their module frames (existing arms:
   `PureProducerShareProducerModuleOutsideSubject`, `PureProducerShareProducerUnresolved`).
2. Warm the ACQUISITION in a Hermetic frame with an "acquisition in flight" frame open. Inside it,
   `eval_service_call`'s existing checkout carve-out is the ONLY effect admitted: each confirmed
   `Filesystem.Read`/`List` records `PreparedCheckoutInput` (digest of the bytes, or Absent) into the
   preparation table; any other dispatch refuses the warm (§3). The value is stored in the tier
   under key `(fn_node, args_hash = ∅)` PLUS the recorded inputs — the entry carries its
   `List<PreparedCheckoutInput>` and is servable only while every one verifies against the
   preparation table. This is the "content identity + roster identity" key the brief asks for;
   a nullary effectful value is never stored content-blind.
3. Warm the PRODUCER by calling it with the ONE argument bound to the acquisition's value, under
   the existing fill-guard protocol, and store under the existing `(fn_node, args_hash(history))`
   key — no new key shape for the fold. Billed to preparation, one `SharedBuildObservation` per
   identity, through the one preparation refusal, as today.

At claim time nothing new happens: the consumer evaluates
`roadmap_acceptance_event_history_load()` → `try_cross_claim_pure_memo` hits the acquisition entry
(inputs verify against the table; zero reads, `effect_dispatch_count` untouched), then
`roadmap_authority_projection(history: <served Rc>)` hits the fold entry. Per-claim cost: two
O(1) serves. Ledger: both under `cache=cross_claim_pure_share`, provenance built-by-preparation.

## 3. Refusal arms — every one typed, located, line-stopping; none widens

| cause | fires when | rung |
|---|---|---|
| `PureProducerShareWarmDispatchedEffect` (NEW, and it closes a hole that exists TODAY) | a `NullaryWarm` row dispatched any effect during its warm | today `warm_cross_claim_pure_producer` stores WITHOUT the `effect_dispatch_count` guard the fold path has, so rostering `roadmap_acceptance_event_history_load` as a plain warm row would silently store a checkout-read-dependent value under a content-blind key — an executing instance of the refused `KeyOmitsAnInputTheValueDependsOn` class. RED authorable now on a fixture. |
| `PreparedEffectInputNotCheckoutInput { operation, ground }` | the acquisition dispatched an effect the carve-out did not confirm (a write, a mock-served service, an out-of-root / `.git` / unresolvable path — the last three arrive as the existing `HermeticHostEffectRefused` grounds) | the carried value must be a function of checkout content only |
| `PreparedEffectInputAcquisitionReadNothing` | the acquisition completed with zero checkout reads | a misfiled row: it is a `NullaryWarm` producer and the two kinds must not answer for one identity (§3 meaning fork) |
| `PreparedEffectInputAcquisitionNotNullary` / `PreparedEffectInputProducerArityNotOne` | declaration shape | the binding is derived from arity; a shape that cannot be derived refuses instead of guessing |
| existing `PureProducerShareWarmNotStored` / `PureProducerShareWarmFailed` | store refused (not portable, byte budget, entry cap) or evaluation failed, for EITHER identity | unchanged |
| serve-time `prepared_input_stale` | an entry's recorded inputs do not verify against the preparation table | NOT a line stop: it can only arise from a second preparation in one process (the control in §4); the serve is refused and COUNTED in the ledger, and the claim recomputes exactly as if never enrolled |

Absent input is NOT a floor refusal: absence under the root is commit-deterministic (the carve-out
reads it OFF THE INPUT), so the carried value is the producer's own typed
`RoadmapAcceptanceEventHistoryLoadRefused`, keyed on `PreparedCheckoutInputAbsent`, and the claims
red with their own located cause. What the mechanism guarantees is that absence can never become
`Loaded { events: [] }` — never an empty default — and that an UNREADABLE input (outside the
root, `.git`, unresolvable) stops the line at preparation with the hermetic ground, rather than
surfacing seven times inside the fold.

## 4. RED / controls (Phase 2, listed now so the negative shapes are fixed before anything runs)

1. Seed unit, key soundness: prepare over content A, store; rebind the table over content B
   (same path, different digest); lookup refuses to serve and counts `prepared_input_stale=1`;
   positive control: rebind over A again, serves the same `Rc` (`Rc::ptr_eq`, the allocation-identity
   red the file already uses).
2. Seed unit, the closed hole: a `NullaryWarm` row over a fixture fn that dispatches one checkout
   read → `PureProducerShareWarmDispatchedEffect`; today's tree stores it (that is the RED).
3. Seed unit, misfiled row → `PreparedEffectInputAcquisitionReadNothing`; non-checkout effect →
   `PreparedEffectInputNotCheckoutInput`.
4. Floor, identical verdict: the seven roadmap claims (+ #10994's witness or its reduced
   reproducer) report the same verdict present-vs-absent, read from `required-floor-disposition`.
5. Floor, cost: one sha-pinned `workflow_dispatch` pair per the corrected procedure in the roster
   header (a PR pair measures two trees; a pushed branch triggers nothing), normalised on claims in
   no consumer module of any enrolled key, read over `required_floor_claim_cost.tsv` (eval_steps,
   host-independent) joined with `[floor-shared-fill]`. Admission is serve-below-recompute over the
   row's OWN consumers; the row lands first as a `PendingShareCandidate` naming that measurement and
   `refuses_if`, and moves to the roster with the receipt.

## 5. What this is NOT, stated so it cannot drift

- Not a 'warm with arguments' arm: no row carries an authored argument. The argument is the carried
  value of a rostered acquisition, and the binding is derived from arity.
- Not a memo: the acquisition happens once at the shared ancestor, and the per-claim serve does no
  read, no parse and no hash — the demand graph is minimized first (§2); the tier discharges the
  obligation the ladder's rule (2) creates.
- Not a fourth roster list: two warm KINDS in one coproduct, one claim-forced list, refused,
  pending — the collision walls unchanged in meaning.
- Not a staleness envelope: no TTL, no "file unchanged since" heuristic. The key IS the content
  identity; a changed input is a different key.

## 6. Open doubt for the reviewers (the one place I am not sure the up-front way is this one)

The REDUCED alternative: store only the fold (step 3), skip the acquisition entry (step 2), and let
each claim re-read + re-parse + hash ~11 KB to form the key. It needs no change to the tier's key
shape at all (the args_hash is the content identity), and the fill disposition is the whole
capability. Its cost is that the acquisition stays re-derived once per claim frame — the rostered
class at a smaller grain — and that the brief's "acquires the effect input ONCE" is then false. I
recommend the full form; if the key-extension (an entry carrying `List<PreparedCheckoutInput>`
verified against the preparation table) is judged too invasive for PR two, the reduced form is the
same model minus §2c step 2 and the acquisition's refusal arms, and step 2 becomes a declared
frontier with that sentence as its trigger.
