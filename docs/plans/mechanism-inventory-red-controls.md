# Mechanism inventory with red-controls (audit, 2026-07-01)

Census of every wired gate/lens/cache reachable from the CI floor plan (`src/v2/workflow/ci_floor_plan.dag`) and the commit workflow (`dsl/gunbc/commit_workflow.dag`): what each claims, its discriminating RED witness (the control that goes red when the gated behavior is wrong — DESIGN §5 spec-without-execution), and its last execution receipt. **Audit only — nothing here is a fix.** One pilot red-control ships with this audit (§6).

Receipts baseline: main tip `ef814b1f67` (2026-07-01). **Main CI is RED**: last green floor run `28486103773` (2026-07-01 00:59 UTC); the most recent completed run `28544311796` (20:02 UTC) fails batch 2 on `extdeps_external_authority_gate_passes` (false), `generated_artifact_drift_gate_passes` (false), and the discovery corpus itself (resolve error: `dsl/extdeps/tools/tools.dag:56:20: no field 'Which' on type 'shell'`, surfaced via `nbd_proxy_serve_transport_witness_test.dag`). Two readings, both true: the gates demonstrably fire on real breakage (not vacuous), and while main stays red the floor protects nothing incrementally — every PR CI run inherits the red, so a new regression is indistinguishable from the standing one.

## 1. Floor gates (10, `Gate` coproduct `dsl/gunbc/ci_gate.dag:3-13`, runnables `ci_floor_plan.dag:157-179`)

All gates reduce to `result.success` of a host effect (or `ProcessExit` → `exit_ok`); no `_ => true` catch-alls (the `_ =>` arms in `exit_ok` map non-success to `false`, fail-closed). Last execution receipt for every gate = the floor run above (batch 1 compile gate green; batch 2 receipts per run `28544311796`).

| Gate | Claims | RED witness | Verdict |
|---|---|---|---|
| RustMonolithGate | changed `.rs` pass fmt/clippy/nextest (`dsl/tools/rust_gates_ci.dag:42-68`) | real cargo failures only; no synthetic perturb. Change-gated skip-to-green when no rust paths changed (`:63-66`); fail-closed on git error (`rust_stage0_gates.dag:100-105`) | wired, red-by-consequence only |
| EmitHostGate | rust/py/go/ts/node smoke rows emit expected byte counts / HTTP 200 (`dsl/tools/emit_host_gate.dag:12-25`) | exact byte-count `test -eq` is the discriminator (`emit_host_transport.dag:34-57`); no planted-wrong-emit fixture | wired, exact-oracle |
| LayeringImportsGate | clean tree + scanner detects 5 planted violations (`dsl/tools/layering_imports_gate.dag:9-19`) | **yes** — 5 perturb `detects_holds` claims (`layering_imports_transport.dag:46-52`) | wired + red-control |
| ExtdepsExternalAuthorityGate | URI witnesses + live clean corpus + red fixtures (`dsl/tools/extdeps_external_authority_gate.dag:10-21`) | **yes, strongest** — `missing_anchor_red`, `bogus_scheme_red`, `file_anchor_red`, `anchor_drop_red` asserted RED (`:17-33,67-83`). Receipt: went red for real on main 2026-07-01 | wired + red-control (currently firing) |
| DslCompileCleanGate | whole tree compiles; perturbs go red (`dsl/tools/dsl_compile_clean_gate.dag:12-25`) | **yes** — planted skew/unresolved-import must FAIL compile (`dsl_compile_clean_transport.dag:70-128`); `red_empty_shell_program_fails_dcc_invocation_check` (`:256`) | wired + red-control |
| GeneratedArtifactDriftGate | committed artifacts == regen, and a perturbed copy drifts (`dsl/tools/generated_artifact_gate.dag:22-42`) | **yes** — `artifact_red_receipt_ok` (`:33-38`); empty-roster vacuity (`all([],…)⇒true`, `:51`) is closed by `generated_artifact_drift_test.dag:24` (`count > 0`). Receipt: red on main 2026-07-01 | wired + red-control (currently firing) |
| SourceRootIngestGate | real source-root ingest manifest + validate-red + closure ingest (`src/v2/workflow/source_root_ingest_gate.dag:7-9`) | **yes** — `..._validate_module_roots_red_on_parsed_roots` (`source_root_ingest_transport.dag:20,55`) | wired + red-control |
| RegenVerifyGate | committed stage0 == fresh regen (`dsl/tools/regen_verify_gate.dag:6-14`) | discrimination lives inside the host binary's `--verify` (`regen_verify_transport.dag:28`); no `.dag`-level negative control | wired, host-trusted red |
| SelfHostRealizedComparisonGate | both sides read real emitted bytes + staleness fixed-point (`self_host_realized_comparison_gate.dag:60-67`) | **WEAK** — subset roster `["lib.rs","v1_rt.rs"]` = 2 of ~89 files, self-labeled INTERIM (`gate:18-19`, `transport:17`); no planted-stale red fixture | wired, weak-red |
| EmitDeterminismGate | two sequential emits byte-identical (`dsl/tools/emit_determinism_gate.dag:6-15`) | `diff -r` is the discriminator (`emit_determinism_transport.dag:48`); no injected-nondeterminism control; theoretical vacuity if the corpus emitted nothing (diff of two empty dirs passes) | wired, self-diff only |

