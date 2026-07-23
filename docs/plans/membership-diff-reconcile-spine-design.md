# Grain-agnostic membership-diff reconcile spine (R2/R3/R5/R9)

Status: LANDED — Pass 1 (spine + R5) and Pass 2 (live_deploy binding) implemented, reviewed pre-commit by calm-ferret-849 (seam) + royal-carp-451 (grain-agnosticism + R5 boundary). See "Pass 2 — LANDED" below for the shape as built (which refined the pre-commit design in three reviewed ways: derived-ownership projection, the `EffectsOrRefusal` sum wall, and the typed wholesale-refuse policy trigger).

## Objective (locked requirements, royal-carp 2026-07-17)

ONE grain-agnostic keyed membership-diff reconcile: **desired set vs observed set, keyed by member identity**. `Added → upsert`, `Removed → teardown`, `Unchanged → noop`. Member may be a service/unit/host **or** a Session (operator: "it should all be the same"). Teardown is **owned-only, fail-closed** (R5). First binding: unify live_deploy apply+retract into ONE diff-driven pass, green against the existing golden.

## Reuse map — single authorities I GENERALIZE, never fork (calm-ferret seam steer)

| Concern | Single authority (reused verbatim) | What's NEW |
|---|---|---|
| the diff | `std.change.keyed_two_way_diff<K,V>(left, right, key_eq, value_eq)` — already fully generic | supply `K=MemberKey, V=Member` |
| teardown generator | `std.change.keyed_invert_patch` (Added↔Removed) | — |
| ownership | `live_deploy/spec.dag DeploymentStep = Owned \| Ensured` (teardown-owned-only already a convention in `deployment_owned_steps_retract_order`) | lift the Owned/Ensured DISTINCTION to a member-parametric `Ownership` marker |
| apply/transport | `host_effect_apply` + `HostEffectTransport` (LocalShell/SshShell/EmitArtifactThenThinRun) | per-member-type apply/teardown DISPATCH (a Realization) |
| host upsert | `fleet_converge_apply.converge_apply_for_host` (verbatim, later) | — |
| fold skeleton | `converge_apply`'s fold-over-members-with-refuse-short-circuit | add a **teardown arm** (converge has upsert-only today) |

**Fork-traps avoided (calm-ferret):** (a) NOT forking `keyed_two_way_diff` per grain — one generic engine; (b) NOT collapsing membership grain and knob grain into one fold — they LAYER (membership is the outer consumer; host knob-converge stays a `K=ConvergeKnobKey` inner consumer, added later under a Host member); (c) NOT minting a new ownership enum — generalizing Owned/Ensured; (d) NOT coupling member-grain to transport — transport is the orthogonal realization handler, member-type is the effect selector.

## The spine (grain-agnostic, pure)

**Shape decision (want reviewer sign-off):** the reconcile spine is a **generic fn** parameterized by the member projections, NOT a hardcoded Member coproduct. This maximizes diff-sharing (the whole partition is generic, not just the `keyed_two_way_diff` call) and gives R9 *true zero spine change* (a new member type is a new instantiation, editing no central type). A heterogeneous-fleet `Member` coproduct is then just ONE instantiation (`M = Member`), not a precondition.

### Ownership re-grounding — §3 single authority (calm-ferret catch)

`Ownership` is the EXTRACTION of the ownership concept currently embedded in live_deploy's `DeploymentStep = Owned{step} | Ensured{step}` — the ONLY ownership marker in the tree today. "Lift" means **re-ground, not mint-alongside**: there must be ONE `Owned|Ensured` authority, not two whose variant names collide. Resolution (**option b**, scoped to the sequence):

- **Pass 1 (spine):** `Ownership` lands in a shared low home as the single authority; the generic spine consumes it. live_deploy is untouched in pass 1, so no consumer yet holds two `Owned|Ensured`s. A **dissolution trigger** (Scaffold disposition) declares that `DeploymentStep` re-grounds onto `Ownership` in pass 2 — countable, never a silent parallel.
- **Pass 2 (live_deploy binding):** `DeploymentStep` re-grounds onto `Ownership` — the ownership classification is read through the single `Ownership` authority (its variant tags stop independently re-encoding the bit; the coproduct stays the payload discriminator, coupled to `Ownership`). Natural here because pass 2 already rewires live_deploy through the spine. The dissolution trigger closes; ONE `Owned|Ensured` stands.

This is the §2 decompose-map-reduce: the ownership concept, embedded in `DeploymentStep`, is extracted to its own authority and `DeploymentStep` references it — not a second representation.

### Two guardrails on the generic choice (calm-ferret, locked)

