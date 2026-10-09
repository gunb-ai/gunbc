# Workspace administration API qualification — 2026-09-28

This is documentation research, not a live organization readback. No credentials were
provisioned and no Google-side setting was changed.

| Control | Supported observation found | Mutation | Evidence |
|---|---|---|---|
| Auth Platform web client | Console readback | Manual console action | [Create clients](https://support.google.com/cloud/answer/15549257) |
| Internal audience | Console readback | Manual console action | [Configure consent](https://developers.google.com/workspace/guides/configure-oauth-consent) |
| This client's Specific Google data access | Console readback | Manual console action | [Control app access](https://support.google.com/a/answer/7281227) |
| External directory sharing | Cloud Identity Policy API, `directory.external_directory_sharing` | API mutation unsupported; manual console action | [Supported settings](https://docs.cloud.google.com/identity/docs/concepts/supported-policy-api-settings), [admin control](https://knowledge.workspace.google.com/admin/users/let-third-party-apps-access-directory-data) |
| Profile photo editing policy | Console readback | Manual console action | [Profile editing policy](https://knowledge.workspace.google.com/admin/users/allow-directory-users-to-change-their-profile-and-photo) |

“No supported read found” is a dated qualification of these documented surfaces, not a claim
that Google can never add an API. The Policy API inventory lists several global API-controls
settings, but those are not the specific-client access control in row 3. The Directory API's
user/photo operations are not an administration API for row 5. IAM `oauthClients` applies to
[Workforce Identity Federation](https://docs.cloud.google.com/iam/docs/workforce-oauth-app),
not the Auth Platform client in row 1.

Row 4 values are `REQUESTER_BASIC_PROFILE_ONLY` and `ORGANIZATION_DIRECTORY_DATA`.
Unspecified/unknown values are not evidence of either. The minimum desired value is the first;
organization discovery requires an explicit rationale.

A future supported observer uses [policies.list/get](https://docs.cloud.google.com/identity/docs/how-to/list-get-policies)
with the separately approved `cloud-identity.policies.readonly` scope. Complete pagination and
[effective-policy reduction](https://docs.cloud.google.com/identity/docs/concepts/policy-api-concepts)
are required; a single raw policy is not necessarily the customer's effective policy. Unread
OU/group overrides remain a refusal. Honor the documented quota and do not derive identity
from the older SYSTEM-policy name/sort-order convention.

The implementation's receipts bind customer/project/client/OU, typed setting, source,
evidence reference, observation time and expiry. A receipt producer remains responsible for
authenticating and durably retaining its evidence. The assessor is pure and cannot independently
verify a screenshot or API response referenced by a caller. Fixture receipts are not deployment
receipts. Matching scope/redirect lists are compared as sets, not presentation order.

Validation: the focused Workspace claim suite uses a byte-identical import closure under the
existing 6 GiB limit. It covers missing observations, mismatch, tenant isolation, reader admission,
expiry/future timestamps, supported API setting identity, unresolved effective coverage and
scope-order equivalence versus extra privilege. See the branch's validation receipt for results.
