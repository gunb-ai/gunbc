# Approve — iOS approval app

SwiftUI (iOS 17+) realizing `gunbc.auth.approval_device_wire` (the byte contract) and `gunbc.auth.approval_device_redemption` (the server's admission) with
`extdeps.apple.{secure_enclave, app_attest, apns}`. `Protocol.swift`, `Wire.swift` and
`ProtocolVectorTests.swift` are GENERATED: emitted through the Swift grammar rows (`v2.extdeps.languages.swift.rows`) from Swift syntax
trees (`gunbc.approve_ios_swift_protocol`, `gunbc.approve_ios_swift_wire`,
`gunbc.approve_ios_swift_protocol_vector_tests`) and drift-gated, so edit the tree, never the file; the rest
is still hand-authored. Both standings are recorded in `gunbc.approve_ios_app`
(`approve_ios_transcribed_swift_frontier`, `approve_ios_retained_swift_rows`). The app invents no wire fact:
every constant, field order and route is a transcription of the `.dag`, and the two byte builders are
held to the `.dag` folds by `dag/test/fixture/approval_device_redemption/vectors.json`, which the
`.dag` witness emits and `ApproveTests` reads.

## Files

| file | what it is |
|---|---|
| `project.yml` | XcodeGen spec, GENERATED from `gunbc.approve_ios_project` (drift-gated; never hand-edit); no `.xcodeproj` is committed |
| `Config/Team.xcconfig` | operator-filled: `DEVELOPMENT_TEAM`, `APNS_ENVIRONMENT`, `APP_ATTEST_ENVIRONMENT`. The server host is not here: `project.yml` carries it, projected from `gunbc.auth.approval_broker_endpoint` |
| `Approve/Protocol.swift` | GENERATED (`gunbc.approve_ios_swift_protocol`). `framed` (the injective `<n>:<field>,` rendering), `enrolment_transcript`, `device_redemption_signing_input`, `device_read_client_data`, the protocol records |
| `Approve/Wire.swift` | GENERATED (`gunbc.approve_ios_swift_wire`). Mirror of the modeled HTTP wire: emitter, strict readers, routes, headers; `URLSession` over the tailnet |
| `Approve/DeviceKeys.swift` | enclave decision key (`[.privateKeyUsage, .biometryCurrentSet]` at creation), App Attest, and the one keychain item shape |
| `Approve/EnrolmentState.swift` | the durable enrolment state: signedOut → enrolled → revoked (phases from `gunbc.approve_ios_enrolment_machine`; a key the enclave refuses reads as signedOut; revoked leaves only by signing in with a new key) |
| `Approve/OperationErrors.swift` | GENERATED (`gunbc.approve_ios_swift_operation_errors` over `gunbc.approve_ios_operation_errors`). One current error per operation: the operation's next success removes it, a new failure replaces it, and leaving the phases it is relevant in drops it. Never a log |
| `ApproveTests/OperationErrorsTests.swift` | GENERATED (same module). The error lifetime against `error_scenarios` / `expected_relevance`, the literal ruling the `.dag` witness also runs |
| `Approve/AppState.swift` | transitions, APNs token forwarding, authenticated reads, redemption; records every operation's outcome into `OperationErrors` |
| `Approve/ApproveApp.swift` | entry point, APNs token delivery, push wakes the list |
| `Approve/Views.swift` | Enrol, Revoked, and the deck: one full-screen request card at a time (swipe right approves, left denies, two large buttons as the non-gesture equivalent) |
| `ApproveTests/ProtocolVectorTests.swift` | GENERATED (`gunbc.approve_ios_swift_protocol_vector_tests`; its surface-name set is built from the fixture's `surface_vectors`). Byte builders and envelopes vs the emitted vectors; absence of the fixture FAILS |
| `Approve/Assets.xcassets/AppIcon.appiconset/` | the generated 1024 PNG (script projection of the mark.svg geometry) |

## What the operator builds on the MacBook

This repository's containers have no Xcode; nothing here has been compiled. Expect small compile
fixes on first build.

1. `brew install xcodegen`
2. Fill `Config/Team.xcconfig` (team id). Keep the team id out of commits.
3. `cd apps/approve-ios && xcodegen generate && open Approve.xcodeproj`
4. In Signing & Capabilities confirm Push Notifications and App Attest are on the App ID (the
   entitlements file declares `aps-environment` and `com.apple.developer.devicecheck.appattest-environment`).
5. Run `ApproveTests` (⌘U). The vector tests are the only equivalence evidence with the `.dag`
   folds, and they are executed nowhere else (see `gunbc.approve_ios_app`). Every section the suite reads is emitted by
   `approval_device_redemption_fixtures` into `vectors.json`; a test whose section is absent fails
   loudly on its own and never skips.
6. Run on a PHYSICAL device: App Attest `isSupported` is false in the Simulator and the app refuses
   enrolment there by design (`app_attest_unavailable_in_simulator_note`).
7. For a distribution (TestFlight) build set `APNS_ENVIRONMENT = production` and
   `APP_ATTEST_ENVIRONMENT = production` (`testflight_is_production_note`).

## Flow

- **Sign in**: refuse unless App Attest is supported → create the enclave key and the App Attest key → fix the
  enrolment transcript → Google sign-in (GoogleSignIn-iOS ≥ 9.0.0, the iOS client only, no `serverClientID`)
  with `nonce = googleEnrolmentNonce(transcript)`, so the ID token is bound to these keys → attest with
  `clientDataHash = SHA256(transcript)` → register for APNs → one `POST /approve/device/enrol` carrying the ID
  token and attestation. An unanswered POST (transport failure) is resent once, byte-identical; the server's
  admission is idempotent. A refused, failed or abandoned sign-in leaves the phone signed out and the answer
  is "sign in again": there is no state to recover, so nothing is persisted before the POST and there is no
  readback or retry. Enrolled is persisted only once the server has answered.
- **Push token**: registered on every enrolled launch; one serialized convergence loop forwards the
  LATEST delivered token via `PUT /approve/device/push` and records success only if that token is
  still the desired one and the enrolment is still the one the update was made for; a failure whose
  token or enrolment has since changed continues at once, a failure for the exact current
  (enrolment, token) pair stops, and the next token delivery or enrolment adoption starts a fresh loop. `adopt()` (enrolment) re-registers for APNs and runs the loop, so
  a token delivered before the enrolment answered is forwarded.
- **Wire.swift** is a mechanical mirror of `gunbc.auth.approval_device_wire` "The HTTP wire": an ordered emitter matching `serialize_json` byte for byte, strict readers (unknown/missing/empty member and unadmitted `kind` refuse at their path), path segments spelled `id-` + padded base64url of the identity's UTF-8 bytes (total, injective, never refused) and transported verbatim. `ApproveTests` round-trips every request envelope in `vectors.json` through decoder → encoder and matches the bytes. **Standing of that check:** authored and un-enrolled — no lane here compiles Swift, so it executes only when the operator runs it (`gunbc.approve_ios_app` `approve_ios_vector_check_unenrolled_frontier` names what would enrol it).
- **Deck** (operator direction 2026-10-24, "one giant button"): ONE request card at a time, full screen, the first of the server's answer; swipe right to approve, left to deny, or the large Approve/Deny buttons; when nothing needs attention the screen says so and shows nothing else. Deciding re-lists, so the decided request leaves and the next card appears. The set is exactly what the server answers, replaced on every read: the requests that still need the operator, i.e. pending and not yet expired (`gunbc.auth.approval_decision_store` `pending_escalations`; a decided or expired request is not returned, the expiry boundary being the one redemption refuses at). `GET /approve/device/pending` on open, pull-to-refresh, and on push, authenticated by an App Attest assertion over `device_read_client_data` (headers `X-Approval-Assertion`, `X-Approval-Enrollment`, `X-Approval-Requested-At`; 60 s skew). The push carries only
  `notification_id`; nothing from it is displayed as the request.
- **Card**: `GET /approve/device/requests/<escalation_id>` returns the stored request byte for byte
  plus a stateless `RedemptionChallenge` and both verbs' capabilities. The screen strictly decodes
  `requester`, `purpose`, `destructive` and `expires_at` from the stored request (any missing member
  refuses the presentation and disables both buttons; typed `ApprovalTarget` and the access lifetime
  arrive with `approval_target_frontier`), shows the challenge expiry only as "refresh required
  after", and keeps the exact signed text in a disclosure; then Approve/Deny signs with the chosen verb's capability:
  `device_redemption_signing_input` with the enclave key (Face ID is the enclave's own prompt — there
  is no `LAContext` pre-check), generates an App Attest assertion over the same bytes, and
  `POST /approve/device/redeem`s the `SignedRedemption`. The server's `{outcome, message}` is rendered as is.