1. **ONE `membership_reconcile`, the single authority** — consumers CALL it with their `(key_of, key_eq, value_eq, ownership_of)` bundle; they NEVER copy-adapt a tweaked reconcile. One fn, N parameter-bundles (a Realization), zero reimplementations. A forked reconcile means the genericity bought nothing.
2. **No speculative `M = Member` coproduct** — build the heterogeneous coproduct ONLY when a real heterogeneous reconcile exists (a fleet mixing host+service+session in one pass). Until then, homogeneous instantiations. §6 YAGNI / purity trap. Deferred, not eliminated — when needed it is one instantiation and becomes THE named "heterogeneous fleet member" authority at that point.

```
type Ownership = Owned | Ensured          // the EXTRACTED ownership authority (DeploymentStep re-grounds onto it, pass 2)

type TeardownRefusalCause = MemberNotOwned | OwnershipUnknown

type MemberAction<M>
  = MemberUpsert   { member: M }          // Added or Modified(drift) → (re)apply
  | MemberTeardown { member: M }          // Removed ∩ Owned
  | MemberTeardownRefused { member: M, cause: TeardownRefusalCause }   // Removed ∩ ¬Owned — R5 wall

type MembershipPlan<M> {
  actions: List<MemberAction<M>>          // typed, located (by member), COUNTABLE (§5)
}

fn membership_reconcile<M, K>(
  desired:      List<M>,
  observed:     List<M>,
  key_of:       fn(M) -> K,               // the key_of the std.change engine lacks; lifted here (find_by_identity precedent)
  key_eq:       fn(K, K) -> Bool,
  value_eq:     fn(M, M) -> Bool,
  ownership_of: fn(M) -> Ownership?,      // Absent = ownership undeterminable → refuse (never assume owned)
) -> MembershipPlan<M>
```

**Key = stable IDENTITY, not content (calm-ferret flag).** `key_of` must return the thing that survives a content change (for live_deploy: `OwnedArtifactKind + path`, never the artifact bytes). Otherwise a content drift reads as Remove+Add (teardown+reinstall) instead of Modified (re-upsert) — for an Owned member that is a needless teardown. The whole value of the keyed diff is identity-keying.

**`value_eq` = presence+content (named choice, calm-ferret flag).** The spine is content-aware: `value_eq` is a real parameter, so a member whose key is present in both sets but whose CONTENT drifted (e.g. server.js bytes changed → a different content hash) is an `Unchanged-key / Changed-value` → `KeyedDiffModifiedHunk` → re-upsert. Membership is NOT presence-only. The degenerate poles never trigger `value_eq` (observed is ∅ or full), but live_deploy's binding supplies a content-equality `value_eq` so the real observed-read (later) re-upserts drifted artifacts for free — zero spine change.

Body: wrap `desired`/`observed` into `KeyedRow<K,M>` via `key_of`; call `keyed_two_way_diff(left: observed_rows, right: desired_rows, key_eq, value_eq)` (engine convention: left=observed/base, right=desired, so Absent-left+Present-right = Added); fold `KeyedPatch.hunks`:

- `KeyedDiffAddedHunk{to}`      → `MemberUpsert{to}`
- `KeyedDiffModifiedHunk{to}`   → `MemberUpsert{to}`  (drift correction)
- `KeyedDiffRemovedHunk{from}`  → match `ownership_of(from)`:
  - `Present{Owned}`   → `MemberTeardown{from}`
  - `Present{Ensured}` → `MemberTeardownRefused{from, MemberNotOwned}`   ← R5 wall
  - `Absent`           → `MemberTeardownRefused{from, OwnershipUnknown}` ← R5 wall (fail-closed, never widen)
- (no hunk for a key present-and-equal in both) → noop, by construction (unchanged = absence of hunk)

**R5 is a construction wall, not a post-check:** a Removed non-owned member CANNOT reach a teardown effect — the partition emits `MemberTeardownRefused` and the apply layer has no effect arm for it. The refusal is typed (`TeardownRefusalCause`), located (the `member`/its key), and counted (`actions` is a list; `MemberTeardownRefused` count is observable). §5: refuse, never widen; no "assume owned" absorbing fallback.

## Apply dispatch (Realization — effectful, per-member-type)

`share the DIFF, dispatch the APPLY` (calm-ferret). The plan is grain-agnostic; execution dispatches per member type through the existing transport:

