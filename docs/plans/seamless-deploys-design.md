# Seamless deploys — design note

**Status: DRAFT, modeling-first. No code lands from this note.** The brief asks that the modeling be
grounded and agreed before implementation; this note does the grounding, proposes the model, and
names the decisions that need operator sign-off. Every claim below carries a receipt against the
live tree or a measurement, or is marked as an open question.

Brief: *Deploying should not take the dashboard away.* Boundary: the restart path only — the serve
process, its unit, and how the browser is told what it is seeing. Not what the deploy installs.

---

## 1. Grounding — what the tree actually says

The brief's four items were checked against the tree. Three confirm exactly, one confirms and
sharpens into a different fix, and two side claims need correcting.

| Brief claim | Verdict | Receipt |
|---|---|---|
| (1) Transport failure and typed refusal render identically | **Confirmed, line-exact** | `dag/gunbc/roadmap_component.dag:2463` (the `.then` arm, guarded by `wj.observe_refused`) and `:2507` (the `.catch` arm) both build the literal `"workflow observation refused: "`, then both call `workflow_refuse_visible_stmt()` and `observation_refused_banner_stmts(source_class: "workflow-observe-refused")` — same prefix, same banner, same source class |
| (2) `Type=simple`, so systemd reports ready before it is | **Confirmed** | `dag/gunbc/live_deploy/emit.dag:123` emits `Type=simple`; the seed has **no** systemd integration at all — zero hits for `sd_notify`, `NOTIFY_SOCKET`, or `LISTEN_FDS` anywhere in `src/v1/stage0/src/` |
| (3) The deploy restarts unconditionally | **Confirmed, and the cause is upstream of the arm** — see §2 | `emit.dag:283–289`: the `SystemdUnit` upsert arm is a fixed 5-step list ending in daemon-reload → enable → restart, with no predicate |
| (4) No handover | **Confirmed** | One unit, one `ExecStart`, `systemctl restart` in place; nothing holds the socket or overlaps instances |
| Outage is 60–90s | **Confirmed and calibrated** | `live_deploy_roadmap_unit_expected_startup = 40s`; poll bound = 40 × 3 = 120s; four independent srv1 journal observations recorded at 35s, 36s, 36s, 39s to listening, all successful starts (`service_ready.dag`, `live_deploy_service_ready_poll_bound_reason`) |
| Browser polls every 2s | **Confirmed** | `workflow_observation_poll_interval = millisecond(count: 2000)` |
| Bind happens after the compile | **Confirmed, with the four exit paths** | `cli_run.rs`: `load_sources_for_entry` → `compile_to_resolved` → `TcpListener::bind` at `:11918`. Four distinct `exit(1)` paths precede the bind |

### Two corrections

**"208 sources" does not match the tree's own figure.** `live_deploy_service_ready_poll_bound_reason`
records the measurement differently: `build_module_index` reads **every** source root — 2,356 `.dag`
files, 12.7 MB at time of writing — in order to serve **a closure of 91**. The brief's 208 is
neither of those numbers. These are different quantities (files read vs. closure compiled) and the
gap between them *is* the cost shape, so the deep fix must not be sized off a single conflated
figure. **Corpus today is 2,719 `.dag` files** — up ~15% from the 2,356 that measurement was taken
against on 2026-07-21, nine days ago. The load term grows with the corpus, which is exactly what
that note predicted, and it means the 40s expectation is already drifting under us.

**The restart count resolves opposite to the caveat.** The brief flags: *"I have not confirmed
whether all 24 restarts were deploys. Some may have been you."* Counted from the deploy side:
**40 `deploy_dashboard_srv1` job executions in the 24h window** ending 2026-07-30T20:11Z (38
success, 2 failure). Every one of them runs the unconditional restart arm. So deploys alone
account for **more than** the 24 observed restarts — the count needs no manual restarts to explain
it, and item 3 is real without qualification.

Honest bound on that: I have **no SSH access to srv1 from this session** (`node@srv1` and
`briansrls@srv1` both refuse publickey), so I could not read the journal directly. What is measured
is 40 restart *commands issued*; that each produced a restart follows from the arm being
unconditional, not from an observation. If the journal shows materially fewer than 38, that
discrepancy is itself a finding and this note should be revisited.

