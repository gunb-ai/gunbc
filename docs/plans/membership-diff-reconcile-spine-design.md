# Grain-agnostic membership-diff reconcile spine (R2/R3/R5/R9)

Status: LANDED — Pass 1 (spine + R5) and Pass 2 (live_deploy binding) implemented, reviewed pre-commit by calm-ferret-849 (seam) + royal-carp-451 (grain-agnosticism + R5 boundary). See "Pass 2 — LANDED" below for the as-built shape, which refined the pre-commit design in three reviewed ways: derived-ownership projection, the `EffectsOrRefusal` sum wall, and the typed wholesale-refuse policy trigger.

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

**Fork-traps avoided (calm-ferret):** (a) no per-grain fork of `keyed_two_way_diff` — one generic engine; (b) membership grain and knob grain are not collapsed into one fold — they LAYER (membership is the outer consumer; host knob-converge stays a `K=ConvergeKnobKey` inner consumer, added later under a Host member); (c) no new ownership enum — Owned/Ensured generalized; (d) member-grain not coupled to transport — transport is the orthogonal realization handler, member-type the effect selector.

## The spine (grain-agnostic, pure)

**Shape decision (want reviewer sign-off):** the reconcile spine is a **generic fn** parameterized by the member projections, NOT a hardcoded Member coproduct — the whole partition is shared (not just the `keyed_two_way_diff` call), and R9 gets *true zero spine change* (a new member type is a new instantiation, no central type edited). A heterogeneous-fleet `Member` coproduct is then ONE instantiation (`M = Member`), not a precondition.

### Ownership re-grounding — §3 single authority (calm-ferret catch)

`Ownership` EXTRACTS the ownership concept embedded in live_deploy's `DeploymentStep = Owned{step} | Ensured{step}` — the ONLY ownership marker in the tree today. "Lift" means **re-ground, not mint-alongside**: ONE `Owned|Ensured` authority, not two with colliding variant names. Resolution (**option b**, scoped to the sequence):

- **Pass 1 (spine):** `Ownership` lands in a shared low home as the single authority; the generic spine consumes it. live_deploy is untouched, so no consumer yet holds two `Owned|Ensured`s. A **dissolution trigger** (Scaffold disposition) declares that `DeploymentStep` re-grounds onto `Ownership` in pass 2 — countable, never a silent parallel.
- **Pass 2 (live_deploy binding):** `DeploymentStep` re-grounds onto `Ownership` — ownership is read through the single `Ownership` authority (its variant tags stop re-encoding the bit; the coproduct stays the payload discriminator, coupled to `Ownership`). Natural here because pass 2 already rewires live_deploy through the spine. The trigger closes; ONE `Owned|Ensured` stands.

§2 decompose-map-reduce: the ownership concept is extracted from `DeploymentStep` to its own authority, which `DeploymentStep` references — not a second representation.

### Two guardrails on the generic choice (calm-ferret, locked)

1. **ONE `membership_reconcile`, the single authority** — consumers CALL it with their `(key_of, key_eq, value_eq, ownership_of)` bundle and NEVER copy-adapt a tweaked reconcile. One fn, N parameter-bundles (a Realization), zero reimplementations; a forked reconcile means the genericity bought nothing.
2. **No speculative `M = Member` coproduct** — build the heterogeneous coproduct ONLY when a real heterogeneous reconcile exists (a fleet mixing host+service+session in one pass); until then, homogeneous instantiations (§6 YAGNI / purity trap). Deferred, not eliminated — when needed it is one instantiation and becomes THE named "heterogeneous fleet member" authority.

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

**Key = stable IDENTITY, not content (calm-ferret flag).** `key_of` returns what survives a content change (live_deploy: `OwnedArtifactKind + path`, never the artifact bytes); otherwise content drift reads as Remove+Add (teardown+reinstall) instead of Modified (re-upsert) — a needless teardown for an Owned member. Identity-keying is the keyed diff's whole value.

**`value_eq` = presence+content (named choice, calm-ferret flag).** The spine is content-aware, not presence-only: a key present in both sets whose CONTENT drifted (e.g. server.js bytes → a different content hash) is `Unchanged-key / Changed-value` → `KeyedDiffModifiedHunk` → re-upsert. The degenerate poles never trigger `value_eq` (observed is ∅ or full), but live_deploy's binding supplies a content-equality `value_eq`, so the later real observed-read re-upserts drifted artifacts for free — zero spine change.

