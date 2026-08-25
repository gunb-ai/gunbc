# Runner service capacity convergence

**Status:** design, not implemented. Written 2026-08-25 from a live incident on srv1/srv3.

## Why this exists

Fleet convergence reported hosts converged while they served no traffic. srv1 held **50 slot
directories and 5 running services**; srv3 went from 8 directories to 21 while its running count
stayed at 5. Nothing in the pipeline could notice, because the convergence object is filesystem
membership and the subject anyone actually cares about is **service capacity**.

Filesystem convergence is a legitimate *subordinate* family. The defect is that it was read as though
it implied serving capacity.

## The incident chain, in order, each blocker masking the next

```
1. ghrunner held no sudo grants        -> apply could not reach installation
2. grants installed                    -> 13-16 runner trees could be materialized
3. install path does not activate      -> directories existed, capacity unchanged
4. units enabled/started by hand       -> pinned runner v2.334.0 reached GitHub
5. GitHub refused fresh 2.334.0        -> systemd restarted unusable processes forever
6. release replaced with v2.336.0      -> serving capacity finally rose
```

The privilege work was necessary and could never have been sufficient: it proved the host could
execute the installer, never that the provider would accept what was installed.

## Three defects, all still open

### 1. Membership is not incarnation-sensitive

`runner_slot_member_value_eq` compares only `slot_dir`. A 2.334.0 tree and a 2.336.0 tree at the same
path are the same member, so a release bump repairs nothing that already exists.

**The receipt:** 29 slot directories were deleted by hand across srv1 and srv3 to force
re-provisioning. Replacement could only be induced by turning an existing member into an absence.

`LocalRunnerSlotIncarnation` carries slot, unit, unit_active and registration_name -- not the
installed artifact -- so it cannot express *same slot, same unit, same registration, different
release*, which is exactly the state that hurt.

### 2. The install path cannot activate — INTENT LANDED, NOTHING ACTIVATES YET

The emitted `apply.sh` carried `daemon-reload` and `disable`. Enable count 0, start count 0.

And the two are separate transitions that must not be re-fused: `enable` is durable boot intent,
`start` is immediate activity. Both consume the activation admission, because enabling authorizes a
future reboot to start the broker under readiness conditions that may no longer hold.

**What landed.** `gunbc.runner_service_activation` models the two transitions, admits each one
separately against `admit_runner_activation`, and derives the emitted lines from
`gunbc.executor_privileged_operation`'s new `SystemdEnableUnit` / `SystemdStartUnit` — so the
sudoers grant and the executed command are one argv, and the fused `enable --now` the hand loop ran
has no grant on any executor host. `fleet_converge_plan` carries an activation family beside the
slot family, with its own line in `plan.txt` and its own refusal count.

**What did NOT change, and this is the part to read before quoting a green apply as converged
capacity.** The admission is unreachable in production, and two blockers sit **in series**:
first, no host observation transaction exists, so every production caller supplies
`ActivationReadinessUnobserved` and the admission is never asked; behind that,
`admit_runner_activation` requires a `HostCompilePoolReady` whose producer answers
`PoolNotDeclaredInTopology` fleet-wide under `CompilePoolInRunnerSlots`. Flipping the placement
changes nothing observable while the first stands — the pool receipt is a field *inside* a readiness
value that is never `Observed`. So both transitions refuse on every host today, `apply.sh` prints a
typed located refusal for each, and serving capacity still does not rise from apply alone.

That state is declared as a typed `GuaranteeStall` row — `runner_activation_reachable_green_stall`
— rather than left in prose, for the reason §4b gives: a gate whose only reachable state is refusal
has to say so, or a reader cannot tell a permanent structural refusal from a transient one. The row
names the observation transaction as the immediate blocker and the placement behind it, in that
order, because naming only the second produces a plan that does the fleet-wide topology work and
measures no change.

**Unreachable in production is not inert.** §4b judges reachability against what a *fixture* may
author, and `test.claim.runner_service_activation` authors `ActivationReadinessObserved` and drives
the admitted enable/start path with it — so the emission has an authorable, exercised RED at the
fixture boundary.

**What the item actually bought,** stated without inflation: an invisible hand loop became a named,
counted, located refusal. `enable count 0` was indistinguishable from a host with no activation
work; `activation-refusals=2` with its cause is not.

**The host boundary is mitigated, not structural, and the safety row says so.** The grants install
the raw `systemctl enable <unit>` / `systemctl start <unit>` verbs for `ghrunner`, so the grantee can
run them directly, outside the admission entirely. Three claims must be kept apart: *enable and start
are separately authorized* is structural (no derived sudoers line matches `enable --now`); *this
emitter cannot activate without readiness* is structural inside the modeled call graph; *the host
cannot activate without readiness* is **false**. `runner_activation_safety_guarantee` records
`Mitigatable` with the next-rung trigger — the job user stops holding the verbs, via a root-owned
helper that validates a receipt or a root-owned convergence service. A narrower sudoers match cannot
climb it: any grant that lets the sanctioned path run the command lets the grantee run it too.

