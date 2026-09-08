# Micro-VM launch: what displacing the live actuation would cost

*Subject: the per-attempt path that starts a real GitHub Actions job in a Firecracker guest on Mt. Collins. This document exists to convert "should we build the FCI-3 executor" from a capacity question into a scoped one. It builds nothing and proposes no schedule; it censuses the live path, names which steps a modeled executor would have to own, and asks whether the cutover can be atomic in the §3 sense.*

*Filed against the class `gunbc.recurring_failure_mode.production_actuation_unmodeled_beside_a_modeled_probe`, which this analysis is the natural continuation of.*

> **Citation standing.** That class and the FCI-2 module cited below are **PROPOSED, NOT LANDED** — they are authored on #10674, which is open and unmerged, so neither name resolves in this revision of the tree. They are cited as proposed work rather than as corpus facts, and every such name is marked at its use site. Nothing in this document's own findings depends on either landing: the census, the reading half and the axis dispositions are all derived from probe captures, `fleet-converge.yml`, and modules that do resolve on `main`.

## The headline, before the verdict

**Half the census is already done, and the displacement mechanism is proven in production.** Steps 1–2 of the live path — host convergence and guest image build — are already displaced: production actuates them by running `gunbc run --entry … --function …` with `LocalExec`, out of `fleet-converge.yml`, on the host itself. So the open question is **not** "can a modeled entry point actuate a host" — that is answered, wet, today. It is "can one actuate at a *different point in the attempt lifecycle*", which is a much smaller unknown.

The verdict below is *medium and gap-intolerant*, and a reader who stops there will take that as the headline and miss this. Both are true at once.

## Why this document and not the executor

`gunbc.fabric_ci_program` FCI-3 asks *what causes the target process*, and its exit predicate is that a persistent executor **re-reads the committed Grant before starting**, so an absent, corrupt or expired Grant yields **zero** target processes.

The tempting next move is to build that executor. It is the wrong move while the live hand-authored path keeps starting real jobs, because a modeled executor that cannot yet take over is X and Y coexisting — the state §3's replacement-migration doctrine forbids, and precisely the class already filed. It would give the modeled home a consumer (itself) and still leave the live path unmodeled: two homes, both now substantial, and the one that runs still outside the substrate.

So FCI-3 is worth building exactly when it **displaces** the live actuation rather than sitting beside it. That is a question about the size of the cut, and nobody has measured it.

## The motive is receipted, not speculative

The live path has a **measured fail-open on exactly FCI-3's predicate**. In `docs/probes/mtcollins_jit_absent_launch_defect_2026-09-07.txt`, host attempt `runner-canary-6-83b96b35` booted with a **zero-byte** credential and the host went on to create the JIT drive, create the workspace, and **launch Firecracker anyway**. The refusal happened *inside the guest*, after a VM had already been started.

Two consequences worth stating separately:

- An absent credential produced **one** target process where the predicate requires **zero**. The wall is in the wrong place — inside the guest, after actuation, rather than before it.
- Firecracker then **exited with status 0** after that guest panicked, so a clean VMM exit is not evidence that any work ran. Any displacement design that reads success from the VMM's exit status inherits this.

The same capture records a second-order harm: because a console drop was scored as a failed boot, the harness **power-cycled a machine running a live CI job**. That is not the launch path's defect, but it is what an unmodeled path's observability costs.

## The live path, step by step

Read from the host serial capture `docs/probes/mtcollins_bound_attempt_host_capture_2026-09-07.txt` (attempt `runner-canary-8-1b3b5aea`, joined to Actions run 34108061111 by a **one-shot** attempt label, not by a reused build label). Each step is classified by whether a `.dag` authority already owns the fact, and separately by whether production **routes through** it — the distinction the filed class is about.

