# Seamless deploys — design note

**Status: DRAFT, modeling-first. No code lands from this note.** The brief asks that the modeling be
grounded and agreed before implementation; this note does the grounding, proposes the model, and
names the decisions that need operator sign-off. Every claim below carries a receipt against the
live tree or a measurement, or is marked as an open question.

Brief: *Deploying should not take the dashboard away.* Boundary as originally written: the restart
path only — the serve process, its unit, and how the browser is told what it is seeing; *not what
the deploy installs*.

**Boundary amendment, 2026-07-30 (operator direction).** That original boundary predates the
direction that deploy should reconcile to intent and apply the minimal items (§2 Concept C), which
widens it. Stated precisely, because the two halves are easy to conflate and item 3 sits across the
seam:

- **Now in scope — how the desired set is applied.** Whether an unchanged member is re-applied, and
  the member modelling that makes "unchanged" decidable (content identity, and the running service
  as a member distinct from the unit file). This is item 3, and it is required work.
- **Still out of scope — the deployed *product*.** What the dashboard does, what it shows beyond the
  honesty fixes the brief itself asks for, and which artifacts constitute the deployment as a
  product decision. Plus the brief's own exclusions: no dispatch or belt changes, and no attempt to
  make the service restart-free in general — only invisible to a viewer.

**Correction (review 45289): an earlier version of this amendment said the ticket "adds no member and
changes no artifact's contents." That is false, and false for nearly every item in it** — an
over-tightening introduced while fixing the previous boundary finding. Implementing this ticket
necessarily changes deployed content:

- **item 1** edits `roadmap_component.dag`, which emits the dashboard's JS — a change to the served
  tree, and the brief explicitly puts *"how the browser is told what it is seeing"* **in** scope;
- **item 2** changes the unit file's own text (`Type=notify`) and the seed binary (`sd_notify`) —
  both deployment artifacts;
- **item 4** may **add a `.socket` unit**, which is a new deployment *member*, not merely new
  contents.

So the honest boundary is not "no artifact changes" — it is that the ticket does not change **what
the deploy is for**. The reviewer reached this through the staleness cue, which is the mildest
instance; the socket unit is the one that actually adds a member. The cue itself stays: it falls
under the brief's own in-scope clause about how the browser is told what it is seeing, and it exists
to stop socket activation from silently showing stale data (§3).

