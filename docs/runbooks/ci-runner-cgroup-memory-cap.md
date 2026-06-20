# Fleet host memory fail-closed ledger (srv1 / srv2)

**Purpose:** every byte on a 128 GiB fleet host is **accounted**; Σ cgroup caps + reserved
headroom ≤ physical RAM. A heavy consumer must be **cgroup-OOM'd (exit 137, loud RED)** — never
allowed to trigger **GLOBAL host OOM** (kernel-proven: `claim_executor` 93.5 GiB anon-rss,
2026-06-20).

**Two halves (both required):**

1. **CI runner slice** — caps `claim_executor` / GitHub Actions (proven crash path).
2. **Agent session containers** — dashboard spawn currently admits 20×31.3 GiB caps on 256 MiB
   reservation; 2–3 hot sessions alone can exceed 128 GiB even with a capped runner.

**Not the fix:** `spawn_width=1` (serializes ~500 witnesses, perf regression). #5375 keeps
memory-aware **width=4**; containment is cgroup caps.

---

## Ledger (128 GiB physical, BMC-grounded 2026-06-20)

Caps must **not** sum to 100% of physical — kernel page-cache, dockerd, the dashboard node
process, and cgroup accounting slop need unbudgeted headroom.

| Budget line | GiB | Enforced how |
|---|---:|---|
| Physical RAM | 128 | BMC `total_memory_bytes` |
| **Unbudgeted headroom** (page-cache, dockerd, dashboard, slop) | **10** | *not capped — leave free* |
| OS + kernel + journald (operational reserve) | 8 | implicit |
| **Budgeted consumer caps** | **110** | cgroup `MemoryMax` |
| → CI runner (`system-actions-runner.slice`) | **56** | systemd slice drop-in + live `set-property` |
| → Agent sessions (aggregate budget) | **38** | dashboard spawn: per-container cap × concurrency gate |
| → sccache + shared build temp | 8 | ops hygiene / `SCCACHE_DIR` on capped fs |
| **Σ (headroom + OS + caps)** | **128** | fail-closed invariant |

**Why runner 56 GiB (not 64):** normal floor width **4** × **14 GiB** modeled ≈ **56 GiB** —
fits exactly; pig runs **4** × **~31 GiB** ≈ **124 GiB** → cgroup OOM under 56 GiB (exit 137).

**Why sessions 38 GiB aggregate:** kernel fact — 20 containers each `memory.max=33578549248`
(~31.3 GiB), admitted on 256 MiB reservation. **2–3 hot sessions = 62–94 GiB** without the
runner. Budget 38 GiB ⇒ e.g. **max 3 concurrent** sessions at **12 GiB** each (= 36 GiB).

---

## Part A — CI runner cap (apply first: proven crash)

### Inspect

```bash
systemctl show system-actions-runner.slice -p MemoryMax -p MemoryHigh -p MemoryCurrent
```

### Persistent drop-in (survives reboot)

```bash
sudo mkdir -p /etc/systemd/system/system-actions-runner.slice.d
sudo tee /etc/systemd/system/system-actions-runner.slice.d/50-memory-cap.conf <<'EOF'
[Slice]
# Fail-closed: CI runner cannot GLOBAL-OOM the host. Pig runs exit 137 inside the slice.
MemoryMax=56G
MemoryHigh=52G
EOF
sudo systemctl daemon-reload
```

### Live apply (running slice — `daemon-reload` alone is NOT enough)

```bash
sudo systemctl set-property system-actions-runner.slice MemoryMax=56G MemoryHigh=52G
```

### Verify

```bash
systemctl show system-actions-runner.slice -p MemoryMax -p MemoryHigh -p MemoryCurrent
# Expect MemoryMax=60129542144 (56 GiB), MemoryHigh=55834574848 (52 GiB)

systemd-cgtop -m   # during a CI run
```

### Rollback

```bash
sudo systemctl set-property system-actions-runner.slice MemoryMax=infinity MemoryHigh=infinity
sudo rm /etc/systemd/system/system-actions-runner.slice.d/50-memory-cap.conf
sudo systemctl daemon-reload
```

---

## Part B — Agent session cap (dashboard / ctrl spawn — other half)

**Problem:** dashboard `docker run` spawn sets per-container `memory.max ≈ 31.3 GiB`
(`33578549248` bytes) with only 256 MiB kernel reservation. Sum of caps is unbounded; 2–3 hot
sessions + runner exceeds 128 GiB → GLOBAL OOM **without** any CI floor running.

### Operator / ctrl changes (route to dashboard spawn config)

| Knob | Current | Target | Rationale |
|---|---|---|---|
| Per-session container `MemoryMax` | ~31.3 GiB | **12 GiB** (`12884901888`) | single session cannot hoard a third of the host |
| Max concurrent agent sessions **per host** | ~20 admitted | **3** | 3 × 12 GiB = 36 GiB ≤ 38 GiB session budget |
| Spawn admission gate | none (always `docker run`) | **refuse** when `live_sessions × cap > 38 GiB` | fail-closed: queue or redirect to srv1 |

### Example `docker run` flags (dashboard spawn handler)

```bash
# Replace existing ~31G cap with 12G hard / 10G soft:
docker run ... \
  --memory=12g \
  --memory-swap=12g \
  --memory-reservation=512m \
  ...
```

### Verify on host

```bash
# Per-container cap (inside a running session container):
cat /sys/fs/cgroup/memory.max
# Expect: 12884901888

# Count hot sessions and aggregate caps:
docker ps --format '{{.Names}}' | wc -l
# dashboard should refuse spawn when count >= 3 on this host (policy)
```

### Fail-closed behavior

- Session spawn when host at capacity → **refused** (operator sees queue/backpressure), not
  admitted-and-OOM-later.
- Single pig session > 12 GiB → **container OOM**, not host GLOBAL OOM.
- CI pig inside runner → **slice OOM exit 137**, CI RED.

---

## In-repo model (destination, not active stopgap)

| Layer | Role |
|---|---|
| `#5375` `placement_spawn_width` | memory-aware **width=4** inside runner (throughput) |
| **This ledger** | host **containment** — cgroup caps on runner + sessions |
| `AllocationClass` + `admit()` | **step-4 destination** — full Σ-live accounting via dashboard |

**Follow-on:** eager-boar-790 slims Arc1 pig witnesses; allocator step 4 routes all spawns
through `admit()`.

---

## Evidence

- Kernel: srv2 `journalctl -k -b -1` — GLOBAL OOM 02:55, `claim_executor` 93.5 GiB anon-rss
- Census: 20 session containers × 31.27 GiB caps, 125 GiB host (`compute-fabric-allocator.md`)
- Postmortem: `docs/postmortems/2026-06-20-ci-flakiness-fleet-overcommit.md`