| # | Live step (capture marker) | Fact modeled in `.dag`? | Production routes through it? |
|---|---|---|---|
| 1 | host readiness: `kvm=present`, firecracker binary | `runner_microvm_host_ready` | **yes** — installed via `fleet-converge` `microvm_host_converge` |
| 2 | guest image build and digests | `runner_guest_image` | **yes** — `guest_image_converge` |
| 3 | media verify, jail chroot, `kvm-owner` | partly (`runner_host_grants` privileged ops) | no |
| 4 | host net: forwarding, nft ruleset, SNAT | `runner_microvm_network` (chains, supernet) | no |
| 5 | tap device, host/guest addresses, MAC | `runner_microvm_network` (names, addresses) | no |
| 6 | JIT mint → credential drive, ownership | transport type only (`JitCredentialTransport`) | no |
| 7 | workspace image mkfs, size, ownership | no authority | no |
| 8 | cgroup leaf, memory/cpu/pids limits | `runner_microvm_attempt` slice properties | no |
| 9 | **the jailer exec that starts the VMM** | **no authority** | **no** |
| 10 | live census, teardown, receipts | receipt modules (prose rows) | after the fact |

The shape this shows is narrower than "the live path is unmodeled". **Steps 1 and 2 are already displaced** — production actuates them by running `gunbc run --entry … --function …` out of `fleet-converge.yml`, so the modeled/live fork does not exist there. Steps 4, 5 and 8 have their *facts* modeled with no production consumer: those are the unconsumed home the class names. Steps 6, 7 and 9 have no authority at all.

**Step 9 is the whole subject.** Everything else is either already displaced or a fact waiting for a consumer; step 9 is the causation FCI-3 owns, and the string `guest-boot-begin … via=jailer` appears nowhere in this repository except inside receipts *quoting* it.

## What a modeled executor would have to own

Not all ten steps. The honest cut is the **per-attempt** sequence, steps 3–9, because 1–2 are converge-time and already modeled, and 10 is observation.

Within that, the executor must own three things it cannot delegate:

1. **The ordering.** Authorization first, actuation last. The FCI-2 half is **proposed on #10674 and not merged** — `gunbc.runner.runner_microvm_execution_authorization`, whose `grant_authorizes_reservation` refuses on the money fence and on Work/Demand/Offer disagreement (`seal_execution_binding` is declared there as a §3c frontier with no consumer yet). Those names do not resolve on `main` today, so they are named as proposed work. What does not exist **anywhere, proposed or landed**, is the thing that **re-reads a committed grant** and refuses to reach step 9 without one.
2. **The credential as a consequence of the grant, not a precondition.** The mint must happen *after* the grant is re-read, so a credential is never in existence for a launch that will not be authorized. The live path mints first and validates in the guest, which is the receipted fail-open.
3. **Durable grant state.** Per ruling, this **extends `gunbc.fabric_allocation_store_substrate`** — which today models durable presence, ownership, mode and prestate for the allocation store — rather than forking a second store. If a grant fact genuinely cannot inhabit that shape, that is a finding to raise, not a licence to fork.

## What the hand-authored tooling would have to lose

The doctrine's test is not "does Y work" but "does X's authority end in one motion". X here is the boot tooling carried in the **host** image — not the guest image. It emits the `GUNBC-RUNNER-HOST` banner and the `=gunbc-runner-host-boot` marker stream during the *host's* own early boot, and it performs steps 3–9 in order, step 9 being where it starts the guest. Displacement means that tooling loses **step 9 and the steps that feed it**, in the same motion, and is deleted rather than left as a fallback. Y may not resolve through it.

## Is the cutover atomic? The three honest answers

**What makes it plausibly small.** The unmodeled surface is one ordered driver plus three missing authorities (credential drive, workspace image, jailer exec). Most step-level facts — tap name, addresses, nft chains, attempt paths, unit name, slice properties, sizing, the config resolution — are already authored and merely unconsumed. And the displacement mechanism already exists and is proven in production for steps 1–2: `fleet-converge` invoking `gunbc run --entry … --function …` on the host. Displacing step 9 does not require inventing a new actuation channel.