*Coordination consequence, not a new dependency:* if slice 4 adds a `.socket` member **and** slice 2
has made membership non-degenerate, that member needs the same bundle as its siblings — identity,
a content-sensitive value, and an ownership stance. Neither slice gates the other (slice 4 can add
it under today's degenerate apply; slice 2 can land before the member exists), so the §5 graph is
unchanged — but whichever lands second inherits the join, and it should not be discovered then.

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

**Operator direction, 2026-07-30** — deploy should use the existing apply/delete/reconcile process
(bmc/srvN apply) and deploy the minimal items needed to reach intent. This answers §7 q3: item 3 is
**in scope and not droppable**, and it is instantiation of a spine several siblings already drive,
not new mechanism (§2 Concept C).

Earlier corrections, retained:

- *Item 4's dependency* was stated inconsistently (review 45241) — one section had handover
  downstream of both readiness and reconciliation, while two others said item 3 is unnecessary for
  the headline. The single authoritative statement is now the dependency graph in §5: **slice 4
  requires slice 3, and nothing else in this ticket gates anything.** *(Superseded by review 45297:
  that last edge was itself unsupported — socket activation queues in the kernel and consumes no
  READY signal, so **item 4 has no prerequisite either**. The ticket now has zero cross-item
  dependencies; see §5.)*

**A note on how this note cites.** Receipts below name **module + declaration**, not `file:line`.
That is not a style preference — it is a defect this document demonstrated on itself. The first
draft cited positions, and within a day **5 of its 7 positional receipts pointed at the wrong
declaration**: `emit.dag:123` (cited for `Type=simple`) had drifted onto a `Description=` line,
`:158` (the degenerate `observed: []`) onto an unrelated ownership helper, `:288` (the roadmap
restart) onto the tree-sync restart, `spec.dag:38` onto a coproduct arm, and `cli_run.rs:11918`
(the bind) onto a `println!`. Every underlying *claim* still held — only the positions moved, as
main advanced. A document whose entire value is traceable evidence cannot carry receipts with that
half-life, so positions are retained only where no named declaration exists.

---

## 1. Grounding — what the tree actually says

The brief's four items were checked against the tree. Three confirm exactly, and one confirms but
sharpens into a different fix. Of the two side claims: the restart count resolves *opposite* to the
brief's own caveat, and the "208 sources" figure — which an earlier draft of this note wrongly
disputed — is **correct**, with the tree's competing figure being the stale one.

| Brief claim | Verdict | Receipt |
|---|---|---|
| (1) Transport failure and typed refusal render identically | **Confirmed** | `dag/gunbc/roadmap_component.dag` `workflow_fetch_request_statements` — its `.then` arm (guarded by `wj.observe_refused`) and its `.catch` arm both build the literal `"workflow observation refused: "`, then both call `workflow_refuse_visible_stmt()` and `observation_refused_banner_stmts(source_class: "workflow-observe-refused")` — same prefix, same banner, same source class |
| (2) `Type=simple`, so systemd reports ready before it is | **Confirmed** | `dag/gunbc/live_deploy/emit.dag` `emit_systemd_unit_doc` emits `Type=simple`; the seed has **no** systemd integration at all — zero hits for `sd_notify`, `NOTIFY_SOCKET`, or `LISTEN_FDS` anywhere in `src/v1/stage0/src/` |
| (3) The deploy restarts unconditionally | **Confirmed, and the cause is upstream of the arm** — see §2 | `emit.dag` `emit_artifact_upsert`, `SystemdUnit` arm: a fixed 5-step list ending in daemon-reload → enable → restart, with no predicate |
| (4) No handover | **Confirmed** | One unit, one `ExecStart`, `systemctl restart` in place; nothing holds the socket or overlaps instances |
| Outage is 60–90s | **Confirmed and calibrated** | `live_deploy_roadmap_unit_expected_startup = 40s`; poll bound = 40 × 3 = 120s; four independent srv1 journal observations recorded at 35s, 36s, 36s, 39s to listening, all successful starts (`service_ready.dag`, `live_deploy_service_ready_poll_bound_reason`) |
| Browser polls every 2s | **Confirmed** | `workflow_observation_poll_interval = millisecond(count: 2000)` |
| Bind happens after the compile | **Confirmed, with the four exit paths** | `cli_run.rs` `handle_serve`: `load_sources_for_entry` → `compile_to_resolved` → `TcpListener::bind`, in that order. Four distinct `exit(1)` paths precede the bind |

### Two corrections

**"208 sources" is correct — this note's earlier objection to it was wrong, and the tree's figure is
the stale one.** An earlier draft said 208 matched neither figure on record and asked where it came
from. Settled by measurement instead of left as a question — running the unit's exact ExecStart
(`gunbc serve --entry dag/gunbc/roadmap_serve.dag --function roadmap_serve_handle`, on a spare port)
prints:

```
[t+56s] resolved 208 sources
[t+59s] ✓ compile.frontend done in 3 seconds
[t+59s] ✓ compile.normalize done in 298ms
[t+74s] ✓ compile.reconcile done in 14 seconds
[t+74s] ✓ compile.analyses done in 213ms
[t+74s] gunbc serve listening -> roadmap_serve_handle()
```

So the brief's 208 is the real resolved-closure count, and
`live_deploy_service_ready_poll_bound_reason`'s *"a closure of 91"* is stale or measuring something
else. **The tree's figure is the one to distrust, not the brief's.**

The phase split is the more valuable half of that measurement, and it **confirms the load-dominant
diagnosis**: **56s of the 74s — 76% — elapses before `resolved 208 sources` even prints**, i.e. in
the load phase, with the entire compile (frontend + normalize + reconcile + analyses) accounting for
18s. Within the compile, `reconcile` is 14s of the 18s.

**Honest caveat on the wall-clock number:** 74s was measured on this build box, which is not srv1
and was concurrently running builds. It is *not* comparable to the four srv1 observations of
35/36/36/39s recorded in the tree, and it must not be read as "startup regressed to 74s". What *is*
machine-independent is the source count (208, exact) and the load-vs-compile *ratio*. What it does
suggest is that `live_deploy_roadmap_unit_expected_startup = 40s` deserves re-measurement on srv1,
since **corpus today is 2,719 `.dag` files** — up ~15% from the 2,356 that expectation was
calibrated against nine days ago — and the load term grows with the corpus exactly as that note
predicted.

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

### Concept B — *Two readiness facts, one of them wrongly asserted* (item 2)

**Corrected (review 45229).** An earlier draft of this section called readiness "modeled twice" and
proposed `sd_notify` as a §3 *de-fork* that would let the healthz poll be argued down. That framing
is wrong, and the tree says so in its own words. Recording the correction rather than quietly
restating, because the mistake is instructive: **the draft committed the same state-space
conflation it was accusing systemd of.**

There are **two distinct facts** here, not two representations of one:

- **F1 — process-bind readiness.** *A process on this unit has acquired the listener and can
  answer.* systemd asserts this today via `Type=simple`, and its assertion is **false by ~35–40s,
  every time** — ready ≡ spawned. This one *is* wrongly modeled, and `sd_notify(READY=1)` at the
  bind (`cli_run.rs` `handle_serve`) is a genuine single-authority fix for it.
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
`oomd_install.dag`; neither targets this unit, so the blast radius today is small); and (b) it gives every
*other* consumer of "is it up" an answer it can trust — systemd's own ordering, a human at
`systemctl status`, and any future dependent unit.

