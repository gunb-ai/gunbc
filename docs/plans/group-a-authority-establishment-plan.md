# Establishing Group A's pair-serving authority on the fabric log

Plan for the D0 prerequisite. Nothing here has been run. Every claim cites the symbol it was read from.
Anything not read from the live store on srv1 is marked as an inference.

## What refused, and why

Run 36220729318 (fleet-converge `spark_v41_engram_access_probe`, srv1) read
`heads/pair-serving-authority-group-a.1` and `…-group-b.1`, and refused srv8 with
`gunbc.spark.host_commitment` `group_standing_over`'s `GroupStandingUnread` arm. That arm fires when a
group's partition folds to `AuthorityUnestablished` **and** `gunbc.spark.pair_serving_authority`
`pair_serving_authority_for` finds a source row for the group.

- **group-a:** `spark_pair_serving_authorities` declares
  `PairServingActive { group: FabricGroupA, exact_realization: PairRealizationUnestablished {…} }`. The
  log has no `AuthorityEstablished` event for it, so its standing is Unread.
- **group-b:** the roster has **no** group-b row. So an unestablished group-b partition reads
  `GroupStandingUnclaimed`, and the probe doesn't refuse on it. **This corrects the brief:** the probe
  output doesn't show that group-b is *established*. It only shows group-b isn't claimed. (Inference: no
  repository route has ever written group-b's partition except the witness fixtures.)

**Why group-a was never established.** There's no route for it. The only entry point is
`gunbc.spark.pair_serving_authority_log` `pair_serving_authority_establish_wet`, and its annotation says
"run once per group by an operator on an executor with the event log placed". No fleet-converge mode
invokes it: the mode list in `.github/workflows/fleet-converge.yml` has `pair_serving_d0` but nothing for
establishment, and no run has ever carried it. The roster row landed in #11501 and the log route in
#11555, and the one-time actuation between them was never done.

## What the run requires

```
gunbc run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/spark/pair_serving_authority_log.dag \
  --function pair_serving_authority_establish_wet --arg group=group-a
```

Inputs: only `group`. The authority is the source row, never an argument. The actor is the executor's
short hostname (`observe_executor_reach`). The store is `gunbc.fabric_event_log_host`
`event_log_store_for_host`, so it must run on a host with a declared dashboard instance. For in-process
access to `/opt/gunbc/fabric-storage`, that host is srv1.

Preconditions, all enforced by `pair_serving_authority_establish` and `preparation_refusal`:

1. The group-a partition folds to `AuthorityUnestablished`. A second run refuses at `established`.
2. `establishment_refusal`: the row commits hosts, and at least one is named (srv5, srv6, srv7, srv8 via
   `fabric_group_hosts`, cage 1). The row holds no lease.
3. On `host-placement`: no pending group-a preparation, and group-a's live commitment is empty (no
   finalization yet).
4. **No live host-effect claim on srv5–8.** A live claim, such as an in-flight V4.1 build, checkpoint
   materialize or row-store encode, makes the run refuse at `host-effects`. Run it when those modes are
   idle.
5. No srv5–8 host is fenced by group-b.

## What it writes (a saga across two partitions)

1. `host-placement`: `PlacementPrepared { operation: "establish group-a", previous: [], next: [srv5..8] }`
2. `host-placement`: `PlacementAppendClaimed { preparation, intent }`
3. `pair-serving-authority-group-a`: `AuthorityEstablished { authority: <the row>, placement: <ref> }`,
   compare-and-set at `HeadAbsent`
4. `host-placement`: `PlacementFinalized`, with up to 3 attempts. If finalization fails, the run exits
   non-zero and says to run `host_placement_finalize_wet`. A stale write or a refused write aborts the
   run's own preparation (`saga_abort_own`).

## Consequence to decide before consenting

After step 4, srv5–8 are **fenced** by group-a (`group_fenced_hosts` ⊇ `group_live_commitment`).
`host_effect_admit_at` then refuses **every** new host-effect claim on those hosts with
`committed: … is fenced by an authority: group-a`. This covers the `spark_v41_*` modes that claim a
Group A host, such as `v41_runtime_image_converge`.

**Establishing the authority does not unblock the Engram probe on srv8.** It changes the refusal from
"unread" to "committed". (Inference: the exact wording depends on the probe's admission route, which
wasn't traced to its end.) So establishment is correct for D0. For the V4.1 staging modes it's a
**regression unless they admit through the group's authority** (`admit_host_held_by_subject`) rather than
through a free-host claim. Parent: sequence staging work against this.

## Operator authorization

The code has **no consent gate** on this route. `pair_serving_d0_door`, by contrast, files an escalation.
It is still an operator-consented act, because:

- It is a one-off, irreversible, privileged write to the shared production log. The only way out of the
  state is a transition, never a re-establishment.
- It fences four production hosts against all other lanes.

Under §3b's authorization-pattern selection (`gunbc.auth.authorization_pattern_selection`), a one-off
effect is **one operator approval**. The two ways to run it both need the operator:

- **(A) Recommended.** The operator runs the command above on srv1 as the serve principal (briansrls).
  briansrls writes under its own umask, so the file-mode defect is avoided. Actor = `srv1`.
- **(B)** Add a fleet-converge mode for it. This is a new modeled route in the workflow emission and
  needs operator sign-off. It also writes as ghrunner under `umask 077`, so **it must wait for #12342**
  and #12342's area-ensure repair. Otherwise the serve side can't read the four new objects or the
  group-a head: they'd be 0600.

Under (A), #12342 still matters downstream. D0 runs as ghrunner in CI (`pair_serving_d0`), and its later
appends are written 0600 until #12342 lands.

## Verification

- The command exits 0.
- Re-dispatch the read-only admission that refused: the Engram probe's admission, or D0's
  `pair_serving_d0_admit_executor`. It must now read group-a as `GroupStandingEstablished`, generation 0,
  `PairServingActive`. It must not say "has not been established".
- On srv1: `heads/pair-serving-authority-group-a.1` exists and names the `AuthorityEstablished` object.
  The `host-placement` head has advanced by three events, ending in `placement-finalized`.
- Run the command a second time. It must refuse at `established`. This is the discriminating control.
