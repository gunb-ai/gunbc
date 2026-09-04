# Wet-evidence convergence: the anti-collapse count, taken before the extraction

Two authorities answer "did the wet evidence hold" for one fold. `v2.workflow.local_repo_wet_terminal`
(landed via #9903) joins a co-resident execution's terminals against a scheduled roster;
`v2.workflow.floor_wet_route` (proposed in #9725) validates a transported receipt envelope against
a subject digest pair. `v2.workflow.floor_changed_witness` imports one of them and, under the union,
would import both — the meaning fork DESIGN §3 forbids.

This document exists for one reason: **a careless unification silently merges typed refusals**, and
the tell is that the extracted join grows one generic "binding mismatch" cause where six real
remedies used to live. A detail string is not a typed cause and cannot be joined on. So the union of
both refusal vocabularies is counted HERE, BEFORE the extraction, and the integrated join must be
able to emit at least as many.

## The count: 18 distinct causes, deduplicated by REMEDY

Where the two modules already agree on a remedy the row is listed once, naming both spellings.

| # | cause | local_repo_wet_terminal | floor_wet_route |
|---|---|---|---|
| 1 | scheduled identity has no terminal | `TerminalMissing` | inside `ReceiptRosterInexact { detail }` |
| 2 | two terminals claim one identity | `TerminalDuplicated { occurrences }` | Rust reader only, untyped in `.dag` |
| 3 | terminal for an identity nobody scheduled | `TerminalUnscheduled` | inside `ReceiptRosterInexact { detail }` |
| 4 | executed a different entry than scheduled | `TerminalForeignEntry { observed, required }` | inside `ReceiptContractMismatch { detail }` |
| 5 | executed a different function than scheduled | `TerminalForeignFunction { observed, required }` | inside `ReceiptContractMismatch { detail }` |
| 6 | observed verdict is not the expected one | `TerminalVerdictNotExpected` | `wet_receipt_unexpected_red_identities` |
| 7a | co-resident: ran against a different prepared subject | `TerminalForeignCandidate { observed, required }` | — (meaningless for this route) |
| 7b | transported: semantic subject digest differs | — | `ReceiptSubjectMismatch { axis, receipt, computed }` |
| 7c | transported: executor contract digest differs | — | `ReceiptExecutorSnapshotDifferent { receipt, computed }` |
| 8 | receipt aged past its staleness budget | — (meaningless for this route) | `ReceiptExpired { age_secs }` |
| 9 | schema version is not the one this floor reads | — | inside `ReceiptContractMismatch { detail }` |
| 10 | executed_at exceeds published_at | — | inside `ReceiptContractMismatch { detail }` |
| 11 | published_at exceeds tree commit beyond declared skew | — | inside `ReceiptContractMismatch { detail }` |
| 12 | outcome wire outside the closed vocabulary | — | Rust reader only, untyped in `.dag` |
| 13 | envelope unreadable | — | Rust reader only |
| 14 | projection missing or orphaned from its authority | — | Rust reader only |
| 15 | projection is not the canonical projection (hand edit) | — | Rust reader only |
| 16 | enrolled expected-red observed passing | — | `wet_receipt_now_passing_identities` |
| 17 | no subject verdict reached | — | `wet_receipt_no_verdict_identities` |
| 18 | completed past its line — a COST, not a defect | `LocalRepoWetCompletedOverBudget` | `wet_receipt_cost_debt_identities` |

**Post-extraction obligation: the integrated join emits ≥ 18, and rows 7a/7b/7c stay three.**
Collapsing them into one "binding mismatch" is the exact failure this count exists to catch.

## What the count found, which was not the expected direction

The collapse is in **floor_wet_route**, not in main's module. Rows 1, 3, 4, 5, 9, 10 and 11 are
typed causes on one side and `detail: String` payloads on the other — seven remedies behind two
string-carrying arms. Rows 2, 12, 13, 14 and 15 exist only in the Rust reader and have no `.dag`
spelling at all.

So the extraction WIDENS the transported route to main's grain. It does not narrow main's join to
meet it, and any version of this work that reduces main's seven causes has failed rather than
converged.

## The vocabulary is not ours to mint: `floor_terminal_ledger` already owns it

`v2.workflow.floor_terminal_ledger` already declares the raw attempt terminal
(`ClaimAttemptTerminal`), observed-verdict versus unreadable-observation (`ClaimObservation`),
expectation (`ClaimExpectation`), the row (`ClaimTerminalRow`), a binding (`LedgerBinding`), and
thirteen derived dispositions. Neither wet module imports it. `floor_changed_witness` — the
conflicted consumer — does.

Its `claim_disposition` already maps a completed-past-limit terminal by expectation:
`ExpectedToHold → PassedOverBudget`, `ExpectedRed → KnownRedNowPassing`. That is row 18 of the count
above, derived independently in `local_repo_wet_terminal`, derived independently again in
`floor_wet_route`, and owned all along by a module none of the three imported.

**Three independent arrivals at one distinction is the proven coincidence DESIGN §6 asks for before
unifying vocabulary** — a better argument for the shared terminal than any reasoning about elegance.
So the terminal half of this convergence is REUSE, not extraction, and minting a third terminal type
between the two would be the same fork wearing a new name.

## What is genuinely new work

`LedgerBinding` carries a repository snapshot and a prepared subject: the co-resident case. The
transported case additionally needs the executor-contract digest, because a receipt produced by a
different executor over the same tree is a different evidential fact. That is one typed widening of
an existing type — two arms feeding one validation — and not a new authority:

- co-resident prepared-subject binding
- transported receipt-subject binding carrying semantic subject and executor contract

Freshness and executor age are meaningless for an execution running inside the floor, so they do not
move onto the local route; the local route's prepared-subject equality is meaningless for a receipt
produced by another process, so it does not move onto the transported one. One interface and one
exact join, two execution realizations, two route-owned roster policies.

## Scope boundary

This is an extraction. It changes no behavior of `local_repo_wet_terminal`: its join, its seven
refusals, its deliberately single-armed expectation and its invocation wall come out intact and land
back on the same population. Anything found wrong in that module during the lift is NAMED here and
routed separately, never repaired under cover of a mechanical extraction.

## What the migration step changed, recorded because the paragraph above no longer describes the head

The scope boundary above is stated for the EXTRACTION, and it was true of it. The migration that
followed is a different step with a different boundary, and leaving only the earlier paragraph would
leave a reader with a sentence that reads as current and is not.

At this head `local_repo_wet_terminal` no longer owns a join. Its verdict, expectation, scheduled,
terminal and join types and its seven join refusals are DELETED; the module retains its roster, the
requirement it derives, the co-resident executor and the invocation wall, and it reaches
`wet_evidence_validate` for everything else. `floor_changed_witness` carries one route-neutral
standing over `WetEvidenceValidation`.

**All seven refusals remain emittable as typed causes**, six under their own names and
`LocalRepoWetTerminalForeignCandidate { identity, observed, required }` as
`WetBindingPreparedSubjectDiffers { identity, observed, required }` — the same three fields compared
by the same equality, because the local requirement sets `prepared_subject` to the candidate. None
became a detail string, which is the condition this plan set for the work to have succeeded rather
than reduced.

**One declared widening.** `wet_disposition_is_agreement` treats `KnownRedHeld` as agreement, which
the deleted single-armed expectation could not express. No roster row constructs it, so no landed
behaviour moves; it is recorded here because a widening discovered later reads as a defect.

**Two refusals had no asserting witness** when the migration reached them —
`WetTerminalForeignEntry` and `WetTerminalForeignFunction`, emittable by the shared validator and
covered by the local route's old acceptance matrix. A refusal an authority can construct and no test
discriminates can be deleted with nothing going red, so they are now asserted in
`v2.test.wet_evidence` with both cross-negatives.
