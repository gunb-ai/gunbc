# C1 R-3 — Canonical Bench Host Decision Matrix

**Status:** PROPOSAL (audit/decision-surface only). Authored 2026-05-02 (silent-boar-29) per Director dispatch via cool-stag-230 (R3 PB).
**Parent:** `docs/audit/c1-tier3-perf-budget-readiness-matrix.md` (PR #1358), `docs/audit/c1-tier3-baseline-capture-procedure.md` (PR #1445), `docs/briefs/r3-pb-tier3-perf-budget-worker.md` (PR #1331).
**Authority basis:** readiness matrix §1 row R-3 + capture-procedure §1 STOP-A + worker brief §"Discipline / baseline noise concerns" (line 108).
**Scope:** docs-only options surface for the canonical bench host R-3 designation. **No host claimed unilaterally; no CI wiring; no runner labels added; no `tier3_baseline.json` capture; no Phase 2 work.** The actual designation is PB Manager / Director authority.

This artifact narrows the decision space to options derivable from current repo state, names the exact authority each option needs, and proposes wording the decider drops in once chosen.

---

## 1. Repo-state inventory at HEAD `<post-#1445 main>`

Single runner label across all four CI jobs in `.github/workflows/ci.yml`:

```
fmt                  → runs-on: ubicloud-standard-2
ci                   → runs-on: ubicloud-standard-2
v3                   → runs-on: ubicloud-standard-2
self_host_ratchet    → runs-on: ubicloud-standard-2
```

(Anchored by job name, not line number — `ci.yml` churns and line offsets drift.)

No other workflow files exist (`ls .github/workflows/` → only `ci.yml`). No `bench`-tagged runner, no dedicated VM, no self-hosted runner pool, no Ubicloud machine pool larger than `standard-2`. No CI step currently invokes `cargo bench`.

**Net:** the only repo-derived runner identity is `ubicloud-standard-2`. Anything else (dedicated bench VM, larger Ubicloud machine type, self-hosted runner) requires an operator-facing change outside repo state.

---

## 2. Option matrix

Each option below has: description, hardware-stability characterization, what authority must land for it to be canonical, and disposition for whether a Phase-1 0c capture against this host satisfies the worker brief's `≤2× median / ≤5× p99` budget intent.

### Option A — `ubicloud-standard-2` (status-quo Ubicloud shared instance type)

| Field | Value |
|---|---|
| Description | Use the existing CI runner label as the canonical bench host. Phase 1 0c runs `cargo bench --bench tier3_mirror_perf -p v3-compiler` on a `ubicloud-standard-2` runner (any instance of that type); Phase 2 perf measurements run on the same label. |
| Hardware stability | **Approximate.** `ubicloud-standard-2` is an Ubicloud-managed shared runner type — same instance specification (2 vCPU, fixed RAM, fixed Linux kernel image), but each CI job spins a fresh VM. Co-tenancy and VM-placement jitter contribute run-to-run variance. The brief's `≤5× p99` bracket absorbs typical shared-VM tail latency; `≤2× median` is tighter and may flag when a noisy run lands. |
| Authority needed | **PB Manager designation** (one-line addition to `docs/r3-structure.md` or sibling). No CI / infra / billing change. **Smallest authority surface.** |
| Phase 1 0c viability | **Yes**, with capture discipline: capture across N runs (suggest N=3 or N=5), record median-of-medians as `median_ns`, and the highest p99 across runs as `p99_ns` (conservative pin). Discipline must be written into the capture-procedure §2 before 0c PR opens. |
| Drift recovery | Cheap. Hardware-spec, runner-image, OS / kernel, or toolchain drift (vCPU model swap, Ubicloud image bump per <https://www.ubicloud.com/docs/github-actions-integration/runner-types>, kernel bump, rustc minor bump) triggers Director-approved recapture. Recapture cost: N independent CI invocations per §4 multi-run discipline (N≥3, suggested 5). |

### Option B — Dedicated bench runner (new label, e.g. `bench-canonical-1`)

| Field | Value |
|---|---|
| Description | Provision a dedicated runner (Ubicloud or self-hosted) labeled `bench-canonical-1`; gate Phase 1 0c capture and all Phase 2 perf workflows on `runs-on: bench-canonical-1`. Single physical VM (or pinned instance) used across runs. |
| Hardware stability | **Best.** Same VM, same kernel, same scheduler; co-tenancy controlled. p99 variance minimized. |
| Authority needed | **Director sign-off + operator action**: Ubicloud account or self-hosted-runner provisioning, billing entry, runner-token wiring, label registration in workflow. None of this is derivable from repo state. |
| Phase 1 0c viability | Yes, with stronger statistical guarantees than Option A. |
| Drift recovery | Costly. If the dedicated VM is decommissioned or migrated, baseline strands; recapture under Director approval. Recapture cost: one provisioning cycle + one CI run. |

### Option C — Named developer workstation (Brian's machine, etc.)

| Field | Value |
|---|---|
| Description | Designate a specific developer-owned host as canonical (e.g., the workstation `briansrls@gunb.ai` runs from). |
| Hardware stability | Best in isolation, but introduces author-machine dependence. |
| Authority needed | Director sign-off; conflicts with Substrate / CI norms ("CI is the authority, not author machines"). |
| Phase 1 0c viability | Possible but **discouraged**: not reproducible across team boundaries; future contributors / managers cannot recapture without Brian's machine; counts as parallel authority outside CI. |
| Drift recovery | Brittle. Hardware sale / OS reinstall / move forces recapture; team coordination cost. |

### Option D — GitHub-hosted `ubuntu-latest`

| Field | Value |
|---|---|
| Description | Switch the bench runner to GitHub's free hosted runners (`ubuntu-latest` / `ubuntu-24.04`). |
| Hardware stability | **Worst** of the four. GitHub hosted runners are highly shared; spec drift between runs is documented. |
| Authority needed | None on the infra side (free); single-line workflow change. But the brief's `≤2× median` bracket would likely false-flag regularly. |
| Phase 1 0c viability | Not recommended. Even with N-run averaging, the noise floor exceeds the budget bracket. |
| Drift recovery | N/A — no stability to lose. |

---

## 3. Recommendation surface (PB Manager call)

The decision is PB Manager authority; this matrix does NOT pick. But to make the call actionable:

- **Recommended candidate, pending PB Manager designation: Option A (`ubicloud-standard-2`)** — smallest authority surface; repo-derivable; no infra change; landed by a one-line edit per §5.1. Also pending the §4 multi-run capture addendum being accepted into the procedure document. **This matrix does NOT satisfy R-3 by recommending A**; R-3 remains unsatisfied until the designation line lands.
- **Escalation candidate, pending Director / operator action: Option B (dedicated `bench-canonical-1`)** — tightest variance; requires operator action outside repo state (budget, provisioning, runner token). Recommended escalation if Phase 2 measurement variance becomes the binding concern.
- **Options C and D are recorded only to demonstrate the decision space; not recommended.**

This matrix takes no position between A and B; A is the only one a docs-only PR can stand up without operator action.

---

## 4. Capture-discipline addendum (Option A only) — *Proposed; not active until the Option A designation line lands*

The text below is a **proposal** authored by this matrix; it is not active discipline until PB Manager (i) chooses Option A via §5.1 and (ii) accepts this addendum into `docs/audit/c1-tier3-baseline-capture-procedure.md` §2 (either by editing the procedure document or by ratifying this matrix as the §2 amendment authority). Until both happen, the capture procedure §2 single-run wording stands and Phase-1 0c capture is not authorized.

If Option A is chosen, the capture procedure must be amended with one paragraph before the Phase-1 0c PR opens:

> **Multi-run baseline capture (Option A only).** On `ubicloud-standard-2`, capture N independent runs of `cargo bench --bench tier3_mirror_perf -p v3-compiler` (N=3 minimum, N=5 preferred) within a single calendar day to bound calendar-time hardware drift. For each bench, `median_ns` in `tier3_baseline.json` is the median across the N runs' per-run median estimates; `p99_ns` is the maximum across the N runs' per-run p99 derivations (per §2.1 path). The capture PR description records the N value chosen, the per-run JSON paths, and a one-paragraph variance receipt (max/min ratio across runs per bench). Runs are independent CI invocations on `ubicloud-standard-2`, not multiple iterations within one invocation.

If Option B is chosen, single-run capture matches the original §2 wording; this addendum is unnecessary.

---

## 5. Proposed wording for `docs/r3-structure.md`

The decider drops one of the following lines into `docs/r3-structure.md` (or a sibling authority doc) under the C1 / T-Tier3-Dissolution lane block. Authority lands the moment the line is committed.

### 5.1 Option A — RATIFIED 2026-05-08 (PB Manager warm-dove-618; Director ratification at gunbc#828 c#4403509523)

> **Canonical bench host for C1 Phase 1 0c (R-3 satisfied 2026-05-08 per PB Manager):** `ubicloud-standard-2` (the existing CI runner label; see `.github/workflows/ci.yml`). Phase 1 0c capture and all Phase 2 perf measurements MUST run on this label. Multi-run capture discipline per `docs/audit/c1-r3-canonical-bench-host-decision-matrix.md` §4 (N=5 preferred, N≥3 minimum). **Recapture triggers (any of, Director-approved):** hardware-spec change (vCPU model / RAM size); runner-image change (Ubicloud `ubicloud-standard-2` image bump per <https://www.ubicloud.com/docs/github-actions-integration/runner-types>); OS / kernel change; toolchain change (rustc minor bump per `actions-rust-lang/setup-rust-toolchain` `toolchain` field).

### 5.2 If Option B chosen (then operator action follows)

> **Canonical bench host for C1 Phase 1 0c (R-3 to satisfy on operator action 2026-MM-DD per PB Manager):** dedicated runner label `bench-canonical-1` (provisioning by `<operator>`; billing entry `<entry>`). Phase 1 0c capture and all Phase 2 perf measurements MUST run on this label. Single-run capture per `docs/audit/c1-tier3-baseline-capture-procedure.md` §2.

---

## 6. STOP+PING (escalation if neither A nor B can be chosen)

If PB Manager cannot pick A or B from current information, the missing input is one of:

- **Operator approval for Option B billing**: Ubicloud account ownership / Director billing-line approval. Not derivable from repo. STOP and route to Director with the §5.2 wording template.
- **Variance budget for Option A**: if the brief's `≤2× median` bracket is treated as a hard fail-closed threshold without N-run averaging, Option A may not be acceptable; Option B becomes the only viable pick. STOP and ask whether the brief's bracket assumes single-run or multi-run discipline.

This PR does NOT escalate either; it surfaces both as routing exits if the manager call cannot land internally.

---

## 7. Acceptance summary

This matrix is intentionally bounded:

- §1 inventories repo-state runner labels (single label `ubicloud-standard-2`).
- §2 enumerates four options A–D with hardware stability / authority / Phase-1 viability / drift cost.
- §3 records the PB Manager call surface (A vs. B; C / D not recommended).
- §4 specifies the multi-run capture addendum required if Option A wins.
- §5 gives drop-in wording for `docs/r3-structure.md`.
- §6 routes STOP+PING shapes if the call cannot land.

**No host claimed. No CI wiring added. No runner labels added. No baseline JSON captured.** The R-3 designation remains PB Manager authority. **R-3 remains unsatisfied until a designation line per §5 lands** — this matrix recommends and routes; it does not satisfy R-3 by itself.
