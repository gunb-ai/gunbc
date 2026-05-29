# GitHub Enterprise → Teams — pre-downgrade audit — 2026-05-29

**Session**: clever-ant-219 · **Authority**: RELEASE_TODO.md §1  
**Operator alignment**: no SAML/SSO, audit-log API, IP allowlists, or Enterprise runner groups in use; one GitHub App believed Teams-compatible.

---

## Verdict: **GO** (no remediation)

No Enterprise-only dependency found in org settings or in-repo automation. Plan downgrade remains operator-only and is not on the Jun 1 critical path.

---

## Verification matrix

| Check | Method | Result |
|-------|--------|--------|
| SAML/SSO | GraphQL `organization.samlIdentityProvider`; REST `credential-authorizations` | **Clear** — provider null; authorizations empty |
| Audit log API | `GET /orgs/gunb-ai/audit-log` | **404** (not available on current plan; not in use) |
| Code: `/orgs/{org}/audit-log` | `rg` gunbc + shallow clone `gunb-ai/ctrl` `scripts/session-dashboard` | **Clear** — no GitHub audit-log API callers; dashboard `audit-log` strings are internal `logEvent` only |
| IP allowlist | `GET /orgs/gunb-ai/ip-allow-list` | **404** (none configured) |
| Org 2FA requirement | `GET /orgs/gunb-ai` | **Clear** — `two_factor_requirement_enabled: false` (Teams-compatible) |
| Runner groups | Operator attestation + CI uses label `runs-on: [self-hosted, linux, arm64]` | **Clear** — no Enterprise runner-group API dependency in workflows |
| `enterprise:` in Actions | `rg` `.github/workflows/` | **Clear** — no matches |
| Installed GitHub Apps | Operator attestation (one app, Teams-compatible) | **Accepted** — token lacked `admin:org` for org installations list; no code dependency surfaced |
| Current plan | `GET /orgs/gunb-ai` → `plan.name` | Reports **`free`** (2026-05-29); not blocking audit conclusion |

---

## Notes

- Session dashboard billing (`gunb-ai/ctrl` `pools_billing.mjs`) uses `/orgs/{org}/settings/billing/actions` — available on Teams.
- Self-hosted runners on srv1/srv2 use standard org-level registration; runner **groups** exist on Teams if needed later.
- **Residual (operator UI, non-blocking)**: confirm the single installed GitHub App in org Settings → GitHub Apps before billing downgrade.

---

## Follow-up (operator-only, post-audit)

RELEASE_TODO.md §1 "Migration steps" (contact support, quota check, post-migration CI smoke) — not gated on this audit.
