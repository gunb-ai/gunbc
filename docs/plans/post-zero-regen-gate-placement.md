# Post-zero regen gate placement — proposal draft

**Status: DRAFT** for operator + crisp-bear-170 decision (loyal-owl-307, 2026-07-07).

## What changed since the first draft

| Item | Was (two-job split #6350/#6352) | Now (#6355 landed) |
| --- | --- | --- |
| `regen_divergence` ratchet | REQUIRED `ci_regen_ratchet` job, monotone-shrink baseline | **REMOVED** — operator judged overcomplicated; audit in `regen-divergence-31-vs-32-reconciliation.md` drove the decision |
| `ci_regen` spawn-width | Crash at plan setup | **FIXED** — `gunbc_ci_regen_floor_plan_spawn_width` (#6355) |
| CI jobs | `ci`, `ci_regen_ratchet`, `ci_regen`, `rust_tests` | `ci`, `ci_regen`, `rust_tests` |
| Required red on main | `ci_regen_ratchet` tightness (baseline vs actual) | **Gone** — no interim ratchet |

**Ratchet retirement is DONE.** The remaining live question is only: when self-host
divergence hits **0** (lively-raven #6357 close-out), should **`RegenVerifyGate`**
(+ `SelfHostStalenessGate`) return to the **required** `ci` surface?

## Premise at divergence 0

| Today (pre-#6357) | After close-out (divergence 0) |
| --- | --- |
| `ci_regen` honest **RED** — `RegenVerifyGate` + `SelfHostStalenessGate` on non-required job | Same gates go **GREEN** — self-host fixed-point is a grounded invariant |
| Branch protection: required `ci` + `rust_tests`; `ci_regen` excluded | Premise "permanently red regen" inverts — protection model must be re-derived |

## Recommendation (draft)

### 1. Promote regen-verify to the REQUIRED path (separate follow-up PR)

**NOT in the lively-raven close-out PR** (#6357 stays minimal: seed accept-fresh only).

Move `RegenVerifyGate` + `SelfHostStalenessGate` enrollment from `GithubActionsCiRegenJob`
→ `GithubActionsCiJob` in `dag/gunbc/commit_workflow.dag`, after #6357 lands divergence 0
on main.

Rationale: at 0, failing regen is a **merge blocker**, not an audit sidebar. Keeping
`ci_regen` non-required while `RegenVerifyGate` is green recreates the escape hatch the
two-job split guarded against — without the ratchet, promotion is the only hard gate.

### 2. Branch-protection change (operator action)

| Check | Today (post-#6355) | Proposed at divergence 0 |
| --- | --- | --- |
| `ci` | required | required (add regen-verify gates to this job's floor) |
| `ci_regen` | excluded (honest red/ green visibility) | **fold into `ci`** or promote to required, then delete duplicate job |
| `rust_tests` | required | required (unchanged) |

### 3. End state for `ci_regen` job

**Preferred:** single `ci` job runs compile-clean floor **and** `regen_verify_gate_passes`
+ `self_host_realized_comparison_staleness_gate_holds`; delete standalone `ci_regen` job
(§2 dissolve-target: one regen execution, one workflow).

**Interim:** promote `ci_regen` to required for one release cycle as visibility buffer;
accept 2× regen cost until dissolve.

### 4. Affordability

`regen_stage0 --verify` ≈ 65s wall on pre-close-out main. Resource profile
(`ci_floor_measurement.dag`): ~1.5 GiB peak per host-compiler spawn — affordable inside
existing `ci` at `spawn_width=1`. Not proposing `route_a_emit_fresh_cargo_green_test`
(~3–5 min) into required path without separate operator ruling.

### 5. Keep-it-zero mechanics (post-promotion)

Without the ratchet, **`RegenVerifyGate` byte-identical** is the sole enforcement:
emit-changing PRs must include in-PR `regen_stage0` seed update or the required gate
refuses. HAND_MAINTAINED files outside `GENERATED_STAGE0_FILES` remain a separate
dissolution track until flipped GENERATED.

## Sequencing

1. **#6357** (lively-raven): seed accept-fresh → divergence 0. No gate promotion.
2. **This doc:** operator/crisp-bear sign-off.
3. **Follow-up PR:** `commit_workflow.dag` + `ci_workflow.dag` enrollment surgery.
4. Operator updates branch-protection required checks.

## Open questions

1. Fold `ci_regen` into `ci` immediately at 0, or promote `ci_regen` required first?
2. **`main.rs` HAND pin:** does close-out flip it GENERATED, or does a separate gate cover hand drift?
3. Return `RegenVerifyGate` to `GitPrePushHook` surface at 0?
