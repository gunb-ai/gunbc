# Org-admin credential acquisition, custody, and read path

Status: modeled; no credential has been obtained, stored, or read. The storage-class and freshness
ruling is from the parent relay of the 2026-08-30 pricing-study memo. Upstream capability and token
facts are owned by `extdeps.github.org_admin_auth` and cite GitHub's documentation there.

## Decision

The terminal credential is a GitHub App installation access token. The durable secret is the app
private key; the actuator mints a narrowly scoped installation token for each invocation and lets
that token expire after one hour. This makes the broadly useful bearer token short-lived and
programmatically renewable.

The acceptable interim is a human-created fine-grained personal access token whose resource owner
is the organization and whose permissions are organization `Self-hosted runners: write` and
`Administration: write`. Store it with an explicit expiry and rotate-before date. A classic PAT
with `admin:org` works but is broader than the fine-grained PAT. An OAuth App token also works after
interactive web or device authorization, but is user-bound and long-lived by default; it adds an
application and refresh lifecycle without improving this interim. Neither is selected.

## Acquisition census

| Shape | Converge access | Acquisition | Lifetime and rotation | Revocation |
|---|---|---|---|---|
| Fine-grained PAT | `organization_self_hosted_runners` read/write covers runner groups, selected repositories, and organization runner registration tokens; `organization_administration` read/write covers the organization Actions policy | Human creates it in GitHub's web UI; organization approval may be required | Configurable expiry; rotate before expiry and revalidate after replacement | Token settings, organization policy/approval, user removal, leak detection |
| Classic PAT | `admin:org` covers the same organization endpoints (and more) | Human creates it in GitHub's web UI | User-selected expiry; GitHub may revoke on expiry, exposure, or prolonged non-use | User credential settings, OAuth authorization API, organization/enterprise incident controls |
| GitHub App installation token | Installation with organization `Self-hosted runners: write` and `Administration: write` covers the converge surface | App registration/key and installation require an owner decision; each access token is then minted programmatically from the app key | Installation token expires after one hour; mint per invocation, never rotate a stored installation token | Suspend/uninstall the installation, reduce permissions/repositories, or revoke/rotate the app private key |
| OAuth App user token | `admin:org` covers the organization endpoints while the authorizing user remains an org admin | Interactive authorization-code web flow or device flow | Long-lived by default; optional eight-hour access token plus refresh token | User revokes authorization, app owner revokes token/authorization, org/enterprise policy, user loses admin standing |

The GitHub endpoint authorities are `extdeps.github.org_admin_auth.runner_groups_citation`,
`actions_permissions_citation`, `installation_token_citation`, and `oauth_flow_citation`. The model
does not bind any of them to `gh`: CLI, REST, and SDK are interchangeable realizations.

## Custody and read path

`gunbc.auth.org_admin_credential.SecretMaterial` is a custody descriptor, not secret bytes. It
records one store, an allow-list of readers, rotation, credential standing, and the validation
receipt. Actual bytes remain behind the existing
`gunbc.auth.materialized_secret.with_materialized_secret` bracket; this plan does not create a
parallel reader.

For CI, use a GitHub Actions organization secret restricted to the repository and workflow lane
that performs org observation/apply. A repository secret is an acceptable narrower placement when
only one repository runs the actuator. For an operator session, use a named local credential-store
profile and expose it to only the validation or converge process. Do not put a token in argv, a
worktree file, committed configuration, a receipt, or this plan.

Bytes-present and working-credential are intentionally different facts. `SecretMaterial.store`
answers custody. `CredentialStanding` answers whether executed evidence supports use:

- `CredentialCurrent` names the evidence digest, exact receipt producer, and validity horizon.
- `CredentialProvisionallyUsable` is explicit but is not admitted to org convergence.
- `CredentialStale` carries either time expiry or an earlier event invalidation: revocation,
  organization-owner change, or scope edit.
- `CredentialMissing` admits nothing.

`admit_org_admin_credential` refuses absent material, unauthorized readers, incomplete capability,
non-current/stale standing, missing validation, and receipt/credential identity mismatch. No arm
turns absence into a skip.

