# Human identity and authorization (owner ruling 2026-09-25)

> **Network placement decides who can reach an endpoint. Google OpenID Connect
> establishes which human is present. Application policy decides what that
> principal may do.**

Tailnet no longer authenticates people for the tracker or approvals app. It
remains transport: development network, TLS/reverse-proxy, a reachability
boundary. A `Tailscale-User-Login` header is optional transport/audit
metadata; it never mints an application principal and never authorizes a
mutation. (The tailnet model itself flags the fragility: the header is
unsigned and trustworthy only when the proxy is provably the sole route.)

## Three separate concepts

```text
Transport:        Direct HTTPS | Tailnet proxy | local development
Authentication:   Google Workspace principal
Authorization:    assign | comment | approve | administer | merge
```

## One qualified-principal authority

```text
PrincipalRef { authority, namespace, subject }

principal:google-oidc/accounts/<google-sub>     -- authority https://accounts.google.com, subject = ID-token sub
principal:gunbc/workflows/fabric                -- authority gunbc, namespace workflows
```

Email, display name, picture are `PrincipalProfile` facts with a
profile_revision — never identity inputs. `sub` is the durable identifier
(email can change; Workspace membership comes from the `hd` claim, not email
parsing). Consequences: email changes don't mint people; email reuse doesn't
inherit authority; avatar failures can't weaken auth; one human = one
principal across tracker, web approvals, native approvals.

The known contradiction at a3fd5eb is BANNED going forward:
`principal_from_email` setting `subject = email` keeps the selector as
durable identity. Comments must not copy that shape.

## Authority placement