One thing the count surfaced that the brief did not ask about: **eight of those 40 deploys started
within ~2 minutes of another deploy** (e.g. 17:49:04 / 17:50:47, 17:38:16 / 17:40:01, 23:59:38 /
00:03:18). That is the known concurrent-deploy race — two runs mutating one unit, previously
diagnosed as `ServedSurfaceStale`. It is **out of this ticket's boundary** but it *interacts* with
every fix below: a handover model that assumes one deploy at a time is wrong on ~20% of deploys.
Flagged in §6 as a coupling, not adopted as scope.

---

## 2. The modeling — where the four items actually cut

The brief says the tangle is four separable things and resists fixing them as one. Agreed. But the
model-level seams do not fall exactly where the four items do. Three concepts carry all four.

### Concept A — *Observation outcome* is a state-space conflation (item 1)

Today the client folds three distinct states into one sentence:

- the server answered, and its answer was a typed refusal (`wj.observe_refused` + `reason`);
- the server answered, and the answer was fine;
- **nothing answered** (`.catch` — connection refused, DNS, abort, parse failure).

The first and third render byte-identically. This is the state-space conflation DESIGN §3 names,
and the code already holds the discriminant — it is literally which of the two callbacks ran — and
discards it at the message string. So the fix is *not* new information; it is refusing to erase
information already in hand.

**The fix is a de-conflation, and the arm must not over-claim.** This is the one place I would
push back on the brief's wording. It proposes the transport arm say *"deploying, back shortly."*
The transport arm does not know that. A `.catch` fires for a deploy, for srv1 being off, for a
crash-looped serve, for the network dropping, and for a malformed body. Asserting *deploying* is
substituting a plausible cause for an unknown one — the fabricated-plausible-output shape DESIGN §5
forbids, just at the UI layer rather than the compiler layer. The arm may honestly say **what it
observed** (*the dashboard is not answering*) and may be styled as transient rather than alarming;
it may **not** name a cause it has not established.

If we want the page to say *deploying*, that has to be **grounded** — the server would have to tell
it so, which is a different mechanism (a deploy-in-progress fact on the wire) and a different
ticket. Worth noting the honest version is nearly as good: a calm *"not answering, retrying"* that
clears itself in 60s reads very differently from a red refusal banner, and it stays true when the
cause is not a deploy.

**Interaction the brief should know about, because it changes the ordering argument:** if socket
activation lands (§3), connections **queue instead of being refused**, so the `.catch` arm stops
firing during deploys altogether. Item 1's new sentence would then be unreachable in the common
case it was written for. That does **not** make item 1 wasted — the transport/typed distinction is
permanent and correct, and it is what makes a *genuine* srv1 outage legible — but it does mean item
1 should be justified as *"the refusal tells the truth"*, not as *"this is how we fix the deploy
banner"*. Both fixes are worth having; they do not stack the way the brief's ordering implies.

### Concept B — *Readiness is currently modeled twice, and one copy is wrong* (item 2, and it is the keystone for item 4)

This is the finding I would most want agreed before anything is built.

Readiness of `gunbc-roadmap.service` has **two representations in the tree today**:

1. **systemd's** — `Type=simple`, which by construction means *ready ≡ process spawned*. This is
   false for this service by ~35–40 seconds, every single time. It is not occasionally wrong; it is
   never right.
2. **`live_deploy`'s** — `service_ready.dag` polls `/healthz` on a 1s cadence to a 120s bound, and
   additionally compares a surface digest (`HealthzSurfaceStale`). This one is correct.

Representation 2 exists *because* representation 1 is a lie. That is a §3 fork: one fact, two
authorities, the derived one compensating for the broken one. And it explains why item 2's
practical blast radius today is smaller than the brief implies — **the deploy already distrusts
systemd**, so it does not suffer from the lie; it routes around it. I checked the other in-tree
consumers of `SystemdUnitActiveState` (`host_hygiene_reaper.dag`, `oomd_install.dag`) and neither
targets `gunbc-roadmap.service`. So today the misled consumers are: a human at `systemctl status`,
systemd's own ordering and `Restart=` semantics, and any future consumer.

