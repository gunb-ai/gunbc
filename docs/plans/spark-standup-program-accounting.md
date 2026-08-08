# DGX Spark standup — program accounting at wind-down (2026-08-07)

**Why this file exists.** The entire Spark hardware program is absent from `ROADMAP.md`
(`Spark` 0, `DGX` 0, `srv5`/`srv6` 0, `OOBE` 0, `PXE` 0, `vLLM` 0 occurrences at the time of
writing). Eight lanes, nine PRs, and two physical machines had no planned home, and several
findings existed only in dashboard messages that do not survive an archive. This note is the
durable record; the `RoadmapNode` rows derived from it are the follow-up.

**Hardware standing, unchanged all day:** 0/2 Sparks provisioned, 0/2 serving, rack Pi not
admitted, 0/1 controllers. The only merged change touching hardware is #7992 (srv5/srv6
allocation).

## Physical facts (settled, operator-supplied)

- `srv5` = router binding `spark-a3ee` = `192.168.1.222`
- `srv6` = router binding `spark-3bd5` = `192.168.1.223`
- Allocation was **arbitrary and operator-delegated** — it carries an `OperatorAllocation`
  receipt on main precisely so it is never re-presented as a discovered fact.
- Rack Pi: Raspberry Pi 3 Model B+, `192.168.1.197`, single radio (so `SameRack` does **not**
  imply `WirelessReachable`).

## Lane accounting

| Lane | PR | Standing at wind-down |
|---|---|---|
| SPARK-ALLOC-LAND-0 | #7992 | **merged** |
| SPARK-PROVISION-0 | #7950 | merge-ready, 2/2, `CLEAN` — owns `std.human_intervention` |
| SPARK-PXE-0 | #7949 | frozen, 2/2, checks pending |
| long-lane extraction | #8010 | clean, `+6/-0`, verified — tree-blocker fix |
| SUBSTRATE-HOLLOW-FIX-0 | #7995 | draft, 2/2 recovered, content-complete |
| SECRET-PROVISION-0 | #7973 | frozen, review in flight |
| SPATIAL-0 | #7981 | frozen on a real `REQUEST_CHANGES` |
| SECRET-VERSION-LIFECYCLE-0 | #8000 | draft, generic kernel only |
| PI-REACH-0 | #7971 | closable — value extracted to #8010 |
| HOST-NAME-0 | #7960 | code-complete, unreviewed; off the critical path |

**Merge-order hazard:** #7949 and #7950 each carry their own `human_intervention.dag` and main
has none. #7950 merges first (it is the ruled authority); #7949 then needs a mechanical
delete+import.

## The critical path to a serving Spark

```
#7950 lands (intervention authority)
  -> #7995 lands (generic Secret Manager decoder, synthetic fixtures)
  -> shared gunbc.github_actions_wif extracted from #7971
  -> SPARK-CANARY-POST-OOBE-0 (SPECIFIED, NOT SPAWNED)
  -> SPARK_CANARY_READY_FOR_MANUAL_OOBE
  -> operator performs OOBE on spark-a3ee exactly once
  -> no further human action
  -> SPARK_CANARY_SERVING
  -> second unit automated via Pi or the first Spark's own Wi-Fi
```

`SPARK-CANARY-POST-OOBE-0` is the missing executable middle. It was **not spawned** during the
wind-down: the fleet could not supply CI rounds, and `rustc` was being SIGKILLed by an
ancestor-cgroup limiter (reproduced by three independent lanes), so a new lane would have
produced modeling against a tree that could not build it.

Its readiness bar is 15 items and several are unimplemented: password-based initial SSH
(installing the fleet key without `ssh-copy-id` or an operator terminal), sudo through the same
bootstrap credential, post-OOBE host-key rescan **and regeneration if the two units ship with
shared host keys**, disabling password SSH only after fleet-key login succeeds, pinned NVIDIA
collector bundle with stdout/stderr/exit capture, pinned serving runtime + image digest + model
revision + API auth, and a real receipt written and uploaded (`upload-artifact` with
`if-no-files-found=error`, not a path string behind `|| true`).

**Standing constraint:** after `ManualFactoryOobeCompleted` the state machine must contain no
`HumanInterventionRequired` arm. `Applied | Interrupted{resumable} | Refused{exact cause}` only.
The administrator credential is pre-generated and stored *before* OOBE precisely so no second
manual handoff is ever required.

## Findings with no PR home

These would otherwise have died with the dashboard messages.

**srv3 BMC credential state is `Unobserved` — a retracted claim.** It was asserted here that
srv3 still runs the OpenBMC published factory default. That inference was **wrong** and is
withdrawn. What is actually established: the *stored payload* for `bmc-srv3-admin` version 3
equals the published default. Whether the live BMC accepts it was never observed. The fixture
claiming wet Pi-lane provenance could not have produced it (the Pi observer fetches
`fleet_automation_ssh_privkey_secret`), and merged July 23 work reports srv3 moved to version 6
and srv4 to version 2 with pre-rotation versions destroyed. **No rotation may proceed without a
read-only WIF-backed probe** that tests factory acceptance and exact-version managed acceptance
*independently* — a product, not a first-match coproduct, because both-accepted and
neither-accepted are real states with different remedies. srv4 must be observed, never assumed
to match srv3.

**Five BMC credential-lifecycle defects**, owed to a future `BMC-CREDENTIAL-ROTATION-0`:
1. credential value conflated with Secret Manager base64 wire encoding;
2. `latest` used as an active-credential pointer (no operational consumer may use `latest` —
   use an exact `ActiveBmcCredentialBinding`, and stage the candidate as Pending so a crash
   between `AddVersion` and target mutation resumes the *same* version rather than minting
   another password);
3. steady-state code permits the factory credential (`factory_acceptable: true`);
4. validators validating the wrong state (`bmc_onboard_validate` greens on factory login
   success — correct as a first-contact probe, opposite of correct as convergence validation);
5. first-wins probing.
Precedent: #7138 found Redfish treating HTTP 401/403/5xx as success because `curl` lacked
`--fail`. Keep four facts independent forever — process exited zero / HTTP succeeded /
credential changed / new credential authenticates.

**`resolve_issued_attempt` requires a predicted version identity** (#8000, recorded on the PR).
It exists for the case where `AddVersion` was issued and its response never seen — so the caller
cannot hold the returned identity, and the only way to supply the `expected` argument is to
*predict* the version number. That is the exact defect the exactly-once wall exists to prevent,
surviving where no returned identity exists to bind. Fix: resolve against a pre-attempt version
snapshot, or bind an upstream request identity — both derive identity from observation.

**Generic `T?` returns collapse under inference.** `fn first_of(items: List<T>) -> T?` loses the
Optional; matches then report `Present`/`Absent` as missing variants *of the element type*, so
six errors landed on unrelated type declarations far from the fault.

**`exit 137` is uninformative until controlled.** A resolve SIGKILL was reported as a substrate
defect (two inhabited binding rows) and held a lane for a session. A control run on unmodified
main died identically — host memory pressure. The shape resolves cleanly on a quiet host.

## Test-input rule (operator ruling)

> Use a domain authority as the test input when testing that domain fact or a policy derived
> from it. Use synthetic data when testing transport, encoding, or generic mechanics.

So the generic Secret Manager decoder fixture stays **synthetic and fictional** — deriving its
base64 from `openbmc_factory_login` would make it an encoder/decoder round trip where both sides
can be wrong together. BMC-specific witnesses consume `openbmc_factory_login` *symbolically*
rather than restating its literals. A managed replacement credential is generated from entropy
and policy only; its sole permitted relationship to the published default is a **refusal** on
equality.
