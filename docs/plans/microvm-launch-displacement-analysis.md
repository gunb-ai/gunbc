# Micro-VM launch: what displacing the live actuation would cost

*Subject: price the displacement of the per-attempt Firecracker launch path on Mt. Collins before scheduling an FCI-3 executor. This analysis builds nothing: it censuses the live path, identifies executor responsibilities and assesses an atomic DESIGN §3 cutover.*

*Continues the proposed class `gunbc.recurring_failure_mode.production_actuation_unmodeled_beside_a_modeled_probe`.*

> **Citation standing.** That class and the FCI-2 module below are **PROPOSED, NOT LANDED**, on open, unmerged #10674; neither resolves in this revision. Findings depend instead on probe captures, `fleet-converge.yml` and modules on `main`.

## The headline, before the verdict

**Steps 1–2 are already displaced:** production runs `gunbc run --entry … --function …` with `LocalExec` from `fleet-converge.yml` on the host to converge it and build the guest image. The remaining question is whether a modeled entry can actuate at the *host-boot* point of the attempt lifecycle. The cut is medium and gap-intolerant; the existing converge-time mechanism is proven but cannot reach that point (reading below).

## Why this document and not the executor

`gunbc.fabric.fabric_ci_program` FCI-3 asks *what causes the target process*, and its exit predicate is that a persistent executor **re-reads the committed Grant before starting**, so an absent, corrupt or expired Grant yields **zero** target processes.

Building an executor beside the live hand-authored launcher would leave two substantial homes and production outside the substrate. FCI-3 is worth building when it displaces that actuation; first measure the cut.

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

Steps 1–2 already route through the model; 4, 5 and 8 have unconsumed modeled facts; 6, 7 and 9 lack authority. **Step 9 is FCI-3's subject:** `guest-boot-begin … via=jailer` occurs only in receipts quoting the actuation.

## What a modeled executor would have to own

Not all ten steps. The honest cut is the **per-attempt** sequence, steps 3–9, because 1–2 are converge-time and already modeled, and 10 is observation.

Within that, the executor must own three things it cannot delegate:

1. **The ordering.** Authorization first, actuation last. The FCI-2 half is **proposed on #10674 and not merged** — `gunbc.runner.runner_microvm_execution_authorization`, whose `grant_authorizes_reservation` refuses on the money fence and on Work/Demand/Offer disagreement (`seal_execution_binding` is declared there as a §3c frontier with no consumer yet). Those names do not resolve on `main` today, so they are named as proposed work. What does not exist **anywhere, proposed or landed**, is the thing that **re-reads a committed grant** and refuses to reach step 9 without one.
2. **The credential as a consequence of the grant, not a precondition.** The mint must happen *after* the grant is re-read, so a credential is never in existence for a launch that will not be authorized. The live path mints first and validates in the guest, which is the receipted fail-open.
3. **Durable grant state.** Per ruling, this **extends `gunbc.fabric.fabric_allocation_store_substrate`** — which today models durable presence, ownership, mode and prestate for the allocation store — rather than forking a second store. If a grant fact genuinely cannot inhabit that shape, that is a finding to raise, not a licence to fork.

## What the hand-authored tooling would have to lose

The doctrine's test is not "does Y work" but "does X's authority end in one motion". X here is the boot tooling carried in the **host** image — not the guest image. It emits the `GUNBC-RUNNER-HOST` banner and the `=gunbc-runner-host-boot` marker stream during the *host's* own early boot, and it performs steps 3–9 in order, step 9 being where it starts the guest. Displacement means that tooling loses **step 9 and the steps that feed it**, in the same motion, and is deleted rather than left as a fallback. Y may not resolve through it.

## Is the cutover atomic? The three honest answers

**What makes it plausibly small.** One ordered driver and three missing authorities: credential drive, workspace image and jailer exec. Tap names, addresses, nft chains, attempt paths, unit name, slice properties, sizing and config resolution already exist. Steps 1–2 prove modeled host actuation through `fleet-converge`; step 9 needs a different lifecycle, not proof that modeled actuation is possible.

**What makes it not small.** Three things resist:

- **The boundary is the HOST's boot.** Steps 3–9 run from BMC virtual media on a tmpfs root, around 17s after the host kernel starts (`GUNBC-RUNNER-HOST`). The guest does not exist until step 9. Y must therefore run at host boot, not converge time. An earlier revision incorrectly said “guest image's own early boot,” which would target the wrong image.

- **The credential is one-shot.** A JIT registration is served exactly once. A staged cutover cannot A/B the two paths on the same attempt, and a failed cutover attempt burns the registration. This is a **gap-intolerant boundary** in the doctrine's sense, which argues for the staged carve-out — Y built in shadow, then one transition — rather than plain delete-first.
- **Six admission axes are unchecked today.** The capture reads `jit-admission-unchecked=signature unit-binding boot-binding attempt-binding expiry replay`. A displacement that reproduces the live behaviour reproduces six absent checks; a displacement that adds them is a larger change than a like-for-like cutover. Which of these the minimum Y must carry is a decision, not a detail — §3 requires the minimum Y to preserve **every required refusal**, so any axis judged required today cannot be dropped to make the cut smaller.

