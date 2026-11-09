# Keen-Otter Archive Triage

**Date:** 2026-11-09  
**Author:** silent-badger-161 (read-only triage for silent-bee-845)  
**Refs examined:** 122 remote refs → 109 unique after deduplication (8 zb/*≡zesty-badger-599/* pairs) minus 5 excluded per brief (equality wall + #16013)

## Exclusions applied

| Excluded | Reason |
|---|---|
| pr-15992, pr-15924, pr-16092, stopgap-3site | equality wall (separate child) |
| pr-16013 | srv1 user-work slice (silent-bee-845 owns) |
| zb/* aliases (8) | duplicate hashes with zesty-badger-599/* |

## Classification counts

| Class | Count |
|---|---|
| RESTORE | 22 |
| RESTORE-AFTER | 4 |
| REWORK | 17 |
| SUPERSEDED | 6 |
| DROP | 60 |
| **Total** | **109** |

---

## RESTORE (22) — finished, still wanted, small integration

| # | Ref | PR | Title | Files | Merge | Notes |
|---|---|---|---|---|---|---|
| 1 | pr-15695 | #15695 | grammar: an else-less if as a statement keeps its condition's position | 4 | CLEAN | Unblocks #15876. Foundational grammar fix. |
| 2 | zesty-badger-599/infer-dead-shape-reason | — | Delete dead infer_branch_shape_invalid_diagnostic; call sites route to infer_malformed_tree_diagnostic | 1 | CLEAN | Dead code removal. 5+/13-. |
| 3 | zesty-badger-599/lambda-contextual-binder | — | Lambda binder reads contextual value name grammar | 2 | CLEAN | Fixes `module => module` parse. 77+/7-. |
| 4 | pr-15567 | #15567 | duct_card_dims: refuse on an unread slot count only for shapes that read the slot pitch | 3 | CLEAN | Single commit, small fix. |
| 5 | pr-15614 | #15614 | std.algebra: one prefix and one suffix predicate over FreeMonoid, both in terms of split_at | 12 | CLEAN | +41/−137. Consolidates duplicate predicates. |
| 6 | pr-15687 | #15687 | compiler: a nested nullary coproduct arm's facts come from the projection's facts-aware helper, not from the enclosing shape | 7 | CLEAN | +352/−37. Fixes arm typing regression. |
| 7 | pr-15648 | #15648 | compiler: fix the rc-ref pair rule so a renamed field binding captures its own name, not the outer one | 4 | CLEAN | +198/−7. Emitter fidelity fix + instrument. |
| 8 | pr-15363 | #15363 | Native body lowering: a `let` in a block lowers as a Bind with the statement-position judgment | 22 | CLEAN | +625/−12. Adds let-statement lowering. |
| 9 | pr-15583 | #15583 | extdeps.llm.openai: model the OpenAI provider (chat + text-completion) | 2 | CLEAN | +336. 17-commit modelled wire types. |
| 10 | pr-16042 | #16042 | std.host_text: one substring home | 7 | CLEAN | +132/−55. Consolidates substring implementations. |
| 11 | pr-16090 | #16090 | v2 lowering: nominal type-application child edges | 1 | CLEAN | +387. Test-claim for child-edge contract. |
| 12 | pr-16038 | #16038 | Remote-build PR 3: durable delegation and fenced attempt controller | 6 | CLEAN | +677. CAS-backed attempt controller. |
| 13 | pr-16020 | #16020 | Harness shell: bounded background execution, one lifecycle owner | 24 | CLEAN | +982/−14. Background process management. |
| 14 | pr-15868 | #16088 | v2 inhabitance: a refusal names its decision, never a borrowed cause | 80 | CLEAN | +2182/−270. Large but clean. |
| 15 | pr-15560 | #15560 | Remove behavioral-receipt-{plan,census,selftest} instruments and host | 34 | CLEAN | −3655. Implements DESIGN 2026-10-30 retire-census ruling. |
| 16 | pr-15551 | #15551 | Retire ambient_name_home kit_probe residue left by #15428 | 1 | CLEAN | +5/−78. Completes census retirement. |
| 17 | pr-15892 | #15892 | roster_import_sweep: two controls that keep the ambient import rule honest | 4 | CLEAN | +105/−11. Import sweep controls. |
| 18 | pr-15534 | #15534 | std.text: the canonical case fold, and the casing predicates on it | 2 | CLEAN | +254. Case-folding implementation (was draft, content is complete). |
| 19 | pr-15613 | #15613 | claim arms: fix the two main-red arms (unbound emit reads the pair rule; unused-binder reads the v1 catch-all) | 3 | CLEAN | +9/−10. Tiny fix (was draft, content is complete). |
| 20 | pr-15609 | #15609 | std.measure: kind Measure<Q: Quantity, S: Scale, M> and pass-through binder kinding | 5 | CLEAN | +197/−22. Kinding pass. (gh title misleading; commits are real compiler work.) |
| 21 | pr-15607 | #15607 | decl_facts_at keyed read: the files its asked keys derive, not the corpus | 10 | CLEAN | +239/−54. Compiler index fix. (gh title misleading.) |
| 22 | pr-15710 | #15710 | Credential-path refusals carry no body; --record refuses credential-bearing operations | 6 | CLEAN | +286/−100. Security: credential redaction in harness. |

---

## RESTORE-AFTER (4) — depends on another archived ref

| # | Ref | PR | Title | Depends on | Files | Merge | Notes |
|---|---|---|---|---|---|---|---|
| 1 | pr-15876 (≡ pr-16094) | #15876 | native lowering: an else-less if as a statement binds with no binder | pr-15695 | 52 | CLEAN | +1180/−385. Restores 35 v1 sites. |
| 2 | pr-15894 | #15894 | Infer reader: a service-operation call reaching the effect_demand is unrealizable — cause, not red herring | pr-15699 | 13 | CLEAN | +318/−115. Compiler diagnostics. |
| 3 | pr-15699 | #15699 | compiler: effect operation declaration accepts a body form | pr-15695 | 5 | CONFLICT | +89/−83. Prerequisite for #15894. |
| 4 | pr-15743 | #15743 | std.algebra: list_append, list_flatten and list_last in terms of the FreeMonoid fold | pr-15621 | 22 | CONFLICT | +750/−150. Needs rebase after pr-15621. |

---

## REWORK (17) — wanted but needs material work

| # | Ref | PR | Title | Files | Merge | What needs work |
|---|---|---|---|---|---|---|
| 1 | pr-15472 | #15472 | v1: the compiler reaches its own seed; its output is what compiles | 60 | CONFLICT | WIP/draft. Self-hosting milestone but far from complete. |
| 2 | pr-15380 | #15380 | Binder codemod: catch-all widening refusal carries its nullary remainder and the arm's binder | 11 | CONFLICT | Draft. Partially superseded by #15488; needs rebase + scope reduction. |
| 3 | pr-15533 | #15533 | std.collections: a Set over FreeMonoid, with the element operations | 13 | CONFLICT | Draft, depends on #15447 (census DROP). Needs rebase without census dep. |
| 4 | pr-15491 | #15491 | toolchain profile: the host-platform toolchain triple | 7 | CLEAN | Draft but small (+272). Needs review sign-off. |
| 5 | pr-15639 | #15639 | toolchain profile + claim: the v1 native compile route, toolchain-resolved | 4 | CONFLICT | Draft. Needs rebase. |
| 6 | pr-15644 | #15644 | CI: the merge-queue red pane, and the cause that keeps the lane open | 6 | CONFLICT | WIP/draft. Needs rebase. |
| 7 | pr-15671 | #15671 | construct tag: the expected type carries the tag on the construction route | 5 | CLEAN | Draft. Close to done (+308). Needs review. |
| 8 | pr-15703 | #15703 | v1 infer: the fold step's result closes the carrier | 3 | CONFLICT | Draft. Related to pr-15993. Needs rebase + dedup. |
| 9 | pr-15993 | #15993 | v1 infer: an open fold carrier closes at the step's produced type | 2 | CLEAN | Draft. Close to done (+178). Needs review. |
| 10 | pr-15935 | #15935 | roadmap: bind PR roadmap provenance | 3 | CLEAN | Draft (+327). Feature wanted; needs review. |
| 11 | pr-15917 | #15917 | seed: stage0 roster stops being hand-maintained | 2 | CONFLICT | Needs rebase. Single commit. |
| 12 | pr-15929 (≡ pr-16091) | #15929 | CI: heal-publisher federates its token instead of borrowing the workflow token | 8 | CONFLICT | Needs rebase. CI infrastructure. |
| 13 | pr-15462 | #15462 | flex autoscaler item 6: home-host carry (gh title misleading) | 13 | CONFLICT | Multiple review rounds; needs rebase. |
| 14 | pr-15530 | #15530 | flex autoscaler item 6 (same content as pr-15462, different ref) | 13 | CONFLICT | Dedup with pr-15462. Pick one. |
| 15 | pr-15626 | #15626 | citation witnesses + design ruling provenance (gh title: flex item 5, misleading) | 10 | CONFLICT | Needs rebase. Mixed concerns. |
| 16 | pr-15723 | #15723 | symbol_index_fill: a self.spelled record field resolves to the record's shape | 3 | CONFLICT | Needs rebase. |
| 17 | pr-14658, pr-14706 | #14658, #14706 | Native emit: generic record and coproduct members; opaque type members | 8, 4 | NO_MERGE_BASE | Very old. Content may be useful but needs full reimplementation. |

---

## SUPERSEDED (6) — already in main; cite the commit

| # | Ref | PR | Superseded by | Evidence |
|---|---|---|---|---|
| 1 | zesty-badger-599/browse-dup-bearer | — | `16c038e7a93` #15737 | Same single-line fix already merged. |
| 2 | zesty-badger-599/coercion-freemonoid-respell | — | `3a265308606` #15672 | Identical respell already merged. |
| 3 | zesty-badger-599/fallout-15247-imports | — | `cca1762ed36` #16063 + others | All three import fixes present in main. |
| 4 | pr-14925 | #14925 | main (SpawnLaunchPending present in source) | Arms already exist. |
| 5 | session-sleek-owl-397 | — | main | 0 files changed; V2NativeCensus already retired. |
| 6 | session-keen-otter-392 | — | main | 0 files changed; srv15 recovery already landed. |

---

## DROP (60) — with reason

### Census/audit instruments (DESIGN 2026-10-30 retire-census ruling)

| Ref | PR | Reason |
|---|---|---|
| pr-15447 | #15447 | Import-only rows in instrument census. |
| pr-15487 | #15487 | Same content as #15447. |
| pr-15994 | #15994 | "DO NOT MERGE: integration evidence for #15993". Measurement. |
| pr-16061 | #16061 | "DO NOT MERGE (measurement)". |

### srv1 execution packets / launchers / classifier fixtures / grant ceremony

| Ref | PR | Reason |
|---|---|---|
| pr-15965 | #15965 | R4/R5/R6 acceptance packets — execution packets. |
| pr-16046 | #16046 | NOT ACCEPTED draft, roadmap contract binding — grant ceremony. |
| pr-16053 | #16053 | Model selected-base bootstrap package coherence — classifier fixture. |
| pr-16056 | #16056 | Model same-candidate shell membership admission — classifier fixture. |
| session-keen-heron-809 | — | WIP model selected-base bootstrap — classifier fixture, CONFLICT. |

### Measurement / RFM rows / draft with no landed value

| Ref | PR | Reason |
|---|---|---|
| pr-15888 | #15888 | RFM seed accepts ExprError node. Draft, test-only. |
| pr-15895 | #15895 | RFM row: effect_demand cannot see a service-operation call. Test-only. |
| pr-16002 | #16002 | rfm: add-arrow positive cells red on main (cause unknown). Draft, measurement. |

### Preservation-only / falsifier / no functional content

| Ref | PR | Reason |
|---|---|---|
| jolly-cat-271/pp7-np7-artifacts | — | "archive: no functional edits". |
| keen-koi-376-falsifier-caseB | — | Falsifier case B, local object only. |
| keen-koi-376-falsifier-control | — | Falsifier control, local object only. |
| keen-koi-376-v6-commit | — | Prepared push candidate; never pushed. |
| sleek-seal-226/packet-freeze2-b1 | — | PRESERVATION (wind-down), no functional edits. |
| wise-moth-707/n7-rev6-draft | — | Unposted, preservation only. NO_MERGE_BASE. |

### Already in main (0 files changed)

| Ref | Reason |
|---|---|
| session-vivid-newt-169 | 0 files; already merged. |

### No merge base / too old / orphaned

| Ref | PR | Reason |
|---|---|---|
| pr-14658 | #14658 | NO_MERGE_BASE. Too old (Jul 2026). REWORK at best. |
| pr-14706 | #14706 | NO_MERGE_BASE. Too old. REWORK at best. |
| pr-14925 | #14925 | NO_MERGE_BASE. Superseded. |
| pr-15149 | #15149 | NO_MERGE_BASE. Old; content not in main but branch unusable. |
| session-tidy-ant-251 | — | NO_MERGE_BASE. Old site layout. REWORK at best. |

### Duplicates (same content as another archived ref, already counted)

| Ref | Duplicate of |
|---|---|
| session-keen-koi-376 | ≡ pr-15997 |
| session-keen-moth-672 | ≡ pr-15993 |
| session-keen-raven-843 | ≡ pr-16046 |
| session-proud-badger-77 | ≡ pr-16090 |
| session-swift-carp-118 | ≡ pr-15551 |
| session-zesty-badger-599 | ≡ pr-15876 |
| session-zesty-dove-98 | ≡ pr-15929 |
| pr-16093 | ≡ pr-15997 |
| pr-16094 | ≡ pr-15876 |
| pr-16097 | ≡ pr-16046 |
| pr-15932-final | Same PR as pr-15932, later version. pr-15932-final is the canonical. |

### Remaining DROPS (non-compiler infrastructure, old, or not wanted)

| Ref | PR | Reason |
|---|---|---|
| pr-15344 | #15344 | Strata request-bound save with vendored patch. Infrastructure-specific; needs full rebase. |
| pr-15447 (dup) | — | Already dropped as census. |
| pr-15462 (dup) | — | Already in REWORK. |
| pr-15477 | #15477 | flex autoscaler item 3: capacity floor from the observed fleet baseline. CONFLICT. Infra; needs rebase. |
| pr-15508 | #15508 | v1 infer: the fold result type closes the carrier (+105). CONFLICT. Duplicate of pr-15703 concept. |
| pr-15530 | #15530 | Duplicate of pr-15462. |
| pr-15551 (via session-swift-carp-118) | — | Already counted in RESTORE. |
| pr-15607 (via gh title "flex item 4") | #15607 | Already counted in RESTORE. |
| pr-15634 | #15634 | host_text: string_length reads the buffer's own byte count. CONFLICT. Superseded by #16040 (tokenize string_length) + main has it. |
| pr-15621 | #15621 | roster_import_sweep: append row moves ambient list join onto list_append. CONFLICT. Dependency for pr-15743 but needs rebase. |
| pr-15699 (duplicate entry) | — | Already in RESTORE-AFTER. |
| pr-15929 (duplicate entry) | — | Already in REWORK. |
| pr-15997 | #15997 | roadmap: the PR body is the roadmap row. CONFLICT. 11 files, +503/−153. Grant/roadmap ceremony. |
| pr-15931 | #15931 | compiler: let_binder_frame — one relation for let and match-arm binders. +30. CONFLICT. Very small; #15488 (merged) already covers the concept. |
| pr-15892 (dup) | — | Already in RESTORE. |
| pr-15932 | #15932 | roadmap_authority: the branch's own commit owns the roadmap row. Superseded by later version (pr-15932-final). |
| session-royal-dove-162 | — | v1 infer: fold carrier closes at produced type. CONFLICT. Duplicate of pr-15993. |

---

## Suggested RESTORE order (top 10 by unblocking power)

| Priority | Ref | Why first |
|---|---|---|
| 1 | **pr-15695** (grammar else-less if) | Unblocks pr-15876 which restores 35 v1 sites; CLEAN, small. |
| 2 | **pr-15876** (lowering else-less if) | Largest user-facing win: removes `statement_precedes_without_binding` refusals across 35 sites. CLEAN, stacks on #15695. |
| 3 | **pr-15560** (retire behavioral-receipt) | −3655 lines of census instrumentation; implements the 2026-10-30 DESIGN ruling. CLEAN. |
| 4 | **pr-15868** (inhabitance refusal) | CLEAN despite 80 files. Large compiler correctness improvement. |
| 5 | **pr-15648** (rc-ref pattern) | Fixes emitter fidelity for renamed field bindings; adds instrument. CLEAN, 4 files. |
| 6 | **pr-15687** (nested nullary coproduct) | CLEAN, 7 files. Fixes arm type regression. |
| 7 | **pr-15363** (native body lowering let) | CLEAN, 22 files. Adds let-statement lowering — needed for #15931. |
| 8 | **pr-15614** (std.algebra prefix/suffix) | CLEAN, 12 files, net −96. Consolidates duplicate code. |
| 9 | **pr-16020** (harness shell background) | CLEAN, 24 files. Unblocks harness use cases for background jobs. |
| 10 | **pr-16038** (remote-build PR3) | CLEAN, 6 files. CAS-backed durable attempt controller. |

**Follow-up batch after top 10:** pr-15695→pr-15876→pr-15699→pr-15894 (chain), pr-15583, pr-16042, pr-16090, pr-15567, zesty-badger-599/infer-dead-shape-reason, zesty-badger-599/lambda-contextual-binder, pr-15892, pr-15551.

---

## Methodology notes

- **Merge-base issues:** 7 refs have NO_MERGE_BASE (very old branches or rewritten history). Their diff stats are unreliable.
- **gh title drift:** Some PR titles captured by `gh pr view` don't match the archived ref's actual content (branches were reused). Classification uses the actual commit history.
- **Duplicate refs:** `zb/*` ≡ `zesty-badger-599/*` (8 pairs, identical hashes). `session-*` refs duplicate some `pr-*` refs (7 matches). Counted once.
- **Draft status:** isDraft=true from GitHub does not mean the content is incomplete. pr-15613 and pr-15534 are marked draft but their diffs are complete and small → classified RESTORE rather than REWORK.
- **merge-tree results:** `CLEAN` means `git merge-tree --write-tree origin/main <ref>` exited 0. `CONFLICT` means it detected conflicts.
- **Cite scope honesty:** git grep "not found" results are noted but treated as evidence-of-absence only within the grep's scope (specific patterns on origin/main source files, not handoff/json metadata).
