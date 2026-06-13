# Claim-witness corpus CI gate — curated exclusions

The gate at `scripts/v4-claim-witness-corpus-gate.sh` enrolls rows from
`src/v4/test/claim/workflow/claim_witness_corpus_ci_runner.dag` (substrate authority).
CI uses `--spot-perturb-check` (1–2 ExpectPass rows/run, `GITHUB_RUN_NUMBER` rotation);
full `--perturb-check` is local-only. Uncontended ~8.7m (7.7m base + ~1m spot).
The following witness families are **not** enrolled (each with a one-line reason):

| Excluded family | Reason |
|-----------------|--------|
| Full `src/v4/test/claim/**` Bool witness scan (~524 `fn () -> Bool` rows) | Budget: sequential execution would exceed the ~20m lens-job ceiling at current per-witness latency |
| `v4_lens_gate` rows (`lens_ci_gate.dag`) | Already gated by `scripts/v4-lens-ci-gate.sh` with perturb-check |
| `v4_lens_ci` node-frontier rows (`affected_set_ci_runner.dag`) | Already gated by `scripts/v4-affected-set-node-frontier-gate.sh` with perturb-check |
| Glob-discovery smoke roster (`v4-discover-owned-data.sh` transport) | Separate discovery-law transport; not manifest-pinned expectations |
| T-38 manual TestClaim corpus eval (`manual_corpus_eval_expected.dag`) | TestClaim-run_fn transport; separate T-38 lane (`#4765`) |
| Executor batch-runner dogfood (`batch_runner.dag` / `claim_executor`) | Already gated by `scripts/v4-batch-executor-gate.sh` |

Adding a witness to the gated set is a **one-row `.dag` data binding** (Cost-of-Change).
