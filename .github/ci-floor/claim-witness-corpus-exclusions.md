# Claim-witness corpus CI gate — curated exclusions

The gate at `scripts/v4-claim-witness-corpus-gate.sh` enrolls rows from
`src/v4/test/claim/workflow/claim_witness_corpus_ci_runner.dag` (substrate authority).
CI uses `--shard a|b --spot-perturb-check` (1–2 ExpectPass rows/run/shard, `GITHUB_RUN_NUMBER`
rotation); full `--perturb-check` is local-only (run both shards). Sign runner (2026-06-13):
monolith 20-row spot-perturb measured 1108s (~18.5m) — exceeds 13m gate; sharded ~10-row
jobs are the recorded recovery path (~10m/shard uncontended ×2 ≤ 20m ceiling/shard).
The following witness families are **not** enrolled (each with a one-line reason):

| Excluded family | Reason |
|-----------------|--------|
| Full `src/v4/test/claim/**` Bool witness scan (~524 `fn () -> Bool` rows) | Budget: sequential execution would exceed the ~20m lens-job ceiling at current per-witness latency |
| `v4_lens_gate` rows (`lens_ci_gate.dag`) | Already gated by `scripts/v4-lens-ci-gate.sh` with perturb-check |
| `v4_lens_ci` node-frontier rows (`affected_set_ci_runner.dag`) | Already gated by `scripts/v4-affected-set-node-frontier-gate.sh` with perturb-check |
| `probe_selector_ci_runner` keystone rows (`probe_selector_ci_runner.dag`) | Already gated by `scripts/v4-probe-selector-keystone-gate.sh` with perturb-check (v4_lens_ci) |
| Glob-discovery smoke roster (`v4-discover-owned-data.sh` transport) | Separate discovery-law transport; not manifest-pinned expectations |
| `probe_selector_cf_import` resolve repro (`dsl/test/claim/probe_selector_compute_fabric_import_repro.dag`) | 🟡 P-PROBE-CF-IMPORT; enrolled ExpectFail in corpus shard b (adhoc-20b17ff7-932); under dsl/ so M1 emit skips it |
| T-38 manual TestClaim corpus eval (`manual_corpus_eval_expected.dag`) | TestClaim-run_fn transport; separate T-38 lane (`#4765`) |
| Executor batch-runner dogfood (`batch_runner.dag` / `claim_executor`) | Already gated by `scripts/v4-batch-executor-gate.sh` |

Adding a witness to the gated set is a **one-row `.dag` data binding** (Cost-of-Change).