**What makes it not small.** Three things resist:

- **The boundary is a boot, not a call — and it is the HOST's boot.** Steps 3–9 run inside the **host image's** own early boot (the banner is `GUNBC-RUNNER-HOST`, ~17s after the *host* kernel starts, on a tmpfs root from BMC virtual media), before anything this repository controls is running. The guest does not exist yet; step 9 is what creates it. So displacing these steps means the modeled executor must run **on the host, at host-boot time**, which is a different lifecycle from the converge-time `gunbc run` that displaced steps 1–2. That is the load-bearing unknown, and it is not settled by this analysis.

  *Stated explicitly because an earlier revision of this document said "the guest image's own early boot" here. That was wrong and it matters: it would point a cut at the guest image, when the tooling to be displaced is in the host image.*
- **The credential is one-shot.** A JIT registration is served exactly once. A staged cutover cannot A/B the two paths on the same attempt, and a failed cutover attempt burns the registration. This is a **gap-intolerant boundary** in the doctrine's sense, which argues for the staged carve-out — Y built in shadow, then one transition — rather than plain delete-first.
- **Six admission axes are unchecked today.** The capture reads `jit-admission-unchecked=signature unit-binding boot-binding attempt-binding expiry replay`. A displacement that reproduces the live behaviour reproduces six absent checks; a displacement that adds them is a larger change than a like-for-like cutover. Which of these the minimum Y must carry is a decision, not a detail — §3 requires the minimum Y to preserve **every required refusal**, so any axis judged required today cannot be dropped to make the cut smaller.

**The verdict this analysis supports.** The cut is **medium and gap-intolerant**, not small. It is one ordered driver over largely-existing facts, which is tractable; but its boundary is a host-side pre-boot lifecycle nobody has run a modeled executor in, and its credential cannot be replayed across a staged transition. The next question is therefore not "who builds FCI-3" but a single measurement: **can a modeled entry point run on the host at the point in the attempt lifecycle where step 9 happens?**

That measurement has since been split and its reading half answered — **no**, for a causal reason rather than a contingent one. See *The reading half of the measurement, answered* below; the paragraphs above are left as the reasoning that produced the question.

## The reading half of the measurement, answered

The measurement this document asked for splits: one half is a reading of what exists, needing no host, and one half needs Mt. Collins. The reading half is done, and it settles the branch this document left open.

**What the proven mechanism actually requires.** The steps 1–2 displacement runs `"$ROOT/target/release/gunbc" run … --function …_wet`, and `runner_microvm_host_ready` performs its effects with transport **`LocalExec`** — the workflow job runs *on the target host*, dispatched to a self-hosted runner labelled with that host's name. So the mechanism presupposes three things on the host: **a running self-hosted runner** to receive the job, **a checkout** of this repository, and **a built binary**.

**None of the three exists at step 9's point in the lifecycle, and that is causal rather than incidental.** The attempt host boots from BMC-attached virtual media onto a **tmpfs root** (`root-fstype=tmpfs`, `/dev/sr0` virtual CDROM), and the banner appears ~17 seconds after kernel start. There is no runner yet — **bringing a runner into being is the entire purpose of the boot**, and it will live inside the guest, not on the host. So the mechanism that displaced steps 1–2 **cannot** displace step 9: it presupposes exactly the thing step 9 creates. The dependency is circular, and no amount of care in placing the entry point removes it.

**And the host boot script is not in this repository at all.** The banner `=gunbc-runner-host-boot`, the `attempt=` identity, and the ordered marker stream appear in **captures, receipts and BMC artifacts only** — never in producing code, in any language, anywhere in the tree. `gunbc.runner.runner_artifact_lifecycle` records the standing that a *repository-built host image* requires `DurablyMirrored`, so the image is understood to be ours; the boot script that image runs is not authored here.