## Acquisition genealogy and runtime dependency

`CredentialAcquisitionGenealogy` recursively records how the credential became obtainable. Every
constructible chain has one `HumanCredentialBootstrap` root: a GitHub organization owner or app
manager uses a token/settings, OAuth-consent/device, or app-registration web surface and records the
bootstrap receipt and its staleness horizon. `CredentialAcquiredFrom` records each later derivation
and whether it can renew without another human interaction. A stale human root creates a scheduled
operator renewal obligation; a programmatic link can re-execute automatically.

This history is deliberately not the runtime dependency graph. For example, the browser session
that registered and installed a GitHub App is acquisition history, while the app private key is the
parent that must remain current whenever an installation token is minted. That live relationship is
`CredentialRuntimeDependency`. `runtime_dependency_standing` propagates missing, revoked, expired,
owner-changed, and scope-changed standing from the real parent to every descendant. Fleet converge
admits the material only when both the genealogy is structurally complete and the whole live chain
is current.

## Fleet-converge consumer

The existing `gunbc.fleet_converge_workflow.fleet_converge_job` owns the first consumer; there is no
parallel workflow. Its `org_actions_observe` mode invokes
`gunbc.fleet.org_actions_converge.org_actions_converge_wet`. The phase refuses a missing secret,
executes the read-only runner-groups validation call, strictly decodes the response, reads each
group's selected repositories through `extdeps.github.org_actions.CliOrgRunnerGroups`, and evaluates
`gunbc.fleet.org_actions_standing.org_actions_settings_diff`. Any missing/stale credential, probe
failure, malformed response, repository-read failure, or desired-state divergence is a failing
typed outcome. It never silently skips and it has no write/fix-divergence arm.

The validation receipt is uploaded as a workflow artifact only after the probe and joined reads
succeed. The artifact contains identity, endpoint, organization, and time—not a token, header,
response body, or token fingerprint. Until the operator creates `GUNBC_ORG_ADMIN_TOKEN`, dispatching
the mode witnesses the intended missing-credential refusal. Once the secret exists, the same route
performs the live read without a code-path switch.

## Interim human handoff

1. In GitHub's fine-grained token UI, the operator selects `gunb-ai` as resource owner, grants
   organization `Self-hosted runners: write` and `Administration: write`, and chooses a bounded
   expiry. No repository content permission is needed for the org-settings probe itself.
2. The operator pastes the value directly into the chosen store: the restricted Actions secret for
   CI, or the named local credential profile for a supervised session. The value is never pasted
   into an issue, PR, shell history, command argument, or receipt.
3. Before any converge depends on it, the validation lane materializes it as an environment-bound
   secret and executes the read-only `GET /orgs/{org}/actions/runner-groups` probe (the
   `ListOrganizationRunnerGroups` endpoint). A 2xx response proves organization subject and
   self-hosted-runner read scope without changing the subject. The later actuator must separately
   require the write grants declared by the credential shape; a successful read is not fabricated
   proof of write access.
4. The probe writes an `OrgAdminCredentialValidationReceipt` containing only material identity,
   credential kind, endpoint, organization, and validation time. `CredentialCurrent` binds its
   evidence digest and exact producer to the validity horizon. It contains no token, response body,
   headers, or recoverable token fingerprint.
5. Expiry, revocation, owner change, scope change, a missing receipt, or a receipt outside its
   horizon changes standing away from `CredentialCurrent`; observe and apply both refuse before
   mutation. Rotation repeats steps 2–4 and then deletes/revokes the prior material according to
   the selected store's deletion semantics.

## Terminal migration

Register and install a dedicated GitHub App with only organization `Self-hosted runners: write` and
`Administration: write`. Store its private key using the same SecretMaterial custody contract and
existing materialization bracket. Mint an installation token per invocation, record its returned
expiry, run the same read-only probe, and discard it after use. Once that path has an executing
positive receipt and absent/revoked negative controls, delete the interim PAT row and revoke the
PAT; do not retain both as fallback authorities.
