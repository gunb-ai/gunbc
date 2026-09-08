# Micro-VM launch: what displacing the live actuation would cost

*Subject: the per-attempt path that starts a real GitHub Actions job in a Firecracker guest on Mt. Collins. This document exists to convert "should we build the FCI-3 executor" from a capacity question into a scoped one. It builds nothing and proposes no schedule; it censuses the live path, names which steps a modeled executor would have to own, and asks whether the cutover can be atomic in the §3 sense.*

*Filed against the class `gunbc.recurring_failure_mode.production_actuation_unmodeled_beside_a_modeled_probe`, which this analysis is the natural continuation of.*

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

1. **The ordering.** Authorization first, actuation last. `gunbc.runner.runner_microvm_execution_authorization` (#10674) is the FCI-2 half: `grant_authorizes_reservation` refuses on the money fence and on Work/Demand/Offer disagreement, and `seal_execution_binding` makes an unauthorized binding unconstructible outside the module. What does not exist is the thing that **re-reads a committed grant** and refuses to reach step 9 without one.
2. **The credential as a consequence of the grant, not a precondition.** The mint must happen *after* the grant is re-read, so a credential is never in existence for a launch that will not be authorized. The live path mints first and validates in the guest, which is the receipted fail-open.
3. **Durable grant state.** Per ruling, this **extends `gunbc.fabric_allocation_store_substrate`** — which today models durable presence, ownership, mode and prestate for the allocation store — rather than forking a second store. If a grant fact genuinely cannot inhabit that shape, that is a finding to raise, not a licence to fork.

## What the hand-authored tooling would have to lose

The doctrine's test is not "does Y work" but "does X's authority end in one motion". X here is the host tooling baked into the runner guest image build: it emits the `GUNBC-RUNNER-HOST` banner and the marker stream, and it performs steps 3–9 in order. Displacement means that tooling loses **step 9 and the steps that feed it**, in the same motion, and is deleted rather than left as a fallback. Y may not resolve through it.

## Is the cutover atomic? The three honest answers

**What makes it plausibly small.** The unmodeled surface is one ordered driver plus three missing authorities (credential drive, workspace image, jailer exec). Most step-level facts — tap name, addresses, nft chains, attempt paths, unit name, slice properties, sizing, the config resolution — are already authored and merely unconsumed. And the displacement mechanism already exists and is proven in production for steps 1–2: `fleet-converge` invoking `gunbc run --entry … --function …` on the host. Displacing step 9 does not require inventing a new actuation channel.

**What makes it not small.** Three things resist:

- **The boundary is a boot, not a call.** Steps 3–9 run *inside the guest image's own early boot*, before anything this repository controls is running. Displacing them means the modeled executor must run **on the host, before the guest**, which is a different lifecycle from the converge-time `gunbc run` that displaced steps 1–2. That is the load-bearing unknown, and it is not settled by this analysis.
- **The credential is one-shot.** A JIT registration is served exactly once. A staged cutover cannot A/B the two paths on the same attempt, and a failed cutover attempt burns the registration. This is a **gap-intolerant boundary** in the doctrine's sense, which argues for the staged carve-out — Y built in shadow, then one transition — rather than plain delete-first.
- **Six admission axes are unchecked today.** The capture reads `jit-admission-unchecked=signature unit-binding boot-binding attempt-binding expiry replay`. A displacement that reproduces the live behaviour reproduces six absent checks; a displacement that adds them is a larger change than a like-for-like cutover. Which of these the minimum Y must carry is a decision, not a detail — §3 requires the minimum Y to preserve **every required refusal**, so any axis judged required today cannot be dropped to make the cut smaller.

**The verdict this analysis supports.** The cut is **medium and gap-intolerant**, not small. It is one ordered driver over largely-existing facts, which is tractable; but its boundary is a host-side pre-boot lifecycle nobody has run a modeled executor in, and its credential cannot be replayed across a staged transition. The next question is therefore not "who builds FCI-3" but a single measurement: **can a modeled entry point run on the host at the point in the attempt lifecycle where step 9 happens?** If yes, the cut is a dispatch. If no, the missing host-side execution lifecycle is the real subject and FCI-3 waits on it.

## What this document deliberately does not do

It does not schedule the work, choose an owner, or propose an executor design. It does not claim the six unchecked admission axes are each required — it names them as a decision. And it transcribes no measurement: every number and marker here is cited to the probe capture that produced it, which remains the authority.
