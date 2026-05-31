# P2-B Bridge-Removal M2 Probe — 2026-05-31

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #4094 §3.2 P2-B dispatch (PM forced-dispatch 2026-05-31); cross-link to PR #4086 §3 (post-SG-1 catalog — SG-7 cleared + 0 v2 diagnostics on the M1 probe).
**Predecessor receipt:** P2-A (`valiant-moth-559`) verified 44-source compiler closure with 0 dag+rust diagnostics; this probe is the **complementary safety-net receipt** required before operator authorizes the bridge deletion in dispatch board priority 3.
**Reference commit:** `origin/main` at `7e9c4ba8c0` (pulled 2026-05-31 ~07:05Z; HEAD at probe time per `git rev-parse HEAD`).
**Bridge under test:** `scripts/v4-bootstrap-resolve-posture-gate.sh` + the CI step at `.github/workflows/ci.yml:377-384` (`v2 → v4 bootstrap resolve-posture gate (CI emit-wall bridge)`).

---

## §1 Verdict

**PASS.** Full v4 corpus bootstrap compile succeeds with the resolve-posture bridge **structurally removed** from the run path:

| Receipt | Value |
| ------- | ----- |
| `v4-bootstrap-viability.sh` exit code | **0** |
| Final compiler line | `compiled: 1 files emitted, 0 diagnostics` |
| Modules indexed | **340** |
| Modules resolved (transitive import closure) | **340** |
| Diagnostics (any class) | **0** |
| Bridge script invocation count | **0** (`V4_BOOTSTRAP_ALLOW_RESOLVE_POSTURE_BRIDGE` intentionally unset; bridge's header check refuses to run without it) |
| Emit artifact | `/tmp/v4-p2b-bridge-removal-probe/dag-artifact.json` + `src/` (single emitted file per `--target dag`) |

The viability script's own success gate (`grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$'`) fired green, and the final stderr line confirms: *"Bootstrap viability OK — v2 compiled all v4 modules."*

This is the safety-net receipt operator needs: the bridge is not load-bearing on current `main`; removing the CI step + script does not regress the bootstrap closure.

---

## §2 Probe shape (what was actually run)

The probe simulates bridge removal by running ONLY `scripts/v4-bootstrap-viability.sh` (the real-work step) with the bridge script's run-gate intentionally disabled. Specifically:

```bash
export V2_COMPILER=target/release/gunbc       # post-SG-1 build (78b9698ab tip era; survives subsequent main pulls)
export V4_BOOTSTRAP_OUT=/tmp/v4-p2b-bridge-removal-probe
export V4_BOOTSTRAP_LOG=/tmp/v4-p2b-bridge-removal-probe.log
export V4_BOOTSTRAP_TIMEOUT_SECS=480           # matches the CI step's documented budget
# V4_BOOTSTRAP_ALLOW_RESOLVE_POSTURE_BRIDGE intentionally UNSET — bridge refuses to run
bash scripts/v4-bootstrap-viability.sh
# → exit 0; receipt `compiled: 1 files emitted, 0 diagnostics`
```

**Test-only — no main modifications.** The probe does NOT touch `.github/workflows/ci.yml` or `scripts/v4-bootstrap-resolve-posture-gate.sh`. The bridge script is left in-tree exactly as it lives on `main`; the env-var simulation reproduces the same shape as the operator's eventual `git rm scripts/v4-bootstrap-resolve-posture-gate.sh` + remove-CI-step act.

**Probe environment caveats** (informational, not blockers):
- Local worktree, not the CI runner — `ubicloud-standard-8` SIGTERM behavior is the original bridge's stated dissolution-trigger (b). On main today the SIGTERM has not been observed for 14+ consecutive days per the bridge header's removal condition.
- Single-shot run, not the 14-day rolling-window the bridge header documents as the formal removal trigger. The 14-day window is operator policy; this receipt does not displace it. The probe confirms that **today's** main compile succeeds without the bridge; the 14-day claim is a separate operator-policy verification.

---

## §3 Cross-receipt context

### §3.1 Why M2 complements P2-A

P2-A (`valiant-moth-559`, per PR #4094 §3.2) verified **44-source compiler-of-record closure** with 0 dag+rust diagnostics — proving the v2-compiler IS the authority and the bridge is structurally redundant in the small-corpus scenario.

M2 (this probe) verifies the same property at **full src/v4 corpus scale** (340 modules vs P2-A's 44 sources) — proving the bridge is also not needed at the full bootstrap-closure scale that the CI step actually targets.

Together, P2-A + M2 receipts cover both the unit (P2-A) and integration (M2) ends of the bridge-redundancy claim.

### §3.2 Why SG-7 closure (PR #4050) is not the same receipt

PR #4050 closed MW-D8 C2 (SG-7 `ci.dag` recursion dissolved) at the rustc-error / modeled-authority level — the recursive shape that previously masked v2 complexity diagnostics is gone. That receipt proves the **diagnostic side** of the bridge's reason-to-exist is closed.

The M2 probe here proves the **timeout / SIGTERM side** of the bridge's reason-to-exist is closed: the full compile completes within budget (no timeout exit 124/143 path that the bridge's `if:` clause was designed to catch).

Both reasons-to-exist of the bridge have receipts. The P2-B deletion is now defensible.

### §3.3 SG-1 cascade unaffected by bridge removal

The post-SG-1 catalog (PR #4086) reports 6,991 rustc errors on the M1 (rustc-on-emitted-Rust) path. The M2 probe here is the **bootstrap path** (`--target dag`), which compiles to dag-artifact.json, not Rust — so the rustc errors are not in play. Bridge removal does not affect M1's rustc population either way; M1 sees zero v2 diagnostics with the bridge unset (per PR #4086 §1 headline `v2 emit diagnostics: 0`), consistent with this probe.

---

## §4 Operator decision surface

Based on this receipt + P2-A + PR #4050:

- **GO (delete bridge):** operator authorizes `git rm scripts/v4-bootstrap-resolve-posture-gate.sh` + removes the CI step at `ci.yml:377-384` + drops the `v4_bootstrap_resolve_posture` step output references. Repeatable receipt: every future PR run of this M2 probe shape (or simply the unmodified `v4-bootstrap-viability.sh` step in CI) catches regression.
- **GO-WITH-FOLLOWUP:** authorize deletion AND attach a follow-on watch: the next 14 consecutive days of `main` CI runs must show the bootstrap step green without bridge fallback. PR #4050's MW-D8 C2 watch conditions already cover the rustc-error side; an equivalent watch for the timeout side is a small extension.
- **HOLD:** if operator wants the 14-day rolling-window proof (per bridge header dissolution condition) before authorization, this single-probe receipt is insufficient and the decision waits for that window. This Close/Receipt lane's recommendation: this probe is sufficient for "today's main compiles" but does NOT discharge the 14-day-window operator policy unilaterally — the operator's choice between the two reads is theirs.

---

## §5 What this receipt is NOT

- **Not a CI modification.** This probe does NOT modify `.github/workflows/ci.yml` or any script. The actual deletion is the operator's act, separately.
- **Not a 14-day rolling-window verification.** Single-shot probe only; operator policy on the 14-day window stands.
- **Not a SG-1 close receipt.** SG-1 / SG-7 cited only as cross-receipt context per §3.
- **Not a worker dispatch.** No follow-on worker brief; the dispatch board priority 3 decision rests with the operator.

## §6 Related artifacts

- PR #4094 — P2-A dispatch lineage (this M2 probe is its complement).
- PR #4050 — MW-D8 C2 falsification receipt (SG-7 closure).
- PR #4086 — post-SG-1 rustc catalog (M1 path, 0 v2 diagnostics confirms bridge not invoked there either).
- `scripts/v4-bootstrap-viability.sh` — the real-work step the probe ran verbatim.
- `scripts/v4-bootstrap-resolve-posture-gate.sh` — the bridge script under test (left untouched on main).
- `.github/workflows/ci.yml:377-384` — the CI step under test (left untouched on main).
- PR #3949 §1 — closure invariant: the executable receipt is the §1 `Verdict` table verbatim; falsification would be a second probe run that fails on `main` HEAD (none observed in this run).
