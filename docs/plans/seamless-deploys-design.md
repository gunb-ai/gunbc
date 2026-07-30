# Seamless deploys — design note

**Status: DRAFT, modeling-first. No code lands from this note.** The brief asks that the modeling be
grounded and agreed before implementation; this note does the grounding, proposes the model, and
names the decisions that need operator sign-off. Every claim below carries a receipt against the
live tree or a measurement, or is marked as an open question.

Brief: *Deploying should not take the dashboard away.* Boundary: the restart path only — the serve
process, its unit, and how the browser is told what it is seeing. Not what the deploy installs.

**Revision, 2026-07-30 (review 45229).** Two central re-cuts in the first draft prescribed models
that could not establish the behaviour they claimed, and both are corrected in place with the
correction recorded rather than silently restated:

- *Item 3* claimed the observed set could be supplied with "no new comparison logic." It cannot —
  `DeploymentArtifactStep` carries `{kind, path}` and no content identity, so that change alone
  inverts into a **silent never-deploy**. Corrected again after review 45232: content identity is
  *also* not sufficient, because the restart is fused to the `SystemdUnit` member, so a binary-only
  change would install the binary and never restart. Item 3 is now 2a (comparable value) → 2b
  (de-fuse the running service from the unit file) → 2c (supply the provider), and the order is
  load-bearing at every seam (§2 Concept C).
- *Item 2* was framed as a §3 de-fork of one readiness fact. There are **two** facts; the tree's own
  `service_ready_means_serving_this_tree_note` and the dated 2026-07-24 incident say so. The digest
  check is irreducible and must survive slice 3 (§2 Concept B).
- *Item 4's dependency* was stated inconsistently (review 45241) — one section had handover
  downstream of both readiness and reconciliation, while two others said item 3 is unnecessary for
  the headline. The single authoritative statement is now the dependency graph in §5: **slice 4
  requires slice 3, and nothing else in this ticket gates anything.**

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

### Concept B — *Two readiness facts, one of them wrongly asserted* (item 2, and it is the keystone for item 4)

**Corrected (review 45229).** An earlier draft of this section called readiness "modeled twice" and
proposed `sd_notify` as a §3 *de-fork* that would let the healthz poll be argued down. That framing
is wrong, and the tree says so in its own words. Recording the correction rather than quietly
restating, because the mistake is instructive: **the draft committed the same state-space
conflation it was accusing systemd of.**

There are **two distinct facts** here, not two representations of one:

- **F1 — process-bind readiness.** *A process on this unit has acquired the listener and can
  answer.* systemd asserts this today via `Type=simple`, and its assertion is **false by ~35–40s,
  every time** — ready ≡ spawned. This one *is* wrongly modeled, and `sd_notify(READY=1)` at the
  bind (`cli_run.rs:11918`) is a genuine single-authority fix for it.
- **F2 — deployment surface identity.** *The answering process is serving the tree this deploy
  installed.* Only the digest comparison establishes this. **systemd can never know it**, so it is
  not a duplicate of F1 and it never dissolves.

The tree already draws this line explicitly, and paid for it. `readiness.dag`'s
`service_ready_means_serving_this_tree_note` records that READY was under-specified and the gap was
*live*: because `gunbc serve` binds its compiled graph **once at process start**, the **pre-restart
process keeps answering** during the new one's ~35s load — so a poll grounding on *any* answer
greens against the very process the deploy is replacing. On 2026-07-24 srv1 served a stale surface
with every check in the system green. Readiness was deliberately strengthened to *answering **and**
the surface is this tree's*.

That receipt is fatal to the de-fork framing, and it carries a warning for slice 3 specifically:
**`sd_notify` fires when the new process binds, which is exactly the moment F2 is still unproven.**
A notify-based readiness signal is not a stronger version of the digest check — on the axis that
caused the 2026-07-24 incident it is *weaker* than what exists today. So:

- The overlap between systemd's claim and the poll is confined to F1 — and within the poll, to its
  `HealthzProbeFailed` arm only. `HealthzBodyUnparseable` and `HealthzSurfaceStale` are irreducible.
