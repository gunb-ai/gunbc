# Google Workspace identity/profile convergence (Cut 4)

Owner directive 2026-09-27: the Google-side configuration the tracker depends on is modeled
and converged — "we need to model this via convergence"; the owner performs only the literal
initial clicks. External review 2026-09-27 (Workspace onboarding) supplies the acceptance
boundaries; this plan is its convergence-shaped reduction.

## Law

1. Every Google-side setting the product depends on is a DECLARED DESIRED FACT in the model.
2. Where Google exposes a supported READ, the setting is OBSERVED through it and the
   observation is what standing claims consume.
3. Where Google exposes a supported WRITE, the setting is CONVERGED through the fleet's
   approval-gated credential path (org_admin_credential, gcp_secret_access, the PR #12421
   IAM pattern).
4. Where Google exposes NEITHER, the setting is a MANUAL-ADMINISTRATION OBLIGATION with a
   readback receipt — never a fabricated convergence API, never "established because login
   worked once".
5. Authentication and profile acquisition are SEPARATE operations. A tenant selects ONE
   profile source by policy: UserInfo (ordinary OIDC) or the authorized Directory source
   (managed Workspace org). Never a runtime fallback chain.
6. The principal stays the only identity. Names/photos are mutable profile projections with
   explicit source, provenance, freshness, and display-audience policy.

## Surfaces and their convergence class

| # | Google-side fact | Read API | Write API | Class |
|---|---|---|---|---|
| 1 | OAuth client registration (client id/secret) for the tracker | no supported read found in the reviewed Auth Platform documentation (2026-09-28) | none | MANUAL obligation (owner's initial clicks); the secret rides Secret Manager via the existing gcp_secret_access path; the deployment config is already modeled (oidc_deployment_config) |
| 2 | Auth Platform audience = Internal (project inside the gunb.ai org) | console readback (documentation reviewed 2026-09-28) | console-only | MANUAL obligation + readback receipt |
| 3 | API controls: THIS client's app access — Specific Google data (openid/email/profile scopes), top OU | console readback (documentation reviewed 2026-09-28) | console-only | MANUAL obligation + readback receipt |
| 4 | Directory sharing: authenticated-user basic profile fields (minimum); org-data sharing ONLY if assignee directory discovery requires it — the reason is recorded either way | Cloud Identity Policy API: `directory.external_directory_sharing` | no supported mutation | MANUAL change obligation + effective Policy API readback |
| 5 | Profile-editing policy (photo) org-wide | console readback (documentation reviewed 2026-09-28) | console-only | MANUAL obligation (already set 2026-09-27) + readback receipt |
| 6 | Managed user photos (assignee avatars, header) | Admin SDK users.photos.get (read-only scope admin.directory.user.readonly) | n/a (read-only source) | OBSERVED: periodic observation → profile projection refresh |
| 7 | Directory people discovery (assignee picker population) | People API people.listDirectoryPeople (directory.readonly) | n/a | OBSERVED: same projection, pagination + deletion semantics preserved (incremental sync lags writes; a completed sync is not a post-change readback) |

The authorized backend identity for 6/7 (service account + domain-wide delegation, if chosen)
is a SEPARATELY APPROVED integration — never silently acquired, never required for basic
login.

## Model shape

- extdeps.google.workspace grows from a citation anchor into the external model of these
  admin controls and directory/profile operations — Google's documented semantics only, no
  gunbc policy inside.
- extdeps.auth.oidc keeps claims + UserInfo protocol + the sub-equality admission (a UserInfo
  result whose sub differs from the ID token's sub is rejected whole).
- gunbc profile projection: revisioned, source-bound, with a declared refresh policy; a new
  login MAY update it but is not its only producer; identity history is stable across profile
  change.
- Photo vocabulary preserved end-to-end: PhotoReturned / ProviderDefaultPhotoReturned /
  PhotoNotReturned / PhotoReadRefused / PhotoNotObserved. PhotoNotReturned never becomes
  AccountHasNoPhoto; omission is never labeled PolicyDenied without a policy observation.
- Display audience is an access-policy term: self-header vs assignee-picker vs comment-thread
  viewers are different audiences; admitted images serve through a controlled application
  route with bounded caching — no tokens in image URLs, no open URL-fetch proxy.

## Acceptance (behavioral matrix, from the external review)

Admin-managed non-public photo retrievable via the authorized Directory path or a located
access refusal (never "make it public"); ID token omitting picture leaves authentication
valid with no photo-absence inference; UserInfo sub mismatch rejects the profile; API
denial/timeout is a distinct refusal, never stored as absence; provider default-photo marker
preserved; photo lost between serialization and readback fails that boundary's propagation
control; photo change/removal updates under the declared refresh policy; revoked profile
access follows the retention policy; child-OU/group overrides reported as effective policy;
second tenant isolated; email change keeps principal history; avatar absence never blocks
issue/approval authorization. Tested with at least one NON-ADMIN managed account.

## Sequencing

C4.1 — Workspace external model + convergence declarations for rows 1–5 (manual-obligation
machinery with readback receipts; VERIFY each row's API standing before classing it).
C4.2 — Profile projection: revisioned store, refresh policy, the five-valued photo
vocabulary, session mint becomes one producer.
C4.3 — UserInfo profile operation (sub-equality admission) for OIDC tenants.
C4.4 — Directory source (rows 6–7) with the separately-approved backend identity; assignee
picker consumes the projection.
C4.5 — Display-audience policy + controlled image route; the header, picker, and comments
share the one projection.
C4.6 — Receipt completion: the structured propagation trace (payload member standing →
parsed claim → verdict → projection → persisted readback → render selection → image-load
standing), plus the positive end-to-end control.

## Done in this lane already (2026-09-27)

- Login claims receipt: post-verdict writes only, typed publication standing
  (LoginClaimsReceiptPublished/PublicationRefused), observed-boundary comment.
- Session profile projection carries name + picture as optional members end-to-end; the
  header prefers the display name and paints the photo with the initials fallback.
- The direct evidence discipline held the diagnosis: three consecutive receipts established
  provider-side omission before any account-level claim was made.

## C4.1 API qualification and receipt boundary (2026-09-28)

The source and limits of this classification are recorded in
[the C4.1 research receipt](receipts/google-workspace-admin-api-2026-09-28.md).
`extdeps.google.workspace` owns the external subjects, values and API standing.
`gunbc.auth.google_workspace_admin_convergence` declares the five obligations and feeds
subject-bound, time-bounded readback into `std.goal_assessment`. It does not mutate Google
configuration or alter login. Rows 6–7 remain separate integrations.

The receipt producer must authenticate its source and persist the referenced evidence before
calling the assessor. These types and tests establish the assessment contract, not a deployed
observer or proof that the live organization matches it. No live admin receipt is supplied by
this change. Missing observations, unread overrides, wrong tenant/client/OU and expired
receipts refuse assessment. The photo obligation is scoped to the declared OU: an organization-wide
claim additionally needs resolved child-OU/group coverage; a top-OU receipt alone is insufficient.
