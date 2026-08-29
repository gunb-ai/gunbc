Two main-tree claims sat at ~80% of the floor's 500ms CPU ceiling (`v2.workflow.required_floor` `required_floor_claim_cpu_safety_limit_ms`) and went red on any runner ~20% slower — every open PR inherited the flake. Measured, not estimated: `required_floor_claim_cost.tsv` (artifact `required-floor-claim-cost`, column `cpu_ms`) from main runs 33251451113 (f1f9dd8a) and 33246969960 (b7265477):

| claim | 33251451113 | 33246969960 |
|---|---|---|
| `v2.test.languages_consumer_census.corpus.rust_language_external_consumer` `corpus_rust_language_has_external_consumer` | 412 | 412 |
| `v2.test.claim.build_artifact_verification_witness` `witness_serialized_conjuncts_and_teeth` | 403 | 395 |

They are two different defects and get two different remedies. Neither raises the ceiling, relocates, or de-enrolls a live check.

## 1. The census claim was a first-touch payer: pay the fill in preparation

`corpus_rust_language_has_external_consumer` is one `free_call` into `v1_compiler.cli_run` `languages_decl_records_cached` — a process-wide `OnceLock` over a token scan of every `.dag` and `.rs` file in the tree. Its sibling in the same file, `corpus_rust_language_external_consumer_count_positive`, reads the same memo milliseconds later and measures **0ms** in both runs; so does `rust_spec_external_consumer`. That is witness cost class 2 exactly as `v2.workflow.required_floor` describes it: the artifact is built once and billed to whoever asked first. The `OnceLock` miss is not bracketed by `record_shared_artifact_fill_cpu`, so `run_claim_measured` could not net it either. Reproduced off-CI (`claim_batch`, BuildBuddy amd64, two repetitions): 524/504ms for the first toucher, 0ms for the sibling.

The floor already owns the remedy: `FloorPreparationPhase` — "the shared builds a floor run pays before any per-claim timer starts … the list grows only when another such artifact is FOUND, never by declaration". This is the fourth such artifact found, so it is the fourth row:

- `v2.workflow.required_floor` `FloorPreparationPhase` gains `LanguagesConsumerCensusBuild`; the member-count prose beside the preparation CPU limit follows (four members, 60% of the job left for the claim fold).
- `required_floor_runner.rs` warms it by calling the declared producer once, under `observe_shared_build`, after the prepared authority is installed (so the warm takes the inventory path the claims take) and before any claim runs; it is pushed onto the same `shared_build_warms` collection and adjudicated by the same three limits as the other phases. Provenance is honest: `languages_decl_records_already_built` reports `already-warm-on-entry` if some earlier phase touched it. Skipped, printed, when the subject does not carry `std.languages` (the census panics on that absence by design).
- `dag/test/claim/floor/floor_preparation_shared_build_witness_test.dag` `w_refusal_carries_its_phase` — the exhaustive match over the phase carrier gains the arm.

One mechanism only. A second `record_shared_artifact_fill_cpu` bracket beside the warm was considered and not done — it is the one-rule-two-homes fork `gunbc.census_memo_seed_growth` refuses.

**Falsifier / before-after.** Before (no row) is main itself: the runs above, 412ms charged to the first toucher, 0ms to the sibling. After (this PR): the `required-floor-claim-cost` artifact of this PR's witnesses run, plus the `[floor-phase] phase=languages-consumer-census-warm` line in the witnesses job log carrying the build. The check that the fill landed in preparation and did not migrate: no other claim in the after-TSV becomes the new first-toucher.

**After, measured** — PR run 33262529978 (head 7baff11c, `required-floor-claim-cost` artifact, `executed=2828`), witnesses job log:

```
[floor-phase] phase=languages-consumer-census-warm state=completed cpu_ms=488 wall_ms=489 rss_growth_bytes=0 decl_rows=72 provenance=built-by-preparation
```

| claim | main 33251451113 | PR 33262529978 |
|---|---|---|
| `corpus_rust_language_has_external_consumer` | 412 | **0** |
| `corpus_rust_language_external_consumer_count_positive` (sibling) | 0 | 0 |
| `rust_spec_external_consumer.corpus_rust_spec_has_external_consumer` | 0 | 0 |
| `rust_statements_composition_only.corpus_rust_statements_is_composition_only` | 0 | 0 |