Corpus runnables: `floor_corpus_node` discovery over `witness_layer_roots = ["dsl","src/v2"]` + scan dirs (`dsl/gunbc/ci_layer_roots.dag:3-5`); `execution_corpus_node` scoped to `src/v2/test/claim/execution` with empty excludes; `grounding_whole_tree` single claim (`ci_floor_plan.dag:193-199`). **Enrollment is opt-out by path substring** (`witness_exclusion_substrings`, `ci_layer_roots.dag:7-19`): a broad substring (`test/manual/`, `test/claim/execution/`) silently un-enrolls any witness whose path contains it, with no red signal. This is exactly how the grounding lens's live whole-tree witness went dark (§3, Group D).

## 2. Commit workflow / merge path (outside the floor)

| Mechanism | Executes | RED witness | Notes |
|---|---|---|---|
| pre-push doc-reachability (`.githooks/pre-push:149-154`) | pre-push hook, glob-gated (`NEED_DOC`) | via `doc_reachability_witness_test.dag` (lens has verdict-flip red, §3) | skipped entirely when no doc-ish path pushed |
| pre-push generated-artifact drift (`:155-184`) | pre-push, glob-gated | same gate body as floor | **auto-commits regen inside a check** (`:180-183`); trigger probe swallows stderr (`:95`) — crash vs drift indistinguishable there |
| pre-push cargo fmt (`:185-201`) | pre-push, glob-gated | cargo's own check | stderr suppressed (`:187`); auto-fix+commit path |
| merge-admission **stamp** (`ci.yml:91`; `merge_admission_stamp.dag:70-99`) | every CI run, keyed to `CI_FLOOR_EXIT` | missing env ⇒ Failure conclusion (fail-closed, `:62-68`) | receipt written `.gunbc/merge-admission-receipt.wire` — visible in run `28544311796` log |
| merge-admission **gate** (`ci.yml:95-99`; `merge_admission_gate.dag:47-88`) | every CI run | receipt-missing/invalid paths can red (`:51-64`). **The staleness discrimination is switched OFF**: `merge_admission_blocks_merge` (`ci_failure_class.dag:173-184`) conjoins on `gating_is_enforced(merge_freshness_gating_status)` and `merge_freshness_gating_status = GatingComputedDeferred` (`:149`) ⇒ always admits | **the central coverage-by-illusion finding.** The pure predicate is fully red-witnessed (`merge_admission_witness_test.dag:62-100` — stale-base/stale-roster block, fresh admits, roster-hash sensitivity), so the model looks proven while the deployed wrapper is inert. Pilot red-control §6 pins this by execution. Honestly scaffold-marked (`ci_failure_class.dag:152-159`) with a named activation trigger |
| rust_tests job (`ci.yml:150-195`) | every CI run with `.rs` changes | real cargo failures; carries the cache-purity + poison-detection rust tests (§4) | receipt: `rust_tests` job SUCCESS on run `28544311796` |
| cgroup-peak measurement (`ci.yml:196-200`) | always | none — measurement only, never gates | by design |
| `branch_merge_admission_model.dag` (RepoStandard / rulesets drift gate / auto-reconcile) | **nowhere** | none | checkpoint-0 sketch by declaration (`:9`); `gunbc_repo_standard` witnessed only in a pure test, never reconciled against live GitHub config |

