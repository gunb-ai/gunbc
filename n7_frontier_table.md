# N7 frontier, by stage (derived from runs, 2026-10-06)
Subject: //v2/test/parse/expression_bodied_fn_decl_parse:all, one test module with 8 tests, native route. All 8 tests share one verdict at every point, so they are one row. Closure files that refuse at ingest are listed separately.

| Subject | (a) main 0b41b35e491 (2026-09-30T18:51Z) | (b) main a0a5c62161b (2026-10-03T11:27Z) | (c) main a453be995a (2026-10-06, post-#13388) |
|---|---|---|---|
| the 8 tests (label) | RESOLVE: resolve_reason_unbound_symbol | PREPARE (infer): infer_reason_projection_receiver_declaration_unavailable | PREPARE (infer): same cause; identical chain on all 8 |
| file refusals (count) | 12 (not itemized) | 5 | 1 |
| body_lowering_fold.dag | not itemized | PARSE parse_g0_tokens_remain 11429:42 ('then', root E) | clear |
| service_realization.dag | not itemized | PARSE parse_g0_tokens_remain 398:22 (root E) | clear |
| machine_constraints.dag | not itemized | PARSE parse_g0_tokens_remain 55:23 (root B) | clear |
| uri_path.dag | not itemized | LOWER body_lowering_reason_return_not_in_tail_position (root F) | clear |
| filesystem_io.dag | not itemized | LOWER body_lowering_reason_service_realization_unreachable (root D) | LOWER same, at 768:1 (XL-2) |

Sources:
- (a) #12628 comment 2026-10-01T01:35Z, table row 'main alone (0b41b35e491) | unbound_symbol | 12' with prepare_ok=0. No run id was recorded.
- (b) tidy-raven-393, msg_67aa805d, a run pinned in-script to a0a5c62161b (rc=2, 901s). No invocation id was recorded.
- (c) this run: BuildBuddy invocation bbfcb6cd-afc0-42e4-a4af-cc46d7ac864f, explicit checkout CHECKED_OUT=a453be995afc4a79e1cbe2a803f754fca3b91079, GITHUB_SHA pinned, wall 831s. Raw output: scratchpad n7_frontier_now.txt.
- Between (b) and (c), also from runs: 3f5d1154a2 (10-04) 4 file refusals, msg_c76b24e1; 59c3845950 (10-05) 1 file refusal, msg_5736a33a. Both have the same PREPARE fatal.

## What now stands under the PREPARE wall (the leaves of the chain at (c); all 8 identical)
| Leaf cause | Where | Open PR that clears it |
|---|---|---|
| infer_match_scrutinee_type_underived, then receiver declaration unavailable (Outcome<ParseArtifact> call result) | 04_infer | #13325 (CLEAN): the body names this as N7's wall; N7 was NOT re-run on its head |
| resolve_reason_unbound_symbol 'count' (declared in several modules), 4x | v2.std.node / node_query | #13307 (CLEAN): binds bare count; no N7 run on its head |
| resolve_reason_unbound_symbol 'from_code_point' | v2 callers | #13378 (DIRTY: needs a main merge) |
| unbound 'EmptyPattern' (declared in several modules) | 01_tokenize / lexing | none |
| ambiguous 'Cardinality': v2.std.node vs std.constructors | node | none (the owed Cardinality unify) |
| ambiguous 'Optional': v2.std.grammar.Optional (the GrammarExpr arm) vs std.optional.Optional | grammar | none (the Optional de-fork, PR B / transition 2) |
| inhabitance_undecidable_generic_formal / formal_unresolved | infer (token_stream type variables) | none named; likely downstream of the rows above |
| filesystem_io file refusal | XL-2 service realization | none: #13430 (PR2b) is the lexer step and does not touch filesystem_io |

Not gating at (c): #13284 (kernel-String concat; not in the chain), filter/any (no open PR; not in the chain), #13408 (floor), #13461 (normalize performance).

## Verdict
- (a) to (b): ONE stage advanced. The label moved from RESOLVE to PREPARE (infer); file refusals went 12 -> 5.
- (b) to (c): ZERO stages advanced on the label (still PREPARE, same fatal). Ingest convergence is real: file refusals went 5 -> 1, and roots B, C, E and F are cleared.
- The PREPARE wall is not a single infer defect. Its chain holds at least 6 independent leaves, 5 of them resolve-level (unbound or ambiguous names in closure modules). Only 3 have an open PR (#13325, #13307, #13378), and none of those has been run against N7. So even if all three land, N7 stays at PREPARE until EmptyPattern, Cardinality and the grammar Optional are resolved.
- Caveat: (b) has no leaf-level chain on record, so leaf movement between (b) and (c) is unmeasured.
