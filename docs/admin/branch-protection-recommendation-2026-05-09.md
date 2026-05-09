# Branch Protection Recommendation — main — 2026-05-09

**Author**: deep-wolf-155 (PM)
**Operator authorization request**: Brian directive 2026-05-09 (~21:50Z): *"Seems like something snuck past CI? we need to make CI a hard block for merge i guess"*
**Authority scope**: PM-tier audit + recommendation. Branch protection requires repository-admin authority (operator-tier); PM cannot apply.

---

## §0. The trigger

PR #2441 merged at `2026-05-09T21:46:56Z` with `ci` job **FAILURE**. The R4-carve dissolution discipline ratchet correctly caught a violation in `docs/briefs/r3-evaluator-tc3-d4-eval-step-producer-worker.md:121`, but branch-protection didn't enforce CI as a hard merge block.

Direct evidence:
```
$ gh pr view 2441 --json statusCheckRollup --jq '.statusCheckRollup[] | {name, conclusion}'
{"conclusion":"SUCCESS","name":"fmt"}
{"conclusion":"FAILURE","name":"ci"}        # ← FAILED
{"conclusion":"SUCCESS","name":"v3"}
{"conclusion":"SKIPPED","name":"self_host_ratchet"}
```

The PR merged anyway. **Conclusion**: `ci` job is not in branch-protection's "required status checks" list.

---

## §1. CI ratchet inventory (10 active scripts)

All ratchets currently running under the `ci` workflow job:

| Ratchet script | Purpose | Self-test passing? |
|---|---|---|
| `check-banked-dissolutions.sh` | Banked-dissolution discipline | (verify) |
| `check-compiler-std-ratchet.sh` | compiler/std consolidation | (verify) |
| `check-fabrication-sentinels.sh` | P0-C fabrication sentinel | (verify) |
| `check-manager-brief-authority.sh` | P2 single-authority on Mgr briefs | (verify) |
| `check-pr-sg0-net-shrink-discipline.sh` | SG-0 PR-window net-shrink discipline | ✓ |
| `check-r4-carve-dissolution-discipline.sh` | R4-carve dissolution + Class P recognition (NEW 2026-05-09) | ✓ |
| `check-release-doc-authority.sh` | P2 release-doc authority | (verify) |
| `check-rust-toolchain-single-authority.sh` | P2 Rust toolchain single-authority | (verify) |
| `check-stage0-freshness.sh` | Stage0 bootstrap freshness | (verify) |
| `check-test-timeout.sh` | Per-test 2s timeout ratchet | (verify) |

All run within the `ci` workflow job. If `ci` is hard-blocked, all ratchets effectively become required.

---

## §2. CI workflow top-level job structure

`.github/workflows/ci.yml` defines 4 top-level jobs visible at PR level:

| Job | Purpose | Continue-on-error? | Recommended for required-checks? |
|---|---|---|---|
| `fmt` | `cargo fmt --all --check` | no | **YES** — required |
| `ci` | All 10 discipline ratchets + bootstrap freshness + L-7 substrate accessor reconstruction | no | **YES** — required |
| `v3` | v3 compiler tests + clippy + heavy compute | no | **YES** — required |
| `self_host_ratchet` | DB-8 determinism + self-host fixed point | **YES** (intentional staging until Lane 1e graduates) | NO — keep informational |

---

## §3. Recommended branch-protection config

Apply via `gh api` (requires admin authority on `gunb-ai/gunbc`):

```bash
gh api -X PUT /repos/gunb-ai/gunbc/branches/main/protection \
  --input - <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["fmt", "ci", "v3"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "required_linear_history": false,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": false,
  "lock_branch": false,
  "allow_fork_syncing": true
}
EOF
```

**Field rationale**:

