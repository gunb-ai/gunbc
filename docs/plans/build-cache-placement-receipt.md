# Build-cache placement — step-1 live receipt (2026-08-01)

Read-only observation of the CI and session sccache instances on srv1, srv2 and srv3.
No restart, unit write, capacity change, runner drain, or host cleanup was performed.

The statement this receipt exists to make incontrovertible:

> **The configured CI cache endpoint on srv1 is owned by a process whose durable execution
> identity is the retired runner slot `srv1-10`; current cache-miss compilers are its
> children and inherit that retired slot's resource ceiling.**

## Method

Endpoint ownership is the primary observation; process name plus cgroup is not enough,
because under `RUSTC_WRAPPER` every in-flight compile also runs a process named `sccache`
in its own runner's cgroup. The chain is entirely non-mutating and invokes no `sccache`:

```
exact endpoint path in /proc/net/unix (St==01, i.e. LISTEN)
  -> listening socket inode
  -> /proc/*/fd/socket:[inode]
  -> endpoint-owner pid
  -> /proc/$pid/{exe,status,cgroup,stat}
  -> systemd unit -> runner-slot identity -> lifecycle vs current desired slots
  -> that cgroup's memory.max / memory.high / memory.current / memory.events
```

No `pgrep`, no `head -1`, no `ss`, no `lsof`, and the jobserver FIFO is never read
(`FIONREAD` only — a plain read *consumes* tokens).

## srv1 — the defect

```
desired runner slots: srv1-01 srv1-02 srv1-03 srv1-04 srv1-05

instance = ci   endpoint = /var/lib/ctrl/sccache-ci/server.sock
  EndpointOwnerObserved
    owner        boot_id=73c8153e-0e0e-4480-a36c-bc7560c50e8b pid=3975946
                 start_time=79870020  age=2697055s (31.2 days)
    executable   /usr/local/bin/sccache
    principal    ghrunner (uid 999)
    cgroup       /system.slice/system-actions\x2drunner.slice/actions-runner@srv1-10.service
    slot         srv1-10   lifecycle=RetiredRunnerSlot  (is-active=inactive, is-enabled=disabled)
    limits       memory.max=17179869184  memory.high=16106127360  memory.current≈15.0e9
    events       low 0  high 5303  max 229780  oom 118  oom_kill 12  oom_group_kill 0
    occupants    sccache(3975946)  rustc(3723134)      <-- a live runner's compile, in the dead slot
  => placement = ServerRunnerOwned { slot: srv1-10, lifecycle: RetiredRunnerSlot }  => REFUSE

instance = session   endpoint = /var/lib/ctrl/sccache/server.sock
  owner pid=3405, principal briansrls, cgroup /system.slice/ctrl-sccache.service
  => unit-owned, but by the ctrl unit, not a gunbc-managed one
```

`max 229780` and `oom_kill 12` reproduce the counters recorded in closed PR #7019.
The `high` counter moved from 5291 to 5303 over ~45 minutes of ambient CI, so the
pressure is **ongoing but episodic** — two samples 90s apart showed no movement at all.
Any cutover measurement must therefore be taken **under deliberate load**; a quiet-host
A/B would show nothing and could be misread as refuting placement.

## srv2 and srv3 — the same birth defect, without the retirement half

```
srv2  desired slots srv2-01..05
      ci endpoint owner pid=2124706 age=172s  cgroup actions-runner@srv2-05.service
      lifecycle=DesiredRunnerSlot (active, enabled)
      events: low 0 high 0 max 0 oom 0 oom_kill 0
      cgroup also holds that slot's own job: Runner.Worker, cargo, rustc
      => placement = ServerRunnerOwned { slot: srv2-05, lifecycle: DesiredRunnerSlot } => REFUSE

srv3  desired slots srv3-01 srv3-02 srv3-03 srv3-05      (note: no srv3-04)
      ci endpoint owner pid=4058886 age=5763s  cgroup actions-runner@srv3-01.service
      lifecycle=DesiredRunnerSlot (active, enabled)
      events: low 0 high 264 max 0 oom 0 oom_kill 0
      => placement = ServerRunnerOwned { slot: srv3-01, lifecycle: DesiredRunnerSlot } => REFUSE
      session endpoint absent (srv3 runs no ctrl-sccache and no session containers)
```

