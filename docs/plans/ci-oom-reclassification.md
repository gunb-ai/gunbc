# CI OOM reclassification — the green-on-main LIVE-flip prerequisite (cross-run clustering consumer)

One-line: a lone `exit-137` fails closed to `Structural` (correct in isolation — one `ProcessExit` cannot tell a co-residence/contention OOM from a structural memory-runaway), but the green-on-main freshness mechanism ([ci-merge-freshness.md](ci-merge-freshness.md) §3/§6) cannot consume that verdict to BLOCK merges until a consumer can reclassify a *contention* OOM back to `Infra`. This doc is the model-only design of that consumer — the **hard prerequisite (a)** ci-merge-freshness §6 names for the LIVE flip. The build is deferred (§6).

## 1. The gap — why a lone exit-137 must fail closed, and why that blocks the flip

`classify_floor_exit` (in `gunbc.ci_failure_class`) maps a bare `exit-137` to `FloorFailed { class: Structural }`, and `floor_exit_blocks_merge ⇒ true`. That is the *correct* single-pass verdict: a single exit code carries no evidence to distinguish a structural memory-runaway (a real defect that SHOULD block) from a co-residence OOM (the fleet over-subscribed the host — environmental, NOT the defect green-on-main targets). Failing closed to `Structural` is right for the single-pass classifier (§5 — a fabricated `Infra` would silently merge a real memory defect).

But if green-on-main enforcement consumed that verdict directly, a contention OOM on a re-run would read `Structural ⇒ block ⇒ re-run ⇒ OOM ⇒ …` — the livelock deep-otter-528 warned of. So the flip needs a SECOND-pass consumer that reclassifies a *clustered* OOM back to `Infra` on positive cross-run evidence. That consumer is this design; without it, the verdict is computed but must stay unconsumed (where #5651 leaves it).

## 2. The fact: one OOM-kill, two Realization handlers

"The kernel killed a process for memory" is ONE grounded fact. Per §2/§3 the agnostic *was-OOM-killed* shape stays central; the ways to OBSERVE it are Realization **handlers** that live peripheral (in `extdeps`) with the dispatch — never forked into the interface. There are two handlers, because Linux kills for memory in two places:

| handler | grounded source | what it catches | read seam |
| --- | --- | --- | --- |
| **kernel cgroup `memory.events`** | the `oom_kill` counter line in the job's cgroup-v2 `memory.events` | a **hard-limit** kill — the cgroup hit `memory.max` and the in-kernel OOM-killer fired | reuses the cgroup-read seam already in `ci_floor_measurement.dag` (it reads `memory.peak` off the same cgroup tree; `oom_kill` is one more line in a sibling file) |
| **systemd-oomd journal** | oomd kill records in the systemd journal | a **PSI-pressure pre-emptive** kill — oomd kills *before* the hard cgroup limit on sustained memory pressure | the journal is oomd's only record: a PSI-preemptive kill never increments the cgroup `oom_kill` counter, so the cgroup handler alone would miss it |

Neither is "the" definition — they are two handlers of one fact (§2 horizontal, one concept across breadths). **Dispatch selects** whichever realization the host exposes (cgroup-v2 with vs without oomd active). **Fail-closed:** if NEITHER handler confirms an OOM-kill, the `exit-137` is not a proven OOM and stays `Structural`.

## 3. The discriminator: cross-run clustering (infra-OOM vs structural-runaway)

An OOM-kill confirmed by §2 is still not automatically `Infra` — a structural memory-runaway is *also* an OOM-kill. The discriminator that separates contention from defect is **clustering** across the run/job set:

- **(a) simultaneous multi-job death** — multiple floor shards/jobs die within one tight window. A co-residence OOM kills *neighbors* (it is the host running out, indifferent to which job); a structural runaway kills only its own shard. Many-at-once ⇒ contention.
- **(b) host-uptime / boot-id reset** — the runner host rebooted inside the window (a hard OOM can take the host down, not just the cgroup). A changed **boot-id** between two samples is an unambiguous reboot oracle.

Reclassify `exit-137` → `Infra` ONLY on positive evidence: a §2 handler-confirmed OOM-kill **AND** a clustering signal (a or b). Absent clustering it stays `Structural`. The move is strictly one-directional — the consumer ONLY ever promotes `Structural → Infra` on evidence; it never demotes `Infra → Structural` and never reclassifies by default (§5 fail-closed: the unproven case is the blocking case).

## 4. Grounding the window and the boot-id read

Both discriminator inputs are grounded, not magic numbers (§4 — in a closed system a heuristic threshold is never necessary):

- **Clustering window = a grounded `Duration`**, not a literal. A co-residence OOM's neighbor-kills land within one scheduler/dispatch tick; the window is sourced from the measured floor batch span (`ci_floor_measurement.dag` already carries measured peak/span samples) — the same single-authority measurement the spawn-width derivation reads, not a fresh constant. A hand-picked threshold here would be the §5 "never"-trap (an un-grounded number masquerading as a wall).
- **boot-id = the grounded reboot oracle.** `/proc/sys/kernel/random/boot_id` is a per-boot UUID; a changed `boot_id` across two run samples == the host rebooted == a hard OOM-reset. It is read as one more host-effect (a `filesystem_read` of `/proc/sys/kernel/random/boot_id`) alongside the §2 cgroup read — another Realization read, fail-closed if unreadable (no reboot proven ⇒ no reclassification).

## 5. Carrier extension (downstream of, not a mutation of, classify_floor_exit)

The consumer EXTENDS the `FloorOutcome` carrier (`gunbc.ci_failure_class`, re-landing in #5651). It does **not** touch `classify_floor_exit` — the single-pass classifier stays fail-closed (§5). The reclassifier is a pure SECOND pass — `reclassify(outcome, oom_handler_readings, clustering_evidence) -> FloorOutcome` — that maps a `FloorFailed { class: Structural }` bare/clustered-137 to `FloorFailed { class: Infra { signature } }` **only** on positive evidence, and is the identity on every other outcome. Because it is a total function over the closed evidence, its discriminating witness is mechanical: a clustered-137 + confirmed OOM ⇒ `Infra` (non-blocking); a lone-137 with no clustering ⇒ unchanged `Structural` (blocks). Same epistemic shape as the existing classifier witnesses.

## 6. Status: model-only; build sequence and the OFF path

**This is a §6 scaffold-with-dissolution-trigger at the design stage.** The grounding above is **signed** (both handlers as one fact + clustering discriminator + fail-closed). The BUILD — the pure-`.dag` reclassifier, the two `extdeps` OOM-kill handlers + their dispatch, the boot-id read, and the discriminating witness — lands on a fresh branch **after #5651 merges** (the `FloorOutcome` carrier must be on `main` first), reusing `ci_floor_measurement.dag`'s cgroup-read surface.

This consumer is prerequisite **(a)** of the LIVE flip in ci-merge-freshness §6; **(b)** cap-recovery must also clear. Even with both clear, wiring the verdict into the CI-floor HOST exit semantics stays **OFF** — that is the livelock path (an infra-OOM mis-blocked ⇒ re-run ⇒ OOM ⇒ …). This consumer makes the verdict *trustworthy enough to gate*; it does not itself turn gating on (the operator merge-queue does, when both prerequisites are witnessed).

## Dissolution trigger (DESIGN §6)

Delete this doc when the cross-run clustering reclassifier is built, wired, and its discriminating witness (clustered-137 ⇒ Infra; lone-137 ⇒ Structural) is floor-green by execution — at which point the green-on-main prerequisite (a) of ci-merge-freshness §6 is satisfied as a witnessed property and this design is superseded by the carrier + witness.