- **The digest check must survive slice 3 unchanged.** Any implementation that treats `Type=notify`
  as licence to drop or weaken it re-opens a known, dated, live incident. This is a red control in
  §6, not a caution.

**What item 2 is actually worth, stated honestly.** Its value is (a) systemd stops lying to its own
consumers about F1 — today the misled parties are a human at `systemctl status`, systemd's ordering
and `Restart=` semantics, and any future consumer (I checked `host_hygiene_reaper.dag` and
`oomd_install.dag`; neither targets this unit, so the blast radius today is small); and (b) it is
the **prerequisite for item 4**, because you cannot sequence a handover without a trustworthy
"the replacement has bound" signal, and the healthz poll cannot serve that role — it runs in the
deploy job, not on the host. What item 2 is *not* is a way to dissolve the poll or to stop
re-calibrating the bound; only the deep fix in §4 does that.

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
The direction is to **supply the observed set** so that `Unchanged → noop` is reached by the spine
that already implements it.

**Correction (review 45229): supplying `observed` alone does not work, and the naive version fails
dangerously.** An earlier draft of this section claimed the observed set could be supplied with "no
new comparison logic, no new authority." That is wrong, and the reason is one level down in the
model. `spec.dag:38–41` declares:

```
type DeploymentArtifactStep { kind: OwnedArtifactKind, path: NonEmptyStr }
```

The member value carries **kind and path only — no content identity.** `deployment_step_value_eq`
is `a == b` over exactly that. So if the observed set is supplied as the same members at the same
paths, `value_eq` returns **true for every member, always** — the binary changed, the tree changed,
the unit file changed, and the diff still reports `Unchanged`. The failure is not "cannot
distinguish changed from unchanged"; it is an **inversion into a silent never-deploy**, which is
strictly worse than today's always-restart because it fails closed on nothing and green on
everything.

That is also precisely the shape DESIGN §5 names as the tell that a check was validation standing
where construction was available: *it can be satisfied by editing the declaration while the
realization still lies.* A `value_eq` over `{kind, path}` is a key-completeness claim whose realizer
is faking the key.

**Second correction (review 45232): content identity alone still does not restart the service.**
Reconciliation is **per member**, and the restart of `gunbc-roadmap.service` exists in exactly one
place — `emit.dag:288`, inside the `SystemdUnit` upsert arm. (The other restart in the module,
`:219`, targets `gunbc-tree-sync.service`, a different unit.) So under a real diff:

> binary content changes → `ServeBinary` is Modified → its arm installs the new binary → **but**
> the unit file is unchanged → `SystemdUnit` is `Unchanged` → noop → **no restart**. New binary on
> disk, old process still serving the old one.

That is the same silent non-deploy in a second costume, and the discriminating control added after
review 45229 — *a binary-only change must still restart* — **would fail against the design as
written**. The re-cut omitted the dependency that makes its own control pass.

**The root is a conflation in the existing model, which the degenerate pole has been hiding.**
`SystemdUnit` names two different things at once: the **unit file** (an artifact on disk, whose
value is its text) and the **running service** (a process, whose correctness depends on the binary
and tree it was started from). Restart got fused onto the file — `emit_deploy_member_effect_note`
says so plainly: *"a SystemdUnit upsert carries its own daemon-reload/enable/restart (once, part of
the unit's realization)"*. That fusion is harmless while every member always applies, because the
restart then always happens anyway. **The moment the diff becomes real, the fusion is the defect.**

The dependency itself is already known — it is just prose. `deployment_apply_order_note` states:
*"all before the unit (ExecStart references binary + tree)"*. So the fact exists at a single
authority in English and nowhere in the model.

**The construction answer — derive the restart from the service's inputs, don't attach it to a
file.** Model the running service as its own member whose value is a function of the identities it
was started from (unit file text + binary identity + tree identity). Then a binary change *changes
the service member's value*, so it is `Modified`, so it restarts — **by construction**, with no
impact table and no `restart_required_by` adjacency list (which would be a second representation of
the dependency the apply order already asserts). The bad state — new binary installed, old process
serving — becomes **unwritable** rather than checked for.