Body: wrap `desired`/`observed` into `KeyedRow<K,M>` via `key_of`; call `keyed_two_way_diff(left: observed_rows, right: desired_rows, key_eq, value_eq)` (engine convention: left=observed/base, right=desired, so Absent-left+Present-right = Added); fold `KeyedPatch.hunks`:

- `KeyedDiffAddedHunk{to}`      → `MemberUpsert{to}`
- `KeyedDiffModifiedHunk{to}`   → `MemberUpsert{to}`  (drift correction)
- `KeyedDiffRemovedHunk{from}`  → match `ownership_of(from)`:
  - `Present{Owned}`   → `MemberTeardown{from}`
  - `Present{Ensured}` → `MemberTeardownRefused{from, MemberNotOwned}`   ← R5 wall
  - `Absent`           → `MemberTeardownRefused{from, OwnershipUnknown}` ← R5 wall (fail-closed, never widen)
- (no hunk for a key present-and-equal in both) → noop, by construction (unchanged = absence of hunk)

**R5 is a construction wall, not a post-check:** a Removed non-owned member CANNOT reach a teardown effect — the partition emits `MemberTeardownRefused` and the apply layer has no effect arm for it. The refusal is typed (`TeardownRefusalCause`), located (the `member`/its key), and counted (the `MemberTeardownRefused` count over `actions` is observable). §5: refuse, never widen; no "assume owned" absorbing fallback.

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

live_deploy has no host-inventory read today (apply installs unconditionally, retract removes unconditionally), so apply and retract are the **two poles of one membership diff** — no premature host-read invented:

- `apply`   = `membership_reconcile(desired = spec members [owned ∪ ensured], observed = ∅)` → all `MemberUpsert` → emits today's apply script
- `retract` = `membership_reconcile(desired = ∅, observed = spec owned members)` → all `MemberTeardown` (all Owned) → emits today's retract script; ensured deps are never in the observed-owned set, so never torn down (R5 holds trivially at the pole)

Emit becomes plan-driven: `emit_reconcile(plan)` folds member actions into the orch_emit `Pipeline` (composes with R7 — the concat-kill's `Pipeline` intent). Byte-identity goldens preserved: `expected_live_deploy_apply_script = emit_reconcile(apply_pole)`, `…retract = emit_reconcile(retract_pole)`. The real host-inventory `observed` provider (partial drift) drops in LATER as just the observed-set source — ZERO spine change.

### R5 discriminating witness — SEPARATE from the poles, a merge requirement (calm-ferret §5 catch)

**The degenerate poles CANNOT prove the R5 wall.** The apply pole (observed=∅) has no Removed members; the retract pole (desired=∅, observed=spec.owned) has only Owned Removed members. `Removed ∩ (Ensured|Unknown)` is EMPTY in both, so the refuse arm never fires — and a fail-closed arm no witness fires is UNPROVEN: it could be silently broken with both goldens green (the §5 masked-deficit trap).

So the R5 slice ships a **separate synthetic discriminating witness, independent of the poles** (no real host-read needed): a hand-built `observed` set with a NON-owned member (one Ensured, one Unknown-ownership) omitted from `desired` (so Removed), asserting:
- it yields `MemberTeardownRefused{MemberNotOwned}` (Ensured) / `{OwnershipUnknown}` (Absent ownership) — typed, located (the member), counted;
- it emits NO teardown effect for that member;
- **siblings continue** — an owned Removed member in the same plan still tears down (the refusal is per-member, not a whole-plan abort).

This witness is a **merge requirement for the R5 slice** — the only thing that proves owned-only teardown by execution (the discriminating RED). A green build without it is a §5 lie.

## R9 accommodation — Session is the same diff (zero spine change)

A future Session member (NOT built here) instantiates the same spine: `M = SessionMember`, `key_of = session identity`, `observed = dispatch_live_sessions(tmux ls)`, `desired = SpawnFrontier.ready`, `ownership_of = did the belt spawn it? Owned : Foreign`, `dispatch.upsert = tmux new-session`, `dispatch.teardown = tmux kill-session`. The Session lane already speaks `ObservationVerdict`/`UpsertDecision` (`DispatchLiveSession.lease_verdict`, `classify_dispatch_request`) and already does ad-hoc set-subtraction (`belt_dispatchable = ready − occupied`), which becomes a `keyed_two_way_diff` call. **Tell that the spine is grain-agnostic: adding Session edits NO spine type or fn — only a new instantiation + its dispatch handlers.**

## Sequence

1. **Spine + R5** — `membership_reconcile` generic fn, `Ownership`/`MemberAction`/`MembershipPlan`, R5 owned-only partition, typed `MemberTeardownRefused`. Pure; witnessed with a RED control (unowned Removed refuses).
2. **live_deploy binding** — unify apply+retract via the poles; plan-driven emit folding into the R7 `Pipeline`; byte-identity goldens; R5 RED control at the ensured-member pole.

## Pass 2 — LANDED (live_deploy binding)

Pass 2 wired live_deploy onto the spine and refined the pre-commit design in three reviewer-approved ways. Behavior-preserving: the committed apply/retract goldens stay **byte-for-byte identical** (the oracle), green by execution (`live_deploy_emit_holds`).

**Re-ground (option b, landed).** `DeploymentStep`'s payload-discriminating variants were renamed `Owned/Ensured → Artifact/Dependency` (payload records `Deployment{Owned,Ensured}Step → Deployment{Artifact,Dependency}Step`), naming the payload domain, not the ownership axis. `deployment_step_ownership(step) -> Ownership` PROJECTS each variant onto the single `gunbc.ownership.Ownership` authority — ownership is DERIVED, never stored, so an inconsistent ownership/payload pair is unwritable (§5; stronger than pre-commit option a's stored field). Only `gunbc.ownership` defines `Owned|Ensured`; the pass-1 reground-trigger Scaffold dissolved when the projection landed.