```
fn member_apply<M>(action: MemberAction<M>, dispatch: MemberDispatch<M>, transport: HostEffectTransport)
    -> Reconciliation<HostEffectIntent, HostEffectEvidence>
```
- `MemberUpsert{m}`   → `dispatch.upsert(m, transport)`   (per-binding: live_deploy artifact → `host_effect_apply(ShellCommand …)`; Host → `converge_apply_for_host` verbatim; Session → tmux new-session)
- `MemberTeardown{m}` → `dispatch.teardown(m, transport)` (live_deploy → owned-artifact removal shell; Session → tmux kill-session)
- `MemberTeardownRefused{m, cause}` → `NotConverged{reason: typed(cause, m), applied: …}` — NO effect, counted

Fold over actions with the `converge_apply` refuse-short-circuit skeleton (reused), extended with the teardown + refuse arms it lacks today.

## Binding 1 — live_deploy: unify apply+retract (proven vs existing golden)

live_deploy has no host-inventory read today (apply installs unconditionally, retract removes unconditionally). So apply and retract are the **two poles of one membership diff** — no premature host-read invented:

- `apply`   = `membership_reconcile(desired = spec members [owned ∪ ensured], observed = ∅)` → all `MemberUpsert` → emits today's apply script
- `retract` = `membership_reconcile(desired = ∅, observed = spec owned members)` → all `MemberTeardown` (all Owned) → emits today's retract script; ensured deps are never in the observed-owned set, so never torn down (R5 holds trivially at the pole)