**Verdict: medium and gap-intolerant.** An ordered driver over largely existing facts is tractable, but the host-boot lifecycle is unproven and the credential cannot be replayed across the transition. The next measurement is whether an entry can execute where step 9 occurs. Its reading half is answered below; host-image placeability remains unmeasured.

## The reading half of the measurement, answered

**The existing mechanism cannot reach step 9.** Steps 1–2 run `"$ROOT/target/release/gunbc" run … --function …_wet`; `runner_microvm_host_ready` uses `LocalExec` in a workflow on a host-labelled self-hosted runner. It requires a running runner, checkout and built binary.

At step 9 the attempt host has just booted from BMC virtual media (`root-fstype=tmpfs`, `/dev/sr0`; banner ~17 seconds after kernel start). Creating a runner *inside the guest* is the boot's purpose. The converge-time route therefore depends on what step 9 creates and cannot displace it.

The host boot script's `=gunbc-runner-host-boot`, `attempt=` and ordered markers were found only in captures, receipts and BMC artifacts. `gunbc.runner.runner_artifact_lifecycle` requires `DurablyMirrored` for a repository-built host image, but no repository build produces this script. The later recovered source below is frozen evidence.

**Y must be carried in the host image and invoked at boot.** This is a host-image content question. Its cost remains unknown because the build is not visible here.

### The execution half, named for routing

Not chased, since this session has no Mt. Collins access. What would have to be run, and what each would establish:

1. ~~Where is the host boot script authored, and what builds `runner-canary-N`?~~ **Answered by search, not by host access — see the section below. It has no authoring home *as a build input*; the source was later recovered off `/var/tmp` on srv2 and is preserved at `docs/recovered/mtcollins-host-image-source/`, so it is readable but still produces nothing.** The item was wrongly filed here; only 2 and 3 need Mt. Collins.
2. **Can a `gunbc` binary be carried in the host image and executed on tmpfs at boot?** Establishes the modeled executor's placeability directly, and is the minimum viable displacement probe.
3. **What is the credential's admission surface at mint time?** Needed for the axis dispositions below, which cannot be closed by reading this repository.

## Item 1, answered: the boot script is authored in no repository *(as a build input — see the 2026-09-10 recovery note below)*

Search answered this without host access: no repository *build* authors the running image. The original broader claim, “no authoring home anywhere,” was falsified by the 2026-09-10 recovery from srv2. Source is now readable as frozen provenance, still not a build input.

**Re-derivation:** run these subject queries with their positive and reachability controls; the controls distinguish an absent producer from an unsearched repository.

```
# 1. the subject: does the banner or marker stream appear in any producing code, org-wide?
gh api -X GET search/code -f q='"gunbc-runner-host-boot" org:gunb-ai'
gh api -X GET search/code -f q='"guest-boot-begin" org:gunb-ai'

# 2. POSITIVE CONTROL for the instrument — the same query where a hit is known to exist
gh api -X GET search/code -f q='"gunbc-runner-host-boot" repo:gunb-ai/gunbc'

# 3. REACHABILITY CONTROL, per private repo — a universal word, to prove the index sees it at all
gh api -X GET search/code -f q='the repo:gunb-ai/ctrl'
gh api -X GET search/code -f q='the repo:gunb-ai/gunbc-private'

# 4. the fallback for any repo control 3 shows the index cannot reach
gh api "repos/gunb-ai/gunbc-private/git/trees/HEAD?recursive=1" -q '.tree[]|select(.type=="blob")|.path'
```

**Results at authoring:**

- Subject queries found only `gunb-ai/gunbc` captures, receipts and BMC artifacts. Query 2 found the same set, establishing the positive control.
- `gunb-ai/ctrl` was searchable: control 3 returned many hits. `firecracker` found planning `.md` only, `jailer` none, and no path named Mt. Collins or `canary`.
- `gunb-ai/gunbc-private` was **not searchable**: control 3 found zero instances of a universal word. Query 4's direct tree read found strategy documents only. An org-wide sweep without that control would have falsely reported a clean repository.

**What ctrl does carry is a sibling, and its history is the precedent.** `scripts/session-dashboard/host/jit-runner.sh` is the ephemeral-runner wrapper for the **non-micro-VM** hosts (srv1, srv2): it mints a JIT config from the App, runs one job, exits, looped by `systemd Restart=always`. It is not the Mt. Collins boot script and does not start a VM. Its header records why it is in the repository at all:

> *"Load-bearing: this is the canonical, version-controlled copy … It was previously host-only and uncommitted — drift that contributed to the 2026-05-29 GHA stall (the App key rotated, every host's jit-runner.sh 401'd at the JWT exchange, and there was no in-repo source to redeploy from)."*

The sibling already caused an outage from unversioned actuation; committing the script and reconciling its host copy repaired it. The micro-VM image still lacks that producer.

**Store standing:** `gunbc.runner.runner_host_image_store` records `known: true`, `resilient: false` (no replication or backup), the risk of receipts outliving their bytes, and the loss already recorded by `first_reap`. The boot script lives inside those images.