**So item 3 is three steps, and the ordering is load-bearing at every seam:**

- **2a — model the comparable artifact value at its single authority.** `DeploymentArtifactStep`
  must carry the content identity that makes two installations comparable (a digest for the binary
  and the unit file; the tree's identity for the source tree). Note the `ContentHash` family
  ambiguity recorded in DESIGN's open threads is live here: whichever family is chosen, the carrier
  must say which it requires rather than leaving it to prose.
- **2b — de-fuse the running service from the unit file**, giving the service a member whose value
  derives from the 2a identities. This is what makes a restart a *consequence* of its inputs.
- **2c — supply the observed provider** reading those identities off the host.

Failure modes of getting the order wrong, both of which typecheck: **2c without 2a** reports
`Unchanged` for everything (never deploys); **2c without 2b** installs new artifacts and never
restarts (deploys the files, not the service). Neither is caught by a type; both are caught by the
§6 binary-only control, which is why that control is the acceptance bar for item 3 rather than a
nicety.

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

### Item 4 (handover) is downstream of B **only**; C is an independent scope reducer

**Corrected (review 45241).** An earlier draft headed this section *"downstream of B and C"* and
treated the two jointly. That contradicted two later statements in this same note — that item 2 is
the handover prerequisite (§2 Concept B) and that item 3 is *not* required for the headline outcome
(§7 q3) — and, read as a plan, it would have gated the cheap handover work on the expensive
carrier changes. Stated unambiguously, once:

- **B is a prerequisite.** Handover means *move traffic when the replacement is ready*, and there is
  no trustworthy "the replacement has bound" signal without it. Item 4 cannot be built correctly
  first.
- **C is not a prerequisite. It is an independent scope reducer.** C changes *how often* a handover
  runs, never *whether it works*. A handover built with C still outstanding is correct — it simply
  performs on all ~40 deploys/day rather than on the subset that changed something. Nothing in item
  4's design reads any reconciliation fact.

So the dependency edge is **4 → B**, and C sits beside it. This matters for sequencing because C is
now the most expensive item in the ticket (two load-bearing carrier changes, §2 Concept C) and the
only one that does not serve the headline: gating item 4 on it would buy nothing and cost the most.

Socket activation (§3) is then one candidate realization for covering the residual window, not the
whole answer.

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

**The dependency graph, stated once and authoritatively** (slice labels are names, not an order):

```
slice 1                  — no dependencies
slice 3  →  slice 4      — the ONLY hard edge in this ticket
slice 2a →  2b →  2c     — internally ordered, and independent of 1, 3, 4
```

There is exactly **one** cross-item dependency: **slice 4 requires slice 3**, because a handover
needs a trustworthy "the replacement has bound" signal. Everything else is independent. In
particular **slice 2 (item 3) gates nothing** — it is a scope reducer that changes how often a
handover runs, never whether it works (§2, *Item 4 is downstream of B only*).

**Recommended order — headline first, cost last:**

1. **Slice 1 — de-conflate the observation outcome (item 1).** Give the transport arm its own
   sentence, stating what was observed and not asserting a cause. No timing change, no deployment
   change, no seed change. The brief's `first_slice`, and correct regardless of what follows.
2. **Slice 3 — correct systemd's F1 assertion (item 2)**, designed with the listener-acquisition
   realization (§3.1). `Type=notify` + a readiness signal at the bind point, modelled as a
   realization rather than cemented. **Does not touch the digest check** (§2 Concept B).
3. **Slice 4 — the residual window (item 4).** Socket activation, evaluated against §3's three
   decisions, with the staleness cue. This is where the headline outcome is actually delivered.
4. **Slice 2a/2b/2c — item 3**, in that internal order, whenever it is worth its cost:
   - **2a** give `DeploymentArtifactStep` the content identity that makes two installations
     comparable (load-bearing carrier change);
   - **2b** de-fuse the running service from the unit file, so a restart is a consequence of its
     inputs rather than a side-effect of writing a file;
   - **2c** supply the observed provider, with "could not observe" refusing typed/located/counted.

   **The internal order is load-bearing** (§2 Concept C): 2c without 2a never deploys; 2c without 2b
   deploys the files but not the service. Both typecheck, so neither failure is caught by a type.

Slices 1 and 3 are independent of each other; 3 should be *designed* alongside §3.1 since they share
the seam.

**Depends on, does not duplicate:** `v1-materialization-kernel` (§4) — for the headline outcome at
the limit, not for any slice above.

---

## 6. Red controls

Adopting the brief's, and adding the ones the analysis surfaced:

- **The discriminating one, from the brief:** a genuine refusal — the projector made absent, or a
  real 503 — **must still raise the banner with its typed reason**. A fix that silences both is a
  regression wearing a fix's clothes.
- A deploy performed while a browser is watching produces no refusal banner and no gap in the
  workflow row.
- **Added (slice 2 — the acceptance bar for item 3, not a nicety):** a deploy in which **only the
  binary content changed** — same kind, same path, unit file untouched — must **install the new
  binary AND restart the service**, and the live process must afterwards be the new binary. This
  single control discriminates *both* defects found in review: it fails against a `{kind, path}`
  value (review 45229) and it fails again if the restart stays fused to the `SystemdUnit` member
  (review 45232). Its mirror: a deploy where genuinely nothing changed must *not* restart.
  Checking only that the new binary reached the disk is what makes both failures look green.
- **Added (slice 2):** a deploy where the observed set **cannot be read** must refuse, typed and
  located — not restart-everything, and not skip-everything.
- **Added (slice 3):** the unit must not report ready before it can answer. Directly falsifiable:
  `systemctl is-active` and a curl must agree at every instant.
- **Added (slice 3, regression guard):** the **surface-digest refusal must still fire** after
  `Type=notify` lands. Concretely, the 2026-07-24 shape — the pre-restart process still answering
  during the replacement's load — must still be refused as `HealthzSurfaceStale`. A notify signal
  proves F1 and says nothing about F2; a slice 3 that greens this case has re-opened a dated live
  incident.
- **Coupling, not scope:** ~20% of deploys overlap another deploy (§1). Any handover model must
  state its behaviour under a concurrent deploy rather than assume exclusivity. The pre-existing
  `ServedSurfaceStale` race is the known instance.

---

## 7. Open questions for sign-off

1. **Item 1's wording.** Do you accept the narrower arm — *"not answering"*, styled transient —
   rather than *"deploying, back shortly"*? The latter asserts a cause the client cannot establish.
   If you want *deploying* specifically, that needs a grounded deploy-in-progress fact on the wire,
   which I would scope as a separate ticket rather than smuggle into item 1.
2. **Item 2's re-cut** (revised after review 45229). Do you agree with the F1/F2 split — systemd's
   `Type=simple` wrongly asserts F1 and `sd_notify` fixes *that*, while F2 (serving this deploy's
   tree) is irreducible and the digest check survives untouched? The practical consequence is that
   item 2 buys the handover prerequisite, not a smaller poll.
3. **Item 3's cost** (revised twice — reviews 45229, 45232). The fix is *supply the observed set*
   (construction, not a changed-predicate), but it is gated on two prior modelling changes: content
   identity on `DeploymentArtifactStep`, **and** de-fusing the running service from the unit file so
   a restart derives from its inputs. Both touch load-bearing deploy carriers. This is no longer a
   cheap slice, and it is the item whose *partial* implementations are dangerous rather than merely
   incomplete. **Confirm those carrier changes are in scope for this ticket** — if they are not,
   item 3 should be dropped from the ticket entirely rather than attempted in any partial order,
   since every partial order typechecks and silently under-deploys. Item 3 is also the one item that
   is *not* required for the headline outcome: slices 1, 3, and 4 make deploys invisible; item 3
   only makes them rarer.
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