**Which means item 2 is not independently valuable as an outage fix — it is the enabling model for
item 4, and a de-fork.** You cannot sequence a handover ("new instance is ready → move traffic")
without a real readiness signal; the healthz poll cannot serve that role because it is external to
the unit and runs in the deploy job, not on the host. Fix the unit's answer (`Type=notify` +
`sd_notify(READY=1)` immediately after the bind at `cli_run.rs:11918`) and the correct fact has a
single authority; the healthz poll then becomes *derivable* rather than compensatory, and can be
argued down to the digest check it uniquely owns.

So I would re-cut item 2 from *"the unit reports ready when it is not"* (true but low-cost today)
to **"readiness has two representations and the authoritative one is wrong"** — which is a §3
violation, is the prerequisite for item 4, and is what makes the poll bound in §1 stop being a
number we have to keep re-calibrating against corpus growth.

### Concept C — *The deploy does not reconcile; it applies* (item 3)

The brief says the `SystemdUnit` arm restarts *"whether or not the unit, binary, or tree changed."*
That is true, but the cause is one level up, and it matters because it changes the fix from a
validation to a construction.

`emit.dag:158` — the apply pole calls the grain-agnostic spine with **`observed: []`**:

```
membership_reconcile(desired: <all deploy members>, observed: [], key_of: …, value_eq: …)
```

The module's own note (`live_deploy_reconcile_binding_note`) states it plainly: *"apply = reconcile(
desired = all deploy members, observed = none) => every member Added => Upsert … value_eq never
fires in the degenerate poles (no Modified — one side is always empty)."*

So the deploy is not failing to diff — **it is structurally incapable of diffing.** Both poles are
degenerate: apply is `observed = ∅` (everything Added), retract is `desired = ∅` (everything
Removed). `Unchanged → noop` is already in the spine, already correct, and never reached.

**The fix is therefore not a predicate on the restart step.** Adding `if unit-file-changed then
restart` would be validation standing where construction was available — a second representation of
"has this changed", checked after the fact, exactly what DESIGN §5 says to prefer construction over.
The fix is to **supply the observed set**: read what is actually installed on srv1 (unit file
content, binary identity, tree digest), pass it as `observed`, and `Unchanged → noop` falls out of
the spine that already implements it. No new comparison logic, no new authority.

Two consequences worth stating up front, both good:

- The spine's `ownership_of` refusal arms (`OwnershipUnknown`, `MemberNotOwned`) are today exercised
  *only by a synthetic witness* — the note says so. A real observed provider makes them live. That
  is the safety mechanism activating as designed, but it means the first non-degenerate deploy can
  *refuse* where it previously always applied, and that refusal path needs to be real before this
  ships.
- Reading the observed set requires observing the live host, which is a host-effect with its own
  failure mode. **The failure arm must refuse, not widen.** "I could not read what is installed"
  must not degrade to "so restart everything" — that is precisely the absorbing fallback (⊤-as-
  answer conflated with ⊤-as-ignorance), and it would silently restore today's behaviour while
  looking like a fix. `ObservationUnavailable` should be a typed, located, **counted** refusal.

### Item 4 (handover) is downstream of B and C

With B (real readiness) and C (only restart when something changed), item 4's remaining scope
shrinks a lot: the restarts that survive are the ones that genuinely need to happen, and there is a
real signal for when the replacement is up. Socket activation (§3) is then one candidate
realization for covering the residual window, not the whole answer.

---

## 3. Socket activation — evaluating the brief's preferred candidate

The brief asks that queueing behaviour be confirmed as desirable before building. Assessment:

**It works, mechanically.** A `.socket` unit makes systemd own the listening fd; it survives the
service restart, so connections land in the kernel accept backlog instead of getting ECONNREFUSED.
Combined with single-flight polling (`workflow_poll_single_flight_note` — at most one workflow
observation in flight per browser), queue depth is bounded by viewer count, not by
viewers × 30 polls. The brief's reasoning here is correct.

