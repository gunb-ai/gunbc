# Approve — iOS approval app

Hand-authored SwiftUI (iOS 17+) realizing `gunbc.auth.approval_device_wire` (the byte contract) and `gunbc.auth.approval_device_redemption` (the server's admission) with
`extdeps.apple.{secure_enclave, app_attest, apns}`. Recorded as seed-retained Swift in
`gunbc.approve_ios_app` (`approve_ios_hand_authored_swift_frontier`). The app invents no wire fact:
every constant, field order and route is a transcription of the `.dag`, and the two byte builders are
held to the `.dag` folds by `dag/test/fixture/approval_device_redemption/vectors.json`, which the
`.dag` witness emits and `ApproveTests` reads.

## Files

| file | what it is |
|---|---|
| `project.yml` | XcodeGen spec; no `.xcodeproj` is committed |
| `Config/Team.xcconfig` | operator-filled: `DEVELOPMENT_TEAM`, `APPROVE_SERVER_HOST` (srv1 tailnet host), `APNS_ENVIRONMENT`, `APP_ATTEST_ENVIRONMENT` |
| `Approve/Protocol.swift` | `framed` (the injective `<n>:<field>,` rendering), `enrolment_transcript`, `device_redemption_signing_input`, `device_read_client_data`, the protocol records |
| `Approve/Wire.swift` | mirror of the modeled HTTP wire: emitter, strict readers, routes, headers; `URLSession` over the tailnet |
| `Approve/DeviceKeys.swift` | enclave decision key (`[.privateKeyUsage, .biometryCurrentSet]` at creation), App Attest, and the one keychain item shape |
| `Approve/EnrolmentState.swift` | the durable enrolment state machine: Unenrolled → Prepared → SubmissionUnknown → Enrolled → KeyInvalidated / Revoked, persisted before every remote call |
| `Approve/AppState.swift` | transitions, APNs token forwarding, authenticated reads, redemption |
| `Approve/ApproveApp.swift` | entry point, APNs token delivery, push wakes the list |
| `Approve/Views.swift` | Enrol, Inbox, Detail |
| `ApproveTests/ProtocolVectorTests.swift` | byte builders and envelopes vs the emitted vectors; absence of the fixture FAILS |
| `Approve/Assets.xcassets/AppIcon.appiconset/` | the generated 1024 PNG (script projection of the mark.svg geometry) |

## What the operator builds on the MacBook

This repository's containers have no Xcode; nothing here has been compiled. Expect small compile
fixes on first build.

1. `brew install xcodegen`
2. Fill `Config/Team.xcconfig` (team id, `APPROVE_SERVER_HOST`). Keep the team id out of commits.
3. `cd apps/approve-ios && xcodegen generate && open Approve.xcodeproj`
4. In Signing & Capabilities confirm Push Notifications and App Attest are on the App ID (the
   entitlements file declares `aps-environment` and `com.apple.developer.devicecheck.appattest-environment`).
5. Run `ApproveTests` (⌘U). `testRedemptionSigningInputMatchesEveryVector` and
   `testEnrolmentTranscriptMatchesEveryVector` are the equivalence evidence with the `.dag` folds.
6. Run on a PHYSICAL device: App Attest `isSupported` is false in the Simulator and the app refuses
   enrolment there by design (`app_attest_unavailable_in_simulator_note`).
7. For a distribution (TestFlight) build set `APNS_ENVIRONMENT = production` and
   `APP_ATTEST_ENVIRONMENT = production` (`testflight_is_production_note`).

## Flow

- **Enrol**: type the one-time code printed by the enrol command the operator runs over SSH on srv1 (it is the challenge identifier; it is not shown on the dashboard) → refuse unless App Attest is
  supported → create the enclave key and the App Attest key, fix the transcript, PERSIST as `Prepared` →
  attest with `clientDataHash = SHA256(enrolment_transcript)` (kept, so a retry re-sends the same
  attestation over the same hash) → register for APNs → persist `SubmissionUnknown` → `POST /approve/device/enrol`.
  From the moment the POST may have left the device the durable state says so; transport/decode
  failure, a crash, a failed persist of Enrolled or a refusal after a retry all leave it standing,
  and only an authenticated readback (active → Enrolled, revoked → terminal, 404 → back to Prepared)
  resolves it. Keys, attestation, code and transcript are retained throughout. The attest key id is
  recorded as enrolled only after the server verified it.
- **Push token**: registered on every enrolled launch; one serialized convergence loop forwards the
  LATEST delivered token via `PUT /approve/device/push` and records success only if that token is
  still the desired one; it also runs after enrolment or readback adopts Enrolled, so a token
  buffered during `SubmissionUnknown` is forwarded.
- **Readback**: `GET /approve/device/enrollments/<enrollment_id>` under the App Attest key just attested, after a lost enrolment answer; the id is `enrollment_id_for_code` over the persisted code, so nothing is asked for.
- **Wire.swift** is a mechanical mirror of `gunbc.auth.approval_device_wire` "The HTTP wire": an ordered emitter matching `serialize_json` byte for byte, strict readers (unknown/missing/empty member and unadmitted `kind` refuse at their path), path segments spelled `id-` + padded base64url of the identity's UTF-8 bytes (total, injective, never refused) and transported verbatim. `ApproveTests` round-trips every request envelope in `vectors.json` through decoder → encoder and matches the bytes.
- **Inbox**: `GET /approve/device/pending` on open, pull-to-refresh, and on push, authenticated by an App Attest assertion over `device_read_client_data` (headers `X-Approval-Assertion`, `X-Approval-Enrollment`, `X-Approval-Requested-At`; 60 s skew). The push carries only
  `notification_id`; nothing from it is displayed as the request.
- **Detail**: `GET /approve/device/requests/<escalation_id>` returns the stored request byte for byte
  plus a stateless `RedemptionChallenge` and both verbs' capabilities. The screen strictly decodes
  `requester`, `purpose`, `destructive` and `expires_at` from the stored request (any missing member
  refuses the presentation and disables both buttons; typed `ApprovalTarget` and the access lifetime
  arrive with `approval_target_frontier`), shows the challenge expiry only as "refresh required
  after", and keeps the exact signed text in a disclosure; then Approve/Deny signs with the chosen verb's capability:
  `device_redemption_signing_input` with the enclave key (Face ID is the enclave's own prompt — there
  is no `LAContext` pre-check), generates an App Attest assertion over the same bytes, and
  `POST /approve/device/redeem`s the `SignedRedemption`. The server's `{outcome, message}` is rendered as is.