**What item 2 is not** (corrected after review 45297): it is **not** a prerequisite for item 4. An
earlier version of this paragraph claimed it was, on the reasoning that a handover cannot be
sequenced without a trustworthy "the replacement has bound" signal. That is true of a *two-instance*
handover, which this note does not propose — under socket activation the kernel queues and nothing
reads READY (§2, *Item 4 depends on nothing in this ticket*). Nor is item 2 a way to dissolve the
healthz poll or stop re-calibrating the bound; only the deep fix in §4 does that.

### Concept C — *The deploy does not reconcile; it applies* (item 3)

The brief says the `SystemdUnit` arm restarts *"whether or not the unit, binary, or tree changed."*
That is true, but the cause is one level up, and it matters because it changes the fix from a
validation to a construction.

`emit.dag` `deployment_apply_plan` — the apply pole calls the grain-agnostic spine with **`observed: []`**:

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
model. `spec.dag` `DeploymentArtifactStep` declares:

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
place — `emit.dag` `emit_artifact_upsert`'s `SystemdUnit` arm. (The other restart in the module,
`tree_sync_restart_step_with_diagnosis`, targets `gunbc-tree-sync.service`, a different unit.) So under a real diff:

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

#### Operator direction (2026-07-30): deploy is a reconcile toward intent, minimal items

> *"i think i'd like deploy to use our existing apply/delete/reconcile type process (i.e. bmc/srvN
> apply) — my point is, i want to deploy the minimal possible items to update to intent."*

This **resolves §7 q3 and re-prioritises the ticket**: item 3 is not optional work to be dropped if
it proves expensive — it is the deployment ask, and "minimal possible items to update to intent" is
exactly the spine's `Unchanged → noop`. The vocabulary already lines up one-to-one: **apply** =
`MemberAdded`/`MemberChanged`, **delete** = `MemberRemoved` (owned-only, else a typed refusal), **reconcile** =
the diff itself, with unchanged members producing no hunk and therefore no effect.

The good news is that this is **instantiation, not invention**. `live_deploy` is the *only*
degenerate consumer of a spine that several siblings already drive non-degenerately, and both halves
of the missing machinery have working precedent:

- **The comparable value (2a) — precedent `gunbc.host_authorized_keys_reconcile`.** Its
  `authorized_key_value_eq` compares *content* (algorithm + material + comment) while `key_of`
  returns *identity* (the key material), so a content drift is `Modified → re-upsert` rather than
  `Remove + Add` — the spine's note calls this out as the reason identity and value must differ.
  That is precisely the split `DeploymentArtifactStep` lacks: it has identity (`path`) and no value.
- **The observation (2c) — precedent `gunbc.tool_readiness`, live today.** It reconciles a desired
  `Pin<CliTool>` against an observed one derived from
  `extdeps.realization.emit_on_demand_host.observed_tool_identity`, which reads a real per-tool
  digest off the host and returns **three typed outcomes** — `Found` / `Missing` / `Duplicate`. Its
  `observed_pin_projection_note` also solves the exact projection problem deploy has: the
  observation carries only a digest, so the observed member is built by taking the desired member
  and replacing *only* the observed field. Deploy wants the same move — observe the artifact's
  content identity, keep everything else from desired.