- `required_status_checks.contexts: ["fmt", "ci", "v3"]` — the 3 substantive jobs. `self_host_ratchet` excluded (intentional `continue-on-error` staging).
- `required_status_checks.strict: true` — branch must be up-to-date with main before merge. Prevents stale-base merges.
- `enforce_admins: false` — admins can bypass for emergency fixes (e.g., main-RED hotfixes). Set `true` if you want admins also subject.
- `required_pull_request_reviews: null` — does NOT require PR reviews (matches current squash-merge cadence; api-review quorum is sufficient per session merge policy). Set to `{"required_approving_review_count": 1}` if you want review approval too.
- `restrictions: null` — no user/team restrictions on push.
- `required_linear_history: false` — allows squash + merge commits both. Set `true` if you want only squash (matches session merge policy).
- `allow_force_pushes: false` — prevents force-push to main.
- `allow_deletions: false` — prevents branch deletion.

**Stricter alternative** (if you want session-policy-aligned defaults):

```bash
gh api -X PUT /repos/gunb-ai/gunbc/branches/main/protection \
  --input - <<'EOF'
{
  "required_status_checks": {"strict": true, "contexts": ["fmt", "ci", "v3"]},
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": true
}
EOF
```

Adds:
- `required_linear_history: true` — only squash-merges (matches "squash-merge" merge policy in session-dashboard preamble)
- `required_conversation_resolution: true` — open review threads must be resolved before merge

---

## §4. Verification (post-apply)

After applying, verify with:

```bash
gh api /repos/gunb-ai/gunbc/branches/main/protection --jq '{checks: .required_status_checks.contexts, strict: .required_status_checks.strict, force: .allow_force_pushes, linear: .required_linear_history}'
```

Expected output:
```json
{"checks":["fmt","ci","v3"],"strict":true,"force":false,"linear":true}
```

Test by creating a small PR with deliberate `ci` failure (e.g., a fmt violation in a non-Rust file would skip both `fmt` and `ci`; instead make a small change that would fail SG-0 net-shrink check). Verify merge button is disabled / GitHub displays "Required status check 'ci' is failing."

---

## §5. Operational note: existing PRs

PRs in flight today (PR #2399, #2437, etc.) already have CI runs from before branch protection landed. After applying:
- Future PRs will be blocked on CI failure
- PRs with stale CI may need re-runs (push empty commit or re-trigger)
- No retroactive enforcement on already-merged PRs

---

## §6. Defense-in-depth complete picture

After applying:

| Layer | Mechanism | Hard-block? |
|---|---|---|
| Pre-PR-window: SG-0 net-shrink discipline | `check-pr-sg0-net-shrink-discipline.sh` (in CI) | YES (after this) |
| Pre-PR-window: R4-carve drift | `check-r4-carve-dissolution-discipline.sh` (in CI) | YES (after this) |
| Pre-PR-window: 8 other ratchets | run in `ci` job | YES (after this) |
| Format check | `fmt` job | YES (after this) |
| Heavy compute / v3 tests / clippy | `v3` job | YES (after this) |
| Review-process patch (mandate-drift discipline) | review-bot soft signal | NO (advisory) |
| Self-host ratchet | `self_host_ratchet` | NO (intentional staging) |

**Net**: 10 ratchets + fmt + heavy compute become hard merge blockers. Drift this session (R4-carve + #2441 ci=FAILURE) would be prevented at the merge layer.

---

## §7. Open question

Should `enforce_admins: true`? — i.e., should you (operator) also be subject to branch-protection on `main`?

**PM recommendation**: `false` initially. You may need emergency fixes (main-RED hotfixes; today's drift sweep was operator-direct-merge per "directly to main" pattern). After a few weeks of stable CI hard-block + no emergency override needs, flip to `true`.

---

## §8. Provenance

- Trigger: Brian directive 2026-05-09 ~21:50Z
- Direct evidence: PR #2441 statusCheckRollup showing `ci=FAILURE` + merged status
- CI workflow analysis: `.github/workflows/ci.yml` job structure
- Ratchet inventory: `ls scripts/check-*.sh`
- This recommendation is PM-tier authoring; operator applies via `gh api` CLI command above (admin authority required)
