# microVM and floor: wind-down state, 2026-09-21

Written on operator instruction to wind down and record remaining items, so that v1
performance and v2 migration can be prioritised. **This is a state record, not a plan**: it
says what is true on `origin/main`, what is owed, and what is blocked, so that resuming any
lane does not begin by re-deriving it.

Every claim below was verified against `origin/main` at `7a145ef9ca` or read from the lane
that produced it. Where a claim is **contested**, it is marked, and neither version is
asserted — DESIGN §4d: a bet is typed as a bet, and the reader who consumes it as a fact is
the defect.

**Consumer, and a declared frontier (DESIGN §3c).** This page is cited by path from the two
re-entry briefs that govern the parked programs, which is where a resuming session reads it.
It is **not** linked from `DESIGN.md`, and it deliberately does not add a
`gunbc.design_document` row to become so: it is a dated state record rather than a doctrine
a DESIGN section governs, and editing that authority is not a wind-down act. It therefore
joins the orphan population already rostered as `gunbc.guarantee_stall`
`doc_graph_orphan_population_stall` (`current: OutsideTheLadder`), and this paragraph is the
admission rather than a silent addition. **Trigger for the frontier:** whichever funded lane
resumes a program named here either links this page from the section that governs its work,
or supersedes it and deletes it. A wind-down record that outlives the wind-down is debt.

**On the numbers below.** Where a figure is load-bearing it is stated with the producer that
re-derives it (DESIGN §6: name the instrument, never transcribe its output). Where a figure
is a lane's reported reading rather than a re-derivable instrument, it is attributed to that
lane and marked as such — those are receipts of a past run, not facts a reader may refresh,
and they are written so that staleness is visible rather than plausible.

---

## 1. The floor reports SUCCESS over its own refusal — the highest-value open item

`.github/workflows/witnesses.yml:67`:

```
if [ "$GUNBC_FLOOR_CLASS" = structural ] && [ "$GUNBC_FLOOR_EXIT" -eq 0 ]; then GUNBC_FLOOR_CLASS='none'
```

`claim_executor` **exits 0 over its own typed refusal**, so the class is downgraded to `none`
and the job concludes success. A job can therefore report SUCCESS over a run that executed
**zero witnesses**, with its own log saying `phases_failed=1`.

**This is not merely a hidden defect — it admitted a new one.** #11731 landed a file that
could not parse *because* the floor was refusing and reporting success. The hollow gate is
upstream of the breakage, not downstream of it (§5: a failure arm must refuse, never widen;
the downgrade is an absorbing fallback executed in YAML).

**Current standing.** The parse class is **closed**: the indented-annotation census is 0
corpus-wide and the floor selects and executes its full planned set, with `not_attempted=0`
and `claims_failed=0`, where previously **zero** witnesses were selected. Closed by #11896,
#11897, #11880, #11920, #11842. **Re-derive rather than trust this sentence:** the producer
is the required floor itself — `claim_executor --required-ci --source-root dag
--source-root src/v2 --required-lane witnesses` — whose run prints its own
`planned/executed/not_attempted/terminal/passed` summary line. The counts move with every
landed witness, so a number copied here would rot within a day.

**CONTESTED — do not propagate either reading.** Whether the downgrade swallows failures
*per phase* is disputed by two lanes:

| reading | source | consequence if true |
|---|---|---|
| per-phase: parse reds the job honestly, declarations passes | `nimble-swift-273` | once parse cleared, a **declarations defect is invisible** |
| no such ordering | four run observations | the discriminator is unknown |

The four observations: main `0de0b6137c7` 13 parse FAILs → **SUCCESS**; #11803 `70faed05501`
340 → **SUCCESS**; #11842 343 → **FAILURE**; #11884 317 → **FAILURE**. No ordering by count,
and parse failures did **not** always red the job. `eager-swift-412` independently reports
the discriminator as **cannot-tell from the logs**.

**The discriminating test nobody has run**, and it is cheap: read `$GUNBC_FLOOR_LOG` — a
**file** the floor writes, *not* the job log, where every `class=` line is echoed script — or
the classifier itself, on **one PASS and one FAIL of the same phase shape**. Counting
signature strings in the GHA job log is **not** discriminating: a fired signature sets an env
var rather than printing, so identical counts appear whether the hypothesis holds or not.

**#11836 and #11829 are competing repairs. Only one should land.** Neither is owned.

## 2. microVM: the part that decides is built; the part that acts is not