- **The apply site — `gunbc.host_effect_realize`** already runs reconciles inside srvN apply:
  `srv3_toolchain_ensure_reconcile` (called from
  `srv3_realize_os_install_actuator_toolchain_ensure_body`) and
  `provision_build_cache_reconcile` (called from `realize_provision_build_cache_body`). That is the
  "bmc/srvN apply" process named in the direction.

So 2a and 2c are **patterned work**, not new mechanism.

**On 2b — an earlier statement here was too pessimistic and is corrected.** It said no sibling has
an artifact whose realization is a running process. That is wrong: `gunbc.roadmap_belt` reconciles
**live dispatch sessions** — running processes — with a genuine observed provider
(`belt_observed_members(live: List<DispatchLiveSession>)`), ownership lifted from an actuator's
observation, and R5 teardown meaning *reap a live session* (refusing when it cannot prove it owns
one). So process-as-member, live observation of processes, and owned-only teardown of a running
thing are **all precedented**.

The genuinely new cell is narrower, and naming it precisely is what makes 2b tractable:

|  | inert artifact | running process |
|---|---|---|
| **presence-only value** | — | `roadmap_belt` (`dispatch_member_value_eq` is constant `true`) |
| **content-sensitive value** | `host_authorized_keys_reconcile`, `tool_readiness` | **← deploy needs this; nothing here yet** |

The belt's member value is deliberately degenerate — `dispatch_member_value_eq(a, b) = true`, so it
never produces a `Modified` hunk. A session either exists (`Unchanged`), is missing (`Added` →
spawn), or is extra (`Removed` → reap). It has no notion of *this running thing is stale relative to
the inputs it was started from* — which is exactly deploy's requirement, and exactly the axis on
which today's always-restart is hiding.

So 2b is the **composition of two existing patterns** (belt's process-member machinery + the
siblings' content-sensitive value), not virgin territory. That is a materially smaller and
lower-risk piece of work than "no precedent" implied, and the design attention it needs is on one
question: *what are the running service's inputs, such that its value changes exactly when a restart
is genuinely required* — answered by 2a's identities (unit file text, binary identity, tree
identity).

**Two decisions this direction surfaces, which the precedents deliberately leave to a human:**

1. **Observed scope, and therefore delete semantics.** `host_authorized_keys_reconcile` explicitly
   declines to fix apply-grain policy because authorized_keys is an *everything-on-host* scope where
   foreign keys are normal, so a foreign member's refusal must not block an owned upsert. Deploy is
   the opposite: its members are a **closed set of owned artifacts at known paths**, so the observed
   scope is bounded and today's wholesale-refuse policy is defensible. But it must be *stated*: with
   a real observed set, `Removed → teardown` becomes reachable on the apply path for the first time,
   where today teardown only exists in the retract pole.
2. **What `Missing` means.** An artifact absent from the host is `Added → upsert` (correct — install
   it). An observation that *could not be taken* is **not** that, and must refuse typed/located/
   counted rather than degrade to "assume absent, reinstall everything" — which would be the
   absorbing fallback wearing this ticket's own clothes, and would silently restore today's
   always-apply behaviour while looking like a reconcile.

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

### Item 4 (handover) depends on nothing in this ticket

**Corrected twice, and the second correction removes the last edge.** Review 45241 fixed a heading
that read *"downstream of B and C"*; review 45291's drain finding then made the residual mechanism
concrete; and **review 45297 showed the remaining `4 → B` edge was never real either.**

**Why the false edge existed, since the mistake is instructive.** I wrote that handover means *"move
traffic when the replacement is ready"*, which is true of a **two-instance** handover — run the new
one alongside, switch traffic when it signals READY. That mechanism genuinely consumes `sd_notify`.
But it is not the mechanism this note proposes. Under socket activation **the kernel does the
queueing and there is no switch to time**: systemd holds the listening socket across the restart,
new connections accumulate in the backlog, and the replacement drains them when it reaches
`accept()`. Nothing reads READY. I inherited the dependency from a generic notion of handover rather
than deriving it from the realization I actually chose — which is precisely the error of asserting a
dependency the mechanism does not have.

So, stated once and authoritatively: **item 4 has no prerequisite in this ticket.**

