# Approval device enrolment as a handshake

Status: design (node://adhoc-19a8196b-f67, operator scope 2026-09-30). Authority once landed:
`gunbc.auth.approval_enrolment_handshake`. This page states the design; the model is the fact.

## The defect this replaces

Today one POST both consumes the enrolment code and mints the enrolment
(`gunbc.auth.approval_device_redemption` `commit_enrolment`: slot generation 1 `code` → 2
`enrolled`). A phone whose POST gets no decodable answer holds `SubmissionUnknown`, and the only way
out is a readback authenticated by the *enrolled* App Attest key. For an enrolment that never
existed, that read can only refuse, and a bare refusal is not proof of absence. So the phone can't
safely discard its keys, can't reuse its code, and has no typed exit. On 2026-09-30 an operator's
phone sat in exactly that state after a front-door 404 (the device routes were unrouted), with an
expired code the server had never spent.

The operator's requirement: **enrolment must never be a death sentence.** Every phone state has a
typed exit, and the phone may always start over safely.

## The one invariant

> **An enrolment can decide only if it is ACTIVE, and it becomes ACTIVE only on a signature by its
> decision key over a server challenge issued for that enrolment.**

Everything else follows. A phone that discards its prepared keys can't produce that signature, so
nothing it discarded before sending the ACK can ever become ACTIVE. The worst case is a PENDING
record nobody holds, and PENDING lapses at its deadline on its own. **So a handshake ends by expiry
or by either party walking away, and the phone never has to tell the server it is leaving**
(operator refinement, 2026-09-30). Cancel is a local discard of the prepared keys. The one state
where a discard can leave something behind is after the ACK may have landed (see
*ConfirmUnknown*). What it leaves is an ACTIVE enrolment whose decision key no longer exists
anywhere, so it can never sign a decision.

## The server record: one CAS slot per code

The code stays the identifier (one thing typed, one slot). The slot's generations form a line, and
each write is a compare-and-set expecting the generation before it
(`std.durable_compare_and_set`), so two writers can never both win:

| gen | record | written by | reached from |
|---|---|---|---|
| 1 | `IssuedCode { code, login, issued_at, expires_at }` | code issue verb (operator, via fleet-converge) | absent |
| 2 | `Pending { enrollment_id, login, platform, decision_key, attestation, submitted_at, pending_deadline, confirm_nonce }` | SUBMIT (SYN) | 1, code live |
| 3 | `Active { …pending, confirmed_at }` | CONFIRM (ACK) | 2, before `pending_deadline` |
| 4 | `Revoked { …active, revoked_at, reason }` | operator revocation | 3 Active |

**The PENDING deadline is read, not swept.** A `Pending` observed at or after `pending_deadline`
*reads* as `Lapsed`, and CONFIRM refuses it. Nothing is ever written to end a handshake. This is the same pattern
the code already uses for its own window (`expires_at` compared at observation by
`utc_instant_before`). No background writer is needed and none can be forgotten.

*Stated divergence (DESIGN §3b, leasing):* the brief said "std.temporal_effect lease". Its
`HeldLease` models an observed running process, `std.durable_exclusive_hold` has no deadline, and
`std.scoped_authorization` `OperatorGrant.expires_at` is an operator's grant, not a party's
provisional hold. So the deadline follows the enrolment code's existing `expires_at` pattern in
this module. If a deadline-bearing hold is ever added to std, this record should inhabit it.

## The legs

| leg | route | authenticated by | effect |
|---|---|---|---|
| SYN | `POST /approve/device/enrol` `{code, platform, decision_key, evidence, push}` | the code (bearer secret) + App Attest attestation over the transcript | gen 1→2 `Pending`; answers SYN-ACK |
| SYN-ACK | (the SYN's answer) `{enrollment_id, confirm_challenge {nonce, pending_deadline}}` | — | — |
| ACK | `POST /approve/device/confirm` `{enrollment_id, signature}` | **decision key** signature over `confirm_signing_input(enrollment_id, nonce)` (one Face ID at enrolment; operator-confirmed 2026-09-30) | gen 2→3 `Active`; answers `{standing: active}` |
| STATUS | `POST /approve/device/enrolment-status` `{code, requested_at}` | possession of the code | none; answers the closed status |

**SYN is idempotent.** A SYN against a `Pending` slot whose stored decision key and attest key id
equal the request's answers the *same* SYN-ACK (same `enrollment_id`, same nonce, same deadline)
and writes nothing. A SYN against a slot already at gen 2+ with a *different* key refuses with
`code_already_used`. That's the only way a spent code refuses, and it can't be reached by the phone
that spent it.

**STATUS is authenticated by the code.** Every state has a code, and the phone holds it from the
moment it's typed. So one authentication covers every state, including states where the server
holds no key for the phone (gen 1, absent). The answer is a closed set:

```
NeverSubmitted     gen 1, code live           → phone may SYN (again)
CodeExpired        gen 1, code past expires_at → phone discards, needs a new code
CodeNotIssued      slot absent                 → phone discards, needs a new code
Pending { enrollment_id, confirm_challenge }  → phone confirms (ACK), or walks away
Lapsed                                         → gen 2 past its deadline; phone discards
Active { enrollment_id, decision_key_matches: Bool }  → phone adopts iff its key matches
Revoked                                        → terminal, phone discards
```

`Active` answers whether the stored decision key equals the one in the request's proof. That's
why STATUS also carries the phone's decision key: a code holder learns only whether *its own* key
is the active one.

## Phone states and their exits (every one typed)

| phone state | how it's left |
|---|---|
| `Unenrolled` | code typed → `Prepared` |
| `Prepared` (keys made, nothing sent) | SYN → `SubmissionUnknown`; Cancel → discard → `Unenrolled` (nothing left the phone) |
| `SubmissionUnknown` (SYN may have landed) | SYN-ACK → `Pending`; STATUS: `NeverSubmitted` → resend SYN; `Pending` → `Pending`; `CodeExpired`/`CodeNotIssued`/`Lapsed` → discard → `Unenrolled`; `Active`+match → `Enrolled` (impossible without an ACK, answered for completeness) |
| `Pending` (SYN-ACK held) | ACK → `ConfirmUnknown`; Cancel → local discard → `Unenrolled` (the PENDING lapses unaided) |
| `ConfirmUnknown` (ACK may have landed) | `{standing: active}` → `Enrolled`; STATUS: `Active`+match → `Enrolled`; `Pending` → re-ACK; `Lapsed` → discard → `Unenrolled`. Cancel → local discard. If the ACK had landed, the ACTIVE record is left with a decision key that no longer exists: inert for deciding, and removed by the operator's existing revocation. The phone shows that consequence before discarding here. |
| `Enrolled` | revocation, key invalidation (existing) |

Cancel is the operator's existing local Cancel button, and it stays local: no route, no signature, no server write. The phone never has to revoke or abandon anything (operator refinement, 2026-09-30); a server-confirmed withdraw route is explicitly not the design.

## RED per transition (a lost answer at each leg)

Each row is a claim in the PR. It names the state the phone holds, what was lost, what the server
holds, and the typed exit the phone must take. The RED is the mutation that must make it fail.

| # | lost | server holds | phone exit | RED |
|---|---|---|---|---|
| R1 | SYN never arrived | gen 1 live | STATUS→`NeverSubmitted`→resend | STATUS on gen 1 answers anything but `NeverSubmitted` |
| R2 | SYN-ACK lost | gen 2 `Pending` | resend SYN → same SYN-ACK, one record | a repeated identical SYN writes or answers a different nonce |
| R3 | SYN with another key | gen 2 | `code_already_used` | a second key's SYN is admitted or overwrites |
| R4 | ACK never arrived | gen 2 | STATUS→`Pending`→re-ACK | a PENDING record decides a redemption |
| R5 | ACK answer lost | gen 3 `Active` | STATUS→`Active`+match→`Enrolled` | STATUS answers `Active` without the key match, or matches a foreign key |
| R6 | ACK after deadline | gen 2 past deadline | refuse → STATUS `Lapsed` → discard | a lapsed PENDING confirms |
| R7 | phone walked away (Cancel) | gen 2 | deadline lapses → reads `Lapsed`; a new code enrols afresh | a lapsed PENDING reads `Pending`, or blocks a new code |
| R8 | two ACKs (retry after a lost answer) | gen 2 or 3 | the second answers `active` idempotently | a repeated ACK writes a second record or refuses a confirmed enrolment |
| R9 | code expired unspent | gen 1 past expiry | STATUS→`CodeExpired`→discard | an expired code reads `NeverSubmitted` |
| R10 | ACK with a key other than the pending one | gen 2 | refuse | a signature by any key but the pending decision key activates |

## Migration

This replaces generations 2 and 3 of the enrolment slot (`enrolled`, `revoked` → `pending`,
`active`, `revoked`). It's a replacement migration cut at the root (DESIGN §3): the
store on srv1 holds **no** enrolled record today (only two expired gen-1 codes), so there's no
persisted population to carry. The redemption path (`device_redemption_admission`) reads `Active`
where it read `EnrolmentCodeConsumed`, and every enrolment-only route that took the enrolled
standing takes `Active` and nothing else.