`gunbc.runner_microvm_lifecycle_realize` **landed** (#11803) and has executed against real
processes on srv1: the lane reported every claim group green at its landing head, including
the wet receipt under `--wet` and the argv-binding wall. That is a **lane receipt of a past
run**, not a standing fact — re-derive it with `claim_batch` over the module's witness
modules at the current head before relying on it. #11677 closed both launch frontiers, so
the JIT credential mint and jail staging are realized.

**Five declared frontiers, all live in that one landed module.** Read them from the file,
not from here.

| # | frontier | status | what is missing |
|---|---|---|---|
| 1 | `controller_main_pid_consumer_frontier` | **dispatchable, start here** | **nothing calls `run_controller`.** Until a slot unit execs it as MainPID the module is a library, not a controller. Every other frontier is worth less until this clears. |
| 2 | `attempt_cleanup_realization_frontier` | dispatchable | no unmount, no delete of credential device / workspace / attempt root, no flush. The **only** host mutation in teardown is the `cgroup.kill` write. |
| 3 | `guest_bring_up_channel_frontier` | dispatchable, spans host+guest | no guest→host readiness channel above `VmmStarted`. Decides only whether a slot is SERVING, never whether a cell is CLEAN. |
| 4 | `slot_network_readings_producer_frontier` | **BLOCKED — security** | no converged slot network on any host; every reading is `Unreadable`, which quarantines. |
| 5 | `workflow_effect_sequencing_frontier` | **not a lane — a design question** | `std` carries **no** effect-sequencing authority. Bigger than microVM: every effect sequence in the corpus rests on the answer. |

**The consequence that governs planning:** the module states it itself — *"no cell settles
CellReady through this module today."* **A first full run quarantines by construction, and
that is not a bug in the readbacks.** A warm pool is impossible until (4) clears.

> Do not let anyone "fix" (4) by omitting `SlotNetworkQuiescent` from the required facts.
> That is the empty-observation narrow `product.fabric.sanitation` exists to refuse.

### Why (4) is blocked, stated so the reason survives

**The converge principal IS the job principal.** Granting it install+restart over files it
also *stages* is granting root: `install <grantee-writable-source> /etc/systemd/system/…`
followed by `systemctl restart` is root-equivalence, however narrowly spelled. So
`microvm_network_granted_operations` deliberately carries **readbacks only**.

To unblock, an administrator principal must (1) own the staged source so the job user cannot
write what root will install and execute; (2) install the nft loader unit and ruleset and
`daemon-reload` + restart it; (3) set `ip_forward` and create the bridge; (4) create and
destroy per-attempt taps — and be **unimpersonable by the job user**. That last is the whole
point, and is why the frontier trigger names a *capability* rather than an artifact (§4b(3)).

#11751 (`microvm_network_observe`) landed. It **installs nothing**, reads every fact with its
own UNREAD arm, and mints `ConvergedNetwork` only when every fact was actually read. On every
host today it will **refuse**, because nothing has installed a tap or the table — that is
honest, not a failure.

**It has now been run, and it refused exactly there.** fleet-converge run 35646413870, mode
`microvm_network_observe`, host srv1, 2026-09-21: the whole receipt is
`microvm-network-observe REFUSED: slot srv1-01: tap disable_ipv6 could not be read`. srv1
carries no `gunbc-tap1`. That is the first end-to-end reading of the mode against a real host
and it establishes the mode works and the host is unconverged; it is **not** a
`ConvergedSlotNetwork`, and `converged_slot_network_producer_frontier` therefore stands.

**The administrator principal has landed: `gunbc.runner_microvm_network_apply`.** It is the
apply half of (4), a fleet-converge mode of its own (`microvm_network_apply`,
`FleetSshKeyConsumed`), executing as `gunbc.fleet_bootstrap_principal`
`executor_bootstrap_principal` over the fleet SSH edge — the same edge
`gunbc.runner_host_file_converge` already uses for root-evaluated files. It refuses when that
administrator IS the job user on the host; it refuses unless the dispatch names the revision
the tree is checked out at, because the bytes it installs are rendered from that revision; it
verifies **every ancestor** of the staging root root-only far-side before staging a byte; it
writes each staged file `root:root 0600` through `converge_typed_remote_file` (stage, mode,
atomic rename, far-side content comparison); it verifies the staged files' own ownership;
then it runs the modeled install operations in order, stopping at the first failure, and
reads the ruleset back. The staging root **moved out of the job user's own tree** in the same
change — it was under `gunbc_fleet_converge_state_dir`, which `runner_host_grants` creates
with `owner=job`, which is precisely what the frontier says does not satisfy it. The job-user
grant roster is **unchanged**: the installs run under a different principal, and re-admitting
them to the job user's sudoers because someone else performs them would grant exactly the
pair the frontier refuses.

**What is still open on (4):** the mode has not been dispatched, so nothing has been
installed on any host and no ruleset listing has been read back.
`microvm_network_host_mutation_principal_frontier` is amended rather than deleted, because
its trigger says the mutations *execute*, and a landed route is not an execution.

**And the premise of this whole section is false on srv1 today, which is the finding that
lane produced.** `/etc/sudoers.d/gunbc-deploy` — GENERATED by `gunbc.ci_deploy_sudoers` from a
roster of bare binary paths — grants `ghrunner ALL=(ALL) NOPASSWD:` over `/usr/bin/install`,
`/usr/bin/systemctl`, `/usr/bin/rm`, `/usr/sbin/visudo`, `/usr/sbin/usermod`,
`/usr/sbin/useradd`, `/usr/bin/apt-get` and `/usr/bin/tailscale`, path-unrestricted (verified
live, 2026-09-21). That is the install+systemctl pair the frontier refuses to grant, already
held by the job user, unbounded. The job user is root-equivalent on srv1 **today**, and "CI
jobs cannot become root" was an unrecorded false belief. Rostered as `gunbc.rung_drop`
`the_job_user_holds_a_path_unrestricted_root_grant` and classed as
`gunbc.recurring_failure_mode`
`the_grant_a_frontier_refuses_is_already_held_from_another_authority`. It does not weaken the
frontier — it means satisfying it is **necessary and not sufficient**. **Closing those grants
is a separate, operator-gated item**, and it is what gates untrusted-PR qualification of the
microVM host: legacy jobs on the host consume them today, so each needs a consumer census
before it can be removed, and no lane may narrow them silently.

### The floor's own job does not fit a cell

`runner_microvm_floor_fit_stall` remains rostered: the floor's measured held-set peak exceeds
`gunbc.runner_slot_allocation` `gunbc_runner_slot_memory_max_bytes` less the realization
reserve. Both sides are derivable rather than transcribed — the cap is that declaration, and
the demand is produced by `gunbc.floor_memory_demand`, whose qualification against readable
limits is what the stall cites. The stall row itself carries the figures it was written
against; read them there, where they sit beside the reading that produced them.

**Grounding it is an operator decision with a fleet cost**, not an engineering task: raise
per-slot `MemoryMax` to ~28 GiB, which **lowers each host's memory-admitted width**, or
reduce floor demand. The stall explicitly rules out deleting the allowance, reserve or
receipt, and rules out using a fixture cell.

### The milestone that would settle the cutover question

**One cell settling `CellReady` once, on one host, after one real job.** Nothing before that
is evidence. If that cannot be reached, the remaining work is unbounded.

## 3. Dynamic per-VM memory — two compiling, unreconciled implementations

Design ruled and approved by the side chat; **build parked**. Six items; item 1 is built
**twice, independently, blind to each other**.

**Neither branch is uncompiled.** The uncompiled attempts are the two *closed* PRs, #11849
and #11850.

| branch | sha | evidence | carries uniquely |
|---|---|---|---|
| `session/bright-ram-63` | `125659f70df` | lane-reported: all claims green under `claim_batch`, plus five mutation controls, tree restored green after each | the **executed** authority-token wall |
| `session/royal-moth-544` | `80ec5065822` | lane-reported: all claims green under `claim_batch`, two mutation controls executed | `std.measure round_up_to_grain` (resolves the unowned rounder fork), ceiling-as-parameter, permit-in-provenance |

The same three file paths exist on both with different contents, so **merging them is a
textual conflict on every file, not a union.** Pick one base and port the other's evidence.

**Both PRs (#11883, #11885) were found NON-DRAFT on 2026-09-21, although both lanes believed
they had left them draft** — so a parked program was consuming CI and reviewer attention on
every push. Both were converted back to draft at wind-down. Keep them draft while parked:
ready buys coverage that was never approved. *The lesson is the gap itself* — "I left it
draft" is a belief about a past action, and the dashboard opens PRs on a lane's behalf, so
it must be re-read rather than remembered.

**The defect, stated correctly** (it is a bypass, not an absence). All three verified against
`origin/main`:

- `CellReserved` holds exactly four fields — offer, slot_key, generation, account. **No
  requirement, no grant, no envelope.**
- `admit_executor_capacity` takes a whole `ExecutionRequirements` and reads exactly **one**
  field, `requirements.capacity_class`. **Cell admission does not consider memory.**
- `unmet_memory_axis` returns `[]` when `needed.envelope.memory` is `Absent`, so **unstated
  memory is admitted** by the supply screen rather than refused.

So memory *is* screened before reservation and the runner *does* read the requirement; what
is missing is that memory is never converted into a **grant**, never **atomically reserved**
against live host use, and never **carried on `CellReservation`**.

`MemoryGrant` and `HostMemoryLedger` return **zero matches anywhere on main**, so items 3-6
have no landed symbols to integrate against. The coordination hazard for item 4 (#11625,
static-vs-dynamic `MemoryHigh`/`MemorySwapMax`) is still **open**, but its head has moved to
`8e75da2aa48` — re-read it before item 4 rather than trusting any recorded sha.

**Topology precondition, a gate rather than an item:** exact per-Work sizing is valid only
where Work is bound **before** the VM boots. A generic prebooted GitHub runner does not know
its eventual job. Do not let a guest discover its Work after boot and call that dynamic
allocation.

**Amendment worth not re-litigating:** a per-host **CAS-linearized** ledger is required.
Per-cell records cannot conserve a host-wide sum — A and B each read 60 of 100 free, each
reserve 30, both succeed, 120 committed. One CAS commits cell occupancy and memory claim
together.

## 4. Owed evidence

**#11845's discriminating RED was never executed.** The repair is real and approved, and it
lands on a green that is hollow for the two reasons in §1. Branch
`verify/11845-red-mutation` (`2499194f4b2`) is pushed and is exactly one hunk off the merge
head: the `test -e` defect put back inside the new classifier. **Both of its blockers have
since cleared**, so the pair is now runnable — expect green at the merge head, red at
`2499194f4b2` on `an_unread_path_reaches_unobservable_and_never_absent`. **If the mutation
passes, the control does not discriminate and the repair needs a better one.** Delete that
branch afterwards; `verify/11845-red-control` is plain main and can go now.

**Two unowned repairs**, both one-liners, both real:

- `v2.lens.reference_derived_residency_reading` has zero occurrences of
  `construction_justification`, while its siblings import it from `v2.lens.common`.
- `dag/test/claim/machine_intake/jade_first_contact_model_witness_test.dag:3` imports `Nat`
  from `std.types`; `Nat` is declared in `dag/std/nat.dag`.

**`required_gate_prefixes` has no `dag/test/claim/runner/` row**, so that file is never
resolved by CI unless a diff touches it — a symbol deletion can strand it silently, as
#11762 did. Likewise no row matches `test.claim.fabric.`, so the dynamic-memory witnesses are
discovered and declined.

## 5. The P0 that opened this session is contained but not closed

srv2 still carries `ExitType=cgroup` in
`/etc/systemd/system/actions-runner@.service.d/70-fleet-teardown.conf` — the configuration
that left PID 56295 orphaned holding 16 GB for 2.7 days. srv1 and srv3 are converged to
`ExitType=main`.

**Residual risk, precisely.** srv2-01..05 are **masked**, so the P0 cannot recur on them
today. But the drop-in lives in the **template** directory and the template is *not* masked,
so any **new** instance (`actions-runner@srv2-06`) inherits `ExitType=cgroup`. The remedy is
to run the modelled fleet converge on srv2; it has simply never run there.

srv4 is off the reach chain (container → srv1 → srv2-lan → srv3) and `gunbc.fleet_reach`
carries no row for it.

## 6. Method findings worth keeping

- **A suite of controls that invoke a pure decision with supplied values is blind to
  everything after the decision** — the cleanup leg, the receipt the withheld arm writes —
  and stays blind however many are added. Two post-sign-off findings on #11902 both lived in
  that one gap. **38/38 is not route coverage.**
- **A repair of a fail-open is a prime site for the same class one layer down.** #11902's
  own (d) repair discarded the located cause on its withheld arm and wrote a constant. Grep
  every declaration your repair *adds* for a call site before pushing, and re-read every new
  refusal arm for whether it names which fact failed.
- **At any power-of-two grain, a checked-add rounder and a subtraction rounder refuse the
  same set**, because 2^63 is a multiple of the grain. A gibibyte-grain mutation control
  stayed green under the mutation. Use a non-power-of-two grain (1000). Two authors asserted
  the opposite from reasoning alone; both were wrong.
- **A refusal-arm claim tests that the arm fires, not that the right inputs reach it.** That
  is how two lanes and a reviewer converged on bounding by the maximum instead of by
  representability.
- **`claim_batch` exits 0 over its own refusal.** A `*Refused` line is a **non-answer** —
  neither pass nor fail — and exit status must never be read as the verdict.
