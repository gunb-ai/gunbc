# v2 closed-PR recovery — census of closed-but-unfolded PRs after the v1 closeout

**Status:** typed planning carrier for the v1-closeout census of closed-but-unfolded PRs (gunbc#13641 landed on main at fa44b981025, 2026-10-10). Authority is `gunbc.plans.v2_closed_pr_recovery`; this page is its generated projection. Linked from ROADMAP parked context. Do not treat a row as an operator revival decision.

**Provenance.** Session neat-wolf-604 on 2026-10-10 judged closed-but-unfolded PRs by gh pr diff --name-only (paths, not line-by-line). DESIGN section 4d: this is that session's inference, typed TranscribedUncited, not a deduced fact. The operator has not yet decided which items to revive; changing a disposition is a row edit.

Every closed branch was kept. Re-derive on current main rather than rebase the old heads: each was red or stale for a reason. Candidate later work (still a row edit, not this carrier's job): N7 (#13558 and #13623), XL-2 string templates, and a small bundle of #13426, #13391 and #13642.

## Census rows

| PRs | Subject | Area | Why closed | Disposition |
| --- | --- | --- | --- | --- |
| #13558, #13623 | N7: payload-binder facts, and typing lambda arguments that have no declared type (src/v2/compiler/03_resolve, 04_infer) | V2Compiler | Built on a base branch other than main; never ran CI | ReviveOnMain — Rebase onto main first. |
| #13430, #13638, #13284 | XL-2: string templates in v2 parse/resolve; kernel String concatenation through a free monoid | V2Compiler | Red, lane cancelled | ReviveOnMain — Open a fresh lane; this is the text crossing the 2026-09-27 ruling commits to, not a rebase of the old WIP. |
| #13426 | Foreign lexers: Rust ingest of a bare +, TypeScript single-quoted strings | V2Compiler | Never ran CI | ReviveOnMain — Re-run the small foreign-lexer patch on current main. |
| #13391 | Refuse a product value where a kernel-scalar type is declared | V2Compiler | Review asked for a whole-corpus census | ReviveOnMain — Take the whole-corpus census the review asked for; the finding is a real floor-class gap. |
| #13642 | Census: a brace in a quoted record key is not a field | V2Compiler | Red | ReviveOnMain — Re-run the census on current main. |
| #13377 | Qualify Cardinality/Optional in v2 lenses | V2Compiler | Made an ambiguity worse | ReDeriveInV2 — carries: The qualification obligation; the patch itself was a symptom fix. First step: Redo from the root rather than the closed symptom patch. |
| #13399 | Move the bottom seam to std.error_primitives | V2Compiler | Red and conflicting | ReviveOnMain — Fresh rebase; the closed head is 31 files. |
| #13390 | RFC 3986 percent-encoding split (from the Unicode scalar from_code_point -> percent-encoding -> git fixture chain) | V2Compiler | Stale against #13453 | ReviveOnMain — Re-derive the percent-coding split on its own; it stands without the rest of the chain. |
| #13378, #13392 | Unicode scalar from_code_point with typed refusal, and the git fixture that followed the percent-encoding split | V2Compiler | Stale against #13453 | ReDeriveInV2 — carries: Typed refusal at from_code_point, and a git fixture that consumes percent-coding. First step: Re-derive against current main rather than rebase the stale chain. |
| #13630 | Recover a callee's arrow type through an instantiation refusal | V2Compiler | Two reviews found a fail-open | Dead — Rejected on design grounds. |
| #13265 | Type parameters bind only inside their own declaration | SharedStdOrV1Written | Rejected because it would bind Result.ok to an unrelated ok and produce a fabricated type | ReDeriveInV2 — carries: Rule A' still applies in v2. First step: Re-derive the binding rule in v2's infer stage. |
| #13330, #13488 | Derived-node identity and nested-optionality design | SharedStdOrV1Written | Mostly src/v1/04_infer and 00_core plus the Rust mirrors | ReDeriveInV2 — carries: The ideas; the v1 code does not. First step: Re-derive the identity and nested-optionality facts in v2 rather than porting the v1 patch. |
| #13475 | An empty [] stops locking the accumulator to List<Unit> | SharedStdOrV1Written | The fix is in v1 | ReDeriveInV2 — carries: The empty-list accumulator bug, if v2 infer still has it. First step: Check whether v2 infer has the same bug, then fix there if it does. |
| #13541 | Retire v2.std.algebra filter/any | SharedStdOrV1Written | The diff now comes back empty | ReDeriveInV2 — carries: The retirement intent, not the closed patch. First step: Redo the retirement from scratch against current main. |
| #13614 | A seed-interpreter fix | SharedStdOrV1Written | Seed-interpreter work | Dead — The interpreter is no longer part of what the seed emits. |
| #13217, #13339, #13639 | Deployment-risk D2 | ProductFleetOps | Closed as red, stacked on the retiring floor, or with REQUEST_CHANGES | ProductPriorityCall — revival is a product-priority call, not a v1/v2 question. |
| #13097 | G1 belt cutover | ProductFleetOps | Closed as red, stacked on the retiring floor, or with REQUEST_CHANGES | ProductPriorityCall — revival is a product-priority call, not a v1/v2 question. |
| #13420 | Managed-host fan/KVM | ProductFleetOps | Closed as red, stacked on the retiring floor, or with REQUEST_CHANGES | ProductPriorityCall — revival is a product-priority call, not a v1/v2 question. |
| #13578 | Hosted OpenAI-compat classifier | ProductFleetOps | Closed as red, stacked on the retiring floor, or with REQUEST_CHANGES | ProductPriorityCall — revival is a product-priority call, not a v1/v2 question. |
| #13252 | The mtcollins1 runner offer | ProductFleetOps | Closed as red, stacked on the retiring floor, or with REQUEST_CHANGES | ProductPriorityCall — revival is a product-priority call, not a v1/v2 question. |
| #13382, #13383, #13411 | Accepting wildcards | ProductFleetOps | Blocked only on the floor's browser-toolchain premise, which no longer applies | ProductPriorityCall — revival is a product-priority call, not a v1/v2 question. |
| #13211, #13288, #13605, #13612, #13620, #13624, #13640, #13629, #13617 | Floor-runner and floor-control PRs, plus #13617 | DeadFloor | v1 floor work; the required floor lanes were withdrawn | Dead — Genuinely dead: v1 floor. |
| #12707, #12737, #12917, #13631, #13632, #13647, #13648, #13652, #13656 | Model-eval runs and the qwen/dusk lanes | DeadScratch | Scratch lanes | Dead — Genuinely dead: scratch. |
| #13125, #13608, #13628 | Duplicates or superseded: the argv, printer, BMC-boot and compile-door duplicates, plus the named PRs | DeadDuplicate | Duplicates or superseded | Dead — Genuinely dead: duplicates or superseded. |
| #13557, #13637 | Plans-only closed PRs | DeadPlansOnly | Plans-only | Dead — Genuinely dead: plans-only; nothing to recover. |

## How a later decision lands

- ReviveOnMain / ReDeriveInV2 / ProductPriorityCall / Dead are closed coproduct arms. The operator's later call is an edit of `disposition` on the named row, not new prose.
- This module does not open revival work items and does not re-judge the census.

## Dissolution trigger (DESIGN §6)

Delete this census when every ReviveOnMain, ReDeriveInV2 and ProductPriorityCall row has been row-edited to a terminal Dead or to landed revival work, so the closeout reading is no longer the reference for unrevived closed PRs.
