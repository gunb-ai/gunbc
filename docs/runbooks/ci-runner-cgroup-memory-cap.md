# Fleet host memory fail-closed ledger (srv1 / srv2)

**Purpose:** every byte on a 128 GiB fleet host is **accounted**; Σ cgroup caps + reserved
headroom ≤ physical RAM. A heavy consumer must be **cgroup-OOM'd (exit 137, loud RED)** — never
allowed to trigger **GLOBAL host OOM** (kernel-proven: `claim_executor` 93.5 GiB anon-rss,
2026-06-20).

**Two halves (both required for full fail-closed):**

1. **CI runner slice** — caps `claim_executor` / GitHub Actions (**proven crash path**; operator
   applied **64 GiB** — keep it).
2. **Agent session containers** — dashboard spawn admits 20×31.3 GiB caps on 256 MiB reservation;
   2–3 hot sessions can exceed 128 GiB even with a capped runner. **Shape** is known; **numbers**
   need measurement + operator sign-off before apply (high blast radius).

**Not the fix:** `spawn_width=1` (serializes ~500 witnesses, perf regression). #5375 keeps
memory-aware **width=4**; containment is cgroup caps.

---

## Scaffold register (in-repo authority)

Persisted in `dsl/product/compute_fabric.dag` — each row is a placeholder until
**generated-from-the-ledger**:

| Scaffold row | Current value | Dissolve when |
|---|---|---|
| `host_memory_scaffold_runner_slice_cap` | **64 GiB** (operator live) | `max(measured normal-CI floor VmHWM @ width=4, memory.peak runner slice)` + pig-tail margin → ledger cap |
| `host_memory_scaffold_ci_per_shard_peak` | **14 GiB** (stale §3 fork) | measured per-witness resolve peak distribution (`claim_batch` `[interp-stats]` VmHWM) |
| `host_memory_scaffold_session_container_cap` | **31.3 GiB** (live dashboard) | operator-signed cap from measured per-session peak-RSS distribution |
| `host_memory_scaffold_session_spawn_reservation` | **256 MiB** (kernel census) | generated-from-the-ledger `admit()` reservation per `AllocationClass` |

`gunbc_ci_floor_corpus_work_demand` reads `host_memory_scaffold_ci_per_shard_peak` — do **not**
size the runner cap from `4 × 14 GiB`; we have **no measured normal-CI peak** yet.

---

## Ledger (128 GiB physical, BMC-grounded 2026-06-20)

Caps must **not** sum to 100% of physical — page-cache, dockerd, dashboard, and cgroup slop
need unbudgeted headroom.

| Budget line | GiB | Enforced how | Status |
|---|---:|---|---|
| Physical RAM | 128 | BMC `total_memory_bytes` | grounded |
| **Unbudgeted headroom** | **12** | *leave free* | required |
| OS + kernel reserve | 8 | implicit | — |
| **Budgeted consumer caps** | **116** | cgroup `MemoryMax` | — |
| → CI runner (`system-actions-runner.slice`) | **64** | systemd slice (operator applied) | **APPLY / KEEP** |
| → Agent sessions (aggregate budget) | **36** | dashboard spawn (illustrative) | **MEASURE → operator policy** |
| → sccache + shared build temp | 8 | ops hygiene | — |
| **Σ** | **128** | fail-closed invariant | — |

**Why runner stays 64 GiB:** conservative placeholder already live on the operator host. Dropping
to 56 GiB assumed `4 × 14 GiB` normal peak — **14 GiB is the stale scaffold constant**, not a
measurement. Until normal-CI VmHWM is recorded, 64 GiB is the safe cap (pig runs OOM inside the
slice; normal runs must not false-RED).

**Session aggregate 36 GiB** is headroom math only (`128 − 12 − 8 − 64 − 8`). Per-container cap
and concurrency are **not** ready-to-route — see Part B.

---

## Measured facts (populate before resizing caps)