## 3. Lenses (`src/v2/lens/*`, floor-discovered via `discover_floor_corpus_rows`, `cli_run.rs:3181`)

The #5433 inert-lens backstop **does execute** in the floor (`dsl/test/claim/inert_lens_hygiene_witness_test.dag`, builtins `cli_run.rs:3150,3165`) but only checks module **import-reachability** from a discovered witness + non-empty universe. A lens that is imported and green-only (or whose live witness is exclusion-hidden) passes it — the gap its own plan (`dsl/gunbc/plans/inert_layer_lens.dag` §9) names.

**A — wired with a discriminating RED witness over the live corpus (9):** inert_carrier, wiring_liveness, doc_reachability, host_language_transport_script, extdeps_external_authority, extdeps_shape_transport_policy, fact_cardinality, languages_consumer_census, disposition_redundancy. (Each has both a live `*_facts_live` read and a red control; citations in the lens files' `corpus/`+`lens_unit/` test dirs.)

**B — wired, live-green-only (red is synthetic) (8):** layering_imports, realization_vocabulary_containment, medium_structure_containment, intent_linearity, module_graph, complexity_linearity_audit, non_fold_residue, no_dual_representation_test. A live pass runs green over the tree; the only red-going case is a planted fixture — a real corpus violation of a subtly different shape can slip.

**C — wired, synthetic-only (no live-corpus read) (~28):** visibility, complexity, cost, complexity_lowering, synthesis, unit_modeling, coverage, table_decision_tree, application(+serializer), identical_variant_payload, structural_resolution, structural_similarity, simulated_relationship, edit_locus/affected_set, wiring_liveness_preflight, mock_totality, grounding_ledger, registry, testgen, and the 8 family-eval lenses (effect, ownership, parallelism, idempotency, unused_parameters, subsumption, discrimination, leaf_model_verification). These execute in the floor but would never catch a real corpus violation.

**D — inert (3):** **grounding** (its live whole-tree witness `src/v2/test/manual/grounding_lens_whole_tree_test.dag` is exclusion-hidden by `test/manual/` — note it IS separately wired as the floor's `grounding_whole_tree_runnable` (`ci_floor_plan.dag:193-199`), so "inert" here means dark to *discovery*, not to the floor; the discovery-exclusion + explicit-runnable pairing is itself a dual-representation to keep coherent), **fact_density** (fires inside the compile-lens bundle but no discovered `test fn` gates its verdict), **affected_set_examples** (fixture carrier, no witness).

## 4. Caches

| Cache | Status in CI | Contract | RED control | Gap |
|---|---|---|---|---|
| sccache | LIVE, opportunistic (`ci.yml:33-39`; fallback ladders `:66-84` etc.) | sccache-internal hashing | `--verify-build-artifacts` (`claim_executor.rs:1107-1160`, `ci.yml:84,173,268`): exists + executable + non-empty, fail-closed | **no freshness assertion** — a stale-but-valid cached binary passes; ROADMAP §1 "kill sccache false-greens" self-marked *partly landed* |
| resolved_graph disk cache | OPT-IN (`GUNBC_RESOLVED_GRAPH_CACHE_DIR`, unset in ci.yml — inert on floor; #5789 always-on reverted) | key = closure content digest × compiler-exe digest (`resolved_graph_cache.rs:99-146`); payload digest verified on read | **strong** — `resolve_cross_process_cache_test.rs:176,213,294`: poisoned entry ⇒ `RejectedHit` ⇒ fresh recompute | none on the mechanism; liveness is the gap (by design, pending streaming IO) |
| warm==cold purity oracle (#5429) | LIVE as rust test (nextest gate, every `.rs` CI run) | hidden-input probe: key-invariant perturbation must not change output (`resolved_graph_cache.rs:499-566`) | **self-proving** — `cache_purity_oracle_test.rs:189` injected impurity ⇒ loud located `CachePurityViolation`; false-positive control `:126` | runs in the rust gate, not the .dag floor corpus |
| ParseTable memo | LIVE, in-process (`v1_interpreter.rs:652-666,2416-2494`) | key = (grammar digest, token digest, position, production) | key-derivation only; no dedicated detective | no cross-run surface, so exposure is low; still the §2 hand-rolled-cache recurrence |
| RecordedFixture | dev/regen tool only (claim_batch, regen_stage0); not in ci.yml | fail-closed taxonomy: Missing/Stale/InputMismatch/Expired/ResponseDrift (`recorded_fixture.rs:200-258`) | the taxonomy IS the detective | inert on floor |
| BuildBuddy CAS | INERT — modeled catalog only (`dsl/extdeps/cache/buildbuddy.dag`) | none | none | placeholder for §2 P2 `realize(subject)` |

## 5. Cross-cutting findings (ranked; not fixed here)

1. **Merge-admission staleness gate computed-but-inert** (`GatingComputedDeferred`, `ci_failure_class.dag:149`) while its pure predicate is fully red-witnessed — the model reads as proven, the deployment admits everything. Pinned by the pilot control (§6).
2. **Main-red erodes the whole floor**: with batch 2 red on main since 2026-07-01 ~01:00, every gate's per-PR discrimination is suspended (a PR cannot distinguish its own breakage from the standing one). The gate-hygiene ROADMAP item ("floor-enrolled gate must be green-on-main at merge") is exactly the missing wall.
3. **Discovery enrollment is opt-out by substring** (`ci_layer_roots.dag:7-19`) — a witness can be authored and never run with no red signal; the grounding lens's live witness is the realized instance.
4. **sccache freshness** — artifact-verify covers exists/exec/non-empty but not staleness.
5. **~28 floor-wired lenses are synthetic-only** (Group C): they execute, so they pass #5433, but cannot catch a real corpus violation; the backstop's reachability check does not see this tier.
6. **SelfHostRealizedComparisonGate** discriminates over 2 of ~89 files (honestly INTERIM-marked).
7. Pre-push checks are glob-gated skips with stderr suppression and repo-mutating auto-commit paths.

## 6. Pilot red-control (the one shipped with this audit)

`dsl/test/claim/merge_admission_deployed_gate_control_test.dag` — floor-discovered, pure `.dag`, no host effect. Two claims over the same stale-base scenario the existing witnesses use:

- `deployed_gate_pure_predicate_would_block_stale_base` — `merge_admission_would_block` = true (the model discriminates);
- `deployed_gate_admits_stale_base_while_gating_deferred` — the **deployed** wrapper `merge_admission_blocks_merge` returns false and does so exactly because `merge_freshness_verdict_is_consumed_to_block()` is false.

Together they make the C2↔B2 gap (fully-witnessed model, switched-off deployment) executable and loud: the day `merge_freshness_gating_status` flips to `GatingEnforced`, the second claim goes RED, forcing the flip to be acknowledged in the same diff — the inert state can no longer be silent. This is a control on the audit finding, not a fix of it (the flip itself stays behind its named activation trigger, `ci_failure_class.dag:151`).