- **B is not a prerequisite.** `sd_notify` makes systemd's F1 assertion honest, which is worth doing
  on its own merits (§2 Concept B) and is what lets *other* consumers trust "is it up" — but no part
  of the socket-activation handover consumes it. Item 4 can be built first.
- **C is not a prerequisite** — it is independent, and in *function* a scope reducer (which says
  nothing about its priority: it is in scope and not droppable, §2 Concept C). C changes *how often*
  a handover runs, never *whether it works*. Nothing in item 4's design reads a reconciliation fact.

**Coordination consequence worth carrying into slice 4's build, not a dependency.** Socket activation
changes the *failure mode* of the deploy's own readiness probe. Today an unbound port gives
connection-refused, which is `HealthzProbeFailed`. With systemd holding the socket, a probe's
`connect()` succeeds immediately and the request then **waits** for the replacement to reach
`accept()` — so the refusal that fires during a slow start changes shape, and the poll's behaviour
becomes dependent on the probe transport's timeout rather than on a prompt refusal. That wants
verifying against the actual probe transport when slice 4 is built; it is not established here, and
it is an argument for doing slice 3 *alongside* slice 4 (so the deploy has an honest READY signal
instead of inferring from a blocking probe) — an argument about convenience, not about dependency.

Socket activation (§3) is then one candidate realization for covering the residual window, not the
whole answer.

---

## 3. Socket activation — evaluating the brief's preferred candidate

The brief asks that queueing behaviour be confirmed as desirable before building. Assessment:

**It works for *new* connections.** A `.socket` unit makes systemd own the listening fd; it survives
the service restart, so connections land in the kernel accept backlog instead of getting
ECONNREFUSED. Combined with single-flight polling (`workflow_poll_single_flight_note` — at most one
workflow observation in flight per browser), queue depth is bounded by viewer count, not by
viewers × 30 polls. The brief's reasoning here is correct as far as it goes.

**But it does not deliver the handback on its own (review 45291), and this is the important
correction in this section.** The backlog only holds connections that have *not yet been accepted*.
A fetch the old process has **already accepted** dies with the process — the client sees a reset,
the browser's `fetch` rejects, and that lands in exactly the `.catch` arm this ticket is trying to
quiet (`workflow_fetch_request_statements`). Verified in the seed rather than assumed:

- `handle_serve` installs **no** SIGTERM handler — grep for `SIGTERM` in `cli_run.rs` returns
  nothing;
- the only SIGTERM machinery in the seed lives in `phase_profile`, is gated on profiling being
  enabled, and even when it fires it calls `std::process::exit(143)` after flushing — it does not
  drain connections;
- so `systemctl restart` → default SIGTERM disposition → immediate termination, mid-request.

The exposure is small per deploy but not zero: with a 2s poll, roughly *request-duration / 2s* of
viewers are mid-flight at the restart instant. Across ~40 deploys/day it will fire. And the brief's
handback is *"a deploy run against a live viewer with no visible interruption"* — a rare banner
still fails that, so **socket activation alone does not meet the acceptance bar.**

**Closing it requires a drain**, which is more seed work on the *same seam* as the other two: on
SIGTERM, stop accepting, finish in-flight requests, then exit — bounded by `TimeoutStopSec` so a
hung request cannot stall the deploy. That makes the seed-integration question (§7 q4) a **three-part
seam**, not one change: `LISTEN_FDS` (inherit the listener), `sd_notify` (honest readiness), and a
SIGTERM drain (graceful handover). All three are the same "the process talks to systemd" boundary,
and answering q4 *no* forecloses all three, not just socket activation.

**Three further things to decide before it is built:**