**Three things to decide before it is built:**

1. **It requires a seed change, and the seed is supposed to shrink.** The process currently *binds*
   (`TcpListener::bind`, `cli_run.rs:11918`). Socket activation requires it to *inherit* fd 3 via
   `LISTEN_FDS`. There is no systemd integration in the seed today. Per DESIGN §7 this needs
   justifying rather than just doing. The clean framing — which I think holds — is that
   **listener acquisition is a §2 Realization**: one shape (*the serve process obtains a bound
   listener*) with N handlers (`SelfBound` today, `Inherited` under socket activation). Modelled
   that way it is a row plus a handler, not systemd cemented into the seed. Modelled carelessly it
   is an `if env::var("LISTEN_FDS")` in the middle of `handle_serve`, which is the thing to avoid.
   The same seam carries `sd_notify` for item 2, so B and this share one realization boundary and
   should be designed together even if they land separately.

2. **A queued request is not obviously better than a refused one, and the brief already suspects
   this.** A 67s-queued fetch means the page shows **stale data with no indication for 67 seconds**.
   The user sees a dashboard that looks live and is not. That is a quieter failure, not
   automatically a better one — and it is worth noticing it has the same shape as the thing we are
   fixing: the UI asserting a state it has not established. My read is that queueing *is* right
   here, because the data is genuinely only ~60s stale and the page never claimed a freshness
   guarantee — but it argues for pairing socket activation with a **visible staleness cue** (last-
   updated age), not with silence. That is a small addition and it keeps the page honest under both
   fixes.

3. **Backlog is finite.** `Backlog=` defaults are modest (128). Bounded by viewer count this is
   fine today. It should be a declared number with a stated assumption about concurrent viewers, not
   a default we inherit silently — and the overflow behaviour should be understood, since a full
   backlog reverts to refusal (which item 1 will then render honestly — the two fixes compose
   correctly here).

---

## 4. The deep version, and the dependency

The brief's instinct is right, and **the tree already agrees with it in writing.** The dissolution
trigger recorded on the poll bound (`live_deploy_service_ready_poll_bound_reason`) reads:

> *"This bound is sized around a cost-shape defect, not a fact of the service: gunbc serve pays a
> corpus-scale load to serve an entry-scale closure (26x the sources it needs). When
> `load_sources_for_entry` / `build_module_index` are demand-scoped to the entry's closure, startup
> returns to roughly compile time alone and expected_startup drops back to single digits."*

So the deep fix is already named, already has a trigger, and this ticket is a **second counted
consumer** of it — which is exactly the argument the brief makes for depending on
`v1-materialization-kernel` rather than duplicating it. Concretely, the roadmap node exists
(`roadmap_authority.dag:948`, id `v1-materialization-kernel`, owner `self-host`, path
`docs/plans/witness-realization-plan.md`) and the edge mechanism is `edge(child:, parent:)`.

Two observations on that dependency:

- The two are **not the same fix**, and the dependency should not be read as "wait for it". Demand-
  scoping the load makes startup fast, which shrinks the outage toward zero and dissolves the poll
  bound. It does **not** fix items 1, 2, or 3 — a 3-second outage still refuses, the unit still lies,
  and the deploy still restarts unnecessarily. So this ticket depends on that node for its *headline
  outcome at the limit*, while items 1–3 are independently correct and independently landable.
- Corpus growth (2,356 → 2,719 in nine days) means the deep fix's value is **increasing**, and the
  40s expectation has a shelf life. Worth carrying into that node's own priority as a second
  consumer with a measured drift rate, per the brief's point that nobody had counted it.

---

## 5. Proposed sequencing

Ordered by displaced cost per unit of risk, respecting the brief's first_slice.

**Slice 1 — de-conflate the observation outcome (item 1).** Give the transport arm its own sentence,
stating what was observed and not asserting a cause. No timing change, no deployment change, no seed
change. Independently correct regardless of what follows.