The fill did not migrate: joining the two TSVs at identity grain (2828 common identities; the only roster difference is the deleted `witness_serialized_conjuncts_and_teeth`), no claim's cpu doubled — zero rows satisfy `after ≥ 150ms and after > 2×main` — while the whole population reads median 1.07× (p10 1.00, p90 1.22) of main's figures, i.e. this PR's runner is uniformly slower, the same shape XL-1 measured. The 488ms the phase reports is the fill itself, now where `v2.workflow.required_floor` says it belongs, adjudicated against the preparation limits rather than a per-claim ceiling.

The revert arm is main: without the row the same claim is charged 412ms on two consecutive main runs.

(On that ~1.1× runner, 8 rows sit at ≥350ms against main's 2 — every one an emit-path row from the reservoir list below. Had the two claims this PR fixes still been enrolled they would have read ≈470 and ≈500. That is the flake, observed once more.)

## 2. The bash claim witnessed dead code: delete-first

`witness_serialized_conjuncts_and_teeth` serializes a two-statement verification script through `v2.workflow.build_step_emit` `verifications_script`. Decomposed with a scratch probe under `claim_batch` (BuildBuddy amd64, one dispatch, two repetitions; CI's arm64 runner is ~1.7× that): a single word leaf costs ~47ms, a `[ ! -x path ]` test 112-114ms, one `if` 174-178ms, the whole script 227-235ms — and a per-`.dag`-fn self-time table (the interpreter's `dag_fn_self_time_snapshot`, printed by a throwaway hook not in this PR) names the cost as the generic grammar matcher and `06_translate` serializer per leaf (`fold_node` 2148 calls, `formal_production_rhs_matches_emitted_*` ≈40ms self, `formal_production_unique_lhs_exact_match` 257 calls for a two-statement script). That is the #9670 class (a traversal that cannot name the node it already visited) — the emit-cost lane's, a load-bearing compiler stage, not a claim-grain cost-shape defect. A hypothesis that the extdeps-grain catalog (`bash_fold_formal_productions`) was rebuilt per leaf was REFUTED by the same table: it is called twice per claim (the interpreter's call memo serves the rest), 0.3ms self.

What changes the remedy is that the witnessed function has **no production consumer**. `tools.build_step_transport` `bst_typed_transport_doc` records the flip (#6457): "no bash is serialized or executed. The former verifications_script serialize_bash harness dissolved with the flip". `build_step_emit.dag` is a one-function module whose own authority note cites byte-identity against a v1 `serialize_bash` that PR-C (#6797) deleted. So the ~400ms row was a witness of dead code, and DESIGN §3 delete-first / §6 "a new artifact with no final consumer" apply: the module and its witness go together.

**Consumer census, identity grain, by the instrument rather than grep** — `tools.module_impact_query_front_door` `dependents_of_module` over `v2.lens.module_graph`'s union of import-derived and reference-derived edges (the direction-typed reading the operator ruled in on 2026-08-08), run as `gunbc run --source-root dag --source-root src/v2 --entry <scratch entry binding subject "v2.workflow.build_step_emit"> --function dependents_of_build_step_emit_report` at f1f9dd8a plus this branch:

1. **The deletion is the census** (DESIGN §3, replacement migrations: "in a fail-closed substrate the deletion is the census — every real dependent refuses loudly"). PR run 33262529978 compiled the whole witnesses-lane closure with the module gone and every claim executed green (`executed=2828`, `required-witnesses-build` and `required-witnesses-floor` both success). A second importer or bare-name consumer of `verifications_script` would have refused at resolve in preparation, before any claim ran.
2. `tools.module_impact_query_front_door` `dependents_of_module("v2.workflow.build_step_emit")` over the pre-deletion tree: run with a `gunbc` built from this branch (`cargo build --release --bin gunbc`, arm64 session container, 501s) on a worktree pinned at main f1f9dd8a plus a 14-line scratch entry binding the subject (not in this PR):

```
impact reading — dependents of v2.workflow.build_step_emit
  standing: PoolIndependentProjectionUnavailable — the observed pool may contain SPURIOUS as well as missing edges, so this reading is neither an upper nor a lower bound. Trigger: P2a pool-independent dependency projection (namespace lane).
  reached (2 module(s)):
    v2.workflow.build_step_emit   [src/v2/workflow/build_step_emit.dag]
    v2.test.claim.build_artifact_verification_witness   [src/v2/test/claim/build_artifact_verification_witness_test.dag]
  causes (1):
    undeclared-target  v2.test.manual.ownership_movable -> v1.compiler.ownership
  cycles (0):
```

   One dependent, the witness module — and inside it `verifications_script` was referenced only by `witness_serialized_conjuncts_and_teeth` (the other three rows import `bash_command_fold_serialize` / `ci_spec` directly). The standing row is the reading's own declared limitation (the namespace lane's pool-independent projection is not established), which is why instrument 1 — the compile with the module gone — is the one that decides; the two agree. The one `causes` row is an unrelated pre-existing undeclared target in `v2.test.manual`, reported because the front door reports every cause it observed.

**Typed disposition** (the carrier vocabulary is not on main yet; same shape here):

```
RetiredWithDeletedSubject {
  claim:    v2.test.claim.build_artifact_verification_witness . witness_serialized_conjuncts_and_teeth
  subject:  v2.workflow.build_step_emit . verifications_script   (module deleted whole)
  evidence: tools.build_step_transport . bst_typed_transport_doc — the #6457 typed-transport flip:
            "no bash is serialized or executed. The former verifications_script serialize_bash
            harness dissolved with the flip"
}
```

What stays: `witness_existence_check_no_mtime` (327/317ms; serializes one statement via `bash_fold_serialize_node` directly — still the no-`-newer` regression control on the Node construction); `tools.build_step` `emit_verifications` and its two consumers (`v2.test.claim.bash_program_fold_support`, `gunbc.live_deploy.srv1_residue_rehearsal`). The orphaned basis row for the deleted identity in `dag/gunbc/witness/witness_row_cost_basis.tsv` is removed; two comments that named `build_step_emit` as the serialization precedent (`gunbc.required_lanes_gate`, `gunbc.witness_floor_workflow`) now name the serializer that survives. Not done, per the brief: no statement-grain regroup (65% is not a fix).

## The reservoir (report only — not this PR's)

At ≥70% of ceiling (≥350ms) main holds exactly the two claims above, in both runs. At ≥60% (≥300ms), run 33251451113 holds 13 identities, every one an emit/serialize-path claim — the same #9670 class:

```
342  v2.test.claim.effect_plan_bash_materialize_test.effect_plan_bash_operation_and_input_perturbations_change_emitted_bytes
332  v2.test.emit.produced_decl_two_target.produced_decl_two_targets_render_own_order
327  v2.test.claim.build_artifact_verification_witness.witness_existence_check_no_mtime
314  v2.test.emit.produced_decl_two_target.produced_decl_module_folds_declarations_in_order
313  v2.test.emit.rust_call_emit.rust_call_fold_closure_swap_discriminates
312  v2.test.execution.emit_host_meet_join_equals_eval.emit_host_meet_equals_eval_holds            (wall 444)
311  v2.test.execution.rust_emit_host_call_equals_eval.rust_emit_host_call_equals_eval_holds       (wall 581)
308  v2.test.emit.rust_call_emit.rust_call_fold_closure_emit_holds
306  v2.test.emit.rust_produced_decl_emit.rust_produced_decl_name_discriminates
303  v2.test.lens_testgen.shadow_ci_receipt.lens_testgen_shadow_ci_run_receipt_holds
301  v2.test.execution.emit_host_complement_equals_eval.emit_host_complement_wrong_fixture_refuses_holds  (wall 432)
300  v2.test.execution.rust_emit_host_call_equals_eval.rust_emit_host_call_wrong_oracle_refuses_holds     (wall 560)
300  v2.test.emit.semantic_decl_routing_generic.semantic_generic_enum_routes_to_enum_emit
```

XL-1 (deep-badger-41) adds the sharpness of that reservoir, measured on the same instrument: PR #9669 (merge-base b726547 + 4 `.dag` modules, 289 declarations, no reference reachability into any heavy claim) inflated the whole heavy population by median 1.11-1.18× (runs 33254674927 attempt 1 and rerun vs 33246969960), against a runner-to-runner control of median 1.02 / p90 1.06 (33246969960 vs 33251451113). Under that inflation `witness_serialized_conjuncts_and_teeth` was censored at 502ms on both attempts and the census claim read 517/462. Mechanism unnamed by them; not this PR's to name — recorded so the reservoir list above is read as "one small PR away from red", not as headroom.

Producer for every number above: `required_floor_claim_cost.tsv` from the named run, or the `claim_batch` dispatch quoted (`ctrl-build --remote -- bash -lc 'cargo build --release --bin claim_batch && ./target/release/claim_batch --source-root dag --source-root src/v2 --entry <file> --functions <csv>'`, `[witness] … cpu=` lines), at the revision named.

No `src/v1` seed growth beyond the runner's fourth warm and two small census helpers (`languages_decl_records_already_built`, `languages_census_subject_carries_authority`); no `.dag`→Rust regen is implicated (no emitted mirror covers these files).