**The apply grain — `EffectsOrRefusal` sum wall.** The apply dispatch consumes effects only, not the full plan. `membership_effects(plan) -> EffectsOrRefusal<M>` is a SUM: `EffectsReady{effects}` (upsert/teardown effects) or `ApplyRefused{refusals}` (the `MemberRefusal{member, cause}` list). The refusal arm carries **no effects**, so a plan with any refusal is un-emittable — a `List<MemberEffect>` exists only in `EffectsReady`, making drop-and-proceed (emit effects, ignore refusals) **unrepresentable**, not merely discouraged (§5 construction-over-validation; the pre-commit `{effects, refusals}` product was droppable). It layers with the plan grain: `membership_reconcile` still produces every per-member action and counts refusals ("siblings continue"); the apply grain refuses wholesale.

**Wholesale-refuse execution semantics.** The sum also encodes a deliberate *execution policy* (any refusal ⇒ the whole apply emits nothing), separable from the type-safety and **scope-dependent** on the future real observed-provider. The scope→policy rule and its dissolution condition are carried by the typed `membership_effects_wholesale_refuse_policy_trigger` in `gunbc.membership_reconcile` (§6 carrier-is-authority). **This paragraph is a gloss; the trigger is the source of truth** — it holds the actual choice and dissolves when the observed-provider lands and the wholesale-vs-siblings-continue call is made.

**Pole collapse.** `apply = membership_reconcile(desired = spec members, observed = ∅)` → all upsert; `retract = membership_reconcile(desired = ∅, observed = owned artifacts)` → all teardown (all Owned, no refusal). `emit_deploy_member_effect` is the per-kind realization handler (share the diff, dispatch the apply); framing (preamble/receipt) wraps outside the reconcile; orchestration derives from the effect kind (a SystemdUnit upsert carries its own daemon-reload/enable/restart; ServerScript carries the `/opt/gunbc` `install -d` it writes into — flagged to calm-ferret as the finest-grain reading of the shared-setup, byte-identical to the prior shared prefix). The poles never produce a refusal, so `ApplyRefused` is unreachable in the real deploy — a backstop for a future non-degenerate observed provider, exercised only by the synthetic spine witness.