**So the answer to "can a modeled entry point run where the jailer exec happens" is no**, and this document's own suspicion is confirmed: **the missing host-side execution lifecycle is the real subject.** That is a more tractable statement than it sounds, because it names a specific shape rather than an open problem — the modeled executor would have to be *carried in the host image and invoked at boot*, taking over the boot script's role, which is a question about host-image **content** rather than about a new execution channel. What this analysis cannot say is whether that is cheap or expensive, because the host image build is not visible from here.

### The execution half, named for routing

Not chased, since this session has no Mt. Collins access. What would have to be run, and what each would establish:

1. ~~Where is the host boot script authored, and what builds `runner-canary-N`?~~ **Answered by search, not by host access — see the section below. It is authored nowhere.** The item was wrongly filed here; only 2 and 3 need Mt. Collins.
2. **Can a `gunbc` binary be carried in the host image and executed on tmpfs at boot?** Establishes the modeled executor's placeability directly, and is the minimum viable displacement probe.
3. **What is the credential's admission surface at mint time?** Needed for the axis dispositions below, which cannot be closed by reading this repository.

## Item 1, answered: the boot script is authored in no repository

This was mis-filed as needing host access. It is a search, and it comes back **outcome three: the host boot script has no authoring home anywhere.**

**What was searched, and the controls.** Org-wide GitHub code search for the banner and the marker stream returns **11 hits, all in `gunb-ai/gunbc`, and every one a capture, receipt or BMC artifact** — never producing code. The instrument was positive-controlled first (the same query against a string known to exist returns those 11), and each private repository was controlled separately rather than trusted:

- **`gunb-ai/ctrl` — searchable and clean.** A universal-word control returns 914 hits, so the index reaches it. `firecracker` returns 2 hits, both planning `.md`; `jailer` returns 0; no path names Mt. Collins or `canary`.
- **`gunb-ai/gunbc-private` — not searchable, so not trusted to search.** A universal-word control returns **0**, meaning the index does not reach it. Cleared instead by reading its tree directly: 178 blobs, strategy documents only.

**What ctrl does carry is a sibling, and its history is the precedent.** `scripts/session-dashboard/host/jit-runner.sh` is the ephemeral-runner wrapper for the **non-micro-VM** hosts (srv1, srv2): it mints a JIT config from the App, runs one job, exits, looped by `systemd Restart=always`. It is not the Mt. Collins boot script and does not start a VM. Its header records why it is in the repository at all:

> *"Load-bearing: this is the canonical, version-controlled copy … It was previously host-only and uncommitted — drift that contributed to the 2026-05-29 GHA stall (the App key rotated, every host's jit-runner.sh 401'd at the JWT exchange, and there was no in-repo source to redeploy from)."*

So **this organisation has already taken an outage from exactly this class**, on the sibling of the very path under discussion, and the remedy applied then was to commit the script and reconcile the host copy from it. The micro-VM path is in the pre-remedy state today.

**And the carrier is a single unreplicated artifact.** `gunbc.runner.runner_host_image_store` records the host images at `/srv/bmc` on srv1, NFS-exported, `known: true`, **`resilient: false`** — no replication, no backup — with the loss consequence stated as receipts outliving the bytes they cite. The boot script lives *inside* those images. So the thing that starts every production micro-VM on Mt. Collins exists as **one copy on one disk, with no source to rebuild it from**, and the same store has already had a reap delete 17 of 20 images.

**Why this belongs to the operator rather than a lane.** Unauthored production actuation on the January critical path is a decision about accepted risk, not a modelling preference — and this is the strongest form of the two-homes class filed on #10674: not *one home modeled and one hand-authored*, but **one home modeled and the other outside the corpus entirely.** That single sentence is the fact a reader should leave this document with.

