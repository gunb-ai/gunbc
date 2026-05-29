#!/usr/bin/env bash
set -x
export DASHBOARD_URL=https://100.65.94.55:3737
exec > /home/briansrls/.worktrees/clever-ant-219/.tmp-dashboard-msg-out.txt 2>&1

echo "=== which dashboard-message ==="
which dashboard-message || true
command -v dashboard-message || true
type dashboard-message || true

echo "=== find dashboard-message ==="
find /usr/local/bin /usr/bin /home/briansrls -name 'dashboard-message' 2>/dev/null | head -20

echo "=== PATH ==="
echo "$PATH"

echo "=== run command ==="
if command -v dashboard-message >/dev/null 2>&1; then
  dashboard-message send --to parent --body "$(cat <<'EOF'
Release Jun1 §1 audit complete (clever-ant-219).

**Current org state (API, 2026-05-29):** gunb-ai reports plan.name=free (not Enterprise). GraphQL samlIdentityProvider=null. two_factor_requirement_enabled=false. 1 org member. GET /orgs/gunb-ai/audit-log → 404. GET /orgs/gunb-ai/ip-allow-list → 404. credential-authorizations=[].

**Codebase (gunbc + gunb-ai/ctrl):** No callers of GitHub /orgs/{org}/audit-log. Session dashboard (ctrl/scripts/session-dashboard) uses internal logEvent audit trail only; pools_billing.mjs uses /orgs/{org}/settings/billing/actions (Teams-compatible). server.mjs mentions SAML only in PAT setup help text. No enterprise: keys in .github/workflows/*.yml; CI uses runs-on [self-hosted, linux, arm64] labels.

**Blocked on admin:org token:** org runner-groups, org runners inventory, org installations list — operator should confirm in GitHub UI.

**Verdict:** No Enterprise-only blockers found in repo/automation; org settings already look non-Enterprise. Safe to proceed with Teams migration checklist in RELEASE_TODO §1 migration steps; operator UI pass still needed for installed Apps + runner groups.
EOF
)"
  echo "exit=$?"
else
  echo "dashboard-message not found, trying dashboard-ops"
  /home/briansrls/.worktrees/clever-ant-219/scripts/session-dashboard/dashboard-ops 2>&1 || true
fi

echo "=== done ==="