**Tally-monoid refinement (operator-requested, landed with the consumption).** The three `membership_*_count` fns were three separate folds; they are now projections of ONE catamorphism `membership_tally(plan) -> MembershipTally` (a counts-monoid: fixed-size record accumulator, O(1)/step, no copy). §2 one-concept.

## Open shape-questions for reviewers

1. **Generic fn vs Member coproduct** (calm-ferret): I lean generic `membership_reconcile<M,K>` (true zero-change for R9; coproduct is one instantiation). Does that satisfy your "Member coproduct + MemberKey" intent, or do you want a central `Member` coproduct from day one?
2. **Verdict→decision DRY** (calm-ferret flagged it forked 4×): my partition works off diff HUNKS, not `ObservationVerdict`, so pass 1 does not need that map. Consolidating the 4× fork is a related but separate §3 cleanup — fold in later, not here. Agree?
3. **`Refuse: NonEmptyStr` upgrade**: R5 wants typed+located+counted refusals. I model `MemberTeardownRefused{member, cause}` at the membership layer rather than upgrading `UpsertDecision.Refuse`. OK, or should the `UpsertDecision.Refuse` authority be upgraded to carry a typed diagnostic?
4. **royal-carp — grain-agnosticism check**: does the "diff takes Session without surgery" test pass on this shape (only a new instantiation + dispatch arms, no spine-type edit)? And is the R5 owned-only boundary (Removed∩¬Owned + Unknown both REFUSE, construction-walled) what you locked?

## Pass 3 — LANDED (occurrence identity + typed replacement classification)

Operator ruling: *"Identity should be declaration-owned and allocator-minted — not derived from unit/path, and not a closed operator-maintained `HostRole` enum."*

### The arms

`MemberAction<M>` is four arms, not three:

```
= MemberAdded          { to: M }
| MemberChanged        { from: M, to: M }
| MemberRemoved        { from: M }
| MemberRemovalRefused { from: M, cause: TeardownRefusalCause }
```

Add and change were previously one `MemberUpsert{member}` carrying only the `to` side. **The discarded `from` is the load-bearing loss**: every safe realization of a change consumes the prior value — an SCM ref update is a compare-and-swap *naming the expected prior* (`git update-ref <ref> <new> <old>`); a host resource whose address moved must retire the prior endpoint. With only `to`, the sole reachable realization is the unconditional write, so the atomicity the capability already offers is unreachable. `MemberEffect` splits the same way (`EffectAdd{to}` / `EffectReplace{from,to}` / `EffectRemove{from}`) so the apply grain can retire a prior realization rather than overwrite it blind.

`MemberRemovalRefused` is a first-class **action**, not an absence — that is what keeps a refusal countable rather than a hunk that silently did not appear.

### What is NOT here, and why (a withdrawn wall)

An earlier draft also refused a Modified hunk whose `from` was not `Owned`, reading replacement as destruction and claiming a live fail-open in five of seven consumers. **Built, refuted by execution, withdrawn.** `gunbc.ownership` defines `Ensured` as *"requires present but does NOT own (never torn down)"* — a constraint on **removal**, not update. `tool_readiness` classifies every pin `Ensured` precisely to re-install a pin whose observed version drifted; `repo_local_git_config` likewise. The wall broke those modules where it fired and was unreachable elsewhere (`host_axis_caps` derives ownership from the path, so a Modified hunk cannot pair a managed desired row with a foreign observed one; foreign `authorized_keys` are absent from desired, so Removed, never Modified). The original asymmetry was correct.

Per DESIGN §4c a carrier note is *not* evidence a machine claim holds, so the withdrawal is enrolled, not written down: `w_ensured_member_converges_but_refuses_removal` puts one `Ensured` member through **both** arms in one plan — it converges when its value drifts and refuses when removed. Reintroducing the change-refusal arm turns it red (verified by planting exactly that perturbation; unrelated rows stayed green).

### Identity: minted, not derived

`gunbc.host_resource`:

```
type HostResourceIdentity { host: HostIdentity, occurrence: ResourceOccurrenceId }
type HostResource<R, V>   { identity: HostResourceIdentity, role: R, value: V }
```