**What it means for the cut.** Displacement is not a cross-repo negotiation (ctrl does not own this) and not a cheap rename (gunbc does not own it under another name). There is no X to delete in any repository — X is bytes inside an ISO. So the replacement migration has no root to uproot in source control, and the first move is not building Y but **recovering or re-authoring X's content so that what production runs is knowable at all**. That is a different and smaller first step than the FCI-3 executor, and it is a precondition for pricing the executor rather than a part of it.

## The six admission axes: dispositions, and what reading them found

`jit-admission-unchecked=signature unit-binding boot-binding attempt-binding expiry replay`.

**The first finding is about the roster itself: none of these six terms is defined anywhere in this repository.** The marker string is their only occurrence. They are named by host tooling that is not authored here, so this repository has no authority saying what any of them checks. Assigning "required" to a term whose meaning I would have to invent is exactly the fabrication that would make this table worse than absent, so the dominant disposition is *deferred with a named trigger*, and the trigger is the same for most rows.

One structural fact constrains several axes at once: `extdeps.github.actions_jit_runner` records that the credential is minted for **one runner identity** and arrives as an **opaque blob**. A host therefore *cannot* check binding axes by parsing the credential; it can only check them against what the **minter** recorded. So unit/boot/attempt binding are mint-side obligations that a host-side check cannot discharge, whatever they turn out to mean.

| axis | disposition | reason / trigger |
|---|---|---|
| signature | **required for cutover** | The receipted failure is on this axis' side of the line: `jitconfig-bytes=0`, **no `jit-admission` verdict recorded at all**, and the host proceeded to create the drive and launch. Whatever "signature" means precisely, a credential that is not established as authentic-and-present must not reach step 9 — that is FCI-3's predicate, not a preference. |
| expiry | **required for cutover** | An expired credential yields a guest that cannot register, i.e. a target process for work that cannot run. FCI-3 names expired grants explicitly alongside absent and corrupt ones, so admitting one is admitting the state the predicate forbids. |
| replay | **deferred** | Trigger: **an authority stating whether one-shot delivery is enforced upstream or must be enforced host-side.** A JIT registration is served once, so replay may already be structurally impossible at GitHub's end — in which case a host-side check is a second authority for an upstream fact. Cannot be settled from here. |
| unit-binding | **deferred** | Trigger: **an authority defining the axis and naming which side can check it.** Not host-checkable against an opaque blob; a mint-side obligation if it is one at all. |
| boot-binding | **deferred — and settle the §3 question first** | Same trigger, plus one that must be answered *before either axis is priced*: **this may be a second name for `attempt-binding`.** A meaning fork inside a six-item roster makes the roster itself the defect, so the count of axes is not trustworthy until this is resolved. The suspicion is concrete: the attempt id already changes every boot, so this may be a second name for attempt-binding — a §3 question to resolve before either is priced. |
| attempt-binding | **deferred** | Same trigger. The join that made the receipted mis-identification possible was a reused **build** label rather than an attempt id, so whatever discharges this axis is load-bearing for evidence identity as well as admission. |

**What this means for sizing the cut.** Two axes are required and both are properties of the credential *as a whole* — is it authentic and present, is it live — rather than bindings needing a parse. Four are deferred behind one trigger, and at least one may dissolve into another. So on today's reading the cut is **not** a six-way credential-admission programme; it is two whole-credential checks plus a definition question. That is the "small" branch merry-bear-25 named — but it rests on the deferred four actually being deferrable, which the axis-definition authority decides, not this document.

**Like-for-like remains the one answer that is definitely wrong.** A cutover reproducing all six absent checks reproduces the receipted fail-open, and §3 forbids dropping a required refusal to shrink the minimum Y.

## What this document deliberately does not do

It does not schedule the work, choose an owner, or propose an executor design. It gives each of the six admission axes a stated disposition rather than a verdict on all six, and where the honest disposition is *deferred* it names the trigger instead of guessing the axis' meaning — four of the six are deferred precisely because this repository carries no definition of them. And it transcribes no measurement: every number and marker here is cited to the probe capture that produced it, which remains the authority.
