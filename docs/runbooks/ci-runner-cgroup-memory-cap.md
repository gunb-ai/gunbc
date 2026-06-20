# CI runner cgroup memory cap (srv1 / srv2)

**Purpose:** fail-closed host containment for `claim_executor` / GitHub Actions floor runs.
A heavy CI job must be **cgroup-OOM'd (exit 137, CI RED)** — never allowed to trigger a
**GLOBAL host OOM** that takes down agent sessions and the kernel.

**Context (kernel-proven 2026-06-20):** `claim_executor` is ONE process; `spawn_width` =
concurrent witness threads inside it, each holding a large `Rc` resolve graph. Peak RSS ≈
`width × per-witness-peak`. The srv2 GLOBAL OOM at 93.5 GiB anon-rss matches ~3–4 concurrent
~31 GiB “pig” witnesses (Arc1-class eager resolves). #5375 keeps memory-aware `spawn_width=4`
for throughput; **this runbook caps the runner slice** so pigs fail loud without serializing
the whole corpus.

**Not the fix:** forcing `spawn_width=1` (perf regression — serializes ~500 witnesses).
**Is the fix:** hard `MemoryMax` on `system-actions-runner.slice`.

---

## Sizing (128 GiB hosts, BMC-grounded 2026-06-20)

| Budget line | GiB | Notes |
|---|---:|---|
| Physical RAM (srv1/srv2) | 128 | BMC `total_memory_bytes` |
| OS + kernel + journald | 8 | headroom for livelock avoidance |
| Agent sessions (concurrent under load) | 48 | 2026-06-20 census ~30 GiB idle, ~48 GiB stressed |
| sccache + build temp | 8 | shared host cache |
| **CI runner slice (`MemoryMax`)** | **64** | **apply this cap** |
| **Total** | **128** | tight but sums |

**Why 64 GiB for the runner**

- Normal floor at #5375 width **4** × **14 GiB**/unit modeled peak ≈ **56 GiB** → fits under 64 GiB.
- Pig floor at width **4** × **~31 GiB**/witness ≈ **124 GiB** → **cgroup OOM (exit 137)** under 64 GiB
  (CI goes RED — correct fail-closed) instead of GLOBAL host OOM.
- A single uncapped pig at ~31 GiB still fits under 64 GiB; multiple concurrent pigs at width 4
  are the case this cap contains.

**Per-host note:** srv1 historically had a lower effective runner cap (~65.6 GiB observed). The
same **64G** cap is safe on both 128 GiB hosts; tighten srv1 only if operator measures less RAM.

---

## Apply (requires root on each fleet host)

Run on **srv1** and **srv2** as root (or via ansible/ctrl):

```bash
# Inspect current slice memory settings (optional)
systemctl show system-actions-runner.slice -p MemoryMax -p MemoryHigh -p MemoryCurrent

# HARD cap — fail-closed containment (persistent across reboot)
sudo mkdir -p /etc/systemd/system/system-actions-runner.slice.d
sudo tee /etc/systemd/system/system-actions-runner.slice.d/50-memory-cap.conf <<'EOF'
[Slice]
# Fail-closed: CI runner cannot GLOBAL-OOM the host. Pig runs exit 137 inside the slice.
MemoryMax=64G
# Soft throttle before hard kill (optional but recommended)
MemoryHigh=60G
EOF

sudo systemctl daemon-reload
# Existing runner jobs pick up on next service restart; new jobs immediately.
sudo systemctl restart 'actions.runner.*' 2>/dev/null || true
```

**Verify:**

```bash
systemctl show system-actions-runner.slice -p MemoryMax -p MemoryHigh
# Expect: MemoryMax=68719476736 (64 GiB)  MemoryHigh=64424509440 (60 GiB)

# During a CI run, watch slice usage:
systemd-cgtop -m
# or:
cat /sys/fs/cgroup/system.slice/system-actions\x2dactions\x2drunner.slice/memory.current
```

**Rollback:**

```bash
sudo rm /etc/systemd/system/system-actions-runner.slice.d/50-memory-cap.conf
sudo systemctl daemon-reload
```

---

## Relationship to in-repo model

| Layer | Role |
|---|---|
| `#5375` `placement_spawn_width` | memory-aware **width=4** inside the runner (throughput) |
| **This cgroup cap** | host **containment** — pigs → exit 137, not srv2 reboot |
| `product.compute_fabric` `AllocationClass` + `admit()` | **destination** (allocator step 4) — routes dashboard sessions through sum-invariant accounting; not the active CI stopgap |

**Follow-on:** eager-boar-790 slims Arc1 pig witnesses; probe-derived per-unit memory replaces the
14 GiB constant when measured.

---

## Evidence pointers

- Postmortem: `docs/postmortems/2026-06-20-ci-flakiness-fleet-overcommit.md`
- Allocator plan: `docs/plans/compute-fabric-allocator.md` (step 4 = dashboard admit)
- Floor width model: `dsl/gunbc/ci_fleet.dag`, `dsl/product/compute_fabric.dag` (`placement_spawn_width`)