Emit becomes plan-driven: `emit_reconcile(plan)` folds member actions into the orch_emit `Pipeline` (composes with R7 — the concat-kill's `Pipeline` intent). Byte-identity goldens preserved: `expected_live_deploy_apply_script = emit_reconcile(apply_pole)`, `…retract = emit_reconcile(retract_pole)`. The real host-inventory `observed` provider (partial drift) drops in LATER as just the observed-set source — ZERO spine change.

### R5 discriminating witness — SEPARATE from the poles, a merge requirement (calm-ferret §5 catch)

**The degenerate poles CANNOT prove the R5 wall.** The apply pole (observed=∅) has no Removed members; the retract pole (desired=∅, observed=spec.owned) has all-Owned Removed members. So `Removed ∩ (Ensured|Unknown)` is EMPTY in both poles — the refuse arm never fires, and a fail-closed arm that never fires in any witness is UNPROVEN (it could be silently broken and both goldens still pass green). This is the §5 masked-deficit trap.

So the R5 slice ships a **separate synthetic discriminating witness, independent of the poles** (no real host-read needed): a hand-built `observed` set containing a NON-owned member (Ensured, and an Unknown-ownership one) in the Removed set (i.e. `desired` omits it, so it's Removed), asserting:
- it yields `MemberTeardownRefused{MemberNotOwned}` (Ensured) / `{OwnershipUnknown}` (Absent ownership) — typed, located (the member), counted;
- it emits NO teardown effect for that member;
- **siblings continue** — an owned Removed member in the same plan still tears down (the refusal is per-member, not a whole-plan abort).

This witness is a **merge requirement for the R5 slice**, not an afterthought — it is the only thing that proves owned-only teardown by execution (the discriminating RED). A green build without it is a §5 lie.

## R9 accommodation — Session is the same diff (zero spine change)

A future Session member (NOT built here) instantiates the same spine: `M = SessionMember`, `key_of = session identity`, `observed = dispatch_live_sessions(tmux ls)`, `desired = SpawnFrontier.ready`, `ownership_of = did the belt spawn it? Owned : Foreign`, `dispatch.upsert = tmux new-session`, `dispatch.teardown = tmux kill-session`. The Session lane already speaks `ObservationVerdict`/`UpsertDecision` (`DispatchLiveSession.lease_verdict`, `classify_dispatch_request`) and already does ad-hoc set-subtraction (`belt_dispatchable = ready − occupied`) — which becomes a `keyed_two_way_diff` call. **Tell that the spine is grain-agnostic: adding Session edits NO spine type or fn — only a new instantiation + its dispatch handlers.**

## Sequence

1. **Spine + R5** — `membership_reconcile` generic fn, `Ownership`/`MemberAction`/`MembershipPlan`, R5 owned-only partition, typed `MemberTeardownRefused`. Pure; witnessed with a RED control (unowned Removed refuses).
2. **live_deploy binding** — unify apply+retract via the poles; plan-driven emit folding into the R7 `Pipeline`; byte-identity goldens; R5 RED control at the ensured-member pole.

## Pass 2 — LANDED (live_deploy binding)

Pass 2 wired live_deploy onto the spine and refined the pre-commit design in three reviewer-approved ways. It is behavior-preserving: the committed apply/retract goldens stay **byte-for-byte identical** (the oracle), proven green by execution (`live_deploy_emit_holds`).

**Re-ground (option b, landed).** `DeploymentStep`'s payload-discriminating variants were renamed `Owned/Ensured → Artifact/Dependency` (and the payload records `Deployment{Owned,Ensured}Step → Deployment{Artifact,Dependency}Step`), so they name the payload domain, not the ownership axis. `deployment_step_ownership(step) -> Ownership` PROJECTS each variant onto the single `gunbc.ownership.Ownership` authority — ownership is DERIVED, never stored, so an inconsistent ownership/payload pair is unwritable (§5, stronger than the pre-commit option a of a stored field). Only `gunbc.ownership` defines `Owned|Ensured`; the pass-1 reground-trigger Scaffold dissolved when the projection landed.

**The apply grain — `EffectsOrRefusal` sum wall.** The apply dispatch consumes an effects-only input, not the full plan. `membership_effects(plan) -> EffectsOrRefusal<M>` is a SUM: `EffectsReady{effects}` (the upsert/teardown effects) or `ApplyRefused{refusals}` (the `MemberRefusal{member, cause}` list). The refusal arm carries **no effects**, so a plan with any refusal is un-emittable — a consumer can obtain a `List<MemberEffect>` only from the `EffectsReady` arm, making drop-and-proceed (emit the effects, ignore the refusals) **unrepresentable**, not merely discouraged (§5 construction-over-validation — the pre-commit `{effects, refusals}` product was droppable; the sum fixes it). This layers cleanly with the plan grain: `membership_reconcile` still produces every per-member action and counts refusals ("siblings continue"); the apply grain refuses wholesale.

**Wholesale-refuse execution semantics.** The sum also encodes a deliberate *execution policy* (any refusal ⇒ the whole apply emits nothing), which is separable from the type-safety and is **scope-dependent** on the future real observed-provider. That decision — the scope→policy rule and its dissolution condition — is carried by the typed `membership_effects_wholesale_refuse_policy_trigger` in `gunbc.membership_reconcile` (§6 carrier-is-authority). **This paragraph is a gloss; the trigger is the source of truth** — read it for the actual choice, and it is what dissolves when the observed-provider lands and the conscious wholesale-vs-siblings-continue call is made.

**Pole collapse.** `apply = membership_reconcile(desired = spec members, observed = ∅)` → all upsert; `retract = membership_reconcile(desired = ∅, observed = owned artifacts)` → all teardown (all Owned, no refusal). `emit_deploy_member_effect` is the per-kind realization handler (share the diff, dispatch the apply); framing (preamble/receipt) wraps outside the reconcile; orchestration is derived from the effect kind (a SystemdUnit upsert carries its own daemon-reload/enable/restart; ServerScript carries the `/opt/gunbc` `install -d` it writes into — flagged to calm-ferret as the finest-grain reading of the shared-setup, byte-identical to the prior shared prefix). The degenerate poles never produce a refusal, so `ApplyRefused` is unreachable in the real deploy — the wall is a backstop for a future non-degenerate observed provider, exercised only by the synthetic spine witness.

**Tally-monoid refinement (operator-requested, landed with the consumption).** The three `membership_*_count` fns were three separate folds; they are now projections of ONE catamorphism `membership_tally(plan) -> MembershipTally` (a counts-monoid: fixed-size record accumulator, O(1)/step, no copy). §2 one-concept — the elegant fold the diff already is.

## Open shape-questions for reviewers

1. **Generic fn vs Member coproduct** (calm-ferret): I lean generic `membership_reconcile<M,K>` (true zero-change for R9; coproduct is one instantiation). Does that satisfy your "Member coproduct + MemberKey" intent, or do you want a central `Member` coproduct from day one?
2. **Verdict→decision DRY** (calm-ferret flagged it forked 4×): my partition works off diff HUNKS, not `ObservationVerdict`, so I do NOT need that map for pass 1. Consolidating the 4× fork is a related but separate §3 cleanup — fold in later, not here. Agree?
3. **`Refuse: NonEmptyStr` upgrade**: R5 wants typed+located+counted refusals. I model `MemberTeardownRefused{member, cause}` (typed+located+counted) at the membership layer rather than upgrading `UpsertDecision.Refuse`. OK, or do you want the `UpsertDecision.Refuse` authority upgraded to carry a typed diagnostic?
4. **royal-carp — grain-agnosticism check**: does the "diff takes Session without surgery" test pass on this shape (only a new instantiation + dispatch arms, no spine-type edit)? And is the R5 owned-only boundary (Removed∩¬Owned + Unknown both REFUSE, construction-walled) what you locked?