| Signal | Source | Status |
|---|---|---|
| Pig / crash peak | kernel OOM: `claim_executor` **93.5 GiB** anon-rss (srv2, 2026-06-20 02:55) | **grounded** |
| Normal CI floor peak RSS | `claim_batch` / `claim_executor` end-of-run `[interp-stats]` **VmHWM** @ width=4, or `memory.peak` on `system-actions-runner.slice` during a green floor | **pending measurement** |
| Session peak-RSS distribution | RC-5 / capacity census: most sessions **< 1 GiB** (LLM idle); few spike to **~31 GiB** on cargo/ctrl-build | **shape known, distribution TBD** |
| Per-session container cap (live) | `memory.max=33578549248` (~31.3 GiB), 20 admitted | **grounded** |
| Spawn reservation (live) | 256 MiB kernel admission | **grounded** (`host_memory_scaffold_session_spawn_reservation`) |

Record normal-CI VmHWM in this table when captured; runner cap dissolves off that row.

---

## Part A — CI runner cap (operator: applied, keep 64 GiB)

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
MemoryMax=64G
MemoryHigh=60G
EOF
sudo systemctl daemon-reload
```

### Live apply (running slice — `daemon-reload` alone is NOT enough)

```bash
sudo systemctl set-property system-actions-runner.slice MemoryMax=64G MemoryHigh=60G
```

### Verify

```bash
systemctl show system-actions-runner.slice -p MemoryMax -p MemoryHigh -p MemoryCurrent
# Expect MemoryMax=68719476736 (64 GiB), MemoryHigh=64424509440 (60 GiB)

systemd-cgtop -m   # during a CI run — capture memory.peak for scaffold dissolution
```

### Rollback

```bash
sudo systemctl set-property system-actions-runner.slice MemoryMax=infinity MemoryHigh=infinity
sudo rm /etc/systemd/system/system-actions-runner.slice.d/50-memory-cap.conf
sudo systemctl daemon-reload
```

---

## Part B — Agent session cap (RECOMMENDATION — do not route to operator yet)

**Problem:** sum of session container caps is unbounded (20 × 31.3 GiB). 2–3 build-heavy
sessions + runner can GLOBAL-OOM the host.

**Correct shape (for step-4 / operator policy later):**

- per-container `MemoryMax` cap
- max concurrent sessions per host
- spawn admission: **refuse / queue / redirect srv1** when aggregate would exceed session budget

**Illustrative numbers (NOT ready-to-apply):** e.g. 12 GiB/container × 3 concurrent = 36 GiB
fits the ledger headroom math — but this **cuts ~20 → 3 sessions/host** and may OOM
cargo/ctrl-build sessions. RC-5: most sessions use **< 1 GiB**; only a few spike to **31 GiB**.
A uniform tight cap punishes idle sessions to contain build-heavy outliers.

**Before operator routing:**

1. Measure per-session peak-RSS distribution over time (dashboard / cgroup metrics).
2. Present cap + concurrency as a **recommendation with capacity trade-off** for operator
   sign-off (affects every running agent).
3. Dissolve `host_memory_scaffold_session_container_cap` from that data.

Runner cap already kills the **proven repeated crasher**; session-spike GLOBAL OOM is a rarer
residual sealed with data + policy, not a rushed blast-radius change.

---

## In-repo model (destination, not active stopgap)

| Layer | Role |
|---|---|
| `#5375` `placement_spawn_width` | memory-aware **width=4** inside runner (throughput) |
| **This ledger + scaffold register** | host **containment** placeholders + dissolution triggers |
| `AllocationClass` + `admit()` | **step-4 destination** — full Σ-live accounting via dashboard |

**Follow-on:** eager-boar-790 slims Arc1 pig witnesses; allocator step 4 routes all spawns
through `admit()`.

---

## Evidence

- Kernel: srv2 `journalctl -k -b -1` — GLOBAL OOM 02:55, `claim_executor` 93.5 GiB anon-rss
- Census: 20 session containers × 31.27 GiB caps, 125 GiB host (`compute-fabric-allocator.md`)
- Plan: `docs/plans/compute-fabric-allocator.md`