**Next.** The host observation transaction — the shared blocker this document already names — and
only then the placement flip, with a width decision attached and re-derived per host from live
`nproc`/`MemTotal` rather than from `gunbc_jobserver_token_override_note`, which is stale for srv1.
Until both land, capacity increases remain manual, and the refusal in `apply.sh` is the place that
says so.

### 3. Provider admissibility is unmodeled

Three different guarantees were collapsed into one pin:

```
ArtifactIntegrity      these are the exact bytes and digest we chose      HELD
ArtifactAvailability   the vendor still publishes those bytes             HELD
ProviderAdmissibility  the vendor's control plane still permits them      MISSING
```

An exact digest proved we faithfully installed the wrong operational version.

`Listening for Jobs` is **not** acceptance -- the rejection arrives after that line.

## Serving vs restartable capacity

The distinction the incident exposed, and the one worth carrying as a number rather than prose:

```
serving_count      members providing capacity now
restartable_count  members whose fresh incarnation the provider still admits
```

Before the replacement srv3 was `ServingButNotRestartable`: five runners on pre-deprecation
registrations, alive only until something restarted them. A fleet that meets load only while nothing
restarts is not converged, and today no carrier can say so.

## The layered object

Independently observed families composing into one verdict -- not a single `runner healthy` Bool:

```
RunnerMaterializationStanding = Absent | CorrectIncarnation | WrongIncarnation | Unobserved
RunnerEnablementStanding      = Enabled | Disabled | Unobserved
RunnerActivityStanding        = Active | Inactive | Activating | Failed | Unobserved
RunnerProviderStanding        = ExistingSessionServing | FreshMessageChannelAccepted
                              | FreshJobAccepted | RejectedDeprecated | Unobserved
```

The host verdict is over the **exact desired GitHub identities**, never raw committed width -- srv3 is
precisely where those differ, and a fabric identity must never inhabit the GitHub service family.

## The shared blocker: there is no host observation transaction

Both remaining repairs wait on the same missing seam. `fleet_converge_plan` observes directory
membership and nothing else -- zero matches for `BuildCacheInstanceReady`, `HostCompilePoolReady` or
`admit_runner_activation`. It cannot answer:

```
which release is installed in this slot?
is the unit enabled? active? failed?
does this exact cache / compile-pool receipt exist?
does GitHub accept a fresh incarnation of this release?
```

The repair is a real observation transaction feeding several reconcilers -- **not** manufacturing the
receipts in the planner, and not an activation action that permanently refuses.

**Every sub-observation needs its own unavailable arm.** One failed `systemctl` read must not erase a
successful installed-version observation, and one unavailable version read must not become
`WrongIncarnation`.

## The compatibility probe, and why it need not run a job

Two questions hide inside "will GitHub hand work to it":

1. will the broker accept this fresh runner version on its message channel?
2. will the scheduler match and deliver an actual job?

**Only the deprecation class is question 1**, and a no-job probe settles it. The runner opens a broker
session then polls a message endpoint, sending `runnerVersion` among other fields; the broker maps
`RunnerVersionTooOld` to exactly the access-denied failure observed on srv3. So a successful
message-poll from a *fresh incarnation* is the discriminator. A canary job is only needed for
question 2, which is not what deprecated us.

Scope it per `release x architecture x scope` with a bounded validity age -- not per slot.

## Do not make "latest" the authority

That trades reproducibility for churn and is wrong during a progressive rollout or a bad release. The
policy is a reviewed pinned candidate plus a compatibility receipt plus a declared maximum
observation age. A new upstream release creates an *obligation* -- test, review the pin change, plan a
rolling replacement -- never an automatic fleet mutation.

## Rolling replacement, not restart

When the desired artifact changes, per slot: stage the correct artifact, drain the old broker, replace
the incarnation, start the new broker, prove fresh provider acceptance, then continue.

**Do not restart the old runners first.** In this incident restart was precisely what converted latent
old capacity into visible failure.

## Sequencing

```
materialization plan/apply -> independent materialization readback
fresh cache/pool observations -> activation admission -> enable -> start -> service readback
provider compatibility readback -> serving/restartable capacity receipt
```

An installation plan minted before readiness must never later authorize starting a runner: activation
needs a fresh subject and fresh observations.

## Landed vs open

**Landed** (PR #9152): one argv grounds both the executed command and its sudoers grant; run-as
narrowed `ALL` -> `root`; wildcards eliminated; the bootstrap principal recorded as a typed row with a
witness (`ubuntu`, not the deploy row's `ssh_target`); release bumped to 2.336.0.

**Built, not landed** (`session/warm-tern-755-runner-activation`): the three-axis service standing with
its 13-row truth table and mutation receipt. Split out for having no production consumer -- which is
this document's shared blocker.

**Open:** incarnation-sensitive membership; the host observation transaction; provider admissibility;
serving/restartable counts; the compile-pool placement flip that gives the activation admission a
reachable green.