The occurrence is **allocator-minted at declaration**, never computed from content, path, or unit name — the closed-system answer to renames. git does not track renames; it *infers* them from content similarity after the fact, and DESIGN §4 rules a heuristic never necessary in a closed system, so a content- or path-derived `key_of` is that inference wearing a bundle. With an allocated occurrence a rename is ONE `MemberChanged` whose `from` and `to` share an identity; nothing is guessed. `role` is a **type parameter**, so `Jobserver` / `BuildCache` / `RunnerSlot` are instantiations, not edits to a central enum (DESIGN §3: adding Opera must not widen a browser-product enum). Unit name, FIFO, storage path and configuration are mutable **value** fields. `HostIdentity` is reused from `product.placement_supply`, not re-minted.

**Why a distinct carrier from `std.occurrence_identity`** (decided by reading, recorded in-carrier): that module is about *source* occurrences — `OccurrenceCategory` enumerates callable/type/constructor/field/method/namespace-segment, its carriers `DeclarationOccurrence`/`ReferenceOccurrence` range over containment paths, and its allocator is coupled via `occurrence_id_allocator_advance_to` to `AuthoredTokenOrdinalSpace`, a source-token watermark meaningless for a host resource. The subject-agnostic residue is only a monotone `Int` counter, and one shared `Int` space would let `occurrence_id_eq` answer `true` on a **numeric coincidence** between a host resource and a source occurrence — the cross-family comparison the `ContentHash` family grounding closed. The real §3 move (one subject-parameterized minted-identity carrier, the `Vendor<Domain>` shape) is registered as `resource_occurrence_generalization_trigger` rather than built, because `std.occurrence_identity` is load-bearing compiler substrate with stage0 work in flight.

### The classification table

`gunbc.change_realization.classify_change` is what makes `MemberChanged` useful rather than merely more faithful — four **different atomicity guarantees**, where picking the wrong row is how a safe transformation acquires an unsafe realization:

| subject / condition | realization |
| --- | --- |
| SCM ref update | atomic compare-and-swap (`git update-ref <ref> <new> <old>`) |
| source declaration move | atomic SCM tree transformation |
| host resource, same runtime address | in-place update |
| host resource, changed host address | staged replacement with **declared** intermediates |

**The conflation this kills:** a `.dag` module re-home is an SCM tree transformation, **not** a host migration — it changes no runtime address, unlinks no socket, restarts no unit. Treating it as a migration made an ordinary source cleanup look like it needed an operational migration plan. The converse error is the dangerous one: renaming a systemd unit or moving a FIFO *is* a host replacement.

**The srv4 specimen, made executable.** srv4's build-cache server died with its cgroup, never unlinked its socket, and ran uncached for weeks because nobody modeled the intermediate state; `gunbc.build_cache_instance` says only in *prose* that a managed unit must clear a stale endpoint on start — where no machine can read it. `StagedIntermediates` is head + tail, so an empty intermediate plan **has no constructor** (§5 construction-over-validation; §4b: unrepresentable, not validated). A staged replacement with undeclared intermediates **refuses**, typed and located by `subject_label` — it neither degrades to an in-place update nor proceeds with an empty plan (both the absorbing fallback §5 forbids; the second *is* the srv4 outcome). Verified discriminating: perturbing the refusal into an `InPlaceUpdate` fallback reds that row alone.

### Consumer migration

All seven consumers plus witnesses migrated; the compiler located every site by exhaustiveness. One real regression caught: `repo_local_git_config`'s apply fold has a `_ => acc` wildcard for its construction-unreachable removal arms, and a `MemberChanged` would have fallen into it — silently dropping exactly the drift that module exists to converge. Both write arms are now explicit and routed through one `apply_repo_local_git_config_binding`. `live_deploy`'s `EffectReplace` arm is structurally unreachable at both degenerate poles, so per that module's discipline it emits a loud typed poison, not a fabricated no-op; the poison names the staged-replacement requirement a real observed provider must satisfy.

`membership_write_count` (adds + changes) is a **derived** projection replacing `membership_upsert_count` for consumers that read it as "how much work is left"; the split is never re-stored as one counter.

### Open

The `(key_of, key_eq, value_eq, ownership_of)` bundle still parameterizes the spine, so existing consumers are unchanged. Migrating them onto identity-carrying members is a separate lane — deliberately unbundled so the srv3 capacity work depending only on the `Added` arm is not blocked.