1. **It requires a seed change, and the seed is supposed to shrink.** The process currently *binds*
   (`TcpListener::bind`, in `cli_run.rs` `handle_serve`). Socket activation requires it to *inherit* fd 3 via
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
(`roadmap_authority.dag`, the node row with id `v1-materialization-kernel`, owner `self-host`, path
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
slice 3                  — no dependencies
slice 4                  — no dependencies
slice 2a →  2b →  2c     — internally ordered; the ONLY ordering in this ticket
```

**There are no cross-item dependencies.** The last one — `slice 3 → slice 4` — was removed after
review 45297 showed it was never supported by the proposed mechanism: socket activation queues in
the kernel and nothing consumes `sd_notify`'s READY (§2, *Item 4 depends on nothing in this
ticket*). The only ordering that survives is **internal to slice 2**, and there it is load-bearing:
2c without 2a never deploys, 2c without 2b deploys the files but not the service.

Consequences for planning: any of 1, 3, 4 can be built first or in parallel, **slice 2 gates
nothing**, and nothing gates slice 4 — so the headline work is reachable immediately, subject only
to the seed decision (§7 q4) that all of slices 3 and 4 rest on. Two coordination notes that are
*not* edges: slice 4 changes the readiness probe's failure mode (§2), and a `.socket` member added
by slice 4 needs the same bundle as its siblings if slice 2 has landed (boundary amendment).

**Recommended order — revised 2026-07-30 by operator direction.** An earlier version ordered this
headline-first/cost-last and offered to drop item 3. That is superseded: deploy reconciling to
intent with minimal items is an explicit ask (§2 Concept C), so slice 2 is **in scope and not
droppable**. It still gates nothing — the graph above is unchanged, and slices 1/3/4 need not wait
for it — but it is no longer the item to cut under pressure. Slice 1 stays first because it is
hours of work and independently correct; slice 2 can proceed in parallel with 3/4 since they share
no seam.

**This numbering is preference, not requirement** (sharpened after review 45297, which removed the
last dependency): with zero cross-item edges, *any* of these can go first. In particular **slice 4
need not wait for slice 3** — if the headline is the priority, slice 4 is the slice that delivers it
and can be built immediately. The order below simply front-loads the cheapest independently-correct
work. Slices 3 and 4 share the seed seam (§7 q4), so they are worth *designing* together even when
built apart.

1. **Slice 1 — de-conflate the observation outcome (item 1).** Give the transport arm its own
   sentence, stating what was observed and not asserting a cause. No timing change, no deployment
   change, no seed change. The brief's `first_slice`, and correct regardless of what follows.
2. **Slice 3 — correct systemd's F1 assertion (item 2)**, designed with the listener-acquisition
   realization (§3.1). `Type=notify` + a readiness signal at the bind point, modelled as a
   realization rather than cemented. **Does not touch the digest check** (§2 Concept B).
3. **Slice 4 — the residual window (item 4).** Socket activation **plus the SIGTERM drain** —
   the drain is not optional polish; without it an already-accepted fetch dies at restart and raises
   the very banner this ticket exists to remove (§3, review 45291). Evaluated against §3's three
   decisions, with the staleness cue. This is where the headline outcome is actually delivered.
4. **Slice 2a/2b/2c — item 3**, in that internal order. **Required, not discretionary** (operator
   direction, §2 Concept C). Listed last because it gates nothing and can run in parallel with
   1/3/4 — *not* because it is optional:
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
  workflow row. **This is the brief's handback control, and it is NOT established by socket
  activation alone** (review 45291): the backlog holds only unaccepted connections, so a fetch the
  old process already accepted dies with it and raises the banner. Meeting this control requires the
  SIGTERM drain in §3. Stated as a control on *slice 4 as a whole*, not on socket activation — and
  if the drain is out of scope, this control must be narrowed to "no banner for connections
  initiated after the restart began", which is a weaker promise than the brief asked for.
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
   item 2 buys an honest answer for systemd's own consumers — **not** a smaller poll, and (corrected
   after review 45297) **not** a prerequisite for item 4 either.
3. ~~**Item 3's cost** — is it in scope?~~ **ANSWERED 2026-07-30 (operator).** Deploy should use the
   existing apply/delete/reconcile process (bmc/srvN apply) and deploy the minimal items needed to
   reach intent. So item 3 is **in scope and not droppable**, and the two carrier changes it needs
   are authorised. Recorded in §2 Concept C with the sibling precedents it should follow —
   `host_authorized_keys_reconcile` (comparable value), `tool_readiness` (live observation), and
   `roadmap_belt` (process-as-member with live observation and owned-only teardown of a running
   thing). **All three sub-slices have precedent.** 2a and 2c are patterned work; **2b is the
   composition of two existing patterns** — belt's process-member machinery with the siblings'
   content-sensitive `value_eq` — occupying the one empty cell of that 2×2, not virgin territory
   (§2 Concept C carries the table and the measurement behind this).

   *Two sub-decisions this raises, both deliberately left to a human by the sibling modules
   (§2 Concept C):* whether deploy's bounded owned-artifact scope keeps today's wholesale-refuse
   policy now that `Removed → teardown` becomes reachable on the apply path, and confirmation that a
   *failed observation* refuses rather than degrading to reinstall-everything.
4. **The seed change — and it is a three-part seam, not one change** (widened after review 45291).
   May the seed grow host integration, modelled as a §2 Realization rather than cemented? The seam
   carries three things, all "the process talks to systemd":
   - `LISTEN_FDS` — inherit the listener instead of binding it (socket activation, slice 4);
   - `sd_notify(READY=1)` — honest F1 readiness (slice 3);
   - **a SIGTERM drain** — stop accepting, finish in-flight, exit within `TimeoutStopSec`
     (slice 4's residual; without it the handback control cannot be met, §3).

   Answering *no* forecloses **all three**, which means slices 3 and 4 both need different
   candidates and the headline outcome has no route I currently see. That is why this is the one
   question genuinely blocking work.
5. ~~**The 208 figure** — where does it come from?~~ **RESOLVED by measurement.** 208 is the real
   resolved-closure count; the tree's *"closure of 91"* is the stale figure (§1). The measurement
   also confirms the deep fix's premise directly: **76% of startup is the load phase**, before
   compilation begins. No decision needed from you — but `expected_startup = 40s` should be
   re-measured on srv1 given ~15% corpus growth since it was set.
6. **srv1 access.** I could not reach srv1 to count restarts from the journal (§1). If you want that
   confirmation rather than the deploy-side count, I need a path in.

---

## Provenance

Grounded against the tree at `0d6ffc4db9` (2026-07-30; first drafted at `a07d1b73f8` and re-checked
after merging main).

**Item receipts** — cited by symbol, since positions rot (5 of this note's 7 original `file:line` receipts had already drifted within a day; see the citation note below). `dag/gunbc/roadmap_component.dag` (`workflow_fetch_request_statements`,
`workflow_observation_poll_interval`, `workflow_poll_single_flight_note`) ·
`dag/gunbc/live_deploy/emit.dag` (`emit_systemd_unit_serve_note`, `emit_systemd_unit_doc`,
`deployment_apply_plan`, `deployment_step_value_eq`, `emit_artifact_upsert`,
`tree_sync_restart_step_with_diagnosis`, `live_deploy_reconcile_binding_note`,
`emit_deploy_member_effect_note`) ·
`dag/gunbc/live_deploy/spec.dag` (`DeploymentArtifactStep`, `deployment_apply_order_note`) ·
`dag/gunbc/live_deploy/service_ready.dag` (`live_deploy_service_ready_poll_bound_reason`,
`live_deploy_roadmap_unit_expected_startup`) · `dag/gunbc/live_deploy/readiness.dag`
(`service_ready_means_serving_this_tree_note`, the F1/F2 split and the 2026-07-24 incident) ·
`src/v1/stage0/src/cli_run.rs` (`handle_serve` — `load_sources_for_entry` → `compile_to_resolved` →
`TcpListener::bind`, in that order) · `dag/gunbc/roadmap_authority.dag` (the
`v1-materialization-kernel` node row).

**Reconcile-precedent receipts** (added for the operator's reconcile-to-intent direction):
`dag/gunbc/membership_reconcile.dag` (`membership_reconcile_authority_note`) ·
`dag/gunbc/host_authorized_keys_reconcile.dag` (identity-vs-value split, staged observation) ·
`dag/gunbc/tool_readiness.dag` (live observation, `observed_pin_projection_note`) ·
`dag/extdeps/realization/emit_on_demand_host.dag` (`observed_tool_identity`, Found/Missing/Duplicate)
· `dag/gunbc/roadmap_belt.dag` (process-as-member; `dispatch_member_value_eq` constant `true`) ·
`dag/gunbc/host_effect_realize.dag` (`srv3_realize_os_install_actuator_toolchain_ensure_body`,
`realize_provision_build_cache_body` — reconciles inside srvN apply).

**Measurements:** deploy-job count — GitHub Actions, `deploy_dashboard_srv1`, 24h window ending
2026-07-30T20:11Z (40 jobs). Serve closure and phase split — the unit's exact ExecStart run on a
spare port: `resolved 208 sources` at t+56s, listening at t+74s (this build box under load, **not**
srv1; see §1's caveat).