**Slice 2 — the observed provider (item 3).** Supply `observed` to the apply pole so `Unchanged →
noop` is reached by construction. Removes the avoidable restarts. Requires the refusal arm for
"could not observe" to be typed/located/counted, and makes the ownership refusal arms live.

**Slice 3 — the readiness de-fork (item 2), designed with the listener-acquisition realization
(§3.1).** `Type=notify` + a readiness signal at the bind point, modelled as a realization rather
than cemented. Enables item 4 and puts readiness on one authority.

**Slice 4 — the residual window (item 4).** Socket activation, evaluated against §3's three
decisions, with the staleness cue.

**Depends on, does not duplicate:** `v1-materialization-kernel`.

Slices 1–3 are independent of each other in implementation, though 3 should be *designed* alongside
§3.1 since they share the seam.

---

## 6. Red controls

Adopting the brief's, and adding the ones the analysis surfaced:

- **The discriminating one, from the brief:** a genuine refusal — the projector made absent, or a
  real 503 — **must still raise the banner with its typed reason**. A fix that silences both is a
  regression wearing a fix's clothes.
- A deploy performed while a browser is watching produces no refusal banner and no gap in the
  workflow row.
- **Added (slice 2):** a deploy where the unit file *did* change must still restart. The noop path
  must not become an absorbing fallback in the other direction.
- **Added (slice 2):** a deploy where the observed set **cannot be read** must refuse, typed and
  located — not restart-everything, and not skip-everything.
- **Added (slice 3):** the unit must not report ready before `/healthz` answers. This is directly
  falsifiable: `systemctl is-active` and a curl must agree at every instant.
- **Coupling, not scope:** ~20% of deploys overlap another deploy (§1). Any handover model must
  state its behaviour under a concurrent deploy rather than assume exclusivity. The pre-existing
  `ServedSurfaceStale` race is the known instance.

---

## 7. Open questions for sign-off

1. **Item 1's wording.** Do you accept the narrower arm — *"not answering"*, styled transient —
   rather than *"deploying, back shortly"*? The latter asserts a cause the client cannot establish.
   If you want *deploying* specifically, that needs a grounded deploy-in-progress fact on the wire,
   which I would scope as a separate ticket rather than smuggle into item 1.
2. **Item 2's re-cut.** Do you agree item 2 is better framed as a §3 de-fork (readiness modelled
   twice, authoritative copy wrong) and as item 4's prerequisite, rather than as a standalone outage
   fix? Its independent displaced cost today is small because the deploy already routes around it.
3. **Item 3's fix shape.** Confirm the fix is *supply the observed set* (construction) and not *add
   a changed-predicate to the restart step* (validation). This is the one with real blast radius: it
   makes the spine's refusal arms live on a path that today always applies.
4. **The seed change.** Is `listener acquisition as a §2 Realization` (SelfBound | Inherited, with
   `sd_notify` on the same seam) an acceptable way to touch the seed for slices 3–4? If the answer
   is that the seed should not grow host integration at all, slice 4 needs a different candidate and
   I would want to know that before designing it.
5. **The 208 figure.** Where does it come from? It matches neither the 91-source closure nor the
   2,356-file read in the tree's own measurement. Worth pinning before the deep fix is sized.
6. **srv1 access.** I could not reach srv1 to count restarts from the journal (§1). If you want that
   confirmation rather than the deploy-side count, I need a path in.

---

## Provenance

Grounded against the tree at `a07d1b73f8` (2026-07-30). Receipts: `dag/gunbc/roadmap_component.dag`
(:2463, :2507, `workflow_observation_poll_interval`, `workflow_poll_single_flight_note`),
`dag/gunbc/live_deploy/emit.dag` (:110, :123, :158, :283–289, `live_deploy_reconcile_binding_note`),
`dag/gunbc/live_deploy/service_ready.dag` (`live_deploy_service_ready_poll_bound_reason`,
`live_deploy_roadmap_unit_expected_startup`), `src/v1/stage0/src/cli_run.rs` (:11885, :11893, :11918),
`dag/gunbc/roadmap_authority.dag` (:948). Deploy-job count: GitHub Actions, `deploy_dashboard_srv1`,
24h window ending 2026-07-30T20:11Z.