> **2026-09-10 corrections.** The earlier `/srv/bmc` on srv1 location was false: the export was on **srv2**, read-only to the Mt. Collins controller, with seven digest-named ISOs intact. Repeating that literal as fact is why `StoreStanding` no longer carries `location`; location awaits a modeled read of BMC `remote/configurations`.
>
> “No source to rebuild it from” was also false. `/var/tmp/gunbc-hostimage` on srv2 held 1,272 authored lines across seven files, now at `docs/recovered/mtcollins-host-image-source/`; `initramfs/init.recovered` is the `@@BUILD_ID@@` template, not repeated instantiations. Recovery makes X readable and gives the migration a subject to translate. It neither models the behavior nor supplies a repository build.

> **Population correction.** “Every production micro-VM” overstated usage. Completed-run `runner_name` census found only `srv1`/`srv2`/`srv3`/`srv4`, running ctrl's `jit-runner.sh`; Mt. Collins served canary jobs only. Re-derive without filtering hosts, so a Mt. Collins runner would appear:
> ```
> for r in $(gh run list --limit 8 --status completed --json databaseId -q '.[].databaseId'); do
>   gh api repos/gunb-ai/gunbc/actions/runs/$r/jobs --jq '.jobs[]|.runner_name'
> done | grep -v null | sed 's/-[0-9]*-[0-9]*$//' | sort | uniq -c | sort -rn
> ```
> The risk is on the migration path: it must pass through the sole micro-VM launcher, which no repository currently rebuilds.

**Decision owed to the operator:** whether to accept this risk on the January critical path. #10674's two-homes class here means one home modeled and the other built by nothing. Recovery changes readability, not that finding. Displacement is neither a negotiation with ctrl nor a rename of an existing gunbc producer.

**Cut consequence:** recovering X was the prerequisite to pricing Y, and that recovery is done. The recovered directory remains frozen evidence with a dissolution condition; deleting it does not cut over production. The authority to uproot is the tooling *inside the host image*, reachable only through the then-powered-off host. A modeled executor beside it cannot satisfy the condition.

## The six admission axes: dispositions, and what reading them found

`jit-admission-unchecked=signature unit-binding boot-binding attempt-binding expiry replay`.

The capture names six axes but this repository defines none of them. Assigning meanings would fabricate authority; unresolved axes therefore carry definition triggers.

One structural fact constrains several axes at once: `extdeps.github.actions_jit_runner` records that the credential is minted for **one runner identity** and arrives as an **opaque blob**. A host therefore *cannot* check binding axes by parsing the credential; it can only check them against what the **minter** recorded. So unit/boot/attempt binding are mint-side obligations that a host-side check cannot discharge, whatever they turn out to mean.

| axis | disposition | reason / trigger |
|---|---|---|
| signature | **required for cutover** | The receipted failure is on this axis' side of the line: `jitconfig-bytes=0`, **no `jit-admission` verdict recorded at all**, and the host proceeded to create the drive and launch. Whatever "signature" means precisely, a credential that is not established as authentic-and-present must not reach step 9 — that is FCI-3's predicate, not a preference. |
| expiry | **required for cutover** | An expired credential yields a guest that cannot register, i.e. a target process for work that cannot run. FCI-3 names expired grants explicitly alongside absent and corrupt ones, so admitting one is admitting the state the predicate forbids. |
| replay | **deferred** | Trigger: **an authority stating whether one-shot delivery is enforced upstream or must be enforced host-side.** A JIT registration is served once, so replay may already be structurally impossible at GitHub's end — in which case a host-side check is a second authority for an upstream fact. Cannot be settled from here. |
| unit-binding | **deferred** | Trigger: **an authority defining the axis and naming which side can check it.** Not host-checkable against an opaque blob; a mint-side obligation if it is one at all. |
| boot-binding | **deferred — and settle the §3 question first** | Same trigger, plus one that must be answered *before either axis is priced*: **this may be a second name for `attempt-binding`.** A meaning fork inside a six-item roster makes the roster itself the defect, so the count of axes is not trustworthy until this is resolved. The suspicion is concrete: the attempt id already changes every boot, so this may be a second name for attempt-binding — a §3 question to resolve before either is priced. |
| attempt-binding | **deferred** | Same trigger. The join that made the receipted mis-identification possible was a reused **build** label rather than an attempt id, so whatever discharges this axis is load-bearing for evidence identity as well as admission. |

**Sizing:** authenticity/presence and expiry require two whole-credential checks. Four axes await definitions, and boot-binding may dissolve into attempt-binding. This is merry-bear-25's “small” branch only if the definition authority confirms those four are deferrable; this document cannot decide that.

**Like-for-like remains the one answer that is definitely wrong.** A cutover reproducing all six absent checks reproduces the receipted fail-open, and §3 forbids dropping a required refusal to shrink the minimum Y.

## What this document deliberately does not do

No schedule, owner or executor design is chosen. The six axes retain their individual dispositions and triggers. Measurements remain tied to probe captures and controlled search queries. An earlier claim that this document transcribed no measurements was false: bare hit counts without a named producer were replaced by the queries and controls above.