**This is the finding that reframes the lane.** No host has a unit-owned CI cache. The
birth-ordering defect is fleet-wide: on every host the CI server was created by a client
inside a runner's cgroup. srv1 is not a different defect — it is the host where the
defect became *permanent*, because the owning slot was later retired and a retired slot's
unit never restarts, so nothing recycles the server. srv2 and srv3 self-limit only by
accident: their owning slots are live ephemeral runners that turn over, which is why their
event counters are near zero and their servers are minutes rather than weeks old.

srv2 and srv3 are therefore **one slot retirement away from srv1's state**, and srv3
already has a slot (`srv3-04`) missing from its desired set.

## srv4 — typed unavailable, not healthy

`ssh briansrls@192.168.1.196` from the srv2 jump host returned
`Permission denied (publickey)`. Recorded as
`EndpointObservationUnavailable { ReachRefused }`, which derives `ServerPlacementUnknown`
and refuses. It is **not** recorded as absent, empty or healthy: a host that could not be
reached has not been observed.

## Controls

| Control | Result |
|---|---|
| Client false-positive | srv1 held 3 transient `sccache` clients under live slots 01/04/05 alongside the endpoint owner. Classification follows the listener owner; the clients cannot perturb it. A "multiple sccache processes ⇒ ambiguous" probe would have falsely refused on all three hosts. |
| Manufactured stats (private endpoint + private dir) | **Refuted the premise.** `--show-stats` did **not** spawn a server: UDS form 0→0 listeners, TCP form 0→0, both returning rc=0 with a table of zeroes. |
| What *does* spawn | One real compile (`sccache /usr/bin/gcc -c`) took a private port 0→1, and the new server **inherited the invoking process's cgroup**. This is the birth mechanism, reproduced deliberately. |
| Retired slot | Owner under a slot absent from desired intent ⇒ runner-owned/retired ⇒ refused. `inactive`/`disabled` do not change the verdict. |
| Absent | Endpoint with no listener ⇒ `EndpointAbsent`; observing creates no process or socket. |
| Ambiguity | Zero or multiple listener owners ⇒ typed unknown, never a guessed owner. |
| Non-mutation | Two observations 20s apart on srv1: owner pid and start_time identical, FIFO tokens 128 → 128, `memory.events` byte-identical. Only natural client churn (8 → 6 sccache processes). |

### The `StatsOk` correction

The in-tree claim that "any sccache invocation auto-starts a server, so `--show-stats`
manufactures the very evidence it reports" is **measured false** for sccache 0.15.0 and
has been corrected at `extdeps.cache.sccache` (`sccache_stats_query_does_not_spawn_note`),
`gunbc.host_build_cache_provision` (`build_cache_supervision_observation_note`) and
`gunbc.readback_independence`.

The defect is real but differently shaped, and the difference changes the justification
rather than the remedy: `--show-stats` answers with a fully-formed table **whether or not
any server exists**, so converging on it greens over a host with no durable daemon. That
is exactly what srv3 exhibited on 2026-07-25 — the observation there was right, the
inference drawn from it was wrong. `StatsOk` may therefore only ever be supplemental data
read *after* ownership is independently grounded, and the auto-spawn wall belongs on the
**compile** path, where a client actually creates a server in its own cgroup.

## What this receipt does not do

Out of scope for step 1, deliberately: capacity selection, system-service realization,
host mutation, ctrl-unit adoption, runner drain, compile-pool realization, retirement
actuation, and the CI fail-closed cutover. The 10 GiB CI capacity is an unset knob
inheriting sccache's default (`SCCACHE_CACHE_SIZE` appears nowhere in the drop-ins, nor
anywhere in gunbc/extdeps); both hosts sit pegged at it (srv1 10.00 GiB / 5,525 entries,
srv2 9.99 GiB / 3,416). That is a real fleet-wide amplifier — it raises miss frequency and
so raises how often concurrent compiler children pressure the inherited cgroup — but it is
not the placement defect and must not be changed before placement is fixed, or the higher
hit rate would mask the bad placement rather than remove it.