Buganizer extdeps keeps `User { email }` (the interface's visible user).
gunbc's principal/profile/auth authorities live OUTSIDE extdeps:

```text
extdeps.google.openid_connect     -- Google's protocol + claim vocabulary
extdeps.google.workspace          -- Workspace org/domain interface facts
gunbc.identity.principal          -- PrincipalRef, workflow principals, profile refs
gunbc.auth.google_workspace_login -- gunb.ai policy over Google OIDC
gunbc.auth.session                -- application session identity + lifetime
gunbc.auth.request_authentication -- raw HTTP request → authenticated request
```

(The existing oauth2.Google.Refresh is API-token refresh, not a login model.)

## Authentication vs authorization (Google's own split)

Login requests ONLY `openid email profile`. No Gmail/Contacts/Directory/Drive
scopes for login. Profile enrichment for the assignee picker is a separately
authorized directory producer feeding the internal principal-profile cache.
Avatar order: internal profile → Workspace directory → cached → initials;
absent/unread never blocks auth, assignment, comments, or approval.

## Web flow (tracker + web approvals)

Authorization-code flow with PKCE (implicit is deprecated-insecure). Routes:
`GET /auth/login`, `GET /auth/google/callback`, `POST /auth/logout`,
`GET /auth/session`. Steps: state+nonce+PKCE verifier persisted as a
short-lived transaction → redirect → verify state → exchange code → verify ID
token (signature/JWKS, iss, aud, exp, nonce, email_verified, hd == gunb.ai) →
mint opaque `AuthenticatedSession` → redirect only to an admitted relative
return path.

Browser gets an opaque cookie: Secure, HttpOnly, SameSite=Lax, bounded
lifetime, rotated on authentication. State-changing forms additionally require
application CSRF proof (login state ≠ CSRF token).

Redirect URIs: exact-match owned domains (`tracker[-dev].gunb.ai`,
`approvals[-dev].gunb.ai`); dev names may resolve only inside the tailnet.
Separate OAuth clients per app × environment × platform (tracker web
dev/prod, approvals web dev/prod, approvals iOS/Android); one admitted
audience roster, no silent cross-app token acceptance.

## Approvals: three proofs, never one

```text
Human authentication   — Google OIDC principal
Decision authority     — one-time approval capability + approval policy
Device/app integrity   — App Attest / Android equivalent / enrolled device
```

GET confirmation page never decides; POST decision is the only mutation,
requiring: authenticated session + CSRF proof + one-time capability +
expected current decision state + approval authorization (an explicit
approver roster/role — `hd == gunb.ai` alone is org membership, not approval
authority; first production cut = an exact stable-principal roster, even of
one). Native apps: platform Google Sign-In → ID token to the backend
validated against the native client audience; the mutation requires Google
principal + device integrity + capability.

## Event-log migration

`roadmap-event/v3`: `author: PrincipalRef`; `Claimed { assignee:
PrincipalRef, return_to: PrincipalRef }`; `CommentCreated { ..., author:
PrincipalRef }`. Historical v1/v2 email strings decode as
`LegacyEmailPrincipal { email, authentication_unestablished }` — readable,
never authorizing, NEVER auto-matched to a current `sub`. New v3 writes
require an authenticated principal; missing/invalid session refuses.

Assignee picker stays email-centric in display, but suggestion rows submit
the stable PrincipalRef. Typed email → directory lookup → exactly one
admitted principal or typed refusal (none / multiple / external domain /
unverified). Fabric: selector `fabric@gunb.ai`, stored
`principal:gunbc/workflows/fabric`.

## Comments and planning behind the seam

Order: stable principal model → Google OIDC authentication → authenticated
comment append → Ask Fabric / planning work requests. The comment CARRIER may
be built against PrincipalRef immediately; its production POST stays blocked
until Google session auth is wired. The one-principal one-work-request design
stands: Issue (durable outcome) / Comment (narrative or explicit request
origin) / IssueWorkRequest (the exact work requested now) /
DeliverableContract (code candidate | issue-revision candidate |
investigation report | review verdict). No planner role, no Planning type.

## The canonical work graph

```text
A. Human identity and authorization
   A1 PrincipalRef authority
   A2 Google OIDC external model
   A3 server session + CSRF boundary
   A4 tracker assignment migration to event v3
   A5 durable comments
   A6 issue-revision/planning work contracts
   A7 web approval migration
   A8 native approval-client identity
   A9 directory/avatar enrichment
B. Stage0 driver iteration (identity split, immutable cache + private
   overlay, A/B qualification, freeze, incremental closure, fixed point)
C. Group B serving (cache proof → prefill bound → FitProved intent →
   persistent convergence/readback)
D. SCM fanin (conversion → generation/CAS save → structural locator →
   integration-head advance)
E. Workflow reliability (population, continuation, alignment, capture,
   launch lease, actuation split)

A1→A2→A3; A3→A4→A5→A6; A3→A7→A8; A1→A9.
B independent of A. C independent of A,B. D independent of A,B,C.
E constrains broad autonomous dispatch.
```

The stage0 (B) acceptance contract is preserved verbatim from the ruling:
never hand-edit the tracked stage0 mirror; scratch driver rendered by
interpreting emit_source_root_eval_driver_main_rs; one exact materialized
library identity; Library/DriverA/DriverB mutation receipts; immutable
content-addressed library + private mutable overlay (overlay output is never
authority until copied under the full content key); freeze the rendered main
digest after qualification (instrumentation lives at the modeled runtime
boundary, not in 05_emit_rust.dag); landing = regen-round receipt + stage0
partition rebuild decision; final acceptance = A′/B′ parallel on two hosts,
full two-build fixed point paid once, generated driver digest == fast-loop
qualified digest.

## Acceptance controls

- A forged Tailscale-User-Login establishes no application principal.
- Direct HTTPS and tailnet-proxied requests authenticate to the same
  principal.
- Wrong iss/aud/exp/nonce/hd or unverified email refuses.
- Same sub + changed email = same principal; same email + different sub ≠
  same principal.
- Comment events store PrincipalRef, not email.
- Historical email-only events readable but unverified.
- Valid capability + wrong principal refuses; GET never decides; POST
  requires capability + principal + authorization + CSRF.
- Avatar/directory failure renders fallback, never affects authorization.
- Service/machine writers cannot use a human Google session as workload
  identity.
- Tailnet loss changes reachability, not stored-event identity semantics.

## Immediate sequencing (owner-set)

1. Amend the comments lane: author is PrincipalRef, not email/Tailnet.
2. HOLD deployment of the assignment POST at a3fd5eb until the v3 migration
   lands there.
3. Build A1–A3 (principal authority, OIDC model, session/CSRF).
4. Cut tracker assignment + comments over (A4, A5).
5. Cut the web approval broker over (A7), then native audiences (A8).
6. Demote tailnet identity to transport observation on user-facing routes.
7. B, C, D continue independently.
